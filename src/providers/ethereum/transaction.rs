use super::key::{PrivateKey, Signature};
use super::types::Bytes;
use super::hash;
use ethereum_types::{Address, U256};
use rlp::RlpStream;
use serde::Serialize;

/// Raw transaction data to sign
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Transaction<'a> {
    /// Nonce to use when signing this transaction.
    pub nonce: U256,
    /// Gas price to use when signing this transaction.
    pub gas_price: U256,
    /// Gas provided by the transaction.
    pub gas: U256,
    /// Receiver of the transaction.
    pub to: Option<Address>,
    /// Value of the transaction in wei.
    pub value: U256,
    /// Call data of the transaction, can be empty for simple value transfers.
    pub data: &'a Bytes,
}

impl<'a> Transaction<'a> {
    /// Sign and return a raw transaction.
    pub fn sign(&self, key: &PrivateKey, chain_id: Option<u64>) -> Bytes {
        let mut rlp = RlpStream::new();
        self.rlp_append_unsigned(&mut rlp, chain_id);

        let hash = hash::keccak256(&rlp.as_raw());
        rlp.clear();

        let sig = key.sign(&hash);
        self.rlp_append_signed(&mut rlp, sig, chain_id);

        rlp.out().into()
    }

    /// RLP encode an unsigned transaction.
    fn rlp_append_unsigned(&self, s: &mut RlpStream, chain_id: Option<u64>) {
        s.begin_list(if chain_id.is_some() { 9 } else { 6 });
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas);
        if let Some(to) = self.to {
            s.append(&to);
        } else {
            s.append(&"");
        }
        s.append(&self.value);
        s.append(&self.data.0);
        if let Some(n) = chain_id {
            s.append(&n);
            s.append(&0u8);
            s.append(&0u8);
        }
    }

    /// RLP encode a transaction with its signature.
    fn rlp_append_signed(&self, s: &mut RlpStream, sig: Signature, chain_id: Option<u64>) {
        let sig_v = add_chain_replay_protection(sig.v, chain_id);
        s.begin_list(9);
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas);
        if let Some(to) = self.to {
            s.append(&to);
        } else {
            s.append(&"");
        }
        s.append(&self.value);
        s.append(&self.data.0);
        s.append(&sig_v);
        s.append(&U256::from(sig.r));
        s.append(&U256::from(sig.s));
    }
}

/// Encode chain ID based on (EIP-155)[https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
fn add_chain_replay_protection(v: u64, chain_id: Option<u64>) -> u64 {
    v + if let Some(n) = chain_id {
        35 + n * 2
    } else {
        27
    }
}
