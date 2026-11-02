//! The reporting facade: one write-once slot any crate can reach, after the
//! `log` crate's `set_logger`.
//!
//! Any crate calls the verbs; the application installs one [`Reporter`] at
//! start-up. With nothing installed, a verb costs one atomic load. See
//! `docs/FACADE.md`.

#[cfg(target_has_atomic = "32")]
mod control;
mod event;
mod guard;
#[cfg(target_has_atomic = "32")]
mod sites;
#[cfg(target_has_atomic = "32")]
mod slot;
#[cfg(target_has_atomic = "32")]
pub(crate) mod state;
#[cfg(not(target_has_atomic = "32"))]
#[path = "inert.rs"]
pub(crate) mod state;
#[cfg(target_has_atomic = "32")]
mod sync;
mod verbs;

use core::fmt;

pub use self::event::{Event, Verb};
#[doc(hidden)]
pub use self::state::Site;
#[cfg(feature = "alloc")]
pub use self::verbs::report_record;
pub use self::verbs::{
    deprecated, notice, notice_with, report, report_with, soft, trigger, warn, warn_with,
};
#[doc(hidden)]
pub use self::verbs::{deprecated_site, notice_args, trigger_args, warn_ready, warn_site};
use crate::{Code, Kinds, Location, Severity};

/// Receives every event the facade delivers.
///
/// The contract, in full in `docs/FACADE.md`: `receive` returns quickly
/// (microseconds) and never blocks on I/O; it may be called from any thread
/// at once; it must not assume one call per error; and it keeps nothing
/// borrowed from the event. With `std`, a verb called from inside `receive`
/// on the same thread is dropped and counted, and a panic inside `receive` is
/// caught and counted. Without `std` neither guard exists, and a reporter
/// must not call verbs or panic.
///
/// # Examples
///
/// ```
/// use define_err::{Event, Reporter};
///
/// struct StderrReporter;
///
/// impl Reporter for StderrReporter {
///     fn receive(&self, event: &Event<'_>) {
///         let at = event.location();
///         if let Some(message) = event.message() {
///             eprintln!("{} {}:{} {message}", event.severity(), at.file(), at.line());
///         }
///     }
/// }
///
/// static REPORTER: StderrReporter = StderrReporter;
/// define_err::set_reporter(&REPORTER).unwrap();
/// define_err::notice("started");
/// ```
pub trait Reporter: Send + Sync {
    /// Takes one event.
    fn receive(&self, event: &Event<'_>);

    /// Writes out anything queued, for example before the process exits.
    fn flush(&self) {}
}

/// Which write-once slot an [`InstallError`] is about.
///
/// # Examples
///
/// ```
/// assert_ne!(define_err::Slot::Reporter, define_err::Slot::CreationHook);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Slot {
    /// The reporter.
    Reporter,
    /// The creation hook.
    CreationHook,
}

/// A write-once slot is already taken.
///
/// `installed_at` is where the first installation succeeded; `None` only if
/// the losing call raced the winner between its claim and its publish.
///
/// # Examples
///
/// ```
/// use define_err::{InstallError, Reporter, Event, Slot};
///
/// struct Quiet;
/// impl Reporter for Quiet {
///     fn receive(&self, _event: &Event<'_>) {}
/// }
/// static FIRST: Quiet = Quiet;
/// static SECOND: Quiet = Quiet;
///
/// define_err::set_reporter(&FIRST).unwrap();
/// let err = define_err::set_reporter(&SECOND).unwrap_err();
/// assert_eq!(err.slot, Slot::Reporter);
/// assert!(err.to_string().contains("already installed"));
/// ```
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct InstallError {
    /// The slot.
    pub slot: Slot,
    /// Where the first installation succeeded.
    pub installed_at: Option<Location>,
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.slot {
            Slot::Reporter => "a reporter",
            Slot::CreationHook => "a creation hook",
        };
        match self.installed_at {
            Some(at) => write!(
                f,
                "{what} is already installed (at {at}); install it once, at start-up"
            ),
            None => write!(
                f,
                "{what} is being installed by another thread; install it once, at start-up"
            ),
        }
    }
}

impl core::error::Error for InstallError {}

/// Installs the process's reporter. It can be set once: a later call returns
/// [`InstallError`] naming the file and line of the installation that won.
///
/// Install first thing in `main`, and treat `Err` as a fatal boot error: a
/// dependency that installed its own reporter first would otherwise see every
/// error the application reports, or silence them.
///
/// Absent on targets without compare-and-swap.
///
/// # Errors
///
/// Returns [`InstallError`] when a reporter is already installed.
///
/// # Examples
///
/// ```
/// use define_err::{Event, Reporter};
///
/// struct Quiet;
/// impl Reporter for Quiet {
///     fn receive(&self, _event: &Event<'_>) {}
/// }
/// static REPORTER: Quiet = Quiet;
///
/// assert!(!define_err::is_installed());
/// define_err::set_reporter(&REPORTER).unwrap();
/// assert!(define_err::is_installed());
/// ```
#[cfg(target_has_atomic = "32")]
#[cfg_attr(docsrs, doc(cfg(target_has_atomic = "32")))]
#[track_caller]
pub fn set_reporter(reporter: &'static dyn Reporter) -> Result<(), InstallError> {
    state::install_reporter(|| reporter, crate::here()).map_err(|installed_at| InstallError {
        slot: Slot::Reporter,
        installed_at,
    })
}

/// [`set_reporter`] for an owned reporter. The box is leaked once, only by
/// the call that wins; a losing call drops it.
///
/// # Errors
///
/// Returns [`InstallError`] when a reporter is already installed.
///
/// # Examples
///
/// ```
/// use define_err::{Event, Reporter};
///
/// struct Quiet;
/// impl Reporter for Quiet {
///     fn receive(&self, _event: &Event<'_>) {}
/// }
///
/// define_err::set_boxed_reporter(Box::new(Quiet)).unwrap();
/// ```
#[cfg(all(feature = "alloc", target_has_atomic = "32"))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[track_caller]
pub fn set_boxed_reporter(reporter: alloc::boxed::Box<dyn Reporter>) -> Result<(), InstallError> {
    state::install_reporter(move || alloc::boxed::Box::leak(reporter), crate::here()).map_err(
        |installed_at| InstallError {
            slot: Slot::Reporter,
            installed_at,
        },
    )
}

/// Whether a reporter is installed. One load: lets a caller skip expensive
/// preparation when nobody listens.
///
/// # Examples
///
/// ```
/// assert!(!define_err::is_installed());
/// ```
#[inline]
#[must_use]
pub fn is_installed() -> bool {
    state::is_ready(state::load())
}

/// The installed reporter, for code that forwards events.
///
/// # Examples
///
/// ```
/// assert!(define_err::reporter().is_none());
/// ```
#[must_use]
pub fn reporter() -> Option<&'static dyn Reporter> {
    state::reporter(state::load())
}

/// Where the reporter was installed, for audits.
///
/// # Examples
///
/// ```
/// assert!(define_err::reporter_installed_at().is_none());
/// ```
#[must_use]
pub fn reporter_installed_at() -> Option<Location> {
    state::reporter_installed_at()
}

/// Calls [`Reporter::flush`]; returns at once when nothing is installed.
///
/// # Examples
///
/// ```
/// define_err::flush();
/// ```
pub fn flush() {
    if let Some(reporter) = reporter() {
        guard::run(|| reporter.flush());
    }
}

/// What `warn` and `soft` do. A process-wide setting, for the application or
/// the error service.
///
/// # Examples
///
/// ```
/// use define_err::WarnMode;
///
/// define_err::set_warn_mode(WarnMode::Error);
/// assert_eq!(define_err::warn_mode(), WarnMode::Error);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum WarnMode {
    /// `warn` and `deprecated` deliver once per call site (the default).
    Once,
    /// Every warning is delivered; switching back to `Once` keeps the sites
    /// that already fired.
    Always,
    /// Warnings are dropped; `report`, `notice`, and `trigger` are unaffected.
    Ignore,
    /// As `Once`, and [`soft`] returns `Err`.
    Error,
}

/// Sets the [`WarnMode`]. No effect on targets without compare-and-swap.
///
/// # Examples
///
/// ```
/// define_err::set_warn_mode(define_err::WarnMode::Always);
/// ```
pub fn set_warn_mode(mode: WarnMode) {
    state::set_mode(mode);
}

/// The current [`WarnMode`].
///
/// # Examples
///
/// ```
/// assert_eq!(define_err::warn_mode(), define_err::WarnMode::Once);
/// ```
#[must_use]
pub fn warn_mode() -> WarnMode {
    state::mode(state::load())
}

/// Drops events below `min` before the event is built or anything is
/// formatted. `None`, the default, passes everything.
///
/// # Examples
///
/// ```
/// use define_err::Severity;
///
/// define_err::set_min_severity(Some(Severity::WARNING));
/// assert_eq!(define_err::min_severity(), Some(Severity::WARNING));
/// ```
pub fn set_min_severity(min: Option<Severity>) {
    state::set_min_severity(min);
}

/// The current minimum severity.
///
/// # Examples
///
/// ```
/// assert_eq!(define_err::min_severity(), None);
/// ```
#[must_use]
pub fn min_severity() -> Option<Severity> {
    state::min_severity()
}

/// Drops events with no kind in `mask`. [`Kinds::ALL`], the default, passes
/// everything.
///
/// # Examples
///
/// ```
/// use define_err::Kinds;
///
/// define_err::set_kinds_mask(Kinds::ALL - Kinds::DEPRECATED);
/// assert!(!define_err::kinds_mask().contains(Kinds::DEPRECATED));
/// ```
pub fn set_kinds_mask(mask: Kinds) {
    state::set_kinds_mask(mask);
}

/// The current kinds mask.
///
/// # Examples
///
/// ```
/// assert_eq!(define_err::kinds_mask(), define_err::Kinds::ALL);
/// ```
#[must_use]
pub fn kinds_mask() -> Kinds {
    state::kinds_mask()
}

/// Counters for what the facade did not deliver normally.
///
/// # Examples
///
/// ```
/// assert_eq!(define_err::stats(), define_err::FacadeStats::default());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub struct FacadeStats {
    /// Warnings from sites past the 512-site table (delivered every time).
    pub site_table_overflow: u64,
    /// Events dropped because a reporter called a verb inside `receive`.
    pub reentrant_dropped: u64,
    /// Panics caught inside `receive`, a panicking payload drop included.
    pub reporter_panics: u64,
}

/// The facade's counters. Relaxed loads; for health checks and tests.
///
/// # Examples
///
/// ```
/// assert_eq!(define_err::stats().reporter_panics, 0);
/// ```
#[must_use]
pub fn stats() -> FacadeStats {
    state::stats()
}

/// What a dynamic error's creator hands a [`CreationHook`].
///
/// # Examples
///
/// ```
/// use define_err::CreationInfo;
///
/// let info = CreationInfo::new(define_err::here(), None, "app::Error");
/// assert_eq!(info.type_name(), "app::Error");
/// ```
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct CreationInfo<'a> {
    location: Location,
    code: Option<Code>,
    type_name: &'a str,
}

impl<'a> CreationInfo<'a> {
    /// The facts of an error being created.
    #[must_use]
    pub fn new(location: Location, code: Option<Code>, type_name: &'a str) -> CreationInfo<'a> {
        CreationInfo {
            location,
            code,
            type_name,
        }
    }

    /// Where the error is being created.
    #[must_use]
    pub fn location(&self) -> Location {
        self.location
    }

    /// The error's code, if it has one.
    #[must_use]
    pub fn code(&self) -> Option<Code> {
        self.code
    }

    /// The error's type name.
    #[must_use]
    pub fn type_name(&self) -> &'a str {
        self.type_name
    }
}

/// Where a [`CreationHook`] writes what it adds.
pub trait Annotate {
    /// Adds a key and a value.
    fn field(&mut self, key: &'static str, value: &dyn fmt::Display);

    /// Adds kinds.
    fn kind(&mut self, kinds: Kinds);
}

/// An observer that adds scope data (a request id, a tenant, a security kind)
/// when a dynamic error is created.
///
/// Only dynamic-error libraries call it; errors generated by
/// `define_errors!` never do, because building a typed error has no side
/// effects. The caller of the hook owns its guards. A hook must return in
/// microseconds and must not create a dynamic error itself.
///
/// # Examples
///
/// ```
/// use define_err::{Annotate, CreationHook, CreationInfo};
///
/// struct RequestScope;
///
/// impl CreationHook for RequestScope {
///     fn on_create(&self, _info: &CreationInfo<'_>, out: &mut dyn Annotate) {
///         out.field("request_id", &42);
///     }
/// }
///
/// static SCOPE: RequestScope = RequestScope;
/// define_err::set_creation_hook(&SCOPE).unwrap();
/// assert!(define_err::creation_hook().is_some());
/// ```
pub trait CreationHook: Send + Sync {
    /// Called when a dynamic error is created.
    fn on_create(&self, info: &CreationInfo<'_>, out: &mut dyn Annotate);
}

/// Installs the creation hook, once, like [`set_reporter`].
///
/// # Errors
///
/// Returns [`InstallError`] with [`Slot::CreationHook`] when a hook is
/// already installed.
#[cfg(target_has_atomic = "32")]
#[cfg_attr(docsrs, doc(cfg(target_has_atomic = "32")))]
#[track_caller]
pub fn set_creation_hook(hook: &'static dyn CreationHook) -> Result<(), InstallError> {
    state::install_hook(hook, crate::here()).map_err(|installed_at| InstallError {
        slot: Slot::CreationHook,
        installed_at,
    })
}

/// The installed creation hook. One load when the slot is empty.
///
/// # Examples
///
/// ```
/// assert!(define_err::creation_hook().is_none());
/// ```
#[inline]
#[must_use]
pub fn creation_hook() -> Option<&'static dyn CreationHook> {
    state::creation_hook()
}
