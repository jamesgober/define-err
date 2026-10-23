# define-err - Project Plan

> The design, strategy, and decision record for `define-err`: library L1 "definitions" of the error collection. It is the stand-alone replacement for `thiserror`, extended with optional codes, severity, categories, kinds, a code catalog, secret redaction, an owned error record, and the reporting facade (`set_reporter` once, then `report`, `notice`, `warn`, `trigger` from any crate). A living document: every design decision, idea, or open question about the crate is written here or in [`ROADMAP.md`](./ROADMAP.md) before it is acted on.
>
> **Read order:** [`../REPS.md`](../REPS.md), then [`DIRECTIVES.md`](./DIRECTIVES.md), then **this file**, then [`ROADMAP.md`](./ROADMAP.md).
>
> **Status:** `0.1.0` published (2026-10-23). The `0.2.0` design lock is written: [`../docs/API.md`](../docs/API.md) (every public item, with signatures and examples) and [`../docs/GENERATED-CODE.md`](../docs/GENERATED-CODE.md) (the contract the attribute front end builds against), with golden files in `tests/golden/`. From `0.2.0`, `docs/API.md` is the authoritative list of signatures; section 8 below keeps the grading, tiers, versions, and costs, and decisions D-008 to D-022 (section 20) record where the locked API differs from the original specification. Waiting for James's approval of `docs/API.md`. Owner: James Gober.

---

## 1. Header

| Field | Value |
|---|---|
| Crate | `define-err` (import path `define_err`). Name chosen by James on 2026-10-22 (D-001). |
| Role | L1 "definitions" of the error collection: functionality F1, F3, F4, F5, F6, F7, F18, plus research additions R1, R4, R5, R9 (facade side), R10, R21, R23, and the kind masks |
| Collection folder | `error-collection/define-err` |
| Repository | [`github.com/jamesgober/define-err`](https://github.com/jamesgober/define-err) (public) |
| Current state | `0.1.0`: repository, manifest, crate root, CI, and this plan. No functional code. |
| Replaces | `thiserror` in every portfolio crate (section 19); the code, level, and metadata parts of `error-forge` 1.0.2 (whose lessons are in 7.3) |
| Supersedes | The definitions parts of the earlier error-suite plan (its sections 4 and 5). Where this file differs, this file wins and the difference is recorded in section 17. |
| Classification | Stateless definitions crate with one process-wide hook slot (the reporting facade). No threads, no timers, no I/O, no allocation unless the `alloc` feature is on and an owned record is asked for. |
| MSRV | 1.81 (D-002: the floor for `core::error::Error` in `no_std`) |
| Edition | 2021 (follows the MSRV; D-002) |
| `no_std` | yes: `core` always, `alloc` optional, `std` optional (default) |
| Runtime dependencies | none |
| License | `Apache-2.0 OR MIT` |
| Owner | James Gober `<me@jamesgober.com>` |

**Read order** for anyone building from this file:

1. Sections 2 to 6: what the crate is, where it sits, what it must do, and the standards it follows.
2. Section 7: the design. 7.4 (the `define_errors!` grammar) and 7.5 (the generated-code contract) are binding for both this crate and L2 (derive).
3. Section 8: the API, graded P0 to P3 and tiered 1 to 3.
4. Sections 10 to 12: threats, performance targets, tests. Every threat and edge case has a test.
5. Sections 14 to 16: roadmap, release routine, definition of done.
6. Section 17: open decisions. A recommendation stands unless James overrides it.
7. Section 19: migration of existing `thiserror` users.
8. Section 20: the decision log.

---
## Contents

1. [Header](#1-header)
2. [Purpose](#2-purpose)
3. [Position in the stack](#3-position-in-the-stack)
4. [Scope](#4-scope)
5. [Requirements](#5-requirements)
6. [Standards](#6-standards)
7. [Design and architecture](#7-design-and-architecture)
8. [API specification](#8-api-specification)
9. [Parity matrix](#9-parity-matrix)
10. [Security](#10-security)
11. [Performance](#11-performance)
12. [Testing plan](#12-testing-plan)
13. [Documentation deliverables](#13-documentation-deliverables)
14. [Version roadmap](#14-version-roadmap)
15. [Release routine](#15-release-routine)
16. [Definition of done](#16-definition-of-done)
17. [Open decisions](#17-open-decisions)
18. [References](#18-references)
19. [Migration of existing users](#19-migration-of-existing-users)
20. [Decision log](#20-decision-log)

---

## 2. Purpose

### 2.1 What it is

`define-err` is how a Rust library says **what its errors are**. A plain error type written with it reads like `thiserror` and compiles to the same code a person would write by hand:

```rust
use define_err::define_errors;

define_errors! {
    /// Errors from the page store.
    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",
        ShortRead { page: u64, got: usize } => "short read on page {page}: got {got} bytes",
        Io(#[from] source: std::io::Error) => "page file i/o failed",
    }
}
```

That expands to the enum (marked `#[non_exhaustive]`, with `Debug`), `Display`, `core::error::Error` with `source()`, and `From<std::io::Error>`. No proc macro, no build dependency, no runtime cost.

When a library wants more, every extra is one optional attribute on the variant that needs it:

```rust
use define_err::prelude::*;     // define_errors!, ErrorInfo (for .code(), .category()), Code, ...

define_errors! {
    namespace = "DB";
    docs = "https://docs.hivedb.com/errors/{code}";

    pub enum DbError {
        #[code(1, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        Corrupt { page: u64 } => "page {page} failed its checksum",
    }
}

let err = DbError::Timeout { after_ms: 5000 };
assert_eq!(err.code().unwrap().to_string(), "DB-0001");
assert_eq!(err.category().http_status(), 504);
```

And any crate that depends only on `define-err` can report to whatever error service the application installed, at the cost of one atomic load when nothing is installed:

```rust
define_err::report(&err);                       // never changes control flow
define_err::warn("cache file is stale, rebuilding");   // once per call site
```

### 2.2 What it is not

- **Not a dynamic error type.** No dynamic `Error` type (report-err's), no `anyhow::Error` equivalent, no `.context()`, no `bail!`. That is L3 (report).
- **Not a renderer.** No colour, no JSON, no RFC 9457 problem details. That is L4 (render). `define-err` gives every renderer the data (codes, categories, public messages, the owned record).
- **Not a delivery engine or an error service.** The facade passes each event to one installed `Reporter` and returns. Listeners, filters per listener, queues, storm control, crash capture, policy files, and event rules live in L7 (error service) on top of dispatch-io and relay-io.
- **Not a proc macro.** The attribute form (`#[derive(Error)]`, `#[error("...")]`) is L2 (derive), a separate crate that generates exactly the code specified in 7.5.
- **Not a recovery library.** No retry loops, backoff, or circuit breakers (throttle-net and task-io). `define-err` only says whether an error is transient and when to retry (`retry_after`).
- **Not a logger.** `notice` and `warn` are error-adjacent signals delivered to the reporter; general logging is log-io.

---

## 3. Position in the stack

### 3.1 Who uses it

| User | What it uses |
|---|---|
| Every portfolio library that declares errors (page-db, wal-db, lsm-db, txn-db, index-db, config-lib, noml-rust, mod-cli, kraken, Hive DB distro and hive-exec, the net and sec collections) | `define_errors!`, `ErrorInfo`, optionally codes, categories, `Secret<T>`, the facade verbs |
| L2 derive | generates the code of 7.5 against this crate's public and `__private` items |
| L3 report (anyhow replacement) | `ErrorInfo`, `identify()` (the identity probe, 7.11) to find the innermost coded error through a `dyn Error` chain, `ErrorRecord`, `Location` |
| L4 render | `Code`, `Category` (HTTP status, exit code), `Severity` names, `CodeInfo` (title, help, URL), public messages, `ErrorRecord` |
| L5 diagnostics | `ErrorInfo`, `Code`, `Severity` |
| L7 error service | implements `Reporter`; reads `Event`; builds `ErrorRecord`; reads every crate's `CATALOG` for the boot-time duplicate check (R23) and the runtime catalog; owns warn policy per code (R9) |
| dispatch-io, relay-io | carry L7's records; they depend on `Kinds`, `Severity`, `Code` only if James wants typed filters there (their specifications decide) |
| The security library and SECURITY-SIGNALS | `Kinds::SECURITY`, `trigger()`; security objects reach the coms bridge through the facade and L7 (James's rule: security is never isolated) |
| The suite | re-exports everything as its Tier 1 |

### 3.2 What it uses

Nothing. `core` always; `alloc` with the `alloc` feature; `std` with the `std` feature (default). No third-party crate, no first-party crate. Dev-dependencies only: `trybuild` (compile-fail tests of the macro), `proptest`, `criterion`, `loom`, and the competitor crates for benchmarks (`thiserror`, `log`, `error-forge`).

### 3.3 Where it sits in WASP

`define-err` is part of the base package every WASP crate builds on. In WASP:

- the protocol module turns a `Category` into the right refusal (HTTP status, gRPC code, database error packet) without per-code tables (the error-suite plan, 4.3);
- L4 builds RFC 9457 problem details from `CodeInfo` (`type` from the documentation URL, `title` from the catalog title, `status` from the category, `detail` from the **public** message only);
- the kernel installs L7 as the reporter in boot phase 0, before any library code runs, so library reports are never lost after boot;
- security refusals carry `Kinds::SECURITY` so the bridge (dispatch-io in process, relay-io across nodes) delivers them to every listener that asked.

### 3.4 Place in the error collection

```
 L1 define-err        (none: no_std, zero dependencies)         <- this crate
 L2 derive            (none at compile time; generated code uses L1)
 L3 report       -->  L1
 L4 render       -->  L1
 L5 diagnostics  -->  L1
 L7 service      -->  L1, dispatch-io (relay-io, L3, L4 optional)
 suite           -->  L1 to L7 + internal sinks
```

L1 is wave 1 (the error collection plan (section 5)). L2 starts only when L1's API is locked at 0.2, because L2 targets the contract in 7.5.

---

## 4. Scope

### 4.1 In scope

| Area | Functionality id | Content |
|---|---|---|
| Typed definitions | F1, R5 | `define_errors!`: enums and structs, `Display` from message templates, `core::error::Error` with `source`, `From` conversions, transparent wrapping, `#[non_exhaustive]` by default |
| Codes | F3 | `Namespace`, `Code` (packed `u64`, displayed `DB-0042`), compile-time constants, parsing |
| Catalog | F4, R10, R23 | `CodeInfo` (title, help, URL template, state), `CodeState` (active, deprecated, retired), retired numbers, compile-time duplicate check inside a block, runtime duplicate check helper |
| Severity | F5 | `Severity` on the OpenTelemetry 1 to 24 scale, named constants, project level-name tables |
| Categories | F6, R21 | `Category` (16 gRPC canonical codes, `OK` excluded), retry advice, HTTP status, gRPC code, process exit code, default severity, action, and public text |
| Metadata | F7, R21 | `ErrorInfo` trait: code, severity, category, kinds, public message, help, action, retryable, `retry_after`, location |
| Kinds | 11.3 | `Kinds` bit mask: bits 0 to 31 platform, 32 to 63 project |
| Redaction | F18 | `Secret<T>`: prints `[redacted]` |
| Owned record | R4 | `ErrorRecord` behind `alloc`, size-capped |
| Reporting facade | R1, 11.2, R9, R10 | `set_reporter` once; `report`, `notice`, `warn`, `trigger`, `deprecated`, `soft`; once per call site; `#[track_caller]`; one atomic load when nothing is installed |
| Identity probe | (enables L3, L4, L7) | stable way to read code, severity, category, kinds, and type name from any `&dyn Error` in a chain (7.11) |
| Test support | F19 (own part) | `testing` feature: a capturing reporter per thread |

### 4.2 Out of scope, and where each item lives instead

| Item | Lives in |
|---|---|
| `#[derive(Error)]`, `#[error("...")]` attribute syntax, doc-comment messages (displaydoc style), generic and lifetime-heavy error types beyond 7.4's limits | L2 derive |
| Dynamic errors, `.context()`, `bail!`, `ensure!`, downcasting helpers, chain walking beyond `identify`, collectors, backtraces, `main` integration, propagation traces | L3 report |
| Console, JSON, logfmt, problem-details output; `NO_COLOR`; localisation | L4 render |
| Source spans and snippets | L5 diagnostics |
| Listeners, filters per listener, queued and urgent delivery, storm control, runtime catalog registry, severity remapping by configuration, warn policy per code, event rules, probes, crash capture, panic hook, breadcrumbs, event ids, timestamps | L7 error service, dispatch-io, relay-io |
| Sinks to log-io, metrics-lib, audit-trail | suite folders |
| Retry, backoff, circuit breaking | throttle-net, task-io |
| Wiping secrets from memory | crypt-io (`Secret<T>` here only redacts output) |
| `serde` support | not in 1.0 (OD-17); L4 writes JSON without it |

---

## 5. Requirements

### 5.1 Functional requirements

| Id | Requirement |
|---|---|
| FR-1 | A plain error type (no code, severity, category, or kind) is written with `define_errors!` in the shape of 7.4 and generates exactly: the type, `Debug`, `Display`, `core::error::Error` with `source`, the `From` impls asked for, an `ErrorInfo` impl whose methods return the defaults, and the identity-probe `description` (7.11). Nothing runs at construction. |
| FR-2 | Messages are compile-time format strings. Named fields are referenced by name (`{page}`, `{page:?}`, `{page:#010x}`); tuple fields are referenced through the binding name given in the definition (`Io(source: io::Error) => "{source}"`); extra arguments use the parenthesised form `=> ("...", expr, name = expr)`. |
| FR-3 | `#[from]` on the single non-location field of a variant generates `From<T>`; the `From::from` impl is `#[track_caller]` and fills a `#[location]` field when present. |
| FR-4 | A field marked `#[from]` or `#[source]`, or named `source`, is returned by `Error::source()`; `Option<E>` and boxed `dyn Error` sources are supported. |
| FR-5 | `=> transparent` forwards `Display` and `source()` to the single field, as `#[error(transparent)]` does in thiserror. |
| FR-6 | Every public enum generated by `define_errors!` is `#[non_exhaustive]` unless the type carries the `#[exhaustive]` marker. |
| FR-7 | Codes, severity, category, kinds, public message, action, retryable override, and `retry_after` are optional per variant; their absence costs nothing at run time and adds no code beyond the default-returning `ErrorInfo` impl. |
| FR-8 | A `Code` is a namespace (1 to 6 characters, A to Z and 0 to 9, starting with a letter) plus a number (0 to 16,777,215), packed in one `u64` by the layout of 7.6, displayed `NS-0042` with a per-namespace digit width (default 4), and parsed back losslessly. |
| FR-9 | Within one `define_errors!` block, duplicate code numbers (including numbers listed as retired) are a compile error. |
| FR-10 | A code requires help text (`#[code(n, help = "...")]`); a code without help is a compile error (the error-suite plan, E-7). |
| FR-11 | Each block that defines codes generates a `CATALOG: &'static [CodeInfo]` per type, including retired numbers, with title, help, documentation URL template, severity, category, kinds, and state. |
| FR-12 | A deprecated code (`#[code(n, help = "...", deprecated = "2.1", replacement = m)]`) keeps working, reports `CodeState::Deprecated`, and its named constant carries `#[deprecated]`. |
| FR-13 | `Severity` holds 1 to 24 (OpenTelemetry SeverityNumber), compares as a number, displays as the OpenTelemetry short name or the portfolio label, and is checked at compile time when written as a literal. |
| FR-14 | `Category` has the 16 non-OK gRPC canonical codes and maps each to an HTTP status, a gRPC code, a process exit code, a retry advice, a default severity, a default action, and a default public message (7.8). |
| FR-15 | `Kinds` is a `u64` bit mask: bits 0 to 31 reserved for platform kinds with fixed meaning, bits 32 to 63 for project kinds made with `Kinds::project::<N>()`, `N` checked at compile time to be below 32. |
| FR-16 | `ErrorInfo::is_retryable()` is true exactly when the category is transient, unless the variant overrides it; `retry_after()` returns the static per-variant value or the `#[retry_after]` field. Only idempotent operations may retry automatically, which the documentation states and `Category::may_retry_automatically(idempotent)` encodes (AIP-194). |
| FR-17 | `Secret<T>` prints `[redacted]` under `Debug` and `Display` with any flags, gives the value back only through `expose()` and `into_inner()`, and never implements `Deref`. |
| FR-18 | `set_reporter(&'static dyn Reporter)` installs a reporter once per process; a second call returns `Err(SetReporterError)` naming the file and line of the first installation. |
| FR-19 | `report(&err)`, `notice(msg)`, `warn(msg)`, `trigger(kinds, msg)`, `deprecated(item)`, and `soft(err)` are `#[track_caller]` functions; their macro forms (`notice!`, `warn!`, `trigger!`, `deprecated!`) accept format arguments. Every verb records the caller's file, line, and column, never the helper's. |
| FR-20 | With no reporter installed, every verb costs one atomic load and returns; it never allocates, formats, or blocks. |
| FR-21 | `warn` and `deprecated` deliver at most once per call site by default; the global warn mode can switch to always, ignore, or error. A warning issued before a reporter is installed does not consume its call site's once flag. |
| FR-22 | `soft(err)` returns `Ok(())` after warning, or `Err(err)` when the warn mode is `Error`. |
| FR-23 | Verbs never change control flow, never panic, and never call the reporter recursively on the same thread (with `std`); a panic inside the reporter is caught and dropped (with `std`). |
| FR-24 | `ErrorRecord` (feature `alloc`) is an owned, `Send + Sync + Clone` snapshot built from any `&dyn Error`: code, severity, category, kinds, public message, the chain of private messages with each entry's code and type name, fields, location trail, optional backtrace text, optional event id and time, and the handled flag; every part is bounded by `RecordLimits`. |
| FR-25 | `identify(&dyn Error)` reads the identity of any error generated by `define-err` or L2 through a stable-Rust mechanism (7.11), so L3 and L4 can find codes inside chains of foreign wrappers. |
| FR-26 | The crate builds and passes its tests with `--no-default-features` (core only), with `alloc`, and with `std`, on hosted and embedded targets (7.17). |
| FR-27 | Generated code never names a type the user can shadow: every path is absolute (`::core::...`, `$crate::...`), so a user enum named `Error`, `Result`, `Option`, `Box`, or `Display` works. |
| FR-28 | Compile errors from misuse of `define_errors!` name the problem and the fix in one sentence (for example "`#[code]` needs `help = \"...\"`: every code ships with its explanation"). |

### 5.2 Non-functional requirements

| Id | Requirement | Measured by |
|---|---|---|
| NFR-1 | Zero runtime dependencies in every feature combination. | `cargo tree`, `cargo deny` |
| NFR-2 | A type generated from a plain definition has the same size, layout, and `Display` speed as the equivalent hand-written or thiserror type. | size assertions, criterion against thiserror |
| NFR-3 | No allocation in any path except `ErrorRecord` construction and `set_boxed_reporter`. | counting-allocator tests |
| NFR-4 | Facade with nothing installed: one atomic load, at most 1 ns [est. target]. | criterion, assembly inspection in `docs/PERFORMANCE.md` |
| NFR-5 | Clean build of a crate using `define-err` at least 20% faster than the same crate using thiserror (which pulls syn, quote, proc-macro2). | timed clean builds, published |
| NFR-6 | `define_errors!` handles an enum of 500 variants at the default `recursion_limit`. | UI test |
| NFR-7 | No panic reachable at run time from any public function. Panics exist only in constant evaluation, where they are compile errors (7.13). | `clippy::panic`, `unwrap_used`, `expect_used` denied outside tests; review of every `panic!` site |
| NFR-8 | `unsafe` confined to one module (`src/facade/slot.rs`), under 40 lines, Miri and loom clean. | `#![deny(unsafe_code)]` at the root, CI Miri and loom jobs |
| NFR-9 | Every public type is `Send + Sync` where its contents allow; `Event` and `Reporter` impls are usable from any thread. | compile-time assertions |
| NFR-10 | Deterministic: the packed code, its text, the probe string, and every mapping table are identical on every platform and every 1.x release. | golden tests, frozen in `docs/STABILITY-1.0.md` |
| NFR-11 | `no_std` builds for `thumbv7em-none-eabihf` (with CAS), `thumbv6m-none-eabi` (no CAS: facade compiled out), `wasm32-unknown-unknown`, and `riscv32imac-unknown-none-elf`. | CI cross-check job |
| NFR-12 | Every compile-error path of the macro has a `trybuild` test with a checked message. | `tests/ui/` |
| NFR-13 | The public API surface fits on one page of `docs/API.md` per module; Tier 1 is learnable from the README alone. | 0.2 design review, 1.0 audit (standard 2.10) |
| NFR-14 | Total source under 5,000 lines (ERROR-SUITE estimate 3 to 5k); crossing 6,000 triggers OD-6 (split codes into their own crate). | line count per release |

### 5.3 Enterprise and distributed

Standard 2.16 applies. **`define-err` is a stateless crate** (definitions, codes, tables, formatting), with one exception: the facade's process-wide reporter slot. The stateless row of 2.16 is met as follows, and the slot is explained after it.

| 2.16 point | How it is met |
|---|---|
| Deterministic, no process-local assumptions | A `Code` is a pure value: the same `u64` and the same text on every node, every platform, every 1.x release (NFR-10). Category mappings, severity numbers, kind bits, and the probe format are fixed tables. Nothing depends on the process, the clock, the host, or a registry. |
| Safe to run the same version on many nodes | Codes need no runtime registration to be compared, displayed, or parsed, so two nodes agree on `DB-0042` without talking. A code parsed from another node's log or wire message (`Code::parse`, `Code::from_u64`) is validated and round-trips. |
| Different versions on different nodes (rolling upgrades) | Codes are never reused (retired numbers stay in the catalog and in the duplicate check); a newer node's unknown code still parses and still has a category the older node can act on, because the category travels with the event (L7) and only the catalog text is missing. |
| Correct under concurrency and heavy load | Generated types are plain data, `Send + Sync` when their fields are. The facade is lock-free: one acquire load on the fast path, compare-and-swap only at installation, atomic flags per call site; tested with loom (12.5) and under 64 threads (12.8). |
| Observable | This crate is the observability hook for every library: stable codes, categories, kinds, and the facade cost nothing when unused (FR-7, FR-20). |
| Operable | Every bound is a documented, configurable value with a safe default: `RecordLimits`, the once-per-site table size, the minimum severity and kind mask of the facade filter. Overload behaviour: verbs never block and never allocate; a flood costs each caller one load plus the reporter's own work, and L7 does storm control. |
| Secure and compliant | Public and private messages are separate; `Secret<T>`; no secret in `Display` or `Debug` of any type here; the first installer of the reporter is recorded and visible (T7). |
| Stable for long-lived systems | SemVer; `#[non_exhaustive]` everywhere it can grow; deprecation before removal (including codes, R10); MSRV policy in `docs/STABILITY-1.0.md`; the packed code layout and probe format frozen at 1.0. |

**The reporter slot.** It holds a pointer to the application's reporter, set once at boot. It is process-local on purpose: it is the entry point into the process's error service (L7), and distribution is that service's job (dispatch-io inside the process, relay-io across processes and nodes). `define-err` makes no assumption about where events go after `Reporter::receive`, so the same library code works in a single binary, in a multi-process deployment, and in a cluster. Under a network partition nothing in this crate changes: the reporter is local, and each node keeps reporting locally (L7 and relay-io define buffering and replay).

---

## 6. Standards

Copied from the portfolio library standard (section 2), adapted where `define-err` genuinely differs. Each adaptation is marked **Adapted**.

### 6.1 The bar

- **Best of its kind.** Measured against the best existing libraries in any language: at least **20% faster** than the best Rust competitor at **equal or stronger security**, with the numbers measured and published, never asserted. **Adapted:** a generated `Display` is one `write!` call, the same instructions thiserror produces, so "20% faster" cannot apply there; the target for generated code is "identical", proven by benchmark and size assertions. The 20% bar applies where `define-err` does work of its own: clean build time against thiserror (NFR-5), the facade against the `log` crate's disabled path, code parsing and catalog lookup against error-forge's string codes and `RwLock<HashMap>` registry (section 11).
- **Production-ready from the first line.** No MVPs, no "good enough", no placeholders in shipped code, no deferring a hard task to a later version unless the roadmap records the move and the reason.
- **Push for perfection.** "Flawless" is not attainable; the standard is to make whole classes of failure impossible by design, and to prove the rest by tests. **Adapted:** the classes made impossible here are reused codes, duplicate numbers in a block, out-of-range severities and kind bits, codes without help, and secret leakage through `Display`: each is a compile error or cannot be expressed.

### 6.2 Performance

- No allocation on hot paths; prove it with a counting allocator in tests. **Adapted:** the hot paths are error construction, `Display`, every `ErrorInfo` method, `Code` display and parse, `identify`, and every facade verb. Only `ErrorRecord` construction and `set_boxed_reporter` allocate.
- Zero-copy wherever the data allows (borrowed views into input buffers). **Adapted:** `Event` borrows the error and the message arguments; nothing is copied unless the reporter asks for an `ErrorRecord`.
- Bounded work per call; no quadratic behaviour on any input. **Adapted:** the compile-time duplicate check is quadratic in the number of codes in one block, at compile time only; a 500-code block is a test case (NFR-6).
- No locks on hot paths; where shared state is unavoidable, read-mostly snapshots, atomics, or sharding, measured.
- Benchmarks (criterion) for every hot path, with baselines; a regression over 5% blocks the change.
- `#[inline]` only where measured; no premature micro-optimisation that costs clarity without a number.

### 6.3 Efficiency

- Zero CPU when idle: no polling loops, no busy waiting, no timers that fire for nothing.
- Memory bounded by configuration, never by input. **Adapted:** the once-per-site table is a fixed static (7.14.6); `ErrorRecord` is capped by `RecordLimits`.
- Buffers and objects reused (pools) where measured to matter. **Adapted:** not applicable; nothing is buffered.
- Lowest energy per operation is a stated goal: fewer instructions, fewer wake-ups, fewer system calls. **Adapted:** no system calls at all.

### 6.4 Security

- **Fail closed**: when a check cannot run, the answer is no. **Adapted:** a code, namespace, severity, or kind that cannot be validated does not compile; at run time `Code::parse` and `Code::from_u64` reject anything not exactly valid; a public message that was not written falls back to the category's generic text, never to the private `Display`.
- Every budget (bytes, counts, depth, time) is checked **before** allocation or work. **Adapted:** `ErrorRecord` checks chain depth and byte budgets before each allocation, through a bounded writer.
- Every parser and decoder has a fuzz target; every threat in the specification has an attack test that must fail to get through. **Adapted:** the parsers are `Code::parse`, `Code::from_u64`, `Severity::parse`, `Category::parse`, and the probe decoder.
- Constant-time comparison for anything secret; secrets wiped after use; no secret in `Debug`, `Display`, errors, or logs. **Adapted:** `Secret<T>` provides the last point; comparison and wiping are crypt-io's job, and `Secret<T>` deliberately implements neither `PartialEq` nor `Deref`.
- Checked or saturating arithmetic on every value derived from untrusted input.
- "A security component that cannot name the attack it stops, and prove it with a test, does not ship."
- Every security measure in the documentation states its cost (latency, memory), estimates labelled as estimates.
- At least one new defence idea per version, recorded as hypothesis, method, and expected cost in the plan's ideas section.

### 6.5 Stability and durability

- No panics in library code: `Result` everywhere; no `unwrap`, `expect`, `todo!`, `unimplemented!`, `unreachable!` outside tests. **Adapted:** `panic!` appears only inside constant evaluation (inline `const` blocks and `const` items), where it is a compile error, never a run-time panic (7.13). Each such site is listed in `docs/ARCHITECTURE.md`.
- Every error variant documented, tested, and actionable ("say what to do"); error enums `#[non_exhaustive]`. **Adapted:** this crate makes the second half the default for its users (FR-6).
- Deterministic behaviour: the same input gives the same output on every platform.
- Graceful degradation under overload: refuse cleanly, never fall over.
- Where state is persisted: crash-safe writes, checksums, tested by killing the process mid-write. **Adapted:** nothing is persisted.

### 6.6 Concurrency and heavy load

- Types are `Send` and `Sync` where that makes sense, checked at compile time.
- No global mutable state. **Adapted:** the facade has exactly three globals, by necessity (the `log` crate pattern): the reporter slot, the packed control word, and the once-per-site table. Each is atomic, documented, and loom-tested; nothing else is global.
- Contended paths are lock-free or sharded; concurrency primitives are tested with `loom`.
- Bounded queues everywhere; back-pressure is explicit; nothing grows without a limit. **Adapted:** no queues; the facade hands each event to the reporter synchronously.
- Sustained-load and soak tests (hours) before 1.0; no memory growth over the soak. **Adapted:** the soak drives every verb from 64 threads for 8 hours against a counting reporter and checks flat memory and exact once-per-site counts.

### 6.7 Cross-platform

- Linux, macOS, and Windows are first class, on stable and the MSRV, in CI. FreeBSD where the crate touches OS event mechanisms. **Adapted:** no OS is touched; FreeBSD is not needed.
- Every `#[cfg(unix)]` path has a Windows counterpart or a documented, tested reason why not. **Adapted:** there is no `cfg(unix)` code. The only `cfg`s are features and `target_has_atomic`.
- `no_std` (with or without `alloc`) for cores that do not need the OS, checked in CI. **Adapted:** required, with embedded cross-checks (NFR-11).

### 6.8 Dependencies and runtime

- **Zero third-party runtime dependencies.** Exceptions, each recorded with its reason: audited cryptography reached through `crypt-io`; `rustls` inside `tls-net` (TLS is never written in-house). **Adapted:** no exception applies; zero dependencies of any kind, first-party included, because every first-party crate will depend on this one.
- First-party crates are used by **published version**, never `path = "../..."` (each crate is its own repository and CI checks it out alone).
- **No async runtime** (no tokio, no async-std) in any crate; cores are sans-IO and work with any driver. The REPS clause "MUST use tokio" is a known mistake and is recorded as an approved exception in each crate's plan.
- Optional adapters for other ecosystems only behind non-default features, and never required. **Adapted:** none in 1.0 (OD-17).
- Dev-dependencies (criterion, proptest) are allowed. **Adapted:** also `trybuild`, `loom`, and the competitors `thiserror`, `log`, `error-forge` for benchmarks.

### 6.9 Unsafe code

- `#![forbid(unsafe_code)]` unless the crate must call the operating system where `std` has no API. **Adapted:** this crate calls no OS, but the `no_std` reporting facade (R1) must store a `&'static dyn Reporter` in a static that is written once at run time, and stable Rust has no safe `no_std` cell for that (`OnceLock` needs `std`; `core::cell::OnceCell` is not `Sync`). The `log` crate solves the same problem with a `static mut` and an atomic state machine. OD-3 records the choice; the recommendation is one small module.
- Such crates confine `unsafe` to one `sys` module per platform, every block with a `// SAFETY:` comment, every `unsafe fn` with a `# Safety` section, `deny(unsafe_op_in_unsafe_fn)`, and tests (Miri where it can run). **Adapted:** the one module is `src/facade/slot.rs` (7.14.2): a `Sync` wrapper around `UnsafeCell<Option<&'static dyn Reporter>>`, one write under the `INSTALLING` state, reads only after an acquire load observes `READY`. Three `unsafe` sites in total (the `unsafe impl Sync`, the one write, the read), each with a `SAFETY:` comment. Crate root `#![deny(unsafe_code)]` with `#[allow(unsafe_code)]` on that module only. Miri runs the whole test suite (the crate has no FFI); loom models install and read.

### 6.10 API design

- **MUST: a simplified API, easy to use and easy to read.** This is a hard requirement from James (stated three times, 2026-10-22), not a preference. A specification whose common uses need more than a line or two of obvious code, or whose names a newcomer cannot read without the docs, fails review. Simplicity is checked at the 0.2 design lock and again at the 1.0 audit. **Adapted (how it is met):** a plain error is one line per variant (`Name { field: Type } => "message"`); every extra is one attribute whose name is a plain word (`code`, `category`, `severity`, `public`, `kinds`, `retry_after`); the verbs are four plain function names called by path; nothing needs a builder, a configuration object, or a type parameter for common use, and the one import a caller needs for metadata methods is `use define_err::prelude::*;`. No inherent methods are generated (they would collide with methods users write on their own types), so `ErrorInfo` comes from the prelude. Section 8.2 shows the Tier 1 surface on one screen.
- **Tier 1 is mandatory**: free functions or one-line constructors that cover about 80% of uses with no builder, no configuration object, and no type parameters the caller must name. Documented first, in the README and in rustdoc. **Adapted:** Tier 1 is `define_errors!` plus the verb functions.
- **Tier 2**: builders and configuration for the next 15%. **Adapted:** facade filter and warn mode, `RecordLimits`, level-name tables, the boxed reporter.
- **Tier 3**: the full, generic, allocation-free surface for the last 5% and for other libraries. **Adapted:** `identify`, `Code::from_u64`, `Kinds::from_bits`, `Event` construction, and the `__private` items L2 targets (documented in `docs/GENERATED-CODE.md`, not in the README).
- Extensible enums and structs are `#[non_exhaustive]` from the first release; `#[must_use]` where dropping a value is a bug; `pub(crate)` by default.
- Names are plain words; no abbreviations a reader must decode.
- Every public item has a doc comment with at least one runnable example.

### 6.11 Testing

- Unit tests for every function and every `Ok` and `Err` path, at the bottom of each file; integration tests in `tests/`.
- Test names: `test_<subject>_<condition>_<expected>`.
- Edge cases for every input: empty, one, maximum, maximum plus one, zero, negative where possible, boundaries (0, 1, n-1, n, n+1), invalid encodings, truncated input, input split at every byte boundary (for streaming parsers), concurrent use, failure in the middle of an operation, corrupted data.
- Property tests (proptest) for every invariant (round trips, "never panics", "never exceeds budget").
- Fuzz targets (cargo-fuzz) for every parser and decoder, with an out-of-repo corpus.
- Conformance suites and RFC test vectors where they exist. **Adapted:** the conformance suite is thiserror's own behaviour: every thiserror test case that `define_errors!` can express is ported and must produce byte-identical `Display` output and the same `source()` chain.
- Counting-allocator tests for every "no allocation" claim.
- `loom` for every concurrency primitive. **Adapted:** the slot, the control word, the per-site flags, and the once-per-site table.
- CI: Linux, macOS, Windows on stable and MSRV; `no_std` build where applicable. **Adapted (added):** compile-fail tests (`trybuild`) for every macro error path; Miri over the full suite.

### 6.12 Documentation

- README from `README_TEMPLATE.md`: header image, badges including MSRV, pitch, features, install, quick start (Tier 1 first), status, documentation links, license, copyright footer. Never remove the header image, the MSRV badge, or the footer.
- `docs/API.md` in the format of `https://github.com/jamesgober/metrics-lib/blob/main/docs/API.md`: nested table of contents, every public item, description, parameters, errors, several examples per use case.
- By 1.0: `docs/ARCHITECTURE.md`, `docs/PERFORMANCE.md` (measured numbers, method, hardware), `docs/SECURITY.md` (threat model), `docs/PLATFORM-NOTES.md`, `docs/STABILITY-1.0.md`. **Adapted (added):** `docs/GENERATED-CODE.md` (the contract L2 targets, 7.5), `docs/CODES.md` (layout, catalog rules, never-reuse policy), `docs/FACADE.md` (the reporter contract), `docs/MIGRATION.md` (from thiserror and from error-forge).
- `CHANGELOG.md` in Keep a Changelog 1.1.0 format, updated in the same change as the code.
- Release notes per version in `docs/release/vX.Y.Z.md`.
- Writing: plain, specific, human prose; past tense in release notes; every claim matches the build; none of the four marketing words banned in the portfolio library standard (2.12); no em dash (U+2014); no emoji; never a reference to `.dev/` in public files. **Adapted (added):** no emoji or decoration in any generated `Display` text either (error-forge printed emoji captions; 7.3).

### 6.13 Repository and tooling

- Layout of the portfolio model scaffold: `.claude/CLAUDE.md` and `.dev/` (both gitignored), `.editorconfig`, `.gitattributes` (`* text=auto eol=lf`), `.github/workflows/ci.yml`, `.gitignore` (also `Cargo.lock`), `benches/`, `examples/`, `tests/`, `Cargo.toml`, `CHANGELOG.md`, `clippy.toml`, `deny.toml`, `rustfmt.toml`, `dev/DIRECTIVES.md`, `dev/PLAN.md`, `dev/ROADMAP.md` (all committed), `docs/`, `LICENSE-APACHE`, `LICENSE-MIT`, `README.md`, `REPS.md` (byte-identical copy), `src/`. **Adapted (added):** `tests/ui/` (trybuild cases), `fuzz/` (corpus out of repo), `tests/conformance/` (ported thiserror cases).
- `Cargo.toml` in the portfolio scaffold layout: edition 2024, MSRV 1.85 unless the specification justifies more, `Apache-2.0 OR MIT`, author `James Gober <me@jamesgober.com>`, release profile `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"`. **Adapted:** MSRV **1.81** and edition **2021** (OD-2): this is the crate every other crate depends on, so its MSRV caps theirs; 1.81 is the lowest version with `core::error::Error`, and four of the crates to migrate (config-lib 1.75, kraken 1.80, mod-cli 1.81, noml-rust 1.82) sit below 1.85. If James chooses 1.85, edition 2024 follows and section 19 lists the MSRV raises.
- Crate root: `#![forbid(unsafe_code)]` (unless 2.9 applies), `#![deny(missing_docs)]`, `#![cfg_attr(docsrs, feature(doc_cfg))]`, `pub const VERSION`. **Adapted:** `#![no_std]` with `extern crate alloc` / `extern crate std` per feature; `#![deny(unsafe_code)]` with the one allowed module (6.9).
- CI as in the portfolio scaffold: matrix Linux/macOS/Windows × stable/MSRV; fmt; clippy with default, all, and no default features under `-D warnings`; tests in the same three modes; examples build; `cargo doc` under `RUSTDOCFLAGS=-D warnings`; a security job with `cargo audit` and `cargo deny check`. **Adapted (added):** a fourth feature mode (`--no-default-features --features alloc`); the `no_std` cross-check job (NFR-11); Miri; loom; trybuild (on stable only, because compiler messages differ by version; the MSRV job runs the pass cases).
- Local gate before every push, on Windows and in WSL Ubuntu, stable and MSRV: fmt, clippy (all three feature modes), tests, `cargo doc` with default **and** all features, `cargo package` in a fresh target directory. Never pipe check commands through `tail`. **Adapted:** clippy and tests in all four feature modes; `cargo +1.81 clippy` locally, not only build (clippy lints differ by toolchain).

### 6.14 Versions

| Version | Milestone |
|---|---|
| 0.1 | scaffold: repository, plan, CI green |
| 0.2 | design lock: API written in `docs/API.md` and approved |
| 0.3 | core |
| 0.4 | extended |
| 0.5 | feature complete |
| 0.6 | optimisation: benchmarks against competitors |
| 0.7 | hardening: fuzzing, attack tests, soak |
| 0.8 | alpha |
| 0.9 | beta and release candidates |
| 1.0 | stable: API frozen until 2.0, full audit passed |

No series is skipped. An MSRV raise is a major change after 1.0. **Adapted:** 0.2 also locks `docs/GENERATED-CODE.md`, because L2 starts from it; 0.3 is the thiserror-parity core, so migrations can begin early (section 14).

### 6.15 Git and authorship

- Commits as `James Gober <me@jamesgober.com>` (`git -c user.name=... -c user.email=...`). No co-author lines; never "Claude", "AI", "generated", or "Anthropic" anywhere.
- Stage explicit paths, never `git add -A`. Check `git log origin/main..HEAD --format='%an <%ae> | %B'` before every push.
- Never `--no-verify`, never force-push, never delete branches or tags without asking.
- Every design decision goes into `dev/PLAN.md`'s decision log in the same change.

---

## 7. Design and architecture

### 7.1 Module tree

```
src/
  lib.rs              crate docs, lint attributes, features, VERSION, re-exports, prelude
  code.rs             Namespace, Code, CodeParseError; packing (7.6); code! and namespace! macros
  severity.rs         Severity, LevelNames; OpenTelemetry short names (7.7)
  category.rs         Category, RetryAdvice, Action; every mapping table (7.8)
  kinds.rs            Kinds, platform constants, Kinds::project::<N>() (7.9)
  info.rs             ErrorInfo trait, CodeInfo, CodeState, UrlDisplay, catalog helpers (7.10, 7.12)
  probe.rs            identity probe: const encoder, decoder, Identity, identify() (7.11)
  secret.rs           Secret<T> (7.16)
  location.rs         Location alias, here() (7.10)
  record.rs           ErrorRecord, RecordLimits, ChainEntry, bounded writer   [feature alloc] (7.15)
  facade/
    mod.rs            set_reporter, set_boxed_reporter, reporter(), flush(), filter and warn mode
    slot.rs           the only unsafe code: the reporter slot (7.14.2)
    control.rs        packed control word, kind mask (7.14.3)
    event.rs          Event, Verb
    verbs.rs          report, notice, warn, trigger, deprecated, soft
    sites.rs          Site (per-call-site flag), the once-per-site table (7.14.6)
    guard.rs          reentrancy guard and panic isolation   [feature std]
  macros/
    define_errors.rs  define_errors! entry point and parser (7.4)
    emit.rs           code emission arms (7.5)
    verbs.rs          notice!, warn!, trigger!, deprecated!
  private.rs          #[doc(hidden)] __private: source_of autoref dispatch, const checks, probe
                      encoder, everything generated code and L2 call (7.5.4)
  testing.rs          capture() and CaptureReporter   [feature testing, needs std]
```

Fifteen source files, 3,000 to 5,000 lines [est.]. Each top-level module is `pub(crate)` internally and re-exported at the crate root; a CI check (as in the error collection plan, 3.3) fails if `code`, `severity`, `category`, or `kinds` import anything from `facade`, `record`, or `macros`, which keeps OD-6 (a later split of codes into their own crate) a mechanical move.

### 7.2 Features

| Feature | Default | Enables | Needs |
|---|---|---|---|
| `std` | yes | `ErrorInfo` and `Category` mapping for `std::io::Error` / `io::ErrorKind`; reentrancy guard and panic isolation in the facade; `testing` becomes possible | `alloc` |
| `alloc` | via `std` | `ErrorRecord`, `RecordLimits`, `set_boxed_reporter`, boxed `dyn Error` sources in `source_of` | none |
| `testing` | no | `testing::capture`, `CaptureReporter` | `std` |

Every feature gates something real and CI checks each mode (error-forge kept five features that gated nothing; 7.3). Generated code never contains `#[cfg(feature = ...)]`, because such a `cfg` would test the **user's** crate features, not this crate's (7.5.6).

### 7.3 Lessons from error-forge 1.0.2

`error-forge` (3,911 lines) is James's earlier crate. What `define-err` keeps and what it fixes:

| error-forge | Problem | `define-err` |
|---|---|---|
| depends on `pastey` (to lowercase constructor names) and `parking_lot`; derive crate on `syn` 1 | third-party runtime dependencies, build time | zero dependencies; no generated constructors, so no case conversion is needed |
| `#[kind(...)]` required on every variant | plain use is not free | every attribute optional (FR-7) |
| metadata generated as **inherent** methods; the macro does not implement the `ForgeError` trait (its README tells users to write a delegating impl) | generated types do not plug into the rest of the system | generated code implements `ErrorInfo` (7.5.2) |
| hook fired from every generated constructor (`call_error_hook`) | creation has side effects and costs a lock-free load plus a call per error, even for errors handled two lines later; OpenTelemetry advises reporting at boundaries | construction has no side effects; reporting is explicit (`report`) |
| error level guessed from flags (`if kind == "Warning"`) | wrong levels, string compares | explicit `Severity`, defaults from the category |
| string codes (`"AUTH-001"`) in `CodedError<E>`; global `RwLock<HashMap<String, ErrorCodeInfo>>` registry | allocation per coded error, lock per lookup, typos found at run time | `Code` is a `u64` constant; the catalog is a `&'static [CodeInfo]` built at compile time |
| `status_code()` and `exit_code()` set per variant by hand, defaulting to 500 and 1 | inconsistent mappings across crates | derived from the category (7.8), overridable only through the category |
| a pictograph prefix in `Display` (`AppError::config` printed a gear symbol before "Configuration Error: ...") | logs and terminals garbled; tests brittle | `Display` is exactly the user's message |
| `async` feature adds a second trait with the same method names (E0034 ambiguity across the build) | feature unification breaks callers | one trait, no async variant; errors are plain data |
| exported `Result<T>` alias shadowing `std::result::Result` in globs | quiet bugs | no `Result` alias exported, ever |
| `ErrorSource` trait users must implement for custom source types | friction | autoref dispatch accepts any `Error`, `Option<E>`, and boxed `dyn Error` (7.5.4) |
| five features (`console`, `backtrace`, `registry`, `collector`, `context`) that gate nothing | misleading manifest | every feature gates something, checked in CI |
| **kept:** re-entrancy guard and panic isolation around the hook | | same, in the facade (7.14.7) |
| **kept:** compile errors that name the unknown tag and list valid ones | | FR-28 |
| **kept:** pass-through of doc comments and other attributes; optional trailing comma | | 7.4 |
| **kept:** a fast non-recursive matcher with a slower general path | large enums | 7.4.5 |

Recovery (retry, backoff, circuit breaker), collectors, context, console output, and logging adapters leave for L3, L4, L7, throttle-net, and task-io.

### 7.4 The `define_errors!` grammar

#### 7.4.1 Shape

```
define_errors! {
    [ namespace = "<NS>" [, width = <1-8>] ; ]        // needed only if a code is used
    [ docs = "<URL template containing {code}>" ; ]  // optional
    [ retired = [ <n>, <n>, ... ] ; ]                // optional; numbers never reused

    <type definition>+
}

<type definition> :=
    <outer attributes>  <vis> enum   <Name> { <variant>, ... [,] }
  | <outer attributes>  <vis> struct <Name> { <field>, ... [,] }   => <message> ;
  | <outer attributes>  <vis> struct <Name> ( <tuple field>, ... ) => <message> ;
  | <outer attributes>  <vis> struct <Name>                        => <message> ;

<variant> :=
    <variant attributes>  <Name>                              => <message>
  | <variant attributes>  <Name> { <field>, ... [,] }         => <message>
  | <variant attributes>  <Name> ( <tuple field>, ... [,] )   => <message>

<field>       := <field attributes> [<vis>] <name> : <Type>
<tuple field> := <field attributes> <binding> : <Type>
               | <field attributes> <Type>              // unnamed: the message cannot use it

<message> := "<format string>"                       // fields by name: {page}, {page:?}, {crc:#010x}
           | ( "<format string>", <argument>, ... )  // extra arguments; fields are in scope by name
           | transparent                             // forward Display and source() to the only field

<outer and variant attributes, any order, each optional>:
    /// doc comments                       passed through
    #[code(<n> [, name = <IDENT>], help = "<text>" [, title = "<text>"]
               [, deprecated = "<since>" [, replacement = <n>]])]
    #[category(<Category variant>)]        e.g. Unavailable, DataLoss
    #[severity(<named level> | <1-24>)]    e.g. ERROR, CRITICAL, 22
    #[kinds(<KIND> | <KIND> | ...)]        platform names or the project's own constants
    #[public("<text>")]                    message a client may see
    #[action(<Action variant>)]            e.g. FixConfig, PageOnCall
    #[retryable] | #[retryable(false)]     override the category's transient flag
    #[retry_after(secs = <n>)] | #[retry_after(ms = <n>)]
    #[exhaustive]                          (types only) do not add #[non_exhaustive]
    #[manual_debug]                        (types only) do not derive Debug
    #[cfg(...)], #[allow(...)], #[doc(...)], any other attribute: passed through

<field attributes>:
    #[from]          From<Type> for this variant; implies #[source]
    #[source]        this field is the cause returned by source()
    #[location]      filled with the caller's location by From::from; type must be Location
    #[retry_after]   this field (Duration or Option<Duration>) is the retry_after value
    /// doc comments and other attributes: passed through
```

Rules:

1. A field named `source` is the source without an attribute (thiserror rule).
2. At most one source field per variant; at most one `#[location]`; at most one `#[retry_after]` field.
3. A `#[from]` variant holds the source field plus, optionally, one `#[location]` field and nothing else (thiserror's rule, with location in place of backtrace).
4. `=> transparent` needs exactly one field besides an optional `#[location]`; it may be `#[from]`.
5. `#[code]` needs a `namespace` header and `help`. Numbers are integer literals from 0 to 16,777,215; number 0 is the namespace's generic code (the error-suite plan, 4.3).
6. `name = IDENT` adds an associated constant `Type::IDENT: Code`; without it, the code is reached through `err.code()` or the catalog.
7. Metadata on a struct goes on the struct's outer attributes.
8. Messages are required. An error with no message is a bug a reader cannot diagnose.
9. A block may hold several types; they share the namespace and the duplicate check.

#### 7.4.2 Why messages come after `=>` and tuple fields are named

The attribute form `#[error("...")]` is L2's (attribute macros may only sit where the compiler expects attributes, and `thiserror` users already write it). `define_errors!` is a declarative macro, which reads tokens in any order, so it puts the message where a reader looks for it: at the end of the line that defines the variant. One placement per crate, no choice to make (OD-12).

thiserror's `{0}` works because its proc macro rewrites the string. A `macro_rules!` macro cannot look inside a string literal, and Rust rejects unused positional format arguments, so `{0}` cannot be supported. Instead a tuple field gets a binding name in the definition (`Io(source: io::Error) => "{source}"`): the enum is still a tuple variant (`Io(io::Error)`), and the message reads like the named-field case. A message that names no tuple field may leave the fields unnamed (`Io(#[from] io::Error) => "i/o error"`). Format implicit capture finds the bindings because both the binding identifier and the string literal come from the user's tokens, so they share hygiene; error-forge relies on the same behaviour.

#### 7.4.3 Examples: plain (thiserror parity)

```rust
use define_err::define_errors;

// 1. Struct-like variants, unit variant, format specs.
define_errors! {
    #[derive(Clone, PartialEq, Eq)]
    pub enum PageError {
        NotFound { page: u64 } => "page {page} not found",
        BadMagic { found: u32, expected: u32 } =>
            "bad page magic: found {found:#010x}, expected {expected:#010x}",
        InvalidSuperblock => "page 0 is not a valid allocator superblock",
    }
}

// 2. Sources and From.
define_errors! {
    pub enum LoadError {
        Io(#[from] source: std::io::Error) => "cannot read the file: {source}",
        Parse { line: u32, #[source] cause: core::num::ParseIntError } => "bad number on line {line}",
    }
}

// 3. Extra arguments and expressions over fields (thiserror's `.field` form).
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

// 4. Transparent wrapping.
define_errors! {
    pub enum AppError {
        Config(#[from] inner: ConfigError) => transparent,
        Other(#[from] inner: Box<dyn core::error::Error + Send + Sync>) => transparent,
    }
}

// 5. Structs.
define_errors! {
    /// A syntax error in a NOML file.
    pub struct SyntaxError { pub line: u32, pub column: u32 } => "syntax error at {line}:{column}";

    pub struct Timeout(pub u64) => "operation timed out";           // unnamed tuple field

    pub struct Cancelled => "operation cancelled";
}

// 6. Closed sets: keep the enum exhaustive.
define_errors! {
    #[exhaustive]
    pub enum Parity {
        Odd => "value is odd",
        Even => "value is even",
    }
}

// 7. Optional source, braces in text, private type.
define_errors! {
    enum Internal {
        Lost { #[source] cause: Option<std::io::Error> } => "connection lost",
        Braces => "use {{ and }} for literal braces",
    }
}
```

#### 7.4.4 Examples: codes, severity, categories, kinds, metadata

```rust
use define_err::{define_errors, Kinds, Location};

pub const BILLING: Kinds = Kinds::project::<0>();

define_errors! {
    namespace = "DB";
    docs = "https://docs.hivedb.com/errors/{code}";
    retired = [9, 12];

    /// Errors from the query layer.
    pub enum DbError {
        /// The generic database error.
        #[code(0, help = "Check the logs for the underlying cause.")]
        Unspecified => "database error",

        #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        #[retry_after(secs = 2)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, name = CORRUPT, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt { page: u64, #[location] at: Location } => "page {page} failed its checksum",

        #[code(3, help = "The pool closed while the request waited.")]
        #[category(Unavailable)]
        PoolClosed { #[retry_after] wait: core::time::Duration } => "connection pool closed",

        #[code(4, help = "Use DB-0010 (DbError::Quota) instead.", deprecated = "2.1", replacement = 10)]
        #[category(ResourceExhausted)]
        OldQuota => "quota exceeded",

        #[code(10, help = "Raise the tenant quota or wait for the window to reset.")]
        #[category(ResourceExhausted)]
        #[kinds(BILLING)]
        #[action(Operator)]
        Quota { tenant: u64 } => "tenant {tenant} is over its quota",

        #[code(11, help = "See the wrapped I/O error.")]
        #[category(Unavailable)]
        Io(#[from] source: std::io::Error, #[location] at: Location) => "i/o error: {source}",
    }

    /// Errors from the planner, same namespace, same duplicate check.
    pub enum PlanError {
        #[code(20, help = "Simplify the query or raise planner.max_joins.")]
        #[category(InvalidArgument)]
        TooManyJoins { joins: u32 } => "query has {joins} joins",
    }
}

// Hive DB style, five-digit codes (DIS-00001):
define_errors! {
    namespace = "DIS", width = 5;

    pub enum DistroError {
        #[code(1, help = "Fix the named field in distro.noml.")]
        #[category(InvalidArgument)]
        #[action(FixConfig)]
        Config { field: String, reason: String } => "configuration error [{field}]: {reason}",

        #[code(3, help = "Check permissions and free space on the path.")]
        #[category(Unavailable)]
        Io { operation: &'static str, path: String, #[source] source: std::io::Error } =>
            "io error during {operation} on {path}: {source}",
    }
}
```

What a caller writes:

```rust
use define_err::prelude::*;

fn handle(err: &DbError) {
    if err.code() == Some(DbError::TIMEOUT) { /* ... */ }
    let status = err.category().http_status();        // 504
    let client_text = err.public_message();            // never the private Display
    if err.is_retryable() { /* back off for err.retry_after() */ }
    define_err::report(err);
}
```

#### 7.4.5 How the macro parses

- **Fast path.** A single non-recursive arm matches the common shape (doc comments, the metadata attributes in a fixed order, fields without attributes other than the four field markers) and emits every item through repetitions, so enums of any size compile at the default `recursion_limit` (NFR-6).
- **General path.** Anything else (attributes in other orders, `#[cfg]` on variants, pass-through attributes) goes through a tt-muncher that normalises one attribute per step into the fast path's shape, then calls the emitter. Recursion depth grows with the attributes of **one** variant, not with the number of variants, because variants are split first and munched independently.
- **Errors.** Every rejected input reaches an arm that calls `::core::compile_error!` with a message naming the problem and the fix (FR-28), placed on the offending tokens where `macro_rules!` allows.

#### 7.4.6 Limits of the declarative macro, stated

| Limit | Why | Where it is handled |
|---|---|---|
| no `{0}` positional placeholders | 7.4.2 | name the binding; or use L2 |
| generic types (`enum E<T>`) and lifetimes | `macro_rules!` cannot parse bounds reliably | simple type parameters without bounds are P3 (DEF-26); everything else uses L2 |
| messages from doc comments (displaydoc) | a doc literal cannot be trimmed or split by `macro_rules!` | L2 (P3 there) |
| constants for every code by default | `macro_rules!` cannot upper-case `Timeout` to `TIMEOUT` without a paste crate | `name = TIMEOUT` opt-in |
| generated constructors (`DbError::timeout(..)`) | same reason; also thiserror has none | build variants directly; `Location::here()` for location fields |

### 7.5 The generated-code contract (binding for L2)

This section is the contract L2 (derive) targets. Both front ends produce the same items with the same behaviour; the only differences are the input syntax and the three listed in 7.5.7. `docs/GENERATED-CODE.md` repeats it with full expansions and is locked at 0.2.

#### 7.5.1 Items generated for every type

For a type `T` with variants `V1..Vn` (a struct is one variant):

| # | Item | Content |
|---|---|---|
| G1 | the type | user attributes passed through; `#[derive(::core::fmt::Debug)]` unless `#[manual_debug]`; `#[non_exhaustive]` on enums unless `#[exhaustive]` or unless the user already wrote `#[non_exhaustive]` (never a second copy; structs: never added, because it would forbid construction outside the crate); field markers removed; `#[location]` fields typed `$crate::Location` |
| G2 | `impl ::core::fmt::Display for T` | one `match self` with one arm per variant binding every field by its name (tuple fields by their binding name, or `_` when unnamed); the arm calls `::core::write!(f, <format string> [, <arguments>])`; `transparent` arms call `::core::fmt::Display::fmt(inner, f)`; `#[allow(unused_variables)]` on the match |
| G3 | `impl ::core::error::Error for T` | `source()`: per variant, `$crate::__private::source_of!(field)` for the source field, `transparent` forwards to `inner.source()`, otherwise `None`; `description()`: per variant the identity probe string (7.11), under `#[allow(deprecated)]` |
| G4 | `impl ::core::convert::From<S> for T` | one per `#[from]` field; `#[track_caller]`; sets a `#[location]` field to `::core::panic::Location::caller()` |
| G5 | `impl $crate::ErrorInfo for T` | always generated; methods with no per-variant data are left to the trait defaults; others are one `match self` returning constants (7.5.2) |
| G6 | `impl T { pub const <NAME>: $crate::Code = ...; }` | one per `name = NAME`; `#[deprecated(since, note)]` when the code is deprecated (R10) |
| G7 | `impl T { pub const CATALOG: &'static [$crate::CodeInfo] = &[...]; }` | only when the block has a namespace; one entry per code in this type, plus (on the first type of the block) one entry per retired number |
| G8 | `const _: () = { ... };` | compile-time checks: duplicate numbers across the whole block including retired numbers, number range, namespace validity, severity range, kind bits, replacement codes exist in the block (7.13) |

Nothing else is generated: no constructors, no inherent methods, no statics, no `#[cfg(feature)]`, no `Result` alias.

#### 7.5.2 `ErrorInfo` methods and what each returns

| Method | Generated when | Value |
|---|---|---|
| `code(&self) -> Option<Code>` | any variant has `#[code]` | `Some(const code)` or `None` per variant |
| `code_info(&self) -> Option<&'static CodeInfo>` | same | `Some(&Self::CATALOG[i])` |
| `category(&self) -> Category` | any variant has `#[category]` | the variant's category, else `Category::Unknown` |
| `severity(&self) -> Severity` | any variant has `#[severity]` or `#[category]` | explicit severity, else the category default (7.8), else `Severity::ERROR` |
| `kinds(&self) -> Kinds` | any variant has `#[kinds]` | explicit kinds, else `Kinds::ERROR` |
| `public_message(&self) -> &'static str` | any variant has `#[public]` or `#[category]` | explicit text, else the category's default text |
| `help(&self) -> Option<&'static str>` | any variant has `#[code]` | the code's help text |
| `action(&self) -> Action` | any variant has `#[action]` or `#[category]` | explicit, else category default |
| `is_retryable(&self) -> bool` | any variant has `#[retryable]` or `#[category]` | explicit, else `category.is_transient()` |
| `retry_after(&self) -> Option<Duration>` | any variant has `#[retry_after]` | field value, or `Some(const)` for the static form, else `None` |
| `location(&self) -> Option<Location>` | any variant has a `#[location]` field | `Some(*at)` or `None` |
| `type_name(&self) -> &'static str` | always | `"T"` or `"T::Variant"` from `stringify!` |

`transparent` variants forward every `ErrorInfo` method to the inner value when the inner type implements `ErrorInfo`; because a declarative macro cannot know that, forwarding uses the same autoref dispatch as `source_of!` (7.5.4): inner types that implement `ErrorInfo` forward, others fall back to the defaults.

#### 7.5.3 Expansion example

Input (from 7.4.4, shortened):

```rust
define_errors! {
    namespace = "DB";
    pub enum DbError {
        #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",
        Io(#[from] source: std::io::Error, #[location] at: Location) => "i/o error: {source}",
    }
}
```

Output (paths shortened to `$crate` and `core`; the real output uses `::core::` and `$crate::` everywhere):

```rust
#[derive(core::fmt::Debug)]
#[non_exhaustive]
pub enum DbError {
    Timeout { after_ms: u64 },
    Io(std::io::Error, $crate::Location),
}

impl core::fmt::Display for DbError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[allow(unused_variables)]
        match self {
            Self::Timeout { after_ms } => core::write!(f, "query timed out after {after_ms} ms"),
            Self::Io(source, at) => core::write!(f, "i/o error: {source}"),
        }
    }
}

impl core::error::Error for DbError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Timeout { .. } => None,
            Self::Io(source, _) => $crate::__private::source_of!(source),
        }
    }
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            Self::Timeout { .. } => const { $crate::__private::probe!(code = (NS, 1), sev = .., cat = DeadlineExceeded, kinds = .., name = "DbError::Timeout") },
            Self::Io(..) => const { $crate::__private::probe!(code = none, .., name = "DbError::Io") },
        }
    }
}

impl core::convert::From<std::io::Error> for DbError {
    #[track_caller]
    fn from(source: std::io::Error) -> Self {
        Self::Io(source, core::panic::Location::caller())
    }
}

impl $crate::ErrorInfo for DbError {
    fn code(&self) -> Option<$crate::Code> {
        match self { Self::Timeout { .. } => Some(Self::TIMEOUT), _ => None }
    }
    fn category(&self) -> $crate::Category {
        match self { Self::Timeout { .. } => $crate::Category::DeadlineExceeded, _ => $crate::Category::Unknown }
    }
    // severity, public_message, action, is_retryable from the category; help, code_info,
    // location, type_name as in 7.5.2.
}

impl DbError {
    pub const TIMEOUT: $crate::Code = $crate::__private::code(NS, 1);
    pub const CATALOG: &'static [$crate::CodeInfo] = &[ /* entry for DB-0001 */ ];
}

const _: () = { $crate::__private::check_block(NS, &[1], &[]); };
```

(`NS` stands for the block's namespace constant, emitted once per block inside a `const` item named from the first type to avoid collisions: `const __DEFINE_ERR_NS_DbError: $crate::Namespace`.)

#### 7.5.4 The `__private` surface L2 may use

`#[doc(hidden)] pub mod __private` is semver-exempt for users but **versioned for L2**: L2 states which `define-err` minor versions it supports, and `__private` items change only in minor releases with a note in the changelog. Items:

| Item | Purpose |
|---|---|
| `source_of!(expr)` | returns `Option<&(dyn Error + 'static)>` for `E: Error`, `Option<E>`, `Box<dyn Error [+ Send [+ Sync]]>` (with `alloc`), and `&'static dyn Error`, by autoref method dispatch (the technique thiserror uses for `as_dyn_error`) |
| `info_of!(expr)` | the same dispatch for `ErrorInfo` forwarding in transparent variants |
| `code(ns, n) -> Code` | `const fn`, panics in const evaluation if `n` is out of range |
| `namespace(text, width) -> Namespace` | `const fn`, panics in const evaluation on invalid text or width |
| `severity::<N>() -> Severity` | `const` with an inline-const check |
| `check_block(ns, &[numbers], &[retired])` | `const fn`; panics in const evaluation on a duplicate, naming the namespace |
| `probe!(...)` | builds the probe string in a `const` block (7.11) |
| `ProbeBuf` | fixed-size buffer type for the probe encoder |
| `platform_kinds`, `named_levels` | modules glob-imported inside generated blocks so `#[kinds(SECURITY)]` and `#[severity(CRITICAL)]` resolve without user imports |

#### 7.5.5 Behaviour guarantees of generated code

1. Constructing a value runs no code (no hooks, no allocation, no atomic).
2. `Display` writes exactly the format string's output; no prefix, no code, no decoration.
3. `source()` returns exactly the source field or `None`; `transparent` forwards.
4. Every `ErrorInfo` method is a `match` returning constants or copying a field: no allocation, no lock, constant time.
5. Every path is absolute (FR-27); the generated code compiles under `#![no_implicit_prelude]`, `#![no_std]`, and `#![forbid(unsafe_code)]` in the user's crate.
6. No lint fires in generated code under the user's `#![deny(warnings)]`, `clippy::pedantic`, `missing_docs` (generated items carry docs: "Generated by define_errors!"), or `unreachable_pub`.
7. The type's `Send`, `Sync`, `Unpin`, size, and alignment are exactly those of its fields.

#### 7.5.6 Why generated code has no feature `cfg`

`#[cfg(feature = "std")]` inside a macro expansion is evaluated in the user's crate, against the user's features. So generated code must work identically whatever `define-err`'s features are. Consequences: the `Error` impl is on `core::error::Error` (hence MSRV 1.81, OD-2); boxed `dyn Error` sources work only when the user's code has `alloc` anyway (they name `Box`), and `source_of!` handles them through an impl that exists only with this crate's `alloc` feature, which any crate naming `Box<dyn Error>` already pulls in through `std` defaults.

#### 7.5.7 Differences L2 may not hide

| Topic | `define_errors!` | L2 derive |
|---|---|---|
| `#[non_exhaustive]` | added unless `#[exhaustive]` | a derive cannot add attributes to the item; L2 documents R5 and leaves the user's attributes as written (thiserror compatibility, EC-4) |
| `Debug` | derived unless `#[manual_debug]` | the user derives it, as with thiserror |
| `{0}` placeholders | not supported (7.4.2) | supported (the proc macro rewrites them) |

### 7.6 Codes: identity and layout

A `Code` is a namespace plus a number, stored in one `u64` so filtering, sorting, and comparison are integer operations and the value can travel between nodes as-is.

```
 bit  63                     32 31  29 28   24 23                       0
     [ namespace: 32 bits      ][width][reserved][ number: 24 bits        ]
       base-37 text, 1-6 chars   3 bits   5 bits   0 - 16,777,215
```

- **Namespace (32 bits).** Up to six characters from `A`-`Z` and `0`-`9`, first character a letter, packed as a base-37 number with the first character most significant and 0 as the padding after the last character (37^6 = 2,565,726,409 fits in 32 bits). The text is recovered from the integer alone, so a code displays without any catalog or registry, and numeric order of the field equals alphabetical order of the text. Lowercase input is accepted and stored uppercase.
- **Width (3 bits).** Display width of the number minus one (1 to 8 digits, default 4), so Hive DB's `DIS-00001` style keeps its five digits. Width is presentation, not identity: equality, ordering, and hashing use `identity()` (the `u64` with the width bits cleared), so `DIS-1`, `DIS-0001`, and `DIS-00001` are the same code.
- **Reserved (5 bits).** Zero in 1.x. `from_u64` rejects non-zero reserved bits, which keeps them available (for example a "vendor" bit, as in HRESULT's customer bit) without ambiguity.
- **Number (24 bits).** 0 to 16,777,215. Number 0 is the namespace's generic code (the error-suite plan, 4.3, after SQLSTATE `XX000`).
- **Text form.** `NS-` followed by the number zero-padded to the width; numbers wider than the width print in full (`DB-12345` with width 4).

This differs from the error-suite plan, 4.1 (16-bit registered namespace id, 32-bit number, 16 reserved bits). The reason: a registered 16-bit id needs a registry at run time to display a code, which `no_std` libraries cannot have and which distributed nodes would have to agree on; reversible text needs nothing. The cost: namespaces are limited to six characters and numbers to 24 bits. OD-4 records the change.

Namespaces collide when two crates pick the same text. Inside one block that cannot happen; across crates, L7 checks every registered catalog at boot (R23 boot side) and stops with a message naming both crates. `define-err` provides `catalog::check_unique` for that check (INF-06).

### 7.7 Severity

`Severity` is a `u8` from 1 to 24 on the OpenTelemetry SeverityNumber scale: TRACE 1-4, DEBUG 5-8, INFO 9-12, WARN 13-16, ERROR 17-20, FATAL 21-24; higher is more severe; 0 (unspecified) is not a `Severity` (use `Option<Severity>`).

| Constant | Value | OpenTelemetry short name | Meaning |
|---|---|---|---|
| `TRACE` | 1 | TRACE | very fine detail |
| `DEBUG` | 5 | DEBUG | debugging; default for cancelled operations (R20) |
| `INFO` | 9 | INFO | routine event |
| `NOTICE` | 10 | INFO2 | normal but significant (`notice`, PHP `E_NOTICE`, syslog notice) |
| `WARNING` | 13 | WARN | not an error, needs attention; default for `warn` and for handled client errors (R20 handled = 13) |
| `ERROR` | 17 | ERROR | an operation failed; default for `report` (R20 unhandled = 17) |
| `CRITICAL` | 20 | ERROR4 | failed and damaging (data loss, corruption); still recoverable |
| `FATAL` | 21 | FATAL | the process or service cannot continue (R20 panic = 21) |

`Display` prints the portfolio label when the value has one, otherwise the OpenTelemetry short name (`ERROR2`); `otel_name()` always prints the short name; `parse` accepts both. A project's own names ("billing-alert" = 22) are a `LevelNames` table (Tier 2): a `const` slice of `(name, Severity)` pairs with lookups both ways, which L7 can also build from configuration (the error-suite plan, 4.2). OD-8 records the label values.

### 7.8 Categories

`Category` is the gRPC canonical status set minus `OK`. Every mapping is a `const fn` over a fixed table.

| Category | gRPC | HTTP | Exit (sysexits) | Transient | Retry advice | Default severity | Default action | Default public message |
|---|---|---|---|---|---|---|---|---|
| `Cancelled` | 1 | 499 | 130 | no | `No` | DEBUG (5) | `None` | "The request was cancelled." |
| `Unknown` | 2 | 500 | 70 | no | `No` | ERROR (17) | `Operator` | "An unexpected error occurred." |
| `InvalidArgument` | 3 | 400 | 64 `EX_USAGE` | no | `No` | WARNING (13) | `FixRequest` | "The request is not valid." |
| `DeadlineExceeded` | 4 | 504 | 75 `EX_TEMPFAIL` | yes | `No` (AIP-194) | ERROR (17) | `Retry` | "The request took too long. Try again." |
| `NotFound` | 5 | 404 | 66 `EX_NOINPUT` | no | `No` | WARNING (13) | `FixRequest` | "The requested item was not found." |
| `AlreadyExists` | 6 | 409 | 73 `EX_CANTCREAT` | no | `No` | WARNING (13) | `FixRequest` | "The item already exists." |
| `PermissionDenied` | 7 | 403 | 77 `EX_NOPERM` | no | `No` | WARNING (13) | `FixRequest` | "You do not have permission to do this." |
| `ResourceExhausted` | 8 | 429 | 75 `EX_TEMPFAIL` | yes | `Later` | ERROR (17) | `Retry` | "Too many requests. Try again later." |
| `FailedPrecondition` | 9 | 400 | 78 `EX_CONFIG` | no | `AfterFix` | WARNING (13) | `FixRequest` | "The request cannot be done in the current state." |
| `Aborted` | 10 | 409 | 75 `EX_TEMPFAIL` | yes | `RestartTransaction` | WARNING (13) | `Retry` | "The request conflicted with another. Try again." |
| `OutOfRange` | 11 | 400 | 65 `EX_DATAERR` | no | `No` | WARNING (13) | `FixRequest` | "A value is out of range." |
| `Unimplemented` | 12 | 501 | 70 `EX_SOFTWARE` | no | `No` | ERROR (17) | `Operator` | "This operation is not supported." |
| `Internal` | 13 | 500 | 70 `EX_SOFTWARE` | no | `No` | ERROR (17) | `Operator` | "An internal error occurred." |
| `Unavailable` | 14 | 503 | 69 `EX_UNAVAILABLE` | yes | `Yes` | ERROR (17) | `Retry` | "The service is unavailable. Try again later." |
| `DataLoss` | 15 | 500 | 74 `EX_IOERR` | no | `No` | CRITICAL (20) | `PageOnCall` | "An internal error occurred." |
| `Unauthenticated` | 16 | 401 | 77 `EX_NOPERM` | no | `No` | WARNING (13) | `FixRequest` | "Authentication is required." |

- HTTP statuses follow `google.rpc.Code`'s "HTTP Mapping" lines. the error-suite plan, 4.3 suggested 409/422 for `FailedPrecondition`; Google maps it to 400, and this specification follows Google so gateways agree with other gRPC-transcoding systems (OD-9).
- **Transient** is R21's "retryable": the condition may clear on its own. **Retry advice** is AIP-194's narrower rule for **automatic** retries: only `Unavailable` (`Yes`), and only for operations whose repeat causes no unintended state change; `Aborted` restarts the whole transaction at a higher level; `ResourceExhausted` waits (`Later`, honour `retry_after`); `FailedPrecondition` only after the state is fixed. `Category::may_retry_automatically(idempotent: bool) -> bool` returns `idempotent && self == Unavailable`.
- Exit codes follow `sysexits(3)`; `Cancelled` uses 130 (the shell's 128 + SIGINT). OD-10 records the table.
- `DataLoss` has the same public text as `Internal` on purpose: clients are never told that data was lost through a generic error path.
- With `std`, `Category::from_io_kind(io::ErrorKind)` maps `NotFound`, `PermissionDenied`, `AlreadyExists`, `InvalidInput`, `InvalidData`, `TimedOut`, `WouldBlock`, `Interrupted`, `OutOfMemory`, `Unsupported`, and the network kinds; anything else is `Unknown`. `ErrorInfo for std::io::Error` uses it.

### 7.9 Kinds

`Kinds` is a `u64` set (PHP `E_*` style). Each kind is one bit; masks combine with `|`, test with `contains` and `intersects`. Severity is an order; kinds are a set; both exist because filters need both ("warnings and above" versus "security or billing, any level").

| Bit | Constant | Meaning |
|---|---|---|
| 0 | `ERROR` | an error (default for reported errors) |
| 1 | `WARNING` | a warning (`warn`) |
| 2 | `NOTICE` | an informational notice (`notice`) |
| 3 | `DEPRECATED` | use of something deprecated (`deprecated`, R10) |
| 4 | `SECURITY` | security relevant; delivered to security listeners on the bridge |
| 5 | `FUTURE` | behaviour that will change in a later version |
| 6 | `RUNTIME` | a run-time condition (PHP and Python runtime warnings) |
| 7 | `RESOURCE` | resource pressure (memory, handles, quota) |
| 8 | `CONFIG` | configuration problem |
| 9 | `USER` | caused by the caller's input |
| 10 | `PANIC` | a caught panic (set by L3 and L7, not by users) |
| 11 | `AUDIT` | an action that must reach the audit trail |
| 12 | `TRIGGER` | raised on purpose by `trigger` |
| 13-31 | reserved | platform kinds added in minor releases; never reassigned |
| 32-63 | project | `Kinds::project::<N>()` with `N` from 0 to 31 |

`Kinds::project::<N>()` checks `N < 32` in an inline `const` block, so `Kinds::project::<32>()` is a compile error and no run-time panic path exists. 64 bits (32 per side) follow the error collection plan, 11.3; widening to `u128` is a 2.0 change (OD-20).

### 7.10 The metadata model

`ErrorInfo` is the one trait through which every part of the collection reads an error's metadata. It is object safe, has `core::error::Error` as supertrait, and every method has a default, so `impl ErrorInfo for MyError {}` is valid for hand-written types.

| Field (the error-suite plan, 4.4) | Here | Notes |
|---|---|---|
| code | `code()`, `code_info()` | compile time |
| severity | `severity()` | default; L7 may override by configuration, outside this crate |
| public message | `public_message()` | never derived from `Display` |
| private message | `Display` | logs only |
| category | `category()` | |
| retryable | `is_retryable()`, `retry_after()` | R21 |
| action | `action()` | `Action`: `None`, `Retry`, `FixRequest`, `FixConfig`, `Operator`, `PageOnCall` (`#[non_exhaustive]`; after IBM z/OS message suffixes) |
| stage, correlation | not here | the reporting context (L7) |
| location | `location()` | `#[location]` fields, filled by `#[track_caller]` |
| source chain | `Error::source()` | |
| help | `help()`, `code_info().url()` | |
| kinds | `kinds()` | 11.3 |
| type name | `type_name()` | OpenTelemetry `exception.type` (R20, L4) |

`Location` is `&'static core::panic::Location<'static>`; `Location::here()` is not possible on a type alias, so the free function `define_err::here()` (`#[track_caller]`) returns the caller's location for building a variant by hand.

### 7.11 The identity probe

**Problem.** L3 holds errors as `dyn Error`; L4 renders chains of `dyn Error`; L7 receives `&dyn Error` in events. None of them can ask a `dyn Error` for its `ErrorInfo`: stable Rust has no way to cast `dyn Error` to another trait object, and the standard mechanism for this (`Error::provide`) is nightly-only and excluded (the error collection plan, 10.4).

**Mechanism.** `core::error::Error::description(&self) -> &str` is stable, deprecated for callers but implementable, and returns a string chosen by the type. Generated code implements it per variant to return a fixed **probe string** built at compile time. Any holder of `&dyn Error` calls `define_err::identify(err)`, which calls `description()` once and decodes the probe if it starts with the magic prefix. Types not generated by `define-err` or L2 return the standard library's default text, which never starts with the prefix, so `identify` returns `None` in one comparison.

**Probe format (frozen at 1.0).**

```
offset  length  content
0       4       magic "\u{0}DE1"  (NUL, 'D', 'E', format version '1')
4       16      code: identity u64 in lowercase hex, or 16 'x' for none
20      2       severity in hex (01-18)
22      2       category gRPC number in hex (01-10)
24      16      kinds u64 in hex
40      2       flags in hex (bit 0: has public message; bit 1: transparent)
42      1       '|'
43      n       type name "Type::Variant" (ASCII; at most 128 bytes)
```

The string is built in an inline `const` block by a `const fn` encoder writing into a fixed `[u8; 172]` buffer, then sliced and checked with `core::str::from_utf8` in const evaluation; nothing happens at run time except returning a `&'static str`. Decoding is fixed-offset hex parsing, no allocation, about 10 ns [est.].

**What it carries.** Only per-variant constants (code, severity, category, kinds, flags, name). Per-value data (location, `retry_after` fields) is reached through `ErrorInfo` on the concrete type, which L3 captures when it still knows the type.

**Transparent wrappers (D-012).** A `transparent` variant adds no entry to the `source()` chain, so its probe is the inner value's probe when the inner value has one; otherwise it is the wrapper's own probe with flag bit 1 set. A chain walker therefore always sees the identity of the error a reader actually sees.

**Trade-offs, stated.** (a) Old code that calls the deprecated `description()` on a `define-err` type gets the probe text instead of a sentence; this is documented, and no portfolio crate calls it. (b) A foreign type could return a forged probe; codes are identification, not authentication (T11). (c) If Rust ever stabilises `provide`, a 1.x minor release adds it alongside; the probe stays for compatibility. OD-5 records the decision.

### 7.12 Catalog and deprecation

`CodeInfo` (all fields `pub`, struct `#[non_exhaustive]`, built only by generated code or `CodeInfo::new` in const context):

| Field | Type | Source |
|---|---|---|
| `code` | `Code` | `#[code(n)]` |
| `title` | `&'static str` | `title = "..."`, else the variant name (`stringify!`) |
| `help` | `&'static str` | `help = "..."` (required) |
| `docs` | `Option<&'static str>` | the block's `docs` template |
| `severity`, `category`, `kinds`, `action` | | the variant's resolved values |
| `public` | `&'static str` | resolved public message |
| `state` | `CodeState` | `Active`, `Deprecated { since, replacement }`, `Retired` |
| `type_name` | `&'static str` | `"Type::Variant"`, or `""` for retired entries |

- `info.url()` returns a `UrlDisplay` that writes the template with `{code}`, `{namespace}`, and `{number}` replaced, without allocation. A template without `{code}`, `{namespace}`, or `{number}` is a compile error.
- **Retired numbers** (`retired = [9, 12];`) produce catalog entries with `state: Retired`, help "This code is retired and is never reused.", and join the duplicate check, so a retired number can never return (rustc `E0001`-style never-reuse rule).
- **Deprecated codes** stay fully usable. At compile time, `name = OLD` constants carry `#[deprecated(since = "2.1", note = "replaced by DB-0010")]`. At run time, `code_info().state` says so, and L7 may warn when a deprecated code is reported (policy, not here).
- **Runtime deprecation of anything else** (an API, a configuration key) is the facade verb `deprecated` (7.14.5), which warns once per call site with `Kinds::DEPRECATED | Kinds::WARNING`.
- `catalog::find(catalogs, code)` and `catalog::check_unique(catalogs)` (Tier 2) take `&[&[CodeInfo]]`, so L7 and tests can search and validate several crates' catalogs without a registry.

### 7.13 Compile-time checks without run-time panics

The crate follows one rule: **a `panic!` may only be reachable in constant evaluation.** Three patterns implement it.

1. **Const generic plus inline const** (stable since 1.79): `Kinds::project::<N>()` and `Severity::at::<N>()` contain `const { assert!(N < 32) }`; the check runs at monomorphisation, so a bad literal is a compile error and the run-time function has no branch.
2. **Const items in generated code**: `const _: () = { $crate::__private::check_block(...) };` and the code and namespace constants are `const` items, so the `const fn`s they call (which do contain `panic!`) only ever run in the compiler. Those `const fn`s are `#[doc(hidden)]` in `__private` and documented "for const evaluation only"; their run-time-callable public twins return `Result` or `Option` (`Code::try_new`, `Namespace::try_new`, `Severity::new`).
3. **Duplicate detection with a span.** `check_block` is an O(n²) `const fn` loop that panics with `concat!("define_errors!: duplicate code number in namespace ", NS)` (const panics cannot format numbers; Rust 1.57 release notes). To point at the duplicate, generated code also emits `#[deny(unreachable_patterns)] match 0u32 { 1 => {}, 2 => {}, 1 => {}, _ => {} }` from the user's literal tokens, so the compiler marks the second `1` itself. Both are tested in `tests/ui/`.

Checked at compile time: code number range, namespace text and width, help present, docs template contains a placeholder, severity 1 to 24, project kind bit below 32, duplicates across the block and the retired list, `replacement` refers to a code in the block and not to itself, one source and one location per variant, `#[from]` variants hold nothing else, `transparent` has one field.

### 7.14 The reporting facade

#### 7.14.1 Model

```
 any crate            define_err::report(&err) / warn(..) / notice(..) / trigger(..)
     |
     v
 [control word]  one acquire load: nothing installed -> return     (cost: one load)
     |  installed
     v
 [warn mode, once-per-site, min severity, kind mask]  -> filtered out -> return
     |
     v
 Event { verb, kinds, severity, code, category, error, message, location }  (borrowed, on the stack)
     |
     v
 &'static dyn Reporter :: receive(&event)        (L7, a test capture, or the application's own)
```

The `log` crate's `set_logger` pattern (a `static mut` logger, an `AtomicUsize` state with `UNINITIALIZED`, `INITIALIZING`, `INITIALIZED`, `compare_exchange` to claim the slot, a `Release` store to publish, an `Acquire` load before reading) is the model, adapted in four ways: the state shares one word with the warn mode and minimum severity; the first installer's location is recorded; once-per-site delivery exists; and reentrancy and panics are contained with `std`.

#### 7.14.2 The slot (the only unsafe code)

```rust
// src/facade/slot.rs (sketch)
pub(crate) struct Slot(UnsafeCell<Option<&'static dyn Reporter>>);
// SAFETY: written exactly once, by the thread that moved CONTROL from EMPTY to INSTALLING,
// before the Release store of READY; read only after an Acquire load observes READY.
unsafe impl Sync for Slot {}
```

`install` claims the slot with `compare_exchange(EMPTY, INSTALLING)` on the state bits, writes the reference and the installer's `Location`, then stores `READY` with `Release`. `get` returns `None` unless an `Acquire` load sees `READY`, then reads the slot. A thread that loses the race spins only while another thread is between claim and publish (a few instructions; the `log` crate does the same) and then returns `Err(SetReporterError { installed_at })`. Three `unsafe` sites (the `Sync` impl, the write, the read), all in this file. Loom models: two installers racing; reader racing installer; reader on a weak-memory model sees either `None` or the complete reference, never a torn value.

#### 7.14.3 The control word

One `AtomicU32`:

| Bits | Content |
|---|---|
| 0-1 | state: 0 empty, 1 installing, 2 ready |
| 2-3 | warn mode: 0 `Once` (default), 1 `Always`, 2 `Ignore`, 3 `Error` |
| 4-8 | minimum severity, 0 to 24 (0 = everything) |
| 9-31 | reserved |

Every verb starts with one `Acquire` load of this word. If the state is not ready, the verb returns (FR-20), except `soft`, which also needs the warn mode from the same load. The kind mask lives in two `AtomicU32`s (platform half, project half), read with `Relaxed` only after the state check, so 32-bit targets without 64-bit atomics behave the same. Mode, severity, and mask setters use `fetch_update`; they never touch the state bits.

#### 7.14.4 Verbs

| Verb | Function | Macro | Default kinds | Default severity | Once per site |
|---|---|---|---|---|---|
| report | `report(err: &(dyn Error + 'static))` | none | from the probe, else `ERROR` | from the probe, else `ERROR` | no |
| notice | `notice(message: &str)` | `notice!("fmt", args)` | `NOTICE` | `NOTICE` | no |
| warn | `warn(message: &str)` | `warn!("fmt", args)` | `WARNING` | `WARNING` | yes |
| trigger | `trigger(kinds: Kinds, message: &str)` | `trigger!(KINDS, "fmt", args)` | `TRIGGER` plus the given kinds | `WARNING` | no |
| deprecated | `deprecated(item: &str)` | `deprecated!("item", since = "x", replacement = "y")` | `DEPRECATED`, `WARNING` | `WARNING` | yes |
| soft | `soft<E: Error + 'static>(err: E) -> Result<(), E>` | none | `WARNING` plus the error's kinds | `WARNING` | no |

- All functions are `#[track_caller]`; the macros expand at the call site, so both record the user's file, line, and column (FR-19). A user helper marked `#[track_caller]` that calls a verb reports its own caller, which is the fix for the PHP `trigger_error` limitation James hit (the error collection plan, 11.2).
- `notice(&str)` and the other `&str` forms never interpret braces: the text is passed as `format_args!("{}", message)`. Only the macros take format strings, and those are compile-time literals (T3).
- Severity and kinds with explicit values: `report_with(err, severity, kinds)`, `notice_with(kinds, message)`, `warn_with(kinds, message)` (Tier 2).
- `trigger` only delivers an event whose verb is `Trigger`; the actions registered for a kind (escalate, alert, lock down) belong to L7 (the error collection plan, 11.2 says the action side "needs its own design").
- `soft(err)` reads the warn mode: `Error` returns `Err(err)` (even with no reporter installed); otherwise it delivers a warning carrying the error and returns `Ok(())`.

#### 7.14.5 Event and Reporter

```rust
pub trait Reporter: Send + Sync {
    fn receive(&self, event: &Event<'_>);
    fn flush(&self) {}
}

#[non_exhaustive]
pub enum Verb { Report, Notice, Warn, Trigger, Deprecated, Soft }

#[non_exhaustive]
pub struct Event<'a> { /* private fields; getters below */ }
impl<'a> Event<'a> {
    pub fn verb(&self) -> Verb;
    pub fn kinds(&self) -> Kinds;
    pub fn severity(&self) -> Severity;
    pub fn code(&self) -> Option<Code>;
    pub fn category(&self) -> Category;
    pub fn error(&self) -> Option<&'a (dyn core::error::Error + 'static)>;
    pub fn message(&self) -> Option<core::fmt::Arguments<'a>>;
    pub fn location(&self) -> Location;
    pub fn type_name(&self) -> Option<&'static str>;
}
```

The event borrows everything and lives on the caller's stack for the duration of `receive`. A reporter that needs to keep it builds an `ErrorRecord` (with `alloc`) or copies what it needs. The contract in `docs/FACADE.md`: `receive` must return quickly (microseconds), must not block on I/O (queue instead; that is what L7 and dispatch-io do), may be called from any thread concurrently, and must not assume it is called once per error (a report and a later warning about the same error are two events).

#### 7.14.6 Once per call site

- **Macros** (`warn!`, `deprecated!`) expand to a `static SITE: $crate::__private::Site = Site::new();` (one `AtomicBool`) at the call site. After the state check and the mode check, `SITE.swap(true, Relaxed)` decides: the first caller delivers, every later caller returns. Exact, unlimited, one static per call site (a static inside a generic function is one static for all instantiations).
- **Functions** (`warn`, `deprecated`) cannot own a static per call site, so they use a fixed lock-free table of 512 `AtomicU64` slots (4 KiB) keyed by a 64-bit hash of the caller's `(file, line, column)`; insert by linear probing with `compare_exchange`; a key found means "already warned". Content hashing, not pointer identity, because the compiler may emit duplicate `Location` statics for one source location across codegen units. False suppression needs a 64-bit hash collision among at most 512 sites (probability below 10^-14). When the table is full, the function delivers every time and counts the overflow (`facade::stats().site_table_overflow`), so a warning is never lost to the table. Targets without 64-bit atomics use 32-bit hashes (collision probability with 512 sites about 3 x 10^-5, documented).
- **Before installation.** The site flag is consulted only after the state check, so a warning issued before a reporter exists does not consume the flag and is delivered the first time it happens after installation (FR-21).
- **Mode `Always`** skips the flag; **`Ignore`** drops warnings; **`Error`** makes `soft` fail. Switching from `Always` back to `Once` keeps flags already set.
- On targets without compare-and-swap (`thumbv6m`), the facade is compiled out: `set_reporter` does not exist, and the verbs are empty `#[inline]` functions. The README and `docs/PLATFORM-NOTES.md` say so; a racy `unsafe` installer (as in `log`) is P3 (FAC-19).

#### 7.14.7 Reentrancy and panics (`std`)

- A thread-local flag marks "inside `receive`". A verb called on the same thread while the flag is set returns without delivering and increments `stats().reentrant_dropped`. Cost: one thread-local access, only after a reporter is installed.
- `receive` runs inside `std::panic::catch_unwind(AssertUnwindSafe(...))`; a panic is caught, its payload dropped inside a second `catch_unwind` (dropping a payload can panic), and `stats().reporter_panics` incremented. Under `panic = "abort"` nothing can be caught, which `docs/FACADE.md` states.
- Without `std` neither guard exists; the reporter contract forbids calling verbs from `receive` and forbids panicking, and `docs/FACADE.md` states it.

#### 7.14.8 Ordering and memory model

- Install: `compare_exchange(EMPTY -> INSTALLING, Acquire, Relaxed)`, slot write, `store(READY, Release)` via `fetch_update` preserving the other bits.
- Verbs: `load(Acquire)`; slot read only when the loaded state is `READY`.
- Site flags, the site table, mode, severity, mask, statistics: `Relaxed` (each is an independent decision; no data is published through them).
- Proven with loom models in `tests/loom.rs` (12.5).

### 7.15 `ErrorRecord` (feature `alloc`)

An owned snapshot for anything that outlives the borrow: queued listeners, the crash ring, JSON output, relay-io messages (R4).

```rust
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct ErrorRecord {
    pub code: Option<Code>,
    pub severity: Severity,
    pub category: Category,
    pub kinds: Kinds,
    pub public: Cow<'static, str>,
    pub chain: Vec<ChainEntry>,            // outermost first: private message, code, type name
    pub fields: Vec<(Cow<'static, str>, String)>,
    pub trail: Vec<TrailEntry>,            // locations: creation, conversions, report site
    pub backtrace: Option<String>,         // text, filled by L3 or L7
    pub event_id: Option<u128>,            // filled by L7 (R17)
    pub time_unix_nanos: Option<u64>,      // filled by L7 (no clock in no_std)
    pub handled: bool,
    pub truncated: bool,
}
```

- `ErrorRecord::from_error(&dyn Error)` walks `source()` up to `limits.max_chain` entries, writes each `Display` through a bounded `fmt::Write` adapter that stops at `limits.max_message_bytes` (cut on a UTF-8 character boundary), reads each entry's identity with `identify`, and takes code, severity, category, kinds, and public message from the outermost entry that has a code; if none has, from the outermost entry. A `Display` that returns `fmt::Error` stores `"<message unavailable>"`.
- `from_event(&Event)` adds the event's verb location to the trail and its message.
- Limits (`RecordLimits`, Tier 2): `max_chain` 32, `max_message_bytes` 1,024, `max_fields` 32, `max_field_bytes` 256, `max_trail` 64, `max_total_bytes` 16,384. Every limit is checked before the allocation it guards; exceeding one sets `truncated`.
- The public text never comes from any `Display` in the chain (T15).
- `Display` for a record: `"[DB-0001] query timed out after 5000 ms: caused by ..."`, one line; L4 owns every other format.
- `Send + Sync + Clone`; no references; no secrets unless a user's `Display` prints one, which `Secret<T>` prevents.

### 7.16 `Secret<T>`

```rust
#[derive(Clone, Copy, Default, Hash)]   // Copy and Hash only when T has them
pub struct Secret<T>(T);
```

`Debug` and `Display` write `[redacted]` ignoring width, fill, precision, and alternate flags (the padding flags are honoured around the word so tables stay aligned). `Secret::new(v)`, `expose(&self) -> &T`, `into_inner(self) -> T`, `From<T>`. No `Deref`, no `PartialEq`, no `AsRef`, no `Borrow`: each of those would let the value slip into a format string or a timing-unsafe comparison. Usable as an error field: `BadToken { token: Secret<String> } => "token {token} rejected"` displays `token [redacted] rejected`. Wiping memory on drop is crypt-io's job; L1 cannot guarantee a wipe without `unsafe` volatile writes, and a partial promise is worse than none (OD-16).

### 7.17 `no_std` and targets

| Build | Contents | CI |
|---|---|---|
| core only | everything except `ErrorRecord`, boxed sources, `set_boxed_reporter`, `std` mappings, guards | `--no-default-features` on hosts; cross-check `thumbv7em-none-eabihf`, `riscv32imac-unknown-none-elf` |
| core + alloc | plus `ErrorRecord`, boxed sources, `set_boxed_reporter` | `--no-default-features --features alloc` |
| std (default) | plus `io` mappings, reentrancy guard, panic isolation | default |
| no compare-and-swap (`thumbv6m-none-eabi`) | facade compiled out (7.14.6) | cross-check |
| `wasm32-unknown-unknown` | everything; atomics exist on single-threaded wasm | cross-check |

`cfg` uses only `feature = "..."` and `target_has_atomic = "..."`; no `target_os`.

### 7.18 Thread safety

Generated types are plain data. `Code`, `Namespace`, `Severity`, `Category`, `Kinds`, `Action`, `CodeInfo`, `Identity`, `ErrorRecord` are `Send + Sync + Copy` (record: `Clone`). `Secret<T>` is `Send`/`Sync` when `T` is. `Event<'a>` holds `&(dyn Error + 'static)` without a `Sync` bound (so any error can be reported) and `fmt::Arguments`, so it is neither `Send` nor `Sync`: it lives on the reporting thread for the duration of `receive`, and a reporter that hands work to another thread copies what it needs or builds an `ErrorRecord` (which is `Send + Sync`). `Reporter` itself is `Send + Sync`, so one reporter serves every thread. Compile-time assertions in `tests/traits.rs`.

### 7.19 What can never be observed (stated in the docs)

Errors are values in Rust; there is no throw event. The facade sees an error only when code calls a verb. It can never see an error discarded with `let _ =`, `.ok()`, or `unwrap_or`, an error handled inside third-party code, or a typed error built and never reported. The README states this next to `report` so no one assumes "every error is reported" (the error collection plan, 10.5).

---

## 8. API specification

### 8.1 Conventions

> From the `0.2.0` design lock, [`docs/API.md`](../docs/API.md) is the authoritative list of signatures. Where it differs from the sketches in this section (D-008 to D-022 in section 20), `docs/API.md` wins and this section's grades, tiers, versions, and costs still apply.


- **Item ids**: `ROOT-nn`, `DEF-nn` (the macro), `CODE-nn`, `SEV-nn`, `CAT-nn`, `KND-nn`, `INF-nn` (metadata and catalog), `PRB-nn` (probe), `SEC-nn`, `FAC-nn`, `REC-nn`, `TST-nn`. The roadmap (section 14) refers to these ids.
- **Columns**: Grade (P0 to P3, standard 1.1), Tier (1, 2, 3; standard 2.10), Ver (version that ships it), Cost ([est.] until measured in 0.6).
- **Errors**: functions that can fail return `Result` with this crate's own small error types (`CodeParseError`, `SetReporterError`), each `#[non_exhaustive]`, each `Display` saying what to do. No function in this crate panics at run time.
- **Import**: `use define_err::prelude::*;` brings `define_errors!`, `ErrorInfo`, `Code`, `Severity`, `Category`, `Kinds`, `Action`, `Secret`, and `Location`. The verb macros are not in the prelude (D-013). The verb functions are called by path (`define_err::report(&e)`) in examples, so a reader always sees where they come from.

Tier summary:

| Tier | What | Examples |
|---|---|---|
| 1 | the macro and the verbs: no configuration, no type parameters | `define_errors!`, `report`, `notice`, `warn`, `trigger`, `deprecated`, `set_reporter`, `err.code()`, `err.category().http_status()`, `Secret::new` |
| 2 | configuration and tables for the next 15% | `set_warn_mode`, `set_min_severity`, `set_kinds_mask`, `report_with`, `RecordLimits`, `LevelNames`, `catalog::find`, `set_boxed_reporter` |
| 3 | the full surface for other libraries | `identify`, `Code::from_u64`, `Kinds::from_bits`, `Event::new`, `ErrorRecord` fields, `__private` (L2 only) |

### 8.2 Tier 1 on one screen

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
    let text = std::fs::read_to_string(path)?;          // From<io::Error>, location-free
    if text.is_empty() {
        define_err::warn("configuration file is empty; using defaults");
    }
    Ok(text)
}

fn main() {
    if let Err(err) = load("app.noml") {
        define_err::report(&err);                         // no-op until a reporter is installed
        eprintln!("{err}");
        std::process::exit(err.category().exit_code().into());
    }
}
```

### 8.3 `define_errors!` (DEF)

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| DEF-01 | enums with unit, struct, and tuple variants; `=> "message"` with named-field capture and format specs | P0 | 1 | 0.3 | zero at run time; `Display` = one `write!` |
| DEF-02 | `Display` (G2) and `core::error::Error` (G3) generation | P0 | 1 | 0.3 | as above |
| DEF-03 | `#[source]`, field named `source`, `Option<E>` and boxed sources (`source_of!`) | P0 | 1 | 0.3 | one match arm |
| DEF-04 | `#[from]` with `#[track_caller]` `From` (G4) | P0 | 1 | 0.3 | same as a hand-written `From` |
| DEF-05 | `=> transparent` | P0 | 1 | 0.3 | one call |
| DEF-06 | structs: braced, tuple, unit (`pub struct X { .. } => "...";`) | P0 | 1 | 0.3 | |
| DEF-07 | tuple-field binding names (`Io(source: io::Error)`) and unnamed tuple fields | P0 | 1 | 0.3 | |
| DEF-08 | parenthesised message with extra arguments `=> ("...", expr, name = expr)` | P1 | 1 | 0.3 | |
| DEF-09 | `#[non_exhaustive]` by default; `#[exhaustive]` opt-out (R5) | P0 | 1 | 0.3 | |
| DEF-10 | automatic `Debug`; `#[manual_debug]` opt-out | P1 | 1 | 0.3 | |
| DEF-11 | several types per block; pass-through of doc comments and other attributes; optional trailing comma | P0 | 1 | 0.3 | |
| DEF-12 | `ErrorInfo` impl with defaults for plain types (G5) | P0 | 1 | 0.3 | zero |
| DEF-13 | identity probe `description()` (G3, 7.11) | P1 | 3 | 0.4 | zero at run time |
| DEF-14 | block header `namespace = "NS" [, width = n];` | P0 | 1 | 0.4 | |
| DEF-15 | `#[code(n, help = "...")]`, optional `name = IDENT`, `title` (G6) | P0 | 1 | 0.4 | constant |
| DEF-16 | `#[category(..)]`, `#[severity(..)]`, `#[kinds(..)]`, `#[public(..)]`, `#[action(..)]`, `#[retryable]` | P0 | 1 | 0.4 | constant |
| DEF-17 | `#[retry_after(secs = n)]` and `#[retry_after]` fields (R21) | P1 | 1 | 0.4 | constant or field copy |
| DEF-18 | `#[location]` fields filled by `From` | P1 | 1 | 0.4 | one pointer store |
| DEF-19 | compile-time duplicate check, range checks, help-required check (R23, 7.13) | P0 | 1 | 0.4 | compile time only |
| DEF-20 | `retired = [..];` header (never-reuse) | P1 | 1 | 0.4 | |
| DEF-21 | `deprecated = "since", replacement = n` in `#[code]`; `#[deprecated]` on named constants (R10) | P1 | 1 | 0.4 | |
| DEF-22 | `docs = "...{code}";` header | P1 | 1 | 0.4 | |
| DEF-23 | `CATALOG` constant per type (G7) | P0 | 2 | 0.4 | static data |
| DEF-24 | `#[cfg(..)]` on variants and fields (arms follow the variant's cfg) | P2 | 1 | 0.5 | |
| DEF-25 | `transparent` forwarding of `ErrorInfo` (`info_of!`) | P1 | 1 | 0.4 | |
| DEF-26 | simple type parameters without bounds (`enum E<T>`); bounds through a `where` clause on the type | P3 | 1 | after 1.0 | |
| DEF-27 | 500-variant enums at the default `recursion_limit` (fast path) | P0 | 1 | 0.3 | compile time |

Errors (compile time), each with a `tests/ui` case: unknown attribute in the metadata position (lists the valid ones); `#[code]` without `help`; `#[code]` without a `namespace` header; number out of range or not a literal; duplicate number; retired number used; namespace invalid (empty, longer than 6, bad character, starts with a digit); width outside 1 to 8; severity outside 1 to 24 or unknown name; unknown category; unknown kind name (resolution error naming the identifier); project kind bit 32 or more; two source fields; two location fields; `#[from]` with extra fields; two `#[from]` of the same type in one enum (the conflicting `From` impls error, with a note added by the macro); `transparent` with zero or two fields; missing `=> message`; message referencing a field that does not exist (rustc's format error, pointing at the literal); `{0}` in a message (rustc's "no arguments were given" plus a note in the macro docs); docs template without a placeholder; `replacement` pointing at itself or at a code outside the block.

### 8.4 Codes (CODE)

```rust
#[derive(Clone, Copy)]
pub struct Namespace { /* packed text + width */ }
impl Namespace {
    pub const fn try_new(text: &str) -> Result<Namespace, CodeParseError>;
    pub const fn try_with_width(text: &str, width: u8) -> Result<Namespace, CodeParseError>;
    pub fn as_str(&self) -> NamespaceStr;            // stack copy, Deref<Target = str>, up to 6 bytes
    pub const fn width(self) -> u8;
    pub const fn code(self, number: u32) -> Option<Code>;
}
#[macro_export] macro_rules! namespace { ("DB") => ...; ("DB", width = 5) => ... }   // const, checked

#[derive(Clone, Copy)]
pub struct Code { /* u64 */ }
impl Code {
    pub const MAX_NUMBER: u32 = 16_777_215;
    pub const fn new(namespace: Namespace, number: u32) -> Option<Code>;
    pub const fn namespace(self) -> Namespace;
    pub const fn number(self) -> u32;
    pub const fn is_generic(self) -> bool;          // number == 0
    pub const fn as_u64(self) -> u64;               // full value including width
    pub const fn identity(self) -> u64;             // width bits cleared; what Eq, Ord, Hash use
    pub const fn from_u64(raw: u64) -> Option<Code>;
    pub fn parse(text: &str) -> Result<Code, CodeParseError>;
}
impl PartialEq, Eq, PartialOrd, Ord, Hash for Code      // by identity()
impl fmt::Display for Code                             // "DB-0042"
impl fmt::Debug for Code                               // Code(DB-0042)
impl core::str::FromStr for Code
#[macro_export] macro_rules! code { ("DB", 42) => ...; (NAMESPACE_CONST, 42) => ... }  // const, checked
```

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| CODE-01 | `Namespace`, `try_new`, `try_with_width`, `as_str`, `width` | P0 | 2 | 0.4 | `try_new` under 10 ns |
| CODE-02 | `namespace!` const macro (compile error on bad text) | P0 | 1 | 0.4 | compile time |
| CODE-03 | `Code`, `new`, `namespace`, `number`, `is_generic` | P0 | 1 | 0.4 | under 1 ns (shifts) |
| CODE-04 | `code!` const macro | P1 | 1 | 0.4 | compile time |
| CODE-05 | `Display`, `Debug` without allocation | P0 | 1 | 0.4 | under 20 ns |
| CODE-06 | `parse`, `FromStr` | P0 | 2 | 0.4 | under 15 ns |
| CODE-07 | `as_u64`, `identity`, `from_u64` | P0 | 3 | 0.4 | under 2 ns |
| CODE-08 | `Eq`, `Ord`, `Hash` by identity | P0 | 1 | 0.4 | integer compare |
| CODE-09 | `CodeParseError` (`#[non_exhaustive]` enum: `Empty`, `MissingSeparator`, `BadNamespace`, `BadNumber`, `NumberTooLarge`, `ReservedBits`) | P0 | 2 | 0.4 | |

`Code::parse` accepts exactly `^[A-Za-z][A-Za-z0-9]{0,5}-[0-9]{1,8}$`; the parsed width is the digit count. It rejects leading or trailing whitespace, `+`, `-` signs in the number, `0x`, Unicode hyphens and digits, a second separator, and any byte outside ASCII. Error `Display` example: "code text `DB-0x2A` is not valid: the number must be decimal digits, for example DB-0042".

### 8.5 Severity (SEV)

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Severity(u8);
impl Severity {
    pub const TRACE: Severity; pub const DEBUG: Severity; pub const INFO: Severity;
    pub const NOTICE: Severity; pub const WARNING: Severity; pub const ERROR: Severity;
    pub const CRITICAL: Severity; pub const FATAL: Severity;
    pub const fn at<const N: u8>() -> Severity;     // compile error unless 1..=24
    pub const fn new(value: u8) -> Option<Severity>;
    pub const fn saturating(value: u8) -> Severity; // 0 -> 1, >24 -> 24
    pub const fn get(self) -> u8;
    pub const fn band(self) -> Band;                // Trace, Debug, Info, Warn, Error, Fatal
    pub const fn otel_name(self) -> &'static str;   // "ERROR4"
    pub const fn label(self) -> Option<&'static str>; // "CRITICAL" for 20
    pub fn parse(text: &str) -> Option<Severity>;   // labels, OTel short names, "1".."24"; case-insensitive
}
impl fmt::Display for Severity                    // label if any, else OTel short name

pub struct LevelNames { /* &'static [(&'static str, Severity)] */ }
impl LevelNames {
    pub const fn new(pairs: &'static [(&'static str, Severity)]) -> LevelNames;
    pub fn name_of(&self, severity: Severity) -> Option<&'static str>; // exact, else nearest lower
    pub fn severity_of(&self, name: &str) -> Option<Severity>;         // case-insensitive
}
```

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| SEV-01 | `Severity` and the eight named constants | P0 | 1 | 0.4 | |
| SEV-02 | `at::<N>()`, `new`, `saturating`, `get` | P0 | 2 | 0.4 | under 1 ns |
| SEV-03 | `band`, `otel_name`, `label`, `Display`, `parse` | P1 | 2 | 0.4 | table lookup |
| SEV-04 | `LevelNames` project tables (F5) | P2 | 2 | 0.4 | linear scan of a short slice |

### 8.6 Categories (CAT)

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
    pub const fn from_grpc(code: u8) -> Option<Category>;     // 0 (OK) and >16 -> None
    pub const fn exit_code(self) -> u8;
    pub const fn is_transient(self) -> bool;
    pub const fn retry_advice(self) -> RetryAdvice;
    pub const fn may_retry_automatically(self, idempotent: bool) -> bool;
    pub const fn default_severity(self) -> Severity;
    pub const fn default_action(self) -> Action;
    pub const fn default_public_message(self) -> &'static str;
    pub const fn name(self) -> &'static str;                  // "deadline_exceeded"
    pub fn parse(text: &str) -> Option<Category>;             // snake_case, SCREAMING_CASE, CamelCase
    #[cfg(feature = "std")] pub fn from_io_kind(kind: std::io::ErrorKind) -> Category;
}
#[non_exhaustive] pub enum RetryAdvice { No, Yes, Later, RestartTransaction, AfterFix }
#[non_exhaustive] pub enum Action { None, Retry, FixRequest, FixConfig, Operator, PageOnCall }
```

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| CAT-01 | `Category`, `ALL`, `name`, `parse`, `Display` | P0 | 1 | 0.4 | |
| CAT-02 | `http_status` | P0 | 1 | 0.4 | table |
| CAT-03 | `grpc_code`, `from_grpc` | P1 | 2 | 0.4 | table |
| CAT-04 | `exit_code` | P1 | 1 | 0.4 | table |
| CAT-05 | `is_transient`, `retry_advice`, `may_retry_automatically`, `RetryAdvice` (R21) | P0 | 1 | 0.4 | table |
| CAT-06 | `default_severity`, `default_action`, `default_public_message`, `Action` | P0 | 2 | 0.4 | table |
| CAT-07 | `from_io_kind` (`std`) | P2 | 2 | 0.4 | match |

### 8.7 Kinds (KND)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Kinds(u64);
impl Kinds {
    pub const NONE: Kinds; pub const ALL: Kinds;
    pub const ERROR: Kinds; pub const WARNING: Kinds; pub const NOTICE: Kinds;
    pub const DEPRECATED: Kinds; pub const SECURITY: Kinds; pub const FUTURE: Kinds;
    pub const RUNTIME: Kinds; pub const RESOURCE: Kinds; pub const CONFIG: Kinds;
    pub const USER: Kinds; pub const PANIC: Kinds; pub const AUDIT: Kinds; pub const TRIGGER: Kinds;
    pub const PLATFORM: Kinds;                       // bits 0-31
    pub const PROJECT: Kinds;                        // bits 32-63
    pub const fn project<const N: u8>() -> Kinds;    // compile error unless N < 32
    pub const fn try_project(bit: u8) -> Option<Kinds>;
    pub const fn contains(self, other: Kinds) -> bool;      // all of other's bits
    pub const fn intersects(self, other: Kinds) -> bool;    // any shared bit
    pub const fn is_empty(self) -> bool;
    pub const fn union(self, other: Kinds) -> Kinds;
    pub const fn bits(self) -> u64;
    pub const fn from_bits(bits: u64) -> Kinds;             // any bits accepted (Tier 3)
    pub fn iter(self) -> KindsIter;                         // single-bit Kinds, low to high
}
impl BitOr, BitOrAssign, BitAnd, BitAndAssign, Not, Sub for Kinds
impl fmt::Display for Kinds     // "ERROR|SECURITY|project(3)"; "NONE" when empty
impl fmt::Debug for Kinds       // Kinds(ERROR|SECURITY|project(3))
```

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| KND-01 | `Kinds`, platform constants, `NONE`, `ALL`, `PLATFORM`, `PROJECT` | P0 | 1 | 0.4 | |
| KND-02 | `project::<N>()`, `try_project` | P0 | 1 | 0.4 | compile time |
| KND-03 | operators, `contains`, `intersects`, `is_empty`, `union` | P0 | 1 | 0.4 | one instruction |
| KND-04 | `Display`, `Debug`, `iter` | P1 | 2 | 0.4 | |
| KND-05 | `bits`, `from_bits` | P1 | 3 | 0.4 | |

`contains(NONE)` is `true` for every mask (empty set is a subset); `intersects(NONE)` is `false`. Both are documented with examples, because PHP's `&` reads like `intersects`, not `contains`.

### 8.8 Metadata and catalog (INF)

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
    fn type_name(&self) -> &'static str { core::any::type_name::<Self>() }   // generated code returns "Type::Variant"
}

#[non_exhaustive]
pub struct CodeInfo { pub code: Code, pub title: &'static str, pub help: &'static str,
    pub docs: Option<&'static str>, pub severity: Severity, pub category: Category,
    pub kinds: Kinds, pub action: Action, pub public: &'static str,
    pub state: CodeState, pub type_name: &'static str }
impl CodeInfo { pub fn url(&self) -> Option<UrlDisplay<'_>>; pub const fn is_active(&self) -> bool; }

#[non_exhaustive]
pub enum CodeState { Active, Deprecated { since: &'static str, replacement: Option<Code> }, Retired }

pub mod catalog {
    pub fn find(catalogs: &[&'static [CodeInfo]], code: Code) -> Option<&'static CodeInfo>;
    pub fn check_unique(catalogs: &[&'static [CodeInfo]]) -> Result<(), DuplicateCode>;
}
#[non_exhaustive] pub struct DuplicateCode { pub code: Code, pub first: &'static str, pub second: &'static str }

pub type Location = &'static core::panic::Location<'static>;
#[track_caller] pub fn here() -> Location;
```

(`core::any::type_name` accepts `?Sized` types, so the default works for hand-written impls; called through `dyn ErrorInfo` it returns the concrete type's name, because each impl gets its own copy of the default method. Generated code overrides it with `"Type::Variant"`.)

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| INF-01 | `ErrorInfo` with defaults | P0 | 1 | 0.3 (defaults), 0.4 (all methods) | match on a discriminant |
| INF-02 | `CodeInfo`, `CodeState` | P0 | 2 | 0.4 | static data |
| INF-03 | `CodeInfo::url` / `UrlDisplay` | P1 | 2 | 0.4 | no allocation |
| INF-04 | `ErrorInfo for std::io::Error` (`std`): category from `kind()`, kinds `ERROR \| RESOURCE` for storage kinds | P2 | 1 | 0.4 | match |
| INF-05 | `catalog::find` | P1 | 2 | 0.4 | linear over the slices [est. under 1 us for 1,000 codes] |
| INF-06 | `catalog::check_unique`, `DuplicateCode` (R23 boot side helper for L7) | P1 | 2 | 0.4 | sort-free O(n log n) with a stack bitmap per namespace [est.] |
| INF-07 | `Location`, `here()` | P1 | 1 | 0.3 | one pointer |

### 8.9 Probe (PRB)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Identity<'a> { pub code: Option<Code>, pub severity: Severity, pub category: Category,
    pub kinds: Kinds, pub has_public: bool, pub transparent: bool, pub type_name: &'a str }

pub fn identify<'a>(error: &'a (dyn core::error::Error + 'static)) -> Option<Identity<'a>>;
pub fn innermost_code<'a>(error: &'a (dyn core::error::Error + 'static)) -> Option<(Code, Identity<'a>)>; // walks source(), max 64 steps
```

`Error::description` returns `&str` borrowed from `&self`, so the decoder cannot promise `'static` for a string that may come from a foreign type. The signatures are therefore `pub struct Identity<'a> { .., pub type_name: &'a str }` and `pub fn identify<'a>(error: &'a (dyn Error + 'static)) -> Option<Identity<'a>>`; `innermost_code` returns `Option<(Code, Identity<'a>)>`. Generated types happen to return `'static` probes; callers that need an owned name copy it (as `ErrorRecord` does).

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| PRB-01 | probe format (7.11), encoder in `__private` | P0 | 3 | 0.4 | compile time |
| PRB-02 | `identify` | P0 | 3 | 0.4 | about 10 ns for a generated type, about 2 ns for a foreign one [est.] |
| PRB-03 | `innermost_code` | P1 | 3 | 0.4 | per step as above |

### 8.10 Secret (SEC)

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| SEC-01 | `Secret<T>`, `new`, `From<T>` | P0 | 1 | 0.3 | zero (newtype) |
| SEC-02 | `expose`, `into_inner` | P0 | 1 | 0.3 | zero |
| SEC-03 | `Debug`/`Display` print `[redacted]`; `Clone`, `Copy`, `Default`, `Hash` when `T` has them; no `Deref`, `PartialEq`, `AsRef`, `Borrow` | P0 | 1 | 0.3 | |

### 8.11 Facade (FAC)

```rust
pub trait Reporter: Send + Sync { fn receive(&self, event: &Event<'_>); fn flush(&self) {} }

pub fn set_reporter(reporter: &'static dyn Reporter) -> Result<(), SetReporterError>;
#[cfg(feature = "alloc")]
pub fn set_boxed_reporter(reporter: Box<dyn Reporter>) -> Result<(), SetReporterError>;
pub fn is_installed() -> bool;
pub fn reporter() -> Option<&'static dyn Reporter>;
pub fn reporter_installed_at() -> Option<Location>;  // where set_reporter succeeded (T7 audits)
pub fn flush();

#[track_caller] pub fn report(error: &(dyn core::error::Error + 'static));
#[track_caller] pub fn report_with(error: &(dyn core::error::Error + 'static), severity: Severity, kinds: Kinds);
#[track_caller] pub fn notice(message: &str);
#[track_caller] pub fn notice_with(kinds: Kinds, message: &str);
#[track_caller] pub fn warn(message: &str);
#[track_caller] pub fn warn_with(kinds: Kinds, message: &str);
#[track_caller] pub fn trigger(kinds: Kinds, message: &str);
#[track_caller] pub fn deprecated(item: &str);
#[track_caller] pub fn soft<E: core::error::Error + 'static>(error: E) -> Result<(), E>;

#[macro_export] macro_rules! notice { ($($arg:tt)+) => { ... } }
#[macro_export] macro_rules! warn { ($($arg:tt)+) => { ... } }          // once per call site
#[macro_export] macro_rules! trigger { ($kinds:expr, $($arg:tt)+) => { ... } }
#[macro_export] macro_rules! deprecated { ($item:literal $(, since = $s:literal)? $(, replacement = $r:literal)?) => { ... } }

#[non_exhaustive] pub enum WarnMode { Once, Always, Ignore, Error }
pub fn set_warn_mode(mode: WarnMode);
pub fn warn_mode() -> WarnMode;
pub fn set_min_severity(min: Option<Severity>);
pub fn set_kinds_mask(mask: Kinds);                 // default Kinds::ALL
pub fn stats() -> FacadeStats;                      // site_table_overflow, reentrant_dropped, reporter_panics

#[non_exhaustive] pub struct SetReporterError { pub installed_at: Option<Location> }
impl fmt::Display for SetReporterError  // "a reporter is already installed (at src/boot.rs:41:5); install it once, at start-up"

impl<'a> Event<'a> { pub fn new(verb: Verb, location: Location) -> Event<'a>; /* with_* setters, Tier 3 */ }
```

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| FAC-01 | `Reporter` trait | P0 | 1 | 0.5 | |
| FAC-02 | `set_reporter`, `SetReporterError` with `installed_at` | P0 | 1 | 0.5 | once |
| FAC-03 | `set_boxed_reporter` (`alloc`) | P1 | 2 | 0.5 | one leak, once |
| FAC-04 | `report` | P0 | 1 | 0.5 | one load when empty; under 10 ns plus the reporter when installed [est.] |
| FAC-05 | `notice`, `notice!` | P0 | 1 | 0.5 | as above |
| FAC-06 | `warn`, `warn!` with once per call site (R9) | P0 | 1 | 0.5 | macro: one load + one swap; function: one load + hash + probe [est. under 5 ns] |
| FAC-07 | `trigger`, `trigger!` | P0 | 1 | 0.5 | as `notice` |
| FAC-08 | `deprecated`, `deprecated!` (R10) | P1 | 1 | 0.5 | as `warn` |
| FAC-09 | `soft` (R9) | P1 | 1 | 0.5 | as `warn` |
| FAC-10 | `Event`, `Verb`, getters | P0 | 2 | 0.5 | on the stack |
| FAC-11 | `report_with`, `notice_with`, `warn_with` | P1 | 2 | 0.5 | |
| FAC-12 | `set_warn_mode`, `warn_mode`, `WarnMode` | P1 | 2 | 0.5 | one `fetch_update` |
| FAC-13 | `set_min_severity`, `set_kinds_mask` (prefilter before formatting) | P1 | 2 | 0.5 | |
| FAC-14 | `is_installed`, `reporter`, `reporter_installed_at`, `flush` | P1 | 2 | 0.5 | one load |
| FAC-15 | reentrancy guard and panic isolation (`std`) | P0 | n/a | 0.5 | one thread-local access when installed |
| FAC-16 | `stats()` / `FacadeStats` | P2 | 2 | 0.5 | relaxed loads |
| FAC-17 | `Event::new` and `with_*` setters (for reporters' tests and L7) | P2 | 3 | 0.5 | |
| FAC-18 | facade compiled out on targets without compare-and-swap | P0 | n/a | 0.5 | zero |
| FAC-19 | `unsafe fn set_reporter_racy` for targets without compare-and-swap | P3 | 3 | after 1.0 | |
| FAC-20 | `report_record(&ErrorRecord)` and `Event::record` (`alloc`; report-err X-8, D-010) | P1 | 2 | 0.5 | one load when empty |
| FAC-21 | creation-hook slot: `CreationHook`, `CreationInfo`, `Annotate`, `set_creation_hook`, `creation_hook` (report-err X-9, D-011) | P1 | 3 | 0.5 | one load when empty |

Behaviour of every verb, in order: (1) load the control word; return if not ready (`soft`: return `Err` if mode is `Error`, else `Ok`); (2) for `warn`/`deprecated`: apply the mode and the site flag; (3) compute severity and kinds (probe for `report`); return if below the minimum severity or outside the mask; (4) with `std`, check and set the reentrancy flag; (5) build the `Event` on the stack; (6) call `receive` (inside `catch_unwind` with `std`); (7) clear the flag. No step allocates. No step formats unless the reporter formats.

### 8.12 Record (REC, feature `alloc`)

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| REC-01 | `ErrorRecord` (7.15) with `ChainEntry`, `TrailEntry` | P0 | 2 | 0.5 | |
| REC-02 | `ErrorRecord::from_error(&dyn Error)`, `from_error_with(&dyn Error, &RecordLimits)` | P0 | 2 | 0.5 | [est. under 1 us for a three-level chain; allocations: one per chain entry plus the vectors] |
| REC-03 | `ErrorRecord::from_event(&Event)` | P1 | 2 | 0.5 | as above |
| REC-04 | `RecordLimits` with `Default` and `const fn` setters | P0 | 2 | 0.5 | |
| REC-05 | `push_field`, `push_trail`, `set_backtrace`, `set_event_id`, `set_time` (bounded; return `false` when a limit is hit) | P1 | 3 | 0.5 | |
| REC-06 | `Display` (one line) | P1 | 2 | 0.5 | |
| REC-07 | `ErrorRecord::new`, `push_chain`, `with_limits`, `set_handled` (report-err X-7, D-009) | P1 | 3 | 0.5 | one allocation per pushed entry |

### 8.13 Testing (TST, feature `testing`)

```rust
pub fn capture<R>(f: impl FnOnce() -> R) -> (R, Vec<CapturedEvent>);
pub struct CapturedEvent { pub verb: Verb, pub kinds: Kinds, pub severity: Severity,
    pub code: Option<Code>, pub message: String, pub file: &'static str, pub line: u32, pub column: u32,
    pub record: Option<ErrorRecord> }
pub struct CaptureReporter;   // installed on first use of capture(); routes to a thread-local buffer
```

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| TST-01 | `capture` (per-thread isolation, so parallel tests do not see each other) | P1 | 2 | 0.5 | test only |
| TST-02 | `CaptureReporter` | P1 | 3 | 0.5 | test only |

`capture` installs `CaptureReporter` if no reporter is installed; if another reporter is installed, it returns an error event explaining that tests must not install their own reporter alongside `capture`. The suite's `assert_code!` (F19) builds on it.

### 8.14 Root (ROOT)

| Id | Item | Grade | Tier | Ver | Cost |
|---|---|---|---|---|---|
| ROOT-01 | `VERSION` | P0 | 1 | 0.1 | |
| ROOT-02 | `prelude` | P0 | 1 | 0.3 | |
| ROOT-03 | crate-level docs: Tier 1 first, "what can never be observed" (7.19), feature table | P0 | 1 | 0.3 | |

---

## 9. Parity matrix

Legend: yes, no, part (partial), n/a. Checked against the documentation of each crate on 2026-10-22 where a URL is in section 18; cells marked [unverified] come from memory and are rechecked at 0.6.

| Capability | define-err | thiserror 2 | displaydoc 0.2 | snafu | error-forge 1.0 | miette | log (facade) |
|---|---|---|---|---|---|---|---|
| `Display` from a message template | yes (`=>`) | yes (`#[error]`) | yes (doc comments) | yes | yes | via thiserror | n/a |
| `Error::source` and `From` | yes | yes | no | yes | part (field named `source`) | via thiserror | n/a |
| transparent wrapping | yes | yes | no | yes [unverified] | no | yes [unverified] | n/a |
| `{0}` positional placeholders | no (L2: yes) | yes | yes | n/a | no | n/a | n/a |
| works without a proc macro | **yes** | no | no | no | part (`define_errors!` needs `pastey`) | no | yes |
| zero dependencies | **yes** | no (syn, quote, proc-macro2 at build) | no | no | no | no | yes |
| `no_std` | yes | yes (`default-features = false`) [unverified on docs.rs page] | yes | yes [unverified] | no | no | yes |
| non_exhaustive by default | **yes** | no | no | no | no | no | n/a |
| stable error codes | **yes (const, packed u64)** | no | no | no | part (strings, runtime registry) | yes (strings) | no |
| catalog with help and URL | **yes (const)** | no | no | no | part (runtime registry) | part (per diagnostic) | no |
| never-reuse and duplicate check | **yes (compile time)** | no | no | no | runtime only | no | no |
| severity on a standard scale | **yes (OTel 1-24)** | no | no | no | part (5 levels) | part (3 levels) | yes (5 levels) |
| categories with HTTP, gRPC, exit codes | **yes** | no | no | no | part (per-variant numbers) | no | no |
| kind masks | **yes** | no | no | no | no | no | no |
| public and private messages | **yes** | no | no | no | part (`user_message`) | no | no |
| retry metadata (`retry_after`, transient) | **yes** | no | no | no | part (`is_retryable`) | no | no |
| location capture | yes (`#[location]`, `?`) | no (backtrace on nightly) | no | yes (implicit location) | no | no | yes (macros) |
| secret redaction type | **yes** | no | no | no | no | no | no |
| reporting facade from any crate | **yes** | no | no | no | part (one hook, fires on creation) | no | yes (logs) |
| once per call site warnings | **yes** | no | no | no | no | no | no |
| owned error record | yes (`alloc`) | no | no | no | no | no | no |
| runtime deprecation warnings | **yes** | no | no | no | no | no | no |
| generic error types | part (P3; L2: yes) | yes | yes | yes | no | yes | n/a |
| backtraces | no (L3) | part (nightly `provide`) | no | yes | part | no | n/a |

Where `define-err` is behind (positional placeholders, generics, backtraces), the gap is closed by L2 or L3 by design, not left open.

---

## 10. Security

### 10.1 Threat model

`define-err` sits in every library, so its threats are about what errors **say** (to clients, to logs, to other nodes) and about the one shared hook. Attackers considered: a remote client who sees error responses; a log reader with partial access; a malicious or compromised dependency in the same process; a peer node sending codes.

| Id | Threat | Defence | Attack test | Cost |
|---|---|---|---|---|
| T1 | Internal details leak to clients (paths, SQL, shard ids in a 500 body) | Public and private messages are separate (`public_message()` vs `Display`); the public default is the category's generic sentence, never `Display`; `ErrorRecord.public` is built only from public messages (T15) | `test_public_message_never_contains_display_text`: every generated variant with a `Display` containing a marker string has a public message without it | none at run time |
| T2 | Secrets in error context reach logs through `Display` or `Debug` | `Secret<T>`: both print `[redacted]` under every flag combination; no `Deref`/`AsRef`/`Borrow`, so the value cannot reach a format string by accident | proptest over all format specs (`{}`, `{:?}`, `{:#?}`, `{:>40}`, `{:.3}`, `{:^9}`) on `Secret<String>` with random content: output never contains the content; `ErrorRecord` of an error with a `Secret` field contains `[redacted]` | none |
| T3 | Format-string injection (user text interpreted as a template) | Templates are compile-time literals (macros only); every `&str` verb passes text through `format_args!("{}", text)` | `notice("{password}")` delivers the literal text `{password}`; a UI test shows a non-literal template does not compile | none |
| T4 | Log injection: newlines and control characters in messages forge log lines | `define-err` never writes logs; it records text verbatim and marks nothing as safe. Sinks escape (L4, log-io). `Code`, `Severity`, `Category`, `Kinds` text forms can never contain control characters (validated alphabet). `ErrorRecord` stores `has_control` per chain entry so sinks can choose a path without rescanning | fuzz: no `Code`/`Severity`/`Kinds` `Display` output contains a byte below 0x20; record of a message with `\n` sets the flag | one byte scan per recorded message [est. under 1 ns per 16 bytes] |
| T5 | Memory exhaustion through huge or cyclic error chains | `ErrorRecord` caps depth (`max_chain`), bytes per message, total bytes; the bounded writer stops consuming a `Display` at the cap; a `source()` that returns itself stops at `max_chain`; `innermost_code` stops at 64 | an error whose `source()` returns `self` and whose `Display` writes 1 GiB lazily: record finishes, `truncated` set, peak allocation under `max_total_bytes` plus vector overhead (counting allocator) | bounded writer check per write call |
| T6 | Error floods through the facade slow every caller | Verbs never block, never allocate, never take a lock; disabled kinds and severities return before formatting; once-per-site for warnings; storm control is L7's | 64 threads x 10 million `warn!` on one site: one delivery; `report` with a prefilter excluding it never calls `Display` (counting `Display` impl) | one load (nothing installed) |
| T7 | Reporter hijack: a dependency calls `set_reporter` first, to read every error (exfiltration) or to silence reporting | The slot is set once and never replaced; the application's own `set_reporter` then fails with `SetReporterError { installed_at }`, which names the file and line of the first installer; the documented boot rule is "install first, and treat `Err` as a fatal boot error" (fail closed). `reporter_installed_at()` exposes the location for audits. | a test crate installs a reporter from a "dependency" module before `main`'s; `main`'s call fails and the error text contains the dependency's file path | one stored pointer |
| T8 | Unbounded recursion: a reporter reports while reporting | `std`: thread-local reentrancy flag drops nested events and counts them; `no_std`: contract in `docs/FACADE.md` | reporter that calls `report` inside `receive`: one delivery, `reentrant_dropped == 1`, no stack overflow | one thread-local access |
| T9 | A panicking reporter unwinds through library code that only wanted to report | `std`: `catch_unwind` around `receive`, payload dropped under a second `catch_unwind` | reporter that panics, and one whose payload panics on drop: the verb returns normally, `reporter_panics` counts both | zero when nothing panics |
| T10 | Code collisions make two errors indistinguishable, or a retired code returns with a new meaning | compile-time duplicate check across the block and the retired list; namespace text validated; `catalog::check_unique` for L7's boot check across crates | UI tests for duplicate and retired numbers; `check_unique` on two catalogs sharing `DB-0001` returns both type names | compile time |
| T11 | Probe forgery: a foreign error type returns a crafted `description()` to claim another crate's code or the `SECURITY` kind | Codes identify, they do not authenticate: documented in `docs/SECURITY.md`. `identify` validates every field (code layout, severity range, category range, ASCII name) and rejects anything malformed. L7 compares the claimed code's catalog type name with the probe's type name and flags a mismatch | fuzz the decoder with arbitrary strings: never panics, never accepts an invalid field; a forged probe with a valid code but a mismatching type name is detected by `catalog::find(..).type_name` comparison (test documents the L7 check) | decoder validation inside the 10 ns budget |
| T12 | Malformed codes from untrusted input (headers, config files, peer messages) | `Code::parse` and `Code::from_u64` accept only the exact grammar and layout; bounded length (at most 15 bytes) checked before any work | fuzz targets `code_parse`, `code_from_u64`; property: `parse(display(c)) == c` for every valid code; every rejected input returns an error without panic | under 15 ns |
| T13 | Data race or use of an uninitialised reporter in the unsafe slot | state machine with acquire and release ordering (7.14.8); loom models of every interleaving of two installers and one reader; Miri over the full suite | loom: `loom_install_race`, `loom_read_during_install`; Miri job | none beyond the atomics |
| T14 | Expensive formatting forced on hot paths by disabled events | events carry `fmt::Arguments` and borrowed errors; nothing formats until a reporter does; prefilter by severity and kinds before the event is built | counting `Display` impl: zero calls when filtered or not installed | none |
| T15 | Private text promoted to public through records | `ErrorRecord.public` comes only from `public_message()` of the chosen entry or its category default; chain `Display` output goes only into `chain[..].message` | record of a chain whose inner `Display` contains a marker: `public` never contains it | none |
| T16 | Kind squatting: a project uses platform bits for its own meaning, so security listeners receive noise or miss events | `Kinds::project::<N>()` can only produce bits 32 to 63; platform constants are fixed and documented as never reassigned; `from_bits` is Tier 3 and documented as "for decoding only" | UI test: `project::<32>()` does not compile; unit test: every `project` value is disjoint from `PLATFORM` | none |
| T17 | Location data reveals build paths (user names, directory layout) in public output | `Location` is never part of `public_message`; L4 strips or relativises paths in public renderings (documented requirement on L4); `ErrorRecord.trail` is private data | test: no public field of any type here contains a `file!()` path | none |

### 10.2 Defence ideas per version (standard 2.4)

| Version | Hypothesis | Method | Expected cost |
|---|---|---|---|
| 0.2 | Error text that echoes rejected input lets a hostile string (control characters, forged log lines, a megabyte of text) reach logs through the parser's own error | `CodeParseError` and every other error type here describe the input class and the fix, never the input itself (D-017); a test feeds every rejected input of E-47 to E-52 and checks the `Display` output contains none of its bytes | none |
| 0.3 | Most secret leaks come from `Debug` derives on error types holding tokens | `Secret<T>` plus a doc section "fields that must be `Secret`"; a clippy-style UI test showing the redacted output | none |
| 0.4 | Clients learn internals through default 500 bodies | category default public messages; L4 problem details uses only them | none |
| 0.5 | A hostile dependency can capture errors by installing first | `installed_at` location in `SetReporterError` and `reporter_installed_at()` | 8 bytes |
| 0.6 | Repeated warnings hide new ones in floods | once-per-site table with an overflow counter instead of silent loss | 4 KiB static |
| 0.7 | Forged probes could spoof security kinds | strict probe validation plus the L7 cross-check against catalogs | under 2 ns |
| 0.8 | Path disclosure in locations | `Location` display helper that prints paths relative to the crate root (`ErrorRecord::trail` keeps both) | small string scan, only when rendering |
| 0.9 | Control characters in messages | `has_control` flag computed once at record time | under 1 ns per 16 bytes |

---

## 11. Performance

### 11.1 Targets

| Operation | Target | Compared with | How measured |
|---|---|---|---|
| construct and return a generated error | identical to a hand-written enum (same instructions, no allocation, no atomic) | thiserror, hand-written | criterion; assembly diff in `docs/PERFORMANCE.md` |
| `Display` of a generated variant | within 2% of thiserror for the same format string (noise) | thiserror | criterion |
| `size_of` of a generated type | equal to the hand-written type | thiserror | compile-time assertions |
| `code()`, `category()`, `severity()`, other `ErrorInfo` methods | under 1 ns [est.] | error-forge inherent methods | criterion |
| `Code` `Display` | under 20 ns, no allocation [est.] | error-forge `format!` of a `String` code (allocates) | criterion, counting allocator |
| `Code::parse` | under 15 ns [est.]; at least 20% faster than a `String`-based lookup | error-forge `ErrorRegistry::get_code_info` (lock + hash) | criterion |
| catalog lookup by code | constant (generated `match`) | error-forge `RwLock<HashMap>` | criterion |
| verb with nothing installed | one atomic load, under 1 ns [est.]; at least 20% faster than `log::warn!` with no logger and max level `Off` | `log` 0.4 | criterion; assembly listing |
| `warn!` after its first delivery | under 2 ns [est.] | `log::warn!` filtered | criterion |
| `warn()` function after first delivery | under 5 ns [est.] | | criterion |
| `report` filtered out by severity | under 3 ns [est.] | | criterion |
| `report` to a no-op reporter | under 10 ns plus the virtual call [est.] | `log::error!` to a no-op logger | criterion |
| `identify` | under 10 ns for a generated type, under 2 ns for a foreign type [est.] | | criterion |
| `ErrorRecord::from_error`, three-level chain | under 1 us [est.]; allocations counted and published | none (no competitor) | criterion, counting allocator |
| clean build of a crate defining 20 error enums | at least 20% faster than the same crate with thiserror 2 [est.: thiserror pulls syn, quote, proc-macro2] | thiserror 2 | `cargo build` from clean, 10 runs, median, on Windows and Linux |
| incremental rebuild after editing one message | no slower than thiserror | thiserror 2 | same |
| expansion of a 500-variant enum | under 1 s of macro expansion [est.] | thiserror 2 (500-variant derive) | `-Z time-passes` is nightly; measured as total build-time delta on stable |

### 11.2 Benchmarks

`benches/definitions.rs` (construction, `Display`, `ErrorInfo`), `benches/codes.rs` (display, parse, `from_u64`, catalog), `benches/facade.rs` (every verb in the three states: not installed, filtered, delivered; single thread and 16 threads on one site), `benches/record.rs`, plus `tools/build-time/` (a script that generates the 20-enum test crate in two flavours and times clean builds). Baselines committed; a regression over 5% blocks the change (standard 2.2).

### 11.3 Method

Hardware, OS, toolchain, and CPU frequency policy recorded in `docs/PERFORMANCE.md`; criterion defaults with 100 samples; competitors at their latest versions on the measurement date; every number with its date. Results that miss a target either get fixed or get a reviewed reason in the decision log; nothing is published as a target that was not measured.

---

## 12. Testing plan

### 12.1 Unit and integration tests

- Every module has unit tests at the bottom of the file for every function and every `Ok`/`Err` path, named `test_<subject>_<condition>_<expected>`.
- `tests/define_basic.rs`, `tests/define_meta.rs`, `tests/define_structs.rs`: every form of 7.4 with `Display`, `source()`, `From`, `ErrorInfo`, and catalog assertions.
- `tests/conformance/`: every thiserror test case expressible in `define_errors!`, with byte-identical `Display` output and identical `source()` chains against the same type written with thiserror (a dev-dependency).
- `tests/facade.rs`: each verb in each state; each test runs in its own process (`harness = false` binaries or `#[test]` with a re-exec helper) because the slot is set once per process.
- `tests/traits.rs`: `Send`/`Sync`/`Copy` assertions (7.18).
- `tests/no_prelude.rs`: generated code in a module with `#![no_implicit_prelude]` and with user types named `Error`, `Result`, `Option`, `Box`, `Display`, `Location`, `Code` in scope.

### 12.2 Compile-fail tests (`trybuild`)

One case per compile error listed under 8.3, plus `Kinds::project::<32>()`, `Severity::at::<0>()`, `Severity::at::<25>()`, `namespace!("")`, `namespace!("TOOLONG")`, `namespace!("1DB")`, `code!("DB", 16777216)`. Each `.stderr` file is reviewed for the FR-28 standard (problem plus fix in one sentence). Run on stable only; messages are pinned to the stable version in CI and refreshed deliberately.

### 12.3 Property tests (proptest)

- `Code`: `from_u64(c.as_u64()) == Some(c)`; `parse(c.to_string()) == Ok(c)`; ordering of codes equals ordering of `(namespace text, number)`; hash and equality ignore width.
- `Severity`: `parse(s.to_string()) == Some(s)`; `new(n).is_some() == (1..=24).contains(&n)`.
- `Kinds`: set laws (union commutative, `contains` reflexive, `a.contains(b) && b.contains(c)` implies `a.contains(c)`), `project` disjoint from `PLATFORM`.
- `Category`: `from_grpc(c.grpc_code()) == Some(c)`; `parse(c.name()) == Some(c)`.
- Probe: `decode(encode(x)) == x` for every valid tuple; decoder never panics on arbitrary strings.
- `ErrorRecord`: never exceeds `max_total_bytes` of message data; truncation always on a char boundary.
- `Secret`: no format spec reveals the content.

### 12.4 Fuzzing (cargo-fuzz, corpus outside the repository)

Targets: `code_parse`, `code_from_u64`, `severity_parse`, `category_parse`, `probe_decode`, `record_from_chain` (arbitrary chain shapes and `Display` outputs, checking limits). Each runs 24 hours before 1.0 with no open finding.

### 12.5 Concurrency: loom and Miri

- loom: `loom_install_race` (two installers, exactly one wins, the loser sees `installed_at`), `loom_read_during_install` (reader sees `None` or the full reference), `loom_site_flag` (two threads, one delivery), `loom_site_table` (insert races, table full), `loom_mode_update` (mode changes never disturb the state bits).
- Miri: the whole test suite except trybuild and fuzz, under `-Zmiri-strict-provenance`.
- Stress: 64 threads x every verb x 10 million iterations against a counting reporter; exact counts asserted.
- Soak: 8 hours of the stress mix; memory flat (no growth beyond the fixed statics), counts exact.

### 12.6 Counting-allocator tests

Every "no allocation" claim: generated construction, `Display` into a fixed buffer, every `ErrorInfo` method, `Code` display and parse, `identify`, every verb in every state, `Secret` formatting. `ErrorRecord` tests assert the exact number of allocations for fixed inputs.

### 12.7 `no_std` and targets

CI builds and runs unit tests with `--no-default-features` and with `--features alloc` on hosts; cross-checks (build only) `thumbv7em-none-eabihf`, `thumbv6m-none-eabi` (facade compiled out), `riscv32imac-unknown-none-elf`, `wasm32-unknown-unknown`. A test crate in `tests/no_std_user/` (`#![no_std]`, panic handler stub) uses `define_errors!` with every attribute and is built for `thumbv7em-none-eabihf`.

### 12.8 Edge cases

**Macro input**

| Id | Case | Expected |
|---|---|---|
| E-1 | enum with zero variants | compiles; `Display` and `source` are `match *self {}`; `ErrorInfo` defaults |
| E-2 | enum with one variant | compiles; no unreachable-pattern warnings |
| E-3 | enum with 500 variants, mixed forms | compiles at the default recursion limit (NFR-6) |
| E-4 | unit, braced, tuple, empty braced `V {}`, empty tuple `V()` variants | all compile and display |
| E-5 | user enum named `Error`, `Result`, `Option`, `Display`, `Code`, `Location` | compiles (FR-27) |
| E-6 | user crate with `#![no_implicit_prelude]`, `#![no_std]`, `#![forbid(unsafe_code)]`, `#![deny(warnings, missing_docs)]` | compiles without warnings |
| E-7 | macro invoked inside a function body (local types) | compiles |
| E-8 | two blocks in one module, both with a namespace | no name clash between generated `const` items |
| E-9 | visibility `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, private | passed through exactly |
| E-10 | field visibility in structs (`pub line: u32`) | passed through |
| E-11 | message with `{{` and `}}` | literal braces |
| E-12 | message with no placeholders | `write_str`-equivalent output |
| E-13 | message with `{field:?}`, `{field:#010x}`, `{field:>8}`, `{field:.3}` | rustc formatting semantics |
| E-14 | raw string message `r"C:\path {p}"`, Unicode message | compiles, displays exactly |
| E-15 | message referencing a missing field | rustc error pointing at the literal |
| E-16 | `{0}` in a tuple variant's message | compile error; docs note |
| E-17 | extra-argument form using a field expression (`diagnostic.format()`) and a path constant (`crate::MIN`) | compiles; output correct |
| E-18 | field named `source` that does not implement `Error` | compile error naming the field |
| E-19 | `#[source]` on `Option<io::Error>`, `None` and `Some` values | `source()` returns `None` / `Some` |
| E-20 | `Box<dyn Error + Send + Sync>` source, and `Box<dyn Error>` | `source()` returns the inner error |
| E-21 | two `#[from]` with the same source type | conflicting `From` impls compile error, with the macro's note |
| E-22 | `#[from]` plus an extra non-location field | compile error |
| E-23 | `#[from]` plus `#[location]`; conversion through `?` | location is the `?` line (track_caller through `From`) |
| E-24 | `#[location]` with a value built by hand via `here()` | location is the `here()` call |
| E-25 | `transparent` over a type with codes | `Display`, `source()`, and `ErrorInfo` forward |
| E-26 | `transparent` over a foreign type | `Display`, `source()` forward; `ErrorInfo` defaults |
| E-27 | `#[cfg(feature = "x")]` on a variant (user's feature off and on) | arms follow the variant |
| E-28 | doc comments on variants and fields, `#[allow(..)]` on variants | passed through |
| E-29 | `#[exhaustive]` on an enum; on a struct (meaningless) | enum: no `non_exhaustive`; struct: compile error "only enums" |
| E-30 | `#[manual_debug]` without a user `Debug` impl | compile error from `Error`'s `Debug` bound (documented) |
| E-31 | user also writes `#[derive(Debug)]` without `#[manual_debug]` | compile error "conflicting implementations"; macro docs explain |
| E-32 | trailing comma present and absent; attributes in any order | identical output |

**Codes and catalog**

| Id | Case | Expected |
|---|---|---|
| E-33 | code number 0 | generic code, `is_generic()` true |
| E-34 | number 16,777,215; 16,777,216 | first compiles; second is a compile error |
| E-35 | negative literal, float literal, `1_000`, hex literal `0x2A`, non-literal constant | `1_000` and `0x2A` accepted as integer literals (display decimal); negative, float, non-literal rejected |
| E-36 | duplicate number in one enum; across two enums in a block; equal to a retired number | compile errors with namespace in the message and a span on the second literal |
| E-37 | same number in two different blocks or namespaces | allowed |
| E-38 | `#[code]` without `namespace` header; without `help` | compile errors (FR-10) |
| E-39 | namespace `""`, `"DB-1"`, `"db"`, `"D B"`, `"1DB"`, `"ABCDEFG"`, `"ÄB"` | `"db"` accepted as `DB`; the rest are compile errors |
| E-40 | width 0, 1, 8, 9 | 1 and 8 accepted; 0 and 9 compile errors |
| E-41 | number wider than the width (`DB` width 4, number 123456) | displays `DB-123456` |
| E-42 | `replacement` equal to its own number; to a missing number | compile errors |
| E-43 | deprecated code used through its named constant | compiler `deprecated` warning at the use site; `code_info().state` is `Deprecated` |
| E-44 | docs template without `{code}`, `{namespace}`, or `{number}` | compile error |
| E-45 | docs template with `{code}` twice | both replaced |
| E-46 | `catalog::check_unique` on empty input, one catalog, two catalogs with a collision, catalogs with retired entries | `Ok`, `Ok`, `Err` naming both types, retired entries count as taken |

**Code text and values**

| Id | Case | Expected |
|---|---|---|
| E-47 | `parse("")`, `parse("-")`, `parse("DB-")`, `parse("-0042")` | `Empty`, `MissingSeparator`/`BadNamespace`, `BadNumber`, `BadNamespace` |
| E-48 | `parse("db-0042")`, `parse("DB-42")`, `parse("DB-00000042")` | all equal `DB-0042` (identity); widths 4, 2, 8 |
| E-49 | `parse("DB-123456789")` (9 digits), `parse("DB-16777216")` | `BadNumber`, `NumberTooLarge` |
| E-50 | `parse(" DB-1")`, `parse("DB-1 ")`, `parse("DB--1")`, `parse("DB-+1")`, `parse("DB-0x2A")`, `parse("DB-1-2")` | errors |
| E-51 | Unicode hyphen `DB‐1` (U+2010), fullwidth digits, Arabic-Indic digits | errors |
| E-52 | `from_u64` with reserved bits set; namespace field above 37^6 - 1; padding before a character (`"D\0B"` packing); all zero | `None` for each |
| E-53 | ordering `DB-0002 < DB-0010 < DBA-0001 < E-0000` | holds |
| E-54 | `Code` hash equal for width 4 and width 5 forms | holds |

**Severity, categories, kinds**

| Id | Case | Expected |
|---|---|---|
| E-55 | `Severity::new(0)`, `new(1)`, `new(24)`, `new(25)`, `new(255)` | `None`, `Some`, `Some`, `None`, `None` |
| E-56 | `saturating(0)`, `saturating(200)` | 1, 24 |
| E-57 | `parse("warn")`, `parse("WARNING")`, `parse("error4")`, `parse("CRITICAL")`, `parse("13")`, `parse("")` | 13, 13, 20, 20, 13, `None` |
| E-58 | `Display` of 10 and of 11 | `NOTICE`, `INFO3` |
| E-59 | `Category::from_grpc(0)`, `from_grpc(17)`, `from_grpc(16)` | `None`, `None`, `Unauthenticated` |
| E-60 | `may_retry_automatically(false)` for every category | `false` |
| E-61 | `from_io_kind` for every stable `io::ErrorKind` | mapped or `Unknown`; no panic for future kinds (`_` arm) |
| E-62 | `Kinds::NONE.contains(Kinds::NONE)`, `Kinds::ERROR.contains(Kinds::NONE)`, `Kinds::NONE.intersects(Kinds::NONE)` | `true`, `true`, `false` |
| E-63 | `project::<0>()`, `project::<31>()`, `try_project(32)` | bit 32, bit 63, `None` |
| E-64 | `!Kinds::NONE == Kinds::ALL`; `Display` of `NONE`, of all bits | holds; `"NONE"`, full list with reserved bits printed as `bit(13)` |

**Facade**

| Id | Case | Expected |
|---|---|---|
| E-65 | every verb before any reporter is installed | returns; no allocation; no `Display` call |
| E-66 | `set_reporter` twice | second returns `Err` with the first call's location |
| E-67 | 16 threads call `set_reporter` at once | exactly one `Ok` |
| E-68 | `warn!` before installation, then installation, then the same `warn!` | delivered once, after installation |
| E-69 | `warn!` in a loop of 1 million | one delivery |
| E-70 | `warn!` inside a generic function instantiated with three types | one delivery |
| E-71 | 64 threads hit one `warn!` site at once | exactly one delivery |
| E-72 | `warn()` function from 600 distinct call sites (table holds 512) | 512 deliver once; the rest deliver every time; overflow counter correct |
| E-73 | `warn()` called through a `#[track_caller]` helper, and through one without | location is the helper's caller / the helper's line |
| E-74 | `report` from inside a macro defined in another crate | location is the outermost macro call site (verified by test, see section 17 unverified facts) |
| E-75 | mode `Always`, then `Once` | sites already fired stay suppressed; new sites fire once |
| E-76 | mode `Ignore` | no warning delivered; `report` still delivered |
| E-77 | mode `Error`: `soft(err)` with and without a reporter | `Err(err)` both times |
| E-78 | min severity `ERROR`: `warn`, `report` of a `WARNING`-severity error, `report` of a `CRITICAL` one | first two dropped before formatting; third delivered |
| E-79 | kinds mask `SECURITY` only: `report` of an error with `ERROR \| SECURITY`; with `ERROR` | delivered; dropped |
| E-80 | reporter calls `report` in `receive` (`std`) | nested call dropped and counted |
| E-81 | reporter panics; payload panics on drop (`std`) | verb returns; both counted |
| E-82 | `flush()` with nothing installed; with a reporter | returns; calls `Reporter::flush` |
| E-83 | `report` of a foreign error: `io::Error` with `std`; any other foreign type | `io::Error`: code `None`, category from `from_io_kind` (the facade downcasts `io::Error` as its one built-in fallback); other: code `None`, category `Unknown`; kinds `ERROR` in both |
| E-84 | `trigger(Kinds::NONE, ..)` | delivered with `TRIGGER` only |
| E-85 | `notice("{x}")` | text delivered literally |
| E-86 | facade on `thumbv6m` | `set_reporter` absent; verbs compile as no-ops |

**Probe and record**

| Id | Case | Expected |
|---|---|---|
| E-87 | `identify` on a generated type, a foreign type, a type whose `description()` returns `"\u{0}DE1"` only, a probe with a bad hex digit, with severity 0, with category 0 or 17, with reserved code bits | `Some`, `None`, `None`, `None`, `None`, `None`, `None` |
| E-88 | type name of exactly 128 bytes; of 129 bytes | first encoded; second is a compile error in the probe builder (`Type::Variant` longer than 128 bytes) |
| E-89 | `innermost_code` on a chain where the code sits at depth 0, 3, 63, 64 | found, found, found, `None` |
| E-90 | record of a chain of depth 0, 1, 32, 33, and a self-referencing chain | 1, 2, 32 (+ truncated) entries; self-reference stops at the cap |
| E-91 | message exactly at `max_message_bytes`; one byte over; over with a multi-byte character straddling the limit | kept; cut; cut before the character |
| E-92 | `Display` returning `fmt::Error` | `"<message unavailable>"` |
| E-93 | `push_field` past `max_fields`; field value over `max_field_bytes` | returns `false`, `truncated` set; value cut on a char boundary |
| E-94 | record of an error with a `Secret` field | `[redacted]` in the chain message |
| E-95 | record built from an event with no error (a `notice`) | chain empty; message in `chain` as a synthetic entry with `type_name` `"notice"` |

**Secret**

| Id | Case | Expected |
|---|---|---|
| E-96 | `format!("{:>12}", Secret::new("pw"))` | `  [redacted]` |
| E-97 | `{:#?}` of a struct holding a `Secret` | field shows `[redacted]` |
| E-98 | `Secret<Secret<String>>` | `[redacted]` |
| E-99 | `Secret<&'static str>` is `Copy` | compiles |
| E-100 | attempt `*secret` or `secret == other` | compile errors (no `Deref`, no `PartialEq`) |

### 12.9 Requirement traceability

`tests/TRACEABILITY.md` maps every FR, NFR, threat, and edge case to the test names that cover it. The 1.0 audit fails if any row has no test.

---

## 13. Documentation deliverables

| Document | Content | By |
|---|---|---|
| `README.md` | template structure (header image, badges with MSRV, pitch, features, install, quick start with Tier 1 first, status, links, license, footer); the 8.2 example; "what can never be observed" | 0.3, updated every version |
| `docs/API.md` | metrics-lib format: nested table of contents, every public item, parameters, errors, several examples per use; written in full at 0.2 | 0.2 |
| `docs/GENERATED-CODE.md` | 7.5 with complete expansions for every form; the `__private` versioning rule; the probe format | 0.2 (locked), updated with every `__private` change |
| `docs/CODES.md` | layout, namespaces, widths, catalog fields, never-reuse and deprecation rules, how L7 checks across crates | 0.4 |
| `docs/FACADE.md` | the reporter contract (fast, non-blocking, thread-safe, no verbs inside `receive` without `std`), installation rule (first, fail closed), once-per-site semantics, targets without CAS | 0.5 |
| `docs/MIGRATION.md` | section 19 in user form: from thiserror (both routes), from error-forge | 0.5 |
| `docs/ARCHITECTURE.md` | modules, the macro's two paths, the const-panic rule with every `panic!` site listed, the facade state machine, the probe | 0.7 |
| `docs/PERFORMANCE.md` | measured numbers, method, hardware, competitors, assembly of the empty-facade fast path | 0.6 |
| `docs/SECURITY.md` | threat model T1 to T17 with defences and costs | 0.7 |
| `docs/PLATFORM-NOTES.md` | feature and target matrix, atomics requirements, `panic = "abort"` notes | 0.7 |
| `docs/STABILITY-1.0.md` | frozen: API, code layout, probe format, mapping tables, kind bit assignments; MSRV policy; support window; the 1.0 audit record | 1.0 |
| `CHANGELOG.md` | Keep a Changelog 1.1.0, every change in the same commit | every version |
| `docs/release/vX.Y.Z.md` | per version, past tense | every version |
| rustdoc | every public item with a runnable example; crate docs mirror the README's Tier 1 | every version |
| `examples/` | `plain.rs` (thiserror-style), `codes.rs`, `facade.rs` (a small reporter), `no_std.rs` (built for the embedded target in CI), `migration_before_after.rs` | 0.5 |

---

## 14. Version roadmap

Every version ends with the release routine (section 15). The anti-deferral rule applies: an item moves to a later version only with a decision-log entry giving the reason. P0 items ship in the version listed and never move.

### 0.1.0 - Scaffold

API: ROOT-01.

- Repository `jamesgober/define-err` created (D-001), cloned to `error-collection/define-err`, portfolio scaffold layout (6.13), `REPS.md` byte-identical.
- `Cargo.toml`: edition 2021, MSRV 1.81 (or per OD-2), features `std` (default), `alloc`, `testing` declared with their implications and one test per mode (D-005), zero dependencies.
- `src/lib.rs`: `#![no_std]`, crate docs, lint attributes (`deny(unsafe_code)`, `deny(missing_docs)`, `docsrs`), `VERSION`.
- `dev/PLAN.md` (this specification in PLAN form), `dev/DIRECTIVES.md`, `dev/ROADMAP.md`; decision log D-001 onward records every OD answered.
- CI: Linux, macOS, Windows x stable and MSRV; four feature modes; `no_std` cross-check job (empty crate builds for every target in 7.17); security job; Miri job (trivial at this point).

**Exit:** CI green on every job; the crate published to claim the name.

### 0.2.0 - Design lock

- `docs/API.md` for every item in section 8, with examples marked by the version that brings them.
- `docs/GENERATED-CODE.md` complete (7.5), with full expansions for every form of 7.4; reviewed together with the L2 owner so L2 can start.
- Probe format, code layout, category tables, kind bits written as golden-test files (`tests/golden/`), even before the code that produces them.
- Every open decision in section 17 answered by James or accepted as recommended.
- Simplicity review (standard 2.10): a reader who has never seen the crate writes the 8.2 example from the README alone.

**Exit:** James approves `docs/API.md` and `docs/GENERATED-CODE.md`.

### 0.3.0 - Core: thiserror parity

API: DEF-01 to DEF-12, DEF-27 · INF-01 (trait with defaults), INF-07 · SEC-01 to SEC-03 · ROOT-02, ROOT-03.

- Fast and general macro paths; every 8.3 compile error with a `trybuild` case.
- Conformance suite against thiserror (12.1); `no_prelude` and naming tests (E-5, E-6).
- Edge cases E-1 to E-32 (except those needing 0.4 metadata), E-96 to E-100.
- Pilot migration on a branch of page-db (55 `#[error]` attributes, 6 positional placeholders) to prove the syntax before metadata lands; findings fed back into the decision log.

**Exit:** page-db's test suite passes on the pilot branch with byte-identical messages; benchmark shows construction and `Display` identical to thiserror.

### 0.4.0 - Extended: codes and metadata

API: DEF-13 to DEF-23, DEF-25 · CODE-01 to CODE-09 · SEV-01 to SEV-04 · CAT-01 to CAT-07 · KND-01 to KND-05 · INF-01 (all methods) to INF-06 · PRB-01 to PRB-03.

- Code packing and parsing with fuzz targets `code_parse`, `code_from_u64` started.
- Compile-time checks (7.13) with UI tests; duplicate span trick verified on stable and MSRV.
- Category tables and kind bits locked against the golden files.
- Edge cases E-33 to E-64, E-87 to E-89.

**Exit:** the 7.4.4 example compiles and every assertion in `tests/define_meta.rs` passes; Hive DB distro's five-digit codes display identically (`DIS-00001`).

### 0.5.0 - Feature complete: facade and record

API: FAC-01 to FAC-18 · REC-01 to REC-06 · TST-01, TST-02 · DEF-24.

- Slot with loom models and Miri; control word; verbs; once-per-site macro flags and function table; reentrancy and panic isolation.
- `ErrorRecord` with limits and the bounded writer.
- `docs/FACADE.md`, `docs/MIGRATION.md`, `examples/`.
- Edge cases E-65 to E-86, E-90 to E-95.

**Exit:** every P0, P1, and P2 item exists; a prototype L7 reporter (in a test) receives every verb with correct locations.

### 0.6.0 - Optimisation

- Benchmarks of 11.2 against thiserror, `log`, error-forge; build-time comparison script; numbers in `docs/PERFORMANCE.md`.
- Assembly inspection of the empty fast path (one load and a branch).
- Every target in 11.1 met or a reviewed reason recorded.

**Exit:** `docs/PERFORMANCE.md` published with dates, hardware, and method.

### 0.7.0 - Hardening

- All fuzz targets 24 hours each; attack tests T1 to T17; Miri over the suite; loom models complete; 8-hour soak.
- `docs/SECURITY.md`, `docs/ARCHITECTURE.md`, `docs/PLATFORM-NOTES.md`.

**Exit:** no open fuzz finding; every threat has a passing attack test.

### 0.8.0 - Alpha: first migrations

- Migrate page-db, Hive DB distro, Hive DB hive-exec (MSRV 1.85 crates, small or piloted) using `define_errors!`; migrate kraken (raising it to 1.81).
- Remove the unused `thiserror` dependency from cli-forge.
- Feedback from each migration recorded; fixes land here.

**Exit:** each migrated crate's CI is green on its own repository against the published 0.8.0.

### 0.9.0 - Beta and release candidates

- Migrate noml-rust (1.82), mod-cli (1.81), config-lib (MSRV raise from 1.75 decided with James, section 19).
- L2 derive builds against `docs/GENERATED-CODE.md` and passes the shared conformance suite; L3's prototype reads the probe.
- Release candidates `0.9.x` until no API change is needed for two weeks of use.

**Exit:** API unchanged across the last release candidate; L2 and L3 owners sign off on the contract.

### 1.0.0 - Stable

- Full audit (API, docs, tests, benchmarks, security) recorded in `docs/STABILITY-1.0.md`; API, code layout, probe format, mapping tables, and kind bits frozen until 2.0.

**Exit:** section 16 complete.

### After 1.0 (P3, design leaves room)

DEF-26 (simple generics in the macro), FAC-19 (racy installer for targets without CAS), `serde` behind a feature if a caller needs it (OD-17), `Error::provide` support next to the probe if Rust stabilises it, `u128` kinds only in 2.0.

---

## 15. Release routine

Copied from the portfolio library standard (section 3.) James's instruction (2026-10-21): after each version is complete, push to GitHub, tag, release, and once CI passes, `cargo doc` and `cargo publish`. In order:

1. **Finish the version**: every item of the version's roadmap entry done; version bumped in every place it appears (`Cargo.toml`, README install snippet, `docs/API.md`, rustdoc examples, `clippy.toml` MSRV if changed); `CHANGELOG.md` rolled (`[Unreleased]` becomes `[X.Y.Z] - YYYY-MM-DD`, compare links updated); `docs/release/vX.Y.Z.md` written; README and `docs/API.md` checked against the build.
2. **Local gate** (6.13) passes on Windows and WSL, stable and MSRV.
3. **Commit and push** to GitHub (the first scaffold to `main`; later versions on a `feature/` branch with a pull request, squash-merged after CI passes).
4. **Wait for CI to pass** on every platform (`gh run list`, `gh run view --log-failed`). If it fails, fix it in place; never cut a follow-up patch release for a red version.
5. **Tag** `vX.Y.Z` and push the tag.
6. **Release**: `gh release create vX.Y.Z --title "vX.Y.Z - <Title>" --notes-file docs/release/vX.Y.Z.md`, marked pre-release for 0.x.
7. **Docs**: `cargo doc --no-deps` with default and with all features under `RUSTDOCFLAGS=-D warnings`: no warnings.
8. **Publish**: `cargo publish --dry-run`, then `cargo publish`.
9. **Verify**: the version is on crates.io and the docs.rs build succeeded.

1.0.0 only after a full audit (API, docs, tests, benchmarks, security) recorded in `docs/STABILITY-1.0.md`.

`define-err` specifics: the local gate runs clippy and tests in four feature modes (default, all, none, `alloc` only) and `cargo +1.81 clippy` (not only build); `cargo doc` also runs with `--no-default-features`, because a doc link to an `alloc`-gated item breaks only when the feature is off; docs.rs builds with `all-features = true` and shows `doc(cfg)` badges. When `__private` changes, `docs/GENERATED-CODE.md` and the changelog say so in the same release, and the L2 owner is told before publishing.

---

## 16. Definition of done

`define-err` 1.0 is done when every line below is true and recorded in `docs/STABILITY-1.0.md`:

1. Every P0, P1, and P2 item in section 8 is implemented, documented with a runnable example, and tested.
2. Every FR and NFR in section 5 has at least one named test that passes (`tests/TRACEABILITY.md`).
3. Every edge case E-1 to E-100 has a passing test.
4. Every threat T1 to T17 has a passing attack test; `docs/SECURITY.md` states each defence's cost.
5. Every parser and decoder has a fuzz target with at least 24 hours of fuzzing and no open finding.
6. loom models and Miri pass; the 8-hour soak shows flat memory and exact counts.
7. Counting-allocator tests prove every "no allocation" claim.
8. `docs/PERFORMANCE.md` published; generated code identical to thiserror's; build time and facade at least 20% faster than the named competitors, or a reviewed reason recorded.
9. Zero runtime dependencies in every feature mode (`cargo tree`); `cargo deny check` and `cargo audit` clean.
10. `unsafe` only in `src/facade/slot.rs`, three sites, each with a `SAFETY:` comment.
11. CI green on Linux, macOS, Windows, stable and MSRV, four feature modes; `no_std` cross-checks green.
12. README, `docs/API.md`, `docs/GENERATED-CODE.md`, and every document in section 13 complete and matching the build; no banned words; no U+2014; no emoji; no `.dev/` reference in public files.
13. L2 derive passes the shared conformance suite against the 1.0 contract; L3 reads codes through the probe.
14. At least page-db, Hive DB distro, hive-exec, kraken, noml-rust, and mod-cli run on the release candidate with green CI; config-lib's path is agreed with James.
15. The release routine (section 15) completed for 1.0.0; docs.rs build green.

---

## 17. Open decisions

Each decision has a recommendation. The recommendation stands unless James overrides it; every outcome goes into the decision log (section 20). At the `0.2.0` design lock every open decision was answered: OD-1 by James, OD-2, OD-3, and OD-19 at 0.1 (D-002 to D-004), the rest accepted as recommended (D-018). No decision is waiting.

| Id | Decision | Options | Recommendation and reason |
|---|---|---|---|
| OD-1 | Crate name | (decided) | **Decided by James, 2026-10-22: `define-err`** (D-001). Checked free on crates.io and GitHub on 2026-10-22 before the repository was created. |
| OD-2 | MSRV and edition | (a) 1.81 + edition 2021; (b) 1.85 + edition 2024 (portfolio default); (c) lower MSRV with the `Error` impl gated on `std` | **Decided (D-002): (a) 1.81 / 2021.** 1.81 is the first release where `core::error::Error` is stable (Rust blog, 2024-09-05), which `no_std` errors need. (c) is impossible for generated code: a `cfg(feature = "std")` in a macro expansion tests the user's features, not this crate's (7.5.6). Between (a) and (b): this crate caps the MSRV of every crate that depends on it, and kraken (1.80), mod-cli (1.81), noml-rust (1.82), and config-lib (1.75) are below 1.85; with (a) only kraken (+1) and config-lib (+6) must move. Inline `const` (1.79) and const `from_utf8` are within 1.81. Edition 2021 costs nothing here (no 2024-only feature is needed; `macro_rules!` fragments follow the defining crate's edition). |
| OD-3 | The facade's `unsafe` | (a) one small `unsafe` slot module, the `log` crate's proven pattern, Miri and loom tested; (b) `forbid(unsafe_code)` with the facade available only with `std` (`OnceLock`); (c) no facade in L1 | **Decided (D-003): (a).** R1 requires `no_std`; stable Rust has no safe `no_std` once-cell that is `Sync`. (b) would make `no_std` libraries unable to report, which is the facade's purpose. Three `unsafe` sites in one file, under 40 lines, with a state machine identical in shape to `log`'s, are auditable in minutes. |
| OD-4 | Code layout | (a) 7.6: 32-bit reversible namespace text, 3-bit width, 5 reserved, 24-bit number; (b) the error-suite plan, 4.1: 16-bit registered namespace id, 32-bit number, 16 reserved | **Accepted as recommended (D-018).** **(a).** A code must display and compare without a registry, in `no_std`, and identically on every node; registered ids need run-time registration that libraries cannot do and nodes must agree on. Costs: six-character namespaces (every namespace in use today fits: DB, DIS, EXE, CV, CFG, AUTH, GATEWAY needs to become GATEWY or GW) and 16.7 million numbers per namespace. |
| OD-5 | Identity probe through `Error::description` | (a) probe (7.11); (b) no probe: metadata only through typed `ErrorInfo`, so L3 cannot see codes of wrapped errors; (c) wait for `Error::provide` | **Accepted as recommended (D-018).** **(a).** It is the only stable mechanism that works through `dyn Error` chains and foreign wrappers; it costs nothing at run time until read. Its one visible effect (the deprecated `description()` returns the probe) affects no current caller. (c) is nightly-only and excluded by the error collection plan, 10.4; add it next to the probe if it stabilises. |
| OD-6 | Split codes, levels, catalog, categories, and kinds into their own library later (the error-forge successor) | (a) keep in L1 for 1.0 with internal module boundaries enforced; (b) split now | **Accepted as recommended (D-018).** **(a).** Every user of codes also defines errors, and splitting a less-than-5k-line crate adds a second version to coordinate for every library. The module-boundary CI check (7.1) keeps a split mechanical. Split if any of: source passes 6,000 lines (NFR-14); a consumer needs codes without the macro or facade (for example a protocol crate parsing codes off the wire); codes need a major version independent of definitions. If split, the codes crate is the natural successor to error-forge's code features, and `error-forge` itself gets the deprecation notice of the error-suite plan, E-3 either way. |
| OD-7 | Help text required with every code | yes (the error-suite plan, E-7); no | **Accepted as recommended (D-018).** **Yes.** A code without an explanation is the failure rustc's `--explain` exists to prevent; the cost is one string per code, written once. |
| OD-8 | Severity label values | `NOTICE` 10, `WARNING` 13, `ERROR` 17, `CRITICAL` 20, `FATAL` 21 (7.7); or the error-suite plan, 4.2's example (notice 10, warning 14, error 18, critical 22, fatal 24) | **Accepted as recommended (D-018).** **7.7.** Base values of the OpenTelemetry bands for WARNING, ERROR, FATAL (as R20's defaults 13, 17, 21 already assume); `CRITICAL` at the top of the ERROR band so it is "worse than error, not yet fatal". ERROR-SUITE's numbers stay valid as one project's own `LevelNames`. |
| OD-9 | HTTP status of `FailedPrecondition` | 400 (google.rpc.Code); 409 or 422 (the error-suite plan, 4.3) | **Accepted as recommended (D-018).** **400**, matching Google's published mapping so gRPC transcoders and this crate agree. A project wanting 409 overrides in L4. |
| OD-10 | Exit code table | 7.8 (sysexits, 130 for cancelled); all failures exit 1 | **Accepted as recommended (D-018).** **7.8.** Scripts can tell usage errors (64) from temporary failures (75) and retry only the latter. `Unknown` and `Internal` map to 70 (`EX_SOFTWARE`); L3's `main` integration may still choose 1 by policy for tools that promise only zero and non-zero. The least certain rows are `FailedPrecondition` (78 `EX_CONFIG` assumes the usual CLI cause is configuration) and `DataLoss` (74 `EX_IOERR`); James may prefer 65 for both. |
| OD-11 | Tuple fields in messages | named bindings (7.4.2); positional `{0}` via a different mechanism | **Accepted as recommended (D-018).** **Named bindings.** `macro_rules!` cannot support `{0}`; positional arguments passed blindly fail to compile when unused. L2 keeps `{0}` for migrations. |
| OD-12 | Message placement in `define_errors!` | `=> "..."` after the variant; `#[error("...")]` attribute (same as L2) | **Accepted as recommended (D-018).** **`=>`**: the message sits where a reader looks, and one placement per front end avoids two ways of writing the same thing in one crate. L2 keeps `#[error]`. |
| OD-13 | Automatic `Debug` | derive by default with `#[manual_debug]` opt-out; never derive (user writes it, as with thiserror) | **Accepted as recommended (D-018).** **Derive by default.** Every error needs `Debug` (supertrait of `Error`), and `Secret<T>` covers the redaction case that motivates a manual impl. |
| OD-14 | `#[non_exhaustive]` default and L2 | macro adds it with `#[exhaustive]` opt-out; L2 cannot add attributes | **Accepted as recommended (D-018).** **As specified (7.5.7).** L2 leaves user attributes alone to keep thiserror migrations a find-and-replace (EC-4). |
| OD-15 | `warn!` clashing with `log::warn!` and `tracing::warn!` | keep `warn!` (James's verb); rename (`warning!`) | **Accepted as recommended (D-018).** **Keep `warn!` and `warn()`.** The names are James's (the error collection plan, 11.2). A clash happens only when both crates' macros are glob-imported in one module; the docs show the path form `define_err::warn!`. |
| OD-16 | `Secret<T>` in L1, and wiping | in L1 without wiping; in L1 with `unsafe` volatile wiping; in crypt-io only | **Accepted as recommended (D-018).** **In L1, redaction only** (EC-5). Wiping without `unsafe` cannot be guaranteed and is crypt-io's job; `docs/SECURITY.md` says so. |
| OD-17 | `serde` | none in 1.0; optional feature | **Accepted as recommended (D-018).** **None in 1.0** (standard 2.8: adapters only when needed). L4 writes JSON itself; add a non-default feature after 1.0 only if a caller needs it. |
| OD-18 | `retry_after` type | `core::time::Duration`; seconds as `u32` | **Accepted as recommended (D-018).** **`Duration`**: in `core`, unambiguous, maps to HTTP `Retry-After` seconds in L4. |
| OD-19 | Visibility and folder | public, `error-collection`; private | **Decided (D-004): public, `error-collection`.** Nothing proprietary; every public portfolio crate depends on it. |
| OD-20 | Kinds width | `u64` (32 + 32); `u128` | **Accepted as recommended (D-018).** **`u64`** (the error collection plan, 11.3). Widening is a 2.0 change; 13 platform bits are used, 19 remain. |
| OD-21 | Platform kind bits | 7.9's thirteen; fewer | **Accepted as recommended (D-018).** **7.9.** Each bit has a producer in the collection (R9's warning categories, R10, `trigger`, L3/L7 panics, audit). |
| OD-22 | Once-per-site table size for the function forms | 512 slots (4 KiB); configurable by feature; none (functions always deliver) | **Accepted as recommended (D-018).** **512 fixed.** Feature-configured sizes multiply the test matrix; overflow degrades to "always deliver" and is counted, never silent. Revisit if a real program exceeds 512 distinct `warn()` call sites. |

---

## 18. References

**Specifications and standards**

- OpenTelemetry Logs Data Model, SeverityNumber and short names: https://opentelemetry.io/docs/specs/otel/logs/data-model/
- `google.rpc.Code` with the HTTP mapping per code: https://github.com/googleapis/googleapis/blob/master/google/rpc/code.proto
- Google AIP-194, automatic retry configuration: https://google.aip.dev/194
- Google AIP-193, errors: https://google.aip.dev/193
- RFC 9457, Problem Details for HTTP APIs: https://www.rfc-editor.org/rfc/rfc9457
- `sysexits(3)`: https://man.freebsd.org/cgi/man.cgi?query=sysexits
- gRPC status codes: https://grpc.io/docs/guides/status-codes/

**Rust language and standard library**

- Rust 1.81.0 release (stable `core::error::Error`): https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/
- Rust 1.79.0 release (inline `const` blocks): https://blog.rust-lang.org/2024/06/13/Rust-1.79.0/
- Rust 1.57.0 release (`panic!` in const contexts, message limits): https://blog.rust-lang.org/2021/12/02/Rust-1.57.0/
- `#[track_caller]`: https://doc.rust-lang.org/std/attribute.track_caller.html
- `core::panic::Location`: https://doc.rust-lang.org/core/panic/struct.Location.html
- `core::error::Error` (including the deprecated `description`): https://doc.rust-lang.org/core/error/trait.Error.html
- `#[track_caller]` on a `From` impl reports the `?` site: https://www.svix.com/blog/try-operator-error-locations-rust-tests/
- Implementation PR for caller location: https://github.com/rust-lang/rust/pull/65258

**Libraries studied**

- thiserror 2.0.21 documentation (attributes, `{var}`/`{0}`/`.field` interpolation, generated impls): https://docs.rs/thiserror/latest/thiserror/
- thiserror repository and README ("deliberately does not appear in your public API"): https://github.com/dtolnay/thiserror
- displaydoc 0.2.7 (doc comments as `Display`, `no_std`, multi-line rules): https://docs.rs/displaydoc/latest/displaydoc/
- log, `set_logger`: https://docs.rs/log/latest/log/fn.set_logger.html
- log source (slot, `STATE` machine, `compare_exchange`, acquire load): https://docs.rs/log/latest/src/log/lib.rs.html
- snafu: https://docs.rs/snafu/latest/snafu/
- miette: https://docs.rs/miette/latest/miette/
- error-forge 1.0.2: https://github.com/jamesgober/error-forge

**Related plans**

- The portfolio library standard (the common standard this plan follows)
- The error collection plan (the split into libraries L1 to L7 and the suite; research additions R1 to R26; the reporting verbs and kind masks)
- The earlier error-suite plan (codes, severity, categories, metadata, threats, performance)
- The report-err plan (L3), whose cross-crate requests X-1 to X-12 are answered at the 0.2 design lock (section 20)

---

## 19. Migration of existing users

### 19.1 Inventory (measured 2026-10-22 by searching each repository)

| Project | thiserror | MSRV / edition | `#[error]` attributes | `#[from]` | `#[source]` | positional `{0}` | `.field` args | Notes |
|---|---|---|---|---|---|---|---|---|
| config-lib 1.1.0 | 1.0 | 1.75 / 2021 | 132 | 11 | 11 | 0 | 0 | enum named `Error`, already `#[non_exhaustive]` |
| page-db 1.0.0 | 2 | 1.85 / 2024 | 55 | 6 | 0 | 6 | 0 | `PageError` already `#[non_exhaustive]`; extra named args (`min = crate::MIN_PAGE_SIZE`); hex format specs |
| noml-rust 0.10.0 | 2.0 | 1.82 / 2021 | 44 | 0 | 4 | 0 | 0 | `NomlError` already `#[non_exhaustive]` |
| kraken 0.9.3 (+ kraken-lsp) | 1.0 | 1.80 / 2021 | 10 | 1 | 0 | 4 | 1 | `"{}", .diagnostic.format()` |
| mod-cli 0.6.4 | 1.0.56 | 1.81 / 2021 | 8 | 2 | 0 | 7 | 0 | `ModCliError` not `#[non_exhaustive]` |
| Hive DB distro | 2 | 1.85 / 2024 | 2 | 0 | 1 | 0 | 0 | codes inside messages: `"DIS-00001: configuration error [{field}]..."` |
| Hive DB hive-exec (+ colvec) | 2 | 1.85 / 2024 | 3 | 1 | 0 | 0 | 0 | codes inside messages: `EXE-00001`, `EXE-00002`, `CV-00001` |
| cli-forge 0.1.0 | 1.0.56 | none / 2021 | 0 | 0 | 0 | 0 | 0 | dependency declared, never used: remove it |

(Counts are lines matching the attribute in `.rs` files outside `target/`; they guide effort, they are not an audit.)

### 19.2 Two routes

1. **L2 derive (near find-and-replace).** Replace `thiserror = "..."` with the L2 crate, and `use thiserror::Error` / `#[derive(thiserror::Error)]` with L2's path. `#[error("...")]`, `#[from]`, `#[source]`, `transparent`, `{0}`, and `.field` arguments keep working (EC-4). Then add metadata attributes where wanted. Best for config-lib (132 attributes) and any crate that wants the smallest diff.
2. **`define_errors!` (no proc macro).** Rewrite each enum into the 7.4 form. Best for small error sets, `no_std` crates, and crates that want zero build dependencies (page-db, Hive DB, kraken, mod-cli, noml-rust). Mechanical rules below.

Both routes produce the same code (7.5), so a crate may move from route 1 to route 2 later without any change visible to its users, except the three differences in 7.5.7.

### 19.3 Conversion rules for route 2

| thiserror | `define_errors!` |
|---|---|
| `#[derive(Debug, Error)] pub enum E { ... }` | `define_errors! { pub enum E { ... } }` (drop the derives; `Debug` is added; keep other derives such as `Clone`) |
| `#[error("page {page_id} not found")] NotFound { page_id: u64 },` | `NotFound { page_id: u64 } => "page {page_id} not found",` |
| `#[error("page i/o error: {0}")] Io(#[from] io::Error),` | `Io(#[from] source: io::Error) => "page i/o error: {source}",` |
| `#[error("invalid usage: {0}")] InvalidUsage(String),` | `InvalidUsage(detail: String) => "invalid usage: {detail}",` |
| `#[error("{}", .diagnostic.format())] Diag { diagnostic: D },` | `Diag { diagnostic: D } => ("{}", diagnostic.format()),` |
| `#[error("size {size} not in {min}..={max}", min = crate::MIN, max = crate::MAX)]` | `=> ("size {size} not in {min}..={max}", min = crate::MIN, max = crate::MAX)` |
| `#[error(transparent)] Other(#[from] OtherError),` | `Other(#[from] inner: OtherError) => transparent,` |
| `#[source] cause: E` or a field named `source` | unchanged |
| `#[non_exhaustive]` on the enum | keep it or delete it: the macro adds it when absent and never adds a second copy |
| enum intentionally exhaustive and published | add `#[exhaustive]` so no breaking change is introduced |
| code inside the message: `#[error("DIS-00001: configuration error [{field}]")]` | `namespace = "DIS", width = 5;` + `#[code(1, help = "...")]` + message without the prefix |

### 19.4 Per project

**page-db (1.85, 55 attributes).** Route 2 at the 0.3 pilot and for real at 0.8. Six `{0}` become named bindings. `PageError` keeps its `#[non_exhaustive]` (default). Extra named arguments use the parenthesised form. Add `namespace = "PAGE"` and codes for the persistent failures (checksum mismatch: `DataLoss`; buffer pool exhausted: `ResourceExhausted`; short read: `Unavailable`) once 0.4 lands; message text unchanged, so log searches keep working. MSRV unchanged.

**Hive DB distro and hive-exec (1.85, 5 attributes).** Route 2. The codes already in the messages (`DIS-00001`, `DIS-00003`, `EXE-00001`, `EXE-00002`, `CV-00001`) move into `#[code]` with `width = 5`, so `err.code().to_string()` produces exactly the old prefix. **Behaviour change to agree on:** `Display` no longer starts with the code. Logs that need the prefix get it from L4 (`[DIS-00001] configuration error ...`) or from L7's record; until L4 exists, keeping the prefix inside the message text is allowed (the code is then in both places) and is removed when L4 lands. Categories: configuration errors `InvalidArgument` with `#[action(FixConfig)]`; I/O errors `Unavailable`; data errors `InvalidArgument` or `DataLoss` by meaning. Hive DB is enterprise-grade (portfolio rule), so every code gets real help text and a docs URL template from day one.

**kraken (1.80 to 1.81, 10 attributes).** Route 2. MSRV raise of one minor version, before kraken's own 1.0, recorded in its changelog. Four `{0}` become named bindings; the `.diagnostic.format()` argument becomes the parenthesised form. kraken-lsp follows the same steps. Diagnostics with source spans move to L5 later, not as part of this migration.

**mod-cli (1.81, 8 attributes).** Route 2. Seven `{0}` become named bindings (`InvalidUsage(detail: String) => "invalid usage: {detail}"`). `ModCliError` gains `#[non_exhaustive]` by default: a breaking change, acceptable in its 0.x series and noted in its changelog; add `#[exhaustive]` instead if James wants it to stay exhaustive. Categories map directly to exit codes (`InvalidUsage` to `InvalidArgument` gives exit 64), which replaces any hand-written exit-code table in mod-cli.

**noml-rust (1.82, 44 attributes).** Route 2 (or route 1 if the diff matters more than the build). Four `#[source]` unchanged. `NomlError` keeps `#[non_exhaustive]`. Parse errors with line and column are candidates for L5 later.

**config-lib (1.75, 132 attributes, published 1.1.0).** Route 1 (L2), because of the size, once L2 reaches its own 0.9. **MSRV decision needed from James:** `define-err` needs 1.81 (with OD-2 (a); 1.85 with (b)), so config-lib's MSRV rises from 1.75. Under config-lib's own stability policy an MSRV raise after 1.0 is a major change (standard 2.14), so the options are: (a) config-lib 2.0 with the migration and any other planned breaking changes; (b) stay on thiserror in 1.x and migrate in 2.0; (c) treat it as minor if config-lib's `docs/STABILITY` allows. Recommendation: (b), batching the migration with config-lib's next planned major release, because the migration alone gives its users no new behaviour.

**cli-forge (0.1.0).** `thiserror` is declared but never used: remove the dependency. No migration.

### 19.5 error-forge users (for completeness)

lsm-db, txn-db, wal-db, better-bucket, rate-net, and throttle-net use error-forge's `ForgeError` metadata instead of thiserror. They move when their next major version is planned: `ForgeError::kind()`/`caption()` become `Category` and `CodeInfo.title`; `is_retryable()` stays (now from the category); `status_code()` and `exit_code()` come from the category; string codes become `#[code(n)]` under a namespace. error-forge itself receives a final release whose README points at `define-err` (the error-suite plan, E-3), with no API removal in 1.x.

### 19.6 Order

page-db (pilot at 0.3, final at 0.8) → Hive DB distro and hive-exec (0.8) → kraken (0.8) → mod-cli and noml-rust (0.9) → cli-forge cleanup (any time) → config-lib (per 19.4 decision) → error-forge users (their next majors).

---

## 20. Decision log

Every design decision, in order. A decision that changes a section above updates that section in the same change.

| Id | Date | Decision | Reason |
|---|---|---|---|
| D-001 | 2026-10-22 | Name `define-err` (import path `define_err`), repository `jamesgober/define-err`, folder `error-collection/define-err`. Answers OD-1. | James's choice. Free on crates.io (no `define-err` or `define_err`) and on GitHub when checked on 2026-10-22, immediately before the repository was created; published at 0.1.0 to claim it. |
| D-002 | 2026-10-23 | MSRV 1.81, edition 2021. Answers OD-2 as recommended. | 1.81 is the first release with `core::error::Error`, which generated code must implement whatever this crate's features are (7.5.6); this crate caps the MSRV of every crate that defines errors, and kraken (1.80), mod-cli (1.81), noml-rust (1.82), and config-lib (1.75) sit below the portfolio default 1.85. Inline `const` (1.79) is within 1.81. Edition 2021 costs nothing here. The portfolio standard's 1.85/2024 default is adapted for this reason (6.13). |
| D-003 | 2026-10-23 | `#![deny(unsafe_code)]` at the crate root from the first release, with `unsafe` allowed only in `src/facade/slot.rs` when the facade lands (0.5). Answers OD-3 as recommended. | The `no_std` facade (R1) needs a write-once `&'static dyn Reporter` slot; stable Rust has no safe `no_std` `Sync` once-cell. The `log` crate's state machine is the model. `deny` rather than `forbid` from 0.1 so the root attribute never changes later; until 0.5 there is no `unsafe` at all, which clippy and the attribute enforce. |
| D-004 | 2026-10-23 | Public repository in `error-collection`. Answers OD-19 as recommended. | Nothing proprietary; every public portfolio crate will depend on it. |
| D-005 | 2026-10-23 | 0.1.0 declares the feature graph (`std` implies `alloc`, `testing` implies `std`) and links `alloc` and `std` when they are on, without public items behind them yet; each mode has a test that checks the graph, and CI builds and tests all four modes. | The roadmap asks for each feature to be exercised from 0.1; a public stub item would be a placeholder in shipped code (6.1). The features gain their items in 0.4 (`std` io mappings), 0.5 (`alloc` record and boxed reporter, `testing` capture), which 7.2 already lists. |
| D-006 | 2026-10-23 | REPS exceptions, approved: (1) the REPS clause "use `thiserror` for libraries" does not apply, because this crate is the portfolio's replacement for `thiserror`; (2) the clause "MUST use tokio" does not apply (no async runtime, standard 2.8); (3) `#![deny(warnings)]` is not written in the crate root: CI denies warnings with `RUSTFLAGS=-D warnings`, so a new compiler lint cannot break users' builds of a published version. | (1) and (2) are portfolio-wide decisions recorded per crate; (3) follows the model scaffold, where the same rule is met in CI. |
| D-007 | 2026-10-23 | CI runs Linux, macOS, and Windows on x86_64 and aarch64 (six runners) on stable and 1.81, four feature modes for clippy and tests, three for `cargo doc`; a `no_std` job builds core-only and core + alloc for `thumbv7em-none-eabihf`, `thumbv6m-none-eabi`, `riscv32imac-unknown-none-elf`, and `wasm32-unknown-unknown`; Miri runs the suite with all and with no default features; `cargo package`; `cargo audit` and `cargo deny`. Loom and trybuild jobs arrive with the code they check (trybuild in 0.3, loom in 0.5). | 6.13 and NFR-11; aarch64 runners as in the newest portfolio scaffold. A loom or trybuild job with nothing to check would be a job that cannot fail. |
| D-008 | 2026-10-23 | report-err X-1 accepted: `Code` holds a `NonZeroU64`, so `Option<Code>` is 8 bytes; `Severity` holds a `NonZeroU8` (1 to 24), so `Option<Severity>` is 1 byte. `from_u64(0)` is `None`. | The packed value is never zero: the namespace field's first character is a letter, worth at least 11 x 37^5. The niche is free and saves the header word report-err asked about; `Option<Severity>` is the facade's minimum-severity setting. |
| D-009 | 2026-10-23 | report-err X-7 accepted: `ErrorRecord::new(code, severity, category, kinds)`, `push_chain(&dyn Display, Option<Code>, &str) -> bool`, `with_limits(RecordLimits)`, and `set_handled`. A record keeps the limits it was built with (private field), so every later `push_*` stays inside them. | A dynamic error must be able to build its record directly, on the cold path, in one pass: its messages rendered by its own guarded writer, plus sections, fields, and a trail that `source()` does not expose. Taking `&dyn Display` lets report-err pass its guarded renderer and lets the record's bounded writer stop a huge message early. |
| D-010 | 2026-10-23 | report-err X-8 accepted: `report_record(&ErrorRecord)` (feature `alloc`, verb `Report`), with `Event::record()` and `Event::with_record`; severity, kinds, code, and category come from the record. | The reporter sees every context layer, section, attachment, and the trail of a dynamic error, already rendered, so it never calls a user's `Display` unguarded. The verb costs one load when nothing is installed, and callers check `is_installed()` before building a record. |
| D-011 | 2026-10-23 | report-err X-9 accepted: the creation-hook slot lives here, beside the reporter slot: `CreationHook`, `CreationInfo`, `Annotate`, `set_creation_hook`, `creation_hook`. Only dynamic-error libraries call it; generated code never does (7.3 unchanged). The caller (report-err) owns the reentrancy guard and panic isolation. Item FAC-21, `0.5`. | The error collection plan places R25 "in L3 via the facade"; one place to install observers means L7 needs no dependency on report-err. The slot reuses the reporter slot's write-once mechanism inside `src/facade/slot.rs`, so the `unsafe` stays in one module (the site count there becomes the sites of one generic write-once cell, still under 40 lines). |
| D-012 | 2026-10-23 | A `transparent` variant's probe is the inner value's probe when the inner value has one (`__private::forward_probe`), else its own with flag bit 1. | A transparent wrapper is not in the `source()` chain, so without forwarding a chain walker would never see the inner code. This matches `Display`, `source()`, and `ErrorInfo`, which already forward. |
| D-013 | 2026-10-23 | The verb macros (`notice!`, `warn!`, `trigger!`, `deprecated!`) are not in the prelude; they are called by path. | One rule for readers (types and traits come from the prelude, verbs are written `define_err::...`), and a glob import of the prelude can never clash with `log::warn!` or `tracing::warn!` (OD-15). |
| D-014 | 2026-10-23 | One error type for both write-once slots: `InstallError { slot: Slot, installed_at: Option<Location> }`, replacing the specification's `SetReporterError`. | Two slots with the same rule need one error, not two; `slot` names which one was taken. |
| D-015 | 2026-10-23 | `RecordLimits` has public fields with `DEFAULT`, `new()`, and `Default`, instead of `const fn` setters. | Changing one limit reads as one assignment; `#[non_exhaustive]` still lets limits be added in a minor release. |
| D-016 | 2026-10-23 | `testing::capture` returns `Result<(R, Vec<CapturedEvent>), InstallError>` and does not run the closure when another reporter is installed, instead of returning a synthetic error event. | A synthetic event could make `assert_eq!(events.len(), 1)` pass for the wrong reason; failing closed is the only safe answer for a test helper. |
| D-017 | 2026-10-23 | `CodeParseError` gains `TooLong` (more than 15 bytes, checked first) and `BadWidth`; no error type here echoes rejected input in its `Display`. | The length check runs before any other work (standard 2.4); a hostile string must not reach logs through the parser's own error (T4, the 0.2 defence idea in 10.2). |
| D-018 | 2026-10-23 | OD-4 to OD-18 and OD-20 to OD-22 accepted as recommended. | Each is a technical call with a recorded reason in section 17; none is a product, visibility, or irreversible decision. The behaviour change for Hive DB (`Display` no longer starts with the code, 19.4) is agreed at its migration in 0.8. |
| D-019 | 2026-10-23 | report-err X-12: the items report-err names (X-1 to X-10, with the answers above) are stable from `0.2.0`: changed only with a decision-log entry here and a note to the report-err owner before release. The probe format is frozen at 1.0. | report-err builds against this API in parallel and must not chase a moving target. |
| D-020 | 2026-10-23 | Code text alphabet for the base-37 namespace: `'0'`-`'9'` are 1 to 10, `'A'`-`'Z'` are 11 to 36, 0 pads after the last character. Golden values in `tests/golden/codes.txt`. | Digits below letters keeps numeric order equal to ASCII order of the text, which 7.6 promises and E-53 tests. The specification fixed the base but not the digit values. |
| D-021 | 2026-10-23 | `CodeInfo` is built with `CodeInfo::new(code, title, help)` and `const` `with_*` methods, used by generated code and by hand-written catalogs. | The struct is `#[non_exhaustive]` (fields can be added), so it cannot be built with a literal outside this crate; `const` builders keep catalogs static data. |
| D-022 | 2026-10-23 | `ErrorRecord` trail entries are `TrailEntry { file: Cow<'static, str>, line, column, step: TrailStep }` with `TrailStep::{Created, Converted, Context, Reported}`; chain entries carry `has_control`. `Kinds` expressions in generated code compile to `Kinds::union` calls. | A record received from another node has no `'static` file names, and a borrowed `Cow` costs nothing locally. `BitOr` is not `const` on stable, while catalogs and probes need `const` kinds. |
