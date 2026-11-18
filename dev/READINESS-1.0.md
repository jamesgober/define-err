# define-err - 1.0 readiness

> Where the crate stands against the definition of done (`PLAN.md` section 16), and everything accepted, deferred, or missed in the 0.x series that is still open. Kept current through 0.x; nothing listed here becomes final silently. Last updated 2026-11-18, at 0.10.0.

## What is left before 1.0

In order. Each step is small and ends in a commit, a release, or a decision.

| # | Step | Who | State |
|---|---|---|---|
| L-1 | Move the two migration branches to 0.10: noml-rust (draft PR 40) and mod-cli (draft PR 1) pin `"0.9"`, and report-err 0.9.0 depends on `^0.10`, so a build with both would hold two facades that Cargo cannot refuse (0.9 does not set `links`). Both use only `define_errors!`; the change is the version line. | this crate's owner | open |
| L-2 | Re-run the 24-hour fuzz of the targets whose code changed in 0.10: `probe_decode` (the `Display` check, the retired public text) and `record_from_chain` (bidirectional controls). | this crate's owner | open; the other five targets are unchanged since D-066 |
| L-3 | Re-run the 8-hour soak of every verb from 64 threads: the kinds mask changed in 0.10 (D-093). loom models the new mask; the soak checks it at scale. | this crate's owner | open |
| L-4 | The performance pass on a quiet machine (ROADMAP 0.9 polish): every figure since 0.6 was taken at a load average of 47 to 130. | this crate's owner | open; needs a quiet machine, not more code |
| L-5 | L2 (the attribute front end) builds against `docs/GENERATED-CODE.md` and passes the shared conformance suite. | the L2 owner | open: L2 is not started; `__private` changed in 0.10 (`ProbeBuf::new`) and is in `GENERATED-CODE.md` |
| L-6 | L3 reads codes through the probe on the 0.10 line: report-err 0.9.0 depends on `^0.10`. | the L3 owner | done for the dependency; its migration away from `Identity::public` is report-err's |
| L-7 | Two weeks of use with no API change. The last change was 0.10.0 on 2026-11-12, so the earliest date is 2026-11-26. | elapsed time | open |
| L-8 | The design of the 1.0 facade sharing (D-092): which version holds the statics, how later majors forward to it, and that only it keeps `links`. Written into `docs/STABILITY-1.0.md` before 1.0, because it binds 2.0. | this crate's owner | open |
| L-9 | J-1 to J-4 below. | James | open |
| L-10 | The full audit recorded in `docs/STABILITY-1.0.md`, then the release routine for 1.0.0. | this crate's owner, after J-3 | open |

## Decisions for James

| # | Decision | Recommendation |
|---|---|---|
| J-1 | config-lib's path (`PLAN.md` 19.4): it is published at 1.1.0 with MSRV 1.75, and define-err needs 1.81, so moving it is an MSRV raise after 1.0. | (b): stay on thiserror in 1.x and migrate in config-lib 2.0, batched with its next planned breaking change. |
| J-2 | The pre-1.0 review, before the API is frozen. | Review `docs/API.md` and `docs/GENERATED-CODE.md` at 0.10.0. The simplicity audit (`PLAN.md` D-089) found the API passes; the one duplicate it kept, `Namespace::code` (the same as `Code::new`), is yours to keep or drop. Since then 0.10 removed `Identity::public` and `has_public`, added `catalog::Catalog` and `would_deliver`, and set `links`, all from the independent review (D-091 to D-096). |
| J-3 | The go for 1.0.0. | Only after L-1 to L-8 are done and every line below is done or carries a reason you accept. |
| J-4 | The support window in `docs/STABILITY-1.0.md`: how long 1.x gets fixes once 2.0 is out. It is a public commitment. | Security and soundness fixes for 1.x for 12 months after 2.0.0 is published, on the 1.x minimum Rust version. |

## Definition of done, line by line

| Line (`PLAN.md` 16) | State |
|---|---|
| 1. P0 to P2 items implemented, documented, tested | Done at 0.5; re-checked in the 0.9 audit. |
| 2. Every FR and NFR has a named test (`tests/TRACEABILITY.md`) | Done in 0.9: every requirement is mapped. NFR-2, NFR-4, and NFR-5 are checked by measurement and NFR-13 by review, not by CI. |
| 3. E-1 to E-100 pass | Done; the 0.9 edge-case review re-runs them and the cases added since. |
| 4. T1 to T18 attack tests, costs stated | Done at 0.7; they pass on every push. The 0.9 review of the run-time code changed since 0.7 (`short_file`, path display, the code checks) found D-081, fixed. The independent review of 0.9.3 found two medium and three low findings and three notes, all fixed in 0.10 (D-091 to D-096); T18 (two copies of the crate) is new. |
| 5. Fuzz, 24 hours per target, no open finding | Done at 0.7 (D-066). The one new run-time parser since, `short_file`, has a property test (`prop_short_file_is_a_suffix_and_keeps_relative_paths`) in place of a fuzz target. Two targets changed in 0.10 and need their 24 hours again (L-2). |
| 6. loom, Miri, 8-hour soak | loom and Miri pass on every push, the new mask models included (0.10). The soak is from 0.7 and is re-run for the new mask (L-3). |
| 7. Counting-allocator proofs | Done. |
| 8. `docs/PERFORMANCE.md`; build and facade 20% faster or a reviewed reason | Partly: the misses below are reviewed (D-060) and open for the 0.9 tuning pass. |
| 9. Zero runtime dependencies; deny and audit clean | Done; checked by CI on every push. |
| 10. `unsafe` only in `src/facade/slot.rs` | Done; five lines, checked by `tools/invariants.py` (D-080). |
| 11. CI green, every platform and mode | Done for every release so far. |
| 12. Documents complete, no banned words | Re-checked in the 0.9 audit. |
| 13. L2 passes the shared conformance suite; L3 reads codes through the probe | L3 is on 0.10 (L-6); L2 is open (L-5). |
| 14. Migrated crates green on the release candidate | page-db, the internal consumer's two crates, the downstream compiler, noml-rust, and mod-cli have draft PRs, which move to the release candidate's version before 1.0 (noml-rust and mod-cli first, L-1); config-lib waits on J-1. |
| 15. Release routine for 1.0.0 | After J-3. |

## Open items from the 0.x series

| Item | Source | State |
|---|---|---|
| `Code` `Display` 24 ns against a 20 ns target | D-060, D-084 | Reduced in 0.9.1 (about 10%). Kept: `write!` of a short `&str` is 10.5 ns on its own, and the safe `from_utf8` check (4.5 ns) stays, since `unsafe` stays in one module. |
| `warn()` after first delivery, 9.5 ns against 5 | D-060, D-084 | Kept: a cheaper key could deliver a site twice or drop a warning on a collision; `warn!` (0.5 ns) is the fast form. |
| `report` filtered, 5.9 ns against 3 | D-060, D-084 | Kept: the cost is the panic guard around the error's own code (D-062); `report_with` is 0.64 ns. |
| `identify` generated, 38 ns against 10 (26 before the 0.10 `Display` check); `report` to a no-op, 33 ns against 10 before 0.10 | D-060, D-091 | Kept unless a faster validation keeps the same checks: a hostile probe is validated once. |
| `log::warn!` comparison at level `Off` | D-060 | Kept: both are one load and a branch; the 20% target cannot be met. |
| Incremental rebuild slower than thiserror on one crate | D-060 | Reduced in 0.9: a plain enum expands in one pass, 28% faster on 500 variants (D-082); a per-variant `const` row was slower (D-077). Still slower than thiserror; types with metadata keep five passes. |
| 500-variant rebuild +11% from path display | D-067 | Kept for thiserror parity; re-measured with the tuning pass. |
| Early severity read drops a malformed low-severity probe | D-063 | Kept: severity filtering is not a security boundary (`docs/SECURITY.md`). |
| Codes of a compiled-out type still count in its block's duplicate check | D-072 | Kept: errs towards a false duplicate, never a missed one. |
| A crate past 1.0 takes define-err as a public dependency | D-070, D-092 | Policy recorded; the semver trick is carried out at 1.0, so a 1.x and a 2.x share one facade. Until then `links` refuses a second copy from 0.10 on. |
| The `Display` check costs `identify` 16 ns | D-091 | Kept: the check is what refuses outside text shaped like a probe on Rust 1.81 to 1.89; it stays off the filtered `report` path. |
| A forged low severity in outside text can drop its own event | D-063, D-091 | Kept: severity filtering is not a security boundary, and the check would cost every filtered `report` 16 ns. The event claims nothing when it is delivered. |
| The downstream compiler's minimum Rust rises to 1.81; its existing lint findings | 19.4, D-076 | The owner's call, stated in the PR. |
| Source size: 5,706 lines of code against a 5,000 target | NFR-14, D-080, D-087, D-088, D-091 to D-096 | Open: under the 6,000 at which OD-6 is reopened; reviewed in the 1.0 audit. |
