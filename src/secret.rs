//! `Secret<T>`: a value that never reaches `Display` or `Debug` output.

use core::fmt;

/// The text every format of a [`Secret`] writes.
const REDACTED: &str = "[redacted]";

/// A field type that keeps its value out of every `Display` and `Debug` output.
///
/// Both formats write `[redacted]` whatever the flags: width, fill, and
/// alignment are honoured around the word so tables stay aligned, while
/// precision and the alternate flag are ignored, so `{:.3}` cannot cut the
/// word and reveal its length. The value comes back only through
/// [`expose`](Secret::expose) and [`into_inner`](Secret::into_inner), which are
/// easy to search for in review.
///
/// `Secret` implements neither `Deref`, `AsRef`, `Borrow`, nor `PartialEq`:
/// each would let the value slip into a format string or a timing-unsafe
/// comparison. It does not wipe memory on drop; it only redacts output.
///
/// # Examples
///
/// ```
/// use define_err::{define_errors, Secret};
///
/// define_errors! {
///     pub enum AuthError {
///         BadToken { user: String, token: Secret<String> } => "token {token} rejected for {user}",
///     }
/// }
///
/// let err = AuthError::BadToken { user: "ada".into(), token: Secret::new("s3cr3t".into()) };
/// assert_eq!(err.to_string(), "token [redacted] rejected for ada");
/// assert!(!format!("{err:?}").contains("s3cr3t"));
/// assert_eq!(format!("{:>12}", Secret::new("pw")), "  [redacted]");
/// ```
#[derive(Clone, Copy, Default, Hash)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Wraps a value.
    ///
    /// # Examples
    ///
    /// ```
    /// let token = define_err::Secret::new("s3cr3t");
    /// assert_eq!(token.to_string(), "[redacted]");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(value: T) -> Secret<T> {
        Secret(value)
    }

    /// Borrows the value. Every use of the secret goes through this call.
    ///
    /// # Examples
    ///
    /// ```
    /// let token = define_err::Secret::new(String::from("s3cr3t"));
    /// assert_eq!(token.expose().len(), 6);
    /// ```
    #[inline]
    #[must_use]
    pub const fn expose(&self) -> &T {
        &self.0
    }

    /// Takes the value back out.
    ///
    /// # Examples
    ///
    /// ```
    /// let token = define_err::Secret::new(42_u32);
    /// assert_eq!(token.into_inner(), 42);
    /// ```
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Secret<T> {
    #[inline]
    fn from(value: T) -> Secret<T> {
        Secret(value)
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_redacted(f)
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_redacted(f)
    }
}

/// Writes `[redacted]`, padded to the formatter's width with its fill and
/// alignment; precision and the alternate flag are ignored on purpose.
fn write_redacted(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use fmt::Write as _;

    let width = f.width().unwrap_or(0);
    let pad = width.saturating_sub(REDACTED.len());
    if pad == 0 {
        return f.write_str(REDACTED);
    }
    let fill = f.fill();
    let (before, after) = match f.align() {
        Some(fmt::Alignment::Right) => (pad, 0),
        Some(fmt::Alignment::Center) => (pad / 2, pad - pad / 2),
        Some(fmt::Alignment::Left) | None => (0, pad),
    };
    for _ in 0..before {
        f.write_char(fill)?;
    }
    f.write_str(REDACTED)?;
    for _ in 0..after {
        f.write_char(fill)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::{format, string::String};

    use super::Secret;

    #[test]
    fn test_secret_display_plain_redacted() {
        assert_eq!(format!("{}", Secret::new("pw")), "[redacted]");
        assert_eq!(format!("{:?}", Secret::new("pw")), "[redacted]");
    }

    #[test]
    fn test_secret_width_and_alignment_honoured() {
        assert_eq!(format!("{:>12}", Secret::new("pw")), "  [redacted]");
        assert_eq!(format!("{:<12}|", Secret::new("pw")), "[redacted]  |");
        assert_eq!(format!("{:^13}|", Secret::new("pw")), " [redacted]  |");
        assert_eq!(format!("{:*>11}", Secret::new("pw")), "*[redacted]");
        assert_eq!(format!("{:12}|", Secret::new(1_u8)), "[redacted]  |");
    }

    #[test]
    fn test_secret_precision_and_alternate_ignored() {
        assert_eq!(format!("{:.3}", Secret::new("password")), "[redacted]");
        assert_eq!(format!("{:#?}", Secret::new("password")), "[redacted]");
        assert_eq!(format!("{:5}", Secret::new("password")), "[redacted]");
    }

    #[test]
    fn test_secret_nested_redacted() {
        let inner = Secret::new(Secret::new(String::from("pw")));
        assert_eq!(format!("{inner} {inner:?}"), "[redacted] [redacted]");
        assert_eq!(inner.expose().expose(), "pw");
    }

    #[test]
    fn test_secret_copy_when_value_is_copy() {
        let a = Secret::new("pw");
        let b = a;
        assert_eq!(a.expose(), b.expose());
    }

    #[test]
    fn test_secret_from_and_into_inner_round_trip() {
        let s: Secret<u32> = 7.into();
        assert_eq!(s.into_inner(), 7);
        assert_eq!(*Secret::<u8>::default().expose(), 0);
    }
}
