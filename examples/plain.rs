//! A plain error type: what thiserror does, without a proc macro.
//!
//! ```text
//! cargo run --example plain
//! ```

use define_err::prelude::*;

define_errors! {
    /// Errors from loading the configuration.
    pub enum ConfigError {
        Missing { key: String } => "missing key {key}",
        BadPort(text: String) => "{text:?} is not a port number",
        Read(#[from] source: std::io::Error) => "cannot read the configuration: {source}",
        Auth { token: Secret<String> } => "token {token} rejected",
    }
}

fn load(path: &str) -> Result<String, ConfigError> {
    let text = std::fs::read_to_string(path)?;
    if text.is_empty() {
        return Err(ConfigError::Missing { key: "name".into() });
    }
    Ok(text)
}

fn main() {
    for err in [
        load("/does/not/exist").unwrap_err(),
        ConfigError::Missing { key: "name".into() },
        ConfigError::BadPort("eighty".into()),
        ConfigError::Auth {
            token: Secret::new("s3cr3t".into()),
        },
    ] {
        println!("{err}");
    }
}
