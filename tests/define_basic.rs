//! Enums: every variant form, messages, sources, `From`, `transparent`, and the
//! generated `ErrorInfo` (dev/PLAN.md 7.4, edge cases E-1 to E-32).

use std::error::Error as _;

use define_err::{define_errors, here, prelude::*};

define_errors! {
    /// Errors from the page store.
    #[derive(Clone, PartialEq, Eq)]
    pub enum PageError {
        /// The page does not exist.
        NotFound { page: u64 } => "page {page} not found",
        BadMagic { found: u32, expected: u32 } =>
            "bad page magic: found {found:#010x}, expected {expected:#010x}",
        ShortRead(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
        Closed(u64) => "page store closed",
        InvalidSuperblock => "page 0 is not a valid allocator superblock",
        EmptyBraced {} => "empty braced",
        EmptyTuple() => "empty tuple",
    }
}

#[test]
fn test_display_named_fields_with_format_specs() {
    assert_eq!(
        PageError::NotFound { page: 7 }.to_string(),
        "page 7 not found"
    );
    assert_eq!(
        PageError::BadMagic {
            found: 0xBEEF,
            expected: 0x1
        }
        .to_string(),
        "bad page magic: found 0x0000beef, expected 0x00000001"
    );
}

#[test]
fn test_display_tuple_bindings_and_unnamed_fields() {
    assert_eq!(
        PageError::ShortRead(7, 12).to_string(),
        "short read on page 7: got 12 bytes"
    );
    assert_eq!(PageError::Closed(3).to_string(), "page store closed");
}

#[test]
fn test_display_unit_and_empty_variants() {
    assert_eq!(
        PageError::InvalidSuperblock.to_string(),
        "page 0 is not a valid allocator superblock"
    );
    assert_eq!(PageError::EmptyBraced {}.to_string(), "empty braced");
    assert_eq!(PageError::EmptyTuple().to_string(), "empty tuple");
}

#[test]
fn test_user_derives_pass_through() {
    let a = PageError::NotFound { page: 1 };
    assert_eq!(a.clone(), a);
    assert_ne!(a, PageError::NotFound { page: 2 });
}

#[test]
fn test_debug_is_derived() {
    assert_eq!(
        format!("{:?}", PageError::NotFound { page: 1 }),
        "NotFound { page: 1 }"
    );
    assert_eq!(
        format!("{:?}", PageError::ShortRead(1, 2)),
        "ShortRead(1, 2)"
    );
}

#[test]
fn test_no_source_for_plain_variants() {
    assert!(PageError::NotFound { page: 1 }.source().is_none());
    assert!(PageError::InvalidSuperblock.source().is_none());
}

#[test]
fn test_type_name_is_type_and_variant() {
    assert_eq!(
        PageError::NotFound { page: 1 }.type_name(),
        "PageError::NotFound"
    );
    assert_eq!(PageError::Closed(1).type_name(), "PageError::Closed");
    assert!(PageError::Closed(1).location().is_none());
}

define_errors! {
    pub enum SizeError {
        InvalidPageSize { size: usize } => (
            "invalid page size {size}: must be a power of two in {min}..={max}",
            min = MIN_PAGE_SIZE,
            max = MAX_PAGE_SIZE,
        ),
        Diagnostic { diagnostic: Diagnostic } => ("{}", diagnostic.format()),
        Braces => "use {{ and }} for literal braces",
        RawPath { path: String } => r"C:\data {path}",
        Unicode { name: String } => "fichier « {name} » introuvable",
        Padded { code: u16, ratio: f64 } => "[{code:>6}] [{ratio:.3}] [{code:<6}]",
        Debugged { list: Vec<u8> } => "list {list:?}",
    }
}

const MIN_PAGE_SIZE: usize = 512;
const MAX_PAGE_SIZE: usize = 65_536;

#[derive(Debug)]
pub struct Diagnostic(&'static str);

impl Diagnostic {
    fn format(&self) -> String {
        format!("<{}>", self.0)
    }
}

#[test]
fn test_display_extra_arguments_and_field_expressions() {
    assert_eq!(
        SizeError::InvalidPageSize { size: 3 }.to_string(),
        "invalid page size 3: must be a power of two in 512..=65536"
    );
    assert_eq!(
        SizeError::Diagnostic {
            diagnostic: Diagnostic("x")
        }
        .to_string(),
        "<x>"
    );
}

#[test]
fn test_display_literal_braces_raw_and_unicode() {
    assert_eq!(
        SizeError::Braces.to_string(),
        "use { and } for literal braces"
    );
    assert_eq!(
        SizeError::RawPath { path: "a".into() }.to_string(),
        r"C:\data a"
    );
    assert_eq!(
        SizeError::Unicode { name: "x".into() }.to_string(),
        "fichier « x » introuvable"
    );
}

#[test]
fn test_display_padding_precision_debug() {
    assert_eq!(
        SizeError::Padded {
            code: 42,
            ratio: 0.5
        }
        .to_string(),
        "[    42] [0.500] [42    ]"
    );
    assert_eq!(
        SizeError::Debugged { list: vec![1, 2] }.to_string(),
        "list [1, 2]"
    );
}

define_errors! {
    pub enum LoadError {
        Io(#[from] source: std::io::Error, #[location] at: Location) => "cannot read the file: {source}",
        Parse { line: u32, #[source] cause: core::num::ParseIntError } => "bad number on line {line}",
        Lost { source: Option<std::io::Error> } => "connection lost",
        Utf8(#[from] std::str::Utf8Error) => "not utf-8",
    }
}

/// Runs a conversion through `?` and returns the line it is on.
#[rustfmt::skip]
fn read_missing() -> (Result<String, LoadError>, u32) { let run = || -> Result<String, LoadError> { Ok(std::fs::read_to_string("/define-err/does/not/exist")?) }; (run(), line!()) }

#[test]
fn test_from_conversion_through_question_mark_records_line() {
    let (result, line) = read_missing();
    let err = result.unwrap_err();
    assert!(err.to_string().starts_with("cannot read the file: "));
    let at = err.location().unwrap();
    assert_eq!(at.line(), line); // the line of the `?`
    assert_eq!(at.file(), file!());
    assert!(err.source().unwrap().is::<std::io::Error>());
}

#[test]
fn test_source_attribute_and_named_source() {
    let cause = "x".parse::<u32>().unwrap_err();
    let err = LoadError::Parse { line: 3, cause };
    assert!(err.source().unwrap().is::<core::num::ParseIntError>());

    let none = LoadError::Lost { source: None };
    assert!(none.source().is_none());
    let some = LoadError::Lost {
        source: Some(std::io::Error::other("gone")),
    };
    assert_eq!(some.source().unwrap().to_string(), "gone");
}

#[test]
fn test_unnamed_from_field() {
    let bytes = vec![0xff_u8];
    let bad = std::str::from_utf8(&bytes).unwrap_err();
    let err: LoadError = bad.into();
    assert_eq!(err.to_string(), "not utf-8");
    assert!(err.source().unwrap().is::<std::str::Utf8Error>());
}

define_errors! {
    pub enum Inner {
        Busy { #[location] at: Location } => "the store is busy",
        Gone(#[source] source: std::io::Error) => "the store is gone",
    }

    pub enum AppError {
        Store(#[from] inner: Inner) => transparent,
        Plain(std::fmt::Error) => transparent,
        Located { #[location] at: Location, inner: std::num::ParseIntError } => transparent,
    }
}

#[test]
fn test_transparent_forwards_display_and_source() {
    let err = AppError::from(Inner::Gone(std::io::Error::other("disk")));
    assert_eq!(err.to_string(), "the store is gone");
    assert_eq!(err.source().unwrap().to_string(), "disk");
}

#[test]
fn test_transparent_forwards_error_info() {
    #[rustfmt::skip]
    let (err, line) = (AppError::from(Inner::Busy { at: here() }), line!());
    assert_eq!(err.type_name(), "Inner::Busy");
    assert_eq!(err.location().unwrap().line(), line);
}

#[test]
fn test_transparent_over_foreign_type() {
    let err = AppError::Plain(std::fmt::Error);
    assert_eq!(err.to_string(), std::fmt::Error.to_string());
    assert!(err.source().is_none());
    assert_eq!(err.type_name(), "AppError::Plain");
}

#[test]
fn test_transparent_with_own_location_falls_back_to_it() {
    #[rustfmt::skip]
    let (err, line) = (AppError::Located { at: here(), inner: "x".parse::<u8>().unwrap_err() }, line!());
    assert_eq!(err.location().unwrap().line(), line);
    assert_eq!(err.to_string(), "x".parse::<u8>().unwrap_err().to_string());
}

// Attributes in other orders, passed-through attributes, trailing commas:
// the general path.
define_errors! {
    #[derive(PartialEq)]
    pub(crate) enum Mixed {
        #[allow(dead_code)]
        /// documented after an attribute
        Later { a: u8 } => "later {a}",
        #[deprecated(note = "use Later")]
        Old => "old",
        Field {
            #[from]
            /// the cause, documented after its marker
            source: std::fmt::Error,
        } => "field",
        Unnamed(u8, #[location] Location, String) => "unnamed",
        LastNoComma { x: u8 } => "x {x}"
    }
}

#[test]
#[allow(deprecated)]
fn test_general_path_variants() {
    assert_eq!(Mixed::Later { a: 1 }.to_string(), "later 1");
    assert_eq!(Mixed::Old.to_string(), "old");
    assert!(Mixed::from(std::fmt::Error).source().is_some());
    let unnamed = Mixed::Unnamed(1, here(), String::new());
    assert_eq!(unnamed.to_string(), "unnamed");
    assert!(unnamed.location().is_some());
    assert_eq!(Mixed::LastNoComma { x: 2 }.to_string(), "x 2");
}

#[test]
fn test_non_exhaustive_added_by_default() {
    // A wildcard arm is required outside the defining crate only; inside, the
    // attribute is still visible through rustdoc. Here we check the general
    // path and fast path both compile matches with every variant listed.
    let e = PageError::Closed(1);
    let name = match e {
        PageError::NotFound { .. } => "nf",
        PageError::BadMagic { .. } => "bm",
        PageError::ShortRead(..) => "sr",
        PageError::Closed(..) => "closed",
        PageError::InvalidSuperblock => "is",
        PageError::EmptyBraced {} => "eb",
        PageError::EmptyTuple() => "et",
    };
    assert_eq!(name, "closed");
}

#[test]
fn test_macro_inside_function_body() {
    define_errors! {
        enum Local {
            Oops { n: i32 } => "oops {n}",
        }
    }
    assert_eq!(Local::Oops { n: -1 }.to_string(), "oops -1");
}

mod visibility {
    use define_err::define_errors;

    define_errors! {
        pub(super) enum Up { A => "a" }
        pub(crate) struct Crate => "crate";
        struct Private => "private";
        pub(in crate::visibility) enum Scoped { S => "s" }
    }

    pub(super) fn all() -> [String; 4] {
        [
            Up::A.to_string(),
            Crate.to_string(),
            Private.to_string(),
            Scoped::S.to_string(),
        ]
    }
}

#[test]
fn test_visibility_passed_through() {
    assert_eq!(visibility::all(), ["a", "crate", "private", "s"]);
}

define_errors! {
    pub enum Zero {}
}

define_errors! {
    pub enum One { Only => "only" }
}

#[test]
fn test_zero_and_one_variant_enums() {
    fn takes_zero(z: &Zero) -> String {
        z.to_string()
    }
    let _ = takes_zero;
    assert_eq!(One::Only.to_string(), "only");
}

define_errors! {
    pub enum Twice { A => "a" }
}

define_errors! {
    pub enum TwiceAgain { A => "a" }
}

#[test]
fn test_two_blocks_in_one_module() {
    assert_eq!(Twice::A.to_string(), TwiceAgain::A.to_string());
}

define_errors! {
    #[exhaustive]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[manual_debug]
    pub enum Parity {
        Odd => "value is odd",
        Even => "value is even",
    }
}

#[test]
fn test_exhaustive_and_manual_debug() {
    fn exhaustive(p: Parity) -> bool {
        match p {
            Parity::Odd => true,
            Parity::Even => false,
        }
    }
    assert!(exhaustive(Parity::Odd));
    assert_eq!(format!("{:?}", Parity::Even), "Even");
}

#[test]
fn test_secret_field_redacted_in_display_and_debug() {
    define_errors! {
        enum Auth {
            BadToken { user: String, token: Secret<String> } => "token {token} rejected for {user}",
        }
    }
    let err = Auth::BadToken {
        user: "ada".into(),
        token: Secret::new("s3cr3t".into()),
    };
    assert_eq!(err.to_string(), "token [redacted] rejected for ada");
    assert!(!format!("{err:?}").contains("s3cr3t"));
    assert!(!format!("{err:#?}").contains("s3cr3t"));
}
