# define-err - Engineering Directives

> Engineering standards and the definition of done for this project. Read alongside `REPS.md` (root, authoritative), [`PLAN.md`](./PLAN.md) (design and decisions), and [`ROADMAP.md`](./ROADMAP.md) (current phase). If anything here conflicts with `REPS.md`, `REPS.md` wins, except where `PLAN.md` records an approved exception (D-006: this crate replaces `thiserror`; no async runtime; warnings denied in CI rather than in the crate root).

---

## 0. Philosophy

This crate is built and maintained to a production standard and treated as a flagship piece of work. Plan the full path, then build one verified step at a time. "Good enough" is treated as a defect. Every library in the portfolio will declare its errors with it, so its generated code must be exactly what a careful person writes by hand, its tables must never change under a caller, and its reporting facade must cost nothing when nobody listens.

---

## 1. What this is

`define-err` is how a Rust library says what its errors are: the `define_errors!` macro (the `thiserror` replacement, no proc macro), plus optional codes, severity, categories, kinds, a code catalog, `Secret<T>`, an owned `ErrorRecord`, and the reporting facade (`set_reporter`, `report`, `notice`, `warn`, `trigger`, `deprecated`, `soft`). See `PLAN.md` sections 2 to 4.

---

## 2. Engineering law (non-negotiable)

- **Zero dependencies.** No runtime dependency of any kind in any feature combination, first-party included (NFR-1). Dev-dependencies (`trybuild`, `proptest`, `criterion`, `loom`, and the competitors for benchmarks) are allowed. No async runtime.
- **`no_std` first.** `core` always; `alloc` and `std` only behind their features. `cfg` uses only `feature = "..."` and `target_has_atomic = "..."`.
- **Generated code has no feature `cfg`.** A `cfg(feature)` in a macro expansion tests the user's features, not this crate's (`PLAN.md` 7.5.6). Every path in generated code is absolute (`::core::`, `$crate::`).
- **Construction has no side effects.** Building an error runs no code: no hook, no allocation, no atomic. Reporting is explicit.
- **No run-time panic.** `panic!` may only be reachable in constant evaluation, where it is a compile error (`PLAN.md` 7.13). Every such site is listed in `docs/ARCHITECTURE.md`. No `unwrap`, `expect`, `todo!`, `unimplemented!`, `unreachable!`, `dbg!`, or printing in library code.
- **Unsafe.** `#![deny(unsafe_code)]` at the root. The only allowed module is `src/facade/slot.rs` (D-003): three sites, each with a `// SAFETY:` comment, Miri and loom tested.
- **Fail closed.** A code, namespace, severity, or kind that cannot be validated does not compile; at run time, parsers accept only the exact grammar. A public message that was not written falls back to the category's generic text, never to the private `Display`.
- **Frozen tables.** The packed code layout, the probe format, the category mappings, and the kind bits are golden-tested from 0.2 and frozen at 1.0.
- **Never reuse a code.** Retired numbers stay in the catalog and in the duplicate check.
- **Secrets.** No secret in `Display` or `Debug` of any type here; `Secret<T>` has no `Deref`, `PartialEq`, `AsRef`, or `Borrow`.
- **Cross-platform.** Linux, macOS, and Windows on x86_64 and aarch64, stable and MSRV 1.81, in CI; `no_std` cross-checks for the targets in `PLAN.md` 7.17.
- **Error handling.** Every fallible function returns `Result` or `Option`; this crate's own error types are `#[non_exhaustive]` and their `Display` says what to do.
- **Production-ready.** `#![deny(missing_docs)]`; every public item has rustdoc with a runnable example; every claim in the docs maps to a test.
- **Performance.** No claim without criterion numbers; the targets in `PLAN.md` section 11 are measured and published in 0.6.

---

## 3. Definition of done

1. Compiles clean on Linux, macOS, and Windows (x86_64 and aarch64), stable and MSRV 1.81, and for every `no_std` target in `PLAN.md` 7.17.
2. `fmt`; `clippy -D warnings` (all targets) and tests in four feature modes (default, all, none, `alloc` only); `cargo doc -D warnings` with default, all, and no default features; `cargo package` in a fresh target directory: clean on Windows and in WSL Ubuntu, stable and 1.81 (`cargo +1.81 clippy`, not only build).
3. `cargo audit` and `cargo deny check` pass; `cargo tree` shows no runtime dependency in any mode.
4. The Tier 1 API (`define_errors!` and the verbs) headlines the README and the crate docs.
5. Every macro compile error has a `trybuild` case; every threat in `PLAN.md` section 10 has an attack test that must fail to get through; every edge case in 12.8 has a test.
6. Every "no allocation" claim has a counting-allocator test; hot-path changes carry benchmarks; no regression over 5%.
7. `README.md`, `docs/API.md`, `docs/GENERATED-CODE.md` (from 0.2), and `CHANGELOG.md` updated in the same change; the matching `docs/release/vX.Y.Z.md` written before the tag; every version string bumped.
8. `PLAN.md` decision log updated for every design decision made during the work; at least one new defence idea per version in `PLAN.md` 10.2.
9. When `__private` changes, `docs/GENERATED-CODE.md` and the changelog say so in the same release, and the L2 (derive) owner is told before publishing.
10. v1.0.0 only after the full audit (API, docs, tests, benchmarks, security) is recorded in `docs/STABILITY-1.0.md` and James approves it.

---

## 4. Project-specific invariants

- A plain definition generates exactly the items of `PLAN.md` 7.5.1 and nothing else: no constructors, no inherent methods, no statics, no `Result` alias.
- `Display` of a generated type writes exactly the user's message: no prefix, no code, no decoration.
- `source()` returns exactly the source field or `None`; `transparent` forwards.
- A type's size, alignment, `Send`, and `Sync` are exactly those of its fields.
- `Code` equality, ordering, and hashing ignore the display width.
- With no reporter installed, every verb costs one atomic load and returns; it never allocates, formats, or blocks.
- A verb never changes control flow, never panics, and never calls the reporter recursively on the same thread (with `std`).
- `ErrorRecord.public` is built only from public messages, never from any `Display` in the chain.

---

## 5. Writing style

- Plain, specific, human prose in every shipping file. No marketing voice.
- Banned in shipping files: "comprehensive", "robust", "seamless", "leverage", the em-dash character (U+2014), and emoji, including in generated `Display` text.
- Every security measure states its cost (latency, memory); estimates are labelled as estimates, measurements as measured.
- Release notes in the past tense.
- Commits: imperative mood, lowercase, concise, no trailing period (REPS).
