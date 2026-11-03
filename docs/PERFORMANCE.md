# define-err performance

Every target in `dev/PLAN.md` 11.1, what was measured, and how. Measured on 2026-11-02 against version `0.6.0`. A number here is a measurement on the machine below, not a promise for yours.

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
| `Code` `Display` | under 20 ns, no allocation | 28 ns | 39 ns | error-forge `format!` 81 ns, allocates | **missed**; no allocation met |
| `Code::parse` | under 15 ns, 20% faster than a `String` lookup | 8.7 ns | 12 ns | error-forge registry 75 ns | **met** |
| catalog lookup by code | constant | | 0.6 ns, `catalog::find` 2.4 ns | error-forge registry 75 ns | **met** |
| verb with nothing installed | one load, under 1 ns | 0.27 to 0.51 ns | 0.35 to 0.68 ns | | **met** |
| the same, 20% faster than `log::warn!` at `Off` | 20% faster | 0.33 ns for `log` | 0.34 ns for `log` | `log` 0.4 | **missed**: parity |
| `warn!` after its first delivery | under 2 ns | 0.50 ns | 0.65 ns | | **met** |
| `warn()` after first delivery | under 5 ns | 9.5 ns | 16 ns | | **missed** |
| `report` filtered by severity | under 3 ns | 5.9 ns | 6.2 ns | `log::warn!` filtered 0.34 ns | **missed** |
| `report_with` filtered | | 0.72 ns | 0.91 ns | | |
| `report` to a no-op reporter | under 10 ns plus the call | 33 ns | 49 ns | `log::error!` 3.5 ns | **missed** |
| `report` of a foreign error, delivered | | 10 ns | 15 ns | | |
| `notice` to a no-op reporter | | 6.2 ns | 7.8 ns | `log::error!` 3.5 ns | |
| `identify`, generated type | under 10 ns | 26 ns | 29 ns | | **missed** |
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
| `Code` `Display` under 20 ns | 28 ns | `str::from_utf8` is the one safe way to a `&str`, and `unsafe` stays in one module. Half the cost of the `format!` it replaces, with no allocation. |
| `warn()` under 5 ns | 9.5 ns | A function has no per-site static, so the site is found by hashing its file, line, and column. The key is by content because the compiler may emit one location twice. `warn!` is exact and costs 0.5 ns, and is the one to use in a hot loop. |
| `report` filtered under 3 ns | 5.9 ns | A call and a virtual call come before the two digits it reads. `report_with` knows its severity and costs 0.72 ns. |
| `identify` generated under 10 ns | 26 ns | The probe is text from a `&dyn Error` that may be hostile, so every field is validated: 38 hex digits, the code's layout, severity and category ranges, every byte of the name. |
| `report` to a no-op under 10 ns | 33 ns | The same validation, once per event. Decoding lazily would look better here and only move the cost to any reporter that reads the code. |
| rebuild no slower than thiserror | 0.2 s slower on Linux | A macro expansion re-runs on every build, and a declarative macro produces more text than a derive. The plain arm halved it for enums without metadata (D-059). |

The 500-variant enum meets its target of under 1 s of expansion, but is still almost twice thiserror's total. Both this and the rebuild come from work per variant, which is the first item in the polish pass.
