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

The design lock. No functional code yet.

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
- `dev/ROADMAP.md`: 0.1.0 marked done; 0.5.0 lists the new items FAC-20,
  FAC-21, and REC-07.

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

[Unreleased]: https://github.com/jamesgober/define-err/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jamesgober/define-err/releases/tag/v0.1.0
