use bson::to_bson;
use mongodb::{self, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::Collection;
use mongodb::coll::options::ReplaceOptions;
use rocket;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors};
use std::{fs, io};
use std::path::Path;

use cfg::Config;
use db;
use model::Sample;
use routes::routes;

const VIDEOS_PATH: &'static str = "./task";

pub fn run() {
    let config = Config::from_env();
    let db_client = db::connect(&config.db);

    db::init(&db_client).expect("Failed initializing DB");
    println!("Initialized DB");

    scan_tasks(&db_client).expect("Failed scanning for videos");

    let mut ignition = rocket::ignite()
        .manage(config)
        .manage(db_client)
        .mount("/", routes());

    let rocket_env = rocket::config::Environment::active()
        .expect("Something is wrong with the Rocket environment");

    if rocket_env.is_dev() {
        let allowed_origins = AllowedOrigins::all();

        let options = Cors {
            allowed_origins: allowed_origins,
            allowed_methods: vec![Method::Get, Method::Post]
                .into_iter()
                .map(From::from)
                .collect(),
            allowed_headers: AllowedHeaders::some(&["Accept", "Content-Type"]),
            ..Cors::default()
        };

        ignition = ignition.attach(options);
    }

    ignition.launch();
}

fn scan_videos(dir_path: &Path, collection: &Collection) -> Result<(), io::Error> {
    let task = dir_path.file_name().unwrap().to_str().unwrap();
    let dir_entries: Vec<_> = fs::read_dir(dir_path)?.collect::<Result<_, _>>()?;
    let mut names: Vec<_> = dir_entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_stem) = path.file_stem() {
                    return Some(file_stem.to_str().unwrap().to_string());
                }
            }

            println!("Warning: Ignoring video entry '{}'", path.to_str().unwrap());
            None
        })
        .collect();
    names.sort();
    names.dedup();

    for name in names {
        let sample = Sample {
            task: task.to_string(),
            name: name,
        };
        let sample_bson = to_bson(&sample).unwrap();
        let sample_doc = sample_bson.as_document().unwrap();

        let insertion_res = collection.replace_one(
            sample_doc.clone(),
            sample_doc.clone(),
            Some(ReplaceOptions {
                upsert: Some(true),
                ..Default::default()
            }),
        );

        if let Err(error) = insertion_res {
            if let mongodb::Error::IoError(error) = error {
                return Err(error);
            }

            return Err(io::Error::new(io::ErrorKind::Other, error));
        }

        println!("Registered video for task '{}': '{}'", task, sample.name);
    }

    Ok(())
}

fn scan_tasks(db_client: &mongodb::Client) -> Result<(), io::Error> {
    let collection = db_client.db(db::NAME).collection(db::COLLECTION_SAMPLE);

    for entry in fs::read_dir(VIDEOS_PATH)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_videos(&path, &collection)?;
        }
    }

    Ok(())
}
