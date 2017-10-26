#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate bson;
extern crate clap;
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
extern crate uuid;

mod cfg;
mod routes;
mod db;
mod model;
mod server;
mod stats;
mod serde_enum;

use clap::{App, Arg, SubCommand};

use cfg::Config;

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
                    Arg::with_name("token")
                        .short("t")
                        .long("token")
                        .takes_value(true)
                        .required(true)
                        .help("User token to see stats for"),
                )
                .arg(
                    Arg::with_name("metric")
                        .short("m")
                        .long("metric")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["realistic", "pleasing"])
                        .help("Type of metric to see stats for"),
                ),
        )
        .get_matches();

    if matches.subcommand_matches("server").is_some() {
        server::run();
    } else if let Some(matches) = matches.subcommand_matches("stats") {
        let token = matches.value_of("token").unwrap();
        let metric = serde_enum::from_str(matches.value_of("metric").unwrap()).unwrap();
        let cfg = Config::from_env();
        stats::print_stats(token, &metric, &cfg.db);
    } else {
        println!("No subcommand used: Exiting.");
    }
}
