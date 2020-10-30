use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use rocket::http::uri::Absolute;

pub mod cors;
pub mod context;
pub mod cache;
pub mod json;

pub fn hex_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    format!("{:#x}", s.finish())
}

//TODO verify we are only touching 'offset' and 'limit'
pub fn extract_query_string(raw_link: &String) -> Option<String> {
    let parsed = Absolute::parse(raw_link).ok()?;
    parsed.origin()?.query().map(|it| it.to_string())
}
