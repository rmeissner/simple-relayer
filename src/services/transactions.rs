use crate::config::{multisend_address, transaction_fee};
use crate::models::{ExecutePayload, PreparePayload, PrepareResult, SafeTransaction};
use crate::providers::accounts::safe::SafeAccount;
use crate::providers::accounts::vault::{VaultAccount, VaultPayload, VaultConfigPayload, VaultConfigFee};
use crate::providers::accounts::{check_fee, check_payment_tx, Account, Estimation};
use crate::providers::ethereum::transaction::Transaction;
use crate::providers::ethereum::types::Bytes;
use crate::providers::ethereum::{to_string_result, EthereumProvider};
use crate::utils::context::Context;
use anyhow::Result;
use ethabi;
use ethabi::{ParamType, Token};
use ethabi_contract::use_contract;
use ethereum_types::{Address, U256};
use serde_json;

// https://github.com/openethereum/ethabi/blob/master/tests/src/lib.rs
use_contract!(multisend, "./res/multisend.json");

pub fn prepare(context: &Context, payload: PreparePayload) -> Result<PrepareResult> {
    let fee = U256::from_dec_str(&transaction_fee())?;
    if fee == U256::zero() {
        // Nothing to prepare
        return Ok(PrepareResult {
            fee,
            fee_receiver: Address::zero(),
            transaction: SafeTransaction {
                to: payload.to,
                value: payload.value,
                data: payload.data,
                operation: payload.operation,
                safe_tx_gas: U256::zero(),
            },
        });
    }

    let eth_provider = EthereumProvider::new(context);
    let relayer = eth_provider.account();
    // We rewrite the transaction to a multisend that performs the transaction and then pays for the transaction
    // First execute the user transction (maybe the account receives coins)
    let tx_1 = build_multisend_bytes(
        &[payload.operation],
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
    Ok(PrepareResult {
        fee,
        fee_receiver: relayer,
        transaction: SafeTransaction {
            to: serde_json::from_value(serde_json::value::Value::String(multisend_address()))?,
            value: U256::from(0),
            data: multisend::functions::multi_send::encode_input(multisend_data).into(),
            operation: 1,
            safe_tx_gas: U256::zero(),
        },
    })
}

fn build_multisend_bytes(opration: &[u8], address: &[u8], value: &[u8], data: &Vec<u8>) -> Vec<u8> {
    let data_len: &[u8] = &ethabi::encode(&[ethabi::Token::Uint(U256::from(data.len()))]);
    [opration, address, value, data_len, data].concat()
}

fn execute_with_estimation(
    eth_provider: &EthereumProvider,
    estimation: Estimation,
) -> Result<String> {
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

pub fn execute_safe(context: &Context, payload: ExecutePayload) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);

    check_fee(
        &eth_provider,
        payload.transaction.to,
        payload.transaction.value,
        &payload.transaction.data.0,
        payload.transaction.operation,
    )?;

    let account = SafeAccount {
        eth_provider: &eth_provider,
    };
    let estimation = account.estimate(&payload)?;

    Ok(execute_with_estimation(&eth_provider, estimation)?)
}

pub fn execute_vault(context: &Context, payload: VaultPayload) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);

    check_fee(
        &eth_provider,
        payload.transaction.to,
        payload.transaction.value,
        &payload.transaction.data.0,
        payload.transaction.operation,
    )?;

    let account = VaultAccount {
        eth_provider: &eth_provider,
    };
    let estimation = account.estimate(&payload)?;

    Ok(execute_with_estimation(&eth_provider, estimation)?)
}

pub fn update_vault(context: &Context, payload: VaultConfigPayload) -> Result<String> {
    let eth_provider = EthereumProvider::new(context);

    let fee = U256::from_dec_str(&transaction_fee())?;
    if fee != U256::zero() {
        let hook_parts = ethabi::decode(&[ParamType::Address, ParamType::Uint(256), ParamType::Bytes, ParamType::Uint(8)], &payload.hook.0)?;
        let decoded_hook = SafeTransaction {
            to: if let Token::Address(v) = hook_parts[0] { v } else { anyhow::bail!("Could not decode hook") },
            value: if let Token::Uint(v) = hook_parts[0] { v } else { anyhow::bail!("Could not decode hook") },
            data: if let Token::Bytes(v) = &hook_parts[0] { Bytes(v.clone()) } else { anyhow::bail!("Could not decode hook") },
            operation:  if let Token::Uint(v) = hook_parts[0] { v.byte(0) } else { anyhow::bail!("Could not decode hook") },
            safe_tx_gas: U256::zero()
        };
        check_payment_tx(
            &eth_provider,
            &decoded_hook,
            fee
        )?;
    }

    let account = VaultAccount {
        eth_provider: &eth_provider,
    };
    let estimation = account.estimate_config_update(&payload)?;

    Ok(execute_with_estimation(&eth_provider, estimation)?)
}

pub fn update_vault_hook(context: &Context) -> Result<VaultConfigFee> {
    let fee = U256::from_dec_str(&transaction_fee())?;
    if fee == U256::zero() {
        // Nothing to prepare
        return Ok(VaultConfigFee {
            fee,
            fee_receiver: Address::zero(),
            hook: Bytes(vec![])
        });
    }
    let eth_provider = EthereumProvider::new(context);
    let relayer = eth_provider.account();
    let hook = ethabi::encode(
        &[Token::Address(relayer.clone()), Token::Uint(fee), Token::Bytes(vec![]), Token::Uint(U256::zero())]
    );
    Ok(VaultConfigFee {
        fee,
        fee_receiver: relayer,
        hook: Bytes(hook)
    })
}