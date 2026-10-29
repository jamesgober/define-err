//! `Code::from_u64` accepts exactly valid values, and they round-trip.
#![no_main]

use define_err::Code;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|raw: u64| {
    if let Some(code) = Code::from_u64(raw) {
        assert_eq!(code.as_u64(), raw);
        assert_eq!(Code::parse(&code.to_string()).map(Code::identity), Ok(code.identity()));
        assert!(code.to_string().bytes().all(|b| b >= 0x20));
    }
});
