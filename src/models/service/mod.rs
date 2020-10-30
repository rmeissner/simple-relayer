use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct About {
    pub transaction_service_base_url: String,
    pub name: String,
    pub version: String,
    pub build_number: Option<String>
}