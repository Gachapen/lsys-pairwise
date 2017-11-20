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

use cfg::Config;
use model::{Metric, Weighting};

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
        .get_matches();

    if matches.subcommand_matches("server").is_some() {
        server::run();
    } else if let Some(matches) = matches.subcommand_matches("stats") {
        let task = matches.value_of("task").unwrap();
        let token = matches.value_of("token").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        stats::print_stats(task, token, &metric, &cfg.db);
    } else if let Some(matches) = matches.subcommand_matches("save-weights") {
        let task = matches.value_of("task").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        save_weights(task, &metric, &cfg.db);
    } else {
        println!("No subcommand used: Exiting.");
    }
}

fn save_weights(task: &str, metric: &Metric, cfg: &cfg::Db) {
    let db_client = db::connect(cfg);
    let db = db_client.db(db::NAME);

    let user_docs = db.collection(db::COLLECTION_USER)
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

    let user_tokens: Vec<_> = user_docs
        .map(|doc| doc.unwrap().get_str("token").unwrap().to_string())
        .collect();

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

    let weights: Vec<Weighting> = weight_docs
        .map(|doc| {
            let bson = Bson::from(doc.unwrap());
            let weighting: db::Weighting = from_bson(bson).unwrap();
            weighting.into()
        })
        .collect();

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("weights.csv")
        .unwrap();
    writer.write_record(&["user", "a", "b", "weight"]).unwrap();

    for weight in weights {
        writer
            .write_record(&[
                weight.token,
                weight.a,
                weight.b,
                format!("{}", weight.weight),
            ])
            .unwrap();
    }
}
