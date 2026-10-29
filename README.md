<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>define-err</b>
    <br>
    <sub><sup>TYPED ERROR DEFINITIONS</sup></sub>
</h1>

<div align="center">
    <a href="https://crates.io/crates/define-err"><img alt="Crates.io" src="https://img.shields.io/crates/v/define-err"></a>
    <a href="https://crates.io/crates/define-err"><img alt="Downloads" src="https://img.shields.io/crates/d/define-err?color=%230099ff"></a>
    <a href="https://docs.rs/define-err"><img alt="docs.rs" src="https://img.shields.io/docsrs/define-err"></a>
    <a href="https://github.com/jamesgober/define-err/actions"><img alt="CI" src="https://github.com/jamesgober/define-err/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.81%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        <strong>define-err</strong> is how a Rust library says what its errors are. One <code>define_errors!</code> block, one line per variant, and the macro writes the enum, <code>Debug</code>, <code>Display</code>, <code>core::error::Error</code> with <code>source()</code>, and the <code>From</code> conversions you asked for: the same code you would write by hand, and the same code thiserror generates, without a proc macro or a build dependency.
    </p>
    <p>
        When a library wants more, each extra is one optional attribute on the variant that needs it: a stable error code (<code>DB-0042</code>) with help text and a compile-time catalog, a severity on the OpenTelemetry scale, a category that maps to an HTTP status, a gRPC code, and a process exit code, kind masks, a message that is safe to show a client, and retry advice. <code>Secret&lt;T&gt;</code> keeps tokens out of logs. A small reporting facade lets any crate call <code>report</code>, <code>notice</code>, <code>warn</code>, or <code>trigger</code>, and costs one atomic load when no reporter is installed.
    </p>
    <p>
        The crate is <code>no_std</code> (with optional <code>alloc</code> and <code>std</code>) and has no dependencies.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.81+</strong> (Rust 2021 edition): the first release with <code>core::error::Error</code>.
    </p>
    <blockquote>
        <strong>Status: core (<code>0.3.0</code>).</strong> <code>define_errors!</code> has thiserror parity: messages, sources, <code>From</code>, <code>transparent</code>, structs, <code>#[non_exhaustive]</code> by default, <code>Secret&lt;T&gt;</code>, and locations. Codes and metadata arrive in <code>0.4.0</code>, the reporting facade in <code>0.5.0</code>. The complete API is in <a href="./docs/API.md"><code>docs/API.md</code></a>, what the macro expands to in <a href="./docs/GENERATED-CODE.md"><code>docs/GENERATED-CODE.md</code></a>, the design in <a href="./dev/PLAN.md"><code>dev/PLAN.md</code></a>, and the schedule in <a href="./dev/ROADMAP.md"><code>dev/ROADMAP.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

## Installation

```toml
[dependencies]
define-err = "0.3.0"
```

Without the standard library:

```toml
[dependencies]
define-err = { version = "0.3.0", default-features = false }            # core only
define-err = { version = "0.3.0", default-features = false, features = ["alloc"] }
```

## Quick start

```rust
use define_err::prelude::*;

define_errors! {
    /// Errors from the page store.
    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",
        ShortRead(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
        Io(#[from] source: std::io::Error) => "page file i/o failed",
        Token { token: Secret<String> } => "token {token} rejected",
    }
}

fn open(path: &str) -> Result<Vec<u8>, PageError> {
    Ok(std::fs::read(path)?)                          // From<io::Error>
}

assert_eq!(PageError::NotFound { page: 7 }.to_string(), "page 7 not found");
assert_eq!(PageError::ShortRead(7, 12).to_string(), "short read on page 7: got 12 bytes");
assert!(open("/does/not/exist").is_err());
```

One line per variant. The enum gets `Debug`, `Display`, `std::error::Error` with `source()`, `From<std::io::Error>`, and `#[non_exhaustive]`; tuple fields get a name so the message can use them. `=> transparent` forwards everything to the one field inside, and `Secret<T>` prints `[redacted]`. Coming from thiserror, `docs/API.md` has the one-to-one mapping; the `Display` output is byte-identical, and a test checks it against thiserror itself.

## Scope

The shape of the full Tier 1 API, with codes and metadata (`0.4.0`) and the reporting facade (`0.5.0`):

```rust
use define_err::prelude::*;

define_errors! {
    namespace = "DB";

    pub enum DbError {
        NotFound { page: u64 } => "page {page} not found",

        #[code(1, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        Io(#[from] source: std::io::Error) => "page file i/o failed",
    }
}

let err = DbError::Timeout { after_ms: 5000 };
assert_eq!(err.code().unwrap().to_string(), "DB-0001");
assert_eq!(err.category().http_status(), 504);

define_err::report(&err);                            // never changes control flow
define_err::warn("cache file is stale, rebuilding");  // once per call site
```

| Part | Arrives in |
|---|---|
| `define_errors!`: enums and structs, messages, `source`, `From`, `transparent`, `#[non_exhaustive]` by default, `Secret<T>`, `Location` | `0.3.0` (shipped) |
| Codes, the catalog, severity, categories, kinds, `ErrorInfo`, the identity probe | `0.4.0` |
| The reporting facade (`set_reporter`, `report`, `notice`, `warn`, `trigger`, `deprecated`, `soft`), `ErrorRecord`, test capture | `0.5.0` |

Errors are values in Rust; there is no throw event. The facade sees an error only when code calls one of its verbs. It cannot see an error discarded with `let _ =`, `.ok()`, or `unwrap_or`, or a typed error that is built and never reported.

See [`dev/PLAN.md`](./dev/PLAN.md) for the full scope, what is deliberately left to the other error libraries (dynamic errors, rendering, diagnostics, the error service), and the decision log.

## Features

| Feature | Default | Brings |
|---|---|---|
| `std` | yes | `std::io::Error` mappings; reentrancy guard and panic isolation in the facade. Implies `alloc`. |
| `alloc` | through `std` | `ErrorRecord`, boxed `dyn Error` sources, `set_boxed_reporter`. |
| `testing` | no | a capturing reporter for tests of code that reports. Implies `std`. |

<hr>
<br>

## Contributing

See <a href="./dev/DIRECTIVES.md"><code>dev/DIRECTIVES.md</code></a> for engineering standards and the definition of done. Before a PR: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` must be clean, and the crate must build with `--no-default-features`.

<br>

<div id="license">
    <h2>License</h2>
    <p>Licensed under either of</p>
    <ul>
        <li><b>Apache License, Version 2.0</b>: <a href="./LICENSE-APACHE">LICENSE-APACHE</a></li>
        <li><b>MIT License</b>: <a href="./LICENSE-MIT">LICENSE-MIT</a></li>
    </ul>
    <p>at your option.</p>
</div>

<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>James Gober.</strong></sup>
</div>
