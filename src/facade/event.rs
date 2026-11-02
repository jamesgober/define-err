//! `Event` and `Verb`: what a reporter receives.

use core::{error::Error, fmt};

#[cfg(feature = "alloc")]
use crate::ErrorRecord;
use crate::{Category, Code, Kinds, Location, Severity};

/// Which verb raised an event.
///
/// # Examples
///
/// ```
/// use define_err::Verb;
///
/// assert_ne!(Verb::Warn, Verb::Notice);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Verb {
    /// `report`, `report_with`, `report_record`.
    Report,
    /// `notice`, `notice_with`, `notice!`.
    Notice,
    /// `warn`, `warn_with`, `warn!`.
    Warn,
    /// `trigger`, `trigger!`.
    Trigger,
    /// `deprecated`, `deprecated!`.
    Deprecated,
    /// `soft`.
    Soft,
}

impl Verb {
    /// The verb's name: `"report"`, `"notice"`, and so on. The type name of
    /// the synthetic chain entry a record gets from a message event.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Verb::Notice.name(), "notice");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Verb::Report => "report",
            Verb::Notice => "notice",
            Verb::Warn => "warn",
            Verb::Trigger => "trigger",
            Verb::Deprecated => "deprecated",
            Verb::Soft => "soft",
        }
    }
}

/// What a reporter receives.
///
/// Everything is borrowed and lives on the caller's stack for the length of
/// [`Reporter::receive`](crate::Reporter::receive); a reporter that keeps
/// anything copies it or builds an `ErrorRecord`. `error()` is set for
/// `report`, `report_with`, and `soft`; `message()` for `notice`, `warn`,
/// `trigger`, and `deprecated`; `record()` for `report_record`.
///
/// The `new` and `with_*` methods let a reporter's tests and the error
/// service build events.
///
/// # Examples
///
/// ```
/// use define_err::{Event, Kinds, Verb};
///
/// let event = Event::new(Verb::Notice, define_err::here())
///     .with_kinds(Kinds::NOTICE)
///     .with_message(format_args!("test notice"));
/// assert_eq!(event.verb(), Verb::Notice);
/// assert_eq!(event.message().unwrap().to_string(), "test notice");
/// ```
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Event<'a> {
    pub(super) verb: Verb,
    pub(super) kinds: Kinds,
    pub(super) severity: Severity,
    pub(super) code: Option<Code>,
    pub(super) category: Category,
    pub(super) error: Option<&'a (dyn Error + 'static)>,
    pub(super) message: Option<fmt::Arguments<'a>>,
    pub(super) location: Location,
    pub(super) type_name: Option<&'a str>,
    #[cfg(feature = "alloc")]
    pub(super) record: Option<&'a ErrorRecord>,
}

impl<'a> Event<'a> {
    /// An event with kinds `NONE`, severity `ERROR`, no code, and category
    /// `Unknown`.
    #[must_use]
    pub fn new(verb: Verb, location: Location) -> Event<'a> {
        Event {
            verb,
            kinds: Kinds::NONE,
            severity: Severity::ERROR,
            code: None,
            category: Category::Unknown,
            error: None,
            message: None,
            location,
            type_name: None,
            #[cfg(feature = "alloc")]
            record: None,
        }
    }

    /// Which verb raised the event.
    #[must_use]
    pub fn verb(&self) -> Verb {
        self.verb
    }

    /// The event's kinds.
    #[must_use]
    pub fn kinds(&self) -> Kinds {
        self.kinds
    }

    /// The event's severity.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The error's code, if it has one.
    #[must_use]
    pub fn code(&self) -> Option<Code> {
        self.code
    }

    /// The error's category; `Unknown` for an event with no error.
    #[must_use]
    pub fn category(&self) -> Category {
        self.category
    }

    /// The error, for `report`, `report_with`, and `soft`.
    #[must_use]
    pub fn error(&self) -> Option<&'a (dyn Error + 'static)> {
        self.error
    }

    /// The message, for `notice`, `warn`, `trigger`, and `deprecated`.
    #[must_use]
    pub fn message(&self) -> Option<fmt::Arguments<'a>> {
        self.message
    }

    /// Where the verb was called.
    #[must_use]
    pub fn location(&self) -> Location {
        self.location
    }

    /// `"Type::Variant"`, from the error's probe or the record.
    #[must_use]
    pub fn type_name(&self) -> Option<&'a str> {
        self.type_name
    }

    /// The record, for `report_record`.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    pub fn record(&self) -> Option<&'a ErrorRecord> {
        self.record
    }

    /// Sets the kinds.
    #[must_use]
    pub fn with_kinds(mut self, kinds: Kinds) -> Event<'a> {
        self.kinds = kinds;
        self
    }

    /// Sets the severity.
    #[must_use]
    pub fn with_severity(mut self, severity: Severity) -> Event<'a> {
        self.severity = severity;
        self
    }

    /// Sets the code.
    #[must_use]
    pub fn with_code(mut self, code: Option<Code>) -> Event<'a> {
        self.code = code;
        self
    }

    /// Sets the category.
    #[must_use]
    pub fn with_category(mut self, category: Category) -> Event<'a> {
        self.category = category;
        self
    }

    /// Sets the error.
    #[must_use]
    pub fn with_error(mut self, error: &'a (dyn Error + 'static)) -> Event<'a> {
        self.error = Some(error);
        self
    }

    /// Sets the message.
    #[must_use]
    pub fn with_message(mut self, message: fmt::Arguments<'a>) -> Event<'a> {
        self.message = Some(message);
        self
    }

    /// Sets the record.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[must_use]
    pub fn with_record(mut self, record: &'a ErrorRecord) -> Event<'a> {
        self.record = Some(record);
        self
    }
}

/// Writes the plain facts. The error and the message are left out, because
/// printing them would run a user's `Display`.
impl fmt::Debug for Event<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("verb", &self.verb)
            .field("kinds", &self.kinds)
            .field("severity", &self.severity)
            .field("code", &self.code)
            .field("category", &self.category)
            .field("location", &self.location)
            .field("type_name", &self.type_name)
            .finish_non_exhaustive()
    }
}
