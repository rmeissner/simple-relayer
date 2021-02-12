use crate::models::{GenericRelayData};
use crate::config::{exec_tx_refunder_address, chain_id};
use crate::providers::ethereum::{to_string_result, Call, CallOptions, EthereumProvider, ItxTransaction, KeyType};
use crate::providers::ethereum::types::Bytes;
use crate::providers::ethereum::hash::{keccak256};
use crate::utils::context::Context;
use anyhow::Result;
use ethabi_contract::use_contract;
use ethereum_types::{Address, U256};
use ethabi;
use serde_json;

use_contract!(refunder, "./res/refunder.json");

fn estimate_gas(
    eth_provider: &EthereumProvider,
    target: &Address,
    data: &Bytes
) -> Result<u64> {
    let call = Call {
        to: Some(target.clone()),
        value: None,
        data: Some(data.clone()),
        gas: None,
        gas_price: None,
        from: Some(eth_provider.account()),
    };
    let options = CallOptions {
        block: "latest".to_string(),
    };
    let estimate_result = to_string_result(eth_provider.estimate_gas(&call, &options)?)?;
    let mut estimate = u64::from_str_radix(estimate_result.trim_start_matches("0x"), 16)?;
    estimate += estimate / 4;
    Ok(estimate)
}

pub fn relay_itx(context: &Context, payload: GenericRelayData) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);

    if payload.method != "0x6a761202" { anyhow::bail!("Invalid method"); }

    let target: Address = serde_json::from_value(serde_json::value::Value::String(exec_tx_refunder_address()))?;
    let data: Bytes = refunder::functions::execute::encode_input(payload.to, payload.method_data).into();
    let estimation = estimate_gas(&eth_provider, &target, &data)?;

    let itx_tx_hash = keccak256(&ethabi::encode(&[
        ethabi::Token::Address(target),
        ethabi::Token::Bytes(data.0.clone()),
        ethabi::Token::Uint(U256::from(estimation)),
        ethabi::Token::Uint(U256::from(chain_id()))
    ]));
    let signature = eth_provider.sign(&itx_tx_hash, KeyType::Itx)?;
    log::debug!("itx account: {}", eth_provider.itx_account());
    let mut signature_vec = [signature.r, signature.s].concat();
    signature_vec.push((signature.v + 27) as u8);
    let itx_tx = ItxTransaction {
        to: target,
        data: data,
        gas: estimation.to_string()
    };
    to_string_result(eth_provider.itx_relay(&itx_tx, &Bytes(signature_vec))?)
}