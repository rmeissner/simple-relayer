use crate::utils::context::Context;
use crate::services::transactions;
use crate::models::service::{ExecutePayload, PreparePayload};
use rocket_contrib::json::Json;
use anyhow::Result;

#[post("/v1/transactions/prepare", format = "json", data = "<update>")]
pub fn estimate(context: Context, update: Json<PreparePayload>) -> Result<String> {
    Ok(serde_json::to_string(&transactions::prepare(&context, update.0)?)?)
}

#[post("/v1/transactions/execute", format = "json", data = "<update>")]
pub fn execute(context: Context, update: Json<ExecutePayload>) -> Result<String> {
    transactions::execute(&context, update.0)
}