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
//! A reporting facade any crate can call (`report`, `notice`, `warn`,
//! `trigger`) arrives in `0.5.0`. Errors are values: nothing is reported
//! unless code reports it, and building an error never runs any code.
//!
//! ## Features
//!
//! | Feature | Default | Brings |
//! |---|---|---|
//! | `std` | yes | implies `alloc`; `ErrorInfo` and `Category::from_io_kind` for `std::io::Error` |
//! | `alloc` | through `std` | boxed `dyn Error` sources (`Box<dyn Error + Send + Sync>` and friends) |
//! | `testing` | no | implies `std`; from `0.5.0`, a capturing reporter for tests |
//!
//! With `--no-default-features` the crate uses `core` only, and every
//! `define_errors!` form works except boxed sources.
//!
//! ## Status
//!
//! `0.4.0`: the thiserror-parity core, plus codes, the catalog, severity,
//! categories, kinds, and the identity probe. The public API of 1.0 is written in
//! `docs/API.md` and what the macro expands to in `docs/GENERATED-CODE.md`
//! (both in the [repository](https://github.com/jamesgober/define-err)).

#![no_std]
#![doc(html_root_url = "https://docs.rs/define-err")]
#![cfg_attr(docsrs, feature(doc_cfg))]
// `deny`, not `forbid`: the reporting facade (0.5) keeps one write-once slot
// in a single allowed module, `src/facade/slot.rs` (dev/PLAN.md, D-003).
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
mod info;
mod kinds;
mod location;
mod macros;
mod probe;
mod secret;
mod severity;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub use crate::{
    category::{Action, Category, RetryAdvice},
    code::{Code, CodeParseError, Namespace, NamespaceStr},
    info::{catalog, CodeInfo, CodeState, ErrorInfo, UrlDisplay},
    kinds::{Kinds, KindsIter},
    location::{here, Location},
    probe::{identify, innermost_code, Identity},
    secret::Secret,
    severity::{Band, LevelNames, Severity},
};

/// The one import a caller needs: `use define_err::prelude::*;`.
///
/// Brings [`define_errors!`], [`ErrorInfo`] (the trait behind `.code()`,
/// `.category()`, `.public_message()`, and the other metadata methods),
/// [`Code`], [`Severity`], [`Category`], [`Kinds`], [`Action`], [`Secret`],
/// and [`Location`]. The reporting verbs (from `0.5.0`) are not in the
/// prelude: they are called by path, `define_err::report(&err)`.
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
