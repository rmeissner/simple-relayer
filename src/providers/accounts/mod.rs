pub mod safe;
pub mod utils;

use crate::config::{multisend_address, transaction_fee};
use crate::providers::ethereum::{EthereumProvider};
use crate::providers::ethereum::types::Bytes;
use utils::decode_multisend_bytes;
use ethereum_types::{Address, U256};
use serde_json;

use anyhow::Result;

pub fn check_fee(eth_provider: &EthereumProvider, to: Address, value: U256, data: &Vec<u8>, operation: u8) -> Result<()> {
    let fee = U256::from_dec_str(&transaction_fee())?;
    if fee != U256::zero() {
        // Check payment
        let ms_address =
            serde_json::from_value(serde_json::value::Value::String(multisend_address()))?;
        anyhow::ensure!(
            to == ms_address,
            "Fee payment requires multisend"
        );
        anyhow::ensure!(
           operation == 1,
            "Multisend requires delegatecall"
        );
        anyhow::ensure!(
            value == U256::zero(),
            "Delegate call should not contain value"
        );
        let txs = decode_multisend_bytes(data);
        anyhow::ensure!(
            txs.len() > 1,
            "There should be at least 1 user tx + the payment tx"
        );
        let payment_tx = &txs[txs.len() - 1];
        anyhow::ensure!(payment_tx.operation == 0, "Payment should be call");
        anyhow::ensure!(
            payment_tx.to == eth_provider.account(),
            "Payment should go to relayer"
        );
        anyhow::ensure!(
            payment_tx.data.0.len() == 0,
            "Payment should not contain data"
        );
        anyhow::ensure!(payment_tx.value == fee, "Full payment be send in native coin");
    };
    Ok(())
}

pub struct Estimation {
    pub wallet: Address,
    pub data: Bytes,
    pub estimate: U256
}

pub trait Account {
    type Payload;

    fn estimate(&self, payload: &Self::Payload) -> Result<Estimation>;
}