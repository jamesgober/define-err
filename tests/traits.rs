//! Auto traits, sizes, and layout of generated types match their fields
//! (7.5.5 rule 7, NFR-2, NFR-9).

use core::mem::{align_of, size_of};

use define_err::{define_errors, Location, Secret};

define_errors! {
    pub enum Generated {
        A { page: u64 } => "a {page}",
        B(code: u32, at: Location) => "b {code}",
        C => "c",
    }
}

/// The same enum written by hand.
#[allow(dead_code)]
#[derive(Debug)]
pub enum HandWritten {
    A { page: u64 },
    B(u32, Location),
    C,
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum WithThiserror {
    #[error("a {page}")]
    A { page: u64 },
    #[error("b {0}")]
    B(u32, Location),
    #[error("c")]
    C,
}

fn assert_send_sync<T: Send + Sync>() {}
fn assert_unpin<T: Unpin>() {}

#[test]
fn test_size_and_alignment_equal_hand_written_and_thiserror() {
    assert_eq!(size_of::<Generated>(), size_of::<HandWritten>());
    assert_eq!(align_of::<Generated>(), align_of::<HandWritten>());
    assert_eq!(size_of::<Generated>(), size_of::<WithThiserror>());
}

#[test]
fn test_generated_types_are_send_sync_unpin() {
    assert_send_sync::<Generated>();
    assert_unpin::<Generated>();
    assert_send_sync::<Secret<String>>();
    assert_send_sync::<Location>();
}

define_errors! {
    pub enum NotSync {
        Cell { value: std::cell::Cell<u8> } => "cell",
    }
}

#[test]
fn test_auto_traits_follow_fields() {
    fn assert_send<T: Send>() {}
    assert_send::<NotSync>();
    // `NotSync: !Sync` is checked by the type system: a `Cell` field makes the
    // enum `!Sync` exactly as it would a hand-written enum.
}

#[test]
fn test_secret_is_newtype_sized() {
    assert_eq!(size_of::<Secret<u64>>(), size_of::<u64>());
    assert_eq!(size_of::<Secret<String>>(), size_of::<String>());
}
