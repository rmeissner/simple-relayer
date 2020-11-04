use crate::utils::context::Context;
use crate::services::transactions;
use crate::models::service::PreparePayload;
use rocket::response::content;
use rocket_contrib::json::Json;
use anyhow::Result;

#[post("/v1/transactions/prepare", format = "json", data = "<update>")]
pub fn estimate(context: Context, update: Json<PreparePayload>) -> Result<String> {
    transactions::prepare(&context, update.0)
}

#[post("/v1/transactions/execute", format = "json", data = "<update>")]
pub fn execute(context: Context, update: Json<String>) -> Result<()> {
    Ok(())
}