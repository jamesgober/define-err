//! The primitives `control.rs`, `slot.rs`, and `sites.rs` are built on.
//! `tests/loom.rs` compiles those three files against its own `sync` module
//! with this same interface, backed by loom.

#[cfg(not(target_has_atomic = "64"))]
pub(crate) use core::sync::atomic::AtomicU32 as AtomicKey;
#[cfg(target_has_atomic = "64")]
pub(crate) use core::sync::atomic::AtomicU64 as AtomicKey;
#[cfg(target_has_atomic = "64")]
pub(crate) use core::sync::atomic::AtomicU64;
pub(crate) use core::sync::atomic::{fence, AtomicU32, Ordering};

/// A pause in a retry loop; loom's models yield instead.
#[inline]
pub(crate) fn spin() {
    core::hint::spin_loop();
}

/// A key of the site table: 64 bits, or 32 on targets without 64-bit atomics.
#[cfg(target_has_atomic = "64")]
pub(crate) type Key = u64;
/// A key of the site table: 64 bits, or 32 on targets without 64-bit atomics.
#[cfg(not(target_has_atomic = "64"))]
pub(crate) type Key = u32;

/// An array of empty keys.
pub(crate) const fn empty_keys<const N: usize>() -> [AtomicKey; N] {
    #[allow(clippy::declare_interior_mutable_const)]
    const EMPTY: AtomicKey = AtomicKey::new(0);
    [EMPTY; N]
}

/// `loom::cell::UnsafeCell`'s interface over the `core` cell.
pub(crate) struct Cell<T>(core::cell::UnsafeCell<T>);

impl<T> Cell<T> {
    pub(crate) const fn new(value: T) -> Cell<T> {
        Cell(core::cell::UnsafeCell::new(value))
    }

    pub(crate) fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }

    pub(crate) fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}

/// A `const fn` here; loom's atomics cannot be built in a constant, so the
/// models' `sync` module makes it a plain `fn`.
macro_rules! maybe_const {
    ($(#[$attr:meta])* $vis:vis fn $($rest:tt)*) => {
        $(#[$attr])* $vis const fn $($rest)*
    };
}

pub(crate) use maybe_const;
