# define-err 1.0 stability

> **Draft, written at 0.9.1.** Nothing here is a promise yet. It becomes one at 1.0.0, when the audit record at the end is filled in and the release is approved. Until then, `0.9.x` releases may still change anything below, and each change is listed in the changelog.

What a crate that depends on `define-err = "1"` can rely on until 2.0, how each promise is checked, and what is left out on purpose.

## What 1.x keeps

| What | The promise | Checked by |
|---|---|---|
| Public API | Everything in [`API.md`](./API.md) keeps its name, signature, and meaning. Minor releases only add. A type marked `#[non_exhaustive]` there can gain variants or fields in a minor release. | `cargo semver-checks` against 1.0.0 in CI, from 1.0 |
| What generated code does | For the same `define_errors!` input, `Display`, `Debug`, `source()`, `From`, and `ErrorInfo` give the same results in every 1.x release. A generated enum is `#[non_exhaustive]` unless it says `#[exhaustive]`. | `tests/conformance.rs`, `tests/define_*.rs`, `tests/fixture` |
| Code layout | A `Code` packs into the same `u64` in every 1.x release, on every platform, and is never zero. | `tests/golden/codes.txt` |
| Code text | A code shows as `DB-0042`, padded to its namespace's width, and parses back. | `tests/golden/codes.txt`, `tests/properties.rs` |
| Probe format | The probe a generated type returns does not change layout in 1.x, so every 1.x `identify` reads every probe any 1.x release wrote. A decoder rejects any probe it does not fully understand, so a new field would be a 2.0 change. | `tests/golden/probe.txt` |
| Categories | Each category's name, gRPC number, HTTP status, exit code, retry advice, default severity, default action, and default public message. | `tests/golden/categories.txt` |
| Severities | The values 1 to 24 and their OpenTelemetry names. | `tests/golden/severity.txt` |
| Kind bits | Bits 0 to 12 keep their meaning. Bits 13 to 31 are for platform kinds added in minor releases; bits 32 to 63 are the project's own. A bit is never reassigned. | `tests/golden/kinds.txt` |
| Codes in a catalog | A retired number stays retired: using it again is a compile error. A deprecated code keeps working, and its catalog entry names a replacement when it has one. | `tests/define_meta.rs`, `tests/ui/retired_*.rs` |
| Dependencies | None, in every feature mode. | `tools/invariants.py` in CI |
| `no_std` | The crate builds with `default-features = false` on targets without `std`. | CI `no_std` jobs on four targets |
| Features | `std` (default), `alloc`, and `testing` stay, with the same meaning. New features are added off by default. | CI runs every feature mode |
| No run-time panic | No function panics at run time; checks on literal input fail at compile time. | `tests/attack*.rs`, fuzz targets, review |

The same bytes, codes, probes, and tables on every platform is NFR-10 of [`dev/PLAN.md`](../dev/PLAN.md).

## Minimum Rust version

1.81 at 1.0. In 1.x the minimum Rust version does not rise: raising it is a breaking change, so it waits for 2.0. CI builds and tests on 1.81 on every push.

## Support window

Not decided yet: how long 1.x gets fixes after 2.0 is released. It is a public commitment, so it is settled before 1.0 and written here.

## Not covered

- **`__private`.** It exists for the code `define_errors!` generates and for the attribute front end, and is not public API. It changes only in minor releases, never in a patch, and every change is in [`GENERATED-CODE.md`](./GENERATED-CODE.md) and the changelog.
- **The exact tokens generated code expands to.** What it does is covered above; how it is written may change in a minor release, for example to build faster.
- **The wording of compile errors.** A minor release may make a message clearer. Code that compiled keeps compiling, and code that failed keeps failing.
- **Performance.** [`PERFORMANCE.md`](./PERFORMANCE.md) has measurements, not promises. A hot path more than 5% slower than its baseline fails `tools/bench-compare.py` on the reference machine.
- **`Error::description()`.** Generated types return their probe there. Its contents are covered by the probe format above; calling it for anything else is not supported.

## Requirements and their tests

[`tests/TRACEABILITY.md`](../tests/TRACEABILITY.md) maps every requirement of `dev/PLAN.md` section 5, 28 functional and 14 non-functional, to the tests that check it. NFR-2, NFR-4, and NFR-5 are checked by measurement and NFR-13 by review, not by CI.

## The 1.0 audit

Filled in at the last release candidate. Until then, [`dev/READINESS-1.0.md`](../dev/READINESS-1.0.md) tracks each line of the definition of done and every open item.

| Area | Record |
|---|---|
| API | |
| Documents | |
| Tests | |
| Benchmarks | |
| Security | |
