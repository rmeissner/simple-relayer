use crate::utils::context::Context;
use crate::services::transactions;
use crate::models::service::{ExecutePayload, PreparePayload};
use crate::providers::accounts::vault::VaultPayload;
use rocket_contrib::json::Json;
use anyhow::Result;

#[post("/v1/transactions/prepare", format = "json", data = "<update>")]
pub fn estimate(context: Context, update: Json<PreparePayload>) -> Result<String> {
    Ok(serde_json::to_string(&transactions::prepare(&context, update.0)?)?)
}

#[post("/v1/transactions/execute/safe", format = "json", data = "<transaction>")]
pub fn execute_safe(context: Context, transaction: Json<ExecutePayload>) -> Result<String> {
    transactions::execute_safe(&context, transaction.0)
}

#[post("/v1/transactions/execute/vault", format = "json", data = "<transaction>")]
pub fn execute_vault(context: Context, transaction: Json<VaultPayload>) -> Result<String> {
    transactions::execute_vault(&context, transaction.0)
}