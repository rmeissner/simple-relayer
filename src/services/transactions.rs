use crate::models::service::PreparePayload;
use crate::providers::ethereum::{Call, CallOptions, EthereumProvider, to_string_result};
use crate::utils::context::Context;
use anyhow::Result;

use ethabi_contract::use_contract;

use_contract!(eip20, "./res/eip20.json");

pub fn prepare(context: &Context, payload: PreparePayload) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);
    let rpc_out = eth_provider.call(
        &Call {
            to: Some(payload.to),
            from: Some(payload.wallet),
            value: Some(payload.value),
            data: Some(payload.data),
            gas: None,
            gas_price: None
        },
        &CallOptions {
            block: "latest".to_string()
        }
    )?;
    to_string_result(rpc_out)
}
