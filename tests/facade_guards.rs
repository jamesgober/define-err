//! The `std` guards around a reporter call (E-80, E-81, T8).

#![cfg(feature = "std")]

use std::sync::atomic::{AtomicUsize, Ordering};

use define_err::{Event, Reporter};

/// Reports from inside `receive`, or panics, on request.
struct Unruly;

static RECEIVED: AtomicUsize = AtomicUsize::new(0);

struct BadDrop;

impl Drop for BadDrop {
    fn drop(&mut self) {
        panic!("a payload that panics on drop");
    }
}

impl Reporter for Unruly {
    fn receive(&self, event: &Event<'_>) {
        let _ = RECEIVED.fetch_add(1, Ordering::Relaxed);
        match event
            .message()
            .and_then(|message| message.as_str())
            .unwrap_or_default()
        {
            "nest" => {
                define_err::notice("nested");
                define_err::warn("nested warning");
                define_err::warn!("nested macro warning");
            }
            "panic" => panic!("reporter failed"),
            "bad payload" => std::panic::panic_any(BadDrop),
            _ => {}
        }
    }
}

fn nested_site_stays_usable() {
    define_err::warn!("a site reached from inside and outside");
}

static UNRULY: Unruly = Unruly;

#[test]
fn test_reentrant_verbs_are_dropped_and_panics_are_contained() {
    std::panic::set_hook(Box::new(|_| {}));
    define_err::set_reporter(&UNRULY).unwrap();

    define_err::notice!("nest");
    let received = RECEIVED.load(Ordering::Relaxed);
    assert_eq!(received, 1, "a nested verb reached the reporter");
    assert_eq!(define_err::stats().reentrant_dropped, 3);

    // E-81: the verb returns and the thread can report again.
    define_err::notice!("panic");
    assert_eq!(define_err::stats().reporter_panics, 1);
    define_err::notice!("bad payload");
    assert_eq!(
        define_err::stats().reporter_panics,
        3,
        "the panic and its payload's drop"
    );
    define_err::notice!("still works");
    assert_eq!(RECEIVED.load(Ordering::Relaxed), 4);

    // The guard flag came down after the panics.
    nested_site_stays_usable();
    assert_eq!(RECEIVED.load(Ordering::Relaxed), 5);
    assert_eq!(define_err::stats().reentrant_dropped, 3);
}
