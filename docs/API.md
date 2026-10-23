<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br><b>define-err</b><br>
    <sub><sup>API REFERENCE</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>API</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="./GENERATED-CODE.md" title="Generated-code contract"><b>GENERATED CODE</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="../dev/PLAN.md" title="Plan"><b>PLAN</b></a>
    </sup>
</div>
<br>

The complete public API of define-err 1.0, written at the `0.2.0` design lock. Every item carries the version it arrives in (**Since**) and its item id from [`../dev/PLAN.md`](../dev/PLAN.md) section 8; the published crate exposes only what is implemented, so an item marked `0.4` does not exist in `0.3`. Signatures here are the contract: a change after `0.2.0` is recorded in the decision log of the plan. What `define_errors!` expands to is in [`GENERATED-CODE.md`](./GENERATED-CODE.md).

Costs are estimates (marked [est.]) until they are measured in `0.6.0`; measured numbers then replace them, labelled as measured.

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>

## Table of contents

- [Setup](#setup)
  - [Features](#features)
  - [`no_std` and targets](#no_std-and-targets)
  - [The prelude](#the-prelude)
- [Conventions](#conventions)
  - [Tiers](#tiers)
  - [No panics, and the errors this crate returns](#no-panics-and-the-errors-this-crate-returns)
  - [Threads](#threads)
  - [What can never be observed](#what-can-never-be-observed)
- [Tier 1 on one screen](#tier-1-on-one-screen)
- [`define_errors!`](#define_errors)
  - [Shape of a block](#shape-of-a-block)
  - [Variants and fields](#variants-and-fields)
  - [Messages](#messages)
  - [Sources and conversions](#sources-and-conversions)
  - [Transparent wrapping](#transparent-wrapping)
  - [Structs](#structs)
  - [Type markers: `#[exhaustive]`, `#[manual_debug]`, passed-through attributes](#type-markers-exhaustive-manual_debug-passed-through-attributes)
  - [Codes: `namespace`, `#[code]`, `docs`, `retired`](#codes-namespace-code-docs-retired)
  - [Metadata attributes](#metadata-attributes)
  - [Locations](#locations)
  - [Compile errors](#compile-errors)
  - [Limits](#limits)
  - [From thiserror](#from-thiserror)
- [`ErrorInfo`](#errorinfo)
- [`Location` and `here`](#location-and-here)
- [Codes](#codes)
  - [`Namespace` and `namespace!`](#namespace-and-namespace)
  - [`Code` and `code!`](#code-and-code)
  - [Text form and parsing](#text-form-and-parsing)
  - [`CodeParseError`](#codeparseerror)
- [`Severity`](#severity)
  - [`Band`](#band)
  - [`LevelNames`](#levelnames)
- [`Category`](#category)
  - [The category table](#the-category-table)
  - [`RetryAdvice`](#retryadvice)
  - [`Action`](#action)
- [`Kinds`](#kinds)
- [Catalog](#catalog)
  - [`CodeInfo`](#codeinfo)
  - [`CodeState`](#codestate)
  - [`UrlDisplay`](#urldisplay)
  - [`catalog::find` and `catalog::check_unique`](#catalogfind-and-catalogcheck_unique)
- [Identity probe](#identity-probe)
  - [`identify`](#identify)
  - [`innermost_code`](#innermost_code)
  - [`Identity`](#identity)
- [`Secret<T>`](#secrett)
- [Reporting facade](#reporting-facade)
  - [`Reporter`](#reporter)
  - [`set_reporter` and `set_boxed_reporter`](#set_reporter-and-set_boxed_reporter)
  - [`InstallError` and `Slot`](#installerror-and-slot)
  - [`is_installed`, `reporter`, `reporter_installed_at`, `flush`](#is_installed-reporter-reporter_installed_at-flush)
  - [`report`, `report_with`, `report_record`](#report-report_with-report_record)
  - [`notice` and `notice!`](#notice-and-notice)
  - [`warn` and `warn!`](#warn-and-warn)
  - [`trigger` and `trigger!`](#trigger-and-trigger)
  - [`deprecated` and `deprecated!`](#deprecated-and-deprecated)
  - [`soft`](#soft)
  - [Filters: `WarnMode`, minimum severity, kinds mask](#filters-warnmode-minimum-severity-kinds-mask)
  - [`Event` and `Verb`](#event-and-verb)
  - [`stats` and `FacadeStats`](#stats-and-facadestats)
  - [Creation hook: `CreationHook`, `CreationInfo`, `Annotate`](#creation-hook-creationhook-creationinfo-annotate)
  - [Targets without compare-and-swap](#targets-without-compare-and-swap)
- [`ErrorRecord`](#errorrecord)
  - [Fields](#fields)
  - [Building a record](#building-a-record)
  - [Adding to a record](#adding-to-a-record)
  - [`RecordLimits`](#recordlimits)
  - [`ChainEntry` and `TrailEntry`](#chainentry-and-trailentry)
- [`testing`](#testing)
- [Crate root](#crate-root)
- [Item index](#item-index)

## Setup

```toml
[dependencies]
define-err = "0.1.0"
```

### Features

| Feature | Default | Implies | Brings |
|---|---|---|---|
| `std` | yes | `alloc` | `ErrorInfo` and `Category::from_io_kind` for `std::io::Error` (`0.4`); the facade's reentrancy guard and panic isolation (`0.5`) |
| `alloc` | through `std` | | `ErrorRecord` and its types, boxed `dyn Error` sources, `set_boxed_reporter`, `report_record` (`0.5`) |
| `testing` | no | `std` | `testing::capture`, `CapturedEvent`, `CaptureReporter` (`0.5`) |

No feature changes what `define_errors!` generates: the generated code is the same in every mode ([`GENERATED-CODE.md`](./GENERATED-CODE.md), rule 2).

### `no_std` and targets

The crate is `#![no_std]`. `default-features = false` gives `core` only: every macro, every type except `ErrorRecord`, and the facade. `features = ["alloc"]` adds the record and boxed sources. CI builds `thumbv7em-none-eabihf`, `thumbv6m-none-eabi`, `riscv32imac-unknown-none-elf`, and `wasm32-unknown-unknown`; on targets without compare-and-swap (`thumbv6m`) the facade is compiled out ([below](#targets-without-compare-and-swap)).

```toml
[dependencies]
define-err = { version = "0.1.0", default-features = false }
```

MSRV is 1.81, the first release with `core::error::Error`.

### The prelude

```rust
use define_err::prelude::*;
```

brings `define_errors!`, `ErrorInfo` (the trait behind `.code()`, `.category()`, `.public_message()`), `Code`, `Severity`, `Category`, `Kinds`, `Action`, `Secret`, and `Location`. Since `0.3` (ROOT-02; the metadata types join it in `0.4`).

The reporting verbs are not in the prelude. They are called by path, `define_err::report(&err)` and `define_err::warn!("...")`, so a reader always sees where they come from and a glob import never clashes with `log::warn!` or `tracing::warn!` (PLAN D-013). A project that wants a short name writes `use define_err as err;` and then `err::report(&e)`.

## Conventions

### Tiers

| Tier | What | Examples |
|---|---|---|
| 1 | the macro and the verbs: no configuration, no type parameters | `define_errors!`, `report`, `notice`, `warn`, `trigger`, `deprecated`, `set_reporter`, `err.code()`, `err.category().http_status()`, `Secret::new` |
| 2 | configuration and tables for the next 15% of uses | `set_warn_mode`, `set_min_severity`, `set_kinds_mask`, `report_with`, `RecordLimits`, `LevelNames`, `catalog::find`, `set_boxed_reporter`, `CodeInfo::new` |
| 3 | the full surface for other libraries | `identify`, `Code::from_u64`, `Kinds::from_bits`, `Event::new`, `ErrorRecord::new`, the creation hook, `__private` (generated code only) |

### No panics, and the errors this crate returns

No function in this crate panics at run time. Checks that can fail on literal input (a code number, a namespace, a severity, a kind bit) run at compile time, and a failure is a compile error. Their run-time twins return `Option` or `Result`: `Code::new`, `Namespace::try_new`, `Severity::new`, `Kinds::try_project`. The crate's own error types are `CodeParseError`, `InstallError`, and `DuplicateCode`; each is `#[non_exhaustive]` and its `Display` says what to do.

### Threads

`Code`, `Namespace`, `Severity`, `Category`, `Kinds`, `Action`, `RetryAdvice`, `CodeInfo`, `Identity`, and `ErrorRecord` are `Send + Sync`; all but `ErrorRecord` are `Copy`. `Secret<T>` is `Send` or `Sync` when `T` is. A generated error type is `Send` and `Sync` exactly when its fields are. `Event` borrows the error and the message for the length of one `Reporter::receive` call and is neither `Send` nor `Sync`; a reporter that hands work to another thread builds an `ErrorRecord`.

### What can never be observed

Errors are values in Rust; there is no throw event. The facade sees an error only when code calls a verb. It can never see an error discarded with `let _ =`, `.ok()`, or `unwrap_or`, an error handled inside third-party code, or a typed error that is built and never reported. Building an error never reports it.

## Tier 1 on one screen

```rust
use define_err::prelude::*;

define_errors! {
    namespace = "CFG";

    pub enum ConfigError {
        Missing { key: String } => "missing key {key}",

        #[code(1, help = "Fix the value in the file named in the message.")]
        #[category(InvalidArgument)]
        BadValue { key: String, file: String } => "bad value for {key} in {file}",

        Read(#[from] source: std::io::Error) => "cannot read the configuration: {source}",
    }
}

fn load(path: &str) -> Result<String, ConfigError> {
    let text = std::fs::read_to_string(path)?;          // From<io::Error>
    if text.is_empty() {
        define_err::warn("configuration file is empty; using defaults");
    }
    Ok(text)
}

fn main() {
    if let Err(err) = load("app.conf") {
        define_err::report(&err);                         // nothing happens until a reporter is installed
        eprintln!("{err}");
        std::process::exit(err.category().exit_code().into());
    }
}
```

A plain error type needs only the first line of the block's body; everything else is optional and costs nothing until it is used.

## `define_errors!`

Since `0.3` (DEF-01 to DEF-12, DEF-27); codes and metadata `0.4` (DEF-13 to DEF-23, DEF-25); `#[cfg]` on variants `0.5` (DEF-24).

```rust
#[macro_export]
macro_rules! define_errors { /* ... */ }
```

Declares one or more error types and generates, for each: the type (with `Debug`, and `#[non_exhaustive]` on enums), `Display` from the messages, `core::error::Error` with `source()`, the `From` conversions marked with `#[from]`, and an `ErrorInfo` impl. No proc macro, no build dependency, no run-time cost: constructing a value runs no code. The full expansion of every form is in [`GENERATED-CODE.md`](./GENERATED-CODE.md).

### Shape of a block

```text
define_errors! {
    [namespace = "<NS>" [, width = <1-8>];]        // needed only when a code is used
    [docs = "<URL template with {code}>";]          // optional
    [retired = [<n>, <n>, ...];]                    // optional; numbers never reused

    <type definition>+
}
```

A block may hold several types. They share the namespace, the documentation template, and the duplicate check, so two types in one block can never define the same code. Two blocks in one module are independent.

```rust
use define_err::define_errors;

define_errors! {
    /// Errors from the query layer.
    pub enum QueryError {
        Empty => "the query is empty",
    }

    /// Errors from the planner.
    pub enum PlanError {
        TooManyJoins { joins: u32 } => "query has {joins} joins",
    }
}
```

The macro works anywhere an item can be written, including inside a function body. Visibility (`pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, private) is passed through as written.

### Variants and fields

```text
<variant> :=  <attributes> Name                              => <message>
           |  <attributes> Name { field: Type, ... }        => <message>
           |  <attributes> Name ( binding: Type, ... )      => <message>
           |  <attributes> Name ( Type, ... )               => <message>   // unnamed: the message cannot use them
```

Variants are separated by commas; a trailing comma is optional. Empty forms `V {}` and `V()` are accepted. Doc comments and other attributes on variants and fields are passed through.

Tuple fields get a **binding name** in the definition (`ShortRead(page: u64, got: usize)`). The generated variant is still a tuple variant, `ShortRead(u64, usize)`, and is built as `PageError::ShortRead(7, 0)`; the name exists only so the message can refer to the field (see [Limits](#limits) for why `{0}` is not supported).

```rust
define_errors! {
    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",
        ShortRead(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
        Closed(u64) => "page store closed",
        InvalidSuperblock => "page 0 is not a valid allocator superblock",
    }
}

let err = PageError::ShortRead(7, 12);
assert_eq!(err.to_string(), "short read on page 7: got 12 bytes");
```

### Messages

```text
<message> := "<format string>"                           // fields by name: {page}, {page:?}, {crc:#010x}
           | ("<format string>", <argument>, ...)        // extra arguments; fields are in scope by name
           | transparent                                 // forward to the only field
```

- Fields are referenced by name with any format spec: `{page}`, `{page:?}`, `{crc:#010x}`, `{name:>8}`, `{ratio:.3}`.
- `{{` and `}}` write literal braces. Raw strings (`r"C:\path {p}"`) and any Unicode text work.
- Extra arguments go in parentheses after the string, exactly as in `write!`: positional (`("{}", diagnostic.format())`) or named (`min = crate::MIN_PAGE_SIZE`). Fields are references inside these expressions.
- The message is the whole `Display` output: no code, prefix, or decoration is added. Messages are required.

```rust
define_errors! {
    pub enum SizeError {
        InvalidPageSize { size: usize } => (
            "invalid page size {size}: must be a power of two in {min}..={max}",
            min = 512,
            max = 65_536,
        ),
        BadMagic { found: u32 } => "bad page magic {found:#010x}",
        Braces => "use {{ and }} for literal braces",
    }
}

assert_eq!(SizeError::BadMagic { found: 0xBEEF }.to_string(), "bad page magic 0x0000beef");
assert_eq!(SizeError::Braces.to_string(), "use { and } for literal braces");
```

### Sources and conversions

| Field marker | Effect |
|---|---|
| `#[source]` | this field is returned by `Error::source()` |
| a field named `source` | the same, without an attribute (thiserror's rule) |
| `#[from]` | generates `From<FieldType>` for the type, and implies `#[source]`; the variant may hold only this field and an optional `#[location]` field |

A source field may be any `E: Error + 'static`, an `Option<E>` (`source()` returns `None` when it is `None`), a `Box<dyn Error>`, `Box<dyn Error + Send>`, `Box<dyn Error + Send + Sync>`, or a `&'static dyn Error`. A variant has at most one source.

The generated `From::from` is `#[track_caller]`: when the variant has a `#[location]` field, a conversion through `?` records the line of the `?`.

```rust
use define_err::{define_errors, Location};

define_errors! {
    pub enum LoadError {
        Io(#[from] source: std::io::Error, #[location] at: Location) => "cannot read the file: {source}",
        Parse { line: u32, #[source] cause: core::num::ParseIntError } => "bad number on line {line}",
        Lost { source: Option<std::io::Error> } => "connection lost",
    }
}

fn read(path: &str) -> Result<String, LoadError> {
    Ok(std::fs::read_to_string(path)?)               // LoadError::Io, located at this line
}

use std::error::Error as _;
let err = LoadError::Lost { source: None };
assert!(err.source().is_none());
```

### Transparent wrapping

`=> transparent` forwards `Display`, `source()`, every `ErrorInfo` method, and the identity probe to the only field (besides an optional `#[location]`). The wrapper adds nothing a reader of the error can see, which is what a crate wants when it re-exports another crate's errors under its own enum.

```rust
define_errors! {
    pub enum AppError {
        Config(#[from] inner: ConfigError) => transparent,
        Other(#[from] inner: Box<dyn core::error::Error + Send + Sync>) => transparent,
    }
}

// err.code(), err.category(), err.to_string() are those of the ConfigError inside.
```

### Structs

```text
<vis> struct Name { field: Type, ... } => <message>;
<vis> struct Name ( binding: Type, ... ) => <message>;
<vis> struct Name => <message>;
```

A struct definition ends with `;`. Field visibility (`pub line: u32`) is passed through. Structs never get `#[non_exhaustive]`, because that would stop other crates from building them. Metadata attributes go on the struct.

```rust
define_errors! {
    /// A syntax error in a NOML file.
    pub struct SyntaxError { pub line: u32, pub column: u32 } => "syntax error at {line}:{column}";
    pub struct Timeout(pub u64) => "operation timed out";
    pub struct Cancelled => "operation cancelled";
}

assert_eq!(SyntaxError { line: 3, column: 9 }.to_string(), "syntax error at 3:9");
```

### Type markers: `#[exhaustive]`, `#[manual_debug]`, passed-through attributes

| On the type | Effect |
|---|---|
| (nothing) | enums get `#[non_exhaustive]`, so the enum can grow in a minor release without breaking callers; a `#[non_exhaustive]` the user already wrote is kept, never doubled |
| `#[exhaustive]` | enums only: no `#[non_exhaustive]` (for closed sets, and for published enums that were exhaustive before) |
| `#[manual_debug]` | no `Debug` derive; the user writes `Debug` (needed because `Error` requires it) |
| any other attribute (`#[derive(Clone, PartialEq)]`, `#[repr(..)]`, `#[allow(..)]`, doc comments) | passed through unchanged |

```rust
define_errors! {
    #[exhaustive]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Parity {
        Odd => "value is odd",
        Even => "value is even",
    }
}
```

### Codes: `namespace`, `#[code]`, `docs`, `retired`

```text
namespace = "<NS>" [, width = <1-8>];
docs = "<template>";
retired = [<n>, ...];
#[code(<n> [, name = <IDENT>], help = "<text>" [, title = "<text>"] [, deprecated = "<since>" [, replacement = <n>]])]
```

- `namespace`: 1 to 6 characters, `A` to `Z` and `0` to `9`, starting with a letter; lowercase is accepted and stored uppercase. `width` is the number of digits shown (default 4): `width = 5` displays `DIS-00001`. Width is presentation only: `DIS-1` and `DIS-00001` are the same code.
- `#[code(n, help = "...")]`: `n` is an integer literal from 0 to 16,777,215 (`1_000` and `0x2A` are accepted and display in decimal). Number 0 is the namespace's generic code. `help` is required: every code ships with what to do about it.
- `name = TIMEOUT` adds the associated constant `DbError::TIMEOUT: Code`. Without it the code is reached through `err.code()` or the catalog.
- `title` is the catalog title; it defaults to the variant name.
- `deprecated = "2.1", replacement = 10`: the code keeps working, its catalog state is `Deprecated`, and the named constant carries `#[deprecated]`, so the compiler warns where it is used.
- `docs`: a URL template; `{code}`, `{namespace}`, and `{number}` are replaced by [`CodeInfo::url`](#codeinfo). A template with none of them is a compile error.
- `retired = [9, 12]`: numbers that were used once and must never return. They appear in the catalog as `Retired` and take part in the duplicate check.

Every number in a block is checked at compile time: a duplicate (including a retired number) is a compile error, and rustc marks the second literal.

```rust
use define_err::prelude::*;

define_errors! {
    namespace = "DB";
    docs = "https://docs.hivedb.com/errors/{code}";
    retired = [9, 12];

    pub enum DbError {
        #[code(0, help = "Check the logs for the underlying cause.")]
        Unspecified => "database error",

        #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(4, help = "Use DB-0010 instead.", deprecated = "2.1", replacement = 10)]
        #[category(ResourceExhausted)]
        OldQuota => "quota exceeded",

        #[code(10, help = "Raise the tenant quota or wait for the window to reset.")]
        #[category(ResourceExhausted)]
        Quota { tenant: u64 } => "tenant {tenant} is over its quota",
    }
}

let err = DbError::Timeout { after_ms: 5000 };
assert_eq!(err.code(), Some(DbError::TIMEOUT));
assert_eq!(DbError::TIMEOUT.to_string(), "DB-0001");
assert_eq!(err.help(), Some("Raise pool.size or lower the load."));
assert_eq!(DbError::CATALOG.len(), 6);              // four codes plus two retired numbers
```

Five-digit codes, as Hive DB uses them:

```rust
define_errors! {
    namespace = "DIS", width = 5;

    pub enum DistroError {
        #[code(1, help = "Fix the named field in distro.noml.")]
        #[category(InvalidArgument)]
        #[action(FixConfig)]
        Config { field: String, reason: String } => "configuration error [{field}]: {reason}",
    }
}

let err = DistroError::Config { field: "port".into(), reason: "not a number".into() };
assert_eq!(err.code().unwrap().to_string(), "DIS-00001");
```

### Metadata attributes

Each is optional, goes on the variant (or the struct) it describes, and may appear in any order.

| Attribute | Value | Default when absent |
|---|---|---|
| `#[category(Name)]` | a [`Category`](#category) variant: `Unavailable`, `DataLoss`, ... | `Unknown` |
| `#[severity(NAME)]` or `#[severity(n)]` | `TRACE`, `DEBUG`, `INFO`, `NOTICE`, `WARNING`, `ERROR`, `CRITICAL`, `FATAL`, or 1 to 24 | the category's default severity |
| `#[kinds(A \| B \| ...)]` | platform kind names (`ERROR`, `SECURITY`, ...) and the project's own `Kinds` constants | `ERROR` |
| `#[public("text")]` | a message safe to show a client | the category's default public message |
| `#[action(Name)]` | an [`Action`](#action) variant | the category's default action |
| `#[retryable]` / `#[retryable(false)]` | override whether the error is transient | `category.is_transient()` |
| `#[retry_after(secs = n)]` / `#[retry_after(ms = n)]` | a fixed retry delay | `None` |

Field markers for metadata:

| Field marker | Field type | Effect |
|---|---|---|
| `#[retry_after]` | `core::time::Duration` or `Option<Duration>` | `retry_after()` returns this field |
| `#[location]` | `Location` | `location()` returns this field; `From` fills it |

```rust
use define_err::{prelude::*, Kinds};

pub const BILLING: Kinds = Kinds::project::<0>();

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        #[retry_after(secs = 2)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt { page: u64 } => "page {page} failed its checksum",

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

let err = DbError::Timeout { after_ms: 5000 };
assert_eq!(err.category().http_status(), 504);
assert_eq!(err.public_message(), "The database is busy. Try again shortly.");
assert!(err.is_retryable());
assert_eq!(err.retry_after(), Some(core::time::Duration::from_secs(2)));

let corrupt = DbError::Corrupt { page: 9 };
assert_eq!(corrupt.severity(), Severity::CRITICAL);
assert!(corrupt.kinds().contains(Kinds::SECURITY));
assert_eq!(corrupt.public_message(), "An internal error occurred.");   // never the private Display
```

### Locations

A `#[location]` field (type [`Location`](#location-and-here)) is filled with the caller's location by the generated `From` (so `?` records its own line), and by [`here()`](#location-and-here) when a variant is built by hand.

```rust
use define_err::{define_errors, here, Location, ErrorInfo};

define_errors! {
    pub enum WriteError {
        Full { #[location] at: Location } => "the log is full",
    }
}

let err = WriteError::Full { at: here() };
assert_eq!(err.location().unwrap().line(), line!() - 1);
```

### Compile errors

Misuse is a compile error that names the problem and the fix. Each has a `trybuild` case in `tests/ui/`.

| Mistake | Message (abridged) |
|---|---|
| `#[code]` without `help` | "`#[code]` needs `help = \"...\"`: every code ships with its explanation" |
| `#[code]` without a `namespace` header | "`#[code]` needs a `namespace = \"...\";` line at the top of the block" |
| unknown attribute in the metadata position | names it and lists the valid ones |
| number out of range, negative, a float, or not a literal | "a code number must be an integer literal from 0 to 16777215" |
| duplicate number, or a retired number reused | "duplicate code number in namespace DB", and rustc marks the second literal |
| invalid namespace (empty, longer than 6, a bad character, starts with a digit) | "namespace must be 1 to 6 letters or digits, starting with a letter" |
| width outside 1 to 8 | "width must be from 1 to 8 digits" |
| severity outside 1 to 24 or an unknown name | "severity must be from 1 to 24" / the unknown name is not found |
| unknown category, action, or kind name | the name is not found, at the name |
| project kind bit 32 or more | "Kinds::project: the bit must be below 32" |
| two source fields, two `#[location]` fields, two `#[retry_after]` fields | names the second one |
| `#[from]` with other fields | "a `#[from]` variant holds the source and, optionally, one `#[location]` field" |
| two `#[from]` of the same type in one enum | conflicting `From` impls |
| `transparent` with zero or two fields | "`transparent` needs exactly one field" |
| missing `=> message` | "every variant needs `=> \"message\"`" |
| a message naming a field that does not exist | rustc's format error at the literal |
| `{0}` in a message | rustc's "no arguments were given"; name the tuple field instead |
| docs template without a placeholder | "the docs template needs {code}, {namespace}, or {number}" |
| `replacement` pointing at itself or outside the block | "replacement must be another code defined in this block" |
| `#[exhaustive]` on a struct | "`#[exhaustive]` applies only to enums" |
| a variant or type name longer than 128 bytes | "a type or variant name is longer than 128 bytes" |

### Limits

| Limit | Why | Instead |
|---|---|---|
| no `{0}` positional placeholders | a declarative macro cannot look inside a string literal, and Rust rejects unused positional arguments | name the tuple field (`Io(source: io::Error) => "{source}"`), or use the attribute front end (L2) |
| no generic types or lifetimes on the error type | `macro_rules!` cannot parse bounds reliably | the attribute front end (L2); simple type parameters are planned after 1.0 (DEF-26) |
| no messages from doc comments | a declarative macro cannot trim a doc literal | the attribute front end (L2) |
| no constants for every code by default | a declarative macro cannot upper-case `Timeout` to `TIMEOUT` | `name = TIMEOUT` |
| no generated constructors | the same reason, and thiserror has none | build variants directly; `here()` for location fields |

An enum of 500 variants compiles at the default `recursion_limit` (NFR-6).

### From thiserror

| thiserror | `define_errors!` |
|---|---|
| `#[derive(Debug, Error)] pub enum E { ... }` | `define_errors! { pub enum E { ... } }` (keep other derives such as `Clone`) |
| `#[error("page {page_id} not found")] NotFound { page_id: u64 },` | `NotFound { page_id: u64 } => "page {page_id} not found",` |
| `#[error("page i/o error: {0}")] Io(#[from] io::Error),` | `Io(#[from] source: io::Error) => "page i/o error: {source}",` |
| `#[error("invalid usage: {0}")] InvalidUsage(String),` | `InvalidUsage(detail: String) => "invalid usage: {detail}",` |
| `#[error("{}", .diagnostic.format())] Diag { diagnostic: D },` | `Diag { diagnostic: D } => ("{}", diagnostic.format()),` |
| `#[error("size {size} not in {min}..={max}", min = A, max = B)]` | `=> ("size {size} not in {min}..={max}", min = A, max = B)` |
| `#[error(transparent)] Other(#[from] OtherError),` | `Other(#[from] inner: OtherError) => transparent,` |
| `#[source] cause: E`, or a field named `source` | unchanged |
| an enum that must stay exhaustive | add `#[exhaustive]` |

`Display` output and `source()` chains are byte-identical to the thiserror version; the conformance suite checks it against thiserror itself.

## `ErrorInfo`

Since `0.3` with defaults (INF-01); all methods generated from `0.4`.

```rust
pub trait ErrorInfo: core::error::Error {
    fn code(&self) -> Option<Code> { None }
    fn code_info(&self) -> Option<&'static CodeInfo> { None }
    fn category(&self) -> Category { Category::Unknown }
    fn severity(&self) -> Severity { self.category().default_severity() }
    fn kinds(&self) -> Kinds { Kinds::ERROR }
    fn public_message(&self) -> &'static str { self.category().default_public_message() }
    fn help(&self) -> Option<&'static str> { None }
    fn action(&self) -> Action { self.category().default_action() }
    fn is_retryable(&self) -> bool { self.category().is_transient() }
    fn retry_after(&self) -> Option<core::time::Duration> { None }
    fn location(&self) -> Option<Location> { None }
    fn type_name(&self) -> &'static str { core::any::type_name::<Self>() }
}
```

The one trait through which every part of the error collection reads an error's metadata. Every generated type implements it; a hand-written error type can implement it with `impl ErrorInfo for MyError {}` and override what it needs. The trait is object safe, so `&dyn ErrorInfo` works.

| Method | Returns |
|---|---|
| `code` | the stable code, if the variant has one |
| `code_info` | the catalog entry of that code: title, help, docs URL, state |
| `category` | the [`Category`](#category): HTTP status, gRPC code, exit code, retry rule |
| `severity` | the [`Severity`](#severity) (L7 may remap it by configuration, outside this crate) |
| `kinds` | the [`Kinds`](#kinds) set, for filters |
| `public_message` | text a client may see; never derived from `Display` |
| `help` | what to do, from the code's `help` |
| `action` | who acts on it ([`Action`](#action)) |
| `is_retryable` | whether the condition may clear on its own; only idempotent operations may retry automatically ([`Category::may_retry_automatically`](#category)) |
| `retry_after` | how long to wait before a retry, when known |
| `location` | where the error was built or converted, when the variant records it |
| `type_name` | `"Type::Variant"` for generated types (OpenTelemetry `exception.type`) |

No method allocates or locks; each is a `match` returning constants or copying a field [est. under 1 ns].

```rust
use define_err::prelude::*;

fn respond(err: &dyn ErrorInfo) -> (u16, &'static str) {
    (err.category().http_status(), err.public_message())
}
```

With `std`, `std::io::Error` implements `ErrorInfo`: `category()` is [`Category::from_io_kind`](#category) of its kind, and `kinds()` adds `RESOURCE` for storage kinds (`StorageFull`, `QuotaExceeded`, `OutOfMemory` where they exist). Since `0.4` (INF-04).

```rust
use define_err::prelude::*;

let err = std::io::Error::from(std::io::ErrorKind::NotFound);
assert_eq!(err.category(), Category::NotFound);
assert_eq!(err.category().http_status(), 404);
```

## `Location` and `here`

Since `0.3` (INF-07).

```rust
pub type Location = &'static core::panic::Location<'static>;

#[track_caller]
pub fn here() -> Location;
```

`Location` is the type of `#[location]` fields: a file, line, and column, 8 bytes on 64-bit targets. `here()` returns the location of its caller, for building a located variant by hand. A `#[track_caller]` function that calls `here()` gets its own caller's location.

```rust
let at = define_err::here();
assert_eq!(at.file(), file!());
```

## Codes

A `Code` is a namespace plus a number, packed into one `u64`, displayed `DB-0042`. It needs no registry: two nodes, two processes, or two versions agree on a code by its value alone. The layout and its golden values are in [`tests/golden/codes.txt`](../tests/golden/codes.txt) and frozen at 1.0:

```text
 bit  63                     32 31  29 28   24 23                       0
     [ namespace: 32 bits      ][width][reserved][ number: 24 bits        ]
       base-37 text, 1-6 chars   3 bits   5 bits   0 - 16,777,215
```

The namespace field is the text read as a base-37 number (`'0'`-`'9'` are 1 to 10, `'A'`-`'Z'` are 11 to 36, 0 pads after the last character), so the text comes back from the integer alone and numeric order is alphabetical order. Because the first character is a letter, the value is never zero, and `Option<Code>` is 8 bytes.

### `Namespace` and `namespace!`

Since `0.4` (CODE-01, CODE-02).

```rust
#[derive(Clone, Copy)]
pub struct Namespace { /* private */ }

impl PartialEq for Namespace;  impl Eq for Namespace;      // by text; the width is ignored
impl PartialOrd for Namespace; impl Ord for Namespace;     // alphabetical
impl core::hash::Hash for Namespace;                       // by text
impl core::fmt::Display for Namespace;                     // "DB"
impl core::fmt::Debug for Namespace;                       // "Namespace(DB, width 4)"

impl Namespace {
    pub const fn try_new(text: &str) -> Result<Namespace, CodeParseError>;
    pub const fn try_with_width(text: &str, width: u8) -> Result<Namespace, CodeParseError>;
    pub fn as_str(&self) -> NamespaceStr;
    pub const fn width(self) -> u8;
    pub const fn code(self, number: u32) -> Option<Code>;
}

#[macro_export]
macro_rules! namespace {
    ($text:literal) => { /* const Namespace, checked at compile time */ };
    ($text:literal, width = $width:literal) => { /* ... */ };
}

pub struct NamespaceStr { /* up to 6 bytes on the stack */ }
impl core::ops::Deref for NamespaceStr { type Target = str; }
impl core::fmt::Display for NamespaceStr;
```

- `try_new` accepts 1 to 6 characters, `A` to `Z`, `a` to `z` (stored uppercase), and `0` to `9`, starting with a letter; width 4. Errors: `CodeParseError::BadNamespace`.
- `try_with_width` also takes the display width, 1 to 8. Errors: `BadNamespace`, `BadWidth`.
- `as_str` returns the text without allocating.
- `code` returns `None` when `number` is above `Code::MAX_NUMBER`.
- `namespace!("DB")` is the compile-time form: an invalid literal does not compile.

Equality, ordering, and hashing compare the text only; the width is presentation.

```rust
use define_err::{namespace, Namespace};

const DB: Namespace = namespace!("DB");
const DIS: Namespace = namespace!("DIS", width = 5);

assert_eq!(&*DB.as_str(), "DB");
assert_eq!(DIS.width(), 5);
assert_eq!(Namespace::try_new("db").unwrap(), DB);
assert!(Namespace::try_new("1DB").is_err());
assert_eq!(DIS.code(1).unwrap().to_string(), "DIS-00001");
```

### `Code` and `code!`

Since `0.4` (CODE-03 to CODE-08).

```rust
#[derive(Clone, Copy)]
pub struct Code { /* NonZeroU64 */ }

impl Code {
    pub const MAX_NUMBER: u32 = 16_777_215;
    pub const fn new(namespace: Namespace, number: u32) -> Option<Code>;
    pub const fn namespace(self) -> Namespace;
    pub const fn number(self) -> u32;
    pub const fn is_generic(self) -> bool;
    pub const fn as_u64(self) -> u64;
    pub const fn identity(self) -> u64;
    pub const fn from_u64(raw: u64) -> Option<Code>;
    pub fn parse(text: &str) -> Result<Code, CodeParseError>;
}

impl PartialEq for Code;  impl Eq for Code;            // by identity()
impl PartialOrd for Code; impl Ord for Code;           // by identity(): namespace text, then number
impl core::hash::Hash for Code;                        // by identity()
impl core::fmt::Display for Code;                      // "DB-0042"
impl core::fmt::Debug for Code;                        // "Code(DB-0042)"
impl core::str::FromStr for Code;                      // Err = CodeParseError

#[macro_export]
macro_rules! code {
    ($namespace:literal, $number:literal) => { /* const Code, checked */ };
    ($namespace:path, $number:literal) => { /* a Namespace constant */ };
}
```

| Method | Behaviour | Cost [est.] |
|---|---|---|
| `new` | `None` when `number > MAX_NUMBER` | under 1 ns |
| `namespace`, `number` | the parts | shifts |
| `is_generic` | `number == 0`, the namespace's generic code | |
| `as_u64` | the full value, width included, for storage and the wire | |
| `identity` | the value with the width bits cleared: what `Eq`, `Ord`, and `Hash` use | |
| `from_u64` | `None` for anything not exactly valid: reserved bits set, a namespace field out of range or with padding before a character, zero | under 2 ns |
| `parse` | the text form below | under 15 ns |

No method allocates; `Display` writes into the formatter directly [est. under 20 ns].

```rust
use define_err::{code, Code};

const TIMEOUT: Code = code!("DB", 1);

assert_eq!(TIMEOUT.to_string(), "DB-0001");
assert_eq!(TIMEOUT.number(), 1);
assert_eq!(Code::from_u64(TIMEOUT.as_u64()), Some(TIMEOUT));
assert_eq!(Code::parse("db-1"), Ok(TIMEOUT));              // same identity, width 1
assert_eq!(core::mem::size_of::<Option<Code>>(), 8);
```

Sorting and storing codes from another node:

```rust
let mut seen = vec![Code::parse("DB-0010")?, Code::parse("DB-0002")?, Code::parse("DBA-0001")?];
seen.sort();
assert_eq!(seen[0].to_string(), "DB-0002");

let raw: u64 = seen[0].as_u64();                            // into a log record or a message
let back = Code::from_u64(raw).ok_or("corrupt code")?;     // validated on the way back
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Text form and parsing

`Display` writes the namespace, `-`, and the number zero-padded to the namespace's width; a number wider than the width prints in full (`DB-12345` with width 4).

`Code::parse` accepts exactly `^[A-Za-z][A-Za-z0-9]{0,5}-[0-9]{1,8}$` (at most 15 bytes, checked before any other work); the parsed width is the number of digits. It rejects surrounding whitespace, `+` and `-` signs in the number, `0x`, Unicode hyphens and digits, a second separator, and any byte outside ASCII.

```rust
use define_err::Code;

assert!(Code::parse("DB-0042").is_ok());
assert_eq!(Code::parse("DB-42"), Code::parse("DB-00000042"));  // equal identity
assert!(Code::parse(" DB-1").is_err());
assert!(Code::parse("DB-0x2A").is_err());
assert!(Code::parse("DB-16777216").is_err());
```

### `CodeParseError`

Since `0.4` (CODE-09).

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum CodeParseError {
    Empty,
    TooLong,
    MissingSeparator,
    BadNamespace,
    BadWidth,
    BadNumber,
    NumberTooLarge,
    ReservedBits,
}
impl core::fmt::Display for CodeParseError;
impl core::error::Error for CodeParseError;
```

| Variant | Meaning | What to do |
|---|---|---|
| `Empty` | the text is empty | pass a code such as `DB-0042` |
| `TooLong` | more than 15 bytes | a code is at most six namespace characters, `-`, and eight digits |
| `MissingSeparator` | no `-` between namespace and number | write `NS-0042` |
| `BadNamespace` | the namespace is empty, too long, starts with a digit, or has a character outside `A`-`Z`, `0`-`9` | use 1 to 6 letters or digits, starting with a letter |
| `BadWidth` | a width outside 1 to 8 | use 1 to 8 |
| `BadNumber` | the number is empty, not decimal ASCII digits, or longer than eight digits | write decimal digits, for example `DB-0042` |
| `NumberTooLarge` | above 16,777,215 | codes are numbered 0 to 16,777,215 |
| `ReservedBits` | (`from_u64` callers decoding by hand) reserved bits are set | the value is not a code from this version |

`Display` names the input class and the fix, for example "code text is not valid: the number must be decimal digits, for example DB-0042". The text never echoes the input, so a hostile string cannot reach a log through this error.

## `Severity`

Since `0.4` (SEV-01 to SEV-03).

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Severity { /* NonZeroU8, 1 to 24 */ }

impl Severity {
    pub const TRACE: Severity;     // 1
    pub const DEBUG: Severity;     // 5
    pub const INFO: Severity;      // 9
    pub const NOTICE: Severity;    // 10
    pub const WARNING: Severity;   // 13
    pub const ERROR: Severity;     // 17
    pub const CRITICAL: Severity;  // 20
    pub const FATAL: Severity;     // 21

    pub const fn at<const N: u8>() -> Severity;
    pub const fn new(value: u8) -> Option<Severity>;
    pub const fn saturating(value: u8) -> Severity;
    pub const fn get(self) -> u8;
    pub const fn band(self) -> Band;
    pub const fn otel_name(self) -> &'static str;
    pub const fn label(self) -> Option<&'static str>;
    pub fn parse(text: &str) -> Option<Severity>;
}
impl core::fmt::Display for Severity;
```

The OpenTelemetry SeverityNumber scale: 1 to 24, higher is more severe, compared as a number. Bands: TRACE 1-4, DEBUG 5-8, INFO 9-12, WARN 13-16, ERROR 17-20, FATAL 21-24. "Unspecified" is not a severity; use `Option<Severity>` (1 byte).

| Item | Behaviour |
|---|---|
| `at::<N>()` | a compile-time checked severity: `Severity::at::<25>()` does not compile |
| `new` | `None` outside 1 to 24 |
| `saturating` | 0 becomes 1, above 24 becomes 24 |
| `otel_name` | the OpenTelemetry short name: `ERROR`, `ERROR2`, ..., `ERROR4` |
| `label` | the portfolio label for the eight named values, else `None` |
| `Display` | the label if there is one, else the short name: 10 is `NOTICE`, 11 is `INFO3` |
| `parse` | labels, short names, and `"1"` to `"24"`, case-insensitive; `"warn"` and `"WARNING"` are both 13 |

The full table is in [`tests/golden/severity.txt`](../tests/golden/severity.txt).

```rust
use define_err::Severity;

assert!(Severity::CRITICAL > Severity::ERROR);
assert_eq!(Severity::new(20), Some(Severity::CRITICAL));
assert_eq!(Severity::CRITICAL.otel_name(), "ERROR4");
assert_eq!(Severity::parse("warn"), Some(Severity::WARNING));
assert_eq!(Severity::at::<22>().to_string(), "FATAL2");
```

### `Band`

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Band { Trace, Debug, Info, Warn, Error, Fatal }
```

The OpenTelemetry band a severity falls in, for sinks that only know six levels (`log` levels, syslog). Since `0.4`.

### `LevelNames`

Since `0.4` (SEV-04).

```rust
#[derive(Clone, Copy, Debug)]
pub struct LevelNames { /* &'static [(&'static str, Severity)] */ }

impl LevelNames {
    pub const fn new(pairs: &'static [(&'static str, Severity)]) -> LevelNames;
    pub fn name_of(&self, severity: Severity) -> Option<&'static str>;
    pub fn severity_of(&self, name: &str) -> Option<Severity>;
}
```

A project's own names for severities. `name_of` returns the exact match, else the name of the nearest lower value in the table, else `None`; `severity_of` matches names case-insensitively. The table is a plain slice, so it can be a `const`; L7 can build one from configuration.

```rust
use define_err::{LevelNames, Severity};

const LEVELS: LevelNames = LevelNames::new(&[
    ("minor", Severity::WARNING),
    ("major", Severity::ERROR),
    ("billing-alert", Severity::at::<22>()),
]);

assert_eq!(LEVELS.name_of(Severity::CRITICAL), Some("major"));     // nearest lower
assert_eq!(LEVELS.severity_of("BILLING-ALERT"), Some(Severity::at::<22>()));
```

## `Category`

Since `0.4` (CAT-01 to CAT-07).

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Category {
    Cancelled, Unknown, InvalidArgument, DeadlineExceeded, NotFound, AlreadyExists,
    PermissionDenied, ResourceExhausted, FailedPrecondition, Aborted, OutOfRange,
    Unimplemented, Internal, Unavailable, DataLoss, Unauthenticated,
}

impl Category {
    pub const ALL: [Category; 16];
    pub const fn http_status(self) -> u16;
    pub const fn grpc_code(self) -> u8;
    pub const fn from_grpc(code: u8) -> Option<Category>;
    pub const fn exit_code(self) -> u8;
    pub const fn is_transient(self) -> bool;
    pub const fn retry_advice(self) -> RetryAdvice;
    pub const fn may_retry_automatically(self, idempotent: bool) -> bool;
    pub const fn default_severity(self) -> Severity;
    pub const fn default_action(self) -> Action;
    pub const fn default_public_message(self) -> &'static str;
    pub const fn name(self) -> &'static str;
    pub fn parse(text: &str) -> Option<Category>;
    #[cfg(feature = "std")]
    pub fn from_io_kind(kind: std::io::ErrorKind) -> Category;
}
impl core::fmt::Display for Category;     // name(): "deadline_exceeded"
```

The gRPC canonical status codes without `OK`. A category decides how every transport refuses: a gateway turns it into an HTTP status, a gRPC service into a status code, a CLI into an exit code, without a table per error code.

| Item | Behaviour |
|---|---|
| `http_status` | Google's published `google.rpc.Code` HTTP mapping |
| `grpc_code`, `from_grpc` | the gRPC number; `from_grpc(0)` (OK) and above 16 are `None` |
| `exit_code` | `sysexits(3)`; `Cancelled` is 130 (the shell's 128 + SIGINT) |
| `is_transient` | the condition may clear on its own (`DeadlineExceeded`, `ResourceExhausted`, `Aborted`, `Unavailable`) |
| `retry_advice` | Google AIP-194's narrower rule for automatic retries |
| `may_retry_automatically` | `idempotent && self == Unavailable`: only an operation whose repeat causes no unintended change retries on its own |
| `default_severity`, `default_action`, `default_public_message` | what a variant gets when it does not say |
| `name`, `parse` | `snake_case` name; `parse` also accepts `SCREAMING_CASE` and `CamelCase` |
| `from_io_kind` | `NotFound`, `PermissionDenied`, `AlreadyExists`, `InvalidInput`, `InvalidData`, `TimedOut`, `WouldBlock`, `Interrupted`, `OutOfMemory`, `Unsupported`, and the network kinds are mapped; any other kind is `Unknown` |

```rust
use define_err::Category;

let c = Category::Unavailable;
assert_eq!(c.http_status(), 503);
assert_eq!(c.exit_code(), 69);
assert!(c.may_retry_automatically(true));
assert!(!c.may_retry_automatically(false));
assert_eq!(Category::from_grpc(4), Some(Category::DeadlineExceeded));
assert_eq!(Category::parse("DEADLINE_EXCEEDED"), Some(Category::DeadlineExceeded));
```

### The category table

Frozen at 1.0; golden file [`tests/golden/categories.txt`](../tests/golden/categories.txt).

| Category | gRPC | HTTP | Exit | Transient | Retry advice | Default severity | Default action | Default public message |
|---|---|---|---|---|---|---|---|---|
| `Cancelled` | 1 | 499 | 130 | no | `No` | DEBUG (5) | `None` | "The request was cancelled." |
| `Unknown` | 2 | 500 | 70 | no | `No` | ERROR (17) | `Operator` | "An unexpected error occurred." |
| `InvalidArgument` | 3 | 400 | 64 | no | `No` | WARNING (13) | `FixRequest` | "The request is not valid." |
| `DeadlineExceeded` | 4 | 504 | 75 | yes | `No` | ERROR (17) | `Retry` | "The request took too long. Try again." |
| `NotFound` | 5 | 404 | 66 | no | `No` | WARNING (13) | `FixRequest` | "The requested item was not found." |
| `AlreadyExists` | 6 | 409 | 73 | no | `No` | WARNING (13) | `FixRequest` | "The item already exists." |
| `PermissionDenied` | 7 | 403 | 77 | no | `No` | WARNING (13) | `FixRequest` | "You do not have permission to do this." |
| `ResourceExhausted` | 8 | 429 | 75 | yes | `Later` | ERROR (17) | `Retry` | "Too many requests. Try again later." |
| `FailedPrecondition` | 9 | 400 | 78 | no | `AfterFix` | WARNING (13) | `FixRequest` | "The request cannot be done in the current state." |
| `Aborted` | 10 | 409 | 75 | yes | `RestartTransaction` | WARNING (13) | `Retry` | "The request conflicted with another. Try again." |
| `OutOfRange` | 11 | 400 | 65 | no | `No` | WARNING (13) | `FixRequest` | "A value is out of range." |
| `Unimplemented` | 12 | 501 | 70 | no | `No` | ERROR (17) | `Operator` | "This operation is not supported." |
| `Internal` | 13 | 500 | 70 | no | `No` | ERROR (17) | `Operator` | "An internal error occurred." |
| `Unavailable` | 14 | 503 | 69 | yes | `Yes` | ERROR (17) | `Retry` | "The service is unavailable. Try again later." |
| `DataLoss` | 15 | 500 | 74 | no | `No` | CRITICAL (20) | `PageOnCall` | "An internal error occurred." |
| `Unauthenticated` | 16 | 401 | 77 | no | `No` | WARNING (13) | `FixRequest` | "Authentication is required." |

`DataLoss` has the same public text as `Internal` on purpose: a client is never told through a generic path that data was lost.

### `RetryAdvice`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum RetryAdvice { No, Yes, Later, RestartTransaction, AfterFix }
impl core::fmt::Display for RetryAdvice;
```

| Variant | Meaning |
|---|---|
| `No` | do not retry this operation automatically |
| `Yes` | retry, with backoff, if the operation is idempotent |
| `Later` | wait (honour `retry_after`), then retry |
| `RestartTransaction` | retry the whole transaction at a higher level, not the single call |
| `AfterFix` | retry only after the state that caused the failure is fixed |

Since `0.4` (CAT-05).

### `Action`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Action { None, Retry, FixRequest, FixConfig, Operator, PageOnCall }
impl core::fmt::Display for Action;
```

Who acts on the error: nobody, the caller retrying, the caller fixing the request, an operator fixing configuration, an operator investigating, or someone paged now. Since `0.4` (CAT-06).

## `Kinds`

Since `0.4` (KND-01 to KND-05).

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Kinds { /* u64 */ }

impl Kinds {
    pub const NONE: Kinds;      pub const ALL: Kinds;
    pub const ERROR: Kinds;     pub const WARNING: Kinds;   pub const NOTICE: Kinds;
    pub const DEPRECATED: Kinds; pub const SECURITY: Kinds; pub const FUTURE: Kinds;
    pub const RUNTIME: Kinds;   pub const RESOURCE: Kinds;  pub const CONFIG: Kinds;
    pub const USER: Kinds;      pub const PANIC: Kinds;     pub const AUDIT: Kinds;
    pub const TRIGGER: Kinds;
    pub const PLATFORM: Kinds;  // bits 0-31
    pub const PROJECT: Kinds;   // bits 32-63

    pub const fn project<const N: u8>() -> Kinds;
    pub const fn try_project(bit: u8) -> Option<Kinds>;
    pub const fn contains(self, other: Kinds) -> bool;
    pub const fn intersects(self, other: Kinds) -> bool;
    pub const fn is_empty(self) -> bool;
    pub const fn union(self, other: Kinds) -> Kinds;
    pub const fn bits(self) -> u64;
    pub const fn from_bits(bits: u64) -> Kinds;
    pub fn iter(self) -> KindsIter;
}
impl core::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign, Not, Sub} for Kinds;
impl core::fmt::Display for Kinds;   // "ERROR|SECURITY|project(3)", "NONE" when empty
impl core::fmt::Debug for Kinds;     // "Kinds(ERROR|SECURITY|project(3))"

pub struct KindsIter;                // Iterator<Item = Kinds>, one bit at a time, low to high
```

Kinds are a set (PHP `E_*` style): each kind is one bit, masks combine with `|`. Severity is an order and answers "warnings and above"; kinds answer "security or billing, at any level". Filters need both.

| Bit | Constant | Meaning |
|---|---|---|
| 0 | `ERROR` | an error (default for reported errors) |
| 1 | `WARNING` | a warning (`warn`) |
| 2 | `NOTICE` | an informational notice (`notice`) |
| 3 | `DEPRECATED` | use of something deprecated |
| 4 | `SECURITY` | security relevant; delivered to security listeners |
| 5 | `FUTURE` | behaviour that will change in a later version |
| 6 | `RUNTIME` | a run-time condition |
| 7 | `RESOURCE` | resource pressure (memory, handles, quota, disk) |
| 8 | `CONFIG` | a configuration problem |
| 9 | `USER` | caused by the caller's input |
| 10 | `PANIC` | a caught panic (set by the dynamic-error and service libraries, not by users) |
| 11 | `AUDIT` | an action that must reach the audit trail |
| 12 | `TRIGGER` | raised on purpose by `trigger` |
| 13-31 | reserved | platform kinds added in minor releases; never reassigned |
| 32-63 | project | `Kinds::project::<0>()` to `Kinds::project::<31>()` |

- `project::<N>()` fails to compile unless `N < 32`, so a project can never take a platform bit; `try_project` is the run-time form.
- `contains(other)` is true when every bit of `other` is set; `intersects(other)` when any is. `contains(NONE)` is always true and `intersects(NONE)` always false. PHP's `$mask & E_WARNING` reads like `intersects`.
- `union` is the `const` form of `|`, for constants built from several kinds.
- `from_bits` accepts any bits; it is for decoding stored values (Tier 3).
- `Display` prints reserved platform bits as `bit(13)`.

```rust
use define_err::Kinds;

pub const BILLING: Kinds = Kinds::project::<0>();
const SECURITY_OR_BILLING: Kinds = Kinds::SECURITY.union(BILLING);

let kinds = Kinds::ERROR | Kinds::SECURITY;
assert!(kinds.contains(Kinds::SECURITY));
assert!(kinds.intersects(SECURITY_OR_BILLING));
assert!(!kinds.contains(SECURITY_OR_BILLING));
assert_eq!(kinds.to_string(), "ERROR|SECURITY");
assert_eq!(BILLING.to_string(), "project(0)");
```

## Catalog

Since `0.4` (INF-02, INF-03, INF-05, INF-06; DEF-23 for the generated `CATALOG`).

Every type whose block has a namespace gets `pub const CATALOG: &'static [CodeInfo]`: one entry per code, plus the block's retired numbers on the first type. The catalog is static data built at compile time; there is no registry and nothing to register.

### `CodeInfo`

```rust
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct CodeInfo {
    pub code: Code,
    pub title: &'static str,
    pub help: &'static str,
    pub docs: Option<&'static str>,
    pub severity: Severity,
    pub category: Category,
    pub kinds: Kinds,
    pub action: Action,
    pub public: &'static str,
    pub state: CodeState,
    pub type_name: &'static str,
}

impl CodeInfo {
    pub const fn new(code: Code, title: &'static str, help: &'static str) -> CodeInfo;
    pub const fn with_docs(self, template: &'static str) -> CodeInfo;
    pub const fn with_severity(self, severity: Severity) -> CodeInfo;
    pub const fn with_category(self, category: Category) -> CodeInfo;
    pub const fn with_kinds(self, kinds: Kinds) -> CodeInfo;
    pub const fn with_action(self, action: Action) -> CodeInfo;
    pub const fn with_public(self, text: &'static str) -> CodeInfo;
    pub const fn with_state(self, state: CodeState) -> CodeInfo;
    pub const fn with_type_name(self, name: &'static str) -> CodeInfo;
    pub fn url(&self) -> Option<UrlDisplay<'_>>;
    pub const fn is_active(&self) -> bool;
}
```

The fields are public to read; the struct is `#[non_exhaustive]`, so entries are built with `new` and the `const` `with_*` methods (generated code does exactly that). `new` starts from category `Unknown` and its defaults, kinds `ERROR`, no docs, state `Active`, and an empty type name. `is_active` is false for deprecated and retired codes.

```rust
use define_err::prelude::*;

for info in DbError::CATALOG {
    println!("{} {}: {}", info.code, info.title, info.help);
    if let Some(url) = info.url() {
        println!("    see {url}");
    }
}
```

A hand-written catalog, for codes that are not tied to a type (protocol status codes, for example):

```rust
use define_err::{code, Category, CodeInfo};

pub const PROTOCOL: &[CodeInfo] = &[
    CodeInfo::new(code!("PROTO", 1), "Frame too large", "Lower max_frame or raise the peer's limit.")
        .with_category(Category::ResourceExhausted),
];
```

### `CodeState`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum CodeState {
    Active,
    Deprecated { since: &'static str, replacement: Option<Code> },
    Retired,
}
```

`Deprecated` codes work as before; L7 may warn when one is reported. `Retired` entries exist only to keep the number from coming back; no variant has one.

### `UrlDisplay`

```rust
pub struct UrlDisplay<'a> { /* borrows the CodeInfo */ }
impl core::fmt::Display for UrlDisplay<'_>;
```

Writes the docs template with every `{code}` replaced by the code's text (`DB-0042`), `{namespace}` by the namespace, and `{number}` by the number without padding. Nothing is allocated; `url()` returns `None` when the block has no `docs` line.

```rust
let info = DbError::Timeout { after_ms: 1 }.code_info().unwrap();
assert_eq!(info.url().unwrap().to_string(), "https://docs.hivedb.com/errors/DB-0001");
```

### `catalog::find` and `catalog::check_unique`

```rust
pub mod catalog {
    pub fn find(catalogs: &[&'static [CodeInfo]], code: Code) -> Option<&'static CodeInfo>;
    pub fn check_unique(catalogs: &[&'static [CodeInfo]]) -> Result<(), DuplicateCode>;
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct DuplicateCode { pub code: Code, pub first: &'static str, pub second: &'static str }
impl core::fmt::Display for DuplicateCode;
impl core::error::Error for DuplicateCode;
```

- `find` searches several catalogs (one per crate, or per type) in order and returns the first entry with that code [est. under 1 us for 1,000 codes].
- `check_unique` is the boot-time check across crates: two crates that picked the same namespace and number are a configuration error. Retired entries count as taken. `first` and `second` are the type names of the two entries (empty for a retired entry). `Display`: "code DB-0001 is defined twice: by DbError::Timeout and by StoreError::Busy; give one crate its own namespace".

The compile-time duplicate check covers one block; this covers everything a binary links.

```rust
use define_err::catalog;

let all: &[&[define_err::CodeInfo]] = &[DbError::CATALOG, PlanError::CATALOG, other_crate::CATALOG];
catalog::check_unique(all)?;                                // at boot: fail closed on a collision
let info = catalog::find(all, code).ok_or("unknown code")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Identity probe

Since `0.4` (PRB-01 to PRB-03; DEF-13 generates the probe).

The problem it solves: libraries that hold errors as `&dyn Error` (a dynamic error type, a renderer, the error service) cannot ask a trait object for its `ErrorInfo`; stable Rust cannot cast between trait objects. Generated code answers through `Error::description`, which every `dyn Error` has, with a fixed string built at compile time. The format is in [`GENERATED-CODE.md`](./GENERATED-CODE.md#the-identity-probe) and frozen at 1.0.

The probe carries only per-variant constants. Per-value data (a location, a `retry_after` field) is reached through `ErrorInfo` on the concrete type. Codes identify an error; they do not authenticate it: a foreign type could return a forged probe, and the error service checks a probe's type name against the catalog when that matters.

### `identify`

```rust
pub fn identify<'a>(error: &'a (dyn core::error::Error + 'static)) -> Option<Identity<'a>>;
```

Reads the identity of an error generated by `define_errors!` or the attribute front end. Returns `None` for any other error, after one comparison of the first four bytes. Every field is validated (code layout, severity range, category range, ASCII name); a malformed probe is `None`. No allocation [est. about 10 ns for a generated type, about 2 ns for a foreign one].

```rust
use define_err::identify;

let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(DbError::Timeout { after_ms: 5 });
let id = identify(&*boxed).unwrap();
assert_eq!(id.code.unwrap().to_string(), "DB-0001");
assert_eq!(id.type_name, "DbError::Timeout");

let foreign = std::fmt::Error;
assert!(identify(&foreign).is_none());
```

### `innermost_code`

```rust
pub fn innermost_code<'a>(error: &'a (dyn core::error::Error + 'static)) -> Option<(Code, Identity<'a>)>;
```

Walks `source()` from `error` and returns the last (innermost) entry that has a code, with its identity. Stops after 64 steps, so a cyclic chain cannot loop. Useful when a coded error is wrapped by context layers or foreign wrappers that know nothing about codes.

```rust
use define_err::innermost_code;

if let Some((code, id)) = innermost_code(&*err) {
    eprintln!("[{code}] {} ({})", err, id.type_name);
}
```

### `Identity`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Identity<'a> {
    pub code: Option<Code>,
    pub severity: Severity,
    pub category: Category,
    pub kinds: Kinds,
    pub has_public: bool,
    pub transparent: bool,
    pub type_name: &'a str,
}
```

`type_name` borrows from the error (a probe from a foreign type could be non-`'static`); copy it to keep it. `has_public` says the variant wrote its own public message (otherwise the category's default applies). `transparent` is set when a transparent wrapper's inner value has no probe of its own.

## `Secret<T>`

Since `0.3` (SEC-01 to SEC-03).

```rust
#[derive(Clone, Copy, Default, Hash)]     // each only when T has it
pub struct Secret<T>(/* private */);

impl<T> Secret<T> {
    pub const fn new(value: T) -> Secret<T>;
    pub const fn expose(&self) -> &T;
    pub fn into_inner(self) -> T;
}
impl<T> From<T> for Secret<T>;
impl<T> core::fmt::Debug for Secret<T>;     // "[redacted]"
impl<T> core::fmt::Display for Secret<T>;   // "[redacted]"
```

A field type that keeps a value out of every `Display` and `Debug` output: both print `[redacted]` whatever the format flags (width and alignment are honoured around the word, so tables stay aligned). The value comes back only through `expose()` and `into_inner()`, which are easy to search for in review.

`Secret` implements neither `Deref`, `AsRef`, `Borrow`, nor `PartialEq`: each would let the value slip into a format string or a timing-unsafe comparison. Wiping memory on drop is not done here (that needs `unsafe` volatile writes and is the job of the crypto crates); `Secret` only redacts output.

```rust
use define_err::{define_errors, Secret};

define_errors! {
    pub enum AuthError {
        BadToken { user: String, token: Secret<String> } => "token {token} rejected for {user}",
    }
}

let err = AuthError::BadToken { user: "ada".into(), token: Secret::new("s3cr3t".into()) };
assert_eq!(err.to_string(), "token [redacted] rejected for ada");
assert!(!format!("{err:?}").contains("s3cr3t"));
assert_eq!(format!("{:>12}", Secret::new("pw")), "  [redacted]");
```

## Reporting facade

Since `0.5` (FAC-01 to FAC-18).

Any crate that depends only on define-err can report errors, warnings, notices, and triggered conditions to whatever the application installed: the error service (L7), a test capture, or the application's own reporter. The application installs a reporter once, at start-up. Until then every verb costs one atomic load and returns; it never allocates, formats, or blocks. No verb ever changes control flow or panics.

```text
 any crate        define_err::report(&err) / warn(..) / notice(..) / trigger(..)
     |
 [control word]   one acquire load: nothing installed -> return
     |
 [warn mode, once per site, minimum severity, kinds mask] -> filtered -> return
     |
 Event { verb, kinds, severity, code, category, error, message, location }   (on the stack)
     |
 &'static dyn Reporter :: receive(&event)
```

Every verb is `#[track_caller]` and every verb macro expands at the call site, so the event's location is the caller's file, line, and column, never a helper's. A `#[track_caller]` helper that calls a verb reports where the helper was called.

### `Reporter`

```rust
pub trait Reporter: Send + Sync {
    fn receive(&self, event: &Event<'_>);
    fn flush(&self) {}
}
```

The contract, also in `docs/FACADE.md` (from `0.5`):

- `receive` returns quickly (microseconds) and never blocks on I/O: queue instead, as the error service does.
- It may be called from any thread at once.
- It must not assume one call per error: a report and a later warning about the same error are two events.
- It must keep nothing borrowed from the event after it returns; it copies what it needs or builds an [`ErrorRecord`](#errorrecord).
- With `std`, a verb called from inside `receive` on the same thread is dropped and counted, and a panic inside `receive` is caught and counted. Without `std` neither guard exists, and the reporter must not call verbs or panic.

```rust
use define_err::{Event, Reporter};

struct StderrReporter;

impl Reporter for StderrReporter {
    fn receive(&self, event: &Event<'_>) {
        let at = event.location();
        match (event.error(), event.message()) {
            (Some(err), _) => eprintln!("{} {}:{} {err}", event.severity(), at.file(), at.line()),
            (None, Some(msg)) => eprintln!("{} {}:{} {msg}", event.severity(), at.file(), at.line()),
            (None, None) => {}
        }
    }
}
```

### `set_reporter` and `set_boxed_reporter`

```rust
pub fn set_reporter(reporter: &'static dyn Reporter) -> Result<(), InstallError>;

#[cfg(feature = "alloc")]
pub fn set_boxed_reporter(reporter: Box<dyn Reporter>) -> Result<(), InstallError>;
```

Installs the process's reporter. It can be set once: the slot is never replaced, so a later call returns `Err(InstallError)` naming the file and line of the installation that won. `set_boxed_reporter` leaks the box (once) to get a `'static` reference.

Install first thing in `main`, before any library code runs, and treat `Err` as a fatal boot error: a dependency that installed its own reporter first would otherwise see every error the application reports, or silence them.

```rust
static REPORTER: StderrReporter = StderrReporter;

fn main() {
    if let Err(err) = define_err::set_reporter(&REPORTER) {
        eprintln!("{err}");                // names where the other reporter was installed
        std::process::exit(70);
    }
    // ...
}
```

### `InstallError` and `Slot`

```rust
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct InstallError {
    pub slot: Slot,
    pub installed_at: Option<Location>,
}
impl core::fmt::Display for InstallError;
impl core::error::Error for InstallError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Slot { Reporter, CreationHook }
```

Returned when a write-once slot is already taken. `installed_at` is where the first installation succeeded (`None` only if the losing call raced the winner between its claim and its publish). `Display`: "a reporter is already installed (at src/boot.rs:41:5); install it once, at start-up".

### `is_installed`, `reporter`, `reporter_installed_at`, `flush`

```rust
pub fn is_installed() -> bool;
pub fn reporter() -> Option<&'static dyn Reporter>;
pub fn reporter_installed_at() -> Option<Location>;
pub fn flush();
```

- `is_installed`: one load. Lets a caller skip expensive preparation (building an `ErrorRecord`) when nobody listens.
- `reporter`: the installed reporter, for code that forwards events.
- `reporter_installed_at`: where the reporter was installed, for audits ("who is reading our errors?").
- `flush`: calls `Reporter::flush` (for example before the process exits); returns at once when nothing is installed.

```rust
if define_err::is_installed() {
    let record = build_expensive_record();
    define_err::report_record(&record);
}
```

### `report`, `report_with`, `report_record`

```rust
#[track_caller]
pub fn report(error: &(dyn core::error::Error + 'static));

#[track_caller]
pub fn report_with(error: &(dyn core::error::Error + 'static), severity: Severity, kinds: Kinds);

#[cfg(feature = "alloc")]
#[track_caller]
pub fn report_record(record: &ErrorRecord);
```

- `report` delivers an error. Severity, kinds, code, and category come from the error's probe; an error without one is reported as `ERROR` with category `Unknown`, except a `std::io::Error` (with `std`), whose category comes from its kind.
- `report_with` sets severity and kinds explicitly, for a caller that knows better than the definition (a timeout that is routine in this context, for example).
- `report_record` delivers an owned record: the event carries it (`Event::record()`), and severity, kinds, code, and category come from the record. This is how a dynamic error type reports what a `source()` chain cannot carry: its sections, attachments, fields, and propagation trail, with every message already rendered by its own guarded writer, so the reporter never calls a user's `Display`.

None of them changes control flow: the caller still returns or handles the error. Report at boundaries (a request handler, a job runner, `main`), not at every `?`.

```rust
use define_err::{Kinds, Severity};

match store.get(key) {
    Ok(value) => Ok(value),
    Err(err) => {
        define_err::report(&err);
        Err(err)
    }
}

// A cache miss that is routine here:
define_err::report_with(&miss, Severity::DEBUG, Kinds::ERROR);
```

Cost [est.]: one load when nothing is installed; under 3 ns when filtered out by severity; under 10 ns plus the reporter's own work when delivered.

### `notice` and `notice!`

```rust
#[track_caller]
pub fn notice(message: &str);

#[track_caller]
pub fn notice_with(kinds: Kinds, message: &str);

#[macro_export]
macro_rules! notice { ($($arg:tt)+) => { /* format arguments, at the call site */ }; }
```

An informational event below a warning (PHP `E_NOTICE`, syslog notice): kinds `NOTICE`, severity `NOTICE` (10). `notice_with` adds kinds. The function never interprets braces in its text; the macro takes a format string, which must be a literal.

```rust
define_err::notice("cache rebuilt from the journal");
define_err::notice!("cache rebuilt from the journal in {} ms", elapsed.as_millis());
```

### `warn` and `warn!`

```rust
#[track_caller]
pub fn warn(message: &str);

#[track_caller]
pub fn warn_with(kinds: Kinds, message: &str);

#[macro_export]
macro_rules! warn { ($($arg:tt)+) => { /* once per call site */ }; }
```

A warning: kinds `WARNING`, severity `WARNING` (13). By default each call site delivers **once** per process, so a warning in a loop or a hot path reaches the reporter one time; the [warn mode](#filters-warnmode-minimum-severity-kinds-mask) can change that.

- `warn!` keeps one `AtomicBool` at its call site: exact, for any number of sites [est. under 2 ns after the first delivery].
- `warn()` (a function cannot own a static per call site) keeps a fixed, lock-free table of 512 sites keyed by a hash of the caller's file, line, and column [est. under 5 ns after the first delivery]. When more than 512 distinct sites have warned, the extra sites deliver every time and `stats().site_table_overflow` counts it: a warning is never lost to the table.
- A warning issued before a reporter is installed does not use up its site: it is delivered the first time it happens after installation.

```rust
fn parse_port(text: &str) -> u16 {
    text.parse().unwrap_or_else(|_| {
        define_err::warn!("port {text:?} is not a number; using 8080");    // once, however often this runs
        8080
    })
}

define_err::warn_with(define_err::Kinds::CONFIG, "configuration file is empty; using defaults");
```

The macro's name matches `log::warn!` and `tracing::warn!`; call it by path (`define_err::warn!`) where both crates are used. It is not in the prelude for this reason.

### `trigger` and `trigger!`

```rust
#[track_caller]
pub fn trigger(kinds: Kinds, message: &str);

#[macro_export]
macro_rules! trigger { ($kinds:expr, $($arg:tt)+) => { /* ... */ }; }
```

Raises a named condition on purpose (PHP `trigger_error`): verb `Trigger`, kinds `TRIGGER` plus the given kinds, severity `WARNING`. The error service may run actions registered for those kinds (alert, escalate, lock down); define-err only delivers the event. `trigger(Kinds::NONE, ..)` delivers kinds `TRIGGER` alone.

```rust
use define_err::Kinds;

if failed_logins > 20 {
    define_err::trigger!(Kinds::SECURITY, "{failed_logins} failed logins for {user} in one minute");
}
```

### `deprecated` and `deprecated!`

```rust
#[track_caller]
pub fn deprecated(item: &str);

#[macro_export]
macro_rules! deprecated {
    ($item:literal $(, since = $since:literal)? $(, replacement = $replacement:literal)?) => { /* once per call site */ };
}
```

A run-time deprecation warning for anything that is not a Rust item (a configuration key, a protocol field, an API endpoint): kinds `DEPRECATED | WARNING`, severity `WARNING`, once per call site like `warn`. The message is "`<item>` is deprecated[ since <since>][; use <replacement> instead]". Rust items use `#[deprecated]` instead; deprecated codes are marked in the catalog.

```rust
if config.contains("pool.max") {
    define_err::deprecated!("pool.max", since = "2.1", replacement = "pool.size");
}
```

### `soft`

```rust
#[track_caller]
pub fn soft<E: core::error::Error + 'static>(error: E) -> Result<(), E>;
```

A soft error: a problem that is a warning today and may become an error under a strict policy. In the default mode `soft` delivers a warning that carries the error (kinds `WARNING` plus the error's kinds, severity `WARNING`) and returns `Ok(())`. When the warn mode is `Error`, it returns `Err(error)`, even when no reporter is installed, so a strict deployment refuses what a lenient one tolerates.

```rust
fn load_plugin(path: &Path) -> Result<Plugin, PluginError> {
    if !signature_present(path) {
        define_err::soft(PluginError::Unsigned { path: path.into() })?;     // warns, or fails in strict mode
    }
    Plugin::open(path)
}
```

### Filters: `WarnMode`, minimum severity, kinds mask

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum WarnMode { Once, Always, Ignore, Error }

pub fn set_warn_mode(mode: WarnMode);
pub fn warn_mode() -> WarnMode;
pub fn set_min_severity(min: Option<Severity>);
pub fn min_severity() -> Option<Severity>;
pub fn set_kinds_mask(mask: Kinds);
pub fn kinds_mask() -> Kinds;
```

| Setting | Default | Effect |
|---|---|---|
| warn mode `Once` | yes | `warn` and `deprecated` deliver once per call site |
| `Always` | | every warning is delivered; switching back to `Once` keeps the sites that already fired |
| `Ignore` | | warnings are dropped; `report`, `notice`, and `trigger` are unaffected |
| `Error` | | as `Once`, and `soft` returns `Err` |
| minimum severity | `None` (everything) | events below it are dropped before the event is built or anything is formatted |
| kinds mask | `Kinds::ALL` | events with no kind in the mask are dropped the same way |

These are process-wide settings for the facade's own prefilter, set by the application (or by the error service from its configuration). Per-listener filtering, storm control, and per-code policy belong to the error service.

```rust
use define_err::{Kinds, Severity, WarnMode};

define_err::set_warn_mode(WarnMode::Error);                 // strict deployment
define_err::set_min_severity(Some(Severity::WARNING));      // drop notices and debug reports
define_err::set_kinds_mask(Kinds::ALL - Kinds::DEPRECATED); // nobody here acts on deprecations
```

### `Event` and `Verb`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Verb { Report, Notice, Warn, Trigger, Deprecated, Soft }

#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Event<'a> { /* private */ }

impl<'a> Event<'a> {
    pub fn verb(&self) -> Verb;
    pub fn kinds(&self) -> Kinds;
    pub fn severity(&self) -> Severity;
    pub fn code(&self) -> Option<Code>;
    pub fn category(&self) -> Category;
    pub fn error(&self) -> Option<&'a (dyn core::error::Error + 'static)>;
    pub fn message(&self) -> Option<core::fmt::Arguments<'a>>;
    pub fn location(&self) -> Location;
    pub fn type_name(&self) -> Option<&'a str>;
    #[cfg(feature = "alloc")]
    pub fn record(&self) -> Option<&'a ErrorRecord>;

    pub fn new(verb: Verb, location: Location) -> Event<'a>;
    pub fn with_kinds(self, kinds: Kinds) -> Event<'a>;
    pub fn with_severity(self, severity: Severity) -> Event<'a>;
    pub fn with_code(self, code: Option<Code>) -> Event<'a>;
    pub fn with_category(self, category: Category) -> Event<'a>;
    pub fn with_error(self, error: &'a (dyn core::error::Error + 'static)) -> Event<'a>;
    pub fn with_message(self, message: core::fmt::Arguments<'a>) -> Event<'a>;
    #[cfg(feature = "alloc")]
    pub fn with_record(self, record: &'a ErrorRecord) -> Event<'a>;
}
impl core::fmt::Debug for Event<'_>;
```

What a reporter receives. Everything is borrowed and lives on the caller's stack for the length of `receive`. `error()` is set for `report`, `report_with`, and `soft`; `message()` for `notice`, `warn`, `trigger`, and `deprecated`; `record()` for `report_record` (whose verb is `Report`). `type_name()` comes from the probe or the record. The `new` and `with_*` methods (Tier 3) let a reporter's tests and the error service build events; `new` starts from kinds `NONE`, severity `ERROR`, no code, category `Unknown`.

```rust
use define_err::{Event, Kinds, Verb};

let event = Event::new(Verb::Notice, define_err::here())
    .with_kinds(Kinds::NOTICE)
    .with_message(format_args!("test notice"));
my_reporter.receive(&event);
```

### `stats` and `FacadeStats`

```rust
pub fn stats() -> FacadeStats;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[non_exhaustive]
pub struct FacadeStats {
    pub site_table_overflow: u64,
    pub reentrant_dropped: u64,
    pub reporter_panics: u64,
}
```

Counters for what the facade did not deliver normally: warnings from sites past the 512-site table (delivered every time), events dropped because a reporter called a verb from inside `receive` (`std`), and panics caught inside `receive` (`std`, counting a panicking payload drop as well). Relaxed loads; for health checks and tests. Since `0.5` (FAC-16).

### Creation hook: `CreationHook`, `CreationInfo`, `Annotate`

```rust
pub trait CreationHook: Send + Sync {
    fn on_create(&self, info: &CreationInfo<'_>, out: &mut dyn Annotate);
}

pub trait Annotate {
    fn field(&mut self, key: &'static str, value: &dyn core::fmt::Display);
    fn kind(&mut self, kinds: Kinds);
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct CreationInfo<'a> { /* private */ }
impl<'a> CreationInfo<'a> {
    pub fn new(location: Location, code: Option<Code>, type_name: &'a str) -> CreationInfo<'a>;
    pub fn location(&self) -> Location;
    pub fn code(&self) -> Option<Code>;
    pub fn type_name(&self) -> &'a str;
}

pub fn set_creation_hook(hook: &'static dyn CreationHook) -> Result<(), InstallError>;
pub fn creation_hook() -> Option<&'static dyn CreationHook>;
```

A second write-once slot, beside the reporter, for an observer that adds scope data (a request id, a tenant, a security kind) **when a dynamic error is created**. Only dynamic-error libraries call it (report-err calls `creation_hook()` when it builds its `Error`); errors generated by `define_errors!` never do, because building a typed error has no side effects. The slot costs one load when empty.

The caller of the hook owns its guards: report-err prevents a hook from recursing on the same thread and isolates a panicking hook (with `std`). A hook must return in microseconds and must not create a dynamic error itself. `set_creation_hook` follows the same once-only rule as `set_reporter` and returns `InstallError { slot: Slot::CreationHook, .. }` on a second call.

```rust
use define_err::{Annotate, CreationHook, CreationInfo};

struct RequestScope;

impl CreationHook for RequestScope {
    fn on_create(&self, _info: &CreationInfo<'_>, out: &mut dyn Annotate) {
        if let Some(id) = current_request_id() {
            out.field("request_id", &id);
        }
    }
}

static SCOPE: RequestScope = RequestScope;
define_err::set_creation_hook(&SCOPE)?;
```

### Targets without compare-and-swap

On targets without atomic compare-and-swap (`thumbv6m-none-eabi`), the slots cannot be claimed safely, so the facade is compiled out: `set_reporter`, `set_boxed_reporter`, and `set_creation_hook` do not exist, and every verb compiles to an empty `#[inline]` function. Library code that reports still compiles unchanged on those targets. (A racy installer for such targets is planned after 1.0, FAC-19.)

## `ErrorRecord`

Since `0.5` (REC-01 to REC-06). Feature `alloc`.

An owned, `Send + Sync + Clone` snapshot of an error, for anything that outlives the borrow: queued listeners, a crash ring, JSON output, messages between processes and nodes. Every part is bounded by [`RecordLimits`](#recordlimits), checked before the allocation it guards.

### Fields

```rust
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ErrorRecord {
    pub code: Option<Code>,
    pub severity: Severity,
    pub category: Category,
    pub kinds: Kinds,
    pub public: Cow<'static, str>,
    pub chain: Vec<ChainEntry>,
    pub fields: Vec<(Cow<'static, str>, String)>,
    pub trail: Vec<TrailEntry>,
    pub backtrace: Option<String>,
    pub event_id: Option<u128>,
    pub time_unix_nanos: Option<u64>,
    pub handled: bool,
    pub truncated: bool,
}
impl core::fmt::Display for ErrorRecord;   // "[DB-0001] query timed out after 5000 ms: caused by ..."
```

| Field | Content |
|---|---|
| `code`, `severity`, `category`, `kinds` | from the outermost entry of the chain that has a code, else from the outermost entry |
| `public` | the public message of that entry, or its category's default; never any `Display` text |
| `chain` | one entry per error, outermost first: the private message, the entry's code, its type name |
| `fields` | key and value pairs added by the creator or a hook |
| `trail` | locations: where the error was created, converted, given context, reported |
| `backtrace` | text, filled by the dynamic-error or service library |
| `event_id`, `time_unix_nanos` | filled by the error service (there is no clock in `no_std`) |
| `handled` | whether the error was handled (OpenTelemetry's distinction for severities) |
| `truncated` | a limit was hit somewhere |

`Display` writes one line (`[code] outer message: caused by inner message ...`); every other format belongs to the render library. The limits the record was built with are kept with it (not a public field), so later additions stay inside the same bounds.

### Building a record

```rust
impl ErrorRecord {
    pub fn from_error(error: &(dyn core::error::Error + 'static)) -> ErrorRecord;
    pub fn from_error_with(error: &(dyn core::error::Error + 'static), limits: &RecordLimits) -> ErrorRecord;
    pub fn from_event(event: &Event<'_>) -> ErrorRecord;
    pub fn from_event_with(event: &Event<'_>, limits: &RecordLimits) -> ErrorRecord;
    pub fn new(code: Option<Code>, severity: Severity, category: Category, kinds: Kinds) -> ErrorRecord;
    pub fn with_limits(self, limits: RecordLimits) -> ErrorRecord;
}
```

- `from_error` walks `source()` up to `max_chain` entries, writes each `Display` through a bounded writer that stops at `max_message_bytes` (cut on a character boundary), and reads each entry's identity with [`identify`](#identify). A `Display` that returns `fmt::Error` is stored as `"<message unavailable>"`. A `source()` that returns itself stops at the limit.
- `from_event` adds the event's location to the trail and its message; an event without an error (a notice) becomes one synthetic chain entry with type name `"notice"` (and likewise for the other verbs). An event that carries a record returns a clone of it with the event's location added.
- `new` starts an empty record for a caller that assembles the chain itself (a dynamic error type that renders its own chain through a guarded writer and adds sections, fields, and a trail that `source()` does not expose); `public` is the category's default message. `with_limits` replaces the default limits before anything is added.

```rust
use define_err::ErrorRecord;

let record = ErrorRecord::from_error(&err);
queue.push(record);                                  // owned, Send + Sync

println!("{record}");                                // [DB-0001] query timed out after 5000 ms
```

Cost [est.]: under 1 us for a three-level chain; allocations: one per chain entry plus the vectors (counted and published in `0.6`).

### Adding to a record

```rust
impl ErrorRecord {
    pub fn push_chain(&mut self, message: &dyn core::fmt::Display, code: Option<Code>, type_name: &str) -> bool;
    pub fn push_field(&mut self, key: impl Into<Cow<'static, str>>, value: &dyn core::fmt::Display) -> bool;
    pub fn push_trail(&mut self, entry: TrailEntry) -> bool;
    pub fn set_backtrace(&mut self, text: &dyn core::fmt::Display) -> bool;
    pub fn set_event_id(&mut self, id: u128);
    pub fn set_time(&mut self, unix_nanos: u64);
    pub fn set_handled(&mut self, handled: bool);
}
```

Each `push_*` and `set_backtrace` writes through the bounded writer and returns `false` when a limit stopped it (the entry is cut or left out, and `truncated` is set). They never fail otherwise and never panic. `push_chain` appends after the existing entries (outermost first, so a builder pushes from the outside in).

```rust
use define_err::{Category, ErrorRecord, Kinds, Severity};

let mut record = ErrorRecord::new(None, Severity::ERROR, Category::Unavailable, Kinds::ERROR);
record.push_chain(&"while syncing tenant 42", None, "context");
record.push_chain(&io_error, None, "std::io::Error");
record.push_field("tenant", &42);
assert!(!record.truncated);
```

### `RecordLimits`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct RecordLimits {
    pub max_chain: usize,          // 32
    pub max_message_bytes: usize,  // 1,024
    pub max_fields: usize,         // 32
    pub max_field_bytes: usize,    // 256
    pub max_trail: usize,          // 64
    pub max_total_bytes: usize,    // 16,384
}
impl RecordLimits {
    pub const DEFAULT: RecordLimits;
    pub const fn new() -> RecordLimits;    // DEFAULT
}
impl Default for RecordLimits;
```

The bounds of a record, safe for servers by default. Change a copy:

```rust
use define_err::{ErrorRecord, RecordLimits};

let mut limits = RecordLimits::DEFAULT;
limits.max_chain = 8;
limits.max_total_bytes = 4_096;
let record = ErrorRecord::from_error_with(&err, &limits);
```

A limit of 0 keeps nothing of that part. `max_total_bytes` bounds all message, field, and backtrace text together.

### `ChainEntry` and `TrailEntry`

```rust
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ChainEntry {
    pub message: String,
    pub code: Option<Code>,
    pub type_name: Cow<'static, str>,
    pub has_control: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TrailEntry {
    pub file: Cow<'static, str>,
    pub line: u32,
    pub column: u32,
    pub step: TrailStep,
}
impl TrailEntry {
    pub fn new(location: Location, step: TrailStep) -> TrailEntry;   // borrows the file name, no allocation
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum TrailStep { Created, Converted, Context, Reported }
```

`has_control` is set when the message contains a control character (a newline, an escape), so a log sink can escape it without scanning again. Messages are stored verbatim: escaping is the sink's job. Trail entries hold owned or borrowed file names, so a record received from another node needs no `'static` data.

## `testing`

Since `0.5` (TST-01, TST-02). Feature `testing` (implies `std`).

```rust
pub mod testing {
    pub fn capture<R>(f: impl FnOnce() -> R) -> Result<(R, Vec<CapturedEvent>), InstallError>;

    #[derive(Clone, Debug, PartialEq)]
    #[non_exhaustive]
    pub struct CapturedEvent {
        pub verb: Verb,
        pub kinds: Kinds,
        pub severity: Severity,
        pub code: Option<Code>,
        pub message: String,
        pub file: &'static str,
        pub line: u32,
        pub column: u32,
        pub record: Option<ErrorRecord>,
    }

    pub struct CaptureReporter;
    impl Reporter for CaptureReporter;
}
```

`capture` runs `f` and returns its result with every event the facade delivered on this thread while `f` ran. It installs `CaptureReporter` on first use; the reporter routes each event to a buffer of the thread that raised it, so tests running in parallel never see each other's events. `message` is the formatted message, or the error's `Display` for error events; `record` is filled for error events.

If a different reporter is already installed, `capture` returns `Err(InstallError)` instead of running `f`: a test cannot capture what goes to someone else's reporter, and an empty or partial list must never pass as the real one.

```rust
use define_err::testing::capture;

#[test]
fn test_parse_port_bad_value_warns_once() {
    let (ports, events) = capture(|| (parse_port("x"), parse_port("y"))).unwrap();
    assert_eq!(ports, (8080, 8080));
    assert_eq!(events.len(), 1);                     // once per call site
    assert!(events[0].message.contains("not a number"));
}
```

## Crate root

```rust
pub const VERSION: &str;          // since 0.1 (ROOT-01)
pub mod prelude;                  // since 0.3 (ROOT-02)
pub mod catalog;                  // since 0.4
#[cfg(feature = "testing")]
pub mod testing;                  // since 0.5
#[doc(hidden)]
pub mod __private;                // generated code only; see GENERATED-CODE.md
```

Every other item is at the crate root: `define_err::Code`, `define_err::report`, `define_err::warn!`. The crate documentation (ROOT-03, `0.3`) shows Tier 1 first, the feature table, and [what can never be observed](#what-can-never-be-observed).

```rust
assert!(!define_err::VERSION.is_empty());
```

## Item index

| Id | Item | Tier | Grade | Since |
|---|---|---|---|---|
| ROOT-01 | `VERSION` | 1 | P0 | 0.1 |
| ROOT-02 | `prelude` | 1 | P0 | 0.3 |
| ROOT-03 | crate documentation | 1 | P0 | 0.3 |
| DEF-01 to DEF-12 | `define_errors!`: variants, messages, `Display`, `Error`, sources, `From`, `transparent`, structs, bindings, extra arguments, `#[non_exhaustive]`, `Debug`, several types per block, `ErrorInfo` defaults | 1 | P0/P1 | 0.3 |
| DEF-13 | identity probe in `description()` | 3 | P1 | 0.4 |
| DEF-14 to DEF-23, DEF-25 | namespace, `#[code]`, metadata attributes, `retry_after`, `#[location]`, compile-time checks, `retired`, deprecated codes, `docs`, `CATALOG`, transparent `ErrorInfo` | 1/2 | P0/P1 | 0.4 |
| DEF-24 | `#[cfg]` on variants and fields | 1 | P2 | 0.5 |
| DEF-26 | simple type parameters | 1 | P3 | after 1.0 |
| DEF-27 | 500-variant enums | 1 | P0 | 0.3 |
| CODE-01 to CODE-09 | `Namespace`, `namespace!`, `Code`, `code!`, text form, `parse`, `as_u64`/`identity`/`from_u64`, ordering, `CodeParseError` | 1/2/3 | P0/P1 | 0.4 |
| SEV-01 to SEV-04 | `Severity`, `at`/`new`/`saturating`/`get`, `band`/`otel_name`/`label`/`parse`, `LevelNames` | 1/2 | P0 to P2 | 0.4 |
| CAT-01 to CAT-07 | `Category` and its tables, `RetryAdvice`, `Action`, `from_io_kind` | 1/2 | P0 to P2 | 0.4 |
| KND-01 to KND-05 | `Kinds` | 1/2/3 | P0/P1 | 0.4 |
| INF-01 | `ErrorInfo` | 1 | P0 | 0.3 (defaults), 0.4 |
| INF-02, INF-03 | `CodeInfo`, `CodeState`, `UrlDisplay` | 2 | P0/P1 | 0.4 |
| INF-04 | `ErrorInfo for std::io::Error` | 1 | P2 | 0.4 |
| INF-05, INF-06 | `catalog::find`, `catalog::check_unique`, `DuplicateCode` | 2 | P1 | 0.4 |
| INF-07 | `Location`, `here` | 1 | P1 | 0.3 |
| PRB-01 to PRB-03 | probe format, `identify`, `innermost_code`, `Identity` | 3 | P0/P1 | 0.4 |
| SEC-01 to SEC-03 | `Secret<T>` | 1 | P0 | 0.3 |
| FAC-01 to FAC-18 | `Reporter`, slots, `InstallError`, verbs and macros, `Event`, `Verb`, filters, `stats`, guards, targets without CAS | 1/2/3 | P0 to P2 | 0.5 |
| FAC-20 | `report_record` and `Event::record` | 2 | P1 | 0.5 |
| FAC-21 | creation-hook slot: `CreationHook`, `CreationInfo`, `Annotate`, `set_creation_hook`, `creation_hook` | 3 | P1 | 0.5 |
| FAC-19 | racy installer for targets without CAS | 3 | P3 | after 1.0 |
| REC-01 to REC-06 | `ErrorRecord`, building, limits, adding, `Display` | 2/3 | P0/P1 | 0.5 |
| REC-07 | `ErrorRecord::new`, `push_chain`, `with_limits`, `set_handled` | 3 | P1 | 0.5 |
| TST-01, TST-02 | `testing::capture`, `CaptureReporter` | 2/3 | P1 | 0.5 |
