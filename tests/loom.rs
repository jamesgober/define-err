//! Exhaustive models of the facade's atomics (dev/PLAN.md 12.5). Runs only
//! under `RUSTFLAGS="--cfg loom"`:
//!
//! ```text
//! RUSTFLAGS="--cfg loom" cargo test --test loom --release
//! ```
//!
//! The modules are the crate's own files, compiled here against this file's
//! `sync` module, which has the interface of `src/facade/sync.rs` over loom's
//! atomics and cell.

#![cfg(loom)]
#![allow(dead_code, unsafe_code)]

mod sync {
    pub use loom::{
        cell::UnsafeCell as Cell,
        sync::atomic::{AtomicU32, AtomicU64 as AtomicKey, Ordering},
    };

    pub type Key = u64;

    pub fn empty_keys<const N: usize>() -> [AtomicKey; N] {
        core::array::from_fn(|_| AtomicKey::new(0))
    }

    macro_rules! maybe_const {
        ($(#[$attr:meta])* $vis:vis fn $($rest:tt)*) => {
            $(#[$attr])* $vis fn $($rest)*
        };
    }

    pub(crate) use maybe_const;
}

#[path = "../src/facade/control.rs"]
mod control;
#[path = "../src/facade/sites.rs"]
mod sites;
#[path = "../src/facade/slot.rs"]
mod slot;

use control::{Control, REPORTER_STATE};
use loom::{sync::Arc, thread};
use sites::{Seen, Site, SiteTable};
use slot::WriteOnce;

const STATE: u32 = REPORTER_STATE;
static FIRST: u32 = 1;
static SECOND: u32 = 2;

struct Shared {
    control: Control,
    slot: WriteOnce<u32>,
}

impl Shared {
    fn new() -> Arc<Shared> {
        Arc::new(Shared {
            control: Control::new(),
            slot: WriteOnce::new(),
        })
    }
}

#[track_caller]
fn install(
    shared: &Shared,
    value: &'static u32,
) -> Result<(), Option<&'static core::panic::Location<'static>>> {
    shared.slot.install(
        &shared.control,
        STATE,
        || value,
        core::panic::Location::caller(),
    )
}

/// Two installers: exactly one wins, and the value read afterwards is its.
#[test]
fn loom_install_race() {
    loom::model(|| {
        let shared = Shared::new();
        let other = shared.clone();
        let racer = thread::spawn(move || install(&other, &SECOND).is_ok());
        let won = install(&shared, &FIRST).is_ok();
        let other_won = racer.join().unwrap();
        assert!(won != other_won, "exactly one installer wins");
        let value = shared.slot.get(shared.control.load(), STATE).unwrap();
        assert_eq!(*value, if won { 1 } else { 2 });
    });
}

/// A loser that saw the slot taken names the winner's location; one that saw
/// it mid-install gets `None`.
#[test]
fn loom_loser_location() {
    loom::model(|| {
        let shared = Shared::new();
        let other = shared.clone();
        let racer = thread::spawn(move || install(&other, &SECOND));
        let result = install(&shared, &FIRST);
        let other_result = racer.join().unwrap();
        for outcome in [result, other_result] {
            if let Err(Some(at)) = outcome {
                assert_eq!(at.file(), file!());
            }
        }
    });
}

/// A reader racing the installer sees nothing or the complete value.
#[test]
fn loom_read_during_install() {
    loom::model(|| {
        let shared = Shared::new();
        let other = shared.clone();
        let reader = thread::spawn(move || other.slot.get(other.control.load(), STATE).copied());
        assert!(install(&shared, &FIRST).is_ok());
        let seen = reader.join().unwrap();
        assert!(seen.is_none() || seen == Some(1));
    });
}

/// Settings written while an install runs lose neither side's bits.
#[test]
fn loom_settings_during_install() {
    loom::model(|| {
        let shared = Shared::new();
        let other = shared.clone();
        let setter = thread::spawn(move || {
            other.control.set_mode(3);
            other.control.set_min_severity(17);
        });
        assert!(install(&shared, &FIRST).is_ok());
        setter.join().unwrap();
        let word = shared.control.load();
        assert!(word.is_ready(STATE));
        assert_eq!(word.mode(), 3);
        assert_eq!(word.min_severity(), 17);
        assert_eq!(shared.slot.get(word, STATE).copied(), Some(1));
    });
}

/// Two threads on one site flag: exactly one delivers.
#[test]
fn loom_site_flag() {
    loom::model(|| {
        let site = Arc::new(Site::new());
        let other = site.clone();
        let racer = thread::spawn(move || other.fire());
        let fired = site.fire();
        assert!(fired != racer.join().unwrap());
    });
}

/// Two threads on one key: exactly one is first. Two keys with the same
/// start: both are recorded.
#[test]
fn loom_site_table() {
    loom::model(|| {
        let table = Arc::new(SiteTable::<2>::new());
        let other = table.clone();
        let racer = thread::spawn(move || other.check(5));
        let seen = table.check(5);
        let other_seen = racer.join().unwrap();
        assert!((seen == Seen::First) != (other_seen == Seen::First));
    });
    loom::model(|| {
        let table = Arc::new(SiteTable::<2>::new());
        let other = table.clone();
        let racer = thread::spawn(move || other.check(2));
        assert_eq!(table.check(4), Seen::First);
        assert_eq!(racer.join().unwrap(), Seen::First);
        assert_eq!(table.check(4), Seen::Again);
        assert_eq!(table.check(2), Seen::Again);
    });
    // A full table: the second key is refused, and the first stays.
    loom::model(|| {
        let table = Arc::new(SiteTable::<1>::new());
        let other = table.clone();
        let racer = thread::spawn(move || other.check(2));
        let seen = table.check(4);
        let other_seen = racer.join().unwrap();
        assert!((seen == Seen::First) != (other_seen == Seen::First));
        assert!(seen == Seen::Full || other_seen == Seen::Full);
    });
}
