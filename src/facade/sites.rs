//! Once per call site: a flag for the macros, a fixed table for the functions.
//!
//! This file is also compiled by `tests/loom.rs`.

use super::sync::{empty_keys, maybe_const, AtomicKey, AtomicU32, Key, Ordering};

/// One call site's flag. `warn!` and `deprecated!` keep a `static` of this
/// type, so the macros are exact for any number of sites.
#[doc(hidden)]
#[derive(Debug)]
pub struct Site(AtomicU32);

impl Site {
    maybe_const! {
        /// A site that has not delivered.
        pub fn new() -> Site {
            Site(AtomicU32::new(0))
        }
    }

    /// Whether this site has delivered.
    #[inline]
    pub(crate) fn has_fired(&self) -> bool {
        self.0.load(Ordering::Relaxed) != 0
    }

    /// Marks the site; `true` for the one caller that gets to deliver.
    #[inline]
    pub(crate) fn fire(&self) -> bool {
        self.0.swap(1, Ordering::Relaxed) == 0
    }
}

impl Default for Site {
    fn default() -> Site {
        Site::new()
    }
}

/// What the table says about a key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Seen {
    /// The first time: deliver.
    First,
    /// Already recorded: drop.
    Again,
    /// The table is full and the key is not in it: deliver, and count it.
    Full,
}

/// A fixed, lock-free, insert-only table of keys, by linear probing. 0 means
/// an empty entry; keys are never 0.
#[derive(Debug)]
pub(crate) struct SiteTable<const N: usize> {
    keys: [AtomicKey; N],
}

impl<const N: usize> SiteTable<N> {
    maybe_const! {
        /// An empty table.
        pub(crate) fn new() -> SiteTable<N> {
            SiteTable { keys: empty_keys() }
        }
    }

    /// Records `key`. Entries are never removed, so a key is found on the
    /// probe path of any thread that recorded it.
    pub(crate) fn check(&self, key: Key) -> Seen {
        let start = key as usize % N;
        for offset in 0..N {
            let entry = &self.keys[(start + offset) % N];
            let found = entry.load(Ordering::Relaxed);
            if found == key {
                return Seen::Again;
            }
            if found == 0 {
                match entry.compare_exchange(0, key, Ordering::Relaxed, Ordering::Relaxed) {
                    Ok(_) => return Seen::First,
                    Err(now) if now == key => return Seen::Again,
                    Err(_) => {}
                }
            }
        }
        Seen::Full
    }
}

/// The key of a source location: FNV-1a over the file name, line, and column.
/// By content, because the compiler may emit one location twice across
/// codegen units. Never 0.
pub(crate) fn key(file: &str, line: u32, column: u32) -> Key {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let tail = line.to_le_bytes().into_iter().chain(column.to_le_bytes());
    for byte in file.bytes().chain(tail) {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    // Folds to 32 bits where `Key` is 32 bits; the high half mixes in.
    let folded = (hash ^ (hash >> 32)) as Key;
    if folded == 0 {
        1
    } else {
        folded
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::{key, Key, Seen, Site, SiteTable};

    #[test]
    fn test_site_fires_once() {
        let site = Site::new();
        assert!(!site.has_fired());
        assert!(site.fire());
        assert!(site.has_fired());
        assert!(!site.fire());
    }

    #[test]
    fn test_table_records_each_key_once() {
        let table = SiteTable::<8>::new();
        assert_eq!(table.check(5), Seen::First);
        assert_eq!(table.check(5), Seen::Again);
        // Same start, different key: the probe moves on.
        assert_eq!(table.check(13), Seen::First);
        assert_eq!(table.check(13), Seen::Again);
        assert_eq!(table.check(5), Seen::Again);
    }

    /// E-72: 600 sites against 512 entries.
    #[test]
    fn test_table_full_never_loses_a_site() {
        let table = SiteTable::<512>::new();
        let keys: [Key; 600] = core::array::from_fn(|i| i as Key + 1);
        let first = keys
            .iter()
            .filter(|&&k| table.check(k) == Seen::First)
            .count();
        assert_eq!(first, 512);
        let again = keys
            .iter()
            .filter(|&&k| table.check(k) == Seen::Again)
            .count();
        let full = keys
            .iter()
            .filter(|&&k| table.check(k) == Seen::Full)
            .count();
        assert_eq!(again, 512);
        assert_eq!(full, 88);
    }

    #[test]
    fn test_key_depends_on_every_part() {
        let base = key("src/a.rs", 1, 2);
        assert_ne!(base, 0);
        assert_ne!(base, key("src/b.rs", 1, 2));
        assert_ne!(base, key("src/a.rs", 2, 2));
        assert_ne!(base, key("src/a.rs", 1, 3));
    }
}
