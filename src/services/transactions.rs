use crate::utils::context::Context;
use anyhow::Result;

use ethabi_contract::use_contract;

use_contract!(eip20, "./res/eip20.json");

pub fn estimate(context: Context, wallet: String, data: String) -> Result<String> {
    Ok("".to_string())
}