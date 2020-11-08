use super::{Account, Estimation};
use crate::models::service::ExecutePayload;
use crate::providers::ethereum::types::Bytes;
use crate::providers::ethereum::{to_string_result, Call, CallOptions, EthereumProvider};
use anyhow::Result;
use ethabi_contract::use_contract;
use ethereum_types::{Address, U256};

pub struct SafeAccount<'a> {
    pub eth_provider: &'a EthereumProvider<'a>,
}

use_contract!(safe, "./res/safe.json");
impl Account for SafeAccount<'_> {
    type Payload = ExecutePayload;
    fn estimate(&self, payload: &Self::Payload) -> Result<Estimation> {
        // TODO check wallet
        let wallet = payload.wallet;
        let data: Bytes = safe::functions::exec_transaction::encode_input(
            payload.transaction.to,
            payload.transaction.value,
            payload.transaction.data.clone(),
            payload.transaction.operation,
            payload.transaction.safe_tx_gas,
            0,
            0,
            Address::zero(),
            Address::zero(),
            payload.signatures.clone(),
        ).into();
        let mut call = Call {
            to: Some(wallet),
            value: None,
            data: Some(data.clone()),
            gas: None,
            gas_price: None,
            from: Some(self.eth_provider.account()),
        };
        let options = CallOptions {
            block: "latest".to_string(),
        };
        let estimate_result = to_string_result(self.eth_provider.estimate_gas(&call, &options)?)?;
        let mut estimate = u64::from_str_radix(estimate_result.trim_start_matches("0x"), 16)?;
        estimate += estimate / 4;
        let mut success = false;
        while !success && estimate < 20_000_000 {
            call.gas = Some(U256::from(estimate));
            let simulate_result = self.eth_provider.call(&call, &options)?;
            let bytes: Bytes = to_string_result(simulate_result)?.into();
            success = safe::functions::exec_transaction::decode_output(&bytes.0)?;
            log::debug!("estimate: {}", estimate);
            if !success {
                estimate = estimate * 2;
            }
        }
        if !success {
            anyhow::bail!("Cannot estimate transaction with success");
        }
        Ok(Estimation { wallet, estimate: U256::from(estimate), data })
    }
}
