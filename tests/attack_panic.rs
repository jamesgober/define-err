//! T9: a verb never unwinds into the code that only wanted to report, also
//! when the panic comes from the error's own `description()`, which the verb
//! calls to read the probe. One test: it installs a panic hook.

#![cfg(feature = "std")]

use std::{
    error::Error,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

use define_err::{Event, Kinds, Reporter, Severity};

#[derive(Debug)]
struct Hostile;

impl fmt::Display for Hostile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("hostile")
    }
}

impl Error for Hostile {
    fn description(&self) -> &str {
        panic!("a probe that panics");
    }
}

struct Counter;

static RECEIVED: AtomicUsize = AtomicUsize::new(0);

impl Reporter for Counter {
    fn receive(&self, _event: &Event<'_>) {
        let _ = RECEIVED.fetch_add(1, Ordering::Relaxed);
    }
}

static COUNTER: Counter = Counter;

#[test]
fn test_panicking_probe_never_reaches_the_caller() {
    std::panic::set_hook(Box::new(|_| {}));
    define_err::set_reporter(&COUNTER).unwrap();

    define_err::report(&Hostile);
    assert_eq!(define_err::stats().reporter_panics, 1);

    define_err::report_with(&Hostile, Severity::ERROR, Kinds::ERROR);
    assert_eq!(define_err::stats().reporter_panics, 2);

    assert!(define_err::soft(Hostile).is_ok());
    assert_eq!(define_err::stats().reporter_panics, 3);

    // The minimum severity's early read is a call to the same method.
    define_err::set_min_severity(Some(Severity::WARNING));
    define_err::report(&Hostile);
    assert_eq!(define_err::stats().reporter_panics, 4);
    define_err::set_min_severity(None);

    // Nothing was delivered, and the thread can still report.
    assert_eq!(RECEIVED.load(Ordering::Relaxed), 0);
    define_err::notice("still works");
    assert_eq!(RECEIVED.load(Ordering::Relaxed), 1);
    assert_eq!(define_err::stats().reentrant_dropped, 0);
}
