#![feature(plugin)]
#![plugin(rocket_codegen)]
#![allow(unknown_lints)]
#![allow(needless_pass_by_value)] // Because of Rocket passing things like State<T> as values

#[macro_use]
extern crate bson;
extern crate chrono;
extern crate clap;
extern crate csv;
extern crate mongodb;
extern crate nalgebra as na;
extern crate rand;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate uuid;

mod cfg;
mod routes;
mod db;
mod model;
mod server;
mod stats;
mod serde_enum;

use bson::{from_bson, to_bson, Bson};
use clap::{App, Arg, SubCommand};
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::FindOptions;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use cfg::Config;
use model::{Metric, Weighting};
use db::User;

fn main() {
    let matches = App::new("lsys-pairwise")
        .version("0.1")
        .author("Magnus Bjerke Vik <mbvett@gmail.com>")
        .about("Pairwise comparison of LSystems")
        .subcommand(SubCommand::with_name("server").about("Run server"))
        .subcommand(
            SubCommand::with_name("stats")
                .about("Calculate statistics from data")
                .arg(
                    Arg::with_name("task")
                        .long("task")
                        .takes_value(true)
                        .required(true)
                        .help("Task to get stats for"),
                )
                .arg(
                    Arg::with_name("token")
                        .long("token")
                        .takes_value(true)
                        .required(true)
                        .help("User token to see stats for"),
                )
                .arg(
                    Arg::with_name("metric")
                        .long("metric")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["realistic", "pleasing"])
                        .help("Type of metric to see stats for"),
                ),
        )
        .subcommand(
            SubCommand::with_name("save-weights")
                .about("Save weights to file")
                .arg(
                    Arg::with_name("task")
                        .long("task")
                        .takes_value(true)
                        .required(true)
                        .help("Task to save weights for"),
                )
                .arg(
                    Arg::with_name("metric")
                        .long("metric")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["realistic", "pleasing"])
                        .help("Type of metric to save weights for"),
                ),
        )
        .subcommand(
            SubCommand::with_name("save-criteria-weights")
                .about("Save weights to file")
                .arg(
                    Arg::with_name("task")
                        .long("task")
                        .takes_value(true)
                        .required(true)
                        .help("Task to save weights for"),
                )
                .arg(
                    Arg::with_name("metric")
                        .long("metric")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["realistic", "pleasing"])
                        .help("Type of metric to save weights for"),
                ),
        )
        .subcommand(
            SubCommand::with_name("save-users")
                .about("Save users to file")
                .arg(
                    Arg::with_name("task")
                        .long("task")
                        .takes_value(true)
                        .required(true)
                        .help("Task to save users for"),
                )
                .arg(
                    Arg::with_name("metric")
                        .long("metric")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["realistic", "pleasing"])
                        .help("Type of metric to save users for"),
                ),
        )
        .subcommand(
            SubCommand::with_name("save-questionnaires")
                .about("Save questionnaires to file")
                .arg(
                    Arg::with_name("task")
                        .long("task")
                        .takes_value(true)
                        .required(true)
                        .help("Task to save for"),
                )
                .arg(
                    Arg::with_name("metric")
                        .long("metric")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["realistic", "pleasing"])
                        .help("Type of metric to save for"),
                ),
        )
        .get_matches();

    if matches.subcommand_matches("server").is_some() {
        server::run();
    } else if let Some(matches) = matches.subcommand_matches("stats") {
        let task = matches.value_of("task").unwrap();
        let token = matches.value_of("token").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        if let Err(err) = stats::print_stats(task, token, &metric, &cfg.db) {
            println!("Failed printing stats: {}", err);
        }
    } else if let Some(matches) = matches.subcommand_matches("save-weights") {
        let task = matches.value_of("task").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        save_weights(task, &metric, &cfg.db);
    } else if let Some(matches) = matches.subcommand_matches("save-criteria-weights") {
        let task = matches.value_of("task").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        save_criteria_weights(task, &metric, &cfg.db);
    } else if let Some(matches) = matches.subcommand_matches("save-users") {
        let task = matches.value_of("task").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        save_users(task, &metric, &cfg.db);
    } else if let Some(matches) = matches.subcommand_matches("save-questionnaires") {
        let task = matches.value_of("task").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        save_questionnaires(task, &metric, &cfg.db);
    } else {
        println!("No subcommand used: Exiting.");
    }
}

fn save_weights(task: &str, metric: &Metric, cfg: &cfg::Db) {
    let db_client = db::connect(cfg);
    let db = db_client.db(db::NAME);

    let pairs: Vec<(String, String)> = {
        let sample_docs: Vec<_> = db.collection(db::COLLECTION_SAMPLE)
            .find(
                Some(doc!{ "task": task }),
                Some(FindOptions {
                    projection: Some(doc!{
                        "_id": 1,
                    }),
                    ..Default::default()
                }),
            )
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let sample_ids: Vec<String> = sample_docs
            .into_iter()
            .map(|doc| {
                doc.get_object_id("_id")
                    .expect("Failed deserializing Sample documents")
                    .to_hex()
            })
            .collect();
        sample_ids
            .iter()
            .enumerate()
            .flat_map(|(i, id_a)| {
                let id_a = id_a.clone();
                sample_ids
                    .iter()
                    .skip(i + 1)
                    .map(move |id_b| (id_a.clone(), id_b.clone()))
            })
            .collect()
    };

    let user_tokens = get_user_tokens(&db_client, task);

    let weights: Vec<Weighting> = {
        let weight_docs = db.collection(db::COLLECTION_WEIGHT)
            .find(
                Some(doc! {
                    "token": {
                        "$in": to_bson(&user_tokens).unwrap(),
                    },
                    "metric": serde_enum::to_string(metric).unwrap(),
                }),
                None,
            )
            .expect("Failed quering weights");
        weight_docs
            .map(|doc| {
                let bson = Bson::from(doc.unwrap());
                let weighting: db::Weighting = from_bson(bson).unwrap();
                weighting.into()
            })
            .collect()
    };

    let incomplete_users = {
        // user -> a -> b, and
        // user -> b -> a
        let mut user_weighted: HashMap<&str, HashMap<&str, HashSet<&str>>> = HashMap::new();
        for weight in &weights {
            user_weighted
                .entry(&weight.token)
                .or_insert_with(HashMap::new)
                .entry(&weight.a)
                .or_insert_with(HashSet::new)
                .insert(&weight.b);
            user_weighted
                .entry(&weight.token)
                .or_insert_with(HashMap::new)
                .entry(&weight.b)
                .or_insert_with(HashSet::new)
                .insert(&weight.a);
        }

        let mut incomplete_users: Vec<String> = Vec::new();
        for (user, weighted) in &user_weighted {
            let has_all_weights = pairs.iter().all(
                |&(ref a, ref b)| match weighted.get(a.as_str()) {
                    None => false,
                    Some(paired) => paired.contains(b.as_str()),
                },
            );

            if !has_all_weights {
                incomplete_users.push(user.to_string());
            }
        }

        incomplete_users
    };

    let weights: Vec<Weighting> = weights
        .into_iter()
        .filter(|w| !incomplete_users.contains(&w.token))
        .collect();

    let sample_names = get_sample_name_map(&db_client, task);

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("weights.csv")
        .unwrap();
    writer
        .write_record(&["user", "a_id", "a_name", "b_id", "b_name", "weight"])
        .unwrap();

    for weight in weights {
        writer
            .write_record(&[
                &weight.token,
                &weight.a,
                &sample_names[&weight.a],
                &weight.b,
                &sample_names[&weight.b],
                &format!("{}", weight.weight),
            ])
            .unwrap();
    }
}

fn save_criteria_weights(task: &str, metric: &Metric, cfg: &cfg::Db) {
    let db_client = db::connect(cfg);

    let user_tokens = get_user_tokens(&db_client, task);
    let sample_names = get_sample_name_map(&db_client, task);

    #[derive(Serialize)]
    struct UserWeight {
        user: String,
        item_id: String,
        item_name: String,
        weight: f32,
    }

    let weights: Vec<UserWeight> = user_tokens
        .into_iter()
        .flat_map(|token| {
            let weights = match stats::calculate_sample_weights(task, &token, metric, &db_client) {
                Ok(weights) => weights,
                Err(_) => Vec::new(),
            };
            let sample_names = &sample_names;

            weights.into_iter().map(move |w| {
                UserWeight {
                    user: token.clone(),
                    item_name: sample_names[&w.name].clone(),
                    item_id: w.name,
                    weight: w.weight,
                }
            })
        })
        .collect();

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("criteria-weights.csv")
        .unwrap();

    for weight in weights {
        writer.serialize(weight).unwrap();
    }
}

fn get_sample_name_map(db_client: &mongodb::Client, task: &str) -> HashMap<String, String> {
    let sample_docs = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_SAMPLE)
        .find(
            Some(doc! {
                "task": task,
            }),
            Some(FindOptions {
                projection: Some(doc! {
                    "_id": 1,
                    "name": 1,
                }),
                ..Default::default()
            }),
        )
        .expect("Failed querying samples");

    HashMap::from_iter(sample_docs.map(|doc| {
        let doc = doc.unwrap();
        let id = doc.get_object_id("_id").unwrap().to_hex();
        let name = doc.get_str("name").unwrap().to_string();
        (id, name)
    }))
}

fn get_user_tokens(db_client: &mongodb::Client, task: &str) -> Vec<String> {
    let user_docs = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .find(
            Some(doc! {
                "task": task,
            }),
            Some(FindOptions {
                projection: Some(doc! {
                    "_id": 0,
                    "token": 1,
                }),
                ..Default::default()
            }),
        )
        .expect("Failed querying users");
    user_docs
        .map(|doc| doc.unwrap().get_str("token").unwrap().to_string())
        .collect()
}

fn find_completed_users(
    db_client: &mongodb::Client,
    task: &str,
    metric: &Metric,
) -> HashSet<String> {
    let user_tokens = get_user_tokens(&db_client, task);

    let db = db_client.db(db::NAME);

    let pairs: Vec<(String, String)> = {
        let sample_docs: Vec<_> = db.collection(db::COLLECTION_SAMPLE)
            .find(
                Some(doc!{ "task": task }),
                Some(FindOptions {
                    projection: Some(doc!{
                        "_id": 1,
                    }),
                    ..Default::default()
                }),
            )
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let sample_ids: Vec<String> = sample_docs
            .into_iter()
            .map(|doc| {
                doc.get_object_id("_id")
                    .expect("Failed deserializing Sample documents")
                    .to_hex()
            })
            .collect();
        sample_ids
            .iter()
            .enumerate()
            .flat_map(|(i, id_a)| {
                let id_a = id_a.clone();
                sample_ids
                    .iter()
                    .skip(i + 1)
                    .map(move |id_b| (id_a.clone(), id_b.clone()))
            })
            .collect()
    };

    let weights: Vec<Weighting> = {
        let weight_docs = db.collection(db::COLLECTION_WEIGHT)
            .find(
                Some(doc! {
                    "token": {
                        "$in": to_bson(&user_tokens).unwrap(),
                    },
                    "metric": serde_enum::to_string(metric).unwrap(),
                }),
                None,
            )
            .expect("Failed quering weights");
        weight_docs
            .map(|doc| {
                let bson = Bson::from(doc.unwrap());
                let weighting: db::Weighting = from_bson(bson).unwrap();
                weighting.into()
            })
            .collect()
    };

    // user -> a -> b, and
    // user -> b -> a
    let mut user_weighted: HashMap<&str, HashMap<&str, HashSet<&str>>> = HashMap::new();
    for weight in &weights {
        user_weighted
            .entry(&weight.token)
            .or_insert_with(HashMap::new)
            .entry(&weight.a)
            .or_insert_with(HashSet::new)
            .insert(&weight.b);
        user_weighted
            .entry(&weight.token)
            .or_insert_with(HashMap::new)
            .entry(&weight.b)
            .or_insert_with(HashSet::new)
            .insert(&weight.a);
    }

    let mut completed_users: HashSet<String> = HashSet::new();
    for (user, weighted) in &user_weighted {
        let has_all_weights = pairs
            .iter()
            .all(|&(ref a, ref b)| match weighted.get(a.as_str()) {
                None => false,
                Some(paired) => paired.contains(b.as_str()),
            });

        if has_all_weights {
            completed_users.insert(user.to_string());
        }
    }

    completed_users
}

fn save_users(task: &str, metric: &Metric, cfg: &cfg::Db) {
    let db_client = db::connect(cfg);

    let completed_users = find_completed_users(&db_client, task, metric);

    let user_docs = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .find(
            Some(doc! {
                "task": task,
            }),
            None,
        )
        .expect("Failed querying users");
    let users: Vec<User> = user_docs
        .map(|doc| from_bson(Bson::from(doc.unwrap())).unwrap())
        .collect();

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("users.csv")
        .unwrap();
    writer
        .write_record(&[
            "token",
            "age",
            "gender",
            "education",
            "occupation",
            "from",
            "source",
            "date",
            "browser.name",
            "browser.version",
            "complete",
            "post",
        ])
        .unwrap();

    for user in users {
        let complete = completed_users.contains(&user.token);
        writer
            .write_record(&[
                user.public,
                format!("{}", user.age),
                serde_enum::to_string(&user.gender).unwrap(),
                serde_enum::to_string(&user.education).unwrap(),
                serde_enum::to_string(&user.occupation).unwrap(),
                user.from.unwrap_or_else(|| "".to_string()),
                user.source,
                user.register_date
                    .format("%Y-%m-%dT%H:%M:%S%.f")
                    .to_string(),
                user.browser
                    .clone()
                    .map(|b| b.name)
                    .unwrap_or_else(|| "".to_string()),
                user.browser
                    .map(|b| b.version)
                    .unwrap_or_else(|| "".to_string()),
                format!("{}", complete),
                format!("{}", user.post_questionnaire.is_some()),
            ])
            .unwrap();
    }
}

fn save_questionnaires(task: &str, metric: &Metric, cfg: &cfg::Db) {
    use model::{Likert5, PostQuestionnaire, PreQuestionnaire};

    #[derive(Serialize)]
    struct Questionnaire {
        user: String,
        plant_work: Likert5,
        plant_like: Likert5,
        video_game: Likert5,
        ranking_agree: Option<Likert5>,
        disagree_why: Option<String>,
        differentiates: Option<String>,
        comments: Option<String>,
    }

    impl From<(String, PreQuestionnaire, Option<PostQuestionnaire>)> for Questionnaire {
        fn from(
            (user, pre, post): (String, PreQuestionnaire, Option<PostQuestionnaire>),
        ) -> Questionnaire {
            let post = post.as_ref();
            Questionnaire {
                user: user,
                plant_work: pre.plant_work,
                plant_like: pre.plant_like,
                video_game: pre.video_game,
                ranking_agree: post.map(|p| p.ranking_agree),
                disagree_why: post.map(|p| p.disagree_why.clone()).unwrap_or(None),
                differentiates: post.map(|p| p.differentiates.clone()).unwrap_or(None),
                comments: post.map(|p| p.comments.clone()).unwrap_or(None),
            }
        }
    }

    let db_client = db::connect(cfg);

    let completed_users = find_completed_users(&db_client, task, metric);

    let user_docs = db_client
        .db(db::NAME)
        .collection(db::COLLECTION_USER)
        .find(
            Some(doc! {
                "task": task,
            }),
            None,
        )
        .expect("Failed querying users");
    let users: Vec<User> = user_docs
        .map(|doc| from_bson(Bson::from(doc.unwrap())).unwrap())
        .collect();

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("questionnaires.csv")
        .unwrap();

    for user in users {
        let complete = completed_users.contains(&user.token);
        if complete {
            if let Some(pre) = user.pre_questionnaire {
                let q = Questionnaire::from((user.public, pre, user.post_questionnaire));
                writer.serialize(&q).unwrap();
            }
        }
    }
}
