pub mod hash;
pub mod key;
pub mod types;
pub mod transaction;

use crate::config::{key_bytes};
use crate::utils::context::Context;
use anyhow::Result;
use serde::Serialize;
use serde_json;
use ethereum_types::{Address};
use jsonrpc_core as rpc;
use transaction::Transaction;

#[derive(Serialize, Debug)]
pub struct CallOptions {
    block: String,
    from: Option<Address>
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
        transaction: &'_ Transaction,
        option: &'_ CallOptions
    ) -> Result<String> {
        let resp = single_rpc_call(self.client, "", build_request(
            1, "eth_call", vec![serde_json::to_value(&transaction)?, serde_json::to_value(&option)?]
        ))?;
        match resp {
            rpc::Value::String(val) => Ok(val),
            _ => anyhow::bail!("Unexpected type"),
        }
    }

    pub fn estimate_gas(
        &self,
        transaction: &'_ Transaction,
        option: &'_ CallOptions
    ) -> Result<String> {
        let resp = single_rpc_call(self.client, "", build_request(
            1, "eth_estimateGas", vec![serde_json::to_value(&transaction)?, serde_json::to_value(&option)?]
        ))?;
        match resp {
            rpc::Value::String(val) => Ok(val),
            _ => anyhow::bail!("Unexpected type"),
        }
    }

    pub fn execute(
        &self,
        transaction: &'_ Transaction
    ) -> Result<String> {
        let signed = transaction.sign(&self.get_key(), None);
        let resp = single_rpc_call(self.client, "", build_request(
            1, "eth_sendRawTransaction", vec![serde_json::to_value(&signed)?]
        ))?;
        match resp {
            rpc::Value::String(val) => Ok(val),
            _ => anyhow::bail!("Unexpected type"),
        }
    }
}

fn single_rpc_call(client: &'_ reqwest::blocking::Client, api: &str, call: rpc::Call) -> Result<rpc::Value> {
    let response = client.post(api).json(&call).send()?.json::<rpc::Response>()?;
    match response {
        rpc::Response::Single(output) => to_result_from_output(output),
        _ => anyhow::bail!("Expected single, got batch."),
    }
}

/// Parse `rpc::Output` into `Result`.
fn to_result_from_output(output: rpc::Output) -> Result<rpc::Value> {
    match output {
        rpc::Output::Success(success) => Ok(success.result),
        rpc::Output::Failure(_) => anyhow::bail!("Json RPC call failed!"),
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