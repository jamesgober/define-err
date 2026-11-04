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

### Added

- `short_file` and `TrailEntry::short_file`: a location's file as a log
  should show it. A registry path, which names the home directory, is cut to
  the crate's directory (`page-db-1.0.0/src/error.rs`); a relative path is
  unchanged (threat T17).

### Fixed

- A `PathBuf`, `Path`, or `&Path` field named in a plain message shows
  through `display()`, as with thiserror (with the `std` feature, a default). It used to be a compile error
  (`PathBuf` does not implement `Display`), so a thiserror message such as
  `"file not found: {0}"` on a `PathBuf` could not be kept as written. Every
  other field formats exactly as before.
- `#[cfg(...)]` on a whole type compiles out its impls with it. They used to
  stay, and fail to compile once the type was gone.
- A `#[code]` without `help`, or with its fields out of order, reports that
  one error. Two of them in a block also reported a false duplicate code.

### Changed

- Examples, docs, and test data use neutral names (`ServiceError`,
  namespaces `SVC`, `JOB`, and `IO`, `docs.example.com`). No behaviour or
  format changes.

### Generated code

- A plain message's `Display` arm rebinds each field through
  `__private::shown_of!` first. New `__private` items: `shown_of!`,
  `ShownViaPath`, `ShownViaAny`, `check_number`, `INVALID_NUMBER`.
  `check_block` skips `INVALID_NUMBER`. The impls of a type with `#[cfg]`
  sit in one `const _` block carrying those `cfg` attributes.
  `docs/GENERATED-CODE.md` has the details.

---

## [0.7.0] - 2026-11-04

Hardening. Every threat in `docs/SECURITY.md` has a passing attack test,
each of the seven fuzz targets ran 24 hours with no finding, and every verb
ran from 64 threads for 8 hours with exact counts and flat memory. No public
API changes.

### Added

- `docs/SECURITY.md`: threats T1 to T17 with each defence, its cost, and its test, and what is not defended, forged probes above all.
- `docs/ARCHITECTURE.md` and `docs/PLATFORM-NOTES.md`.
- Attack tests for every threat (`tests/attack*.rs`), a stress and soak of every verb from 64 threads (`tests/stress.rs`), and the fuzz target `kinds_display`.

### Fixed

- A panic in a reported error's own `description()` is contained like a panic in `receive`, and counted in `stats().reporter_panics`. It used to unwind into the caller of `report`, `report_with`, or `soft`. Cost: a filtered `report` takes about 0.4 ns more, a delivered report of a foreign error about 1.5 ns more; `docs/PERFORMANCE.md` has the figures.

### Changed

- The panic guard checks and sets its flag in one thread-local access.
- `benches/baseline.txt` holds 0.7.0's floors.

---

## [0.6.0] - 2026-11-03

Optimisation. Every target in `dev/PLAN.md` 11.1 is met or has a reviewed
reason in the decision log (D-060). No public API changes.

### Added

- `docs/PERFORMANCE.md`: every target, what was measured, how, and where.
- Benchmarks against `log` and error-forge: `benches/codes.rs`,
  `benches/facade.rs`, `benches/record.rs`, and `benches/floor.rs`, which
  stays stable on a busy machine. `benches/baseline.txt` and
  `tools/bench-compare.py` fail a regression over 5%.
- `tools/build-time/compare.py` times clean and incremental builds against
  thiserror and refuses a build that compiled nothing. `tools/asm/` is the
  crate whose assembly `docs/PERFORMANCE.md` reads.

### Changed

- A plain enum (no namespace, no metadata, nothing transparent) expands to
  what `docs/GENERATED-CODE.md` always described, with literal probes: a clean
  build of 20 enums is 77% faster than with thiserror.
- The identity probe decodes eight digits at a time, in the same format and
  with the same validation: `identify` 51 ns to 29 ns, foreign errors 8 ns to
  2 ns.
- `report` below the minimum severity reads two digits of the probe: 99 ns to
  6 ns. `report` delivered: 96 ns to 49 ns.
- `warn()` after a site has fired: 41 ns to 16 ns, with a new site hash.
- `Code`'s `Display`: 73 ns to 39 ns.
- The empty path of `notice`, `warn`, `trigger`, and `deprecated` is one load
  and a branch, with nothing set up first; `warn()` 3.7 ns to 0.5 ns.

### Fixed

- A non-ASCII type or variant name is a compile error, as documented. It
  compiled, and `identify` then returned `None` for it.

### Internal

- `__private::check_names` is new, for the plain arm. `probe_len` rejects a
  non-ASCII name. Both are in `docs/GENERATED-CODE.md`.

---

## [0.5.0] - 2026-11-02

The reporting facade and the owned record. Feature complete.

### Added

- The reporting facade: `Reporter`, `set_reporter`, `set_boxed_reporter`,
  `is_installed`, `reporter`, `reporter_installed_at`, `flush`; the verbs
  `report`, `report_with`, `report_record`, `notice`, `notice_with`, `warn`,
  `warn_with`, `trigger`, `deprecated`, `soft`, and the macros `notice!`,
  `warn!`, `trigger!`, `deprecated!`; `Event`, `Verb`; `WarnMode`,
  `set_warn_mode`, `set_min_severity`, `set_kinds_mask` and their readers;
  `stats` and `FacadeStats`; `InstallError` and `Slot`. One atomic load with
  nothing installed. With `std`, a verb called inside a reporter is dropped
  and counted, and a panic inside one is caught and counted. On targets
  without compare-and-swap the verbs compile to nothing.
- Once per call site: an exact flag behind the macros, a lock-free 512-site
  table behind the functions, and an overflow counter, so a warning is never
  lost to the table. A warning that could not be delivered keeps its site.
- The creation hook: `CreationHook`, `CreationInfo`, `Annotate`,
  `set_creation_hook`, `creation_hook`.
- `ErrorRecord`, `RecordLimits`, `ChainEntry`, `TrailEntry`, `TrailStep`: an
  owned, bounded snapshot of an error, with `from_error`, `from_event`,
  `new`, the `push_*` methods, and a one-line `Display`.
- `testing::capture` and `CaptureReporter`: per-thread capture for tests of
  code that reports.
- `#[cfg]` on variants and on plain fields of braced variants and structs.
- `Identity::public`: a variant's `#[public]` text, readable through
  `&dyn Error`.
- `docs/FACADE.md`, `docs/MIGRATION.md`, `docs/CODES.md`; examples `plain`,
  `codes`, `facade`, `migration_before_after`, and the `examples/no_std`
  crate, built on all four bare-metal targets in CI.
- Tests: loom models of the slot, settings, site flags, and table, with a CI
  job; facade tests of every verb, filter, and guard; record bounds, with a
  property test, the fuzz target `record_from_chain`, and exact allocation
  counts; every verb checked allocation-free in every state.

### Generated code

- New `__private` items: `probe_len`, `Site`, `notice_args`, `warn_ready`,
  `warn_site`, `trigger_args`, `deprecated_site`.
- `ProbeBuf` is now `ProbeBuf<const N: usize>`, exactly as long as its
  probe, and `new` takes `transparent: bool` and `public: Option<&str>`.
- The probe adds `|` and the public message when flag bit 0 is set;
  `tests/golden/probe.txt` updated. A public message longer than 512 bytes
  is a compile error.
- Arms, `From` impls, constants, catalog entries, and the block's checks
  carry a variant's `#[cfg]`, copied from the input.

### Changed

- `docs/API.md`, `docs/GENERATED-CODE.md`: the items above, and the behaviour
  of filtered warnings.

---

## [0.4.0] - 2026-11-02

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
  identified code displays as it was defined (`SVC-00001`);
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

[Unreleased]: https://github.com/jamesgober/define-err/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/jamesgober/define-err/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/jamesgober/define-err/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/jamesgober/define-err/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/jamesgober/define-err/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/jamesgober/define-err/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jamesgober/define-err/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/define-err/releases/tag/v0.1.0
