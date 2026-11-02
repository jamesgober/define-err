//! The reporting facade with a reporter installed: every verb, the filters,
//! once-per-site delivery, and locations (E-66, E-69 to E-71, E-73 to E-79,
//! E-82 to E-85). `Recorder` plays the error service: the 0.5 exit criterion.

mod common;

use std::thread;

use common::{serial, RECORDER};
use define_err::{
    define_errors, here, Category, Code, Event, InstallError, Kinds, Reporter, Severity, Slot,
    Verb, WarnMode,
};

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, name = TIMEOUT, help = "Raise pool.size.")]
        #[category(DeadlineExceeded)]
        Timeout => "query timed out",

        #[code(2, help = "Restore the page.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt => "page failed its checksum",

        #[code(3, help = "Ignore it.")]
        #[severity(WARNING)]
        Mild => "a mild problem",
    }
}

#[derive(Debug)]
struct Foreign;

impl core::fmt::Display for Foreign {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("foreign")
    }
}

impl core::error::Error for Foreign {}

/// Panics if the facade formats a filtered event.
struct NeverFormat;

impl core::fmt::Display for NeverFormat {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        panic!("a dropped event was formatted");
    }
}

#[test]
fn test_report_carries_the_probe_and_the_caller() {
    let _serial = serial();
    let line = line!() + 1;
    define_err::report(&DbError::Corrupt);
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.verb, Verb::Report);
    assert_eq!(event.severity, Severity::CRITICAL);
    assert_eq!(event.kinds, Kinds::ERROR | Kinds::SECURITY);
    assert_eq!(event.code.map(|c| c.number()), Some(2));
    assert_eq!(event.category, Category::DataLoss);
    assert_eq!(event.message, "page failed its checksum");
    assert_eq!(event.type_name.as_deref(), Some("DbError::Corrupt"));
    assert_eq!((event.file, event.line), (file!(), line));
}

#[test]
fn test_report_with_overrides_severity_and_kinds() {
    let _serial = serial();
    define_err::report_with(&DbError::Timeout, Severity::DEBUG, Kinds::AUDIT);
    let events = RECORDER.take();
    assert_eq!(events[0].severity, Severity::DEBUG);
    assert_eq!(events[0].kinds, Kinds::AUDIT);
    assert_eq!(events[0].code, Some(DbError::TIMEOUT));
}

/// E-83: a foreign error.
#[test]
fn test_report_of_a_foreign_error() {
    let _serial = serial();
    define_err::report(&Foreign);
    let events = RECORDER.take();
    assert_eq!(events[0].code, None);
    assert_eq!(events[0].category, Category::Unknown);
    assert_eq!(events[0].severity, Severity::ERROR);
    assert_eq!(events[0].kinds, Kinds::ERROR);
    assert_eq!(events[0].type_name, None);
}

/// E-83: `io::Error` is the one built-in fallback.
#[cfg(feature = "std")]
#[test]
fn test_report_of_an_io_error() {
    let _serial = serial();
    define_err::report(&std::io::Error::from(std::io::ErrorKind::NotFound));
    let events = RECORDER.take();
    assert_eq!(events[0].code, None);
    assert_eq!(events[0].category, Category::NotFound);
    assert_eq!(events[0].kinds, Kinds::ERROR);
    assert_eq!(events[0].type_name.as_deref(), Some("std::io::Error"));
}

/// E-85: braces in the text mean nothing.
#[test]
fn test_notice_is_literal() {
    let _serial = serial();
    define_err::notice("{x}");
    let events = RECORDER.take();
    assert_eq!(events[0].verb, Verb::Notice);
    assert_eq!(events[0].kinds, Kinds::NOTICE);
    assert_eq!(events[0].severity, Severity::NOTICE);
    assert_eq!(events[0].message, "{x}");
}

#[test]
fn test_notice_macro_and_extra_kinds() {
    let _serial = serial();
    let elapsed = 12;
    let line = line!() + 1;
    define_err::notice!("rebuilt in {elapsed} ms");
    define_err::notice_with(Kinds::CONFIG, "defaults");
    let events = RECORDER.take();
    assert_eq!(events[0].message, "rebuilt in 12 ms");
    assert_eq!(events[0].line, line);
    assert_eq!(events[1].kinds, Kinds::NOTICE | Kinds::CONFIG);
}

#[test]
fn test_warn_kinds_and_severity() {
    let _serial = serial();
    define_err::warn("one");
    define_err::warn_with(Kinds::CONFIG, "two");
    let events = RECORDER.take();
    assert_eq!(events[0].verb, Verb::Warn);
    assert_eq!(events[0].kinds, Kinds::WARNING);
    assert_eq!(events[0].severity, Severity::WARNING);
    assert_eq!(events[1].kinds, Kinds::WARNING | Kinds::CONFIG);
}

/// E-69: a loop delivers once, by macro and by function.
#[test]
fn test_warn_in_a_loop_delivers_once() {
    let _serial = serial();
    // Miri runs this about ten thousand times slower.
    let rounds = if cfg!(miri) { 100 } else { 1_000_000 };
    for i in 0..rounds {
        define_err::warn!("macro {i}");
        define_err::warn("function");
    }
    let events = RECORDER.take();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].message, "macro 0");
}

#[test]
fn test_fired_site_does_not_evaluate_its_arguments() {
    let _serial = serial();
    let mut evaluated = 0;
    for _ in 0..3 {
        define_err::warn!("count {}", {
            evaluated += 1;
            evaluated
        });
    }
    assert_eq!(evaluated, 1);
}

/// E-70: one static for every instantiation.
#[test]
fn test_warn_in_a_generic_function_is_one_site() {
    #[allow(clippy::extra_unused_type_parameters)]
    fn generic<T>() {
        define_err::warn!("generic");
    }
    let _serial = serial();
    generic::<u8>();
    generic::<u16>();
    generic::<String>();
    assert_eq!(RECORDER.take().len(), 1);
}

/// E-71: one site under contention.
#[test]
fn test_warn_from_many_threads_delivers_once() {
    fn site() {
        define_err::warn!("contended");
    }
    let _serial = serial();
    thread::scope(|scope| {
        for _ in 0..64 {
            let _ = scope.spawn(site);
        }
    });
    assert_eq!(RECORDER.take().len(), 1);
}

#[track_caller]
fn tracked_helper() {
    define_err::warn("tracked");
}

fn plain_helper() {
    define_err::notice("plain");
}

/// E-73: `#[track_caller]` helpers report their caller.
#[test]
fn test_track_caller_helpers() {
    let _serial = serial();
    let line = line!() + 1;
    tracked_helper();
    plain_helper();
    let events = RECORDER.take();
    assert_eq!(events[0].line, line);
    assert!(events[1].line < line && events[1].line > 0);
}

macro_rules! report_here {
    ($error:expr) => {
        define_err::report(&$error)
    };
}

/// E-74: a macro's call site, also for the crate's own macros.
#[test]
fn test_macros_report_the_outermost_call_site() {
    let _serial = serial();
    let line = line!() + 1;
    report_here!(DbError::Timeout);
    define_err::notice!("from the macro");
    let events = RECORDER.take();
    assert_eq!(events[0].line, line);
    assert_eq!(events[1].line, line + 1);
}

#[test]
fn test_trigger_adds_its_kind() {
    let _serial = serial();
    define_err::trigger(Kinds::SECURITY, "lockout");
    let failed = 21;
    define_err::trigger!(Kinds::NONE, "{failed} failures");
    let events = RECORDER.take();
    assert_eq!(events[0].verb, Verb::Trigger);
    assert_eq!(events[0].kinds, Kinds::TRIGGER | Kinds::SECURITY);
    assert_eq!(events[0].severity, Severity::WARNING);
    // E-84: no kinds given.
    assert_eq!(events[1].kinds, Kinds::TRIGGER);
    assert_eq!(events[1].message, "21 failures");
}

#[test]
fn test_deprecated_messages_and_once_per_site() {
    fn config_key() {
        define_err::deprecated!("pool.max", since = "2.1.0", replacement = "pool.size");
        define_err::deprecated!("pool.min", since = "2.1.0");
        define_err::deprecated!("pool.idle", replacement = "pool.size");
        define_err::deprecated!("pool.old");
        define_err::deprecated("pool.fn");
    }
    let _serial = serial();
    config_key();
    config_key();
    let events = RECORDER.take();
    let messages: Vec<&str> = events.iter().map(|e| e.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "`pool.max` is deprecated since 2.1.0; use pool.size instead",
            "`pool.min` is deprecated since 2.1.0",
            "`pool.idle` is deprecated; use pool.size instead",
            "`pool.old` is deprecated",
            "`pool.fn` is deprecated",
        ]
    );
    assert_eq!(events[0].verb, Verb::Deprecated);
    assert_eq!(events[0].kinds, Kinds::DEPRECATED | Kinds::WARNING);
    assert_eq!(events[0].severity, Severity::WARNING);
}

/// E-75: `Always`, then `Once`.
#[test]
fn test_mode_always_then_once() {
    fn fired() {
        define_err::warn!("fired");
    }
    fn fresh() {
        define_err::warn!("fresh");
    }
    let _serial = serial();
    fired();
    define_err::set_warn_mode(WarnMode::Always);
    fired();
    fired();
    assert_eq!(RECORDER.take().len(), 3);
    define_err::set_warn_mode(WarnMode::Once);
    fired();
    fresh();
    fresh();
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].message, "fresh");
}

/// E-76: `Ignore` drops warnings and nothing else, and uses up no site.
#[test]
fn test_mode_ignore() {
    fn site() {
        define_err::warn!("ignored for now");
    }
    let _serial = serial();
    define_err::set_warn_mode(WarnMode::Ignore);
    site();
    define_err::report(&DbError::Timeout);
    assert!(define_err::soft(DbError::Mild).is_ok());
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].verb, Verb::Report);
    define_err::set_warn_mode(WarnMode::Once);
    site();
    assert_eq!(RECORDER.take().len(), 1);
}

#[test]
fn test_soft_delivers_a_warning_that_carries_the_error() {
    let _serial = serial();
    assert!(define_err::soft(DbError::Corrupt).is_ok());
    let events = RECORDER.take();
    assert_eq!(events[0].verb, Verb::Soft);
    assert_eq!(events[0].severity, Severity::WARNING);
    assert_eq!(
        events[0].kinds,
        Kinds::WARNING | Kinds::ERROR | Kinds::SECURITY
    );
    assert_eq!(events[0].code.map(|c| c.number()), Some(2));
}

/// E-77: strict mode, with a reporter (`facade_empty.rs` covers without).
#[test]
fn test_mode_error_makes_soft_fail() {
    let _serial = serial();
    define_err::set_warn_mode(WarnMode::Error);
    assert!(matches!(
        define_err::soft(DbError::Mild),
        Err(DbError::Mild)
    ));
    assert!(RECORDER.take().is_empty());
}

/// E-78: dropped before formatting.
#[test]
fn test_min_severity_filters_before_formatting() {
    let _serial = serial();
    define_err::set_min_severity(Some(Severity::ERROR));
    define_err::warn!("never {}", NeverFormat);
    define_err::notice!("never {}", NeverFormat);
    define_err::warn("dropped");
    define_err::report(&DbError::Mild);
    define_err::report(&DbError::Corrupt);
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].severity, Severity::CRITICAL);
}

#[test]
fn test_filtered_warning_keeps_its_site() {
    fn site() {
        define_err::warn!("filtered, then delivered");
    }
    let _serial = serial();
    define_err::set_min_severity(Some(Severity::ERROR));
    site();
    assert!(RECORDER.take().is_empty());
    define_err::set_min_severity(None);
    site();
    assert_eq!(RECORDER.take().len(), 1);
}

/// E-79: the kinds mask.
#[test]
fn test_kinds_mask_filters() {
    let _serial = serial();
    define_err::set_kinds_mask(Kinds::SECURITY);
    define_err::report(&DbError::Corrupt);
    define_err::report(&DbError::Timeout);
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].code.map(|c| c.number()), Some(2));
}

#[test]
fn test_default_mask_passes_an_event_with_no_kinds() {
    let _serial = serial();
    define_err::report_with(&DbError::Timeout, Severity::ERROR, Kinds::NONE);
    assert_eq!(RECORDER.take().len(), 1);
}

#[test]
fn test_settings_do_not_disturb_each_other() {
    let _serial = serial();
    define_err::set_warn_mode(WarnMode::Always);
    define_err::set_min_severity(Some(Severity::CRITICAL));
    assert_eq!(define_err::warn_mode(), WarnMode::Always);
    assert_eq!(define_err::min_severity(), Some(Severity::CRITICAL));
    assert!(define_err::is_installed());
    define_err::set_min_severity(None);
    assert_eq!(define_err::min_severity(), None);
    assert_eq!(define_err::warn_mode(), WarnMode::Always);
    assert!(define_err::is_installed());
}

/// E-66: the second installer names the first.
#[test]
fn test_second_install_names_the_first() {
    let _serial = serial();
    struct Other;
    impl Reporter for Other {
        fn receive(&self, _event: &Event<'_>) {}
    }
    static OTHER: Other = Other;
    let err: InstallError = define_err::set_reporter(&OTHER).unwrap_err();
    assert_eq!(err.slot, Slot::Reporter);
    let at = err.installed_at.unwrap();
    assert_eq!(
        at.file(),
        "tests/common/mod.rs".replace('/', std::path::MAIN_SEPARATOR_STR)
    );
    assert_eq!(define_err::reporter_installed_at(), Some(at));
    assert!(err
        .to_string()
        .starts_with("a reporter is already installed (at tests"));
    assert!(err.to_string().ends_with("install it once, at start-up"));
}

#[test]
fn test_event_builder_and_debug_leave_out_user_text() {
    let code = Code::parse("DB-0001").unwrap();
    let event = Event::new(Verb::Report, here())
        .with_code(Some(code))
        .with_category(Category::Internal)
        .with_kinds(Kinds::ERROR)
        .with_severity(Severity::FATAL)
        .with_message(format_args!("text"));
    assert_eq!(event.code(), Some(code));
    assert_eq!(event.category(), Category::Internal);
    assert_eq!(event.severity(), Severity::FATAL);
    assert!(event.error().is_none());
    assert!(event.type_name().is_none());
    assert!(format!("{event:?}").starts_with("Event { verb: Report"));
}
