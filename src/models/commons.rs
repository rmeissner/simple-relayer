use serde_repr::{Deserialize_repr, Serialize_repr};
use serde::{Deserialize, Serialize};
use crate::utils::json;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, Hash)]
#[repr(u8)]
pub enum Operation {
    CALL = 0,
    DELEGATE = 1,
}
