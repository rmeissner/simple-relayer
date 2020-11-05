use crate::providers::ethereum::types::Bytes;
use serde::{Deserialize, Serialize};
use ethereum_types::{Address, U256};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct About {
    pub transaction_service_base_url: String,
    pub name: String,
    pub version: String,
    pub build_number: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PreparePayload {
    pub wallet: Address,
    pub to: Address,
    pub value: U256,
    pub data: Bytes
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
pub struct SafeTransaction {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub operation: u8,
    pub safe_tx_gas: U256
}