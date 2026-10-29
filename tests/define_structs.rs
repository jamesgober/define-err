//! Structs: braced, tuple, unit; field visibility; sources and `From`.

use std::error::Error as _;

use define_err::{define_errors, here, prelude::*};

define_errors! {
    /// A syntax error in a NOML file.
    #[derive(Clone, PartialEq)]
    pub struct SyntaxError {
        /// The line, from 1.
        pub line: u32,
        pub column: u32,
    } => "syntax error at {line}:{column}";

    pub struct Timeout(pub u64) => "operation timed out";

    pub struct Elapsed(pub secs: u64) => "timed out after {secs} s";

    pub struct Cancelled => "operation cancelled";

    pub struct Wrapped {
        #[from]
        source: std::io::Error,
        #[location]
        at: Location,
    } => "wrapped i/o error";

    pub struct Forward(#[from] inner: std::fmt::Error) => transparent;

    pub struct Many(pub u8, pub u16, pub(crate) u32, String) => "many";
}

#[test]
fn test_braced_struct_display_and_public_fields() {
    let err = SyntaxError { line: 3, column: 9 };
    assert_eq!(err.to_string(), "syntax error at 3:9");
    assert_eq!(err.clone(), err);
    assert_eq!(err.type_name(), "SyntaxError");
}

#[test]
fn test_tuple_structs_named_and_unnamed() {
    assert_eq!(Timeout(5).to_string(), "operation timed out");
    assert_eq!(Timeout(5).0, 5);
    assert_eq!(Elapsed(7).to_string(), "timed out after 7 s");
    assert_eq!(Many(1, 2, 3, String::new()).to_string(), "many");
}

#[test]
fn test_unit_struct() {
    assert_eq!(Cancelled.to_string(), "operation cancelled");
    assert!(Cancelled.source().is_none());
    assert_eq!(format!("{Cancelled:?}"), "Cancelled");
}

#[test]
fn test_struct_from_fills_location() {
    #[rustfmt::skip]
    let (err, line): (Wrapped, u32) = (std::io::Error::other("disk").into(), line!());
    assert_eq!(err.location().unwrap().line(), line);
    assert_eq!(err.source().unwrap().to_string(), "disk");
}

#[test]
fn test_transparent_struct() {
    let err = Forward::from(std::fmt::Error);
    assert_eq!(err.to_string(), std::fmt::Error.to_string());
    assert!(err.source().is_none());
}

#[test]
fn test_struct_built_by_hand_with_here() {
    #[rustfmt::skip]
    let (err, line) = (Wrapped { source: std::io::Error::other("x"), at: here() }, line!());
    assert_eq!(err.location().unwrap().line(), line);
}

#[test]
fn test_structs_are_not_non_exhaustive() {
    // Built with a struct literal: this only compiles because no
    // `#[non_exhaustive]` was added.
    let err = SyntaxError { line: 1, column: 1 };
    let SyntaxError { line, column } = err;
    assert_eq!((line, column), (1, 1));
}
