//! `Severity::parse` never panics, and accepts only 1 to 24.
#![no_main]

use define_err::Severity;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    if let Some(severity) = Severity::parse(text) {
        assert!((1..=24).contains(&severity.get()));
        assert_eq!(Severity::parse(&severity.to_string()), Some(severity));
    }
});
