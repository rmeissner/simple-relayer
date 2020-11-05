pub mod hash;
pub mod key;
pub mod types;
pub mod transaction;

use crate::config::{key_bytes, base_rpc_url};
use crate::utils::context::Context;
use types::Bytes;
use ethereum_types::{Address, U256};
use anyhow::Result;
use serde::Serialize;
use serde_json;
use jsonrpc_core as rpc;
use transaction::Transaction;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Call {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub gas_price: Option<U256>,
    pub gas: Option<U256>,
    pub value: Option<U256>,
    /// Call data of the transaction, can be empty for simple value transfers.
    pub data: Option<Bytes>
}

#[derive(Serialize, Debug)]
pub struct CallOptions {
    pub block: String,
}

pub struct EthereumProvider<'p> {
    client: &'p reqwest::blocking::Client
}

impl EthereumProvider<'_> {
    pub fn new<'p>(context: &'p Context) -> EthereumProvider<'p> {
        EthereumProvider {
            client: context.client()
        }
    }

    fn get_key(&self) -> key::PrivateKey {
        key::PrivateKey::from_hex_str(key_bytes()).unwrap()
    }

    pub fn account(&self) -> Address {
        self.get_key().public_address()
    }

    pub fn call(
        &self,
        transaction: &'_ Call,
        option: &'_ CallOptions
    ) -> Result<rpc::Output> {
        single_rpc_call(self.client, build_request(
            1, "eth_call", vec![serde_json::to_value(&transaction)?, serde_json::to_value(&option.block)?]
        ))
    }

    pub fn estimate_gas(
        &self,
        transaction: &'_ Call,
        option: &'_ CallOptions
    ) -> Result<rpc::Output> {
        single_rpc_call(self.client, build_request(
            1, "eth_estimateGas", vec![serde_json::to_value(&transaction)?, serde_json::to_value(&option.block)?]
        ))
    }

    pub fn execute(
        &self,
        transaction: &'_ Transaction
    ) -> Result<rpc::Output> {
        let signed = transaction.sign(&self.get_key(), None);
        single_rpc_call(self.client, build_request(
            1, "eth_sendRawTransaction", vec![serde_json::to_value(&signed)?]
        ))
    }

    pub fn nonce(&self) -> Result<rpc::Output> {
        single_rpc_call(self.client, build_request(
            1, "eth_getTransactionCount", vec![serde_json::to_value(self.account())?, serde_json::to_value("pending")?]
        ))
    }
}

fn single_rpc_call(client: &'_ reqwest::blocking::Client, call: rpc::Call) -> Result<rpc::Output> {
    let response = client.post(&base_rpc_url()).json(&call).send()?.json::<rpc::Response>()?;
    match response {
        rpc::Response::Single(output) => Ok(output),
        _ => anyhow::bail!("Expected single, got batch."),
    }
}

pub fn to_string_result(output: rpc::Output) -> Result<String> {
    let resp = to_result_from_output(output)?;
    match resp {
        rpc::Value::String(val) => Ok(val),
        _ => anyhow::bail!("Unexpected type"),
    }
}

/// Parse `rpc::Output` into `Result`.
pub fn to_result_from_output(output: rpc::Output) -> Result<rpc::Value> {
    match output {
        rpc::Output::Success(success) => Ok(success.result),
        rpc::Output::Failure(error) => anyhow::bail!("Json RPC call failed! {:?}", error),
    }
}

/// Build a JSON-RPC request.
fn build_request(id: usize, method: &str, params: Vec<rpc::Value>) -> rpc::Call {
    rpc::Call::MethodCall(rpc::MethodCall {
        jsonrpc: Some(rpc::Version::V2),
        method: method.into(),
        params: rpc::Params::Array(params),
        id: rpc::Id::Num(id as u64),
    })
}