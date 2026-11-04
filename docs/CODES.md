# Error codes

A code is a stable name for one kind of failure: `DB-0042`. It is what a person searches for, what a runbook is filed under, and what a dashboard counts, so it outlives the message around it. Signatures are in [`API.md`](./API.md#code).

## Layout

A code is a namespace and a number.

| Part | Rule |
|---|---|
| Namespace | 1 to 6 characters, `A` to `Z` and `0` to `9`, starting with a letter. Lowercase is accepted and stored uppercase. |
| Number | 0 to 16,777,215. 0 is the namespace's generic code. |
| Width | The digits shown, 1 to 8, default 4. Presentation only: `SVC-1` and `SVC-00001` are the same code. A number wider than the width is shown in full. |

A `Code` packs into a `u64`, is never zero (so `Option<Code>` is 8 bytes), and is compared, ordered, and hashed by namespace text and number, the width ignored. The packing is frozen at 1.0; [`tests/golden/codes.txt`](../tests/golden/codes.txt) holds it.

## Namespaces

One namespace per domain, in one `define_errors!` block or several. Pick something short and specific: `DB`, `SVC`, `JOB`, `AUTH`. The same number in two namespaces is two codes.

A block checks its own numbers at compile time. Numbers across crates cannot be checked by a compiler, so `catalog::check_unique` does it at start-up or in a test, and the error service runs the same check over every catalog it loads:

```rust
use define_err::catalog;

catalog::check_unique(&[DbError::CATALOG, StoreError::CATALOG]).unwrap();
```

## The catalog

Every type in a block with a namespace has `Type::CATALOG`, built at compile time: one [`CodeInfo`](./API.md#codeinfo) per code, plus the block's retired numbers on its first type.

| Field | Source | Default |
|---|---|---|
| `code` | `#[code(n, ...)]` | required |
| `title` | `title = "..."` | the variant's name |
| `help` | `help = "..."` | required: every code ships with what to do about it |
| `category` | `#[category(...)]` | `Unknown` |
| `severity` | `#[severity(...)]` | the category's default |
| `kinds` | `#[kinds(...)]` | `ERROR` |
| `action` | `#[action(...)]` | the category's default |
| `public` | `#[public("...")]` | the category's default |
| `type_name` | the type and variant | |
| `docs` | the block's `docs = "..."` template | none |
| `state` | `deprecated = "..."`, `retired = [...]` | `Active` |

`CodeInfo::url()` fills the `docs` template's `{code}`, `{namespace}`, and `{number}` without allocating. A docs site or an error service can render its whole reference from `CATALOG`.

## Numbers are never reused

A code's meaning is a promise to everyone who filed a runbook under it. Once a number has shipped, it means that failure forever.

**Deprecating.** The variant keeps working. Name what replaces it:

```rust
#[code(4, name = OLD_QUOTA, help = "Use DB-0010 instead.", deprecated = "2.1.0", replacement = 10)]
OldQuota => "quota exceeded",
```

The catalog state becomes `Deprecated`, naming the replacement, and the named constant carries `#[deprecated]`, so the compiler warns where it is used. Write a full version: the text goes into `#[deprecated(since = ...)]`, and clippy asks for one.

**Retiring.** When the variant is gone, keep its number:

```rust
define_errors! {
    namespace = "DB";
    retired = [4, 9];
    // ...
}
```

Retired numbers appear in the catalog as `Retired` and join the duplicate check, so a retired number used again is a compile error.

## Reading a code

```rust
err.code();                                   // the typed error
err.code_info();                              // its catalog entry
DbError::TIMEOUT;                             // a constant, from `name = TIMEOUT`
define_err::identify(&*boxed);                // through `&dyn Error`
define_err::innermost_code(&*boxed);          // the deepest code in a `source()` chain
define_err::catalog::find(&[DbError::CATALOG], code);
Code::parse("DB-0042");                       // from text; the error never repeats the input
```

Codes identify an error; they do not authenticate it. A foreign type can write any probe it likes, so a security decision must not rest on a code read from a `&dyn Error`.

## Moving codes out of messages

A code inside a message string becomes an attribute, so `err.code()` works and the message reads the same:

```rust
// Before: #[error("SVC-00001: configuration error [{field}]")]
define_errors! {
    namespace = "SVC", width = 5;

    pub enum ServiceError {
        #[code(1, help = "Fix the named field in service.noml.")]
        Config { field: String } => "configuration error [{field}]",
    }
}
```
