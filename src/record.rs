//! `ErrorRecord`: an owned, bounded snapshot of an error.

use alloc::{borrow::Cow, string::String, vec::Vec};
use core::{
    error::Error,
    fmt::{self, Write as _},
};

use crate::{
    facade::{Event, Verb},
    probe, Category, Code, Kinds, Location, Severity,
};

/// What a chain entry says when the error's `Display` fails.
const UNAVAILABLE: &str = "<message unavailable>";
/// A chain entry's type name when the error has no probe.
const UNKNOWN_TYPE: &str = "unknown";
/// The longest type name kept, the same bound the probe has.
const MAX_TYPE_NAME: usize = 128;

/// The bounds of a record, safe for servers by default. Change a copy. A
/// limit of 0 keeps nothing of that part, and `max_total_bytes` bounds all
/// message, field, and backtrace text together.
///
/// # Examples
///
/// ```
/// use define_err::RecordLimits;
///
/// let mut limits = RecordLimits::DEFAULT;
/// limits.max_chain = 8;
/// assert_eq!(limits.max_message_bytes, 1024);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct RecordLimits {
    /// Chain entries, 32 by default.
    pub max_chain: usize,
    /// Bytes of one chain message, 1,024 by default.
    pub max_message_bytes: usize,
    /// Fields, 32 by default.
    pub max_fields: usize,
    /// Bytes of one field key or value, 256 by default.
    pub max_field_bytes: usize,
    /// Trail entries, 64 by default.
    pub max_trail: usize,
    /// Bytes of all text together, 16,384 by default.
    pub max_total_bytes: usize,
}

impl RecordLimits {
    /// The default limits.
    pub const DEFAULT: RecordLimits = RecordLimits {
        max_chain: 32,
        max_message_bytes: 1_024,
        max_fields: 32,
        max_field_bytes: 256,
        max_trail: 64,
        max_total_bytes: 16_384,
    };

    /// The default limits.
    #[must_use]
    pub const fn new() -> RecordLimits {
        RecordLimits::DEFAULT
    }
}

impl Default for RecordLimits {
    fn default() -> RecordLimits {
        RecordLimits::DEFAULT
    }
}

/// One error of a record's chain.
///
/// # Examples
///
/// ```
/// use define_err::{Category, ErrorRecord, Kinds, Severity};
///
/// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
/// record.push_chain(&"line one\nline two", None, "context");
/// assert!(record.chain[0].has_control);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ChainEntry {
    /// The private message, stored verbatim: escaping is the sink's job.
    pub message: String,
    /// The entry's code.
    pub code: Option<Code>,
    /// `"Type::Variant"`, or `"unknown"` for an error without a probe.
    pub type_name: Cow<'static, str>,
    /// The message contains a control character (a newline, an escape), so a
    /// log sink can escape it without scanning again.
    pub has_control: bool,
}

/// What happened at one location of a record's trail.
///
/// # Examples
///
/// ```
/// use define_err::{TrailEntry, TrailStep};
///
/// let entry = TrailEntry::new(define_err::here(), TrailStep::Created);
/// assert_eq!(entry.step, TrailStep::Created);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum TrailStep {
    /// The error was created.
    Created,
    /// The error was converted.
    Converted,
    /// Context was added.
    Context,
    /// The error was reported.
    Reported,
}

/// One location of a record's trail. Holds an owned or borrowed file name, so
/// a record received from another node needs no `'static` data.
///
/// # Examples
///
/// ```
/// use define_err::{TrailEntry, TrailStep};
///
/// let entry = TrailEntry::new(define_err::here(), TrailStep::Reported);
/// assert_eq!(entry.file, file!());
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TrailEntry {
    /// The file.
    pub file: Cow<'static, str>,
    /// The line.
    pub line: u32,
    /// The column.
    pub column: u32,
    /// What happened there.
    pub step: TrailStep,
}

impl TrailEntry {
    /// An entry at `location`. Borrows the file name; no allocation.
    #[must_use]
    pub fn new(location: Location, step: TrailStep) -> TrailEntry {
        TrailEntry {
            file: Cow::Borrowed(location.file()),
            line: location.line(),
            column: location.column(),
            step,
        }
    }

    /// The file as a log should show it: [`short_file`](crate::short_file)
    /// of `file`. The record keeps the whole path.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{TrailEntry, TrailStep};
    ///
    /// let entry = TrailEntry::new(define_err::here(), TrailStep::Created);
    /// assert!(!entry.short_file().starts_with('/'));
    /// ```
    #[must_use]
    pub fn short_file(&self) -> &str {
        crate::short_file(&self.file)
    }
}

/// An owned, `Send + Sync + Clone` snapshot of an error, for anything that
/// outlives the borrow: queued listeners, a crash ring, JSON output, messages
/// between processes. Every part is bounded by [`RecordLimits`], checked
/// before the allocation it guards.
///
/// # Examples
///
/// ```
/// use define_err::{define_errors, ErrorRecord};
///
/// define_errors! {
///     namespace = "DB";
///     pub enum DbError {
///         #[code(1, help = "Raise pool.size.")]
///         #[category(DeadlineExceeded)]
///         Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",
///     }
/// }
///
/// let record = ErrorRecord::from_error(&DbError::Timeout { after_ms: 5000 });
/// assert_eq!(record.to_string(), "[DB-0001] query timed out after 5000 ms");
/// assert_eq!(record.chain.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ErrorRecord {
    /// From the outermost chain entry that has a code, else the outermost.
    pub code: Option<Code>,
    /// As `code`.
    pub severity: Severity,
    /// As `code`.
    pub category: Category,
    /// As `code`.
    pub kinds: Kinds,
    /// That entry's `#[public]` message, or its category's default; never any
    /// `Display` text.
    pub public: Cow<'static, str>,
    /// One entry per error, outermost first.
    pub chain: Vec<ChainEntry>,
    /// Keys and values added by the creator or a hook.
    pub fields: Vec<(Cow<'static, str>, String)>,
    /// Where the error was created, converted, given context, reported.
    pub trail: Vec<TrailEntry>,
    /// Text, filled by the dynamic-error or service library.
    pub backtrace: Option<String>,
    /// Filled by the error service.
    pub event_id: Option<u128>,
    /// Filled by the error service (there is no clock in `no_std`).
    pub time_unix_nanos: Option<u64>,
    /// Whether the error was handled. `true` unless a caller says otherwise.
    pub handled: bool,
    /// A limit was hit somewhere.
    pub truncated: bool,
    limits: RecordLimits,
    used: usize,
}

/// A `fmt::Write` that stops at a byte limit, on a character boundary. It
/// allocates only what it keeps, and after the limit it refuses everything,
/// so a `Display` that ignores errors cannot grow it.
struct Bounded {
    text: String,
    limit: usize,
    cut: bool,
}

impl fmt::Write for Bounded {
    fn write_str(&mut self, part: &str) -> fmt::Result {
        if self.cut {
            return Err(fmt::Error);
        }
        let room = self.limit - self.text.len();
        if part.len() <= room {
            self.text.push_str(part);
            return Ok(());
        }
        self.text.push_str(&part[..floor_boundary(part, room)]);
        self.cut = true;
        Err(fmt::Error)
    }
}

/// The largest character boundary of `text` at or below `index`.
fn floor_boundary(text: &str, index: usize) -> usize {
    if index >= text.len() {
        return text.len();
    }
    let mut at = index;
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Writes `value` within `limit` bytes; returns the text and whether it was
/// cut. A `Display` that fails by itself gives the fixed marker.
fn bounded(value: &dyn fmt::Display, limit: usize) -> (String, bool) {
    let mut out = Bounded {
        text: String::new(),
        limit,
        cut: false,
    };
    let failed = write!(out, "{value}").is_err();
    if failed && !out.cut {
        let keep = floor_boundary(UNAVAILABLE, limit);
        return (String::from(&UNAVAILABLE[..keep]), keep < UNAVAILABLE.len());
    }
    (out.text, out.cut)
}

impl ErrorRecord {
    /// A snapshot of `error` and its `source()` chain, with the default
    /// limits. See [`from_error_with`](ErrorRecord::from_error_with).
    #[must_use]
    pub fn from_error(error: &(dyn Error + 'static)) -> ErrorRecord {
        ErrorRecord::from_error_with(error, &RecordLimits::DEFAULT)
    }

    /// A snapshot of `error` and its `source()` chain.
    ///
    /// Walks at most `max_chain` entries, so a chain that returns to itself
    /// stops at the limit. Each `Display` goes through a bounded writer that
    /// stops at `max_message_bytes`; a `Display` that fails is stored as
    /// `"<message unavailable>"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{define_errors, ErrorRecord, RecordLimits};
    ///
    /// define_errors! {
    ///     pub struct Timeout => "query timed out";
    /// }
    ///
    /// let mut limits = RecordLimits::DEFAULT;
    /// limits.max_message_bytes = 5;
    /// let record = ErrorRecord::from_error_with(&Timeout, &limits);
    /// assert_eq!(record.chain[0].message, "query");
    /// assert!(record.truncated);
    /// ```
    #[must_use]
    pub fn from_error_with(error: &(dyn Error + 'static), limits: &RecordLimits) -> ErrorRecord {
        let outermost = probe::facts(error);
        let mut record = ErrorRecord::from_facts(&outermost, limits);
        let mut found_code = outermost.code.is_some();
        // Steps, not kept entries: a limit that leaves entries out must still
        // end the walk of a chain that returns to itself.
        let mut next = Some(error);
        for _ in 0..limits.max_chain {
            let Some(current) = next else {
                return record;
            };
            let facts = probe::facts(current);
            if !found_code && facts.code.is_some() {
                found_code = true;
                record.take_header(&facts);
            }
            let _ = record.push_chain(
                &current,
                facts.code,
                facts.type_name.unwrap_or(UNKNOWN_TYPE),
            );
            next = current.source();
        }
        record.truncated |= next.is_some();
        record
    }

    /// A snapshot of what a reporter received, with the default limits. See
    /// [`from_event_with`](ErrorRecord::from_event_with).
    #[must_use]
    pub fn from_event(event: &Event<'_>) -> ErrorRecord {
        ErrorRecord::from_event_with(event, &RecordLimits::DEFAULT)
    }

    /// A snapshot of what a reporter received.
    ///
    /// Adds the event's location to the trail. An event with no error (a
    /// notice) becomes one synthetic chain entry whose type name is the verb.
    /// An event that carries a record returns a clone of it, under its own
    /// limits, with the location added.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{ErrorRecord, Event, Kinds, Severity, Verb};
    ///
    /// let event = Event::new(Verb::Notice, define_err::here())
    ///     .with_kinds(Kinds::NOTICE)
    ///     .with_severity(Severity::NOTICE)
    ///     .with_message(format_args!("cache rebuilt"));
    /// let record = ErrorRecord::from_event(&event);
    /// assert_eq!(record.chain[0].message, "cache rebuilt");
    /// assert_eq!(record.chain[0].type_name, "notice");
    /// ```
    #[must_use]
    pub fn from_event_with(event: &Event<'_>, limits: &RecordLimits) -> ErrorRecord {
        let mut record = if let Some(record) = event.record() {
            record.clone()
        } else if let Some(error) = event.error() {
            let mut record = ErrorRecord::from_error_with(error, limits);
            record.severity = event.severity();
            record.kinds = event.kinds();
            record
        } else {
            let mut record = ErrorRecord::new(
                event.code(),
                event.severity(),
                event.category(),
                event.kinds(),
            );
            record.limits = *limits;
            if let Some(message) = event.message() {
                let _ = record.push_chain(&message, event.code(), Verb::name(event.verb()));
            }
            record
        };
        let _ = record.push_trail(TrailEntry::new(event.location(), TrailStep::Reported));
        record
    }

    /// An empty record, for a caller that assembles the chain itself (a
    /// dynamic error type that renders through its own guarded writer).
    /// `public` is the category's default message.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Unavailable, Kinds::ERROR);
    /// record.push_chain(&"while syncing tenant 42", None, "context");
    /// record.push_field("tenant", &42);
    /// assert!(!record.truncated);
    /// ```
    #[must_use]
    pub fn new(
        code: Option<Code>,
        severity: Severity,
        category: Category,
        kinds: Kinds,
    ) -> ErrorRecord {
        ErrorRecord {
            code,
            severity,
            category,
            kinds,
            public: Cow::Borrowed(category.default_public_message()),
            chain: Vec::new(),
            fields: Vec::new(),
            trail: Vec::new(),
            backtrace: None,
            event_id: None,
            time_unix_nanos: None,
            handled: true,
            truncated: false,
            limits: RecordLimits::DEFAULT,
            used: 0,
        }
    }

    /// Replaces the limits. Call it before anything is added: later additions
    /// stay inside these bounds, and what is already there stays.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, RecordLimits, Severity};
    ///
    /// let mut limits = RecordLimits::DEFAULT;
    /// limits.max_fields = 0;
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR)
    ///     .with_limits(limits);
    /// assert!(!record.push_field("tenant", &42));
    /// assert!(record.truncated);
    /// ```
    #[must_use]
    pub fn with_limits(mut self, limits: RecordLimits) -> ErrorRecord {
        self.limits = limits;
        self
    }

    fn from_facts(facts: &probe::Facts<'_>, limits: &RecordLimits) -> ErrorRecord {
        let mut record = ErrorRecord::new(facts.code, facts.severity, facts.category, facts.kinds);
        record.limits = *limits;
        record.take_public(facts);
        record
    }

    fn take_header(&mut self, facts: &probe::Facts<'_>) {
        self.code = facts.code;
        self.severity = facts.severity;
        self.category = facts.category;
        self.kinds = facts.kinds;
        self.public = Cow::Borrowed(facts.category.default_public_message());
        self.take_public(facts);
    }

    fn take_public(&mut self, facts: &probe::Facts<'_>) {
        if let Some(text) = facts.public {
            let (kept, cut) = bounded(&text, self.limits.max_message_bytes.min(self.room()));
            self.public = Cow::Owned(kept);
            self.truncated |= cut;
        }
    }

    /// The bytes left of `max_total_bytes`.
    fn room(&self) -> usize {
        self.limits.max_total_bytes.saturating_sub(self.used)
    }

    /// Appends a chain entry, after the existing ones (outermost first, so a
    /// builder pushes from the outside in). Returns `false` when a limit cut
    /// it or left it out, and sets `truncated`. Never fails otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    /// assert!(record.push_chain(&"while saving", None, "context"));
    /// assert_eq!(record.to_string(), "while saving");
    /// ```
    pub fn push_chain(
        &mut self,
        message: &dyn fmt::Display,
        code: Option<Code>,
        type_name: &str,
    ) -> bool {
        let limit = self.limits.max_message_bytes.min(self.room());
        if self.chain.len() >= self.limits.max_chain || limit == 0 {
            self.truncated = true;
            return false;
        }
        let (text, cut) = bounded(message, limit);
        let name = &type_name[..floor_boundary(type_name, MAX_TYPE_NAME)];
        self.used += text.len();
        self.chain.push(ChainEntry {
            has_control: text.chars().any(char::is_control),
            message: text,
            code,
            type_name: Cow::Owned(String::from(name)),
        });
        self.truncated |= cut;
        !cut
    }

    /// Adds a key and a value. Returns `false` when a limit cut it or left it
    /// out, and sets `truncated`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    /// assert!(record.push_field("tenant", &42));
    /// assert_eq!(record.fields[0].1, "42");
    /// ```
    pub fn push_field(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        value: &dyn fmt::Display,
    ) -> bool {
        let limit = self.limits.max_field_bytes.min(self.room());
        if self.fields.len() >= self.limits.max_fields || limit == 0 {
            self.truncated = true;
            return false;
        }
        let mut key = key.into();
        let mut cut = false;
        let keep = floor_boundary(&key, limit);
        if keep < key.len() {
            key.to_mut().truncate(keep);
            cut = true;
        }
        let limit = limit.min(self.room() - key.len());
        let (text, value_cut) = bounded(value, limit);
        self.used += key.len() + text.len();
        self.fields.push((key, text));
        let cut = cut || value_cut;
        self.truncated |= cut;
        !cut
    }

    /// Adds a location. Returns `false` when a limit left it out, and sets
    /// `truncated`. An owned file name counts against `max_total_bytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity, TrailEntry, TrailStep};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    /// assert!(record.push_trail(TrailEntry::new(define_err::here(), TrailStep::Created)));
    /// ```
    pub fn push_trail(&mut self, entry: TrailEntry) -> bool {
        let owned = match entry.file {
            Cow::Owned(ref file) => file.len(),
            Cow::Borrowed(_) => 0,
        };
        if self.trail.len() >= self.limits.max_trail || owned > self.room() {
            self.truncated = true;
            return false;
        }
        self.used += owned;
        self.trail.push(entry);
        true
    }

    /// Stores backtrace text, replacing any before it. Returns `false` when a
    /// limit cut it, and sets `truncated`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    /// assert!(record.set_backtrace(&"0: main"));
    /// ```
    pub fn set_backtrace(&mut self, text: &dyn fmt::Display) -> bool {
        if let Some(old) = self.backtrace.take() {
            self.used = self.used.saturating_sub(old.len());
        }
        let (kept, cut) = bounded(text, self.room());
        self.used += kept.len();
        self.backtrace = Some(kept);
        self.truncated |= cut;
        !cut
    }

    /// Sets the event id.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    /// record.set_event_id(7);
    /// assert_eq!(record.event_id, Some(7));
    /// ```
    pub fn set_event_id(&mut self, id: u128) {
        self.event_id = Some(id);
    }

    /// Sets the time, in nanoseconds since the Unix epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    /// record.set_time(1);
    /// assert_eq!(record.time_unix_nanos, Some(1));
    /// ```
    pub fn set_time(&mut self, unix_nanos: u64) {
        self.time_unix_nanos = Some(unix_nanos);
    }

    /// Sets whether the error was handled.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, ErrorRecord, Kinds, Severity};
    ///
    /// let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    /// record.set_handled(false);
    /// assert!(!record.handled);
    /// ```
    pub fn set_handled(&mut self, handled: bool) {
        self.handled = handled;
    }
}

/// Writes one line: `[code] outer message: caused by inner message`. Control
/// characters become spaces; every other format belongs to the render library.
impl fmt::Display for ErrorRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code {
            write!(f, "[{code}]")?;
            if !self.chain.is_empty() {
                f.write_char(' ')?;
            }
        }
        for (i, entry) in self.chain.iter().enumerate() {
            if i > 0 {
                f.write_str(": caused by ")?;
            }
            if entry.has_control {
                for c in entry.message.chars() {
                    f.write_char(if c.is_control() { ' ' } else { c })?;
                }
            } else {
                f.write_str(&entry.message)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{bounded, floor_boundary};

    #[test]
    fn test_bounded_cuts_on_a_character_boundary() {
        assert_eq!(bounded(&"abc", 3), ("abc".into(), false));
        assert_eq!(bounded(&"abcd", 3), ("abc".into(), true));
        // `é` is two bytes and straddles the limit.
        assert_eq!(bounded(&"abé", 3), ("ab".into(), true));
        assert_eq!(bounded(&"abc", 0), ("".into(), true));
    }

    #[test]
    fn test_floor_boundary() {
        assert_eq!(floor_boundary("abc", 10), 3);
        assert_eq!(floor_boundary("é", 1), 0);
    }
}
