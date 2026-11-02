//! Capturing what the facade delivers, for tests of code that reports.

use std::{
    cell::RefCell,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    facade::{reporter, set_reporter, Event, InstallError, Reporter, Verb},
    Code, ErrorRecord, Kinds, Severity,
};

/// One event, owned.
///
/// # Examples
///
/// ```
/// use define_err::testing::capture;
///
/// let ((), events) = capture(|| define_err::notice("started")).unwrap();
/// assert_eq!(events[0].message, "started");
/// assert_eq!(events[0].line, line!() - 2);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CapturedEvent {
    /// Which verb raised it.
    pub verb: Verb,
    /// Its kinds.
    pub kinds: Kinds,
    /// Its severity.
    pub severity: Severity,
    /// Its code.
    pub code: Option<Code>,
    /// The formatted message, or the error's `Display` for error events.
    pub message: String,
    /// The caller's file.
    pub file: &'static str,
    /// The caller's line.
    pub line: u32,
    /// The caller's column.
    pub column: u32,
    /// The record, for error events.
    pub record: Option<ErrorRecord>,
}

impl CapturedEvent {
    fn new(event: &Event<'_>) -> CapturedEvent {
        let (message, record) = if let Some(record) = event.record() {
            (record.to_string(), Some(ErrorRecord::from_event(event)))
        } else if let Some(error) = event.error() {
            (error.to_string(), Some(ErrorRecord::from_event(event)))
        } else {
            (
                event
                    .message()
                    .map(|text| text.to_string())
                    .unwrap_or_default(),
                None,
            )
        };
        CapturedEvent {
            verb: event.verb(),
            kinds: event.kinds(),
            severity: event.severity(),
            code: event.code(),
            message,
            file: event.location().file(),
            line: event.location().line(),
            column: event.location().column(),
            record,
        }
    }
}

std::thread_local! {
    static BUFFER: RefCell<Option<Vec<CapturedEvent>>> = const { RefCell::new(None) };
}

/// The reporter [`capture`] installs. It routes each event to a buffer of the
/// thread that raised it and drops events from threads that are not
/// capturing. Sized, so its address tells it apart from any other reporter.
///
/// # Examples
///
/// ```
/// use define_err::{testing::CaptureReporter, Reporter};
///
/// fn is_reporter<R: Reporter>() {}
/// is_reporter::<CaptureReporter>();
/// ```
#[derive(Debug)]
pub struct CaptureReporter {
    _address: u8,
}

static REPORTER: CaptureReporter = CaptureReporter { _address: 0 };

impl Reporter for CaptureReporter {
    fn receive(&self, event: &Event<'_>) {
        BUFFER.with(|buffer| {
            if let Some(events) = buffer.borrow_mut().as_mut() {
                events.push(CapturedEvent::new(event));
            }
        });
    }
}

/// Puts back the outer buffer of a nested `capture`, once: by `finish`, or by
/// `drop` when `f` panics.
struct Restore(Option<Option<Vec<CapturedEvent>>>);

impl Restore {
    fn finish(&mut self) -> Vec<CapturedEvent> {
        match self.0.take() {
            Some(outer) => BUFFER
                .with(|buffer| buffer.replace(outer))
                .unwrap_or_default(),
            None => Vec::new(),
        }
    }
}

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

/// Runs `f` and returns its result with every event the facade delivered on
/// this thread while `f` ran.
///
/// Tests running in parallel never see each other's events. Installs
/// [`CaptureReporter`] on first use. Warn sites are process-wide, so a site
/// that has fired stays silent in a later capture.
///
/// # Errors
///
/// Returns [`InstallError`] instead of running `f` if a different reporter is
/// already installed: a test cannot capture what goes to someone else's
/// reporter, and an empty list must never pass as the real one.
///
/// # Examples
///
/// ```
/// use define_err::testing::capture;
///
/// fn parse_port(text: &str) -> u16 {
///     text.parse().unwrap_or_else(|_| {
///         define_err::warn!("port {text:?} is not a number; using 8080");
///         8080
///     })
/// }
///
/// let (ports, events) = capture(|| (parse_port("x"), parse_port("y"))).unwrap();
/// assert_eq!(ports, (8080, 8080));
/// assert_eq!(events.len(), 1);                     // once per call site
/// assert!(events[0].message.contains("not a number"));
/// ```
pub fn capture<R>(f: impl FnOnce() -> R) -> Result<(R, Vec<CapturedEvent>), InstallError> {
    if let Err(error) = set_reporter(&REPORTER) {
        let ours: *const CaptureReporter = &REPORTER;
        let current =
            reporter().map(|installed| core::ptr::from_ref(installed).cast::<CaptureReporter>());
        if current != Some(ours) {
            return Err(error);
        }
    }
    let mut restore = Restore(Some(BUFFER.with(|buffer| buffer.replace(Some(Vec::new())))));
    let result = f();
    let events = restore.finish();
    Ok((result, events))
}
