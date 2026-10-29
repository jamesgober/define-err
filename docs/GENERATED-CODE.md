<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br><b>define-err</b><br>
    <sub><sup>GENERATED-CODE CONTRACT</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./API.md" title="API reference"><b>API</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>GENERATED CODE</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="../dev/PLAN.md" title="Plan"><b>PLAN</b></a>
    </sup>
</div>
<br>

What `define_errors!` expands to, item by item, and the hidden `__private` surface that generated code calls. Two front ends produce this code: the `define_errors!` macro in this crate, and the attribute front end (L2, `#[derive(Error)]` with `#[error("...")]`), a separate crate. Both generate the same items with the same behaviour; the only differences are the input syntax and the three listed in [Differences between the front ends](#differences-between-the-front-ends).

Locked at the `0.2.0` design lock. The items described here ship in `0.3.0` (definitions) and `0.4.0` (codes, metadata, the probe). A change to anything in this file after `0.2.0` is recorded in the decision log of [`../dev/PLAN.md`](../dev/PLAN.md) and in the changelog of the release that makes it.

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>

## Table of contents

- [Rules every expansion follows](#rules-every-expansion-follows)
- [Items generated for every type](#items-generated-for-every-type)
- [Expansions](#expansions)
  - [1. Plain enum: unit, braced, and tuple variants](#1-plain-enum-unit-braced-and-tuple-variants)
  - [2. Sources and `From`](#2-sources-and-from)
  - [3. Extra arguments](#3-extra-arguments)
  - [4. Transparent wrapping](#4-transparent-wrapping)
  - [5. Structs](#5-structs)
  - [6. `#[exhaustive]` and `#[manual_debug]`](#6-exhaustive-and-manual_debug)
  - [7. Codes and metadata](#7-codes-and-metadata)
  - [8. Retired and deprecated codes](#8-retired-and-deprecated-codes)
  - [9. `#[cfg]` on variants](#9-cfg-on-variants)
- [`ErrorInfo` methods](#errorinfo-methods)
- [The identity probe](#the-identity-probe)
- [Compile-time checks](#compile-time-checks)
- [The `__private` surface](#the-__private-surface)
  - [Versioning rule](#versioning-rule)
  - [Items](#items)
- [Differences between the front ends](#differences-between-the-front-ends)

## Rules every expansion follows

1. **Absolute paths.** Every path starts with `::core::` or `$crate::` (the attribute front end writes `::define_err::`). A user type named `Error`, `Result`, `Option`, `Box`, `Display`, `Location`, or `Code` never changes the meaning of generated code, and generated code compiles in a module with `#![no_implicit_prelude]`.
2. **No feature `cfg`.** A `#[cfg(feature = "...")]` inside an expansion would test the user's features. Generated code is identical whatever features this crate has.
3. **No side effects at construction.** Building a value runs no code: no hook, no allocation, no atomic operation.
4. **Nothing extra.** No constructors, no inherent methods other than the opt-in code constants and `CATALOG`, no statics, no `Result` alias.
5. **No lints fire.** Generated items are clean under the user's `#![deny(warnings)]`, `clippy::pedantic`, `missing_docs` (generated public items carry a doc line), `unreachable_pub`, `#![no_std]`, and `#![forbid(unsafe_code)]`. Generated code contains no `unsafe`.
6. **Layout is the user's.** The type's size, alignment, `Send`, `Sync`, and `Unpin` are exactly those of its fields; field markers are removed and nothing is added to the type.

In the expansions below, paths are written in full once and then shortened: `$crate` stands for `::define_err`, and `core` for `::core`. Generated helper names start with `__define_err_` and are private to the user's module.

The expansions are shown in the shape of a hand-written impl, for reading. The real expansion differs only in form, never in behaviour:

- every arm matches `*self` and binds every field with `ref` (`match *self { Self::NotFound { ref page } => ... }`), which gives the same `&T` bindings and also works for an enum with no variants;
- `source()` and `ErrorInfo::location` are always a full `match`, returning `None` for variants that have no source or location (PLAN D-025);
- each generated impl carries `#[allow(deprecated)]`, so a `#[deprecated]` variant warns where the user names it, never inside generated code;
- unnamed tuple fields are bound as `__define_err_source` (a `#[from]` or `#[source]` field), `__define_err_location` (a `#[location]` field), or `__define_err_0` to `__define_err_15` (any other), and the `From` parameter is `__define_err_value`; these names are hygienic and never clash with the user's.

## Items generated for every type

For a type `T` with variants `V1..Vn` (a struct is one variant):

| # | Item | Content |
|---|---|---|
| G1 | the type | user attributes passed through; `#[derive(::core::fmt::Debug)]` unless `#[manual_debug]`; `#[non_exhaustive]` on enums unless `#[exhaustive]` or the user already wrote `#[non_exhaustive]` (never a second copy; structs: never added, because it would forbid construction outside the crate); field markers removed; `#[location]` fields keep their declared type, which must be `$crate::Location` |
| G2 | `impl ::core::fmt::Display for T` | one `match self` with one arm per variant binding every field by its name (tuple fields by their binding name, or `_` when unnamed); the arm calls `::core::write!(f, <format string> [, <arguments>])`; `transparent` arms call `::core::fmt::Display::fmt(inner, f)`; `#[allow(unused_variables)]` on the match |
| G3 | `impl ::core::error::Error for T` | `source()`: per variant, `$crate::__private::source_of!(field)` for the source field, `transparent` forwards to the inner value's own `source()` through `$crate::__private::forward_source!(inner)`, otherwise `None`; from `0.4`, `description()`: per variant the identity probe string, under `#[allow(deprecated)]`; `transparent` arms forward the inner probe when the inner value has one |
| G4 | `impl ::core::convert::From<S> for T` | one per `#[from]` field; `#[track_caller]`; sets a `#[location]` field to `::core::panic::Location::caller()` |
| G5 | `impl $crate::ErrorInfo for T` | always generated; methods with no per-variant data are left to the trait defaults; others are one `match self` returning constants or copying a field |
| G6 | `impl T { pub const <NAME>: $crate::Code = ...; }` | one per `name = NAME` in `#[code]`; `#[deprecated(since, note)]` when the code is deprecated |
| G7 | `impl T { pub const CATALOG: &'static [$crate::CodeInfo] = &[...]; }` | only when the block has a namespace; one entry per code of this type, plus, on the first type of the block, one entry per retired number |
| G8 | `const _: () = { ... };` | compile-time checks for the whole block (see [Compile-time checks](#compile-time-checks)) |

## Expansions

### 1. Plain enum: unit, braced, and tuple variants

Input:

```rust
define_errors! {
    /// Errors from the page store.
    #[derive(Clone, PartialEq, Eq)]
    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",
        BadMagic { found: u32, expected: u32 } =>
            "bad page magic: found {found:#010x}, expected {expected:#010x}",
        ShortRead(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
        InvalidSuperblock => "page 0 is not a valid allocator superblock",
    }
}
```

Output:

```rust
/// Errors from the page store.
#[derive(Clone, PartialEq, Eq)]
#[derive(::core::fmt::Debug)]
#[non_exhaustive]
pub enum PageError {
    NotFound { page: u64 },
    BadMagic { found: u32, expected: u32 },
    ShortRead(u64, usize),
    InvalidSuperblock,
}

impl ::core::fmt::Display for PageError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        #[allow(unused_variables)]
        match self {
            Self::NotFound { page } => ::core::write!(f, "page {page} not found"),
            Self::BadMagic { found, expected } => ::core::write!(
                f,
                "bad page magic: found {found:#010x}, expected {expected:#010x}"
            ),
            Self::ShortRead(page, got) => ::core::write!(f, "short read on page {page}: got {got} bytes"),
            Self::InvalidSuperblock => ::core::write!(f, "page 0 is not a valid allocator superblock"),
        }
    }
}

impl ::core::error::Error for PageError {
    fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
        ::core::option::Option::None
    }

    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            Self::NotFound { .. } => {
                const B: &$crate::__private::ProbeBuf = &$crate::__private::ProbeBuf::new(
                    ::core::option::Option::None, 17, 2, 1, 0,
                    ::core::concat!(::core::stringify!(PageError), "::", ::core::stringify!(NotFound)),
                );
                const S: &str = B.as_str();
                S
            }
            // one arm of the same shape per variant
            # _ => ::core::unreachable!(),
        }
    }
}

impl $crate::ErrorInfo for PageError {
    fn type_name(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "PageError::NotFound",
            Self::BadMagic { .. } => "PageError::BadMagic",
            Self::ShortRead(..) => "PageError::ShortRead",
            Self::InvalidSuperblock => "PageError::InvalidSuperblock",
        }
    }
}
```

(The `# _ =>` line marks an elision in this document; the real expansion lists every arm and has no catch-all.) A plain type's `source()` returns `None` from every arm, and its `ErrorInfo` impl overrides `location` (`None` from every arm) and `type_name`; every other method is the trait default (from `0.4`: `None`, `Category::Unknown`, `Severity::ERROR`, `Kinds::ERROR`, and so on). A format string that writes no field compiles to the same `write!` call; rustc turns a string with no arguments into a single `write_str`. The `description()` probe arrives in `0.4`; in `0.3` it is the standard library's default.

An enum with zero variants generates `match *self {}` in `fmt`, `source`, `description`, and `type_name`.

### 2. Sources and `From`

Input:

```rust
define_errors! {
    pub enum LoadError {
        Io(#[from] source: std::io::Error) => "cannot read the file: {source}",
        Parse { line: u32, #[source] cause: core::num::ParseIntError } => "bad number on line {line}",
        Lost { source: Option<std::io::Error> } => "connection lost",
        Other(#[from] inner: Box<dyn core::error::Error + Send + Sync>, #[location] at: Location) =>
            "unexpected failure: {inner}",
    }
}
```

Output (`Display` as in expansion 1):

```rust
#[derive(::core::fmt::Debug)]
#[non_exhaustive]
pub enum LoadError {
    Io(std::io::Error),
    Parse { line: u32, cause: core::num::ParseIntError },
    Lost { source: Option<std::io::Error> },
    Other(Box<dyn core::error::Error + Send + Sync>, Location),
}

impl ::core::error::Error for LoadError {
    fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
        match self {
            Self::Io(source) => $crate::__private::source_of!(source),
            Self::Parse { cause, .. } => $crate::__private::source_of!(cause),
            Self::Lost { source } => $crate::__private::source_of!(source),
            Self::Other(inner, _) => $crate::__private::source_of!(inner),
        }
    }
    // description() as in expansion 1
}

impl ::core::convert::From<std::io::Error> for LoadError {
    #[track_caller]
    fn from(source: std::io::Error) -> Self {
        Self::Io(source)
    }
}

impl ::core::convert::From<Box<dyn core::error::Error + Send + Sync>> for LoadError {
    #[track_caller]
    fn from(inner: Box<dyn core::error::Error + Send + Sync>) -> Self {
        Self::Other(inner, ::core::panic::Location::caller())
    }
}

impl $crate::ErrorInfo for LoadError {
    fn location(&self) -> ::core::option::Option<$crate::Location> {
        match self {
            Self::Other(_, at) => ::core::option::Option::Some(*at),
            _ => ::core::option::Option::None,
        }
    }
    // type_name() as in expansion 1
}
```

- A field named `source` is the source without an attribute (`Lost`). `#[from]` implies `#[source]`.
- `source_of!` accepts any `E: Error + 'static`, `Option<E>`, `Box<dyn Error>`, `Box<dyn Error + Send>`, `Box<dyn Error + Send + Sync>`, and `&'static dyn Error`, by autoref method dispatch: the macro calls a method on `&&&Wrap(field)`, and the first matching impl wins. Box sources need nothing from the user's features: the impls for `Box` exist when this crate's `alloc` feature is on, and any crate that names `Box<dyn Error>` uses `std` (this crate's default) or enables `alloc`.
- `#[track_caller]` on `from` makes `?` record the line of the `?` itself: `Location::caller()` inside `from` is the location of the conversion's call site, which for `?` is the `?` expression.
- `#[location]` fields hold `&'static core::panic::Location<'static>`, 8 bytes on 64-bit targets.

### 3. Extra arguments

Input:

```rust
define_errors! {
    pub enum SizeError {
        InvalidPageSize { size: usize } => (
            "invalid page size {size}: must be a power of two in {min}..={max}",
            min = crate::MIN_PAGE_SIZE,
            max = crate::MAX_PAGE_SIZE,
        ),
        Diagnostic { diagnostic: Diagnostic } => ("{}", diagnostic.format()),
    }
}
```

Output (`Display` only):

```rust
impl ::core::fmt::Display for SizeError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        #[allow(unused_variables)]
        match self {
            Self::InvalidPageSize { size } => ::core::write!(
                f,
                "invalid page size {size}: must be a power of two in {min}..={max}",
                min = crate::MIN_PAGE_SIZE,
                max = crate::MAX_PAGE_SIZE,
            ),
            Self::Diagnostic { diagnostic } => ::core::write!(f, "{}", diagnostic.format()),
        }
    }
}
```

The arguments are copied after the format string unchanged; every field is in scope by name as a reference, as in a thiserror `.field` argument.

### 4. Transparent wrapping

Input:

```rust
define_errors! {
    pub enum AppError {
        Config(#[from] inner: ConfigError) => transparent,
        Other(#[from] inner: Box<dyn core::error::Error + Send + Sync>) => transparent,
    }
}
```

Output (the parts that differ):

```rust
impl ::core::fmt::Display for AppError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::Config(inner) => ::core::fmt::Display::fmt(inner, f),
            Self::Other(inner) => ::core::fmt::Display::fmt(inner, f),
        }
    }
}

impl ::core::error::Error for AppError {
    fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
        match self {
            Self::Config(inner) => $crate::__private::forward_source!(inner),   // inner.source()
            Self::Other(inner) => $crate::__private::forward_source!(inner),    // (**inner).source()
        }
    }

    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            Self::Config(inner) => $crate::__private::forward_probe(
                ::core::error::Error::description(inner),
                { const B: &$crate::__private::ProbeBuf = &/* own probe, flags = 0x02 */; const S: &str = B.as_str(); S },
            ),
            // Other: the same shape
            # _ => ::core::unreachable!(),
        }
    }
}

impl $crate::ErrorInfo for AppError {
    fn code(&self) -> ::core::option::Option<$crate::Code> {
        match self {
            Self::Config(inner) => match $crate::__private::info_of!(inner) {
                ::core::option::Option::Some(info) => info.code(),
                ::core::option::Option::None => ::core::option::Option::None,
            },
            # _ => ::core::unreachable!(),
        }
    }
    // every other ErrorInfo method forwards the same way; the fallback is the trait default
}
```

- A transparent variant is its inner value as far as anyone can tell: `Display`, `source()`, every `ErrorInfo` method, and the probe all come from the inner value. A wrapper adds no entry to the error chain, so a chain walker that reads probes must see the inner identity here, not the wrapper's.
- `forward_probe(inner, own)` returns `inner` when it starts with the probe magic, and `own` otherwise. `own` carries flag bit 1 (transparent), so a transparent wrapper around a foreign error is still recognised as generated code.
- `info_of!(inner)` returns `Option<&dyn ErrorInfo>` by the same autoref dispatch as `source_of!`: `Some` when the inner type implements `ErrorInfo`, `None` otherwise (then each method returns the trait default).
- `transparent` needs exactly one field besides an optional `#[location]`; it may be `#[from]`.

### 5. Structs

Input:

```rust
define_errors! {
    /// A syntax error in a NOML file.
    pub struct SyntaxError { pub line: u32, pub column: u32 } => "syntax error at {line}:{column}";
    pub struct Timeout(pub u64) => "operation timed out";
    pub struct Cancelled => "operation cancelled";
}
```

Output (`Display` and the type; the other items as in expansion 1):

```rust
/// A syntax error in a NOML file.
#[derive(::core::fmt::Debug)]
pub struct SyntaxError { pub line: u32, pub column: u32 }

impl ::core::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        #[allow(unused_variables)]
        let Self { line, column } = self;
        ::core::write!(f, "syntax error at {line}:{column}")
    }
}

#[derive(::core::fmt::Debug)]
pub struct Timeout(pub u64);

#[derive(::core::fmt::Debug)]
pub struct Cancelled;
```

Structs never get `#[non_exhaustive]` (it would forbid building them outside the crate); a struct's metadata attributes sit on the struct. The probe and `type_name` of a struct are the struct's name alone (`"SyntaxError"`).

### 6. `#[exhaustive]` and `#[manual_debug]`

```rust
define_errors! {
    #[exhaustive]
    #[manual_debug]
    pub enum Parity {
        Odd => "value is odd",
        Even => "value is even",
    }
}
impl core::fmt::Debug for Parity { /* the user's own */ }
```

`#[exhaustive]` leaves out `#[non_exhaustive]`; `#[manual_debug]` leaves out the `Debug` derive. Both markers are removed from the output. `#[exhaustive]` on a struct is a compile error ("`#[exhaustive]` applies only to enums"). Without `#[manual_debug]`, a user `#[derive(Debug)]` conflicts with the generated one; the compiler reports conflicting implementations.

### 7. Codes and metadata

Input:

```rust
use define_err::{Kinds, Location};

pub const BILLING: Kinds = Kinds::project::<0>();

define_errors! {
    namespace = "DB";
    docs = "https://docs.hivedb.com/errors/{code}";

    pub enum DbError {
        #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        #[retry_after(secs = 2)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt { page: u64, #[location] at: Location } => "page {page} failed its checksum",

        #[code(3, help = "The pool closed while the request waited.")]
        #[category(Unavailable)]
        PoolClosed { #[retry_after] wait: core::time::Duration } => "connection pool closed",

        #[code(10, help = "Raise the tenant quota or wait for the window to reset.")]
        #[category(ResourceExhausted)]
        #[kinds(BILLING)]
        #[action(Operator)]
        Quota { tenant: u64 } => "tenant {tenant} is over its quota",
    }
}
```

Output (the items codes and metadata add):

```rust
const __define_err_ns_DbError: $crate::Namespace = $crate::__private::namespace("DB", 4);

impl DbError {
    /// Generated by define_errors!: the code `DB-0001`.
    pub const TIMEOUT: $crate::Code = $crate::__private::code(__define_err_ns_DbError, 1);

    /// Generated by define_errors!: the catalog entries of this type.
    pub const CATALOG: &'static [$crate::CodeInfo] = &[
        $crate::CodeInfo::new(
            $crate::__private::code(__define_err_ns_DbError, 1),
            "Timeout",
            "Raise pool.size or lower the load.",
        )
        .with_docs("https://docs.hivedb.com/errors/{code}")
        .with_category($crate::Category::DeadlineExceeded)
        .with_severity($crate::Category::DeadlineExceeded.default_severity())
        .with_kinds($crate::Kinds::ERROR)
        .with_action($crate::Category::DeadlineExceeded.default_action())
        .with_public("The database is busy. Try again shortly.")
        .with_type_name("DbError::Timeout"),
        // DB-0002, DB-0003, DB-0010 in the same shape
    ];
}

impl $crate::ErrorInfo for DbError {
    fn code(&self) -> ::core::option::Option<$crate::Code> {
        match self {
            Self::Timeout { .. } => ::core::option::Option::Some(Self::CATALOG[0].code),
            Self::Corrupt { .. } => ::core::option::Option::Some(Self::CATALOG[1].code),
            Self::PoolClosed { .. } => ::core::option::Option::Some(Self::CATALOG[2].code),
            Self::Quota { .. } => ::core::option::Option::Some(Self::CATALOG[3].code),
        }
    }
    fn code_info(&self) -> ::core::option::Option<&'static $crate::CodeInfo> {
        match self {
            Self::Timeout { .. } => ::core::option::Option::Some(&Self::CATALOG[0]),
            # _ => ::core::unreachable!(),
        }
    }
    fn category(&self) -> $crate::Category {
        match self {
            Self::Timeout { .. } => $crate::Category::DeadlineExceeded,
            Self::Corrupt { .. } => $crate::Category::DataLoss,
            Self::PoolClosed { .. } => $crate::Category::Unavailable,
            Self::Quota { .. } => $crate::Category::ResourceExhausted,
        }
    }
    fn severity(&self) -> $crate::Severity {
        match self {
            Self::Corrupt { .. } => { use $crate::__private::named_levels::*; CRITICAL }
            _ => $crate::Category::default_severity(self.category()),
        }
    }
    fn kinds(&self) -> $crate::Kinds {
        match self {
            Self::Corrupt { .. } => { use $crate::__private::platform_kinds::*; $crate::Kinds::NONE.union(ERROR).union(SECURITY) }
            Self::Quota { .. } => { use $crate::__private::platform_kinds::*; $crate::Kinds::NONE.union(BILLING) }
            _ => $crate::Kinds::ERROR,
        }
    }
    fn public_message(&self) -> &'static str {
        match self {
            Self::Timeout { .. } => "The database is busy. Try again shortly.",
            _ => $crate::Category::default_public_message(self.category()),
        }
    }
    fn help(&self) -> ::core::option::Option<&'static str> {
        match self {
            Self::Timeout { .. } => ::core::option::Option::Some("Raise pool.size or lower the load."),
            # _ => ::core::unreachable!(),
        }
    }
    fn action(&self) -> $crate::Action {
        match self {
            Self::Quota { .. } => $crate::Action::Operator,
            _ => $crate::Category::default_action(self.category()),
        }
    }
    fn retry_after(&self) -> ::core::option::Option<::core::time::Duration> {
        match self {
            Self::Timeout { .. } => ::core::option::Option::Some(::core::time::Duration::from_secs(2)),
            Self::PoolClosed { wait } => $crate::__private::retry_after_of!(wait),
            _ => ::core::option::Option::None,
        }
    }
    fn location(&self) -> ::core::option::Option<$crate::Location> {
        match self {
            Self::Corrupt { at, .. } => ::core::option::Option::Some(*at),
            _ => ::core::option::Option::None,
        }
    }
    // is_retryable() is the trait default: self.category().is_transient()
    // type_name() as in expansion 1
}

const _: () = {
    $crate::__private::check_block(__define_err_ns_DbError, &[1, 2, 3, 10], &[], &[]);
    $crate::__private::check_docs("https://docs.hivedb.com/errors/{code}");
    #[deny(unreachable_patterns)]
    match 0u32 { 1 => {} 2 => {} 3 => {} 10 => {} _ => {} }
};
```

- The namespace constant is emitted once per block, named after the block's first type, so two blocks in one module never clash.
- Kind expressions are written with `|` and compiled to `Kinds::union` calls, which are `const fn`, so the same expression serves the catalog, the probe, and `kinds()`. Platform names resolve through a glob import of `__private::platform_kinds`; a project constant such as `BILLING` resolves from the user's scope.
- Severity names (`CRITICAL`) resolve through `__private::named_levels`; a number (`#[severity(22)]`) becomes `$crate::__private::severity::<22>()`, which fails to compile outside 1 to 24.
- `retry_after_of!(wait)` turns a `Duration` field into `Some(*wait)` and an `Option<Duration>` field into `*wait`, by the same autoref dispatch as `source_of!`.
- A variant without `#[category]` reports `Category::Unknown`; its severity, action, retry flag, and public message come from that category.

### 8. Retired and deprecated codes

Input:

```rust
define_errors! {
    namespace = "DB";
    retired = [9, 12];

    pub enum DbError {
        #[code(4, name = OLD_QUOTA, help = "Use DB-0010 (DbError::Quota) instead.", deprecated = "2.1", replacement = 10)]
        #[category(ResourceExhausted)]
        OldQuota => "quota exceeded",

        #[code(10, help = "Raise the tenant quota or wait for the window to reset.")]
        #[category(ResourceExhausted)]
        Quota { tenant: u64 } => "tenant {tenant} is over its quota",
    }
}
```

Output (the parts that differ):

```rust
impl DbError {
    /// Generated by define_errors!: the code `DB-0004`.
    #[deprecated(since = "2.1", note = "replaced by DB-0010")]
    pub const OLD_QUOTA: $crate::Code = $crate::__private::code(__define_err_ns_DbError, 4);

    pub const CATALOG: &'static [$crate::CodeInfo] = &[
        /* DB-0004 */ $crate::CodeInfo::new(/* ... */)
            .with_state($crate::CodeState::Deprecated {
                since: "2.1",
                replacement: ::core::option::Option::Some($crate::__private::code(__define_err_ns_DbError, 10)),
            }),
        /* DB-0010 */ $crate::CodeInfo::new(/* ... */),
        /* DB-0009 */ $crate::__private::retired_code(__define_err_ns_DbError, 9),
        /* DB-0012 */ $crate::__private::retired_code(__define_err_ns_DbError, 12),
    ];
}

const _: () = {
    $crate::__private::check_block(__define_err_ns_DbError, &[4, 10], &[9, 12], &[(4, 10)]);
    #[deny(unreachable_patterns)]
    match 0u32 { 4 => {} 10 => {} 9 => {} 12 => {} _ => {} }
};
```

`retired_code` builds an entry with `CodeState::Retired`, the help text "This code is retired and is never reused.", and an empty type name. Retired entries go on the first type of the block only. Generated code never names a deprecated constant itself, so the `deprecated` warning fires only where a user names `DbError::OLD_QUOTA`.

### 9. `#[cfg]` on variants

```rust
define_errors! {
    pub enum NetError {
        Refused => "connection refused",
        #[cfg(feature = "tls")]
        Tls { alert: u8 } => "tls alert {alert}",
    }
}
```

Every arm generated for `Tls` in every impl (`Display`, `source`, `description`, each `ErrorInfo` method, a `From` impl, a catalog entry, the duplicate check) carries the same `#[cfg(feature = "tls")]`, which is the user's own feature in the user's crate. This is the one place a `cfg` appears in generated code, and it is copied from the user's input, never added. Arrives in `0.5.0` (DEF-24).

## `ErrorInfo` methods

| Method | Generated when | Value |
|---|---|---|
| `code(&self) -> Option<Code>` | any variant has `#[code]` | `Some(code)` or `None` per variant |
| `code_info(&self) -> Option<&'static CodeInfo>` | same | `Some(&Self::CATALOG[i])` |
| `category(&self) -> Category` | any variant has `#[category]` | the variant's category, else `Category::Unknown` |
| `severity(&self) -> Severity` | any variant has `#[severity]` or `#[category]` | explicit severity, else the category default |
| `kinds(&self) -> Kinds` | any variant has `#[kinds]` | explicit kinds, else `Kinds::ERROR` |
| `public_message(&self) -> &'static str` | any variant has `#[public]` or `#[category]` | explicit text, else the category's default text |
| `help(&self) -> Option<&'static str>` | any variant has `#[code]` | the code's help text |
| `action(&self) -> Action` | any variant has `#[action]` or `#[category]` | explicit, else the category default |
| `is_retryable(&self) -> bool` | any variant has `#[retryable]` | explicit, else `category().is_transient()` |
| `retry_after(&self) -> Option<Duration>` | any variant has `#[retry_after]` | field value, or `Some(constant)` for the static form, else `None` |
| `location(&self) -> Option<Location>` | any variant has a `#[location]` field | `Some(*at)` or `None` |
| `type_name(&self) -> &'static str` | always | `"Type::Variant"`, or `"Type"` for a struct |

A method not in the "generated" column for a type is the trait default. Every generated method is a `match` returning constants or copying a field: no allocation, no lock, constant time. `transparent` variants forward every method to the inner value (expansion 4).

## The identity probe

`core::error::Error::description(&self) -> &str` is stable, deprecated for callers, and implementable. Generated code returns a fixed string per variant, built at compile time, which `define_err::identify` decodes from any `&dyn Error`. Format (frozen at 1.0):

| Offset | Length | Content |
|---|---|---|
| 0 | 4 | magic: the bytes `00 44 45 31` (NUL, `D`, `E`, format version `1`) |
| 4 | 16 | the code's identity (`Code::identity`, width bits cleared) in lowercase hex, or 16 `x` for no code |
| 20 | 2 | severity in lowercase hex (`01` to `18`) |
| 22 | 2 | category gRPC number in lowercase hex (`01` to `10`) |
| 24 | 16 | kinds bits in lowercase hex |
| 40 | 2 | flags in lowercase hex: bit 0 the variant has `#[public]`; bit 1 transparent; other bits zero |
| 42 | 1 | `\|` |
| 43 | 1 to 128 | type name, `Type::Variant` or `Type` (ASCII) |

Examples, from [`tests/golden/probe.txt`](../tests/golden/probe.txt) (`\0` is the NUL byte):

```text
DbError::Timeout     \0DE13b34a2f2000000011104000000000000000101|DbError::Timeout
PageError::NotFound  \0DE1xxxxxxxxxxxxxxxx1102000000000000000100|PageError::NotFound
```

The probe is built by `__private::ProbeBuf::new` in a `const` item and turned into `&'static str` by a `const fn`, so at run time `description()` returns a pointer and a length. A type name longer than 128 bytes is a compile error. Code that calls the deprecated `description()` on a generated type gets the probe text instead of a sentence; no portfolio crate does.

## Compile-time checks

The rule: a `panic!` may only be reachable in constant evaluation, where it is a compile error. Generated code calls `const fn`s from `__private` in `const` items only.

| Check | Where | Message |
|---|---|---|
| namespace text: 1 to 6 characters, `A`-`Z` and `0`-`9`, first a letter; lowercase accepted | `__private::namespace` | "define_errors!: namespace must be 1 to 6 letters or digits, starting with a letter, for example \"DB\"" |
| width 1 to 8 | `__private::namespace` | "define_errors!: width must be from 1 to 8 digits" |
| code number 0 to 16,777,215 | `__private::code` | "define_errors!: a code number must be from 0 to 16777215" |
| duplicate number in the block, including retired numbers | `__private::check_block` and the `match` | "define_errors!: duplicate code number in namespace DB"; rustc also marks the second literal as an unreachable pattern |
| `replacement` names a code of the block other than itself | `__private::check_block` | "define_errors!: replacement must be another code defined in this block" |
| docs template contains `{code}`, `{namespace}`, or `{number}` | `__private::check_docs` | "define_errors!: the docs template needs {code}, {namespace}, or {number}" |
| severity number 1 to 24 | `__private::severity::<N>()` | "define_errors!: severity must be from 1 to 24 (OpenTelemetry SeverityNumber)" |
| project kind bit below 32 | `Kinds::project::<N>()` | "Kinds::project: the bit must be below 32" |
| type name at most 128 bytes | `__private::ProbeBuf::new` | "define_errors!: a type or variant name is longer than 128 bytes" |

Rules the macro itself rejects with `compile_error!` (each names the fix): `#[code]` without `help`, `#[code]` without a `namespace` header, an unknown attribute in the metadata position (the message lists the valid ones), two source fields, two `#[location]` fields, two `#[retry_after]` fields, `#[from]` with fields other than the source and an optional `#[location]`, `transparent` with zero or two fields, a missing `=> message`, `#[exhaustive]` on a struct, and a non-literal code number.

## The `__private` surface

### Versioning rule

`#[doc(hidden)] pub mod __private` is not part of the public API for users, and is exempt from semantic versioning for them. It **is** versioned for the attribute front end (L2):

- `__private` items change only in minor releases (`0.x` to `0.x+1`, `1.x` to `1.x+1`), never in a patch release.
- Every change is listed in the changelog under a "Generated code" heading and in this file, and the L2 owner is told before the release is published.
- L2 states which define-err minor versions it supports and depends on `define-err = ">=X.Y, <X.Z"` accordingly.
- Items are added before old ones are removed; a removed item stays for at least one minor release, marked `#[deprecated]`.

### Items

| Item | Signature or form | Purpose | Since |
|---|---|---|---|
| `source_of!` | `source_of!(expr) -> Option<&(dyn Error + 'static)>` | source dispatch for `E: Error`, `Option<E>`, boxed `dyn Error` (three forms), `&'static dyn Error`; any other type fails with a plain-words `SourceField` error | 0.3 |
| `forward_source!` | `forward_source!(expr) -> Option<&(dyn Error + 'static)>` | the inner value's own `source()` for transparent variants, including boxed trait objects | 0.3 |
| `info_of!` | `info_of!(expr) -> Option<&dyn ErrorInfo>` | `ErrorInfo` forwarding for transparent variants | 0.3 |
| `Wrap`, dispatch traits | `struct Wrap<'a, T: ?Sized>(pub &'a T)`; `SourceViaError`, `SourceViaOption`, `SourceViaObject`, `SourceViaNothing`, `ForwardViaError`, `ForwardViaObject`, `InfoViaErrorInfo`, `InfoViaNothing` | the autoref dispatch behind the three macros above | 0.3 |
| `SourceField` | `trait SourceField` with `#[diagnostic::on_unimplemented]`, implemented for every `E: Error` | the compile error for a source field that is not an error | 0.3 |
| internal macros | `__define_err_block!`, `__define_err_type!`, `__define_err_enum!`, `__define_err_fields!`, `__define_err_emit!`, `__define_err_fmt!`, `__define_err_source!`, `__define_err_location!`, `__define_err_type_name!`, `__define_err_transparent!`, `__define_err_from!` | the parser and the emitter (`#[doc(hidden)]`, exported because `macro_rules!` helpers must be); the attribute front end calls none of them | 0.3 |
| `retry_after_of!` | `retry_after_of!(expr) -> Option<Duration>` | `#[retry_after]` fields of type `Duration` or `Option<Duration>` | 0.4 |
| `namespace` | `const fn namespace(text: &str, width: u8) -> Namespace` | the block's namespace; panics in const evaluation on invalid input | 0.4 |
| `code` | `const fn code(namespace: Namespace, number: u32) -> Code` | a code constant; panics in const evaluation when out of range | 0.4 |
| `retired_code` | `const fn retired_code(namespace: Namespace, number: u32) -> CodeInfo` | a retired catalog entry | 0.4 |
| `severity` | `const fn severity<const N: u8>() -> Severity` | a numeric severity, checked at compile time | 0.4 |
| `check_block` | `const fn check_block(namespace: Namespace, numbers: &[u32], retired: &[u32], replacements: &[(u32, u32)])` | duplicates and replacements; panics in const evaluation | 0.4 |
| `check_docs` | `const fn check_docs(template: &str)` | the docs template has a placeholder | 0.4 |
| `ProbeBuf` | `struct ProbeBuf`; `const fn new(code: Option<u64>, severity: u8, category: u8, kinds: u64, flags: u8, name: &str) -> ProbeBuf`; `const fn as_str(&self) -> &str` | builds the probe string at compile time | 0.4 |
| `forward_probe` | `fn forward_probe<'a>(inner: &'a str, own: &'static str) -> &'a str` | transparent variants | 0.4 |
| `platform_kinds` | module of `Kinds` constants `ERROR` to `TRIGGER` | glob-imported where `#[kinds]` is expanded | 0.4 |
| `named_levels` | module of `Severity` constants `TRACE` to `FATAL` | glob-imported where `#[severity(NAME)]` is expanded | 0.4 |
| `Site` | `struct Site`; `const fn new() -> Site` | once-per-site flag behind `warn!` and `deprecated!` | 0.5 |
| verb helpers | `fn notice_args(kinds, Arguments, Location)`, `fn warn_site(&'static Site, kinds, Arguments, Location)`, `fn trigger_args(kinds, Arguments, Location)`, `fn deprecated_site(&'static Site, Arguments, Location)` | the expansion targets of `notice!`, `warn!`, `trigger!`, `deprecated!` | 0.5 |

Public items that generated code uses directly: `Code`, `Namespace`, `Severity`, `Category`, `Kinds`, `Action`, `CodeInfo` and its `const` `with_*` methods, `CodeState`, `ErrorInfo`, `Location`. These follow semantic versioning like any public item.

## Differences between the front ends

| Topic | `define_errors!` | L2 attribute front end |
|---|---|---|
| `#[non_exhaustive]` | added unless `#[exhaustive]` | a derive cannot add attributes to the item it derives for; L2 leaves the user's attributes as written, so a thiserror migration is a find-and-replace |
| `Debug` | derived unless `#[manual_debug]` | the user derives it, as with thiserror |
| `{0}` placeholders | not supported: `macro_rules!` cannot look inside a string literal, so tuple fields are named in the definition (`Io(source: io::Error)`) | supported: the proc macro rewrites them |

Everything else in this file is identical between the two.
