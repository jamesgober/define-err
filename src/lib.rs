//! # define_err
//!
//! Typed error definitions for Rust, without a proc macro. A library says
//! what its errors are with one `define_errors!` block: each variant is one
//! line, `Name { field: Type } => "message"`, and the macro writes the enum,
//! `Debug`, `Display`, `core::error::Error` with `source()`, and the `From`
//! conversions asked for: the code a person would write by hand, and the
//! code thiserror generates.
//!
//! Everything beyond that is optional and costs nothing until it is used:
//! stable error codes (`DB-0042`) with a compile-time catalog, severity on
//! the OpenTelemetry scale, categories that map to HTTP, gRPC, and exit
//! codes, kind masks, public and private messages, retry advice, secret
//! redaction, an owned error record, and a reporting facade any crate can
//! call (`report`, `notice`, `warn`, `trigger`) at the cost of one atomic
//! load when no reporter is installed.
//!
//! The crate is `no_std` with optional `alloc` and `std` layers, and has no
//! dependencies.
//!
//! ## Tier 1
//!
//! ```
//! println!("define-err {}", define_err::VERSION);
//! ```
//!
//! The Tier 1 entry points arrive from `0.3.0` on:
//!
//! ```text
//! define_errors! {
//!     pub enum PageError {
//!         NotFound { page: u64 } => "page {page} not found",
//!         Io(#[from] source: std::io::Error) => "page file i/o failed",
//!     }
//! }
//!
//! define_err::report(&err);                        // never changes control flow
//! define_err::warn("cache file is stale");         // once per call site
//! ```
//!
//! ## Features
//!
//! | Feature | Default | Brings |
//! |---|---|---|
//! | `std` | yes | `std::io::Error` mappings; reentrancy guard and panic isolation in the facade. Implies `alloc`. |
//! | `alloc` | through `std` | `ErrorRecord`, boxed `dyn Error` sources, `set_boxed_reporter`. |
//! | `testing` | no | a capturing reporter for tests. Implies `std`. |
//!
//! With `--no-default-features` the crate uses `core` only.
//!
//! ## Status
//!
//! Scaffold. The design and the version plan are in `dev/PLAN.md` and
//! `dev/ROADMAP.md` in the [repository](https://github.com/jamesgober/define-err);
//! the public API is written in `docs/API.md` at the `0.2.0` design lock. The
//! thiserror-parity core ships in `0.3.0`.

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
