pub mod hash;
pub mod key;
pub mod types;
pub mod transaction;

use crate::config::{key_bytes};
use crate::utils::context::Context;
use transaction::Transaction;
use ethereum_types::{Address};

pub struct RpcCall {

}

pub struct EthereumProvider<'p> {
    client: &'p reqwest::blocking::Client
}

impl EthereumProvider<'_> {
    pub fn new<'p>(context: &'p Context) -> EthereumProvider<'p> {
        EthereumProvider {
            client: context.client()
        }
    }

    fn get_key(&self) -> key::PrivateKey {
        key::PrivateKey::from_hex_str(key_bytes()).unwrap()
    }

    pub fn account(&self) -> Address {
        self.get_key().public_address()
    }

    fn call(
        &self,
        transaction: &'_ Transaction
    ) -> String {
        "".to_string()
    }

    fn execute(
        &self,
        transaction: &'_ Transaction
    ) -> String {
        let signed = transaction.sign(&self.get_key(), None);
        signed.to_string()
    }
}