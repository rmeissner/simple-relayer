use crate::config::{multisend_address, transaction_fee};
use crate::models::service::{ExecutePayload, PreparePayload, SafeTransaction};
use crate::providers::ethereum::transaction::Transaction;
use crate::providers::accounts::{Account, check_fee, Estimation};
use crate::providers::accounts::safe::SafeAccount;
use crate::providers::ethereum::{to_string_result, EthereumProvider};
use crate::utils::context::Context;
use anyhow::Result;
use ethabi;
use ethabi_contract::use_contract;
use ethereum_types::{U256};
use serde_json;

// https://github.com/openethereum/ethabi/blob/master/tests/src/lib.rs
use_contract!(multisend, "./res/multisend.json");

pub fn prepare(context: &Context, payload: PreparePayload) -> Result<SafeTransaction> {
    let fee = U256::from_dec_str(&transaction_fee())?;
    if fee == U256::zero() {
        // Nothing to prepare
        return Ok(SafeTransaction {
            to: payload.to,
            value: payload.value,
            data: payload.data,
            operation: 0,
            safe_tx_gas: U256::zero(),
        });
    }

    let eth_provider = EthereumProvider::new(context);
    let relayer = eth_provider.account();
    // We rewrite the transaction to a multisend that performs the transaction and then pays for the transaction
    // First execute the user transction (maybe the account receives coins)
    let tx_1 = build_multisend_bytes(
        &[0u8],
        &payload.to.to_fixed_bytes(),
        &ethabi::encode(&[ethabi::Token::Uint(payload.value)]),
        &payload.data.0,
    );
    // Second pay the fee
    let tx_2 = build_multisend_bytes(
        &[0u8],
        &relayer.to_fixed_bytes(),
        &ethabi::encode(&[ethabi::Token::Uint(fee)]),
        &vec![],
    );
    let multisend_data = vec![tx_1, tx_2].concat();
    Ok(SafeTransaction {
        to: serde_json::from_value(serde_json::value::Value::String(multisend_address()))?,
        value: U256::from(0),
        data: multisend::functions::multi_send::encode_input(multisend_data).into(),
        operation: 1,
        safe_tx_gas: U256::zero(),
    })
}

fn build_multisend_bytes(opration: &[u8], address: &[u8], value: &[u8], data: &Vec<u8>) -> Vec<u8> {
    let data_len: &[u8] = &ethabi::encode(&[ethabi::Token::Uint(U256::from(data.len()))]);
    [opration, address, value, data_len, data].concat()
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

pub fn execute(context: &Context, payload: ExecutePayload) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);

    check_fee(
        &eth_provider,
        payload.transaction.to,
        payload.transaction.value,
        &payload.transaction.data.0,
        payload.transaction.operation,
    )?;

    let account = SafeAccount { eth_provider: &eth_provider };
    let estimation = account.estimate(&payload)?;

    //TODO check fee > gas * gas_price´
    Ok(execute_with_estimation(&eth_provider, estimation)?)
}
