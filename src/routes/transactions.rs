use crate::utils::context::Context;
use crate::services::transactions;
use rocket::response::content;
use rocket_contrib::json::Json;
use anyhow::Result;

#[post("/v1/transactions", format = "json", data = "<update>")]
pub fn estimate(context: Context, update: Json<String>) -> Result<String> {
    transactions::estimate(context, "".to_string(), "".to_string())
}

#[post("/v1/transactions", format = "json", data = "<update>")]
pub fn execute(context: Context, update: Json<String>) -> Result<()> {
    Ok(())
}