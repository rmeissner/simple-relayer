#![feature(proc_macro_hygiene, decl_macro, option_result_contains)]

extern crate log;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

extern crate dotenv;

#[macro_use]
pub mod macros;

mod config;
mod routes;
mod services;
mod models;
mod utils;
mod providers;

#[cfg(test)]
mod json;

use dotenv::dotenv;
use utils::cache::{ServiceCache};
use utils::cors::{CORS};
use routes::active_routes;
use std::collections::HashMap;
use rocket::config::{Value};
use crate::routes::error_catchers;

fn main() {
    dotenv().ok();
    env_logger::init();

    // Ignore Rocket.toml only use ENV
    let mut config = rocket::config::RocketConfig::active_default().unwrap().active().clone();
    
    // Set database extra (hacky as hell)
    let mut extras = config.extras.clone();
    let database_config: HashMap<String, Value> = [("url".to_string(), Value::from(config::service_cache_url()))].iter().cloned().collect();
    let databases: HashMap<String, Value> = [("service_cache".to_string(), Value::from(database_config))].iter().cloned().collect();
    extras.insert("databases".to_string(), Value::from(databases));
    config.set_extras(extras);
    
    rocket::custom(config)
        .mount("/", active_routes())
        .manage(reqwest::blocking::Client::new())
        .attach(CORS())
        .attach(ServiceCache::fairing())
        .register(error_catchers())
        .launch();
}
