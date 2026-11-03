//! A kinds value from the wire never prints a control byte, and its bits
//! survive `from_bits`.
#![no_main]

use define_err::Kinds;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bits: u64| {
    let kinds = Kinds::from_bits(bits);
    assert_eq!(kinds.bits(), bits);
    assert!(kinds.to_string().bytes().all(|b| b >= 0x20 && b != 0x7f));
    assert!(format!("{kinds:?}").bytes().all(|b| b >= 0x20 && b != 0x7f));
    assert_eq!(kinds.iter().fold(Kinds::NONE, |all, one| all | one), kinds);
});
