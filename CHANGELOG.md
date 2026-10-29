<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>CHANGELOG</b>
</h1>
<p>
  All notable changes to <code>define-err</code> will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

---

## [Unreleased]

---

## [0.4.0] - 2026-10-29

Codes and metadata.

### Added

- Codes: `Namespace` (1 to 6 characters, width 1 to 8 digits), `Code` (a
  `NonZeroU64`, so `Option<Code>` is 8 bytes) with `new`, `namespace`,
  `number`, `is_generic`, `as_u64`, `identity`, `from_u64`, `parse`,
  `Display` (`DB-0042`), and ordering and hashing by identity, the width
  ignored; `CodeParseError`, whose message never repeats the input;
  `namespace!` and `code!` for constants checked at compile time.
- `Severity` (OpenTelemetry 1 to 24, a `NonZeroU8`) with the named levels,
  `at::<N>`, `new`, `saturating`, `band`, `otel_name`, `label`, `parse`;
  `Band`; `LevelNames`.
- `Category` (the 16 gRPC canonical codes without `OK`) with `http_status`,
  `grpc_code`, `from_grpc`, `exit_code`, `is_transient`, `retry_advice`,
  `may_retry_automatically`, default severity, action, and public message,
  `name`, `parse`, and `from_io_kind`; `RetryAdvice`; `Action`.
- `Kinds`: a 64-bit mask with 14 platform bits, 32 project bits
  (`project::<N>`, `try_project`), set operations, `Display`, and an iterator.
- `ErrorInfo` complete: `code`, `code_info`, `category`, `severity`,
  `kinds`, `public_message`, `help`, `action`, `is_retryable`,
  `retry_after`, with defaults; implemented for `std::io::Error` (with
  `std`).
- The catalog: `CodeInfo` (const `new` and `with_*`), `CodeState`
  (`Active`, `Deprecated` with its replacement, `Retired`), `CodeInfo::url`
  with `UrlDisplay`, `catalog::find`, `catalog::check_unique` and
  `DuplicateCode`.
- The identity probe: `identify`, `innermost_code`, and `Identity` read the
  code, severity, category, kinds, and type name of a generated error held
  as `&dyn Error`, with strict validation of every field.
- `define_errors!`: the block header `namespace = "DB"` (with `width = N`),
  `docs = "..."`, and `retired = [...]`, in any order; `#[code(n, name =
  NAME, help = "...", title = "...", deprecated = "since", replacement =
  n)]`; `#[category]`, `#[severity]`, `#[kinds]`, `#[public]`, `#[action]`,
  `#[retryable]`, `#[retry_after(secs = n)]`, and the `#[retry_after]` field
  marker; generated code constants and `Type::CATALOG`; every `ErrorInfo`
  method; the probe in `description()`. Transparent variants forward
  metadata to the inner value, and fall back to their own when it has none.
- Compile-time checks, each reported once: namespace and width, number
  range, duplicates (retired numbers included, with rustc marking the second
  literal), replacements, the docs placeholder, severity range, project bit
  range, attribute order, repeated attributes, metadata on an enum, and type
  names over 128 bytes. 48 compile-fail tests.
- `prelude`: adds `Action`, `Category`, `Code`, `Kinds`, `Severity`.
- Tests: golden files for code packing, probe strings, category tables,
  severity names, and kind bits; property tests (proptest) for codes,
  severity, kinds, and probes; edge cases E-33 to E-64 and E-87 to E-89;
  the allocation check extended to codes and probes.
- Fuzz targets `code_parse`, `code_from_u64`, `severity_parse`,
  `category_parse`, and `probe_decode` (`fuzz/`), built in CI.

### Generated code

- New `__private` items: `namespace`, `namespace_or_placeholder`, `code`,
  `code_or_placeholder`, `retired_code`, `severity`, `check_block`,
  `check_docs`, `catalog_len`, `catalog`, `ProbeBuf`, `forward_probe`,
  `description_of!`, `retry_after_of!`, the `DescribeVia*` and `RetryVia*`
  dispatch traits, and the `platform_kinds` and `named_levels` modules.
  Every type now implements every `ErrorInfo` method and `description()`.
  `docs/GENERATED-CODE.md` describes each.

### Changed

- The probe carries the code's full value, width included, so an
  identified code displays as it was defined (`DIS-00001`);
  `tests/golden/probe.txt` updated before any release carried probes.
- `docs/API.md`: the `#[code]` argument order, the deprecated note, the
  `io::ErrorKind` mapping, the transparent fallback, and when a numeric
  severity or project bit is checked.
- `dev/ROADMAP.md`: a pre-1.0 polish pass in 0.9.x that revisits every
  accepted gap, deferred item, and missed target, retunes performance, and
  repeats the security and edge-case review; anything still open goes into
  the 1.0 readiness note.

---

## [0.3.0] - 2026-10-29

The thiserror-parity core.

### Added

- `define_errors!`: enums and structs (braced, tuple, unit) with `Display`
  from `=> "message"` (fields by name, any format spec, `{{` and `}}`),
  extra format arguments `=> ("...", expr, name = expr)`, and
  `=> transparent`; `core::error::Error` with `source()` from `#[source]`,
  `#[from]`, or a field named `source`, including `Option<E>`, boxed
  `dyn Error` (three forms, with `alloc`), and `&'static dyn Error`;
  `From` conversions from `#[from]`, marked `#[track_caller]`, filling a
  `#[location]` field with the line of the `?`; `#[derive(Debug)]` unless
  `#[manual_debug]`; `#[non_exhaustive]` on enums unless `#[exhaustive]`;
  doc comments and lint attributes passed through; several types per block;
  tuple-field binding names (`ShortRead(page: u64, got: usize)`) and unnamed
  tuple fields.
- `ErrorInfo` with `location()` and `type_name()` (`"Type::Variant"`),
  implemented by every generated type; `transparent` variants forward both.
- `Location` and `here()`.
- `Secret<T>`: prints `[redacted]` under `Display` and `Debug` with any
  flags (width, fill, and alignment honoured; precision ignored), with
  `new`, `expose`, `into_inner`, `From<T>`, and no `Deref` or `PartialEq`.
- `prelude`: `define_errors!`, `ErrorInfo`, `Location`, `Secret`.
- Compile errors that name the problem and the fix for every misuse
  (unknown attributes, two sources, two locations, two markers on a field,
  `#[from]` with extra fields, `transparent` without exactly one field, a
  missing message, a bad message form, `#[exhaustive]` on a struct, a source
  field that is not an error), with 23 compile-fail tests.
- Tests: a conformance suite against thiserror (byte-identical `Display` and
  `source()` chains), a 500-variant enum at the default `recursion_limit`,
  generated code without the prelude and next to types named `Error`,
  `Result`, `Option`, `Box`, `Display`, `Location`, `Code`, size and auto-trait
  checks against hand-written and thiserror enums, and a counting allocator
  proving construction, `Display`, `source()`, `ErrorInfo`, `From`, and
  `Secret` formatting allocate nothing.
- A criterion benchmark of construction and `Display` against thiserror and
  hand-written code.

### Changed

- `docs/GENERATED-CODE.md` describes the expansion as built: `source()` and
  `location()` are always generated, arms bind fields with `ref` on
  `match *self`, impls carry `#[allow(deprecated)]`, and the `__private`
  dispatch items are listed.
- `docs/API.md`: the canonical form and the recursion note, the attributes
  passed through, and which fields must be `Secret`.
- CI: dependencies for the 1.81 jobs are resolved against `rust-version`, and
  the compile-fail tests run in their own job pinned to Rust 1.99.

---

## [0.2.0] - 2026-10-29

The design lock, approved by James. No functional code yet.

### Added

- `docs/API.md`: the complete public API of 1.0 with signatures, behaviour,
  errors, costs, examples, and the version each item arrives in:
  `define_errors!` (grammar, messages, sources, `transparent`, structs,
  codes, metadata attributes, compile errors, limits, the thiserror
  mapping), `ErrorInfo`, `Location` and `here`, `Namespace`, `Code`,
  `CodeParseError`, `Severity`, `Band`, `LevelNames`, `Category`,
  `RetryAdvice`, `Action`, `Kinds`, the catalog (`CodeInfo`, `CodeState`,
  `UrlDisplay`, `catalog::find`, `catalog::check_unique`), the identity
  probe (`identify`, `innermost_code`, `Identity`), `Secret<T>`, the
  reporting facade (`Reporter`, `set_reporter`, `report`, `report_with`,
  `report_record`, `notice`, `warn`, `trigger`, `deprecated`, `soft`,
  filters, `Event`, `stats`, the creation-hook slot), `ErrorRecord`, and
  `testing::capture`.
- `docs/GENERATED-CODE.md`: what `define_errors!` expands to, form by form,
  the identity probe format, the compile-time checks, and the hidden
  `__private` surface with its versioning rule, for the attribute front end.
- `tests/golden/`: code packing, probe strings, category tables, severity
  names, and kind bits, written before the code that produces them.

### Changed

- `dev/PLAN.md`: open decisions OD-4 to OD-22 answered; decisions D-008 to
  D-022, including report-err's requests (a non-zero niche in `Code`,
  `ErrorRecord::new` and `push_chain`, `report_record`, and the creation-hook
  slot, all accepted); one error type `InstallError` for both write-once
  slots; verb macros called by path, not in the prelude; transparent
  variants forward the identity probe; a new defence idea.
- `dev/ROADMAP.md`: 0.1.0 and 0.2.0 marked done; 0.5.0 lists the new items
  FAC-20, FAC-21, and REC-07.
- README, crate documentation, and install snippets for 0.2.0.

---

## [0.1.0] - 2026-10-23

The scaffold and the plan. No functional code yet; this release claims the name.

### Added

- Crate manifest: edition 2021, MSRV 1.81, no dependencies, features `std`
  (default, implies `alloc`), `alloc`, and `testing` (implies `std`),
  `Apache-2.0 OR MIT` license, README, REPS, `deny.toml`, `clippy.toml`,
  `rustfmt.toml`, `.editorconfig`, `.gitattributes`.
- `#![no_std]` crate root with `alloc` and `std` linked by their features,
  the `VERSION` constant, and the lint set: `unsafe_code` and `missing_docs`
  denied, `unwrap`/`expect`/`panic` and printing denied in library code.
- CI on Linux, macOS, and Windows, each on x86_64 and aarch64, on stable and
  1.81, with clippy and tests in four feature modes (default, all, none,
  `alloc` only); documentation built in three modes; `no_std` builds for
  `thumbv7em-none-eabihf`, `thumbv6m-none-eabi`,
  `riscv32imac-unknown-none-elf`, and `wasm32-unknown-unknown`; Miri;
  `cargo package`; `cargo audit` and `cargo deny`.
- `dev/PLAN.md` (purpose, position in the error collection, scope,
  requirements, the `define_errors!` grammar and the generated-code
  contract, codes, severity, categories, kinds, the identity probe, the
  reporting facade, `ErrorRecord`, the API by tier and grade, threats T1 to
  T17, performance targets, the testing plan with edge cases E-1 to E-100,
  the migration plan for existing thiserror users, open decisions, and the
  decision log D-001 to D-007), `dev/DIRECTIVES.md`, and `dev/ROADMAP.md`.
- `docs/API.md` with what 0.1.0 exposes.

[Unreleased]: https://github.com/jamesgober/define-err/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/jamesgober/define-err/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/jamesgober/define-err/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jamesgober/define-err/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/define-err/releases/tag/v0.1.0
