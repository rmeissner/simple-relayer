use crate::utils::context::Context;
use crate::services::transactions;
use crate::services::itx;
use crate::models::{ExecutePayload, GenericRelayData, PreparePayload};
use crate::providers::accounts::vault::{VaultPayload, VaultConfigPayload};
use rocket::response::content;
use rocket_contrib::json::Json;
use anyhow::Result;

#[post("/v1/transactions/prepare", format = "json", data = "<update>")]
pub fn estimate(context: Context, update: Json<PreparePayload>) -> Result<content::Json<String>> {
    Ok(content::Json(serde_json::to_string(&transactions::prepare(&context, update.0)?)?))
}

#[post("/v1/transactions/execute/safe", format = "json", data = "<transaction>")]
pub fn execute_safe(context: Context, transaction: Json<ExecutePayload>) -> Result<String> {
    transactions::execute_safe(&context, transaction.0)
}

#[post("/v1/transactions/execute/vault", format = "json", data = "<transaction>")]
pub fn execute_vault(context: Context, transaction: Json<VaultPayload>) -> Result<String> {
    transactions::execute_vault(&context, transaction.0)
}

#[post("/v1/transactions/update/vault", format = "json", data = "<update>")]
pub fn update_vault(context: Context, update: Json<VaultConfigPayload>) -> Result<String> {
    transactions::update_vault(&context, update.0)
}

#[get("/v1/transactions/update/vault", format = "json")]
pub fn update_vault_fee(context: Context) -> Result<content::Json<String>> {
    Ok(content::Json(serde_json::to_string(&transactions::update_vault_hook(&context)?)?))
}

#[post("/v1/transactions/execute/generic", format = "json", data = "<transaction>")]
pub fn relay_itx(context: Context, transaction: Json<GenericRelayData>) -> Result<String> {
    itx::relay_itx(&context, transaction.0)
}