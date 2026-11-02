//! `capture` refuses to run when someone else's reporter is installed: an
//! empty list must never pass as the real one.

#![cfg(feature = "testing")]

use define_err::{testing::capture, Event, Reporter, Slot};

struct Other;

impl Reporter for Other {
    fn receive(&self, _event: &Event<'_>) {}
}

static OTHER: Other = Other;

#[test]
fn test_capture_refuses_a_foreign_reporter() {
    define_err::set_reporter(&OTHER).unwrap();
    let mut ran = false;
    let err = capture(|| ran = true).unwrap_err();
    assert_eq!(err.slot, Slot::Reporter);
    assert!(err.installed_at.is_some());
    assert!(!ran);
}
