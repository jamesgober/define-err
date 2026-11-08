//! # define_err
//!
//! Typed error definitions for Rust, without a proc macro. A library says
//! what its errors are with one `define_errors!` block: each variant is one
//! line, `Name { field: Type } => "message"`, and the macro writes the enum,
//! `Debug`, `Display`, `core::error::Error` with `source()`, and the `From`
//! conversions asked for: the code a person would write by hand, and the
//! code thiserror generates.
//!
//! ## Tier 1
//!
//! ```
//! use define_err::prelude::*;
//!
//! define_errors! {
//!     /// Errors from loading the configuration.
//!     pub enum ConfigError {
//!         Missing { key: String } => "missing key {key}",
//!         BadValue { key: String, file: String } => "bad value for {key} in {file}",
//!         Read(#[from] source: std::io::Error) => "cannot read the configuration: {source}",
//!     }
//! }
//!
//! fn load(path: &str) -> Result<String, ConfigError> {
//!     let text = std::fs::read_to_string(path)?;          // From<io::Error>
//!     if text.is_empty() {
//!         return Err(ConfigError::Missing { key: "name".into() });
//!     }
//!     Ok(text)
//! }
//!
//! let err = load("/does/not/exist").unwrap_err();
//! assert!(err.to_string().starts_with("cannot read the configuration"));
//! assert!(std::error::Error::source(&err).is_some());
//! ```
//!
//! Each variant is one line; a plain error type needs nothing else. Enums
//! get `#[non_exhaustive]` (unless marked `#[exhaustive]`) so they can grow
//! without breaking callers. The full grammar is on [`define_errors!`].
//!
//! [`Secret<T>`](Secret) keeps tokens and passwords out of every `Display`
//! and `Debug` output:
//!
//! ```
//! use define_err::prelude::*;
//!
//! define_errors! {
//!     pub enum AuthError {
//!         BadToken { token: Secret<String> } => "token {token} rejected",
//!     }
//! }
//!
//! let err = AuthError::BadToken { token: Secret::new("s3cr3t".into()) };
//! assert_eq!(err.to_string(), "token [redacted] rejected");
//! ```
//!
//! ## Codes and metadata
//!
//! Each extra is one attribute on the variant that needs it: a stable code
//! (`DB-0001`) with help text, a [`Category`] that maps to HTTP, gRPC, and
//! exit codes, a [`Severity`] on the OpenTelemetry scale, [`Kinds`], a
//! public message, and retry advice. [`ErrorInfo`] reads them back.
//!
//! ```
//! use define_err::prelude::*;
//!
//! define_errors! {
//!     namespace = "DB";
//!
//!     pub enum DbError {
//!         #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
//!         #[category(DeadlineExceeded)]
//!         Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",
//!
//!         #[code(2, help = "Restore the page from a replica, then run a scrub.")]
//!         #[category(DataLoss)]
//!         #[severity(CRITICAL)]
//!         Corrupt { page: u64 } => "page {page} failed its checksum",
//!     }
//! }
//!
//! let err = DbError::Timeout { after_ms: 5000 };
//! assert_eq!(err.code(), Some(DbError::TIMEOUT));
//! assert_eq!(err.code().unwrap().to_string(), "DB-0001");
//! assert_eq!(err.category().http_status(), 504);
//! assert!(err.is_retryable());
//! assert_eq!(DbError::CATALOG.len(), 2);
//! ```
//!
//! Every value is a constant built at compile time, and every mistake (a
//! duplicate number, a code without help) is a compile error. Libraries
//! that hold errors as `&dyn Error` read the same facts with [`identify`].
//!
//! ## Reporting
//!
//! Any crate can call the verbs ([`report`], [`notice()`], [`warn()`],
//! [`trigger()`], [`deprecated()`], [`soft`]); the application installs one
//! [`Reporter`] at start-up with [`set_reporter`]. With nothing installed, a
//! verb costs one atomic load.
//!
//! ```
//! use define_err::{Event, Reporter};
//!
//! struct StderrReporter;
//!
//! impl Reporter for StderrReporter {
//!     fn receive(&self, event: &Event<'_>) {
//!         let at = event.location();
//!         match (event.error(), event.message()) {
//!             (Some(err), _) => eprintln!("{} {}:{} {err}", event.severity(), at.file(), at.line()),
//!             (None, Some(msg)) => eprintln!("{} {}:{} {msg}", event.severity(), at.file(), at.line()),
//!             (None, None) => {}
//!         }
//!     }
//! }
//!
//! static REPORTER: StderrReporter = StderrReporter;
//!
//! define_err::set_reporter(&REPORTER).unwrap();
//! define_err::warn("cache file is stale, rebuilding");  // once per call site
//! ```
//!
//! `ErrorRecord` (with `alloc`) is the owned snapshot for anything that
//! outlives the event.
//!
//! ## What can never be observed
//!
//! Errors are values; there is no throw event. The facade sees an error only
//! when code calls one of its verbs. It cannot see an error discarded with
//! `let _ =`, `.ok()`, or `unwrap_or`, or a typed error that is built and
//! never reported: building an error never runs any code.
//!
//! ## Features
//!
//! | Feature | Default | Brings |
//! |---|---|---|
//! | `std` | yes | implies `alloc`; `ErrorInfo` and `Category::from_io_kind` for `std::io::Error`; the facade's guards |
//! | `alloc` | through `std` | `ErrorRecord`, boxed `dyn Error` sources, `set_boxed_reporter` |
//! | `testing` | no | implies `std`; `testing::capture`, a capturing reporter for tests |
//!
//! With `--no-default-features` the crate uses `core` only, and every
//! `define_errors!` form works except boxed sources. `std` adds the facade's
//! reentrancy guard and panic isolation. On targets without compare-and-swap
//! the verbs compile to nothing and `set_reporter` does not exist.
//!
//! ## Status
//!
//! `0.9.0`, the beta: feature complete, measured, hardened, checked against
//! real migrations, and in its pre-1.0 polish pass. The public API of 1.0 is written in `docs/API.md` and what the macro expands to in
//! `docs/GENERATED-CODE.md` (both in the
//! [repository](https://github.com/jamesgober/define-err)).

#![no_std]
#![doc(html_root_url = "https://docs.rs/define-err")]
#![cfg_attr(docsrs, feature(doc_cfg))]
// `deny`, not `forbid`: the reporting facade keeps one write-once slot in a
// single allowed module, `src/facade/slot.rs` (dev/PLAN.md, D-003).
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
#![deny(unused_results)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::dbg_macro,
        clippy::print_stdout,
        clippy::print_stderr,
        clippy::undocumented_unsafe_blocks
    )
)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod category;
mod code;
mod facade;
mod info;
mod kinds;
mod location;
mod macros;
mod probe;
#[cfg(feature = "alloc")]
mod record;
mod secret;
mod severity;

/// Capturing what the facade delivers, for tests of code that reports.
#[cfg(all(feature = "testing", target_has_atomic = "32"))]
#[cfg_attr(docsrs, doc(cfg(feature = "testing")))]
pub mod testing;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

#[cfg(feature = "alloc")]
pub use crate::facade::report_record;
#[cfg(all(feature = "alloc", target_has_atomic = "32"))]
pub use crate::facade::set_boxed_reporter;
#[cfg(target_has_atomic = "32")]
pub use crate::facade::{set_creation_hook, set_reporter};
#[cfg(feature = "alloc")]
pub use crate::record::{ChainEntry, ErrorRecord, RecordLimits, TrailEntry, TrailStep};
pub use crate::{
    category::{Action, Category, RetryAdvice},
    code::{Code, CodeParseError, Namespace, NamespaceStr},
    facade::{
        creation_hook, deprecated, flush, is_installed, kinds_mask, min_severity, notice,
        notice_with, report, report_with, reporter, reporter_installed_at, set_kinds_mask,
        set_min_severity, set_warn_mode, soft, stats, trigger, warn, warn_mode, warn_with,
        Annotate, CreationHook, CreationInfo, Event, FacadeStats, InstallError, Reporter, Slot,
        Verb, WarnMode,
    },
    info::{catalog, CodeInfo, CodeState, ErrorInfo, UrlDisplay},
    kinds::{Kinds, KindsIter},
    location::{here, short_file, Location},
    probe::{identify, innermost_code, Identity},
    secret::Secret,
    severity::{Band, LevelNames, Severity},
};

/// The one import a caller needs: `use define_err::prelude::*;`.
///
/// Brings [`define_errors!`], [`ErrorInfo`] (the trait behind `.code()`,
/// `.category()`, `.public_message()`, and the other metadata methods),
/// [`Code`], [`Severity`], [`Category`], [`Kinds`], [`Action`], [`Secret`],
/// and [`Location`]. The reporting verbs are not in the prelude: they are
/// called by path, `define_err::report(&err)`.
///
/// # Examples
///
/// ```
/// use define_err::prelude::*;
///
/// define_errors! {
///     pub struct Cancelled => "operation cancelled";
/// }
///
/// assert_eq!(Cancelled.type_name(), "Cancelled");
/// ```
pub mod prelude {
    pub use crate::{
        define_errors, Action, Category, Code, ErrorInfo, Kinds, Location, Secret, Severity,
    };
}

/// Crate version string, populated by Cargo at build time.
///
/// # Examples
///
/// ```
/// assert!(!define_err::VERSION.is_empty());
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn test_version_matches_manifest() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        // Pre-release and build metadata (`1.0.0-rc.1`, `+build`) follow the core.
        let core = VERSION.split(['-', '+']).next().unwrap_or_default();
        assert_eq!(core.split('.').count(), 3, "expected MAJOR.MINOR.PATCH");
        assert!(
            core.split('.').all(|part| part.parse::<u64>().is_ok()),
            "every version component is a number"
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_alloc_feature_links_alloc() {
        let text = alloc::string::String::from("DB-0042");
        assert_eq!(text.len(), 7);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_std_feature_implies_alloc() {
        const { assert!(cfg!(feature = "alloc"), "`std` enables `alloc`") };
        let kind = std::io::ErrorKind::NotFound;
        assert_eq!(kind, std::io::ErrorKind::NotFound);
    }

    #[test]
    #[cfg(feature = "testing")]
    fn test_testing_feature_implies_std() {
        const { assert!(cfg!(feature = "std"), "`testing` enables `std`") };
        let name = std::string::String::from("capture");
        assert_eq!(name.len(), 7);
    }

    #[test]
    #[cfg(not(feature = "alloc"))]
    fn test_core_only_build_has_no_alloc() {
        const { assert!(!cfg!(feature = "std"), "`std` cannot be on without `alloc`") };
        assert_eq!(
            core::mem::size_of::<&str>(),
            2 * core::mem::size_of::<usize>()
        );
    }
}
