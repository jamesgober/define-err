//! The README's quick start and codes example, kept compiling.

use define_err::prelude::*;

define_errors! {
    /// Errors from the page store.
    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",
        ShortRead(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
        Io(#[from] source: std::io::Error) => "page file i/o failed",
        Token { token: Secret<String> } => "token {token} rejected",
    }
}

define_errors! {
    namespace = "DB";
    docs = "https://docs.example.com/errors/{code}";

    pub enum DbError {
        NotFound { page: u64 } => "page {page} not found",

        #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt { page: u64 } => "page {page} failed its checksum",
    }
}

fn open(path: &str) -> Result<Vec<u8>, PageError> {
    Ok(std::fs::read(path)?)
}

#[test]
fn test_readme_quick_start() {
    assert_eq!(
        PageError::NotFound { page: 7 }.to_string(),
        "page 7 not found"
    );
    assert_eq!(
        PageError::ShortRead(7, 12).to_string(),
        "short read on page 7: got 12 bytes"
    );
    assert!(open("/does/not/exist").is_err());
    let token = PageError::Token {
        token: Secret::new("s3cr3t".into()),
    };
    assert_eq!(token.to_string(), "token [redacted] rejected");
}

#[test]
fn test_readme_codes_and_metadata() {
    let err = DbError::Timeout { after_ms: 5000 };
    assert_eq!(err.code(), Some(DbError::TIMEOUT));
    assert_eq!(DbError::TIMEOUT.to_string(), "DB-0001");
    assert_eq!(err.category().http_status(), 504);
    assert_eq!(
        err.public_message(),
        "The database is busy. Try again shortly."
    );
    assert!(err.is_retryable());

    let corrupt = DbError::Corrupt { page: 7 };
    assert_eq!(corrupt.severity(), Severity::CRITICAL);
    assert!(corrupt.kinds().contains(Kinds::SECURITY));

    assert_eq!(DbError::CATALOG.len(), 2);
    assert_eq!(
        DbError::CATALOG[0].url().unwrap().to_string(),
        "https://docs.example.com/errors/DB-0001"
    );

    let boxed: Box<dyn std::error::Error> = Box::new(err);
    assert_eq!(
        define_err::identify(&*boxed).unwrap().code,
        Some(DbError::TIMEOUT)
    );
    assert!(DbError::NotFound { page: 1 }.code().is_none());
}
