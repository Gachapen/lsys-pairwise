#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate bson;
extern crate mongodb;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
#[macro_use]
extern crate serde_derive;

mod cfg;
mod routes;
mod db;

use bson::to_bson;
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::ReplaceOptions;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors};
use std::{env, fs, io};

use db::Sample;
use cfg::Config;
use routes::routes;

const VIDEOS_PATH: &'static str = "./video";

fn main() {
    let config = Config {
        db: cfg::Db {
            host: env::var("CORINOR_DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        },
    };

    let db_client = mongodb::Client::connect(&config.db.host, 27017)
        .expect(&format!("Failed to connect to DB at {}", config.db.host));
    println!("Connected to MongoDB at {}", config.db.host);

    db::init(&db_client).expect("Failed initializing DB");
    println!("Initialized DB");

    scan_videos(&db_client).expect("Failed scanning for videos");

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

fn scan_videos(db_client: &mongodb::Client) -> Result<(), io::Error> {
    let collection = db_client.db(db::NAME).collection(db::COLLECTION_SAMPLE);

    for entry in fs::read_dir(VIDEOS_PATH)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_stem) = path.file_stem() {
                let sample = Sample {
                    name: file_stem.to_str().unwrap().to_string(),
                };
                let sample_bson = to_bson(&sample).unwrap();
                let sample_doc = sample_bson.as_document().unwrap();

                let insertion_res = collection.replace_one(
                    doc! { "name": &sample.name },
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

                println!("Registered video '{}'", sample.name);
            } else {
                println!("Warning: Ignoring video entry '{}'", path.to_str().unwrap());
            }
        }
    }

    Ok(())
}
