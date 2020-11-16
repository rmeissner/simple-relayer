use crate::utils::context::Context;
use crate::services::deployment;
use crate::models::{DeployPayload};
use rocket_contrib::json::Json;
use anyhow::Result;

#[post("/v1/deployment/execute", format = "json", data = "<update>")]
pub fn deploy(context: Context, update: Json<DeployPayload>) -> Result<String> {
    Ok(serde_json::to_string(&deployment::deploy(&context, update.0)?)?)
}