use crate::providers::ethereum::types::Bytes;
use serde::{Deserialize, Serialize};
use ethereum_types::{Address, U256};

#[derive(Serialize, Debug)]
pub struct About {
    pub transaction_service_base_url: String,
    pub name: String,
    pub version: String,
    pub build_number: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct PreparePayload {
    pub wallet: Address,
    pub to: Address,
    pub value: U256,
    pub data: Bytes
}