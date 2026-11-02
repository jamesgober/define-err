//! `Code::parse` never panics, and every accepted text round-trips.
#![no_main]

use define_err::Code;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    if let Ok(code) = Code::parse(text) {
        assert!(text.len() <= 15);
        let shown = code.to_string();
        assert_eq!(Code::parse(&shown), Ok(code));
        assert!(shown.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    }
});
