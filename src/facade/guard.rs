//! Reentrancy guard and panic isolation around a reporter call.
//!
//! With `std`, a verb called from inside `receive` on the same thread is
//! dropped and counted, and a panic inside `receive` is caught and counted.
//! Without `std` neither guard exists; `docs/FACADE.md` states the contract.

#[cfg(feature = "std")]
mod imp {
    use core::{any::Any, cell::Cell};
    use std::{
        boxed::Box,
        panic::{catch_unwind, AssertUnwindSafe},
    };

    use crate::facade::state;

    std::thread_local! {
        static INSIDE: Cell<bool> = const { Cell::new(false) };
    }

    /// Counts and refuses a verb called inside a reporter call.
    #[inline]
    pub(crate) fn refuses() -> bool {
        let inside = INSIDE.try_with(Cell::get).unwrap_or(false);
        if inside {
            state::count_reentrant();
        }
        inside
    }

    /// Runs `call` as a reporter call.
    pub(crate) fn run(call: impl FnOnce()) {
        if refuses() {
            return;
        }
        let _ = INSIDE.try_with(|flag| flag.set(true));
        let outcome = catch_unwind(AssertUnwindSafe(call));
        let _ = INSIDE.try_with(|flag| flag.set(false));
        if let Err(payload) = outcome {
            state::count_panic();
            drop_payload(payload);
        }
    }

    /// Drops a panic payload. Dropping one can itself panic, with a payload
    /// of its own. Those are dropped too, up to a limit, so a payload that
    /// keeps panicking leaks instead of looping forever.
    fn drop_payload(mut payload: Box<dyn Any + Send>) {
        for _ in 0..4 {
            match catch_unwind(AssertUnwindSafe(move || drop(payload))) {
                Ok(()) => return,
                Err(next) => {
                    state::count_panic();
                    payload = next;
                }
            }
        }
        core::mem::forget(payload);
    }
}

#[cfg(not(feature = "std"))]
mod imp {
    /// Refuses nothing: there is no guard without `std`.
    #[inline]
    pub(crate) fn refuses() -> bool {
        false
    }

    /// Runs `call` as a reporter call.
    #[inline]
    pub(crate) fn run(call: impl FnOnce()) {
        call();
    }
}

pub(crate) use imp::{refuses, run};
