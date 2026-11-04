# define-err security

`define-err` sits in every library, so its threats are about what errors **say** (to clients, to logs, to other nodes) and about the one shared reporting slot. This file is the threat model: what each defence is, what it costs, which test holds it, and what is not defended.

Attackers considered: a remote client who sees error responses, a log reader with partial access, a malicious or compromised dependency in the same process, and a peer node sending codes.

Report a suspected vulnerability privately to `me@jamesgober.com`.

## The threats

Costs are measured in [`PERFORMANCE.md`](./PERFORMANCE.md) unless marked as an estimate.

| Id | Threat | Defence | Cost |
|---|---|---|---|
| T1 | Internal details reach clients | `public_message()` and `Display` are separate. The default public message is the category's sentence, never `Display`. | none at run time |
| T2 | Secrets reach logs through `Display` or `Debug` | `Secret<T>` prints `[redacted]` under every format spec and has no `Deref`, `AsRef`, or `Borrow`. | none |
| T3 | User text read as a format template | Templates are literals. Every `&str` verb passes its text through `{}`. | none |
| T4 | Newlines and control characters forge log lines | This crate writes no logs. `Code`, `Severity`, `Category`, and `Kinds` text cannot hold a control byte. A record keeps text as written and sets `has_control`, so a sink can escape without scanning. | one scan per recorded message, under 1 ns per 16 bytes (estimate) |
| T5 | Memory exhausted by a huge or cyclic chain | `RecordLimits` caps depth, bytes per message, and total bytes. The bounded writer refuses text past the cap. A `source()` that returns itself stops at `max_chain`; `innermost_code` stops at 64. | one check per write |
| T6 | Error floods slow every caller | Verbs never block and never allocate. Filtered events return before formatting. A site delivers once. | 0.27 to 0.51 ns with nothing installed |
| T7 | A dependency installs first, to read or silence reports | The slot is written once and never replaced. The application's own install fails and names the winner's file and line. Treat that as a fatal boot error. `reporter_installed_at()` answers the same question for audits. | one stored pointer |
| T8 | A reporter reports while reporting | With `std`, a thread-local flag drops the nested event and counts it. Without `std`, the contract in [`FACADE.md`](./FACADE.md) forbids it. | one thread-local access |
| T9 | A panic unwinds through code that only wanted to report | With `std`, `receive` and the read of an error's `description()` run under `catch_unwind`. A payload that panics when dropped is caught too. | about 0.4 ns on a filtered `report`, 1.5 ns on a delivered foreign one: the read sits inside `catch_unwind` |
| T10 | Two errors share a code, or a retired code returns with a new meaning | Duplicates and retired numbers are compile errors within a block. `catalog::check_unique` finds them across crates at boot. | compile time |
| T11 | A foreign error forges a probe to claim a code or `SECURITY` | See [Forged probes](#forged-probes). | `identify` 26 ns, inside which every field is validated |
| T12 | Malformed codes from headers, files, or peers | `Code::parse` and `Code::from_u64` accept only the exact grammar and layout. Length is checked before any work. | `parse` 8.7 ns |
| T13 | A data race in the slot | A state machine with `Acquire` and `Release`, modelled by loom. `unsafe` exists in `src/facade/slot.rs` and nowhere else. | none beyond the atomics |
| T14 | Formatting forced on a hot path by an event nobody wants | Events carry `fmt::Arguments` and borrowed errors. Nothing formats until a reporter does. | none |
| T15 | Private text promoted to public through a record | `ErrorRecord.public` comes only from a public message or the category's sentence. `Display` output goes only into `chain[..].message`. | none |
| T16 | A project uses platform kind bits for its own meaning | `Kinds::project::<N>()` reaches bits 32 to 63 only, checked at compile time. Platform constants never change. | none |
| T17 | Locations reveal build paths | `Location` is never part of a public message. `ErrorRecord.trail` is private data. | none |

## The tests

Every threat has an attack test. Each one drives the attack, not the feature.

| Id | Tests |
|---|---|
| T1 | `tests/attack.rs`: `test_public_message_never_contains_display_text`, `test_probe_never_carries_display_text` |
| T2 | `tests/attack.rs`: `prop_secret_hides_its_value_under_every_format_spec`. `tests/record.rs`: `test_secret_stays_redacted`. `tests/ui/secret_no_deref.rs` |
| T3 | `tests/attack_facade.rs`: `test_user_text_is_never_a_template`. `tests/ui/notice_not_a_literal.rs`, `tests/ui/bad_message_form.rs` |
| T4 | `tests/attack.rs`: `prop_kinds_text_has_no_control_bytes`, `prop_code_text_has_no_control_bytes`, `test_severity_and_category_text_has_no_control_bytes`, `test_record_marks_a_forged_log_line`. Fuzz: `kinds_display`, `code_parse`, `code_from_u64`, `record_from_chain` |
| T5 | `tests/attack_memory.rs`: `test_cyclic_gigabyte_error_stays_inside_the_limits`, `test_cyclic_error_ends_the_code_walk`. Fuzz: `record_from_chain` |
| T6 | `tests/attack_facade.rs`: `test_warn_flood_delivers_once`. `tests/facade.rs`: `test_min_severity_filters_before_formatting`. `tests/stress.rs` |
| T7 | `tests/attack_hijack.rs`: `test_hijacking_install_is_named`. `tests/attack_facade.rs`: `test_installed_reporter_cannot_be_replaced` |
| T8 | `tests/facade_guards.rs`: `test_reentrant_verbs_are_dropped_and_panics_are_contained` |
| T9 | `tests/facade_guards.rs`, as above. `tests/attack_panic.rs`: `test_panicking_probe_never_reaches_the_caller` |
| T10 | `tests/ui/duplicate_code.rs`, `duplicate_code_across_types.rs`, `retired_code_reused.rs`. `tests/define_meta.rs`: `check_unique` cases |
| T11 | `tests/attack.rs`: `test_catalog_detects_a_claimed_code_under_another_name`, `test_every_header_byte_is_validated`, `test_out_of_range_fields_are_refused`, `test_oversized_probe_is_refused`. `tests/attack_facade.rs`: `test_forged_security_claim_keeps_its_forged_name`, `test_forged_severity_can_only_drop_its_own_event`, `test_malformed_probe_loses_its_claims`. Fuzz: `probe_decode` |
| T12 | Fuzz: `code_parse`, `code_from_u64`. `tests/properties.rs` |
| T13 | `tests/loom.rs`: every model. The whole suite under Miri with strict provenance |
| T14 | `tests/facade.rs`: `test_min_severity_filters_before_formatting`. `tests/facade_empty.rs` |
| T15 | `tests/attack.rs`: `test_record_public_never_takes_inner_display_text`, `test_record_of_a_foreign_chain_uses_the_category_sentence` |
| T16 | `tests/ui/project_kind_out_of_range.rs`. `src/kinds.rs`: `test_kinds_project_bits` |
| T17 | `tests/attack_facade.rs`: `test_location_stays_out_of_public_text` |

## Forged probes

**A code identifies an error. It does not authenticate one.** The probe is text from `Error::description()`, and any type can return any text. Treat a code, a severity, and a kind on a foreign error as a claim.

What is defended:

- `identify` validates every field, and a probe with any bad byte is not an identity at all. The error is treated as foreign: no code, category `Unknown`, severity `ERROR`, kinds `ERROR`. It keeps none of what it claimed.
- A probe longer than any real one is refused without being scanned.
- A valid probe claiming another crate's code is caught by comparing names, which is the check an error service runs before it acts on a code from an untrusted source:

```rust
use define_err::{catalog, identify};

let identity = identify(error)?;
let entry = catalog::find(&[DbError::CATALOG], identity.code?)?;
let trusted = entry.type_name == identity.type_name;
```

What is not defended:

- A forger who copies the type name as well passes that check. Rust gives this crate no way to tell the two apart, because the claim comes from code in the same process. `error.downcast_ref::<DbError>()` proves the type when the caller knows it.
- **Do not act on `Kinds::SECURITY` from a foreign error without one of these checks.** A forged probe can claim it, so an attacker can raise noise.
- **Severity filtering is not a security boundary.** A forged probe can state a low severity and be dropped by a minimum severity, whether or not its other fields are valid. It drops only its own event; it cannot reach another error or the reporter.

## What else is not defended

- **`Secret<T>` redacts; it does not wipe.** The value stays in memory until dropped, and it can be moved out. Wiping cannot be guaranteed without `unsafe`, which this crate keeps to one module. Use a crate built for key material when that matters.
- **Text in a message is the caller's.** A value placed in a `Display` template without `Secret` is printed. The defence is the type of the field, not a scan of the text.
- **Sinks escape.** A record keeps control characters as written and marks them. Whatever writes a log line must escape it.
- **`Display` runs the error's own code.** `ErrorRecord::from_error` calls it, so a `Display` that panics unwinds into the caller of `from_error`, and one that never returns hangs it. The limits bound memory, not time. Inside `receive`, the guard contains the panic.
- **Without `std`, or under `panic = "abort"`, there is no panic guard.** See [`PLATFORM-NOTES.md`](./PLATFORM-NOTES.md).
- **The boot rule is the application's job.** The slot names who installed first. Only the application can refuse to start.
- **Availability of the reporter.** A reporter that blocks stalls its callers. The contract in [`FACADE.md`](./FACADE.md) says to queue. Storm control belongs to the error service.

## How this is checked

Before each release: the attack tests above, the fuzz targets, loom, Miri, and a stress and soak of every verb from 64 threads with exact counts and flat memory. CI runs all of them except the long fuzz and the soak on every change, and builds every fuzz target.
