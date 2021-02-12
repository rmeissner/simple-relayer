use crate::providers::ethereum::types::Bytes;
use serde::{Deserialize, Serialize};
use ethereum_types::{Address, U256};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct About {
    pub name: String,
    pub version: String,
    pub build_number: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PreparePayload {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub operation: u8
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResult {
    pub fee: U256,
    pub fee_receiver: Address,
    pub transaction: SafeTransaction
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePayload {
    pub wallet: Address,
    pub signatures: Bytes,
    pub transaction: SafeTransaction
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeployPayload {
    pub implementation: Address,
    pub validators: Vec<Address>,
    pub signatures: Bytes,
    pub transaction: SafeTransaction,
    pub nonce: U256
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SafeTransaction {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub operation: u8,
    pub safe_tx_gas: U256
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenericRelayData {
    pub to: Address,
    pub method: String,
    pub method_data: Bytes
}