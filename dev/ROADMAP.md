# define-err - Roadmap

> Path from scaffold to a stable 1.0. Each phase has hard exit criteria and ends with the release routine (bump, CHANGELOG, release notes, local gate, push, green CI, tag, GitHub release, `cargo doc`, `cargo publish`).
> Design, API item ids (ROOT-, DEF-, CODE-, SEV-, CAT-, KND-, INF-, PRB-, SEC-, FAC-, REC-, TST-), requirements (FR-, NFR-), threats (T1 to T17), and edge cases (E-1 to E-100): [`PLAN.md`](./PLAN.md).
>
> **Anti-deferral rule:** no listed task moves to a later phase unless this file records the move and the reason, with a decision-log entry in `PLAN.md`. P0 items ship in the version listed and never move.
>
> **Consumers waiting on this crate:** L2 (derive) starts from `docs/GENERATED-CODE.md` once 0.2 is approved; L3 (report-err) needs the 0.2 answers to its requests X-1 to X-12 before its 0.4.

## v0.1.0 - Scaffold (DONE, 2026-10-23)
Exit: CI green on every job; the crate published to claim the name.
- [x] Repository `jamesgober/define-err` created and cloned to `error-collection/define-err` (D-001).
- [x] Manifest (edition 2021, MSRV 1.81, features `std` (default), `alloc`, `testing`, zero dependencies), README, CHANGELOG, REPS, dual license, `deny.toml`, `clippy.toml`, `rustfmt.toml`, `.editorconfig`, `.gitattributes`.
- [x] `src/lib.rs`: `#![no_std]`, crate docs, lint attributes (`deny(unsafe_code)`, `deny(missing_docs)`, `docsrs`), `VERSION` (ROOT-01), one test per feature mode (D-005).
- [x] `dev/PLAN.md` (the specification as the plan, decision log D-001 to D-007), `dev/DIRECTIVES.md`, `dev/ROADMAP.md`.
- [x] CI: Linux, macOS, Windows on x86_64 and aarch64, stable and 1.81, four feature modes; `no_std` cross-check job; Miri; package; security job (D-007).
- [x] CI green on all 20 jobs, tagged `v0.1.0`, GitHub pre-release, published on crates.io (2026-10-23).

## v0.2.0 - Design lock
Exit: James approves `docs/API.md` and `docs/GENERATED-CODE.md`.
- [x] `docs/API.md` for every item in `PLAN.md` section 8 (signatures, errors, examples), each marked with the version it arrives in (2026-10-23).
- [x] `docs/GENERATED-CODE.md`: the contract of `PLAN.md` 7.5 with full expansions for every form of 7.4 and the `__private` versioning rule, ready for the L2 owner (2026-10-23).
- [x] Probe format, code layout, category tables, severity names, and kind bits written as golden files (`tests/golden/`), before the code that produces them; the probe builder and the source dispatch were prototyped on 1.81 and match them.
- [x] Every open decision in `PLAN.md` section 17 answered: OD-1 by James, the rest accepted as recommended (D-002 to D-004, D-018).
- [x] report-err's cross-crate requests decided: X-1 (D-008), X-7 (D-009), X-8 (D-010), X-9 (D-011) all accepted; X-12 stability (D-019).
- [x] Simplicity review (standard 2.10): the Tier 1 example is one block plus two calls; verbs called by path, metadata through one prelude import (D-013).
- [x] One new defence idea recorded (`PLAN.md` 10.2: error text never echoes rejected input, D-017).
- [ ] James approves `docs/API.md` and `docs/GENERATED-CODE.md`.

## v0.3.0 - Core: thiserror parity
API: DEF-01 to DEF-12, DEF-27, INF-01 (trait with defaults), INF-07, SEC-01 to SEC-03, ROOT-02, ROOT-03.
Exit: page-db's test suite passes on a pilot branch with byte-identical messages; construction and `Display` benchmark identical to thiserror.
- [ ] `define_errors!` fast and general paths; every compile error of `PLAN.md` 8.3 with a `trybuild` case (CI job added).
- [ ] Conformance suite against thiserror (12.1); `no_prelude` and naming tests (E-5, E-6).
- [ ] Edge cases E-1 to E-32 (those not needing 0.4 metadata), E-96 to E-100.
- [ ] Pilot migration of page-db on a branch; findings in the decision log.

## v0.4.0 - Extended: codes and metadata
API: DEF-13 to DEF-23, DEF-25, CODE-01 to CODE-09, SEV-01 to SEV-04, CAT-01 to CAT-07, KND-01 to KND-05, INF-01 (all methods) to INF-06, PRB-01 to PRB-03.
Exit: the `PLAN.md` 7.4.4 example compiles and every assertion in `tests/define_meta.rs` passes; five-digit codes display as `DIS-00001`.
- [ ] Code packing and parsing; fuzz targets `code_parse`, `code_from_u64` started.
- [ ] Compile-time checks (7.13) with UI tests; the duplicate-span technique verified on stable and 1.81.
- [ ] Category tables and kind bits locked against the golden files.
- [ ] Edge cases E-33 to E-64, E-87 to E-89.

## v0.5.0 - Feature complete: facade and record
API: FAC-01 to FAC-18, FAC-20, FAC-21, REC-01 to REC-07, TST-01, TST-02, DEF-24.
Exit: every P0, P1, and P2 item exists; a prototype L7 reporter (in a test) receives every verb with correct locations.
- [ ] Reporter slot with loom models and Miri; control word; verbs; once-per-site flags and table; reentrancy and panic isolation (loom CI job added).
- [ ] `ErrorRecord` with limits and the bounded writer.
- [ ] `docs/FACADE.md`, `docs/MIGRATION.md`, `docs/CODES.md`, `examples/`.
- [ ] Edge cases E-65 to E-86, E-90 to E-95.

## v0.6.0 - Optimisation
Exit: `docs/PERFORMANCE.md` published with dates, hardware, and method.
- [ ] Benchmarks of `PLAN.md` 11.2 against thiserror, `log`, and error-forge; build-time comparison script.
- [ ] Assembly inspection of the empty facade path (one load and a branch).
- [ ] Every target in 11.1 met or a reviewed reason recorded.

## v0.7.0 - Hardening
Exit: no open fuzz finding; every threat has a passing attack test.
- [ ] Every fuzz target 24 hours; attack tests T1 to T17; Miri over the suite; loom models complete; 8-hour soak.
- [ ] `docs/SECURITY.md`, `docs/ARCHITECTURE.md`, `docs/PLATFORM-NOTES.md`.

## v0.8.0 - Alpha: first migrations
Exit: each migrated crate's CI is green on its own repository against the published 0.8.0.
- [ ] Migrate page-db, Hive DB distro, hive-exec; migrate kraken (MSRV 1.80 to 1.81).
- [ ] Remove the unused `thiserror` dependency from cli-forge.
- [ ] Feedback from each migration recorded; fixes land here.

## v0.9.0 - Beta and release candidates
Exit: API unchanged across the last release candidate; L2 and L3 owners sign off on the contract; James's review below done.
- [ ] Migrate noml-rust and mod-cli; config-lib's path decided with James (`PLAN.md` 19.4).
- [ ] L2 builds against `docs/GENERATED-CODE.md` and passes the shared conformance suite; L3 reads codes through the probe.
- [ ] **James's pre-1.0 review**, before the API is frozen; anything he adds goes into this roadmap before 1.0.
- [ ] Release candidates `0.9.x` until no API change is needed for two weeks of use.
- [ ] `docs/STABILITY-1.0.md` drafted with the requirement-to-test map (`tests/TRACEABILITY.md`).

## v1.0.0 - Stable
Exit: `PLAN.md` section 16 complete and James approves.
- [ ] Full audit (API, docs, tests, benchmarks, security) recorded in `docs/STABILITY-1.0.md`; API, code layout, probe format, mapping tables, and kind bits frozen until 2.0.
- [ ] Never published without James's explicit go.
