//! The verbs: `report`, `notice`, `warn`, `trigger`, `deprecated`, `soft`.
//!
//! Every verb starts with one `Acquire` load and returns when nothing is
//! installed. Every function is `#[track_caller]`, so an event carries the
//! caller's location, never a helper's.

use core::{error::Error, fmt};

use super::{
    event::{Event, Verb},
    guard, state,
    state::{Site, Word},
    WarnMode,
};
#[cfg(feature = "alloc")]
use crate::ErrorRecord;
use crate::{here, probe, Kinds, Location, Severity};

/// Hands an event to the reporter.
#[inline]
fn deliver(word: Word, event: &Event<'_>) {
    if let Some(reporter) = state::reporter(word) {
        guard::run(|| reporter.receive(event));
    }
}

/// An event for an error, from its probe.
fn error_event<'a>(verb: Verb, error: &'a (dyn Error + 'static), location: Location) -> Event<'a> {
    let facts = probe::facts(error);
    let mut event = Event::new(verb, location)
        .with_kinds(facts.kinds)
        .with_severity(facts.severity)
        .with_code(facts.code)
        .with_category(facts.category)
        .with_error(error);
    event.type_name = facts.type_name;
    event
}

/// Delivers a report after the `is_ready` check. A minimum severity is
/// checked from the probe's two digits first, so dropping an event does not
/// decode the rest.
fn report_ready(word: Word, error: &(dyn Error + 'static), location: Location) {
    if state::has_minimum(word)
        && probe::stated_severity(error)
            .and_then(Severity::new)
            .is_some_and(|severity| state::below_minimum(word, severity))
    {
        return;
    }
    let event = error_event(Verb::Report, error, location);
    if state::passes(word, event.severity, event.kinds) {
        deliver(word, &event);
    }
}

/// Reports an error. Severity, kinds, code, and category come from its
/// probe; a foreign error is `ERROR` with category `Unknown`, except a
/// `std::io::Error`, whose category comes from its kind.
///
/// Reporting never changes control flow: the caller still returns or handles
/// the error. Report at boundaries (a request handler, a job runner, `main`),
/// not at every `?`. One load when nothing is installed.
///
/// # Examples
///
/// ```
/// use define_err::define_errors;
///
/// define_errors! {
///     pub struct Timeout => "query timed out";
/// }
///
/// define_err::report(&Timeout);         // returns at once: nothing is installed
/// ```
#[track_caller]
#[inline]
pub fn report(error: &(dyn Error + 'static)) {
    let word = state::load();
    if state::is_ready(word) {
        report_ready(word, error, here());
    }
}

/// Reports an error with an explicit severity and kinds, for a caller that
/// knows better than the definition (a timeout that is routine here).
///
/// # Examples
///
/// ```
/// use define_err::{define_errors, Kinds, Severity};
///
/// define_errors! {
///     pub struct Miss => "cache miss";
/// }
///
/// define_err::report_with(&Miss, Severity::DEBUG, Kinds::ERROR);
/// ```
#[track_caller]
#[inline]
pub fn report_with(error: &(dyn Error + 'static), severity: Severity, kinds: Kinds) {
    let word = state::load();
    if state::is_ready(word) && state::passes(word, severity, kinds) {
        let event = error_event(Verb::Report, error, here())
            .with_severity(severity)
            .with_kinds(kinds);
        deliver(word, &event);
    }
}

/// Delivers an owned record: the event carries it, and severity, kinds,
/// code, and category come from it. This is how a dynamic error type reports
/// what a `source()` chain cannot carry.
///
/// # Examples
///
/// ```
/// use define_err::{Category, ErrorRecord, Kinds, Severity};
///
/// let record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
/// if define_err::is_installed() {
///     define_err::report_record(&record);
/// }
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[track_caller]
pub fn report_record(record: &ErrorRecord) {
    let word = state::load();
    if state::is_ready(word) && state::passes(word, record.severity, record.kinds) {
        let mut event = Event::new(Verb::Report, here())
            .with_kinds(record.kinds)
            .with_severity(record.severity)
            .with_code(record.code)
            .with_category(record.category)
            .with_record(record);
        event.type_name = record.chain.first().map(|entry| &*entry.type_name);
        deliver(word, &event);
    }
}

/// Delivers a message event, after the `is_ready` check.
fn message_ready(
    word: Word,
    verb: Verb,
    severity: Severity,
    kinds: Kinds,
    args: fmt::Arguments<'_>,
    location: Location,
) {
    if state::passes(word, severity, kinds) {
        let event = Event::new(verb, location)
            .with_kinds(kinds)
            .with_severity(severity)
            .with_message(args);
        deliver(word, &event);
    }
}

/// `message_ready` for plain text. Out of line, so the inline gate of a verb
/// sets up nothing before its load.
fn text_ready(
    word: Word,
    verb: Verb,
    severity: Severity,
    kinds: Kinds,
    text: &str,
    location: Location,
) {
    message_ready(
        word,
        verb,
        severity,
        kinds,
        format_args!("{text}"),
        location,
    );
}

/// `notice!`'s target.
#[doc(hidden)]
#[inline]
pub fn notice_args(kinds: Kinds, args: fmt::Arguments<'_>, location: Location) {
    let word = state::load();
    if state::is_ready(word) {
        message_ready(
            word,
            Verb::Notice,
            Severity::NOTICE,
            Kinds::NOTICE | kinds,
            args,
            location,
        );
    }
}

/// An informational event below a warning: kinds `NOTICE`, severity
/// `NOTICE`. The text is delivered as written; braces mean nothing here.
///
/// # Examples
///
/// ```
/// define_err::notice("cache rebuilt from the journal");
/// ```
#[track_caller]
#[inline]
pub fn notice(message: &str) {
    notice_with(Kinds::NONE, message);
}

/// [`notice`] with more kinds.
///
/// # Examples
///
/// ```
/// define_err::notice_with(define_err::Kinds::CONFIG, "using the default port");
/// ```
#[track_caller]
#[inline]
pub fn notice_with(kinds: Kinds, message: &str) {
    let word = state::load();
    if state::is_ready(word) {
        text_ready(
            word,
            Verb::Notice,
            Severity::NOTICE,
            Kinds::NOTICE | kinds,
            message,
            here(),
        );
    }
}

/// How a warning remembers its call site.
#[derive(Clone, Copy)]
enum Memory<'a> {
    Site(&'a Site),
    Table,
}

impl Memory<'_> {
    fn fires(self, location: Location) -> bool {
        match self {
            Memory::Site(site) => state::site_fires(site),
            Memory::Table => state::table_fires(location),
        }
    }
}

/// Delivers a warning, once per site unless the mode says otherwise. A site
/// is marked only when the warning would reach the reporter, so a warning
/// filtered out, or issued before installation, keeps its delivery.
fn warning(
    verb: Verb,
    kinds: Kinds,
    args: fmt::Arguments<'_>,
    location: Location,
    memory: Memory<'_>,
) {
    let word = state::load();
    if !state::is_ready(word) || !state::passes(word, Severity::WARNING, kinds) {
        return;
    }
    let mode = state::mode(word);
    if mode == WarnMode::Ignore || guard::refuses() {
        return;
    }
    if mode == WarnMode::Always || memory.fires(location) {
        let event = Event::new(verb, location)
            .with_kinds(kinds)
            .with_severity(Severity::WARNING)
            .with_message(args);
        deliver(word, &event);
    }
}

/// Whether `warn!` or `deprecated!` should build its message: the check that
/// keeps a fired site from evaluating its arguments.
#[doc(hidden)]
#[inline]
pub fn warn_ready(site: &Site) -> bool {
    let word = state::load();
    state::is_ready(word) && (state::mode(word) == WarnMode::Always || !state::site_has_fired(site))
}

/// `warn`'s slow path, out of line for the reason `text_ready` is.
fn warn_text(kinds: Kinds, message: &str, location: Location) {
    warning(
        Verb::Warn,
        Kinds::WARNING | kinds,
        format_args!("{message}"),
        location,
        Memory::Table,
    );
}

/// `deprecated`'s slow path.
fn deprecated_text(item: &str, location: Location) {
    warning(
        Verb::Deprecated,
        Kinds::DEPRECATED | Kinds::WARNING,
        format_args!("`{item}` is deprecated"),
        location,
        Memory::Table,
    );
}

/// `warn!`'s target.
#[doc(hidden)]
pub fn warn_site(site: &Site, kinds: Kinds, args: fmt::Arguments<'_>, location: Location) {
    warning(
        Verb::Warn,
        Kinds::WARNING | kinds,
        args,
        location,
        Memory::Site(site),
    );
}

/// `deprecated!`'s target.
#[doc(hidden)]
pub fn deprecated_site(site: &Site, args: fmt::Arguments<'_>, location: Location) {
    warning(
        Verb::Deprecated,
        Kinds::DEPRECATED | Kinds::WARNING,
        args,
        location,
        Memory::Site(site),
    );
}

/// A warning: kinds `WARNING`, severity `WARNING`. By default each call site
/// delivers once per process, so a warning in a loop reaches the reporter one
/// time. A warning issued before a reporter exists does not use up its site.
///
/// A function cannot own a static per call site, so this form keeps a table
/// of 512 sites; past that, a site delivers every time and
/// [`stats`](crate::stats) counts it. [`warn!`](crate::warn!) is exact for
/// any number of sites.
///
/// # Examples
///
/// ```
/// define_err::warn("cache file is stale, rebuilding");
/// ```
#[track_caller]
#[inline]
pub fn warn(message: &str) {
    warn_with(Kinds::NONE, message);
}

/// [`warn`] with more kinds.
///
/// # Examples
///
/// ```
/// define_err::warn_with(define_err::Kinds::CONFIG, "configuration file is empty; using defaults");
/// ```
#[track_caller]
#[inline]
pub fn warn_with(kinds: Kinds, message: &str) {
    if state::is_ready(state::load()) {
        warn_text(kinds, message, here());
    }
}

/// `trigger!`'s target.
#[doc(hidden)]
#[inline]
pub fn trigger_args(kinds: Kinds, args: fmt::Arguments<'_>, location: Location) {
    let word = state::load();
    if state::is_ready(word) {
        message_ready(
            word,
            Verb::Trigger,
            Severity::WARNING,
            Kinds::TRIGGER | kinds,
            args,
            location,
        );
    }
}

/// Raises a named condition on purpose: verb `Trigger`, kinds `TRIGGER` plus
/// the given kinds, severity `WARNING`. The error service may run actions
/// registered for those kinds; this crate only delivers the event.
///
/// # Examples
///
/// ```
/// use define_err::Kinds;
///
/// define_err::trigger(Kinds::SECURITY, "too many failed logins");
/// ```
#[track_caller]
#[inline]
pub fn trigger(kinds: Kinds, message: &str) {
    let word = state::load();
    if state::is_ready(word) {
        text_ready(
            word,
            Verb::Trigger,
            Severity::WARNING,
            Kinds::TRIGGER | kinds,
            message,
            here(),
        );
    }
}

/// A run-time deprecation warning for anything that is not a Rust item (a
/// configuration key, a protocol field): kinds `DEPRECATED | WARNING`,
/// severity `WARNING`, once per call site like [`warn`].
///
/// # Examples
///
/// ```
/// define_err::deprecated("pool.max");
/// ```
#[track_caller]
#[inline]
pub fn deprecated(item: &str) {
    if state::is_ready(state::load()) {
        deprecated_text(item, here());
    }
}

/// A soft error: a warning today that a strict deployment refuses.
///
/// In the default mode this delivers a warning that carries the error (kinds
/// `WARNING` plus the error's, severity `WARNING`) and returns `Ok(())`. When
/// the warn mode is [`WarnMode::Error`] it returns `Err(error)`, even with no
/// reporter installed.
///
/// # Errors
///
/// Returns `Err(error)` when the warn mode is [`WarnMode::Error`].
///
/// # Examples
///
/// ```
/// use define_err::define_errors;
///
/// define_errors! {
///     pub struct Unsigned => "plugin is not signed";
/// }
///
/// assert!(define_err::soft(Unsigned).is_ok());
/// ```
#[track_caller]
#[inline]
pub fn soft<E: Error + 'static>(error: E) -> Result<(), E> {
    let word = state::load();
    match state::mode(word) {
        WarnMode::Error => Err(error),
        WarnMode::Ignore => Ok(()),
        _ => {
            if state::is_ready(word) && !state::below_minimum(word, Severity::WARNING) {
                soft_ready(word, &error, here());
            }
            Ok(())
        }
    }
}

fn soft_ready(word: Word, error: &(dyn Error + 'static), location: Location) {
    let event = error_event(Verb::Soft, error, location);
    let kinds = Kinds::WARNING | event.kinds;
    if state::passes(word, Severity::WARNING, kinds) {
        deliver(
            word,
            &event.with_kinds(kinds).with_severity(Severity::WARNING),
        );
    }
}
