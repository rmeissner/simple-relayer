use crate::config::{multisend_address, transaction_fee};
use crate::models::service::{ExecutePayload, PreparePayload, SafeTransaction};
use crate::providers::ethereum::transaction::Transaction;
use crate::providers::ethereum::types::Bytes;
use crate::providers::ethereum::{to_string_result, Call, CallOptions, EthereumProvider};
use crate::utils::context::Context;
use anyhow::Result;
use ethabi;
use ethabi_contract::use_contract;
use ethereum_types::{Address, U256};
use serde_json;
use std::convert::TryInto;

// https://github.com/openethereum/ethabi/blob/master/tests/src/lib.rs
use_contract!(safe, "./res/safe.json");
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

fn decode_multisend_bytes(bytes: &Vec<u8>) -> Vec<SafeTransaction> {
    let mut txs = vec![];
    let multisend_start = &multisend::functions::multi_send::encode_input(vec![])[0..36];
    if multisend_start != &bytes[0..36] { panic!("Invalid multisend bytes") }
    let multisend_data_length = U256::from(&bytes[36..68]).as_usize();
    if multisend_data_length + 68 > bytes.len() { panic!("Invalid multisend data length") }
    let mut bytes_index = 68;
    // We should have always at least 85 bytes of data (check to avoid running into padding)
    while bytes_index + 85 <= bytes.len() {
        let operation_bytes = bytes[bytes_index];
        bytes_index += 1;
        let address_bytes: &[u8; 20] = bytes[bytes_index..bytes_index + 20]
            .try_into()
            .expect("20 bytes for address");
        bytes_index += 20;
        let value_bytes: &[u8; 32] = bytes[bytes_index..bytes_index + 32]
            .try_into()
            .expect("32 bytes for value");
        bytes_index += 32;
        let data_length_bytes: &[u8; 32] = bytes[bytes_index..bytes_index + 32]
            .try_into()
            .expect("32 bytes for data length");
        bytes_index += 32;
        let data_length = U256::from(data_length_bytes).as_usize();
        log::debug!("data_length: {}", data_length);
        let data_bytes: &[u8] = &bytes[bytes_index..bytes_index+data_length];
        bytes_index += data_length;
        txs.push(SafeTransaction {
            operation: operation_bytes,
            to: Address::from(address_bytes),
            value: U256::from(value_bytes),
            data: Bytes(data_bytes.to_vec()),
            safe_tx_gas: U256::zero(),
        });
    }
    txs
}

pub fn execute(context: &Context, payload: ExecutePayload) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);
    let fee = U256::from_dec_str(&transaction_fee())?;
    if fee != U256::zero() {
        // Check payment
        let ms_address =
            serde_json::from_value(serde_json::value::Value::String(multisend_address()))?;
        anyhow::ensure!(
            payload.transaction.to == ms_address,
            "Fee payment requires multisend"
        );
        anyhow::ensure!(
            payload.transaction.operation == 1,
            "Multisend requires delegatecall"
        );
        anyhow::ensure!(
            payload.transaction.value == U256::zero(),
            "Delegate call should not contain value"
        );
        let txs = decode_multisend_bytes(&payload.transaction.data.0);
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
    }
    // TODO check wallet
    let mut call = Call {
        to: Some(payload.wallet),
        value: None,
        data: Some(
            safe::functions::exec_transaction::encode_input(
                payload.transaction.to,
                payload.transaction.value,
                payload.transaction.data,
                payload.transaction.operation,
                payload.transaction.safe_tx_gas,
                0,
                0,
                Address::zero(),
                Address::zero(),
                payload.signatures,
            )
            .into(),
        ),
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
    let mut success = false;
    while !success && estimate < 20_000_000 {
        call.gas = Some(U256::from(estimate));
        let simulate_result = eth_provider.call(&call, &options)?;
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
    let nonce_result = to_string_result(eth_provider.nonce()?)?;
    let nonce = u64::from_str_radix(nonce_result.trim_start_matches("0x"), 16)?;
    let tx = Transaction {
        to: call.to,
        value: call.value.unwrap_or(U256::zero()),
        data: &call.data.unwrap(),
        gas: U256::from(estimate),
        gas_price: U256::from(1_000_000_000),
        nonce: U256::from(nonce),
    };
    //TODO check fee > gas * gas_price
    Ok(to_string_result(eth_provider.execute(&tx)?)?)
}
