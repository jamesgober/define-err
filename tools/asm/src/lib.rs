//! One exported function per path. Read the listing with:
//!
//! ```text
//! cargo rustc --release -- --emit asm -C debuginfo=0
//! ```

use std::{error::Error, fmt};

use define_err::define_errors;

define_errors! {
    pub enum Ours {
        NotFound { page: u64 } => "page {page} not found",
        Short(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Theirs {
    #[error("page {page} not found")]
    NotFound { page: u64 },
    #[error("short read on page {0}: got {1} bytes")]
    Short(u64, usize),
}

#[derive(Debug)]
pub enum Hand {
    NotFound { page: u64 },
    Short(u64, usize),
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hand::NotFound { page } => write!(f, "page {page} not found"),
            Hand::Short(page, got) => write!(f, "short read on page {page}: got {got} bytes"),
        }
    }
}

impl Error for Hand {}

#[no_mangle]
pub fn asm_report(error: &(dyn Error + 'static)) {
    define_err::report(error);
}

#[no_mangle]
pub fn asm_notice() {
    define_err::notice("text");
}

#[no_mangle]
pub fn asm_notice_macro(value: u32) {
    define_err::notice!("text {value}");
}

#[no_mangle]
pub fn asm_warn_macro(value: u32) {
    define_err::warn!("text {value}");
}

#[no_mangle]
pub fn asm_is_installed() -> bool {
    define_err::is_installed()
}

#[no_mangle]
pub fn asm_make_ours(page: u64, short: bool) -> Result<u64, Ours> {
    if short {
        Err(Ours::Short(page, 7))
    } else {
        Err(Ours::NotFound { page })
    }
}

#[no_mangle]
pub fn asm_make_theirs(page: u64, short: bool) -> Result<u64, Theirs> {
    if short {
        Err(Theirs::Short(page, 7))
    } else {
        Err(Theirs::NotFound { page })
    }
}

#[no_mangle]
pub fn asm_make_hand(page: u64, short: bool) -> Result<u64, Hand> {
    if short {
        Err(Hand::Short(page, 7))
    } else {
        Err(Hand::NotFound { page })
    }
}
