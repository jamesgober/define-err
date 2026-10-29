//! Boxed `dyn Error` sources and transparent wrappers over boxes. They need
//! this crate's `alloc` feature (on with the default `std`).

#![cfg(feature = "alloc")]

use std::error::Error as _;

use define_err::{define_errors, prelude::*};

define_errors! {
    pub enum Boxed {
        Plain(#[source] Box<dyn std::error::Error>) => "plain box",
        Send { #[source] inner: Box<dyn std::error::Error + Send> } => "send box",
        SendSync(#[source] inner: Box<dyn std::error::Error + Send + Sync>) => "send sync box: {inner}",
        Static { source: &'static (dyn std::error::Error + Send + Sync) } => "static",
        Forward(#[from] Box<dyn std::error::Error + Send + Sync>) => transparent,
    }
}

static FMT: std::fmt::Error = std::fmt::Error;

#[test]
fn test_boxed_sources_return_the_inner_error() {
    let err = Boxed::Plain(Box::new(std::fmt::Error));
    assert!(err.source().unwrap().is::<std::fmt::Error>());
    let err = Boxed::Send {
        inner: Box::new(std::io::Error::other("s")),
    };
    assert_eq!(err.source().unwrap().to_string(), "s");
    let err = Boxed::SendSync(Box::new(std::io::Error::other("ss")));
    assert_eq!(err.to_string(), "send sync box: ss");
    assert_eq!(err.source().unwrap().to_string(), "ss");
    let err = Boxed::Static { source: &FMT };
    assert!(err.source().unwrap().is::<std::fmt::Error>());
}

define_errors! {
    pub enum Inner {
        Gone(#[source] source: std::io::Error) => "the store is gone",
    }
}

#[test]
fn test_transparent_over_box_forwards_display_source_and_info() {
    let boxed: Box<dyn std::error::Error + Send + Sync> =
        Box::new(Inner::Gone(std::io::Error::other("x")));
    let err = Boxed::from(boxed);
    assert_eq!(err.to_string(), "the store is gone");
    assert_eq!(err.source().unwrap().to_string(), "x");
    // A box is not `ErrorInfo`, so the wrapper's own name is reported.
    assert_eq!(err.type_name(), "Boxed::Forward");
}
