# Migrating to define-err

## From thiserror

There are two routes, and they generate the same code.

1. **`define_errors!` (no proc macro).** Rewrite each enum into the form below. Best for small error sets, `no_std` crates, and crates that want no build dependency.
2. **The attribute front end.** A derive that takes thiserror's attributes, for very large error sets, once it is published. It lives in its own crate because a proc macro cannot live here.

A crate can move from route 2 to route 1 later with no change its users can see.

### Conversion rules

| thiserror | `define_errors!` |
|---|---|
| `#[derive(Debug, Error)] pub enum E { ... }` | `define_errors! { pub enum E { ... } }` (drop `Debug` and `Error`, which are added; keep other derives such as `Clone`) |
| `#[error("page {page_id} not found")] NotFound { page_id: u64 },` | `NotFound { page_id: u64 } => "page {page_id} not found",` |
| `#[error("page i/o error: {0}")] Io(#[from] io::Error),` | `Io(#[from] source: io::Error) => "page i/o error: {source}",` |
| `#[error("invalid usage: {0}")] InvalidUsage(String),` | `InvalidUsage(detail: String) => "invalid usage: {detail}",` |
| `#[error("file not found: {0}")] NotFound(PathBuf),` | `NotFound(path: PathBuf) => "file not found: {path}",`: a `Path`, `&Path`, or `PathBuf` shows through `display()`, as with thiserror (since 0.8, with the `std` feature; on 0.7, write `("file not found: {}", path.display())`) |
| `#[error("{}", .diagnostic.format())] Diag { diagnostic: D },` | `Diag { diagnostic: D } => ("{}", diagnostic.format()),` |
| `#[error("size {size} not in {min}..={max}", min = crate::MIN, max = crate::MAX)]` | `=> ("size {size} not in {min}..={max}", min = crate::MIN, max = crate::MAX)` |
| `#[error(transparent)] Other(#[from] OtherError),` | `Other(#[from] inner: OtherError) => transparent,` |
| `#[source] cause: E`, or a field named `source` | unchanged |
| `#[non_exhaustive]` on the enum | keep it or delete it: the macro adds it when absent and never adds a second copy |
| an enum published as exhaustive | add `#[exhaustive]`, so the migration is not a breaking change |
| a code inside the message: `#[error("SVC-00001: configuration error [{field}]")]` | `namespace = "SVC", width = 5;`, `#[code(1, help = "...")]`, and the message without the prefix; or keep the prefix in the message, so `Display` stays byte-identical, and state the code again in `#[code]` |
| an existing code with no explanation | every `#[code]` needs `help = "..."`: the catalog ships each code with what to do about it |

Tuple fields get a name, `Read(page: u64, got: usize)`, so the message can use it: a declarative macro cannot look inside a string literal for `{0}`. The field is still positional when the variant is built, `ShortRead(7, 12)`.

`examples/migration_before_after.rs` shows one enum both ways. The `Display` output is byte-identical; `tests/conformance.rs` checks it against thiserror itself.

### What changes for your users

- An enum that was not `#[non_exhaustive]` becomes so unless you write `#[exhaustive]`. For a published crate, that is a breaking change you must choose.
- `Error::description()` returns the identity probe instead of the deprecated default text. Nothing in the standard library calls it.
- The MSRV becomes at least 1.81, the first release with `core::error::Error`.
- The type implements `define_err::ErrorInfo`, so define-err becomes a public dependency. For a crate past 1.0 that matters: a later define-err minor is a different `ErrorInfo`, and `cargo semver-checks` cannot see that. Until define-err 1.0, say in your changelog that the `ErrorInfo` impl follows define-err's version. At 1.0, define-err's last 0.x minor gets a patch release that re-exports 1.0's traits, so crates on either share one `ErrorInfo`.
- A type that already has its own inherent `code()` keeps it: an inherent method wins over `ErrorInfo::code`, so `err.code()` returns what it always did, and `ErrorInfo::code(&err)` returns the `Code`.

### After the move

Nothing else is required. A plain error type stays one line per variant. When you want more, each extra is one attribute on the variant that needs it:

```rust
define_errors! {
    namespace = "PAGE";

    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",

        #[code(1, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[public("The data could not be read.")]
        Corrupt { page: u64 } => "page {page} failed its checksum",
    }
}
```

## From error-forge

error-forge's `ForgeError` metadata maps onto `ErrorInfo`, and its global hook maps onto the reporting facade.

| error-forge | define-err |
|---|---|
| `ForgeError::kind()`, `caption()` | [`Category`](./API.md#category), [`Code`](./API.md#code), and `#[public]` |
| `is_retryable()` | `ErrorInfo::is_retryable`, with `#[retryable]` to override the category |
| `is_fatal()` | `severity() >= Severity::CRITICAL` |
| the `define_errors!` macro with `#[error(...)]` per variant | the form above, with `=> "message"` after the fields |
| the global error hook | [`set_reporter`](./API.md#set_reporter-and-set_boxed_reporter): write-once, and it names who installed it |
| a hook that fires when an error is built | none. Building a typed error has no side effects here; call `define_err::report(&err)` at a boundary |

The last row is the real change: a hook on construction turns a value into a side effect, and fires for errors that are handled and never matter. Report at boundaries instead. A dynamic error type that needs creation data uses the [creation hook](./API.md#creation-hook-creationhook-creationinfo-annotate).

## Reporting from a library

A library depends on define-err and calls the verbs. It never installs a reporter: that is the application's decision, once, at start-up. See [`FACADE.md`](./FACADE.md).
