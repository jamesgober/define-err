//! Edge cases of the identity probe and the `io::Error` mapping that need a
//! chain or a long name: E-61, E-88, E-89 (dev/PLAN.md 12.3).

#![cfg(feature = "std")]

use core::fmt;
use std::error::Error;

use define_err::{define_errors, identify, innermost_code, Category};

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(7, help = "Restore the page from a replica.")]
        Corrupt => "page failed its checksum",
    }
}

/// A foreign wrapper: one link of a chain, with no probe of its own.
#[derive(Debug)]
struct Link(Box<dyn Error + 'static>);

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("link")
    }
}

impl Error for Link {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

/// A chain whose coded error sits at `depth` (0 is the outermost entry).
fn chain(depth: usize) -> Box<dyn Error + 'static> {
    let mut error: Box<dyn Error + 'static> = Box::new(DbError::Corrupt);
    for _ in 0..depth {
        error = Box::new(Link(error));
    }
    error
}

#[test]
fn test_e89_innermost_code_depth_limit() {
    for depth in [0, 3, 63] {
        let error = chain(depth);
        let (code, id) = innermost_code(&*error).unwrap_or_else(|| panic!("depth {depth}"));
        assert_eq!(code.to_string(), "DB-0007");
        assert_eq!(id.type_name, "DbError::Corrupt");
    }
    assert!(innermost_code(&*chain(64)).is_none());
}

// `T::` plus a 125-byte variant name: a type name of exactly 128 bytes.
define_errors! {
    pub enum T {
        Vaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa => "long",
    }
}

#[test]
fn test_e88_type_name_of_128_bytes_is_encoded() {
    let error = T::Vaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;
    let id = identify(&error).unwrap();
    assert_eq!(id.type_name.len(), 128);
    assert!(id.type_name.starts_with("T::V"));
}

#[test]
fn test_e61_from_io_kind_for_every_stable_kind() {
    use std::io::ErrorKind::*;
    let mapped = [
        (NotFound, Category::NotFound),
        (PermissionDenied, Category::PermissionDenied),
        (ConnectionRefused, Category::Unavailable),
        (ConnectionReset, Category::Unavailable),
        (ConnectionAborted, Category::Unavailable),
        (NotConnected, Category::Unavailable),
        (AddrInUse, Category::AlreadyExists),
        (AddrNotAvailable, Category::Unavailable),
        (BrokenPipe, Category::Unavailable),
        (AlreadyExists, Category::AlreadyExists),
        (WouldBlock, Category::Unavailable),
        (InvalidInput, Category::InvalidArgument),
        (InvalidData, Category::InvalidArgument),
        (TimedOut, Category::DeadlineExceeded),
        (WriteZero, Category::Unavailable),
        (Interrupted, Category::Unavailable),
        (Unsupported, Category::Unimplemented),
        (UnexpectedEof, Category::OutOfRange),
        (OutOfMemory, Category::ResourceExhausted),
        (Other, Category::Unknown),
    ];
    for (kind, category) in mapped {
        assert_eq!(Category::from_io_kind(kind), category, "{kind:?}");
        let error = std::io::Error::from(kind);
        assert_eq!(
            define_err::ErrorInfo::category(&error),
            category,
            "{kind:?}"
        );
    }
}
