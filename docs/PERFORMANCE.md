# define-err performance

Every target in `dev/PLAN.md` 11.1, what was measured, and how. Measured on 2026-11-02 against version `0.6.0`; the paths 0.7 changed were measured again on 2026-11-03, see [What 0.7 changed](#what-07-changed). A number here is a measurement on the machine below, not a promise for yours.

## Summary

- **A generated error costs what a hand-written one costs.** The same type written three ways compiles to one function, and `Display` is within noise of both.
- **A crate with error types builds faster.** A clean build of 20 enums takes 77% less time than with thiserror, because there is no `syn`, `quote`, or `proc-macro2` to compile.
- **With no reporter installed, every verb is one load and a branch**, the same as `log` with its level `Off`.
- **Rebuilding one crate is slower on Linux, by about 0.2 s**, and an enum of 500 variants takes almost twice as long to compile as with thiserror. A declarative macro expands to more text than a derive. See [Missed targets](#missed-targets).

## Method

| | |
|---|---|
| CPU | AMD Ryzen 9 9950X3D, 16 cores, 32 threads |
| OS | Ubuntu under WSL2, kernel 6.6.87; build times also on Windows 11 |
| Toolchain | rustc 1.99.0 |
| Competitors | thiserror 2.0.21, log 0.4.35, error-forge 1.0.2, criterion 0.5.1 |
| Frequency policy | not controlled: WSL2 cannot set it |
| Load | **a load average of 47 on 32 threads throughout** |

**The machine was not quiet.** Other builds ran during every measurement, and criterion means moved by up to 40% between identical runs. So each hot path has two numbers:

- **Floor:** from `benches/floor.rs`, the fastest of 4,000 batches of 1,000 calls. It is what the code costs when nothing competes for the core, and it repeats. This is what `tools/bench-compare.py` compares with `benches/baseline.txt`, and a path over 5% fails. The baseline belongs to this machine; on another, regenerate it first.
- **Mean:** from criterion, 100 samples, default settings. It carries the load, so read it as an upper bound, and read comparisons within one group, which ran together.

Floors include about 0.25 ns of loop overhead (`empty_loop`), so sub-nanosecond figures are close to the limit of the method.

**Build settings.** Benchmarks use `[profile.bench]`: LTO off, 16 codegen units, as an application builds. A library's release profile never reaches its users, and with this crate's fat LTO the hash in `warn()` folded away at compile time and the benchmark read 1.4 ns against a true 9.5 ns.

To reproduce:

```bash
cargo bench --all-features
```

```bash
cargo bench --bench floor --all-features > floor.txt
```

```bash
python tools/bench-compare.py floor.txt
```

```bash
python tools/build-time/compare.py --runs 10
```

## Results

| Operation | Target | Floor | Mean | Compared with | Result |
|---|---|---|---|---|---|
| construct and return a generated error | identical to hand-written | | 1.0 ns | thiserror 1.2 ns, hand-written 1.2 ns | **met**: same instructions, see [Assembly](#assembly) |
| `Display` of a generated variant | within 2% of thiserror | | 120.3 ns | thiserror 120.4 ns, hand-written 120.9 ns | **met** |
| `size_of` of a generated type | equal to hand-written | | | | **met**: asserted in `tests/traits.rs` |
| `code()` and other `ErrorInfo` methods | under 1 ns | 0.24 ns | 0.5 to 0.7 ns | error-forge 0.4 to 0.6 ns | **met** |
| `Code` `Display` | under 20 ns, no allocation | 24 ns | 39 ns (0.8) | error-forge `format!` 81 ns, allocates | **missed**; no allocation met |
| `Code::parse` | under 15 ns, 20% faster than a `String` lookup | 8.7 ns | 12 ns | error-forge registry 75 ns | **met** |
| catalog lookup by code | constant | | `code_info` 0.6 ns; `catalog::find` 2.4 ns for two codes, linear; `Catalog::find` O(log n), est. under 50 ns at 10,000 codes | error-forge registry 75 ns | **met** by `code_info` and `Catalog` (0.10); `catalog::find` is a scan |
| verb with nothing installed | one load, under 1 ns | 0.27 to 0.51 ns | 0.35 to 0.68 ns | | **met** |
| the same, 20% faster than `log::warn!` at `Off` | 20% faster | 0.33 ns for `log` | 0.34 ns for `log` | `log` 0.4 | **missed**: parity |
| `warn!` after its first delivery | under 2 ns | 0.50 ns | 0.65 ns | | **met** |
| `warn()` after first delivery | under 5 ns | 7.0 ns | 16 ns | | **missed** |
| `report` filtered by severity | under 3 ns | 5.7 ns | 6.2 ns | `log::warn!` filtered 0.34 ns | **missed** |
| `report_with` filtered | | 0.64 ns | 0.91 ns | | |
| `report` to a no-op reporter | under 10 ns plus the call | 31 ns (0.9) | 49 ns (0.9) | `log::error!` 2.9 ns | **missed**; 0.10 adds the `identify` `Display` check, about 16 ns at the floor |
| `report` of a foreign error, delivered | | 10.7 ns | 15 ns | | |
| `notice` to a no-op reporter | | 5.1 ns | 7.8 ns | `log::error!` 2.9 ns | |
| `identify`, generated type | under 10 ns | 38 ns (0.10; 26 ns in 0.9) | 48 ns (0.10, under load) | | **missed**: 0.10 adds the `Display` check (D-091) |
| `identify`, foreign type | under 2 ns | 1.8 ns | 2.1 ns | | **met** |
| `ErrorRecord::from_error`, three levels | under 1 us, allocations published | | 645 ns; one level 394 ns; clone 210 ns | none | **met**: 7 allocations for three foreign levels, 4 for one generated level, pinned in `tests/alloc_free.rs` |
| `warn!`, 16 threads, one site | | | 1.6 ns | | |
| clean build, 20 enums | 20% faster than thiserror | | see [Build times](#build-times) | thiserror 2 | **met**: 77% faster |
| rebuild after editing one message | no slower than thiserror | | see [Build times](#build-times) | thiserror 2 | **missed on Linux**, met on Windows |
| expansion of a 500-variant enum | under 1 s of macro expansion | | 0.82 s under load, before the plain arm | thiserror 2 | **met** |

Every verb allocates nothing on any path; `tests/alloc_free.rs` checks each with a counting allocator.

### What 0.6 changed

Criterion means at the same load. The first column was built with this crate's fat LTO, which favours it, and the second with the settings above.

| Path | Before | After |
|---|---|---|
| `report` filtered | 99 ns | 6.2 ns |
| `report` delivered | 96 ns | 49 ns |
| `warn()` after first delivery | 41 ns | 16 ns |
| `warn()` not installed | 3.7 ns | 0.50 ns |
| `identify`, generated | 51 ns | 29 ns |
| `identify`, foreign | 8.1 ns | 2.1 ns |
| `Code` `Display` | 73 ns | 39 ns |

### What 0.7 changed

0.7 reads an error's probe inside the panic guard, because `description()` is the error's own code and may panic (D-062 in `dev/PLAN.md`). Floors of `0.6.0` and `0.7.0` built and run in turn, four rounds each, at a load average above 100; the median of each is shown, and the floors in the table above are 0.7's best. Means were not run again.

| Path | 0.6.0 | 0.7.0 | Why |
|---|---|---|---|
| `report` filtered | 5.4 ns | 5.8 ns | the severity read is inside `catch_unwind` |
| `report` of a foreign error, delivered | 9.4 ns | 10.9 ns | the same |
| `report` to a no-op reporter | 33.2 ns | 32.3 ns | noise |
| `warn()` after first delivery | 9.3 ns | 7.1 ns | code layout around the guard |
| `notice` to a no-op reporter | 5.8 ns | 5.3 ns | the same |

Reading the probe outside the guard would win back the half nanosecond and let a `description()` that reports itself recurse without limit.

### What 0.8 changed

0.8 shows a `Path` or `PathBuf` field through `display()`, as thiserror does (D-067 in `dev/PLAN.md`). Nothing changes at run time: the rebinding inlines to the reference the arm already had. It costs at compile time, one `let` and one method lookup per field of every plain message. Rebuilds of the crates of `tools/build-time/compare.py`, `0.7.0` and `0.8.0` built in turn for 15 rounds, fastest of each:

| Rebuild | 0.7.0 | 0.8.0 | Change |
|---|---|---|---|
| 500-variant crate | 0.409 s | 0.454 s | +10.9% |
| 20-enum crate, one message edited | 0.100 s | 0.106 s | +6.0% |

The first form of the change called a helper macro per field and cost 4.5 to 8% more than this; it was rewritten before release.

### What 0.9 changed

A plain enum (no namespace, no metadata, every message a literal) is written in one pass over its variants instead of five (D-082 in `dev/PLAN.md`). The generated code is the same, token for token. The cost was in rustc's matching of `macro_rules!` patterns: each pass over the whole list of variants matches a pattern with many optional groups, and that work grows faster than the number of variants (250, 500, and 1,000 variants of the old fast path alone took 0.12, 0.29, and 1.05 s to expand). Rebuilds of the crates of `tools/build-time/compare.py`, the two versions alternated round by round, fastest of each:

| Rebuild | before | after | Change |
|---|---|---|---|
| 500-variant crate, 15 rounds | 0.571 s | 0.414 s | -27.6% |
| 500-variant crate, 10 rounds | 0.646 s | 0.457 s | -29.2% |
| 20-enum crate, 15 rounds | 0.141 s | 0.123 s | -12.5% |
| 20-enum crate, 10 rounds | 0.165 s | 0.163 s | -1.3% |

Types with a namespace or metadata keep the general passes. Nothing changes at run time.

0.9.1 computes each character and digit of a `Code`'s text from the whole value, where 0.9.0 divided by 37, then by 10, one step after another (D-084). The text is the same, checked against `std` formatting by a property test. `benches/floor.rs` code_display, the two versions alternated for four rounds at a load average of 60: 25.7 to 26.8 ns before, 23.3 to 23.9 ns after, about 10% less. For scale, on the same runs: `write!` of the seven-byte `&str` `"DB-0001"` 10.5 ns, of a four-digit `u32` 15.4 ns; the same `Display` without its `str::from_utf8` 19.6 to 20.2 ns. A 16-byte aligned buffer, which lets `from_utf8` check a word at a time, saved under 1 ns more and was not kept.

0.9.1 also scans a record's chain message for control characters eight bytes at a time (D-085). Its budget in `dev/PLAN.md` 10.2 is 1 ns per 16 bytes; `chars().any(char::is_control)` took 6 to 10. Floors at a load average of 60:

| Message | 0.9.0 | 0.9.1 |
|---|---|---|
| 29 bytes, ASCII | 11.6 ns | 8.2 ns |
| 132 bytes, ASCII | 49 ns | 10.8 ns (1.3 ns per 16 bytes) |
| 115 bytes, with umlauts | 43 ns | 11.2 ns |
| 21 bytes, a newline at byte 9 | 4.3 ns | 7.0 ns |

The last row is slower because the old scan stopped at the first control character and the new one reads the whole message.

0.9.2 reads an enum 32 variants per macro step, where 0.9.1 matched one pattern over the whole list (D-087). The change is about the recursion limit (an enum not in the canonical form could fail to compile from 12 variants), but it also speeds up large enums with metadata, because rustc's matcher copies everything it has matched so far each time it enters an optional group, so one pattern over a long list costs time in proportion to the square of its length. Expansion time (CPU seconds of `rustc -Z unpretty=expanded`, the two versions alternated, fastest of three rounds, load average 80 to 95), each variant with a code and a category:

| Crate | 0.9.1 | 0.9.2 |
|---|---|---|
| one enum of 500 variants | 4.14 s | 2.33 s |
| one enum of 1,000 variants | 27.6 s | 6.1 s |
| 20 enums of 8 variants, six rounds | 0.66 s | 0.58 s |
| plain enum of 500 variants (the D-082 arm, unchanged) | 0.52 s | 0.46 s |

The expanded code is the same, token for token, on these crates and on every test target, except the bindings of unnamed tuple fields with a field marker, which are now named by position.

0.9.3 changes the block's duplicate-code check (D-088). Measured with `-Z time-passes` on a full debug build of 1,000 variants with codes, type checking took more time than expansion, and `__private::check_block` was about 2.5 s of it: it compared every pair of code numbers in const evaluation. It now sorts them and looks numbers up by binary search, with the same messages from the same lines. CPU seconds of a full debug build, the two versions alternated, fastest of four rounds (six for the two smaller crates), load average about 85:

| Crate | 0.9.2 | 0.9.3 |
|---|---|---|
| one enum of 1,000 variants | 24.9 s | 21.9 s |
| one enum of 500 variants | 9.95 s | 8.39 s |
| one enum of 250 variants | 3.63 s | 3.13 s |
| 20 enums of 8 variants | 3.27 s | 3.02 s |

What is left for a type with metadata is spread over every variant: the probe costs about 3 ms of CPU per variant at this load, in its four `const` items and the UTF-8 check of `as_str`, and the `code_info` entry is built in expansion as well as in the catalog. These are linear, not quadratic, and D-077 found the obvious restructuring slower.

0.10.0 makes `identify` refuse an error whose `Display` also begins with the probe's magic (D-091): on Rust 1.81 to 1.89, std's string errors return their message from `description()`, so outside text could pass for a generated error. The check is one `Display` call into a writer that stops after four bytes, made only for an otherwise valid probe, and for a generated type that is the cost of its own `Display` up to its first piece. 0.9.3 and 0.10 alternated in one harness (two million calls, fastest of 40 runs, three rounds, load average about 130), `identify` on `DbError::Timeout { after_ms }`: 22.2 to 22.8 ns before, 37.3 to 39.9 ns after; the check alone 15.4 to 16.8 ns. The criterion benches, the two versions alternated for three rounds (a fourth, run beside the test gate, is left out): `identify` 24.4 to 29.1 ns mean before, 47.2 to 48.9 after. The minimum-severity drop in `report` does not make the check, so `report` with nothing installed and filtered `report` are unchanged (filtered 8.6 to 9.8 ns before and 9.1 to 9.2 after in the same runs). The kinds mask is now one 64-bit load where it was two 32-bit loads (D-093).

## Build times

`tools/build-time/compare.py`, median of 10 runs: 20 enums of 8 variants, and one enum of 500. The two crates declare the same errors. The script refuses a build that compiled nothing, because a fresh cargo fingerprint looks like a very fast rebuild.

| Linux, seconds | define-err | thiserror | Change |
|---|---|---|---|
| clean build, dependencies included | 1.21 | 5.30 | **-77%** |
| rebuild after editing one message | 0.46 | 0.26 | +76% |
| rebuild of the 500-variant crate | 1.33 | 0.73 | +83% |

| Windows, seconds | define-err | thiserror | Change |
|---|---|---|---|
| clean build, dependencies included | 2.24 | 9.32 | **-76%** |
| rebuild after editing one message | 0.88 | 0.97 | -9% |
| rebuild of the 500-variant crate | 2.29 | 1.29 | +77% |

The Windows run has `CARGO_INCREMENTAL=0`, because its build output is on exFAT, so its rebuild compiles the whole crate. The Linux run uses incremental compilation. Most crates are rebuilt rarely and compiled clean on every CI run, which is where the larger difference sits, but a developer editing an error type feels the rebuild.

A type with metadata generates more than these plain enums: a match per `ErrorInfo` method and a probe built in a `const`.

## Assembly

`tools/asm/` compiled with default release settings (`cargo rustc --release -- --emit asm`), x86-64, trimmed to the instructions.

**Construction.** `asm_make_ours`, `asm_make_theirs`, and `asm_make_hand` build the same two-variant error generated, with thiserror, and by hand. The compiler merged them, so there is one body:

```text
asm_make_ours = asm_make_hand
asm_make_theirs = asm_make_hand
```

**The empty path.** `define_err::report(error)`, as compiled into the caller:

```text
mov   rax, [CONTROL]         ; the address of the control word
mov   edi, dword ptr [rax]   ; the one load
mov   eax, edi
and   eax, 3                 ; the reporter's state
cmp   eax, 2
jne   done                   ; not ready: return
...                          ; tail call to the out-of-line path
done:
ret
```

`notice`, `warn`, `trigger`, and `deprecated` have the same shape, with nothing set up before the load. `notice!` and `trigger!` do not evaluate their arguments when nothing is installed; `warn!` also reads its site's flag, and does not evaluate them once the site has fired.

## Missed targets

Each has a reviewed reason (D-060 in `dev/PLAN.md`) and goes back on the list in the pre-1.0 polish pass, which also re-measures on a quiet machine. The estimates in the plan were written before any of this code existed.

| Target | Result | Why it stands |
|---|---|---|
| verb 20% faster than `log` at `Off` | parity | Both are one load and a branch. There is nothing left to remove. |
| `Code` `Display` under 20 ns | 24 ns | `write!` of a seven-byte `&str` alone costs 10.5 ns on this machine, and `str::from_utf8` of the text 4.5 ns more. Without that check it reads 20 ns, and `unsafe` stays in one module. Under half the cost of the `format!` it replaces, with no allocation. |
| `warn()` under 5 ns | 7.0 ns | A function has no per-site static, so the site is found by hashing its file, line, and column. The key is by content because the compiler may emit one location twice. `warn!` is exact and costs 0.5 ns, and is the one to use in a hot loop. |
| `report` filtered under 3 ns | 5.7 ns | A call, the panic guard, and a virtual call come before the two digits it reads. `report_with` knows its severity and costs 0.64 ns. |
| `identify` generated under 10 ns | 26 ns | The probe is text from a `&dyn Error` that may be hostile, so every field is validated: 38 hex digits, the code's layout, severity and category ranges, every byte of the name. |
| `report` to a no-op under 10 ns | 31 ns | The same validation, once per event. Decoding lazily would look better here and only move the cost to any reporter that reads the code. |
| rebuild no slower than thiserror | 0.2 s slower on Linux | A macro expansion re-runs on every build, and a declarative macro produces more text than a derive. The plain arm halved it for enums without metadata (D-059). |

The 500-variant enum meets its target of under 1 s of expansion, but is still almost twice thiserror's total. Both this and the rebuild come from work per variant, which is the first item in the polish pass. 0.8's path display adds to it, by the figures in [What 0.8 changed](#what-08-changed).
