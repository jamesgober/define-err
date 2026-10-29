//! The identity-probe decoder never panics and never accepts an invalid field.
#![no_main]

use define_err::{identify, Code};
use libfuzzer_sys::fuzz_target;

#[derive(Debug)]
struct Forged(String);

impl std::fmt::Display for Forged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("forged")
    }
}

impl std::error::Error for Forged {
    fn description(&self) -> &str {
        &self.0
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    let forged = Forged(text.to_owned());
    if let Some(identity) = identify(&forged) {
        assert!((1..=24).contains(&identity.severity.get()));
        if let Some(code) = identity.code {
            assert_eq!(Code::from_u64(code.as_u64()), Some(code));
        }
        assert!(!identity.type_name.is_empty() && identity.type_name.len() <= 128);
        assert!(identity.type_name.bytes().all(|b| b.is_ascii_graphic()));
    }
});
