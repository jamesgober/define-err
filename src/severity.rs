//! `Severity` on the OpenTelemetry SeverityNumber scale, `Band`, and
//! project level-name tables.

use core::{fmt, num::NonZeroU8};

/// A severity from 1 to 24, OpenTelemetry SeverityNumber: higher is more
/// severe.
///
/// Bands: TRACE 1-4, DEBUG 5-8, INFO 9-12, WARN 13-16, ERROR 17-20, FATAL
/// 21-24. "Unspecified" is not a severity; use `Option<Severity>`, which is
/// one byte. The table is in `tests/golden/severity.txt` and frozen at 1.0.
///
/// # Examples
///
/// ```
/// use define_err::Severity;
///
/// assert!(Severity::CRITICAL > Severity::ERROR);
/// assert_eq!(Severity::new(20), Some(Severity::CRITICAL));
/// assert_eq!(Severity::CRITICAL.otel_name(), "ERROR4");
/// assert_eq!(Severity::parse("warn"), Some(Severity::WARNING));
/// assert_eq!(Severity::at::<22>().to_string(), "FATAL2");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Severity(NonZeroU8);

/// A severity from a value already known to be 1 to 24.
const fn severity(value: u8) -> Severity {
    match NonZeroU8::new(value) {
        Some(value) => Severity(value),
        None => Severity(NonZeroU8::MIN),
    }
}

/// OpenTelemetry short names, index = value - 1.
const OTEL_NAMES: [&str; 24] = [
    "TRACE", "TRACE2", "TRACE3", "TRACE4", "DEBUG", "DEBUG2", "DEBUG3", "DEBUG4", "INFO", "INFO2",
    "INFO3", "INFO4", "WARN", "WARN2", "WARN3", "WARN4", "ERROR", "ERROR2", "ERROR3", "ERROR4",
    "FATAL", "FATAL2", "FATAL3", "FATAL4",
];

impl Severity {
    /// 1: very fine detail.
    pub const TRACE: Severity = severity(1);
    /// 5: debugging; the default for cancelled operations.
    pub const DEBUG: Severity = severity(5);
    /// 9: a routine event.
    pub const INFO: Severity = severity(9);
    /// 10: normal but significant (`notice`).
    pub const NOTICE: Severity = severity(10);
    /// 13: not an error, but needs attention; the default for warnings and
    /// for handled client errors.
    pub const WARNING: Severity = severity(13);
    /// 17: an operation failed; the default for reported errors.
    pub const ERROR: Severity = severity(17);
    /// 20: failed and damaging (data loss, corruption), still recoverable.
    pub const CRITICAL: Severity = severity(20);
    /// 21: the process or service cannot continue.
    pub const FATAL: Severity = severity(21);

    /// A severity checked at compile time: `Severity::at::<25>()` does not
    /// compile.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Severity::at::<17>(), define_err::Severity::ERROR);
    /// ```
    #[must_use]
    pub const fn at<const N: u8>() -> Severity {
        const {
            assert!(
                N >= 1 && N <= 24,
                "Severity::at: a severity must be from 1 to 24 (OpenTelemetry SeverityNumber)"
            )
        };
        severity(N)
    }

    /// A severity, or `None` outside 1 to 24.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Severity;
    ///
    /// assert_eq!(Severity::new(13), Some(Severity::WARNING));
    /// assert_eq!(Severity::new(0), None);
    /// assert_eq!(Severity::new(25), None);
    /// ```
    #[must_use]
    pub const fn new(value: u8) -> Option<Severity> {
        if value >= 1 && value <= 24 {
            Some(severity(value))
        } else {
            None
        }
    }

    /// A severity, with 0 raised to 1 and values above 24 lowered to 24.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Severity;
    ///
    /// assert_eq!(Severity::saturating(0).get(), 1);
    /// assert_eq!(Severity::saturating(200).get(), 24);
    /// ```
    #[must_use]
    pub const fn saturating(value: u8) -> Severity {
        if value == 0 {
            severity(1)
        } else if value > 24 {
            severity(24)
        } else {
            severity(value)
        }
    }

    /// The value, 1 to 24.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Severity::ERROR.get(), 17);
    /// ```
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0.get()
    }

    /// The OpenTelemetry band the severity falls in.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Band, Severity};
    ///
    /// assert_eq!(Severity::CRITICAL.band(), Band::Error);
    /// assert_eq!(Severity::NOTICE.band(), Band::Info);
    /// ```
    #[must_use]
    pub const fn band(self) -> Band {
        match (self.get() - 1) / 4 {
            0 => Band::Trace,
            1 => Band::Debug,
            2 => Band::Info,
            3 => Band::Warn,
            4 => Band::Error,
            _ => Band::Fatal,
        }
    }

    /// The OpenTelemetry short name: `ERROR`, `ERROR2`, ..., `ERROR4`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Severity::NOTICE.otel_name(), "INFO2");
    /// ```
    #[must_use]
    pub const fn otel_name(self) -> &'static str {
        OTEL_NAMES[(self.get() - 1) as usize]
    }

    /// The portfolio label for the eight named values, else `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Severity;
    ///
    /// assert_eq!(Severity::at::<20>().label(), Some("CRITICAL"));
    /// assert_eq!(Severity::at::<11>().label(), None);
    /// ```
    #[must_use]
    pub const fn label(self) -> Option<&'static str> {
        match self.get() {
            1 => Some("TRACE"),
            5 => Some("DEBUG"),
            9 => Some("INFO"),
            10 => Some("NOTICE"),
            13 => Some("WARNING"),
            17 => Some("ERROR"),
            20 => Some("CRITICAL"),
            21 => Some("FATAL"),
            _ => None,
        }
    }

    /// Parses a label, an OpenTelemetry short name, or `"1"` to `"24"`,
    /// ignoring case.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Severity;
    ///
    /// assert_eq!(Severity::parse("WARNING"), Some(Severity::WARNING));
    /// assert_eq!(Severity::parse("error4"), Some(Severity::CRITICAL));
    /// assert_eq!(Severity::parse("13"), Some(Severity::WARNING));
    /// assert_eq!(Severity::parse(""), None);
    /// ```
    #[must_use]
    pub fn parse(text: &str) -> Option<Severity> {
        if text.is_empty() || text.len() > 8 {
            return None;
        }
        if text.bytes().all(|b| b.is_ascii_digit()) {
            return text.parse::<u8>().ok().and_then(Severity::new);
        }
        let labels: [(&str, u8); 8] = [
            ("TRACE", 1),
            ("DEBUG", 5),
            ("INFO", 9),
            ("NOTICE", 10),
            ("WARNING", 13),
            ("ERROR", 17),
            ("CRITICAL", 20),
            ("FATAL", 21),
        ];
        if let Some(&(_, value)) = labels
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(text))
        {
            return Severity::new(value);
        }
        OTEL_NAMES
            .iter()
            .position(|name| name.eq_ignore_ascii_case(text))
            .and_then(|index| Severity::new(index as u8 + 1))
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.label().unwrap_or(self.otel_name()))
    }
}

impl fmt::Debug for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Severity({} {})", self.get(), self)
    }
}

/// The OpenTelemetry band of a severity, for sinks that only know six levels.
///
/// # Examples
///
/// ```
/// use define_err::{Band, Severity};
///
/// assert_eq!(Severity::FATAL.band(), Band::Fatal);
/// assert!(Band::Warn < Band::Error);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Band {
    /// 1 to 4.
    Trace,
    /// 5 to 8.
    Debug,
    /// 9 to 12.
    Info,
    /// 13 to 16.
    Warn,
    /// 17 to 20.
    Error,
    /// 21 to 24.
    Fatal,
}

/// A project's own names for severities.
///
/// A plain slice of `(name, severity)` pairs, so the table can be a `const`;
/// the error service can build one from configuration.
///
/// # Examples
///
/// ```
/// use define_err::{LevelNames, Severity};
///
/// const LEVELS: LevelNames = LevelNames::new(&[
///     ("minor", Severity::WARNING),
///     ("major", Severity::ERROR),
///     ("billing-alert", Severity::at::<22>()),
/// ]);
///
/// assert_eq!(LEVELS.name_of(Severity::CRITICAL), Some("major"));
/// assert_eq!(LEVELS.severity_of("BILLING-ALERT"), Some(Severity::at::<22>()));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LevelNames {
    pairs: &'static [(&'static str, Severity)],
}

impl LevelNames {
    /// A table from its pairs, in any order.
    ///
    /// # Examples
    ///
    /// ```
    /// let levels = define_err::LevelNames::new(&[("bad", define_err::Severity::ERROR)]);
    /// assert_eq!(levels.name_of(define_err::Severity::FATAL), Some("bad"));
    /// ```
    #[must_use]
    pub const fn new(pairs: &'static [(&'static str, Severity)]) -> LevelNames {
        LevelNames { pairs }
    }

    /// The name of `severity`, else the name of the nearest lower severity in
    /// the table, else `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{LevelNames, Severity};
    ///
    /// let levels = LevelNames::new(&[("minor", Severity::WARNING)]);
    /// assert_eq!(levels.name_of(Severity::NOTICE), None);
    /// ```
    #[must_use]
    pub fn name_of(&self, severity: Severity) -> Option<&'static str> {
        self.pairs
            .iter()
            .filter(|(_, level)| *level <= severity)
            .max_by_key(|(_, level)| *level)
            .map(|(name, _)| *name)
    }

    /// The severity named `name`, ignoring ASCII case.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{LevelNames, Severity};
    ///
    /// let levels = LevelNames::new(&[("minor", Severity::WARNING)]);
    /// assert_eq!(levels.severity_of("Minor"), Some(Severity::WARNING));
    /// ```
    #[must_use]
    pub fn severity_of(&self, name: &str) -> Option<Severity> {
        self.pairs
            .iter()
            .find(|(entry, _)| entry.eq_ignore_ascii_case(name))
            .map(|(_, level)| *level)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::{Band, Severity};

    #[test]
    fn test_severity_new_bounds() {
        assert_eq!(Severity::new(0), None);
        assert_eq!(Severity::new(1).map(Severity::get), Some(1));
        assert_eq!(Severity::new(24).map(Severity::get), Some(24));
        assert_eq!(Severity::new(25), None);
        assert_eq!(Severity::new(255), None);
    }

    #[test]
    fn test_severity_saturating() {
        assert_eq!(Severity::saturating(0).get(), 1);
        assert_eq!(Severity::saturating(200).get(), 24);
        assert_eq!(Severity::saturating(13), Severity::WARNING);
    }

    #[test]
    fn test_severity_parse_forms() {
        assert_eq!(Severity::parse("warn"), Some(Severity::WARNING));
        assert_eq!(Severity::parse("WARNING"), Some(Severity::WARNING));
        assert_eq!(Severity::parse("error4"), Some(Severity::CRITICAL));
        assert_eq!(Severity::parse("CRITICAL"), Some(Severity::CRITICAL));
        assert_eq!(Severity::parse("13"), Some(Severity::WARNING));
        assert_eq!(Severity::parse("0"), None);
        assert_eq!(Severity::parse("25"), None);
        assert_eq!(Severity::parse(""), None);
        assert_eq!(Severity::parse("loud"), None);
    }

    #[test]
    fn test_severity_display_label_or_otel_name() {
        assert_eq!(Severity::new(10).unwrap().to_string(), "NOTICE");
        assert_eq!(Severity::new(11).unwrap().to_string(), "INFO3");
        for value in 1..=24 {
            let s = Severity::new(value).unwrap();
            assert_eq!(Severity::parse(&s.to_string()), Some(s));
            assert_eq!(Severity::parse(s.otel_name()), Some(s));
        }
    }

    #[test]
    fn test_severity_bands() {
        assert_eq!(Severity::TRACE.band(), Band::Trace);
        assert_eq!(Severity::new(8).unwrap().band(), Band::Debug);
        assert_eq!(Severity::new(16).unwrap().band(), Band::Warn);
        assert_eq!(Severity::new(24).unwrap().band(), Band::Fatal);
    }

    #[test]
    fn test_option_severity_is_one_byte() {
        assert_eq!(core::mem::size_of::<Option<Severity>>(), 1);
    }
}
