#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate bson;
extern crate clap;
extern crate mongodb;
extern crate rand;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

mod cfg;
mod routes;
mod db;
mod model;
mod server;

use clap::{App, SubCommand};

fn main() {
    let matches = App::new("lsys-pairwise")
        .version("0.1")
        .author("Magnus Bjerke Vik <mbvett@gmail.com>")
        .about("Pairwise comparison of LSystems")
        .subcommand(SubCommand::with_name("server").about("Run server"))
        .get_matches();

    if matches.subcommand_matches("server").is_some() {
        server::run();
    } else {
        println!("No subcommand used: Exiting.");
    }
}
