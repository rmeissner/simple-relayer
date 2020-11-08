extern crate rocket;

use rocket::Catcher;
use rocket::Route;
use rocket_contrib::json::JsonValue;

pub mod about;
pub mod transactions;

pub fn active_routes() -> Vec<Route> {
    routes![
        about::backbone,
        about::info,
        transactions::estimate,
        transactions::execute_safe,
        transactions::execute_vault
    ]
}

pub fn error_catchers() -> Vec<Catcher> {
    catchers![not_found, panic]
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}
#[catch(500)]
fn panic() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Server error occurred."
    })
}
