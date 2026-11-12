# define-err architecture

How the crate is put together, for anyone changing it. The public surface is in [`API.md`](./API.md), and what the macro must expand to is in [`GENERATED-CODE.md`](./GENERATED-CODE.md).

## Modules

| Path | What it holds |
|---|---|
| `src/macros/define_errors.rs` | `define_errors!`: parses the block into one canonical form |
| `src/macros/emit.rs` | turns the canonical form into the type and its impls |
| `src/macros/verbs.rs` | `notice!`, `warn!`, `trigger!`, `deprecated!` |
| `src/code.rs` | `Code`, `Namespace`, `code!`, `namespace!`, parsing and text |
| `src/severity.rs`, `category.rs`, `kinds.rs` | the three classifications |
| `src/info.rs` | `ErrorInfo`, `CodeInfo`, the `catalog` module |
| `src/probe.rs` | `identify`, the probe decoder, `innermost_code` |
| `src/record.rs` | `ErrorRecord`, its limits, the bounded writer |
| `src/secret.rs` | `Secret<T>` |
| `src/facade/` | the reporting facade, below |
| `src/testing.rs` | `testing::capture` |
| `src/private.rs` | `__private`: the const functions generated code calls |

No file depends on a crate outside `core`, `alloc`, and `std`.

## The macro

`define_errors!` is a declarative macro, so a build needs no proc macro and no build dependency.

**How an enum is read (D-087).** A `macro_rules!` macro can gather results only by passing them along a chain of calls, and every link adds one to the expansion depth that `recursion_limit` caps. So the enum parser keeps the chain short:

1. One step splits the body into one token group per variant.
2. Each later step reads 32 groups with a single arm. Inside every `#[...]` the arm matches the attribute by its first word, so variant attributes may come in any order. Unnamed tuple fields are first given names (`__define_err_0`, ...) by a step that rewrites the 32 groups, then read the same way. A 500-variant enum compiles with `recursion_limit` set as low as 56 to 72, depending on its forms; the default is 128.
3. A chunk the arm cannot read is retried one variant at a time, and a variant it still cannot read goes to the munching parser, which normalises one attribute or field per step and reports any mistake with a message naming the fix. Structs always take that parser; they are one variant.

Every path produces the same canonical form, so `emit.rs` has one input. A plain enum (no namespace, no metadata, every message a literal) has an arm ahead of all of them, which writes its items in one pass with no canonical form in between; it writes exactly what the general path does (D-082).

**The canonical form.** Each variant carries eight metadata slots and a ninth slot of `cfg` attributes. Each field is `{ [attrs] [vis] [cfg] role name : Type }`.

**Two arms in `@info`.** `emit.rs` writes `Display` first, then `@info`, then the `From` impls. `@info` picks by pattern:

- The **plain arm** takes a type with no namespace, no metadata, and no transparent or formatted message. It emits literal probe strings, one `check_names` constant for the whole type, and only the `ErrorInfo` overrides that differ from the defaults. This is most of a clean build's saving against thiserror (D-059).
- The **general arm** takes everything else: a match per `ErrorInfo` method, a probe built in a `const`, the catalog, and the checks on codes. `tests/define_meta.rs` holds both arms to the same answers.

**What generated code may do.** Absolute paths only, no `cfg(feature)`, no constructors or inherent methods beyond the associated constants for codes and the catalog, and it compiles under `#![no_implicit_prelude]` and `#![no_std]` in the user's crate. `tests/no_prelude.rs` checks it.

## The const-panic rule

No function panics at run time. `panic!` and `assert!` are reachable only in constant evaluation, where they are compile errors. These are all of the sites outside tests:

| Site | What it rejects |
|---|---|
| `src/private.rs` `namespace` | a bad namespace, a width outside 1 to 8 |
| `src/private.rs` `code` | a number that does not fit |
| `src/private.rs` `check_block` | a duplicate code, a reused retired code, a bad `replacement` |
| `src/private.rs` `check_docs` | a docs template with no placeholder |
| `src/private.rs` `probe_len` | a name over 128 bytes or not ASCII, a public message over 512 bytes |
| `src/private.rs` `ProbeBuf::as_str` | a probe that is not UTF-8 |
| `src/kinds.rs` `Kinds::project` | a project bit of 32 or more |
| `src/severity.rs` `Severity::at` | a severity outside 1 to 24 |
| `src/lib.rs` | feature combinations that cannot exist |

Each of the checks that takes user input has a compile-fail test in `tests/ui/`. Run-time twins return `Option` or `Result`.

## The facade

```text
verb -> Acquire load of the control word -> not ready: return
                                         -> ready: the gate's checks -> guard -> receive
```

**The control word.** One `AtomicU32` in `src/facade/control.rs`:

```text
bits 0-1    reporter slot: 0 empty, 1 installing, 2 ready
bits 2-3    warn mode
bits 4-8    minimum severity, 0 to 24 (0 = everything)
bits 9-10   creation-hook slot
bits 11-31  reserved
```

The kinds mask is two more `AtomicU32` values, read only after the word says a reporter is ready. Every verb reads the word once, so the empty path is one load and a branch. A `Word` value exists only from an `Acquire` load, so holding one that says "ready" proves the slot was published. That proof is what makes the read in `src/facade/slot.rs` sound, and it is the only `unsafe` in the crate.

**Installing.** A compare-and-swap claims the state bits (`Claim::Won`, `Busy`, or `Taken`). The winner writes the reference and the location, then publishes with a `Release` store of "ready". A loser that finds the slot mid-install returns at once with no location: a verb and an installer never wait on another thread.

**Verbs.** Each is an inline gate over an out-of-line slow path, so the caller's code holds the load and the branch and sets nothing up first. A slow path that reads an error's probe does so under the guard, because that is a call into the error's own code.

**The guard.** `src/facade/guard.rs`. With `std`, a thread-local flag drops a verb called inside a reporter call and counts it, and `catch_unwind` contains a panic. A payload that panics when dropped is caught too, up to four times, and then leaked.

**Once per site.** `warn!` and `deprecated!` own a static atomic at the call site. `warn` and `deprecated` are functions and cannot, so they use a table of 512 atomic slots in `src/facade/sites.rs`, keyed by a hash of the file name's text, the line, and the column, with linear probing by compare-and-swap. The compiler may emit one location twice, so the key hashes the text and not its address. `deprecated(item)` mixes the item's text into the key, so each item warns once at a site. Past 512 sites a warning delivers every time and `stats()` counts it: more noise, never silence.

**The kinds mask.** `src/facade/mask.rs`. One `AtomicU64` where the target has 64-bit atomics. Elsewhere two 32-bit halves under a sequence count: writers take turns through the count, and a reader retries while a write is in progress, so it never combines one write's low half with another's high half. After 64 tries that each overlapped a write it returns every bit set, which can deliver an event the new mask excludes and never drops one.

**No compare-and-swap.** `src/facade/inert.rs` replaces `state`. See [`PLATFORM-NOTES.md`](./PLATFORM-NOTES.md).

`tests/loom.rs` compiles `control.rs`, `slot.rs`, `sites.rs`, and `mask.rs` under loom, so the models check this code and not a copy.

**One copy per build.** The facade's statics belong to one copy of the crate, and before 1.0 every minor version is a separate copy. `Cargo.toml` sets `links = "define_err_facade"` with an empty `build.rs`, so Cargo refuses a build with two copies from 0.10 on (`tests/two_copies.sh`).

## The probe

How a value held as `&dyn Error` says what it is, with no `TypeId` table and no registration. Every generated type returns a probe from the deprecated `Error::description()`:

```text
\0DE1  16 hex code  2 severity  2 category  16 kinds  2 flags  |  name  [ | public ]
```

The code field is sixteen `x` characters when there is no code. Flag bit 0 says a public message follows; bit 1 says the variant is transparent. The whole text is built at compile time, so producing it costs one match.

**Why `description()`.** It is on every `Error`, also through `dyn` and with no `std`. A foreign type returns its default text and stops at the first four bytes, which is why `identify` on a foreign error costs 2 ns.

**The format is frozen at 1.0.** Two builds of this crate in one process must read each other's probes.

**Decoding.** `src/probe.rs`. A foreign error stops at the magic in an inlined gate. Otherwise `decode_fields` reads sixteen-digit fields eight digits at a time with word arithmetic in safe code, and checks every field. A probe is untrusted input: see [`SECURITY.md`](./SECURITY.md#forged-probes). `probe_decode` fuzzes this path.

**The severity shortcut.** `report` with a minimum severity reads only the two severity digits first. A forged probe can already state any severity, so this gives it nothing it did not have.

## The record

`ErrorRecord` is the owned form of an error for anything that outlives the event. It walks `source()` up to `max_chain`, and writes each `Display` through a bounded writer that refuses text past the limits and always cuts on a character boundary. A `Display` that ignores write errors cannot grow it. `public` is set only from a public message or the category's sentence, never from `Display`.

## Tests by kind

| Kind | Where |
|---|---|
| unit | `#[cfg(test)]` at the end of each file |
| behaviour | `tests/*.rs`, one process per file because the slot is write-once |
| attack | `tests/attack*.rs`, mapped to threats in [`SECURITY.md`](./SECURITY.md) |
| conformance | `tests/conformance.rs`, against thiserror |
| compile-fail | `tests/ui/`, pinned to one toolchain |
| golden | `tests/golden/`, the frozen text and bit values |
| property | `tests/properties.rs`, `tests/attack.rs` |
| fuzz | `fuzz/fuzz_targets/` |
| concurrency | `tests/loom.rs`, Miri, `tests/stress.rs` |
| allocation | `tests/alloc_free.rs`, `tests/attack_memory.rs` |
