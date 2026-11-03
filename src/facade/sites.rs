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

/// One round of the key hash: a 64 by 64 multiply folded back to 64 bits.
#[inline]
fn mix(hash: u64, word: u64) -> u64 {
    let wide = u128::from(hash ^ word) * 0x9e37_79b9_7f4a_7c15_u128;
    wide as u64 ^ (wide >> 64) as u64
}

/// The key of a source location: the file name, line, and column, hashed
/// eight bytes at a time so a long path costs a few rounds, not one per byte.
/// By content, because the compiler may emit one location twice across
/// codegen units. Never 0.
pub(crate) fn key(file: &str, line: u32, column: u32) -> Key {
    let mut hash = 0xcbf2_9ce4_8422_2325 ^ file.len() as u64;
    let mut bytes = file.as_bytes();
    while let Some((chunk, rest)) = bytes.split_first_chunk::<8>() {
        hash = mix(hash, u64::from_le_bytes(*chunk));
        bytes = rest;
    }
    let mut tail = [0_u8; 8];
    for (slot, byte) in tail.iter_mut().zip(bytes) {
        *slot = *byte;
    }
    hash = mix(hash, u64::from_le_bytes(tail));
    hash = mix(hash, u64::from(line) << 32 | u64::from(column));
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
        assert_ne!(key("src/a.rs", 1, 2), key("src/a.rs\0", 1, 2));
        assert_ne!(
            key("src/lib/one/mod.rs", 1, 2),
            key("src/lib/two/mod.rs", 1, 2)
        );
    }

    /// A collision silences a warning, so no two of a large set of sites may
    /// share a key.
    #[test]
    fn test_key_has_no_collision_in_a_large_set() {
        extern crate std;
        let (files, lines) = if cfg!(miri) { (2, 16) } else { (64, 512) };
        let mut keys = std::collections::HashSet::new();
        for file in 0..files {
            let name = std::format!("crates/app/src/module_{file}/mod.rs");
            for line in 1..=lines {
                for column in [1, 5, 9, 13, 17, 21, 25, 29] {
                    assert!(keys.insert(key(&name, line, column)));
                }
            }
        }
    }
}
