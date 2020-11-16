#![feature(proc_macro_hygiene, decl_macro, option_result_contains)]

extern crate log;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

extern crate dotenv;

mod config;
mod routes;
mod services;
mod models;
mod utils;
mod providers;

use dotenv::dotenv;
use utils::cors::{CORS};
use routes::active_routes;
use crate::routes::error_catchers;

fn main() {
    dotenv().ok();
    env_logger::init();
    
    rocket::ignite()
        .mount("/", active_routes())
        .manage(reqwest::blocking::Client::new())
        .attach(CORS())
        .register(error_catchers())
        .launch();
}
