//! `state.rs` for targets without compare-and-swap (`thumbv6m-none-eabi`): no
//! slot can be claimed safely, so nothing is ever installed, the settings
//! stay at their defaults, and every verb returns at once.

use super::{CreationHook, FacadeStats, Reporter, WarnMode};
use crate::{Kinds, Location, Severity};

/// The control word that is never ready.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Word;

/// One call site's flag, unused here.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct Site;

impl Site {
    /// A site.
    pub const fn new() -> Site {
        Site
    }
}

#[inline]
pub(crate) fn load() -> Word {
    Word
}

#[inline]
pub(crate) fn is_ready(_word: Word) -> bool {
    false
}

#[inline]
pub(crate) fn mode(_word: Word) -> WarnMode {
    WarnMode::Once
}

#[inline]
pub(crate) fn passes(_word: Word, _severity: Severity, _kinds: Kinds) -> bool {
    false
}

#[inline]
pub(crate) fn has_minimum(_word: Word) -> bool {
    false
}

#[inline]
pub(crate) fn below_minimum(_word: Word, _severity: Severity) -> bool {
    false
}

#[inline]
pub(crate) fn reporter(_word: Word) -> Option<&'static dyn Reporter> {
    None
}

pub(crate) fn reporter_installed_at() -> Option<Location> {
    None
}

#[inline]
pub(crate) fn creation_hook() -> Option<&'static dyn CreationHook> {
    None
}

pub(crate) fn set_mode(_mode: WarnMode) {}

pub(crate) fn set_min_severity(_min: Option<Severity>) {}

pub(crate) fn min_severity() -> Option<Severity> {
    None
}

pub(crate) fn set_kinds_mask(_mask: Kinds) {}

pub(crate) fn kinds_mask() -> Kinds {
    Kinds::ALL
}

pub(crate) fn table_fires(_location: Location) -> bool {
    false
}

pub(crate) fn item_fires(_location: Location, _item: &str) -> bool {
    false
}

#[inline]
pub(crate) fn site_has_fired(_site: &Site) -> bool {
    true
}

#[inline]
pub(crate) fn site_fires(_site: &Site) -> bool {
    false
}

#[cfg(feature = "std")]
pub(crate) fn count_reentrant() {}

#[cfg(feature = "std")]
pub(crate) fn count_panic() {}

pub(crate) fn stats() -> FacadeStats {
    FacadeStats::default()
}
