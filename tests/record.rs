//! `ErrorRecord`: bounds, chains, and events (E-90 to E-95).

#![cfg(feature = "alloc")]

use core::fmt;
use std::{borrow::Cow, error::Error};

use define_err::{
    define_errors, here, Category, ErrorRecord, Event, Kinds, RecordLimits, Secret, Severity,
    TrailEntry, TrailStep, Verb,
};

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, help = "Raise pool.size.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, help = "Restore the page.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        Corrupt => "page failed its checksum",

        Token { token: Secret<String> } => "token {token} rejected",
    }

    pub enum AppError {
        Store(#[from] inner: DbError) => "storage failed",
    }
}

/// A chain link with no probe.
#[derive(Debug)]
struct Link(Option<Box<dyn Error + 'static>>);

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("link")
    }
}

impl Error for Link {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.as_deref()
    }
}

fn chain(depth: usize) -> Box<dyn Error + 'static> {
    let mut error: Box<dyn Error + 'static> = Box::new(Link(None));
    for _ in 0..depth {
        error = Box::new(Link(Some(error)));
    }
    error
}

/// `source()` returns itself.
#[derive(Debug)]
struct Loop;

impl fmt::Display for Loop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("loop")
    }
}

impl Error for Loop {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

#[derive(Debug)]
struct Failing;

impl fmt::Display for Failing {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
}

impl Error for Failing {}

/// Ignores write errors and keeps writing.
#[derive(Debug)]
struct Stubborn;

impl fmt::Display for Stubborn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..10_000 {
            let _ = f.write_str("0123456789");
        }
        Ok(())
    }
}

impl Error for Stubborn {}

#[test]
fn test_header_comes_from_the_outermost_coded_entry() {
    let err = AppError::from(DbError::Timeout { after_ms: 5000 });
    let record = ErrorRecord::from_error(&err);
    assert_eq!(record.code.unwrap().to_string(), "DB-0001");
    assert_eq!(record.category, Category::DeadlineExceeded);
    assert_eq!(record.public, "The database is busy. Try again shortly.");
    assert_eq!(record.chain.len(), 2);
    assert_eq!(record.chain[0].message, "storage failed");
    assert_eq!(record.chain[0].code, None);
    assert_eq!(record.chain[0].type_name, "AppError::Store");
    assert_eq!(record.chain[1].type_name, "DbError::Timeout");
    assert_eq!(
        record.to_string(),
        "[DB-0001] storage failed: caused by query timed out after 5000 ms"
    );
    assert!(!record.truncated);
    assert!(record.handled);
}

#[test]
fn test_public_defaults_to_the_category_and_never_to_display() {
    let record = ErrorRecord::from_error(&DbError::Corrupt);
    assert_eq!(record.public, Category::DataLoss.default_public_message());
    assert!(!record.public.contains("checksum"));
    let foreign = ErrorRecord::from_error(&Link(None));
    assert_eq!(foreign.category, Category::Unknown);
    assert_eq!(foreign.chain[0].type_name, "unknown");
    assert_eq!(foreign.public, Category::Unknown.default_public_message());
}

/// E-90: chain depth.
#[test]
fn test_chain_depth_limits() {
    for (depth, entries, truncated) in [
        (0, 1, false),
        (1, 2, false),
        (31, 32, false),
        (32, 32, true),
        (33, 32, true),
    ] {
        let record = ErrorRecord::from_error(&*chain(depth));
        assert_eq!(record.chain.len(), entries, "depth {depth}");
        assert_eq!(record.truncated, truncated, "depth {depth}");
    }
    let looped = ErrorRecord::from_error(&Loop);
    assert_eq!(looped.chain.len(), 32);
    assert!(looped.truncated);
}

#[test]
fn test_self_reference_ends_when_a_limit_keeps_no_entry() {
    let mut limits = RecordLimits::DEFAULT;
    limits.max_message_bytes = 0;
    let record = ErrorRecord::from_error_with(&Loop, &limits);
    assert!(record.chain.is_empty());
    assert!(record.truncated);
    limits = RecordLimits::DEFAULT;
    limits.max_chain = 0;
    assert!(ErrorRecord::from_error_with(&DbError::Corrupt, &limits)
        .chain
        .is_empty());
}

/// E-91: the message limit.
#[test]
fn test_message_limit_on_character_boundaries() {
    let mut limits = RecordLimits::DEFAULT;
    limits.max_message_bytes = 5;
    let make = |text: &'static str| {
        let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR)
            .with_limits(limits);
        let kept = record.push_chain(&text, None, "t");
        (record.chain[0].message.clone(), kept, record.truncated)
    };
    assert_eq!(make("abcde"), ("abcde".to_string(), true, false));
    assert_eq!(make("abcdef"), ("abcde".to_string(), false, true));
    assert_eq!(make("abcdé"), ("abcd".to_string(), false, true));
}

/// E-92: a failing `Display`.
#[test]
fn test_failing_display_is_marked() {
    let record = ErrorRecord::from_error(&Failing);
    assert_eq!(record.chain[0].message, "<message unavailable>");
    assert!(!record.truncated);
}

#[test]
fn test_display_that_ignores_errors_cannot_grow_past_the_limit() {
    let record = ErrorRecord::from_error(&Stubborn);
    assert_eq!(record.chain[0].message.len(), 1024);
    assert!(record.truncated);
}

/// E-93: field limits.
#[test]
fn test_field_limits() {
    let mut limits = RecordLimits::DEFAULT;
    limits.max_fields = 1;
    limits.max_field_bytes = 4;
    let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR)
        .with_limits(limits);
    assert!(record.push_field("key", &"abc"));
    assert!(!record.truncated);
    assert!(!record.push_field("more", &1));
    assert!(record.truncated);
    assert_eq!(record.fields.len(), 1);

    limits.max_fields = 4;
    let mut cut = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR)
        .with_limits(limits);
    assert!(!cut.push_field("k", &"abcé"));
    assert_eq!(cut.fields[0].1, "abc");
    assert!(!cut.push_field(String::from("a long key"), &1));
    assert_eq!(cut.fields[1].0, Cow::Borrowed("a lo"));
}

#[test]
fn test_total_limit_bounds_all_text() {
    let mut limits = RecordLimits::DEFAULT;
    limits.max_total_bytes = 10;
    let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR)
        .with_limits(limits);
    assert!(record.push_chain(&"123456", None, "t"));
    assert!(record.push_field("k", &"123"));
    assert!(!record.set_backtrace(&"more than is left"));
    assert_eq!(record.backtrace.as_deref(), Some(""));
    assert!(!record.push_chain(&"nothing left", None, "t"));
    assert_eq!(record.chain.len(), 1);
    assert!(record.truncated);
}

#[test]
fn test_trail_limit_and_owned_file_names() {
    let mut limits = RecordLimits::DEFAULT;
    limits.max_trail = 1;
    let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR)
        .with_limits(limits);
    assert!(record.push_trail(TrailEntry::new(here(), TrailStep::Created)));
    assert_eq!(record.trail[0].line, line!() - 1);
    assert!(!record.push_trail(TrailEntry::new(here(), TrailStep::Context)));
    assert!(record.truncated);
}

/// E-94: secrets stay redacted.
#[test]
fn test_secret_stays_redacted() {
    let err = DbError::Token {
        token: Secret::new("s3cr3t".into()),
    };
    let record = ErrorRecord::from_error(&err);
    assert_eq!(record.chain[0].message, "token [redacted] rejected");
    assert!(!format!("{record:?}").contains("s3cr3t"));
}

/// E-95: an event with no error.
#[test]
fn test_record_from_a_message_event() {
    let event = Event::new(Verb::Notice, here())
        .with_kinds(Kinds::NOTICE)
        .with_severity(Severity::NOTICE)
        .with_message(format_args!("cache rebuilt"));
    let record = ErrorRecord::from_event(&event);
    assert_eq!(record.chain.len(), 1);
    assert_eq!(record.chain[0].message, "cache rebuilt");
    assert_eq!(record.chain[0].type_name, "notice");
    assert_eq!(record.severity, Severity::NOTICE);
    assert_eq!(record.trail[0].step, TrailStep::Reported);
    assert_eq!(record.trail[0].file, file!());
}

#[test]
fn test_record_from_an_error_event_keeps_the_events_severity() {
    let err = DbError::Corrupt;
    let event = Event::new(Verb::Report, here())
        .with_error(&err)
        .with_kinds(Kinds::AUDIT)
        .with_severity(Severity::DEBUG);
    let record = ErrorRecord::from_event(&event);
    assert_eq!(record.severity, Severity::DEBUG);
    assert_eq!(record.kinds, Kinds::AUDIT);
    assert_eq!(record.code.unwrap().to_string(), "DB-0002");
}

#[test]
fn test_record_from_a_record_event_is_a_clone_with_the_location() {
    let source = ErrorRecord::from_error(&DbError::Corrupt);
    let event = Event::new(Verb::Report, here()).with_record(&source);
    let record = ErrorRecord::from_event(&event);
    assert_eq!(record.chain, source.chain);
    assert_eq!(record.trail.len(), 1);
    assert!(source.trail.is_empty());
}

#[test]
fn test_display_is_one_line() {
    let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    assert_eq!(record.to_string(), "");
    assert!(record.push_chain(&"line one\nline\ttwo", None, "t"));
    assert!(record.chain[0].has_control);
    assert_eq!(record.chain[0].message, "line one\nline\ttwo");
    assert_eq!(record.to_string(), "line one line two");
}

#[test]
fn test_setters() {
    let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    record.set_event_id(9);
    record.set_time(10);
    record.set_handled(false);
    assert!(record.set_backtrace(&"one"));
    assert!(record.set_backtrace(&"two"));
    assert_eq!(
        (
            record.event_id,
            record.time_unix_nanos,
            record.handled,
            record.backtrace.as_deref()
        ),
        (Some(9), Some(10), false, Some("two"))
    );
}

#[test]
fn test_record_is_send_sync_clone() {
    fn assert_all<T: Send + Sync + Clone + 'static>() {}
    assert_all::<ErrorRecord>();
}
