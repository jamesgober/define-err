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

/// A location's file as a log should show it: an absolute path is cut to the
/// crate's directory and what follows, so a dependency built from a registry
/// does not print the home directory it was unpacked into. A relative path,
/// which is what the crates of a workspace get, is returned unchanged.
///
/// `/home/a/.cargo/registry/src/index.crates.io-x/page-db-1.0.0/src/error.rs`
/// becomes `page-db-1.0.0/src/error.rs`. An absolute path with no `src`
/// directory becomes its file name. One scan of the bytes, and no allocation.
///
/// # Examples
///
/// ```
/// use define_err::short_file;
///
/// assert_eq!(short_file("src/lib.rs"), "src/lib.rs");
/// assert_eq!(short_file("/home/a/.cargo/registry/src/x/page-db-1.0.0/src/error.rs"), "page-db-1.0.0/src/error.rs");
/// assert_eq!(short_file(r"C:\Users\a\.cargo\registry\src\x\page-db-1.0.0\src\pool.rs"), r"page-db-1.0.0\src\pool.rs");
/// ```
#[must_use]
pub fn short_file(file: &str) -> &str {
    let bytes = file.as_bytes();
    let separator = |b: u8| b == b'/' || b == b'\\';
    let absolute = match bytes {
        [first, ..] if separator(*first) => true,
        [drive, b':', next, ..] => drive.is_ascii_alphabetic() && separator(*next),
        _ => false,
    };
    if !absolute {
        return file;
    }
    // The crate directory is the component just before the last `src`; with
    // none before it, the path starts at `src`. Every index here follows an
    // ASCII separator, so it is a character boundary.
    let mut last: Option<usize> = None;
    let mut before: Option<usize> = None;
    let mut from = None;
    for (i, &b) in bytes.iter().enumerate() {
        if !separator(b) {
            continue;
        }
        if let Some(start) = last {
            if bytes.get(start + 1..i) == Some(b"src".as_slice()) {
                from = Some(before.unwrap_or(start) + 1);
            }
        }
        before = last;
        last = Some(i);
    }
    let from = from.or(last.map(|end| end + 1)).unwrap_or(0);
    file.get(from..).unwrap_or(file)
}

#[cfg(test)]
mod tests {
    use super::{here, short_file, Location};

    #[test]
    fn test_short_file_keeps_relative_paths() {
        for file in [
            "src/lib.rs",
            "crates/exec/src/lib.rs",
            "lib.rs",
            "",
            "src\\a.rs",
        ] {
            assert_eq!(short_file(file), file);
        }
    }

    #[test]
    fn test_short_file_cuts_absolute_paths_to_the_crate() {
        let cases = [
            (
                "/home/a/.cargo/registry/src/i-1/page-db-1.0.0/src/error.rs",
                "page-db-1.0.0/src/error.rs",
            ),
            ("/build/app/src/main.rs", "app/src/main.rs"),
            ("/build/app/src/net/tcp.rs", "app/src/net/tcp.rs"),
            (
                r"C:\Users\a\.cargo\registry\src\i-1\page-db-1.0.0\src\pool.rs",
                r"page-db-1.0.0\src\pool.rs",
            ),
            (r"\\server\share\app\src\lib.rs", r"app\src\lib.rs"),
            ("/build/app/tests/it.rs", "it.rs"),
            ("/src/lib.rs", "src/lib.rs"),
            ("/", ""),
        ];
        for (file, short) in cases {
            assert_eq!(short_file(file), short, "{file}");
        }
    }

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
