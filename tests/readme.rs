//! The README's quick start, kept compiling.

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
