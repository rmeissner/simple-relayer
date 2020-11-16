use super::{Account, Estimation};
use crate::providers::ethereum::types::Bytes;
use crate::providers::ethereum::{to_string_result, Call, CallOptions, EthereumProvider};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use ethabi_contract::use_contract;
use ethereum_types::{Address, H256, U256};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultConfigPayload {
    pub wallet: Address,
    pub implementation: Address,
    pub signers: Vec<Address>,
    pub threshold: U256,
    pub signature_validator: Address,
    pub request_guard: Address,
    pub fallback_handler: Address,
    pub hook: Bytes,
    pub nonce: U256,
    pub meta_hash: H256,
    pub validation_data: Bytes
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultConfigFee {
    pub fee: U256,
    pub fee_receiver: Address,
    pub hook: Bytes
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultPayload {
    pub wallet: Address,
    pub validation_data: Bytes,
    pub transaction: VaultTransaction
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VaultTransaction {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub operation: u8,
    pub min_available_gas: U256,
    pub nonce: U256,
    pub meta_hash: H256
}

pub struct VaultAccount<'a> {
    pub eth_provider: &'a EthereumProvider<'a>,
}

use_contract!(stateless_vault, "./res/vault.json");

impl VaultAccount<'_> {

    pub fn estimate_config_update(&self, payload: &VaultConfigPayload) -> Result<Estimation> {
        let wallet = payload.wallet;
        let data: Bytes = stateless_vault::functions::update_config::encode_input(
            payload.implementation,
            payload.signers.clone(),
            payload.threshold,
            payload.signature_validator,
            payload.request_guard,
            payload.fallback_handler,
            payload.hook.clone(),
            payload.nonce,
            payload.meta_hash,
            payload.validation_data.clone()
        ).into();
        let call = Call {
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
        Ok(Estimation { wallet, estimate: U256::from(estimate), data })
    }

}

impl Account for VaultAccount<'_> {
    type Payload = VaultPayload;
    fn estimate(&self, payload: &Self::Payload) -> Result<Estimation> {
        // TODO check wallet
        let wallet = payload.wallet;
        let data: Bytes = stateless_vault::functions::exec_transaction::encode_input(
            payload.transaction.to,
            payload.transaction.value,
            payload.transaction.data.clone(),
            payload.transaction.operation,
            payload.transaction.min_available_gas,
            payload.transaction.nonce,
            payload.transaction.meta_hash,
            payload.validation_data.clone(),
            true
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
            success = stateless_vault::functions::exec_transaction::decode_output(&bytes.0)?;
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
