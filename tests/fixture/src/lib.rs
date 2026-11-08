//! Types defined with `define_errors!` in a crate of their own: `#[non_exhaustive]`
//! only binds outside the defining crate, so FR-6 needs a second crate to test.

#![no_std]

use define_err::define_errors;

define_errors! {
    /// An enum with the default: `#[non_exhaustive]` is added.
    pub enum Open {
        A => "a",
        B => "b",
    }

    /// An enum that opts out with `#[exhaustive]`.
    #[exhaustive]
    pub enum Closed {
        A => "a",
        B => "b",
    }
}
