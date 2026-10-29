//! `Category` (the gRPC canonical codes without `OK`), `RetryAdvice`, and
//! `Action`, with every mapping table. The tables are in
//! `tests/golden/categories.txt` and frozen at 1.0.

use core::fmt;

use crate::Severity;

/// What kind of failure an error is, in the gRPC canonical set (without `OK`).
///
/// A category decides how every transport refuses: an HTTP status, a gRPC
/// code, a process exit code, whether a retry may help, and what a client may
/// be told, without a table per error code.
///
/// # Examples
///
/// ```
/// use define_err::Category;
///
/// let c = Category::Unavailable;
/// assert_eq!(c.http_status(), 503);
/// assert_eq!(c.exit_code(), 69);
/// assert!(c.may_retry_automatically(true));
/// assert!(!c.may_retry_automatically(false));
/// assert_eq!(Category::from_grpc(4), Some(Category::DeadlineExceeded));
/// assert_eq!(Category::parse("DEADLINE_EXCEEDED"), Some(Category::DeadlineExceeded));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Category {
    /// The operation was cancelled, usually by the caller.
    Cancelled,
    /// An error with no better category; the default.
    Unknown,
    /// The caller sent something invalid, whatever the system's state.
    InvalidArgument,
    /// The deadline passed before the operation finished.
    DeadlineExceeded,
    /// A requested item does not exist.
    NotFound,
    /// The item the caller tried to create already exists.
    AlreadyExists,
    /// The caller is not allowed to do this.
    PermissionDenied,
    /// A quota or a resource ran out (rate limits, disk, memory).
    ResourceExhausted,
    /// The system is not in a state that allows the operation.
    FailedPrecondition,
    /// The operation conflicted with another (a transaction abort).
    Aborted,
    /// A value is outside the valid range.
    OutOfRange,
    /// The operation is not implemented or not supported.
    Unimplemented,
    /// An invariant the system relies on was broken.
    Internal,
    /// The service cannot be reached now; it may succeed later.
    Unavailable,
    /// Data was lost or corrupted beyond repair.
    DataLoss,
    /// The caller is not authenticated.
    Unauthenticated,
}

impl Category {
    /// Every category, in gRPC order.
    pub const ALL: [Category; 16] = [
        Category::Cancelled,
        Category::Unknown,
        Category::InvalidArgument,
        Category::DeadlineExceeded,
        Category::NotFound,
        Category::AlreadyExists,
        Category::PermissionDenied,
        Category::ResourceExhausted,
        Category::FailedPrecondition,
        Category::Aborted,
        Category::OutOfRange,
        Category::Unimplemented,
        Category::Internal,
        Category::Unavailable,
        Category::DataLoss,
        Category::Unauthenticated,
    ];

    /// The HTTP status, from Google's `google.rpc.Code` mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Category::FailedPrecondition.http_status(), 400);
    /// ```
    #[must_use]
    pub const fn http_status(self) -> u16 {
        match self {
            Category::Cancelled => 499,
            Category::Unknown | Category::Internal | Category::DataLoss => 500,
            Category::InvalidArgument | Category::FailedPrecondition | Category::OutOfRange => 400,
            Category::DeadlineExceeded => 504,
            Category::NotFound => 404,
            Category::AlreadyExists | Category::Aborted => 409,
            Category::PermissionDenied => 403,
            Category::ResourceExhausted => 429,
            Category::Unimplemented => 501,
            Category::Unavailable => 503,
            Category::Unauthenticated => 401,
        }
    }

    /// The gRPC status code, 1 to 16.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Category::Unauthenticated.grpc_code(), 16);
    /// ```
    #[must_use]
    pub const fn grpc_code(self) -> u8 {
        match self {
            Category::Cancelled => 1,
            Category::Unknown => 2,
            Category::InvalidArgument => 3,
            Category::DeadlineExceeded => 4,
            Category::NotFound => 5,
            Category::AlreadyExists => 6,
            Category::PermissionDenied => 7,
            Category::ResourceExhausted => 8,
            Category::FailedPrecondition => 9,
            Category::Aborted => 10,
            Category::OutOfRange => 11,
            Category::Unimplemented => 12,
            Category::Internal => 13,
            Category::Unavailable => 14,
            Category::DataLoss => 15,
            Category::Unauthenticated => 16,
        }
    }

    /// The category for a gRPC code; `None` for 0 (`OK`) and above 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Category;
    ///
    /// assert_eq!(Category::from_grpc(16), Some(Category::Unauthenticated));
    /// assert_eq!(Category::from_grpc(0), None);
    /// assert_eq!(Category::from_grpc(17), None);
    /// ```
    #[must_use]
    pub const fn from_grpc(code: u8) -> Option<Category> {
        if code >= 1 && code <= 16 {
            Some(Category::ALL[(code - 1) as usize])
        } else {
            None
        }
    }

    /// The process exit code, from `sysexits(3)`; `Cancelled` is 130 (the
    /// shell's 128 + SIGINT).
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Category::InvalidArgument.exit_code(), 64);
    /// ```
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Category::Cancelled => 130,
            Category::Unknown | Category::Unimplemented | Category::Internal => 70,
            Category::InvalidArgument => 64,
            Category::DeadlineExceeded | Category::ResourceExhausted | Category::Aborted => 75,
            Category::NotFound => 66,
            Category::AlreadyExists => 73,
            Category::PermissionDenied | Category::Unauthenticated => 77,
            Category::FailedPrecondition => 78,
            Category::OutOfRange => 65,
            Category::Unavailable => 69,
            Category::DataLoss => 74,
        }
    }

    /// Whether the condition may clear on its own (`DeadlineExceeded`,
    /// `ResourceExhausted`, `Aborted`, `Unavailable`).
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Category;
    ///
    /// assert!(Category::Unavailable.is_transient());
    /// assert!(!Category::NotFound.is_transient());
    /// ```
    #[must_use]
    pub const fn is_transient(self) -> bool {
        matches!(
            self,
            Category::DeadlineExceeded
                | Category::ResourceExhausted
                | Category::Aborted
                | Category::Unavailable
        )
    }

    /// Google AIP-194's rule for automatic retries.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, RetryAdvice};
    ///
    /// assert_eq!(Category::Aborted.retry_advice(), RetryAdvice::RestartTransaction);
    /// ```
    #[must_use]
    pub const fn retry_advice(self) -> RetryAdvice {
        match self {
            Category::Unavailable => RetryAdvice::Yes,
            Category::ResourceExhausted => RetryAdvice::Later,
            Category::Aborted => RetryAdvice::RestartTransaction,
            Category::FailedPrecondition => RetryAdvice::AfterFix,
            _ => RetryAdvice::No,
        }
    }

    /// Whether an operation may be retried without a person deciding to:
    /// only an idempotent operation (one whose repeat causes no unintended
    /// change) failing with `Unavailable`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Category;
    ///
    /// assert!(Category::Unavailable.may_retry_automatically(true));
    /// assert!(!Category::DeadlineExceeded.may_retry_automatically(true));
    /// ```
    #[must_use]
    pub const fn may_retry_automatically(self, idempotent: bool) -> bool {
        idempotent && matches!(self, Category::Unavailable)
    }

    /// The severity a variant gets when it does not say.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Category, Severity};
    ///
    /// assert_eq!(Category::DataLoss.default_severity(), Severity::CRITICAL);
    /// ```
    #[must_use]
    pub const fn default_severity(self) -> Severity {
        match self {
            Category::Cancelled => Severity::DEBUG,
            Category::InvalidArgument
            | Category::NotFound
            | Category::AlreadyExists
            | Category::PermissionDenied
            | Category::FailedPrecondition
            | Category::Aborted
            | Category::OutOfRange
            | Category::Unauthenticated => Severity::WARNING,
            Category::DataLoss => Severity::CRITICAL,
            _ => Severity::ERROR,
        }
    }

    /// The action a variant gets when it does not say.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::{Action, Category};
    ///
    /// assert_eq!(Category::DataLoss.default_action(), Action::PageOnCall);
    /// ```
    #[must_use]
    pub const fn default_action(self) -> Action {
        match self {
            Category::Cancelled => Action::None,
            Category::Unknown | Category::Unimplemented | Category::Internal => Action::Operator,
            Category::DeadlineExceeded
            | Category::ResourceExhausted
            | Category::Aborted
            | Category::Unavailable => Action::Retry,
            Category::DataLoss => Action::PageOnCall,
            _ => Action::FixRequest,
        }
    }

    /// The message a client may see when a variant does not write its own.
    ///
    /// `DataLoss` has the same text as `Internal` on purpose: a client is
    /// never told through a generic path that data was lost.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Category;
    ///
    /// assert_eq!(Category::NotFound.default_public_message(), "The requested item was not found.");
    /// assert_eq!(Category::DataLoss.default_public_message(), Category::Internal.default_public_message());
    /// ```
    #[must_use]
    pub const fn default_public_message(self) -> &'static str {
        match self {
            Category::Cancelled => "The request was cancelled.",
            Category::Unknown => "An unexpected error occurred.",
            Category::InvalidArgument => "The request is not valid.",
            Category::DeadlineExceeded => "The request took too long. Try again.",
            Category::NotFound => "The requested item was not found.",
            Category::AlreadyExists => "The item already exists.",
            Category::PermissionDenied => "You do not have permission to do this.",
            Category::ResourceExhausted => "Too many requests. Try again later.",
            Category::FailedPrecondition => "The request cannot be done in the current state.",
            Category::Aborted => "The request conflicted with another. Try again.",
            Category::OutOfRange => "A value is out of range.",
            Category::Unimplemented => "This operation is not supported.",
            Category::Internal | Category::DataLoss => "An internal error occurred.",
            Category::Unavailable => "The service is unavailable. Try again later.",
            Category::Unauthenticated => "Authentication is required.",
        }
    }

    /// The `snake_case` name: `"deadline_exceeded"`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(define_err::Category::DeadlineExceeded.name(), "deadline_exceeded");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Category::Cancelled => "cancelled",
            Category::Unknown => "unknown",
            Category::InvalidArgument => "invalid_argument",
            Category::DeadlineExceeded => "deadline_exceeded",
            Category::NotFound => "not_found",
            Category::AlreadyExists => "already_exists",
            Category::PermissionDenied => "permission_denied",
            Category::ResourceExhausted => "resource_exhausted",
            Category::FailedPrecondition => "failed_precondition",
            Category::Aborted => "aborted",
            Category::OutOfRange => "out_of_range",
            Category::Unimplemented => "unimplemented",
            Category::Internal => "internal",
            Category::Unavailable => "unavailable",
            Category::DataLoss => "data_loss",
            Category::Unauthenticated => "unauthenticated",
        }
    }

    /// Parses `snake_case`, `SCREAMING_CASE`, or `CamelCase` names.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Category;
    ///
    /// assert_eq!(Category::parse("not_found"), Some(Category::NotFound));
    /// assert_eq!(Category::parse("NOT_FOUND"), Some(Category::NotFound));
    /// assert_eq!(Category::parse("NotFound"), Some(Category::NotFound));
    /// assert_eq!(Category::parse("ok"), None);
    /// ```
    #[must_use]
    pub fn parse(text: &str) -> Option<Category> {
        if text.is_empty() || text.len() > 32 {
            return None;
        }
        let same = |name: &str| {
            let mut a = name.bytes().filter(|&b| b != b'_');
            let mut b = text.bytes().filter(|&b| b != b'_');
            loop {
                match (a.next(), b.next()) {
                    (None, None) => return true,
                    (Some(x), Some(y)) if x.eq_ignore_ascii_case(&y) => {}
                    _ => return false,
                }
            }
        };
        Category::ALL.into_iter().find(|category| same(category.name()))
    }

    /// The category of a `std::io::ErrorKind`. Kinds without a clear match,
    /// and kinds added to the standard library later, are `Unknown`.
    ///
    /// # Examples
    ///
    /// ```
    /// use define_err::Category;
    ///
    /// assert_eq!(Category::from_io_kind(std::io::ErrorKind::NotFound), Category::NotFound);
    /// assert_eq!(Category::from_io_kind(std::io::ErrorKind::TimedOut), Category::DeadlineExceeded);
    /// ```
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[must_use]
    pub fn from_io_kind(kind: std::io::ErrorKind) -> Category {
        use std::io::ErrorKind;
        match kind {
            ErrorKind::NotFound => Category::NotFound,
            ErrorKind::PermissionDenied => Category::PermissionDenied,
            ErrorKind::AlreadyExists => Category::AlreadyExists,
            ErrorKind::InvalidInput => Category::InvalidArgument,
            ErrorKind::InvalidData => Category::InvalidArgument,
            ErrorKind::UnexpectedEof => Category::OutOfRange,
            ErrorKind::TimedOut => Category::DeadlineExceeded,
            ErrorKind::WouldBlock | ErrorKind::Interrupted => Category::Unavailable,
            ErrorKind::OutOfMemory => Category::ResourceExhausted,
            ErrorKind::Unsupported => Category::Unimplemented,
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::BrokenPipe
            | ErrorKind::AddrNotAvailable => Category::Unavailable,
            ErrorKind::AddrInUse => Category::AlreadyExists,
            ErrorKind::WriteZero => Category::Unavailable,
            _ => Category::Unknown,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}

/// Google AIP-194's rule for retrying an operation automatically.
///
/// # Examples
///
/// ```
/// use define_err::{Category, RetryAdvice};
///
/// assert_eq!(Category::ResourceExhausted.retry_advice(), RetryAdvice::Later);
/// assert_eq!(RetryAdvice::Later.to_string(), "later");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum RetryAdvice {
    /// Do not retry this operation automatically.
    No,
    /// Retry with backoff, if the operation is idempotent.
    Yes,
    /// Wait (honour `retry_after`), then retry.
    Later,
    /// Retry the whole transaction at a higher level, not the single call.
    RestartTransaction,
    /// Retry only after the state that caused the failure is fixed.
    AfterFix,
}

impl fmt::Display for RetryAdvice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            RetryAdvice::No => "no",
            RetryAdvice::Yes => "yes",
            RetryAdvice::Later => "later",
            RetryAdvice::RestartTransaction => "restart_transaction",
            RetryAdvice::AfterFix => "after_fix",
        })
    }
}

/// Who acts on an error.
///
/// # Examples
///
/// ```
/// use define_err::{Action, Category};
///
/// assert_eq!(Category::InvalidArgument.default_action(), Action::FixRequest);
/// assert_eq!(Action::PageOnCall.to_string(), "page_on_call");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Action {
    /// Nobody needs to act.
    None,
    /// The caller retries.
    Retry,
    /// The caller fixes the request.
    FixRequest,
    /// An operator fixes the configuration.
    FixConfig,
    /// An operator investigates.
    Operator,
    /// Someone is paged now.
    PageOnCall,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Action::None => "none",
            Action::Retry => "retry",
            Action::FixRequest => "fix_request",
            Action::FixConfig => "fix_config",
            Action::Operator => "operator",
            Action::PageOnCall => "page_on_call",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Category;

    #[test]
    fn test_category_grpc_round_trip() {
        for category in Category::ALL {
            assert_eq!(Category::from_grpc(category.grpc_code()), Some(category));
            assert_eq!(Category::parse(category.name()), Some(category));
        }
        assert_eq!(Category::from_grpc(0), None);
        assert_eq!(Category::from_grpc(17), None);
    }

    #[test]
    fn test_category_never_retries_non_idempotent() {
        for category in Category::ALL {
            assert!(!category.may_retry_automatically(false));
        }
    }
}
