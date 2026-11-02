//! `Category::parse` never panics, and what it accepts round-trips.
#![no_main]

use define_err::Category;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    if let Some(category) = Category::parse(text) {
        assert_eq!(Category::parse(category.name()), Some(category));
    }
});
