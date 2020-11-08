use crate::models::service::{SafeTransaction};
use crate::providers::ethereum::types::Bytes;
use ethereum_types::{Address, U256};
use ethabi_contract::use_contract;
use std::convert::TryInto;

use_contract!(multisend, "./res/multisend.json");

pub fn decode_multisend_bytes(bytes: &Vec<u8>) -> Vec<SafeTransaction> {
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