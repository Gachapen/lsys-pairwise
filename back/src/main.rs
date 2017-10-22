#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate mongodb;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
#[macro_use]
extern crate serde_derive;

mod cfg;
mod routes;

use mongodb::ThreadedClient;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors};
use std::env;

use cfg::Config;
use routes::routes;

fn main() {
    let config = Config {
        db: cfg::Db {
            host: env::var("CORINOR_DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        },
    };

    let db_client = mongodb::Client::connect(&config.db.host, 27017)
        .expect(&format!("Failed to connect to DB at {}", config.db.host));
    println!("Connected to MongoDB at {}", config.db.host);

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
