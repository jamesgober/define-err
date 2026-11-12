//! The kinds mask, read whole by every verb. One 64-bit atomic where the
//! target has one; elsewhere two 32-bit halves under a sequence count, so a
//! reader never combines one write's low half with another's high half.
//! `tests/loom.rs` compiles this file against its own `sync` module and
//! models both forms.

#[cfg(target_has_atomic = "64")]
use super::sync::AtomicU64;
use super::sync::{
    fence, maybe_const, spin, AtomicU32,
    Ordering::{Acquire, Relaxed, Release},
};

/// The mask on targets with 64-bit atomics: one relaxed load or store.
#[cfg(target_has_atomic = "64")]
pub(crate) struct Whole(AtomicU64);

#[cfg(target_has_atomic = "64")]
impl Whole {
    maybe_const! {
        pub(crate) fn new(bits: u64) -> Whole {
            Whole(AtomicU64::new(bits))
        }
    }

    #[inline]
    pub(crate) fn get(&self) -> u64 {
        self.0.load(Relaxed)
    }

    pub(crate) fn set(&self, bits: u64) {
        self.0.store(bits, Relaxed);
    }
}

/// How many times a reader tries before it gives up on a write in progress.
const TRIES: usize = 64;

/// The mask on targets with only 32-bit atomics: a sequence lock. The count
/// is odd while a write is in progress. Writers take turns through the
/// count; readers never wait on a writer for long: after `TRIES` attempts
/// that each overlapped a write, a reader returns every bit set, which
/// delivers an event the new mask might exclude and never drops one.
#[cfg_attr(target_has_atomic = "64", allow(dead_code))]
pub(crate) struct Split {
    count: AtomicU32,
    low: AtomicU32,
    high: AtomicU32,
}

#[cfg_attr(target_has_atomic = "64", allow(dead_code))]
impl Split {
    maybe_const! {
        pub(crate) fn new(bits: u64) -> Split {
            Split {
                count: AtomicU32::new(0),
                low: AtomicU32::new(bits as u32),
                high: AtomicU32::new((bits >> 32) as u32),
            }
        }
    }

    #[inline]
    pub(crate) fn get(&self) -> u64 {
        for _ in 0..TRIES {
            let before = self.count.load(Acquire);
            let low = self.low.load(Relaxed);
            let high = self.high.load(Relaxed);
            fence(Acquire);
            if before & 1 == 0 && self.count.load(Relaxed) == before {
                return u64::from(low) | u64::from(high) << 32;
            }
            spin();
        }
        u64::MAX
    }

    pub(crate) fn set(&self, bits: u64) {
        let mut count = self.count.load(Relaxed);
        loop {
            if count & 1 == 0 {
                match self.count.compare_exchange_weak(
                    count,
                    count.wrapping_add(1),
                    Acquire,
                    Relaxed,
                ) {
                    Ok(_) => break,
                    Err(now) => count = now,
                }
            } else {
                spin();
                count = self.count.load(Relaxed);
            }
        }
        fence(Release);
        self.low.store(bits as u32, Relaxed);
        self.high.store((bits >> 32) as u32, Relaxed);
        self.count.store(count.wrapping_add(2), Release);
    }
}

/// The form this target uses.
#[cfg(target_has_atomic = "64")]
pub(crate) type Mask = Whole;
/// The form this target uses.
#[cfg(not(target_has_atomic = "64"))]
pub(crate) type Mask = Split;
