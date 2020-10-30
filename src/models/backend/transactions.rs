use super::super::commons::Operation;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MultisigTransaction {
    pub safe: String,
    pub to: String,
    pub value: Option<String>,
    pub data: Option<String>,
    pub operation: Option<Operation>,
    pub gas_token: Option<String>,
    pub safe_tx_gas: Option<usize>,
    pub base_gas: Option<usize>,
    pub gas_price: Option<String>,
    pub refund_receiver: Option<String>,
    pub nonce: u64,
    pub execution_date: Option<DateTime<Utc>>,
    pub submission_date: DateTime<Utc>,
    pub modified: Option<DateTime<Utc>>,
    pub block_number: Option<usize>,
    pub transaction_hash: Option<String>,
    pub safe_tx_hash: String,
    pub executor: Option<String>,
    pub is_executed: bool,
    pub is_successful: Option<bool>,
    pub eth_gas_price: Option<String>,
    pub gas_used: Option<usize>,
    pub fee: Option<String>,
    pub origin: Option<String>,
    pub confirmations_required: Option<u64>,
    pub confirmations: Option<Vec<Confirmation>>,
    pub signatures: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Confirmation {
    pub owner: String,
    pub submission_date: DateTime<Utc>,
    pub transaction_hash: Option<String>,
    pub signature_type: String,
    pub signature: Option<String>,
}