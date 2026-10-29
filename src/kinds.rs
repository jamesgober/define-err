//! `Kinds`: a 64-bit set of error kinds, PHP `E_*` style.
//!
//! Bits 0 to 31 are platform kinds with fixed meaning, never reassigned;
//! bits 32 to 63 are the project's own, from `Kinds::project::<N>()`. The
//! assignments are in `tests/golden/kinds.txt` and frozen at 1.0.

use core::{fmt, ops};

/// A set of error kinds: each kind is one bit, masks combine with `|`.
///
/// Severity is an order and answers "warnings and above"; kinds are a set and
/// answer "security or billing, at any level". Filters need both.
///
/// # Examples
///
/// ```
/// use define_err::Kinds;
///
/// pub const BILLING: Kinds = Kinds::project::<0>();
/// const SECURITY_OR_BILLING: Kinds = Kinds::SECURITY.union(BILLING);
///
/// let kinds = Kinds::ERROR | Kinds::SECURITY;
/// assert!(kinds.contains(Kinds::SECURITY));
/// assert!(kinds.intersects(SECURITY_OR_BILLING));
/// assert!(!kinds.contains(SECURITY_OR_BILLING));
/// assert_eq!(kinds.to_string(), "ERROR|SECURITY");
/// assert_eq!(BILLING.to_string(), "project(0)");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Kinds(u64);

/// Names of the platform bits in use, index = bit.
const NAMES: [&str; 13] = [
    "ERROR", "WARNING", "NOTICE", "DEPRECATED", "SECURITY", "FUTURE", "RUNTIME", "RESOURCE",
    "CONFIG", "USER", "PANIC", "AUDIT", "TRIGGER",
];

impl Kinds {
    /// No kind. `contains(NONE)` is true for every set.
    pub const NONE: Kinds = Kinds(0);
    /// Every bit.
    pub const ALL: Kinds = Kinds(u64::MAX);
    /// Bit 0: an error; the default for reported errors.
    pub const ERROR: Kinds = Kinds(1 << 0);
    /// Bit 1: a warning.
    pub const WARNING: Kinds = Kinds(1 << 1);
    /// Bit 2: an informational notice.
    pub const NOTICE: Kinds = Kinds(1 << 2);
    /// Bit 3: use of something deprecated.
    pub const DEPRECATED: Kinds = Kinds(1 << 3);
    /// Bit 4: security relevant; delivered to security listeners.
    pub const SECURITY: Kinds = Kinds(1 << 4);
    /// Bit 5: behaviour that will change in a later version.
    pub const FUTURE: Kinds = Kinds(1 << 5);
    /// Bit 6: a run-time condition.
    pub const RUNTIME: Kinds = Kinds(1 << 6);
    /// Bit 7: resource pressure (memory, handles, quota, disk).
    pub const RESOURCE: Kinds = Kinds(1 << 7);
    /// Bit 8: a configuration problem.
    pub const CONFIG: Kinds = Kinds(1 << 8);
    /// Bit 9: caused by the caller's input.
    pub const USER: Kinds = Kinds(1 << 9);
    /// Bit 10: a caught panic (set by the dynamic-error and service
    /// libraries, not by users).
    pub const PANIC: Kinds = Kinds(1 << 10);
    /// Bit 11: an action that must reach the audit trail.
    pub const AUDIT: Kinds = Kinds(1 << 11);
    /// Bit 12: raised on purpose by `trigger`.
    pub const TRIGGER: Kinds = Kinds(1 << 12);
    /// Bits 0 to 31, the platform half.
    pub const PLATFORM: Kinds = Kinds(0x0000_0000_FFFF_FFFF);
    /// Bits 32 to 63, the project half.
    pub const PROJECT: Kinds = Kinds(0xFFFF_FFFF_0000_0000);

    /// The project kind `N` (bit `32 + N`). `N` must be below 32: a project can
    /// never take a platform bit, and `Kinds::project::<32>()` does not
    /// compile.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Kinds;
    ///
    /// const BILLING: Kinds = Kinds::project::<0>();
    /// assert_eq!(BILLING.bits(), 1 << 32);
    /// assert!(Kinds::PROJECT.contains(BILLING));
    /// ```
    #[must_use]
    pub const fn project<const N: u8>() -> Kinds {
        const { assert!(N < 32, "Kinds::project: the bit must be below 32") };
        Kinds(1 << (32 + N as u32))
    }

    /// The project kind `bit`, or `None` when `bit` is 32 or more.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Kinds;
    ///
    /// assert_eq!(Kinds::try_project(31).map(Kinds::bits), Some(1 << 63));
    /// assert_eq!(Kinds::try_project(32), None);
    /// ```
    #[must_use]
    pub const fn try_project(bit: u8) -> Option<Kinds> {
        if bit < 32 {
            Some(Kinds(1 << (32 + bit as u32)))
        } else {
            None
        }
    }

    /// Whether every bit of `other` is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Kinds;
    ///
    /// assert!((Kinds::ERROR | Kinds::SECURITY).contains(Kinds::ERROR));
    /// assert!(Kinds::ERROR.contains(Kinds::NONE));
    /// ```
    #[must_use]
    pub const fn contains(self, other: Kinds) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether any bit of `other` is set. `intersects(NONE)` is always false.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Kinds;
    ///
    /// assert!(Kinds::ERROR.intersects(Kinds::ERROR | Kinds::WARNING));
    /// assert!(!Kinds::NONE.intersects(Kinds::NONE));
    /// ```
    #[must_use]
    pub const fn intersects(self, other: Kinds) -> bool {
        self.0 & other.0 != 0
    }

    /// Whether no bit is set.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(define_err::Kinds::NONE.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The set of both: the `const` form of `|`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Kinds;
    ///
    /// const MASK: Kinds = Kinds::ERROR.union(Kinds::SECURITY);
    /// assert_eq!(MASK, Kinds::ERROR | Kinds::SECURITY);
    /// ```
    #[must_use]
    pub const fn union(self, other: Kinds) -> Kinds {
        Kinds(self.0 | other.0)
    }

    /// The bits.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Kinds::SECURITY.bits(), 0x10);
    /// ```
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// A set from stored bits; any bits are accepted. For decoding stored or
    /// received values.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Kinds;
    ///
    /// assert_eq!(Kinds::from_bits(0x11), Kinds::ERROR | Kinds::SECURITY);
    /// ```
    #[must_use]
    pub const fn from_bits(bits: u64) -> Kinds {
        Kinds(bits)
    }

    /// The single-bit kinds in the set, lowest bit first.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Kinds;
    ///
    /// let all: Vec<Kinds> = (Kinds::SECURITY | Kinds::ERROR).iter().collect();
    /// assert_eq!(all, [Kinds::ERROR, Kinds::SECURITY]);
    /// ```
    #[must_use]
    pub fn iter(self) -> KindsIter {
        KindsIter(self.0)
    }
}

impl ops::BitOr for Kinds {
    type Output = Kinds;

    fn bitor(self, other: Kinds) -> Kinds {
        Kinds(self.0 | other.0)
    }
}

impl ops::BitOrAssign for Kinds {
    fn bitor_assign(&mut self, other: Kinds) {
        self.0 |= other.0;
    }
}

impl ops::BitAnd for Kinds {
    type Output = Kinds;

    fn bitand(self, other: Kinds) -> Kinds {
        Kinds(self.0 & other.0)
    }
}

impl ops::BitAndAssign for Kinds {
    fn bitand_assign(&mut self, other: Kinds) {
        self.0 &= other.0;
    }
}

impl ops::Not for Kinds {
    type Output = Kinds;

    fn not(self) -> Kinds {
        Kinds(!self.0)
    }
}

impl ops::Sub for Kinds {
    type Output = Kinds;

    fn sub(self, other: Kinds) -> Kinds {
        Kinds(self.0 & !other.0)
    }
}

impl fmt::Display for Kinds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            return f.write_str("NONE");
        }
        let mut first = true;
        for bit in 0..64_u32 {
            if self.0 & (1 << bit) == 0 {
                continue;
            }
            if !first {
                f.write_str("|")?;
            }
            first = false;
            match NAMES.get(bit as usize) {
                Some(name) => f.write_str(name)?,
                None if bit < 32 => write!(f, "bit({bit})")?,
                None => write!(f, "project({})", bit - 32)?,
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Kinds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kinds({self})")
    }
}

/// The single-bit kinds of a set, lowest bit first. From [`Kinds::iter`].
///
/// # Examples
///
/// ```
/// let mut kinds = define_err::Kinds::ALL.iter();
/// assert_eq!(kinds.next(), Some(define_err::Kinds::ERROR));
/// assert_eq!(kinds.count(), 63);
/// ```
#[derive(Clone, Debug)]
pub struct KindsIter(u64);

impl Iterator for KindsIter {
    type Item = Kinds;

    fn next(&mut self) -> Option<Kinds> {
        if self.0 == 0 {
            return None;
        }
        let low = self.0 & self.0.wrapping_neg();
        self.0 &= !low;
        Some(Kinds(low))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for KindsIter {}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::Kinds;

    #[test]
    fn test_kinds_empty_set_rules() {
        assert!(Kinds::NONE.contains(Kinds::NONE));
        assert!(Kinds::ERROR.contains(Kinds::NONE));
        assert!(!Kinds::NONE.intersects(Kinds::NONE));
    }

    #[test]
    fn test_kinds_project_bits() {
        assert_eq!(Kinds::project::<0>().bits(), 1 << 32);
        assert_eq!(Kinds::project::<31>().bits(), 1 << 63);
        assert_eq!(Kinds::try_project(32), None);
        for bit in 0..32 {
            let kind = Kinds::try_project(bit).unwrap();
            assert!(!kind.intersects(Kinds::PLATFORM));
        }
    }

    #[test]
    fn test_kinds_not_and_display() {
        assert_eq!(!Kinds::NONE, Kinds::ALL);
        assert_eq!(Kinds::NONE.to_string(), "NONE");
        let all = Kinds::ALL.to_string();
        assert!(all.starts_with("ERROR|WARNING|NOTICE"));
        assert!(all.contains("TRIGGER|bit(13)|"));
        assert!(all.ends_with("project(31)"));
        assert_eq!((Kinds::ERROR | Kinds::SECURITY - Kinds::ERROR).to_string(), "ERROR|SECURITY");
        assert_eq!(((Kinds::ERROR | Kinds::SECURITY) - Kinds::ERROR).to_string(), "SECURITY");
    }

    #[test]
    fn test_kinds_iter_counts_bits() {
        assert_eq!(Kinds::ALL.iter().len(), 64);
        assert_eq!(Kinds::NONE.iter().count(), 0);
    }
}
