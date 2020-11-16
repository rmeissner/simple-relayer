extern crate reqwest;

use crate::config::{version, build_number};
use anyhow::Result;
use crate::models::About;

pub fn get_about() -> Result<About> {
    Ok(About {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: version(),
        build_number: build_number(),
    })
}
