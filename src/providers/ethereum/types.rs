//https://github.com/tomusdrw/rust-web3/blob/master/src/types/bytes.rs
use anyhow::Result;
use rustc_hex::{FromHex, ToHex};
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Raw bytes wrapper
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Bytes(pub Vec<u8>);

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut serialized = "0x".to_owned();
        serialized.push_str(self.0.to_hex::<String>().as_ref());
        write!(f, "{}", serialized)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(data: Vec<u8>) -> Self {
        Bytes(data.into())
    }
}

fn parse_hex_string(data: String) -> Result<Bytes> {
    if data.len() >= 2 && &data[0..2] == "0x" {
        let bytes = FromHex::from_hex(&data[2..])?;
        Ok(Bytes(bytes))
    } else {
        anyhow::bail!("Not hex prefixed string")
    }
}

impl From<String> for Bytes {
    fn from(data: String) -> Self {
        parse_hex_string(data).unwrap()
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(data: Bytes) -> Self {
        data.0
    }
}

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'a> Deserialize<'a> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_identifier(BytesVisitor)
    }
}

struct BytesVisitor;

impl<'a> Visitor<'a> for BytesVisitor {
    type Value = Bytes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a 0x-prefixed hex-encoded vector of bytes")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        parse_hex_string(value.to_string())
            .map_err(|_| Error::invalid_value(Unexpected::Str(value), &"Invalid hex"))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(value.as_ref())
    }
}
