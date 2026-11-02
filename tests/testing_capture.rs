//! `testing::capture` (TST-01, TST-02).

#![cfg(feature = "testing")]

use std::thread;

use define_err::{define_errors, testing::capture, Kinds, Severity, Verb};

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, help = "Raise pool.size.")]
        #[category(DeadlineExceeded)]
        Timeout => "query timed out",
    }
}

#[test]
fn test_capture_returns_result_and_events() {
    let line = line!() + 2;
    let (value, events) = capture(|| {
        define_err::notice("started");
        define_err::report(&DbError::Timeout);
        7
    })
    .unwrap();
    assert_eq!(value, 7);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].verb, Verb::Notice);
    assert_eq!(events[0].message, "started");
    assert_eq!(events[0].kinds, Kinds::NOTICE);
    assert_eq!(events[0].severity, Severity::NOTICE);
    assert_eq!((events[0].file, events[0].line), (file!(), line));
    assert!(events[0].column > 0);
    assert!(events[0].record.is_none());
    assert_eq!(events[1].message, "query timed out");
    assert_eq!(events[1].code.unwrap().to_string(), "DB-0001");
    assert_eq!(events[1].record.as_ref().unwrap().chain.len(), 1);
}

#[test]
fn test_capture_sees_only_its_own_thread() {
    let ((), first) = capture(|| {
        define_err::notice("mine");
        thread::spawn(|| define_err::notice("another thread's"))
            .join()
            .unwrap();
    })
    .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].message, "mine");
}

#[test]
fn test_parallel_captures_stay_apart() {
    thread::scope(|scope| {
        for i in 0..8 {
            let _ = scope.spawn(move || {
                let ((), events) = capture(|| define_err::notice(&format!("thread {i}"))).unwrap();
                assert_eq!(events.len(), 1);
                assert_eq!(events[0].message, format!("thread {i}"));
            });
        }
    });
}

#[test]
fn test_nested_capture_keeps_both_lists() {
    let (inner, outer) = capture(|| {
        define_err::notice("outer one");
        let (((), inner), ()) = (capture(|| define_err::notice("inner")).unwrap(), ());
        define_err::notice("outer two");
        inner
    })
    .unwrap();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner[0].message, "inner");
    let outer: Vec<&str> = outer.iter().map(|e| e.message.as_str()).collect();
    assert_eq!(outer, ["outer one", "outer two"]);
}

#[test]
fn test_events_outside_a_capture_go_nowhere() {
    let ((), events) = capture(|| {}).unwrap();
    define_err::notice("nobody is capturing");
    assert!(events.is_empty());
    let ((), later) = capture(|| {}).unwrap();
    assert!(later.is_empty());
}

#[test]
fn test_capture_ends_cleanly_when_the_closure_panics() {
    let outcome = thread::spawn(|| {
        let result = std::panic::catch_unwind(|| capture(|| panic!("in the test")));
        assert!(result.is_err());
        define_err::notice("after the panic");
        capture(|| ()).unwrap().1.len()
    })
    .join()
    .unwrap();
    assert_eq!(outcome, 0);
}
