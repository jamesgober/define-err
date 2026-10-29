//! Generated code uses absolute paths only: it compiles in a module without
//! the prelude and next to user types named like the items it uses
//! (FR-27, E-5, E-6).

#![no_implicit_prelude]
#![deny(warnings, missing_docs, unreachable_pub)]

extern crate define_err;
extern crate std;

/// Shadows everything generated code could name by accident.
pub mod shadowed {
    #![allow(dead_code)]

    /// Not `core::error::Error`.
    pub struct Error;
    /// Not `core::result::Result`.
    pub struct Result;
    /// Not `core::option::Option`.
    pub struct Option;
    /// Not `alloc::boxed::Box`.
    pub struct Box;
    /// Not `core::fmt::Display`.
    pub struct Display;
    /// Not `define_err::Location`.
    pub struct Location;
    /// Not a code.
    pub struct Code;
    /// Not `Self`'s formatter.
    pub struct Formatter;
}

/// Errors defined where nothing is in scope.
pub mod errors {
    #[allow(unused_imports)]
    use super::shadowed::*;

    ::define_err::define_errors! {
        /// An enum named like the trait.
        pub enum Error {
            /// A braced variant.
            NotFound {
                /// The item.
                id: u64,
            } => "item {id} not found",
            /// A tuple variant with a source.
            Io(
                /// The cause.
                #[from] source: ::std::io::Error,
                /// Where the conversion happened.
                #[location] at: ::define_err::Location,
            ) => "i/o: {source}",
            /// A transparent wrapper.
            Fmt(
                /// The cause.
                #[from] ::std::fmt::Error,
            ) => transparent,
            /// A unit variant.
            Unit => "unit",
        }

        /// A struct named like the enum in core.
        pub struct Result {
            /// The code.
            pub code: u16,
        } => "code {code}";
    }
}

#[test]
fn test_generated_code_without_prelude() {
    use ::std::{assert, assert_eq, string::ToString};

    use errors::Error;

    assert_eq!(Error::NotFound { id: 4 }.to_string(), "item 4 not found");
    let err: Error = ::std::convert::From::from(::std::io::Error::other("x"));
    assert!(::std::error::Error::source(&err).is_some());
    assert!(::define_err::ErrorInfo::location(&err).is_some());
    assert_eq!(errors::Result { code: 7 }.to_string(), "code 7");
    assert_eq!(
        ::define_err::ErrorInfo::type_name(&Error::Unit),
        "Error::Unit"
    );
}
