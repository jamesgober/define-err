//! Items that generated code calls. Not part of the public API for users:
//! see `docs/GENERATED-CODE.md` for the versioning rule that applies to them.
//!
//! The dispatch macros pick an implementation by autoref method lookup: the
//! macro calls a method on `&&&Wrap(value)`, and method resolution tries the
//! receiver with the most references first, so the impl for the most specific
//! type wins (`Box<dyn Error>` before `Option<E>` before `E: Error`). This is
//! how a declarative macro, which cannot see types, still generates the right
//! code for every field type.

use core::error::Error;

use crate::ErrorInfo;

/// A borrowed value for autoref dispatch.
#[doc(hidden)]
pub struct Wrap<'a, T: ?Sized>(pub &'a T);

impl<T: ?Sized> Clone for Wrap<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Wrap<'_, T> {}

/// Implemented by every type a source field may have. Its only job is the
/// compile error when a source field holds something that is not an error.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "define_errors!: `{Self}` is not an error type, so it cannot be the variant's source",
    label = "this field is the source: it is named `source`, or marked `#[source]` or `#[from]`",
    note = "a source must implement `core::error::Error`; rename the field, or remove `#[source]` or `#[from]`"
)]
pub trait SourceField {}

impl<E: Error + ?Sized> SourceField for E {}

/// `source()` for a field of type `E: Error + 'static`.
#[doc(hidden)]
pub trait SourceViaError<'a> {
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a, E: Error + 'static> SourceViaError<'a> for Wrap<'a, E> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(self.0)
    }
}

/// `source()` for a field of type `Option<E>`.
#[doc(hidden)]
pub trait SourceViaOption<'a> {
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a, E: Error + 'static> SourceViaOption<'a> for &Wrap<'a, Option<E>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        match self.0 {
            Some(error) => Some(error),
            None => None,
        }
    }
}

/// `source()` for boxed and `'static` trait-object fields.
#[doc(hidden)]
pub trait SourceViaObject<'a> {
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a> SourceViaObject<'a> for &&Wrap<'a, &'static (dyn Error + 'static)> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(*self.0)
    }
}

impl<'a> SourceViaObject<'a> for &&Wrap<'a, &'static (dyn Error + Send + Sync + 'static)> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(*self.0)
    }
}

#[cfg(feature = "alloc")]
impl<'a> SourceViaObject<'a> for &&Wrap<'a, alloc::boxed::Box<dyn Error + 'static>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(&**self.0)
    }
}

#[cfg(feature = "alloc")]
impl<'a> SourceViaObject<'a> for &&Wrap<'a, alloc::boxed::Box<dyn Error + Send + 'static>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(&**self.0)
    }
}

#[cfg(feature = "alloc")]
impl<'a> SourceViaObject<'a> for &&Wrap<'a, alloc::boxed::Box<dyn Error + Send + Sync + 'static>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(&**self.0)
    }
}

/// The last resort, reached only when no impl above applies: the field is
/// not an error, and the `SourceField` bound reports it in plain words.
#[doc(hidden)]
pub trait SourceViaNothing<'a, T> {
    fn __define_err_source(self) -> Option<&'a (dyn Error + 'static)>
    where
        T: SourceField;
}

impl<'a, T> SourceViaNothing<'a, T> for Wrap<'a, T> {
    #[inline]
    fn __define_err_source(self) -> Option<&'a (dyn Error + 'static)>
    where
        T: SourceField,
    {
        None
    }
}

/// The inner value's own `source()`, for `transparent` variants.
#[doc(hidden)]
pub trait ForwardViaError<'a> {
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a, E: Error + 'static> ForwardViaError<'a> for Wrap<'a, E> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        self.0.source()
    }
}

/// The inner value's own `source()` when it is a boxed trait object.
#[doc(hidden)]
pub trait ForwardViaObject<'a> {
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)>;
}

#[cfg(feature = "alloc")]
impl<'a> ForwardViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + 'static>> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        (**self.0).source()
    }
}

#[cfg(feature = "alloc")]
impl<'a> ForwardViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + Send + 'static>> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        (**self.0).source()
    }
}

#[cfg(feature = "alloc")]
impl<'a> ForwardViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + Send + Sync + 'static>> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        (**self.0).source()
    }
}

/// `ErrorInfo` of the inner value of a `transparent` variant, when it has one.
#[doc(hidden)]
pub trait InfoViaErrorInfo<'a> {
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo>;
}

impl<'a, E: ErrorInfo> InfoViaErrorInfo<'a> for &Wrap<'a, E> {
    #[inline]
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo> {
        Some(self.0)
    }
}

/// The fallback: the inner value does not implement `ErrorInfo`.
#[doc(hidden)]
pub trait InfoViaNothing<'a> {
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo>;
}

impl<'a, T: ?Sized> InfoViaNothing<'a> for Wrap<'a, T> {
    #[inline]
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo> {
        None
    }
}

/// `source()` of a field: `$crate::__private::source_of!(field)`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_source_of {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{
            SourceViaError as _, SourceViaNothing as _, SourceViaObject as _, SourceViaOption as _,
        };
        (&&&$crate::__private::Wrap($field)).__define_err_source()
    }};
}

/// The inner value's `source()`, for `transparent` variants.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_forward_source {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{ForwardViaError as _, ForwardViaObject as _};
        (&&$crate::__private::Wrap($field)).__define_err_forward()
    }};
}

/// `Option<&dyn ErrorInfo>` of a value, for `transparent` variants.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_info_of {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{InfoViaErrorInfo as _, InfoViaNothing as _};
        (&&$crate::__private::Wrap($field)).__define_err_info()
    }};
}

// Codes, severity, kinds, the catalog, and the probe (0.4). The `const fn`s
// that panic are called only from `const` items in generated code, so a
// failure is a compile error, never a run-time panic (dev/PLAN.md 7.13).

/// The block's namespace. Const evaluation only.
#[doc(hidden)]
#[allow(clippy::panic)]
pub const fn namespace(text: &str, width: u8) -> crate::Namespace {
    match crate::Namespace::try_with_width(text, width) {
        Ok(namespace) => namespace,
        Err(crate::CodeParseError::BadWidth) => {
            panic!("define_errors!: width must be from 1 to 8 digits")
        }
        Err(_) => panic!(
            "define_errors!: namespace must be 1 to 6 letters or digits, starting with a letter, for example \"DB\""
        ),
    }
}

/// A code constant. Const evaluation only.
#[doc(hidden)]
#[allow(clippy::panic)]
pub const fn code(namespace: crate::Namespace, number: u32) -> crate::Code {
    match namespace.code(number) {
        Some(code) => code,
        None => panic!("define_errors!: a code number must be from 0 to 16777215"),
    }
}

/// A retired catalog entry. Const evaluation only.
#[doc(hidden)]
pub const fn retired_code(namespace: crate::Namespace, number: u32) -> crate::CodeInfo {
    crate::CodeInfo::new(
        code(namespace, number),
        "retired",
        "This code is retired and is never reused.",
    )
    .with_state(crate::CodeState::Retired)
}

/// A numeric severity, checked at compile time.
#[doc(hidden)]
pub const fn severity<const N: u8>() -> crate::Severity {
    crate::Severity::at::<N>()
}

/// The block's checks: no number twice (retired numbers included), and every
/// replacement names another code of the block. Const evaluation only.
#[doc(hidden)]
#[allow(clippy::panic)]
pub const fn check_block(
    duplicate_message: &str,
    numbers: &[u32],
    retired: &[u32],
    replacements: &[&[(u32, u32)]],
) {
    let mut i = 0;
    while i < numbers.len() {
        let mut j = i + 1;
        while j < numbers.len() {
            if numbers[i] == numbers[j] {
                panic!("{}", duplicate_message);
            }
            j += 1;
        }
        let mut k = 0;
        while k < retired.len() {
            if numbers[i] == retired[k] {
                panic!("{}", duplicate_message);
            }
            k += 1;
        }
        i += 1;
    }
    let mut a = 0;
    while a < retired.len() {
        let mut b = a + 1;
        while b < retired.len() {
            if retired[a] == retired[b] {
                panic!("{}", duplicate_message);
            }
            b += 1;
        }
        a += 1;
    }
    let mut r = 0;
    while r < replacements.len() {
        let mut p = 0;
        while p < replacements[r].len() {
            let (from, to) = replacements[r][p];
            let mut found = false;
            let mut n = 0;
            while n < numbers.len() {
                if numbers[n] == to {
                    found = true;
                }
                n += 1;
            }
            if from == to || !found {
                panic!("define_errors!: replacement must be another code defined in this block");
            }
            p += 1;
        }
        r += 1;
    }
}

/// The docs template names at least one placeholder. Const evaluation only.
#[doc(hidden)]
#[allow(clippy::panic)]
pub const fn check_docs(template: &str) {
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{'
            && (starts_with(bytes, i, b"{code}")
                || starts_with(bytes, i, b"{namespace}")
                || starts_with(bytes, i, b"{number}"))
        {
            return;
        }
        i += 1;
    }
    panic!("define_errors!: the docs template needs {{code}}, {{namespace}}, or {{number}}");
}

/// Whether `bytes[at..]` starts with `prefix`.
const fn starts_with(bytes: &[u8], at: usize, prefix: &[u8]) -> bool {
    if bytes.len() - at < prefix.len() {
        return false;
    }
    let mut i = 0;
    while i < prefix.len() {
        if bytes[at + i] != prefix[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// A placeholder entry for building catalog arrays; never seen by users.
const PLACEHOLDER: crate::CodeInfo = retired_code(namespace("A", 4), 0);

/// The number of entries in a type's catalog parts.
#[doc(hidden)]
pub const fn catalog_len(parts: &[&[crate::CodeInfo]]) -> usize {
    let mut total = 0;
    let mut i = 0;
    while i < parts.len() {
        total += parts[i].len();
        i += 1;
    }
    total
}

/// A type's catalog from its per-variant parts (each empty or one entry).
#[doc(hidden)]
pub const fn catalog<const N: usize>(parts: &[&[crate::CodeInfo]]) -> [crate::CodeInfo; N] {
    let mut out = [PLACEHOLDER; N];
    let mut at = 0;
    let mut i = 0;
    while i < parts.len() {
        let mut j = 0;
        while j < parts[i].len() {
            if at < N {
                out[at] = parts[i][j];
            }
            at += 1;
            j += 1;
        }
        i += 1;
    }
    out
}

/// Lowercase hex digits.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// The probe string of one variant, built at compile time
/// (`docs/GENERATED-CODE.md`, "The identity probe").
#[doc(hidden)]
pub struct ProbeBuf {
    bytes: [u8; crate::probe::HEADER + crate::probe::MAX_NAME],
    len: usize,
}

impl ProbeBuf {
    /// Builds a probe. Const evaluation only.
    #[allow(clippy::panic)]
    pub const fn new(
        code: Option<u64>,
        severity: u8,
        category: u8,
        kinds: u64,
        flags: u8,
        name: &str,
    ) -> ProbeBuf {
        let mut bytes = [0_u8; crate::probe::HEADER + crate::probe::MAX_NAME];
        let magic = crate::probe::MAGIC;
        bytes[0] = magic[0];
        bytes[1] = magic[1];
        bytes[2] = magic[2];
        bytes[3] = magic[3];
        let mut i = 0;
        while i < 16 {
            bytes[4 + i] = match code {
                Some(value) => HEX[((value >> (60 - 4 * i)) & 0xf) as usize],
                None => b'x',
            };
            i += 1;
        }
        bytes[20] = HEX[(severity >> 4) as usize];
        bytes[21] = HEX[(severity & 0xf) as usize];
        bytes[22] = HEX[(category >> 4) as usize];
        bytes[23] = HEX[(category & 0xf) as usize];
        let mut k = 0;
        while k < 16 {
            bytes[24 + k] = HEX[((kinds >> (60 - 4 * k)) & 0xf) as usize];
            k += 1;
        }
        bytes[40] = HEX[(flags >> 4) as usize];
        bytes[41] = HEX[(flags & 0xf) as usize];
        bytes[42] = b'|';
        let name = name.as_bytes();
        if name.is_empty() || name.len() > crate::probe::MAX_NAME {
            panic!("define_errors!: a type or variant name is longer than 128 bytes");
        }
        let mut n = 0;
        while n < name.len() {
            bytes[crate::probe::HEADER + n] = name[n];
            n += 1;
        }
        ProbeBuf { bytes, len: crate::probe::HEADER + name.len() }
    }

    /// The probe text. Const evaluation only.
    #[allow(clippy::panic)]
    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.bytes.split_at(self.len).0) {
            Ok(text) => text,
            Err(_) => panic!("define_errors!: a type or variant name must be ASCII"),
        }
    }
}

/// The probe of a transparent variant: the inner value's when it has one,
/// else the wrapper's own.
#[doc(hidden)]
#[inline]
pub fn forward_probe<'a>(inner: &'a str, own: &'static str) -> &'a str {
    if inner.as_bytes().starts_with(crate::probe::MAGIC) {
        inner
    } else {
        own
    }
}

/// The inner value's `description()`, for transparent variants.
#[doc(hidden)]
pub trait DescribeViaError<'a> {
    fn __define_err_describe(&self) -> &'a str;
}

impl<'a, E: Error + 'a> DescribeViaError<'a> for Wrap<'a, E> {
    #[inline]
    #[allow(deprecated)]
    fn __define_err_describe(&self) -> &'a str {
        self.0.description()
    }
}

/// The inner value's `description()` when it is a boxed trait object.
#[doc(hidden)]
pub trait DescribeViaObject<'a> {
    fn __define_err_describe(&self) -> &'a str;
}

#[cfg(feature = "alloc")]
impl<'a> DescribeViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + 'static>> {
    #[inline]
    #[allow(deprecated)]
    fn __define_err_describe(&self) -> &'a str {
        (**self.0).description()
    }
}

#[cfg(feature = "alloc")]
impl<'a> DescribeViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + Send + 'static>> {
    #[inline]
    #[allow(deprecated)]
    fn __define_err_describe(&self) -> &'a str {
        (**self.0).description()
    }
}

#[cfg(feature = "alloc")]
impl<'a> DescribeViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + Send + Sync + 'static>> {
    #[inline]
    #[allow(deprecated)]
    fn __define_err_describe(&self) -> &'a str {
        (**self.0).description()
    }
}

/// `retry_after` from a `#[retry_after]` field of type `Duration`.
#[doc(hidden)]
pub trait RetryViaDuration {
    fn __define_err_retry(&self) -> Option<core::time::Duration>;
}

impl RetryViaDuration for Wrap<'_, core::time::Duration> {
    #[inline]
    fn __define_err_retry(&self) -> Option<core::time::Duration> {
        Some(*self.0)
    }
}

/// `retry_after` from a `#[retry_after]` field of type `Option<Duration>`.
#[doc(hidden)]
pub trait RetryViaOption {
    fn __define_err_retry(&self) -> Option<core::time::Duration>;
}

impl RetryViaOption for &Wrap<'_, Option<core::time::Duration>> {
    #[inline]
    fn __define_err_retry(&self) -> Option<core::time::Duration> {
        *self.0
    }
}

/// Platform kind names for `#[kinds(...)]`.
#[doc(hidden)]
pub mod platform_kinds {
    use crate::Kinds;

    pub const ERROR: Kinds = Kinds::ERROR;
    pub const WARNING: Kinds = Kinds::WARNING;
    pub const NOTICE: Kinds = Kinds::NOTICE;
    pub const DEPRECATED: Kinds = Kinds::DEPRECATED;
    pub const SECURITY: Kinds = Kinds::SECURITY;
    pub const FUTURE: Kinds = Kinds::FUTURE;
    pub const RUNTIME: Kinds = Kinds::RUNTIME;
    pub const RESOURCE: Kinds = Kinds::RESOURCE;
    pub const CONFIG: Kinds = Kinds::CONFIG;
    pub const USER: Kinds = Kinds::USER;
    pub const PANIC: Kinds = Kinds::PANIC;
    pub const AUDIT: Kinds = Kinds::AUDIT;
    pub const TRIGGER: Kinds = Kinds::TRIGGER;
}

/// Severity names for `#[severity(...)]`.
#[doc(hidden)]
pub mod named_levels {
    use crate::Severity;

    pub const TRACE: Severity = Severity::TRACE;
    pub const DEBUG: Severity = Severity::DEBUG;
    pub const INFO: Severity = Severity::INFO;
    pub const NOTICE: Severity = Severity::NOTICE;
    pub const WARNING: Severity = Severity::WARNING;
    pub const ERROR: Severity = Severity::ERROR;
    pub const CRITICAL: Severity = Severity::CRITICAL;
    pub const FATAL: Severity = Severity::FATAL;
}

/// The inner value's `description()`, for transparent variants.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_description_of {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{DescribeViaError as _, DescribeViaObject as _};
        (&&$crate::__private::Wrap($field)).__define_err_describe()
    }};
}

/// `retry_after` from a `#[retry_after]` field.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_retry_after_of {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{RetryViaDuration as _, RetryViaOption as _};
        (&&$crate::__private::Wrap($field)).__define_err_retry()
    }};
}

#[doc(hidden)]
pub use crate::{
    __define_err_description_of as description_of, __define_err_forward_source as forward_source,
    __define_err_info_of as info_of, __define_err_retry_after_of as retry_after_of,
    __define_err_source_of as source_of,
};

#[cfg(test)]
mod tests {
    extern crate std;

    use core::{error::Error, fmt};
    #[cfg(feature = "alloc")]
    use std::boxed::Box;

    use super::{forward_source, info_of, source_of};

    #[derive(Debug)]
    struct Leaf;

    impl fmt::Display for Leaf {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("leaf")
        }
    }

    impl Error for Leaf {}

    #[derive(Debug)]
    struct Node(Leaf);

    impl fmt::Display for Node {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("node")
        }
    }

    impl Error for Node {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    impl crate::ErrorInfo for Node {}

    #[test]
    fn test_source_of_plain_error_returns_it() {
        let leaf = Leaf;
        let source = source_of!(&leaf);
        assert_eq!(source.map(|e| std::format!("{e}")).as_deref(), Some("leaf"));
    }

    #[test]
    fn test_source_of_option_none_and_some() {
        let none: Option<Leaf> = None;
        assert!(source_of!(&none).is_none());
        let some = Some(Leaf);
        assert!(source_of!(&some).is_some());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_source_of_boxed_objects_return_inner() {
        let a: Box<dyn Error> = Box::new(Leaf);
        let b: Box<dyn Error + Send> = Box::new(Leaf);
        let c: Box<dyn Error + Send + Sync> = Box::new(Leaf);
        assert!(source_of!(&a).is_some_and(|e| e.is::<Leaf>()));
        assert!(source_of!(&b).is_some_and(|e| e.is::<Leaf>()));
        assert!(source_of!(&c).is_some_and(|e| e.is::<Leaf>()));
    }

    #[test]
    fn test_source_of_static_object_returns_it() {
        static LEAF: Leaf = Leaf;
        let object: &'static (dyn Error + 'static) = &LEAF;
        assert!(source_of!(&object).is_some());
    }

    #[test]
    fn test_forward_source_skips_the_wrapped_value() {
        let node = Node(Leaf);
        assert!(forward_source!(&node).is_some_and(|e| e.is::<Leaf>()));
        assert!(forward_source!(&Leaf).is_none());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_forward_source_through_box() {
        let boxed: Box<dyn Error + Send + Sync> = Box::new(Node(Leaf));
        assert!(forward_source!(&boxed).is_some_and(|e| e.is::<Leaf>()));
    }

    #[test]
    fn test_info_of_finds_error_info_or_none() {
        assert!(info_of!(&Node(Leaf)).is_some());
        assert!(info_of!(&Leaf).is_none());
    }
}
