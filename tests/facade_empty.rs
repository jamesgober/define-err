//! The facade with nothing installed, then the first installation (E-65,
//! E-68, E-77, E-82). One test: the order matters, and the slot is write-once.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

use define_err::{define_errors, Event, Kinds, Reporter, WarnMode};

define_errors! {
    pub struct Refused => "refused";
}

/// Panics if the facade formats anything while nothing is installed.
struct NeverFormat;

impl core::fmt::Display for NeverFormat {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        panic!("formatted with nothing installed");
    }
}

struct Counter {
    flushes: AtomicUsize,
    messages: Mutex<Vec<String>>,
}

impl Reporter for Counter {
    fn receive(&self, event: &Event<'_>) {
        if let Some(message) = event.message() {
            self.messages.lock().unwrap().push(message.to_string());
        }
    }

    fn flush(&self) {
        let _ = self.flushes.fetch_add(1, Ordering::Relaxed);
    }
}

static COUNTER: Counter = Counter {
    flushes: AtomicUsize::new(0),
    messages: Mutex::new(Vec::new()),
};

fn stale() {
    define_err::warn!("stale cache");
}

fn stale_function() {
    define_err::warn("stale function");
}

#[test]
fn test_nothing_installed_then_first_install() {
    // E-65: every verb returns, and nothing is formatted.
    assert!(!define_err::is_installed());
    assert!(define_err::reporter().is_none());
    assert!(define_err::reporter_installed_at().is_none());
    define_err::report(&Refused);
    define_err::report_with(&Refused, define_err::Severity::FATAL, Kinds::ALL);
    define_err::notice("nobody hears this");
    define_err::notice!("{}", NeverFormat);
    define_err::trigger(Kinds::SECURITY, "nobody hears this");
    define_err::trigger!(Kinds::SECURITY, "{}", NeverFormat);
    define_err::deprecated("nobody");
    assert!(define_err::soft(Refused).is_ok());

    // E-82: flush with nothing installed.
    define_err::flush();

    // E-77: strict mode refuses without a reporter too.
    define_err::set_warn_mode(WarnMode::Error);
    assert!(define_err::soft(Refused).is_err());
    define_err::set_warn_mode(WarnMode::Once);

    // E-68: warnings before installation keep their sites.
    stale();
    stale_function();
    define_err::set_reporter(&COUNTER).unwrap();
    stale();
    stale();
    stale_function();
    stale_function();
    assert_eq!(
        *COUNTER.messages.lock().unwrap(),
        ["stale cache", "stale function"]
    );

    // E-82: flush reaches the reporter.
    define_err::flush();
    assert_eq!(COUNTER.flushes.load(Ordering::Relaxed), 1);
}
