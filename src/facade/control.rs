//! The packed control word: one atomic read by every verb.
//!
//! ```text
//! bits 0-1    reporter slot state: 0 empty, 1 installing, 2 ready
//! bits 2-3    warn mode
//! bits 4-8    minimum severity, 0 to 24 (0 = everything)
//! bits 9-10   creation-hook slot state
//! bits 11-31  reserved
//! ```
//!
//! This file, `slot.rs`, and `sites.rs` are also compiled by `tests/loom.rs`.

use super::sync::{maybe_const, AtomicU32, Ordering};

/// The shift of the reporter slot's state bits.
pub(crate) const REPORTER_STATE: u32 = 0;
/// The shift of the creation-hook slot's state bits.
pub(crate) const HOOK_STATE: u32 = 9;

const INSTALLING: u32 = 1;
const READY: u32 = 2;
const MODE_SHIFT: u32 = 2;
const MODE_MASK: u32 = 0b11 << MODE_SHIFT;
const MIN_SHIFT: u32 = 4;
const MIN_MASK: u32 = 0b1_1111 << MIN_SHIFT;

/// What a thread found when it tried to claim a slot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Claim {
    /// This thread is now the only writer of the slot.
    Won,
    /// Another thread has claimed the slot and not published yet.
    Busy,
    /// The slot is installed.
    Taken,
}

/// One value of the control word, read with `Acquire`. Only [`Control::load`]
/// makes one, so a word that says "ready" proves the slot was published.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Word(u32);

impl Word {
    /// Whether the slot with these state bits is published.
    #[inline]
    pub(crate) const fn is_ready(self, state: u32) -> bool {
        (self.0 >> state) & 0b11 == READY
    }

    /// The warn mode bits.
    #[inline]
    pub(crate) const fn mode(self) -> u32 {
        (self.0 & MODE_MASK) >> MODE_SHIFT
    }

    /// The minimum severity, 0 for none.
    #[inline]
    pub(crate) const fn min_severity(self) -> u32 {
        (self.0 & MIN_MASK) >> MIN_SHIFT
    }
}

/// The control word.
#[derive(Debug)]
pub(crate) struct Control(AtomicU32);

impl Control {
    maybe_const! {
        /// A word with both slots empty and every setting at its default.
        pub(crate) fn new() -> Control {
            Control(AtomicU32::new(0))
        }
    }

    /// The word every verb starts from.
    #[inline]
    pub(crate) fn load(&self) -> Word {
        Word(self.0.load(Ordering::Acquire))
    }

    /// Tries to become the slot's only writer.
    pub(crate) fn claim(&self, state: u32) -> Claim {
        let mut current = self.0.load(Ordering::Relaxed);
        loop {
            match (current >> state) & 0b11 {
                0 => {
                    let claimed = current | (INSTALLING << state);
                    match self.0.compare_exchange_weak(
                        current,
                        claimed,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => return Claim::Won,
                        Err(now) => current = now,
                    }
                }
                INSTALLING => return Claim::Busy,
                _ => return Claim::Taken,
            }
        }
    }

    /// Publishes a claimed slot: installing (01) becomes ready (10). The
    /// setters below are read-modify-writes, so they stay in this store's
    /// release sequence and a word they write still proves the publish.
    pub(crate) fn publish(&self, state: u32) {
        let _ = self.0.fetch_xor(0b11 << state, Ordering::Release);
    }

    /// Sets the warn mode bits; never touches the state bits.
    pub(crate) fn set_mode(&self, mode: u32) {
        self.update(MODE_MASK, mode << MODE_SHIFT);
    }

    /// Sets the minimum severity; never touches the state bits.
    pub(crate) fn set_min_severity(&self, min: u32) {
        self.update(MIN_MASK, min << MIN_SHIFT);
    }

    fn update(&self, mask: u32, bits: u32) {
        let mut current = self.0.load(Ordering::Relaxed);
        loop {
            let next = (current & !mask) | bits;
            match self
                .0
                .compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return,
                Err(now) => current = now,
            }
        }
    }
}
