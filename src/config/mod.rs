use std::env;

pub fn base_rpc_url() -> String {
    env::var("RPC_URL").unwrap()
}

pub fn transaction_fee() -> String {
    env::var("TRANSACTION_FEE").unwrap_or("0".to_string())
}

pub fn multisend_address() -> String {
    env::var("MULTISEND_ADDRESS").unwrap()
}

pub fn factory_address() -> String {
    env::var("FACTORY_ADDRESS").unwrap()
}

pub fn key_bytes() -> String {
    env::var("KEY_BYTES").unwrap()
}

pub fn scheme() -> String {
    env::var("SCHEME").unwrap_or(String::from("https"))
}

fn usize_with_default(key: &str, default: usize) -> usize {
    match env::var(key) {
        Ok(value) => value.parse().unwrap(),
        Err(_) => default
    }
}

pub fn build_number() -> Option<String> {
    option_env!("BUILD_NUMBER").map(|it| it.to_string())
}

pub fn version() -> String {
    option_env!("VERSION").unwrap_or(env!("CARGO_PKG_VERSION")).to_string()
}