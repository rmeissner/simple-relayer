use crate::config::{factory_address, transaction_fee};
use crate::models::{DeployPayload, PreparePayload, PrepareResult, SafeTransaction};
use crate::providers::ethereum::transaction::Transaction;
use crate::providers::accounts::{check_fee, Estimation};
use crate::providers::ethereum::{to_string_result, Call, CallOptions, EthereumProvider};
use crate::providers::ethereum::types::Bytes;
use crate::utils::context::Context;
use anyhow::Result;
use ethabi_contract::use_contract;
use ethereum_types::{Address, U256};
use serde_json;

// https://github.com/openethereum/ethabi/blob/master/tests/src/lib.rs
use_contract!(factory, "./res/factory.json");

fn estimate(eth_provider: &EthereumProvider, payload: &DeployPayload) -> Result<Estimation> {
    log::debug!("estimate");
    let data: Bytes = factory::functions::create_proxy_with_initializor::encode_input(
        payload.implementation,
        payload.transaction.to,
        payload.transaction.value,
        payload.transaction.data.clone(),
        payload.transaction.operation,
        payload.validators.clone(),
        payload.signatures.clone(),
        payload.nonce
    ).into();
    let factory_address = serde_json::from_value(serde_json::value::Value::String(factory_address()))?;
    log::debug!("factory: {}", factory_address);
    let call = Call {
        to: Some(factory_address),
        value: None,
        data: Some(data.clone()),
        gas: None,
        gas_price: None,
        from: Some(eth_provider.account()),
    };
    let options = CallOptions {
        block: "latest".to_string(),
    };
    log::debug!("call: {:?}", eth_provider.call(&call, &options)?);
    let estimate_result = to_string_result(eth_provider.estimate_gas(&call, &options)?)?;
    log::debug!("estimate_result: {}", estimate_result);
    let mut estimate = u64::from_str_radix(estimate_result.trim_start_matches("0x"), 16)?;
    estimate += estimate / 4;
    Ok(Estimation { wallet: factory_address, estimate: U256::from(estimate), data })
}

fn execute_with_estimation(eth_provider: &EthereumProvider, estimation: Estimation) -> Result<String> {
    let nonce_result = to_string_result(eth_provider.nonce()?)?;
    let nonce = u64::from_str_radix(nonce_result.trim_start_matches("0x"), 16)?;
    let tx = Transaction {
        to: Some(estimation.wallet),
        value: U256::zero(),
        data: &estimation.data,
        gas: estimation.estimate,
        gas_price: U256::from(1_000_000_000),
        nonce: U256::from(nonce),
    };
    //TODO check fee > gas * gas_price
    to_string_result(eth_provider.execute(&tx)?)
}

pub fn deploy(context: &Context, payload: DeployPayload) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);

    check_fee(
        &eth_provider,
        payload.transaction.to,
        payload.transaction.value,
        &payload.transaction.data.0,
        payload.transaction.operation,
    )?;

    let estimation = estimate(&eth_provider,&payload)?;

    Ok(execute_with_estimation(&eth_provider, estimation)?)
}
