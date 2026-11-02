//! Codes and metadata: the 0.4 exit example (dev/PLAN.md 7.4.4) and edge
//! cases E-33 to E-46, E-87 to E-89.

#![allow(deprecated)]

use std::{error::Error as _, time::Duration};

use define_err::{
    catalog, define_errors, identify, innermost_code, prelude::*, CodeInfo, CodeState, Kinds,
};

pub const BILLING: Kinds = Kinds::project::<0>();

define_errors! {
    namespace = "DB";
    docs = "https://docs.hivedb.com/errors/{code}";
    retired = [9, 12];

    /// Errors from the query layer.
    pub enum DbError {
        /// The generic database error.
        #[code(0, help = "Check the logs for the underlying cause.")]
        Unspecified => "database error",

        #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        #[retry_after(secs = 2)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, name = CORRUPT, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt { page: u64, #[location] at: Location } => "page {page} failed its checksum",

        #[code(3, help = "The pool closed while the request waited.")]
        #[category(Unavailable)]
        PoolClosed { #[retry_after] wait: core::time::Duration } => "connection pool closed",

        #[code(4, name = OLD_QUOTA, help = "Use DB-0010 (DbError::Quota) instead.", deprecated = "2.1.0", replacement = 10)]
        #[category(ResourceExhausted)]
        OldQuota => "quota exceeded",

        #[code(10, help = "Raise the tenant quota or wait for the window to reset.")]
        #[category(ResourceExhausted)]
        #[kinds(BILLING)]
        #[action(Operator)]
        Quota { tenant: u64 } => "tenant {tenant} is over its quota",

        #[code(11, help = "See the wrapped I/O error.")]
        #[category(Unavailable)]
        Io(#[from] source: std::io::Error, #[location] at: Location) => "i/o error: {source}",
    }

    /// Errors from the planner, same namespace, same duplicate check.
    pub enum PlanError {
        #[code(20, help = "Simplify the query or raise planner.max_joins.", title = "Too many joins")]
        #[category(InvalidArgument)]
        TooManyJoins { joins: u32 } => "query has {joins} joins",

        #[code(21, help = "Plans wait at most this long.")]
        #[retryable]
        #[retry_after(ms = 250)]
        Busy => "planner busy",

        #[code(22, help = "Never retried.")]
        #[category(Unavailable)]
        #[retryable(false)]
        Gone => "planner gone",

        #[code(23, help = "Optional delay.")]
        Later { #[retry_after] after: Option<core::time::Duration> } => "later",

        Uncoded => "no code here",
    }
}

#[test]
fn test_codes_and_named_constants() {
    let err = DbError::Timeout { after_ms: 5000 };
    assert_eq!(err.code(), Some(DbError::TIMEOUT));
    assert_eq!(DbError::TIMEOUT.to_string(), "DB-0001");
    assert_eq!(DbError::CORRUPT.number(), 2);
    assert!(DbError::Unspecified.code().unwrap().is_generic());
    assert_eq!(PlanError::Uncoded.code(), None);
    assert_eq!(
        PlanError::TooManyJoins { joins: 9 }
            .code()
            .unwrap()
            .to_string(),
        "DB-0020"
    );
}

#[test]
fn test_categories_and_their_defaults() {
    let err = DbError::Timeout { after_ms: 5000 };
    assert_eq!(err.category(), Category::DeadlineExceeded);
    assert_eq!(err.category().http_status(), 504);
    assert_eq!(err.severity(), Severity::ERROR);
    assert_eq!(err.action(), Action::Retry);
    assert!(err.is_retryable());
    assert_eq!(
        err.public_message(),
        "The database is busy. Try again shortly."
    );
    assert_eq!(err.help(), Some("Raise pool.size or lower the load."));

    let uncoded = PlanError::Uncoded;
    assert_eq!(uncoded.category(), Category::Unknown);
    assert_eq!(uncoded.public_message(), "An unexpected error occurred.");
    assert_eq!(uncoded.help(), None);
}

#[test]
fn test_explicit_severity_kinds_action() {
    let corrupt = DbError::Corrupt {
        page: 9,
        at: define_err::here(),
    };
    assert_eq!(corrupt.severity(), Severity::CRITICAL);
    assert!(corrupt.kinds().contains(Kinds::SECURITY));
    assert!(corrupt.kinds().contains(Kinds::ERROR));
    assert_eq!(corrupt.public_message(), "An internal error occurred.");
    assert_eq!(corrupt.action(), Action::PageOnCall);
    assert!(corrupt.location().is_some());

    let quota = DbError::Quota { tenant: 1 };
    assert_eq!(quota.kinds(), BILLING);
    assert_eq!(quota.action(), Action::Operator);
    assert_eq!(quota.category().http_status(), 429);
}

#[test]
fn test_retry_metadata() {
    assert_eq!(
        DbError::Timeout { after_ms: 1 }.retry_after(),
        Some(Duration::from_secs(2))
    );
    let wait = Duration::from_millis(1500);
    assert_eq!(DbError::PoolClosed { wait }.retry_after(), Some(wait));
    assert_eq!(
        PlanError::Busy.retry_after(),
        Some(Duration::from_millis(250))
    );
    assert!(PlanError::Busy.is_retryable());
    assert!(!PlanError::Gone.is_retryable());
    assert!(PlanError::Gone.category().is_transient());
    assert_eq!(PlanError::Later { after: None }.retry_after(), None);
    assert_eq!(
        PlanError::Later { after: Some(wait) }.retry_after(),
        Some(wait)
    );
    assert_eq!(PlanError::Uncoded.retry_after(), None);
}

#[test]
fn test_catalog_entries_titles_docs_and_retired() {
    let catalog = DbError::CATALOG;
    // Seven codes plus the two retired numbers on the block's first type.
    assert_eq!(catalog.len(), 9);
    let timeout = catalog.iter().find(|e| e.code == DbError::TIMEOUT).unwrap();
    assert_eq!(timeout.title, "Timeout");
    assert_eq!(timeout.type_name, "DbError::Timeout");
    assert_eq!(timeout.category, Category::DeadlineExceeded);
    assert_eq!(timeout.public, "The database is busy. Try again shortly.");
    assert_eq!(
        timeout.url().unwrap().to_string(),
        "https://docs.hivedb.com/errors/DB-0001"
    );
    assert!(timeout.is_active());

    let retired: Vec<u32> = catalog
        .iter()
        .filter(|e| e.state == CodeState::Retired)
        .map(|e| e.code.number())
        .collect();
    assert_eq!(retired, [9, 12]);
    assert!(catalog
        .iter()
        .filter(|e| e.state == CodeState::Retired)
        .all(|e| e.type_name.is_empty()));

    assert_eq!(PlanError::CATALOG.len(), 4);
    assert_eq!(PlanError::CATALOG[0].title, "Too many joins");
}

#[test]
fn test_deprecated_code_state_and_constant() {
    let old = DbError::OldQuota;
    let info = old.code_info().unwrap();
    assert_eq!(
        info.state,
        CodeState::Deprecated {
            since: "2.1.0",
            replacement: Some(DbError::CATALOG[5].code)
        }
    );
    assert!(!info.is_active());
    assert_eq!(DbError::OLD_QUOTA.to_string(), "DB-0004");
    assert_eq!(old.to_string(), "quota exceeded");
}

#[test]
fn test_code_info_matches_catalog() {
    let err = DbError::Quota { tenant: 1 };
    let info = err.code_info().unwrap();
    let entry =
        catalog::find(&[DbError::CATALOG, PlanError::CATALOG], err.code().unwrap()).unwrap();
    assert_eq!(info.code, entry.code);
    assert_eq!(info.help, entry.help);
    assert_eq!(info.kinds, BILLING);
    assert!(PlanError::Uncoded.code_info().is_none());
}

#[test]
fn test_catalogs_are_unique_across_the_block() {
    assert!(catalog::check_unique(&[DbError::CATALOG, PlanError::CATALOG]).is_ok());
    let dup = catalog::check_unique(&[DbError::CATALOG, DbError::CATALOG]).unwrap_err();
    assert_eq!(dup.first, dup.second);
}

#[test]
fn test_from_conversion_with_metadata() {
    let err: DbError = std::io::Error::other("disk").into();
    assert_eq!(err.code().unwrap().number(), 11);
    assert_eq!(err.category(), Category::Unavailable);
    assert!(err.source().is_some());
    assert!(err.location().is_some());
}

#[test]
fn test_identify_reads_every_generated_variant() {
    let corrupt = DbError::Corrupt {
        page: 1,
        at: define_err::here(),
    };
    let id = identify(&corrupt).unwrap();
    assert_eq!(id.code, Some(DbError::CORRUPT));
    assert_eq!(id.code.unwrap().to_string(), "DB-0002");
    assert_eq!(id.severity, Severity::CRITICAL);
    assert_eq!(id.category, Category::DataLoss);
    assert_eq!(id.kinds, Kinds::ERROR | Kinds::SECURITY);
    assert!(!id.has_public);
    assert_eq!(id.type_name, "DbError::Corrupt");

    let timeout = identify(&DbError::Timeout { after_ms: 1 }).unwrap();
    assert!(timeout.has_public);
    assert!(identify(&PlanError::Uncoded).unwrap().code.is_none());
    assert!(identify(&std::fmt::Error).is_none());
}

define_errors! {
    namespace = "APP";

    pub enum AppError {
        #[code(1, help = "See the storage error.")]
        #[category(Unavailable)]
        Storage(#[from] inner: DbError) => "storage failed",

        Wrapped(#[from] inner: PlanError) => transparent,

        #[category(Internal)]
        Foreign(#[from] inner: std::fmt::Error) => transparent,
    }
}

#[test]
fn test_innermost_code_walks_the_chain() {
    let err = AppError::from(DbError::Quota { tenant: 1 });
    let (code, id) = innermost_code(&err).unwrap();
    assert_eq!(code.to_string(), "DB-0010");
    assert_eq!(id.type_name, "DbError::Quota");
    assert_eq!(err.code().unwrap().to_string(), "APP-0001");
}

#[test]
fn test_transparent_forwards_metadata_and_probe() {
    let err = AppError::from(PlanError::TooManyJoins { joins: 3 });
    assert_eq!(err.code().unwrap().to_string(), "DB-0020");
    assert_eq!(err.category(), Category::InvalidArgument);
    assert_eq!(err.type_name(), "PlanError::TooManyJoins");
    assert_eq!(identify(&err).unwrap().type_name, "PlanError::TooManyJoins");

    // A transparent wrapper over a foreign error keeps its own metadata.
    let foreign = AppError::from(std::fmt::Error);
    assert_eq!(foreign.category(), Category::Internal);
    assert_eq!(foreign.code(), None);
    let id = identify(&foreign).unwrap();
    assert!(id.transparent);
    assert_eq!(id.type_name, "AppError::Foreign");
}

define_errors! {
    namespace = "DIS", width = 5;

    pub enum DistroError {
        #[code(1, help = "Fix the named field in distro.noml.")]
        #[category(InvalidArgument)]
        #[action(FixConfig)]
        Config { field: String, reason: String } => "configuration error [{field}]: {reason}",

        #[code(3, help = "Check permissions and free space on the path.")]
        #[category(Unavailable)]
        Io { operation: &'static str, path: String, #[source] source: std::io::Error } =>
            "io error during {operation} on {path}: {source}",
    }
}

#[test]
fn test_five_digit_codes() {
    let err = DistroError::Config {
        field: "port".into(),
        reason: "not a number".into(),
    };
    assert_eq!(err.code().unwrap().to_string(), "DIS-00001");
    assert_eq!(err.action(), Action::FixConfig);
    assert_eq!(
        identify(&err).unwrap().code.unwrap().to_string(),
        "DIS-00001"
    );
}

// Metadata in another order, with lint attributes: the general path.
define_errors! {
    namespace = "GEN";

    pub enum General {
        #[allow(dead_code)]
        #[category(NotFound)]
        /// Documented after a metadata attribute.
        #[code(1, help = "Look elsewhere.")]
        Missing => "missing",

        #[retry_after(secs = 1)]
        #[code(2, help = "Wait.")]
        #[severity(22)]
        Slow(u8) => "slow",
    }

    /// Metadata on a struct goes on the struct.
    #[code(3, help = "Retry later.")]
    #[category(Unavailable)]
    pub struct Down {
        pub service: String,
    } => "{service} is down";
}

#[test]
fn test_general_path_metadata_and_structs() {
    assert_eq!(General::Missing.code().unwrap().to_string(), "GEN-0001");
    assert_eq!(General::Missing.category(), Category::NotFound);
    assert_eq!(General::Slow(1).severity().get(), 22);
    assert_eq!(General::Slow(1).retry_after(), Some(Duration::from_secs(1)));
    let down = Down {
        service: "auth".into(),
    };
    assert_eq!(down.to_string(), "auth is down");
    assert_eq!(down.code().unwrap().to_string(), "GEN-0003");
    assert_eq!(down.category().http_status(), 503);
    assert_eq!(Down::CATALOG.len(), 1);
    assert_eq!(General::CATALOG.len(), 2);
}

define_errors! {
    namespace = "LIT";

    pub enum Literals {
        #[code(1_000, help = "Underscores are accepted.")]
        Thousand => "thousand",
        #[code(0x2A, help = "Hex is accepted.")]
        Hex => "hex",
        #[code(16_777_215, help = "The largest number.")]
        Max => "max",
    }
}

#[test]
fn test_code_literal_forms() {
    assert_eq!(Literals::Thousand.code().unwrap().to_string(), "LIT-1000");
    assert_eq!(Literals::Hex.code().unwrap().to_string(), "LIT-0042");
    assert_eq!(Literals::Max.code().unwrap().number(), 16_777_215);
}

define_errors! {
    namespace = "SAME";
    pub enum SameNumbers { #[code(1, help = "a")] A => "a" }
}

define_errors! {
    namespace = "OTHER";
    pub enum OtherNumbers { #[code(1, help = "b")] B => "b" }
}

#[test]
fn test_same_number_in_two_blocks_is_allowed() {
    assert_eq!(
        SameNumbers::A.code().unwrap().number(),
        OtherNumbers::B.code().unwrap().number()
    );
    assert_ne!(SameNumbers::A.code(), OtherNumbers::B.code());
}

#[test]
fn test_hand_written_catalog_entries() {
    const ENTRY: CodeInfo = CodeInfo::new(
        define_err::code!("PROTO", 1),
        "Frame too large",
        "Lower max_frame.",
    )
    .with_category(Category::ResourceExhausted);
    assert_eq!(ENTRY.severity, Severity::ERROR);
    assert!(catalog::find(&[&[ENTRY]], define_err::code!("PROTO", 1)).is_some());
}
