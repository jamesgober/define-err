//! The same error type before and after moving from thiserror. The "before"
//! is in a comment so this example needs nothing but define-err; the full
//! rules are in docs/MIGRATION.md.
//!
//! ```text
//! cargo run --example migration_before_after
//! ```

// Before:
//
//     #[derive(Debug, thiserror::Error)]
//     #[non_exhaustive]
//     pub enum PageError {
//         #[error("page {page} not found")]
//         NotFound { page: u64 },
//         #[error("invalid usage: {0}")]
//         InvalidUsage(String),
//         #[error("size {size} not in {min}..={max}", min = MIN, max = MAX)]
//         BadSize { size: u32 },
//         #[error("page i/o error: {0}")]
//         Io(#[from] std::io::Error),
//         #[error(transparent)]
//         Other(#[from] std::fmt::Error),
//     }

const MIN: u32 = 512;
const MAX: u32 = 65_536;

define_err::define_errors! {
    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",
        InvalidUsage(detail: String) => "invalid usage: {detail}",
        BadSize { size: u32 } => ("size {size} not in {min}..={max}", min = MIN, max = MAX),
        Io(#[from] source: std::io::Error) => "page i/o error: {source}",
        Other(#[from] inner: std::fmt::Error) => transparent,
    }
}

fn main() {
    println!("{}", PageError::NotFound { page: 7 });
    println!("{}", PageError::InvalidUsage("empty key".into()));
    println!("{}", PageError::BadSize { size: 3 });
    println!("{}", PageError::from(std::fmt::Error));
}
