//! The facade's globals: the control word with its two slots, the kind mask,
//! the once-per-site table, and the counters. `inert.rs` has the same items
//! for targets without compare-and-swap.

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

pub(crate) use super::control::Word;
pub use super::sites::Site;
use super::{
    control::{Control, HOOK_STATE, REPORTER_STATE},
    mask::Mask,
    sites::{item_key, key, Seen, SiteTable},
    slot::WriteOnce,
    sync::Key,
    CreationHook, FacadeStats, Reporter, WarnMode,
};
use crate::{Kinds, Location, Severity};

/// How many sites the function forms can remember.
const SITES: usize = 512;

static CONTROL: Control = Control::new();
static REPORTER: WriteOnce<dyn Reporter> = WriteOnce::new();
static HOOK: WriteOnce<dyn CreationHook> = WriteOnce::new();
static MASK: Mask = Mask::new(u64::MAX);
static TABLE: SiteTable<SITES> = SiteTable::new();
static OVERFLOW: AtomicUsize = AtomicUsize::new(0);
static REENTRANT: AtomicUsize = AtomicUsize::new(0);
static PANICS: AtomicUsize = AtomicUsize::new(0);

#[inline]
pub(crate) fn load() -> Word {
    CONTROL.load()
}

#[inline]
pub(crate) fn is_ready(word: Word) -> bool {
    word.is_ready(REPORTER_STATE)
}

#[inline]
pub(crate) fn mode(word: Word) -> WarnMode {
    match word.mode() {
        0 => WarnMode::Once,
        1 => WarnMode::Always,
        2 => WarnMode::Ignore,
        _ => WarnMode::Error,
    }
}

/// Whether the filters let an event through. The default mask passes
/// everything, an event with no kinds included.
#[inline]
pub(crate) fn passes(word: Word, severity: Severity, kinds: Kinds) -> bool {
    !below_minimum(word, severity) && {
        let mask = kinds_mask();
        mask == Kinds::ALL || kinds.intersects(mask)
    }
}

/// Whether a minimum severity is set.
#[inline]
pub(crate) fn has_minimum(word: Word) -> bool {
    word.min_severity() != 0
}

/// Whether the minimum drops this severity, whatever its kinds.
#[inline]
pub(crate) fn below_minimum(word: Word, severity: Severity) -> bool {
    u32::from(severity.get()) < word.min_severity()
}

#[inline]
pub(crate) fn reporter(word: Word) -> Option<&'static dyn Reporter> {
    REPORTER.get(word, REPORTER_STATE)
}

pub(crate) fn reporter_installed_at() -> Option<Location> {
    REPORTER.installed_at(load(), REPORTER_STATE)
}

pub(crate) fn install_reporter(
    make: impl FnOnce() -> &'static dyn Reporter,
    at: Location,
) -> Result<(), Option<Location>> {
    REPORTER.install(&CONTROL, REPORTER_STATE, make, at)
}

#[inline]
pub(crate) fn creation_hook() -> Option<&'static dyn CreationHook> {
    HOOK.get(load(), HOOK_STATE)
}

pub(crate) fn install_hook(
    hook: &'static dyn CreationHook,
    at: Location,
) -> Result<(), Option<Location>> {
    HOOK.install(&CONTROL, HOOK_STATE, || hook, at)
}

pub(crate) fn set_mode(mode: WarnMode) {
    CONTROL.set_mode(match mode {
        WarnMode::Once => 0,
        WarnMode::Always => 1,
        WarnMode::Ignore => 2,
        WarnMode::Error => 3,
    });
}

pub(crate) fn set_min_severity(min: Option<Severity>) {
    CONTROL.set_min_severity(min.map_or(0, |severity| u32::from(severity.get())));
}

pub(crate) fn min_severity() -> Option<Severity> {
    Severity::new(load().min_severity() as u8)
}

pub(crate) fn set_kinds_mask(mask: Kinds) {
    MASK.set(mask.bits());
}

/// The mask, read whole: never one write's low half with another's high.
#[inline]
pub(crate) fn kinds_mask() -> Kinds {
    Kinds::from_bits(MASK.get())
}

/// Whether a function-form site gets to deliver. A full table delivers every
/// time and counts it, so a warning is never lost to the table.
pub(crate) fn table_fires(location: Location) -> bool {
    table_fires_key(key(location.file(), location.line(), location.column()))
}

/// `table_fires` for one item at a site (`deprecated(item)`).
pub(crate) fn item_fires(location: Location, item: &str) -> bool {
    table_fires_key(item_key(
        location.file(),
        location.line(),
        location.column(),
        item,
    ))
}

fn table_fires_key(key: Key) -> bool {
    match TABLE.check(key) {
        Seen::First => true,
        Seen::Again => false,
        Seen::Full => {
            let _ = OVERFLOW.fetch_add(1, Relaxed);
            true
        }
    }
}

/// Whether a macro-form site has delivered.
#[inline]
pub(crate) fn site_has_fired(site: &Site) -> bool {
    site.has_fired()
}

/// Marks a macro-form site; `true` for the one caller that delivers.
#[inline]
pub(crate) fn site_fires(site: &Site) -> bool {
    site.fire()
}

#[cfg(feature = "std")]
pub(crate) fn count_reentrant() {
    let _ = REENTRANT.fetch_add(1, Relaxed);
}

#[cfg(feature = "std")]
pub(crate) fn count_panic() {
    let _ = PANICS.fetch_add(1, Relaxed);
}

pub(crate) fn stats() -> FacadeStats {
    FacadeStats {
        site_table_overflow: OVERFLOW.load(Relaxed) as u64,
        reentrant_dropped: REENTRANT.load(Relaxed) as u64,
        reporter_panics: PANICS.load(Relaxed) as u64,
    }
}
