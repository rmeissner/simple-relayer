use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub mod cors;
pub mod context;
pub mod json;

pub fn hex_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    format!("{:#x}", s.finish())
}
