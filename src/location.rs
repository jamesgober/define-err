//! `Location` and `here()`.

/// Where an error was built or converted: a file, line, and column.
///
/// The type of `#[location]` fields. 8 bytes on 64-bit targets. The generated
/// `From` conversion fills it with the location of the conversion (so `?`
/// records its own line), and [`here`] fills it when a variant is built by hand.
///
/// # Examples
///
/// ```
/// let at: define_err::Location = define_err::here();
/// assert_eq!(at.file(), file!());
/// ```
pub type Location = &'static core::panic::Location<'static>;

/// Returns the location of its caller.
///
/// A `#[track_caller]` function that calls `here()` gets its own caller's
/// location, so a helper that builds errors can record where it was called.
///
/// # Examples
///
/// ```
/// let at = define_err::here();
/// assert_eq!(at.line(), line!() - 1);
/// ```
#[track_caller]
#[inline]
#[must_use]
pub fn here() -> Location {
    core::panic::Location::caller()
}

#[cfg(test)]
mod tests {
    use super::{here, Location};

    #[track_caller]
    fn helper() -> Location {
        here()
    }

    #[test]
    fn test_here_returns_caller_line() {
        let at = here();
        assert_eq!(at.line(), line!() - 1);
        assert_eq!(at.file(), file!());
    }

    #[test]
    fn test_here_through_track_caller_helper_reports_helper_caller() {
        let at = helper();
        assert_eq!(at.line(), line!() - 1);
    }
}
