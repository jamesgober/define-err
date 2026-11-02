//! The only unsafe code in the crate: a write-once slot, the `log` crate's
//! `set_logger` pattern.
//!
//! A thread writes the slot only after [`Control::claim`] made it the one
//! writer, and publishes with a `Release` store. Readers need a [`Word`] that
//! says ready, and only an `Acquire` load makes a `Word`, so a reader sees the
//! complete value or `None`, never a torn one. Once published, nothing writes
//! the slot again. `tests/loom.rs` models every interleaving; Miri runs the
//! test suite over it.

#![allow(unsafe_code)]

use super::{
    control::{Claim, Control, Word},
    sync::{maybe_const, Cell},
};

type Location = &'static core::panic::Location<'static>;

/// A reference set at most once, with where it was set.
pub(crate) struct WriteOnce<T: ?Sized + 'static> {
    value: Cell<Option<&'static T>>,
    at: Cell<Option<Location>>,
}

// SAFETY: `install` writes only as the single claimant, before the `Release`
// publish; `get` and `installed_at` read only with a `Word` that says ready,
// so the write happens before every read and no write follows. The stored
// `&'static T` crosses threads, which `T: Sync` allows.
unsafe impl<T: ?Sized + Sync> Sync for WriteOnce<T> {}

impl<T: ?Sized + 'static> WriteOnce<T> {
    maybe_const! {
        /// An empty slot.
        pub(crate) fn new() -> WriteOnce<T> {
            WriteOnce { value: Cell::new(None), at: Cell::new(None) }
        }
    }

    /// Installs the value `make` returns, which runs only if this call wins.
    /// On a loss, returns where the winner installed, or `None` while the
    /// winner has not published yet.
    pub(crate) fn install(
        &self,
        control: &Control,
        state: u32,
        make: impl FnOnce() -> &'static T,
        at: Location,
    ) -> Result<(), Option<Location>> {
        match control.claim(state) {
            Claim::Won => {}
            Claim::Busy => return Err(None),
            Claim::Taken => return Err(self.installed_at(control.load(), state)),
        }
        let value = make();
        // SAFETY: the claim makes this the only writer, and no reader looks
        // before the publish below.
        self.value.with_mut(|slot| unsafe { *slot = Some(value) });
        // SAFETY: as above.
        self.at.with_mut(|slot| unsafe { *slot = Some(at) });
        control.publish(state);
        Ok(())
    }

    /// The installed value, when `word` says the slot is ready.
    #[inline]
    pub(crate) fn get(&self, word: Word, state: u32) -> Option<&'static T> {
        if word.is_ready(state) {
            // SAFETY: a ready `Word` came from an `Acquire` load that saw the
            // publish, and nothing writes after it.
            self.value.with(|slot| unsafe { *slot })
        } else {
            None
        }
    }

    /// Where the value was installed, when `word` says the slot is ready.
    pub(crate) fn installed_at(&self, word: Word, state: u32) -> Option<Location> {
        if word.is_ready(state) {
            // SAFETY: as in `get`.
            self.at.with(|slot| unsafe { *slot })
        } else {
            None
        }
    }
}
