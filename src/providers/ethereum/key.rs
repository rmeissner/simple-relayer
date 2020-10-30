// https://github.com/gnosis/ethcontract-rs/blob/main/src/secret.rs
//! This module implements secrets in the form of protected memory.

use super::hash;
use thiserror::Error;
use secp256k1::key::ONE_KEY;
use secp256k1::Error as Secp256k1Error;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use ethereum_types::Address;
use zeroize::{DefaultIsZeroes, Zeroizing};

/// An error indicating an invalid private key. Private keys for secp256k1 must
/// be exactly 32 bytes and fall within the range `[1, n)` where `n` is the
/// order of the generator point of the curve.
#[derive(Debug, Error)]
#[error("invalid private key")]
pub struct InvalidPrivateKey;

pub struct Signature {
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u64
}

impl From<Secp256k1Error> for InvalidPrivateKey {
    fn from(err: Secp256k1Error) -> Self {
        match err {
            Secp256k1Error::InvalidSecretKey => {}
            _ => {
                // NOTE: Assert that we never try to make this conversion with
                //   errors not related to `SecretKey`.
                debug_assert!(false, "invalid conversion to InvalidPrivateKey error");
            }
        }
        InvalidPrivateKey
    }
}

/// A secret key used for signing and hashing.
///
/// This type has a safe `Debug` implementation that does not leak information.
/// Additionally, it implements `Drop` to zeroize the memory to make leaking
/// passwords less likely.
#[derive(Clone)]
pub struct PrivateKey(Zeroizing<ZeroizeableSecretKey>);

impl PrivateKey {
    /// Creates a new private key from raw bytes.
    pub fn from_raw(raw: [u8; 32]) -> Result<Self, InvalidPrivateKey> {
        PrivateKey::from_slice(&raw)
    }

    /// Creates a new private key from a slice of bytes.
    pub fn from_slice<B: AsRef<[u8]>>(raw: B) -> Result<Self, InvalidPrivateKey> {
        let secret_key = SecretKey::from_slice(raw.as_ref())?;
        Ok(PrivateKey(Zeroizing::new(secret_key.into())))
    }

    /// Creates a new private key from a hex string representation. Accepts hex
    /// string with or without leading `"0x"`.
    pub fn from_hex_str<S: AsRef<str>>(s: S) -> Result<Self, InvalidPrivateKey> {
        let hex_str = {
            let s = s.as_ref();
            if s.starts_with("0x") {
                &s[2..]
            } else {
                s
            }
        };
        let secret_key = SecretKey::from_str(hex_str)?;
        Ok(PrivateKey(Zeroizing::new(secret_key.into())))
    }

    /// Gets the public address for a given private key.
    pub fn public_address(&self) -> Address {
        let secp = Secp256k1::signing_only();
        let public_key = PublicKey::from_secret_key(&secp, &*self).serialize_uncompressed();

        // NOTE: An ethereum address is the last 20 bytes of the keccak hash of
        //   the public key. Note that `libsecp256k1` public key is serialized
        //   into 65 bytes as the first byte is always 0x04 as a tag to mark a
        //   uncompressed public key. Discard it for the public address
        //   calculation.
        debug_assert_eq!(public_key[0], 0x04);
        let hash = hash::keccak256(&public_key[1..]);

        Address::from_slice(&hash[12..])
    }

    pub fn sign(&self, hash: &[u8]) -> Signature {
        let message = Message::from_slice(&hash).expect("hash is an invalid secp256k1 message");
        let (recovery_id, sig) = Secp256k1::signing_only()
            .sign_recoverable(&message, &self)
            .serialize_compact();
        let (sig_r, sig_s) = {
            let (mut r, mut s) = ([0u8; 32], [0u8; 32]);
            r.copy_from_slice(&sig[..32]);
            s.copy_from_slice(&sig[32..]);
            (r, s)
        };
        Signature {
            r: sig_r,
            s: sig_s,
            v: (recovery_id.to_i32() as u64)
        }
    }
}

impl FromStr for PrivateKey {
    type Err = InvalidPrivateKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PrivateKey::from_hex_str(s)
    }
}

impl Deref for PrivateKey {
    type Target = SecretKey;

    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("PrivateKey")
            .field(&self.public_address())
            .finish()
    }
}

/// An internal type that allows us to implement `Zeroize` on `SecretKey`. This
/// allows `PrivateKey` to correctly zeroize (almost, we use the `ONE_KEY`
/// instead of `0`s since it is the first valid key) in a way that does not get
/// optimized away by the compiler or get access reordered.
///
/// For more information, consult the `zeroize` crate
/// [`README`](https://github.com/iqlusioninc/crates/tree/develop/zeroize).
#[derive(Clone, Copy)]
struct ZeroizeableSecretKey(SecretKey);

impl From<SecretKey> for ZeroizeableSecretKey {
    fn from(secret_key: SecretKey) -> Self {
        ZeroizeableSecretKey(secret_key)
    }
}

impl Default for ZeroizeableSecretKey {
    fn default() -> Self {
        ONE_KEY.into()
    }
}

impl DefaultIsZeroes for ZeroizeableSecretKey {}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroize;

    #[test]
    fn private_key_address() {
        // retrieved test vector from both (since the two cited examples use the
        // same message and key - as the hashes and signatures match):
        // https://web3js.readthedocs.io/en/v1.2.5/web3-eth-accounts.html#sign
        // https://web3js.readthedocs.io/en/v1.2.5/web3-eth-accounts.html#recover
        let key = key!("0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318");
        let address = addr!("0x2c7536E3605D9C16a7a3D7b1898e529396a65c23");

        assert_eq!(key.public_address(), address);
    }

    #[test]
    fn drop_private_key() {
        let mut key = key!("0x0102030405060708091011121314151617181920212223242526272829303132");
        key.0.zeroize();
        assert_eq!(*key, ONE_KEY);
    }
}