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

**Two paths.** A block is first matched by a non-recursive fast arm, which takes the common shapes in one step and reaches 500 variants under the default recursion limit. Anything else goes through a general path in continuation-passing style, one token group at a time, which handles every attribute and every field form. Both produce the same canonical form, so `emit.rs` has one input. A plain enum (no namespace, no metadata, every message a literal) has a third arm ahead of both, which writes its items in one pass with no canonical form in between; it writes exactly what the fast path does (D-082).

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

**Once per site.** `warn!` and `deprecated!` own a static atomic at the call site. `warn` and `deprecated` are functions and cannot, so they use a table of 512 atomic slots in `src/facade/sites.rs`, keyed by a hash of the file name's text, the line, and the column, with linear probing by compare-and-swap. The compiler may emit one location twice, so the key hashes the text and not its address. Past 512 sites a warning delivers every time and `stats()` counts it: more noise, never silence.

**No compare-and-swap.** `src/facade/inert.rs` replaces `state`. See [`PLATFORM-NOTES.md`](./PLATFORM-NOTES.md).

`tests/loom.rs` compiles `control.rs`, `slot.rs`, and `sites.rs` under loom, so the models check this code and not a copy.

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
