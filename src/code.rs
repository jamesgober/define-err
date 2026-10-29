//! Error codes: `Namespace`, `Code`, `CodeParseError`, and their text form.
//!
//! A code is a namespace plus a number in one `u64`:
//!
//! ```text
//!  bit  63                     32 31  29 28   24 23                       0
//!      [ namespace: 32 bits      ][width][reserved][ number: 24 bits        ]
//! ```
//!
//! The namespace field is the text read as a base-37 number, first character
//! most significant: `'0'`-`'9'` are 1 to 10, `'A'`-`'Z'` are 11 to 36, and 0
//! pads after the last character. The first character is a letter, so the
//! field is at least 11 x 37^5 and a code is never zero. Golden values are in
//! `tests/golden/codes.txt` and frozen at 1.0.

use core::{fmt, num::NonZeroU64};

/// 37^6: one past the largest namespace field.
const NAMESPACE_LIMIT: u32 = 2_565_726_409;
/// Powers of 37 for the six character positions, most significant first.
const POWERS: [u32; 6] = [69_343_957, 1_874_161, 50_653, 1_369, 37, 1];
/// The width bits (29 to 31).
const WIDTH_MASK: u64 = 0b111 << 29;
/// The reserved bits (24 to 28).
const RESERVED_MASK: u64 = 0b1_1111 << 24;
/// The number bits (0 to 23).
const NUMBER_MASK: u64 = 0xFF_FFFF;
/// The longest text a code can have: six characters, `-`, eight digits.
const MAX_TEXT: usize = 15;

/// The base-37 value of one character, or 0 when it is not allowed.
const fn char_value(byte: u8) -> u32 {
    match byte {
        b'0'..=b'9' => (byte - b'0') as u32 + 1,
        b'A'..=b'Z' => (byte - b'A') as u32 + 11,
        b'a'..=b'z' => (byte - b'a') as u32 + 11,
        _ => 0,
    }
}

/// The character for a base-37 value from 1 to 36.
const fn value_char(value: u32) -> u8 {
    if value <= 10 {
        b'0' + (value as u8 - 1)
    } else {
        b'A' + (value as u8 - 11)
    }
}

/// Packs namespace text into its 32-bit field.
const fn pack_text(bytes: &[u8]) -> Result<u32, CodeParseError> {
    if bytes.is_empty() || bytes.len() > 6 {
        return Err(CodeParseError::BadNamespace);
    }
    if !bytes[0].is_ascii_alphabetic() {
        return Err(CodeParseError::BadNamespace);
    }
    let mut field = 0_u32;
    let mut i = 0;
    while i < bytes.len() {
        let value = char_value(bytes[i]);
        if value == 0 {
            return Err(CodeParseError::BadNamespace);
        }
        field += value * POWERS[i];
        i += 1;
    }
    Ok(field)
}

/// Checks that a 32-bit field is a valid namespace: below 37^6, a letter
/// first, and no character after the padding.
const fn valid_field(field: u32) -> bool {
    if field >= NAMESPACE_LIMIT {
        return false;
    }
    let first = field / POWERS[0];
    if first < 11 {
        return false;
    }
    let mut padded = false;
    let mut i = 1;
    while i < 6 {
        let value = (field / POWERS[i]) % 37;
        if value == 0 {
            padded = true;
        } else if padded {
            return false;
        }
        i += 1;
    }
    true
}

/// The namespace of a code: one to six characters, `A` to `Z` and `0` to `9`,
/// starting with a letter, plus the number of digits a code is shown with.
///
/// Equality, ordering, and hashing use the text only; the width is
/// presentation. Ordering is alphabetical (digits before letters, as in ASCII).
///
/// # Examples
///
/// ```
/// use define_err::{namespace, Namespace};
///
/// const DB: Namespace = namespace!("DB");
/// const DIS: Namespace = namespace!("DIS", width = 5);
///
/// assert_eq!(&*DB.as_str(), "DB");
/// assert_eq!(DIS.width(), 5);
/// assert_eq!(Namespace::try_new("db").unwrap(), DB);
/// assert!(Namespace::try_new("1DB").is_err());
/// assert_eq!(DIS.code(1).unwrap().to_string(), "DIS-00001");
/// ```
#[derive(Clone, Copy)]
pub struct Namespace {
    field: u32,
    width: u8,
}

impl Namespace {
    /// A namespace from its text, shown with four digits.
    ///
    /// Lowercase letters are accepted and stored uppercase.
    ///
    /// # Errors
    ///
    /// `CodeParseError::BadNamespace` when the text is empty, longer than six
    /// characters, starts with a digit, or has a character outside `A`-`Z`,
    /// `a`-`z`, `0`-`9`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Namespace;
    ///
    /// assert_eq!(&*Namespace::try_new("auth").unwrap().as_str(), "AUTH");
    /// assert!(Namespace::try_new("TOOLONG").is_err());
    /// ```
    pub const fn try_new(text: &str) -> Result<Namespace, CodeParseError> {
        Namespace::try_with_width(text, 4)
    }

    /// A namespace from its text and the number of digits its codes are shown
    /// with (1 to 8).
    ///
    /// # Errors
    ///
    /// `CodeParseError::BadNamespace` for invalid text (see
    /// [`try_new`](Namespace::try_new)); `CodeParseError::BadWidth` for a width
    /// outside 1 to 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Namespace;
    ///
    /// let dis = Namespace::try_with_width("DIS", 5).unwrap();
    /// assert_eq!(dis.width(), 5);
    /// assert!(Namespace::try_with_width("DIS", 9).is_err());
    /// ```
    pub const fn try_with_width(text: &str, width: u8) -> Result<Namespace, CodeParseError> {
        if width == 0 || width > 8 {
            return Err(CodeParseError::BadWidth);
        }
        match pack_text(text.as_bytes()) {
            Ok(field) => Ok(Namespace { field, width }),
            Err(error) => Err(error),
        }
    }

    /// The text, on the stack, without allocating.
    ///
    /// # Examples
    ///
    /// ```
    /// let ns = define_err::namespace!("CFG");
    /// assert_eq!(&*ns.as_str(), "CFG");
    /// assert_eq!(ns.as_str().to_string(), "CFG");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> NamespaceStr {
        let mut bytes = [0_u8; 6];
        let mut len = 0;
        for (i, power) in POWERS.iter().enumerate() {
            let value = (self.field / power) % 37;
            if value == 0 {
                break;
            }
            bytes[i] = value_char(value);
            len = i + 1;
        }
        NamespaceStr {
            bytes,
            len: len as u8,
        }
    }

    /// The number of digits codes in this namespace are shown with.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::namespace!("DB").width(), 4);
    /// ```
    #[must_use]
    pub const fn width(self) -> u8 {
        self.width
    }

    /// A code in this namespace, or `None` when `number` is above
    /// [`Code::MAX_NUMBER`].
    ///
    /// # Examples
    ///
    /// ```
    /// let db = define_err::namespace!("DB");
    /// assert_eq!(db.code(42).unwrap().to_string(), "DB-0042");
    /// assert!(db.code(16_777_216).is_none());
    /// ```
    #[must_use]
    pub const fn code(self, number: u32) -> Option<Code> {
        Code::new(self, number)
    }
}

impl PartialEq for Namespace {
    fn eq(&self, other: &Namespace) -> bool {
        self.field == other.field
    }
}

impl Eq for Namespace {}

impl PartialOrd for Namespace {
    fn partial_cmp(&self, other: &Namespace) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Namespace {
    fn cmp(&self, other: &Namespace) -> core::cmp::Ordering {
        self.field.cmp(&other.field)
    }
}

impl core::hash::Hash for Namespace {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.field.hash(state);
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&self.as_str())
    }
}

impl fmt::Debug for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Namespace({}, width {})", &*self.as_str(), self.width)
    }
}

/// The text of a [`Namespace`], up to six bytes held on the stack.
///
/// # Examples
///
/// ```
/// let text = define_err::namespace!("GW").as_str();
/// assert_eq!(text.len(), 2);
/// ```
#[derive(Clone, Copy)]
pub struct NamespaceStr {
    bytes: [u8; 6],
    len: u8,
}

impl core::ops::Deref for NamespaceStr {
    type Target = str;

    fn deref(&self) -> &str {
        // The bytes are always ASCII letters and digits, so this never fails.
        core::str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or("")
    }
}

impl fmt::Display for NamespaceStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self)
    }
}

impl fmt::Debug for NamespaceStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// A stable error code: a namespace plus a number, shown `DB-0042`.
///
/// One `u64`, so codes compare, sort, and hash as integers and travel between
/// processes and nodes as-is; no registry is needed to show or compare one.
/// The value is never zero, so `Option<Code>` is 8 bytes. Equality, ordering,
/// and hashing ignore the display width: `DB-42` and `DB-0042` are the same
/// code.
///
/// # Examples
///
/// ```
/// use define_err::{code, Code};
///
/// const TIMEOUT: Code = code!("DB", 1);
///
/// assert_eq!(TIMEOUT.to_string(), "DB-0001");
/// assert_eq!(TIMEOUT.number(), 1);
/// assert_eq!(Code::from_u64(TIMEOUT.as_u64()), Some(TIMEOUT));
/// assert_eq!(Code::parse("db-1"), Ok(TIMEOUT));
/// assert_eq!(core::mem::size_of::<Option<Code>>(), 8);
/// ```
#[derive(Clone, Copy)]
pub struct Code(NonZeroU64);

impl Code {
    /// The largest code number, 16,777,215.
    pub const MAX_NUMBER: u32 = 16_777_215;

    /// The code `number` in `namespace`, or `None` when `number` is above
    /// [`MAX_NUMBER`](Code::MAX_NUMBER).
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{namespace, Code};
    ///
    /// assert!(Code::new(namespace!("DB"), 7).is_some());
    /// assert!(Code::new(namespace!("DB"), Code::MAX_NUMBER + 1).is_none());
    /// ```
    #[must_use]
    pub const fn new(namespace: Namespace, number: u32) -> Option<Code> {
        if number > Code::MAX_NUMBER {
            return None;
        }
        let raw = ((namespace.field as u64) << 32)
            | (((namespace.width - 1) as u64) << 29)
            | number as u64;
        match NonZeroU64::new(raw) {
            Some(value) => Some(Code(value)),
            None => None,
        }
    }

    /// The namespace, with this code's display width.
    ///
    /// # Examples
    ///
    /// ```
    /// let code = define_err::code!("DB", 42);
    /// assert_eq!(&*code.namespace().as_str(), "DB");
    /// ```
    #[must_use]
    pub const fn namespace(self) -> Namespace {
        let raw = self.0.get();
        Namespace {
            field: (raw >> 32) as u32,
            width: ((raw & WIDTH_MASK) >> 29) as u8 + 1,
        }
    }

    /// The number, 0 to 16,777,215.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::code!("DB", 42).number(), 42);
    /// ```
    #[must_use]
    pub const fn number(self) -> u32 {
        (self.0.get() & NUMBER_MASK) as u32
    }

    /// Whether this is the namespace's generic code, number 0.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(define_err::code!("DB", 0).is_generic());
    /// ```
    #[must_use]
    pub const fn is_generic(self) -> bool {
        self.number() == 0
    }

    /// The full value, width included, for storage and the wire.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::code!("DB", 1).as_u64(), 0x3b34_a2f2_6000_0001);
    /// ```
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0.get()
    }

    /// The value with the width bits cleared: what `Eq`, `Ord`, and `Hash`
    /// use.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::code!("DB", 1).identity(), 0x3b34_a2f2_0000_0001);
    /// ```
    #[must_use]
    pub const fn identity(self) -> u64 {
        self.0.get() & !WIDTH_MASK
    }

    /// A code from its `u64` value, or `None` for anything that is not exactly
    /// a valid code: zero, reserved bits set, or a namespace field that is out
    /// of range, starts with a digit, or has a character after its padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Code;
    ///
    /// let code = define_err::code!("DB", 42);
    /// assert_eq!(Code::from_u64(code.as_u64()), Some(code));
    /// assert_eq!(Code::from_u64(0), None);
    /// assert_eq!(Code::from_u64(code.as_u64() | 1 << 24), None);
    /// ```
    #[must_use]
    pub const fn from_u64(raw: u64) -> Option<Code> {
        if raw & RESERVED_MASK != 0 {
            return None;
        }
        if !valid_field((raw >> 32) as u32) {
            return None;
        }
        match NonZeroU64::new(raw) {
            Some(value) => Some(Code(value)),
            None => None,
        }
    }

    /// Parses `NS-0042`: one to six letters or digits starting with a letter,
    /// `-`, and one to eight decimal digits. The display width becomes the
    /// number of digits written.
    ///
    /// # Errors
    ///
    /// See [`CodeParseError`]. Surrounding whitespace, signs, `0x`, Unicode
    /// hyphens and digits, a second separator, and text longer than 15 bytes
    /// are all rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Code, CodeParseError};
    ///
    /// assert_eq!(Code::parse("DB-0042").unwrap().to_string(), "DB-0042");
    /// assert_eq!(Code::parse("DB-42"), Code::parse("DB-00000042"));
    /// assert_eq!(Code::parse("DB-0x2A"), Err(CodeParseError::BadNumber));
    /// assert_eq!(Code::parse(" DB-1"), Err(CodeParseError::BadNamespace));
    /// ```
    pub fn parse(text: &str) -> Result<Code, CodeParseError> {
        let bytes = text.as_bytes();
        if bytes.is_empty() {
            return Err(CodeParseError::Empty);
        }
        if bytes.len() > MAX_TEXT {
            return Err(CodeParseError::TooLong);
        }
        let separator = bytes
            .iter()
            .position(|&b| b == b'-')
            .ok_or(CodeParseError::MissingSeparator)?;
        let field = pack_text(&bytes[..separator])?;
        let digits = &bytes[separator + 1..];
        if digits.is_empty() || digits.len() > 8 || !digits.iter().all(u8::is_ascii_digit) {
            return Err(CodeParseError::BadNumber);
        }
        let number = digits
            .iter()
            .fold(0_u32, |n, &d| n * 10 + u32::from(d - b'0'));
        if number > Code::MAX_NUMBER {
            return Err(CodeParseError::NumberTooLarge);
        }
        let namespace = Namespace {
            field,
            width: digits.len() as u8,
        };
        Code::new(namespace, number).ok_or(CodeParseError::NumberTooLarge)
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Code) -> bool {
        self.identity() == other.identity()
    }
}

impl Eq for Code {}

impl PartialOrd for Code {
    fn partial_cmp(&self, other: &Code) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Code {
    fn cmp(&self, other: &Code) -> core::cmp::Ordering {
        self.identity().cmp(&other.identity())
    }
}

impl core::hash::Hash for Code {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.identity().hash(state);
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let namespace = self.namespace();
        write!(
            f,
            "{}-{:0width$}",
            &*namespace.as_str(),
            self.number(),
            width = usize::from(namespace.width)
        )
    }
}

impl fmt::Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Code({self})")
    }
}

impl core::str::FromStr for Code {
    type Err = CodeParseError;

    fn from_str(text: &str) -> Result<Code, CodeParseError> {
        Code::parse(text)
    }
}

/// Why a code, a namespace, or a width was rejected.
///
/// The messages describe the input class and the fix; they never repeat the
/// input, so a hostile string cannot reach a log through this error.
///
/// # Examples
///
/// ```
/// use define_err::{Code, CodeParseError};
///
/// let err = Code::parse("DB-0x2A").unwrap_err();
/// assert_eq!(err, CodeParseError::BadNumber);
/// assert_eq!(
///     err.to_string(),
///     "code text is not valid: the number must be one to eight decimal digits, for example DB-0042"
/// );
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum CodeParseError {
    /// The text is empty. Pass a code such as `DB-0042`.
    Empty,
    /// The text is longer than 15 bytes: six namespace characters, `-`, and
    /// eight digits at most.
    TooLong,
    /// There is no `-` between the namespace and the number. Write `NS-0042`.
    MissingSeparator,
    /// The namespace is empty, longer than six characters, starts with a
    /// digit, or has a character outside `A`-`Z`, `0`-`9`. Use one to six
    /// letters or digits, starting with a letter.
    BadNamespace,
    /// The display width is outside 1 to 8.
    BadWidth,
    /// The number is empty, is not decimal ASCII digits, or has more than
    /// eight digits.
    BadNumber,
    /// The number is above 16,777,215.
    NumberTooLarge,
    /// A decoded value has reserved bits set: it is not a code from this
    /// version.
    ReservedBits,
}

impl fmt::Display for CodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CodeParseError::Empty => "code text is empty: pass a code such as DB-0042",
            CodeParseError::TooLong => {
                "code text is too long: a code is at most six namespace characters, `-`, and eight digits"
            }
            CodeParseError::MissingSeparator => {
                "code text is not valid: put `-` between the namespace and the number, for example DB-0042"
            }
            CodeParseError::BadNamespace => {
                "code namespace is not valid: use one to six letters or digits, starting with a letter"
            }
            CodeParseError::BadWidth => "code width is not valid: use 1 to 8 digits",
            CodeParseError::BadNumber => {
                "code text is not valid: the number must be one to eight decimal digits, for example DB-0042"
            }
            CodeParseError::NumberTooLarge => "code number is too large: codes are numbered 0 to 16777215",
            CodeParseError::ReservedBits => "code value is not valid: it has reserved bits set",
        })
    }
}

impl core::error::Error for CodeParseError {}

/// A namespace constant, checked at compile time.
///
/// `namespace!("DB")` has width 4; `namespace!("DIS", width = 5)` sets it. An
/// invalid literal does not compile.
///
/// # Examples
///
/// ```
/// use define_err::{namespace, Namespace};
///
/// const DIS: Namespace = namespace!("DIS", width = 5);
/// assert_eq!(DIS.code(1).unwrap().to_string(), "DIS-00001");
/// ```
#[macro_export]
macro_rules! namespace {
    ($text:literal) => {{
        const NAMESPACE: $crate::Namespace = $crate::__private::namespace($text, 4);
        NAMESPACE
    }};
    ($text:literal, width = $width:literal) => {{
        const NAMESPACE: $crate::Namespace = $crate::__private::namespace($text, $width);
        NAMESPACE
    }};
}

/// A code constant, checked at compile time.
///
/// `code!("DB", 42)` uses a namespace literal with width 4; `code!(DIS, 1)`
/// uses a `Namespace` constant. An invalid namespace or number does not
/// compile.
///
/// # Examples
///
/// ```
/// use define_err::{code, namespace, Code, Namespace};
///
/// const DIS: Namespace = namespace!("DIS", width = 5);
/// const CONFIG: Code = code!(DIS, 1);
/// const TIMEOUT: Code = code!("DB", 1);
///
/// assert_eq!(CONFIG.to_string(), "DIS-00001");
/// assert_eq!(TIMEOUT.to_string(), "DB-0001");
/// ```
#[macro_export]
macro_rules! code {
    ($namespace:literal, $number:literal) => {{
        const CODE: $crate::Code =
            $crate::__private::code($crate::__private::namespace($namespace, 4), $number);
        CODE
    }};
    ($namespace:path, $number:literal) => {{
        const CODE: $crate::Code = $crate::__private::code($namespace, $number);
        CODE
    }};
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::{format, string::ToString};

    use super::{Code, CodeParseError, Namespace};

    fn ns(text: &str) -> Namespace {
        Namespace::try_new(text).unwrap()
    }

    #[test]
    fn test_namespace_text_round_trip_and_uppercase() {
        for text in ["A", "DB", "DIS", "AUTH", "GW", "Z9", "ZZZZZZ", "A00000"] {
            assert_eq!(&*ns(text).as_str(), text);
        }
        assert_eq!(&*ns("db").as_str(), "DB");
        assert_eq!(ns("db"), ns("DB"));
    }

    #[test]
    fn test_namespace_invalid_text_rejected() {
        for text in ["", "DB-1", "D B", "1DB", "ABCDEFG", "ÄB", "D_B"] {
            assert_eq!(
                Namespace::try_new(text),
                Err(CodeParseError::BadNamespace),
                "{text:?}"
            );
        }
    }

    #[test]
    fn test_namespace_width_bounds() {
        assert!(Namespace::try_with_width("DB", 1).is_ok());
        assert!(Namespace::try_with_width("DB", 8).is_ok());
        assert_eq!(
            Namespace::try_with_width("DB", 0),
            Err(CodeParseError::BadWidth)
        );
        assert_eq!(
            Namespace::try_with_width("DB", 9),
            Err(CodeParseError::BadWidth)
        );
    }

    #[test]
    fn test_code_number_bounds() {
        let db = ns("DB");
        assert!(db.code(0).unwrap().is_generic());
        assert_eq!(db.code(Code::MAX_NUMBER).unwrap().number(), 16_777_215);
        assert!(db.code(Code::MAX_NUMBER + 1).is_none());
    }

    #[test]
    fn test_code_display_width_and_wide_numbers() {
        assert_eq!(ns("DB").code(42).unwrap().to_string(), "DB-0042");
        assert_eq!(ns("DB").code(123_456).unwrap().to_string(), "DB-123456");
        let dis = Namespace::try_with_width("DIS", 5).unwrap();
        assert_eq!(dis.code(1).unwrap().to_string(), "DIS-00001");
        assert_eq!(format!("{:?}", ns("DB").code(42).unwrap()), "Code(DB-0042)");
    }

    #[test]
    fn test_code_parse_valid_forms_equal_by_identity() {
        let a = Code::parse("db-0042").unwrap();
        let b = Code::parse("DB-42").unwrap();
        let c = Code::parse("DB-00000042").unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a.namespace().width(), 4);
        assert_eq!(b.namespace().width(), 2);
        assert_eq!(c.namespace().width(), 8);
        assert_eq!(b.to_string(), "DB-42");
    }

    #[test]
    fn test_code_parse_errors() {
        assert_eq!(Code::parse(""), Err(CodeParseError::Empty));
        assert_eq!(Code::parse("-"), Err(CodeParseError::BadNamespace));
        assert_eq!(Code::parse("DB"), Err(CodeParseError::MissingSeparator));
        assert_eq!(Code::parse("DB-"), Err(CodeParseError::BadNumber));
        assert_eq!(Code::parse("-0042"), Err(CodeParseError::BadNamespace));
        assert_eq!(Code::parse("DB-123456789"), Err(CodeParseError::BadNumber));
        assert_eq!(
            Code::parse("DB-16777216"),
            Err(CodeParseError::NumberTooLarge)
        );
        assert_eq!(
            Code::parse("ABCDEFG-12345678"),
            Err(CodeParseError::TooLong)
        );
        for text in [
            " DB-1", "DB-1 ", "DB--1", "DB-+1", "DB-0x2A", "DB-1-2", "DB‐1", "DB-１", "DB-٣",
        ] {
            assert!(Code::parse(text).is_err(), "{text:?}");
        }
    }

    #[test]
    fn test_code_from_u64_rejects_invalid_values() {
        let code = ns("DB").code(42).unwrap();
        assert_eq!(Code::from_u64(code.as_u64()), Some(code));
        assert_eq!(Code::from_u64(0), None);
        assert_eq!(Code::from_u64(code.as_u64() | 1 << 24), None);
        assert_eq!(
            Code::from_u64(u64::from(super::NAMESPACE_LIMIT) << 32),
            None
        );
        // "D", padding, "B": a character after the padding.
        let gap = (14 * super::POWERS[0] + 12 * super::POWERS[2]) as u64;
        assert_eq!(Code::from_u64(gap << 32), None);
        // A digit first.
        assert_eq!(Code::from_u64(u64::from(5 * super::POWERS[0]) << 32), None);
    }

    #[test]
    fn test_code_ordering_is_namespace_then_number() {
        let order = ["DB-0002", "DB-0010", "DBA-0001", "E-0000"].map(|t| Code::parse(t).unwrap());
        assert!(order.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn test_code_hash_ignores_width() {
        use core::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let hash = |c: Code| {
            let mut h = DefaultHasher::new();
            c.hash(&mut h);
            h.finish()
        };
        assert_eq!(
            hash(Code::parse("DIS-0001").unwrap()),
            hash(Code::parse("DIS-00001").unwrap())
        );
    }

    #[test]
    fn test_code_parse_error_display_never_echoes_input() {
        for text in [
            "DB-0x2A",
            "\n\u{1b}[31mFORGED",
            "",
            "DB--1",
            "AAAAAAAAAAAAAAAAAAAAA",
        ] {
            let message = Code::parse(text).unwrap_err().to_string();
            assert!(text.is_empty() || !message.contains(text), "{message}");
            assert!(message.bytes().all(|b| b >= 0x20));
        }
    }

    #[test]
    fn test_option_code_is_eight_bytes() {
        assert_eq!(core::mem::size_of::<Option<Code>>(), 8);
    }
}
