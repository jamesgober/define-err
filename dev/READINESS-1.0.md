# define-err - 1.0 readiness

> Where the crate stands against the definition of done (`PLAN.md` section 16), and everything accepted, deferred, or missed in the 0.x series that is still open. Kept current through 0.9; nothing listed here becomes final silently. Last updated 2026-11-08, at 0.9.2.

## Decisions for James

| # | Decision | Recommendation |
|---|---|---|
| J-1 | config-lib's path (`PLAN.md` 19.4): it is published at 1.1.0 with MSRV 1.75, and define-err needs 1.81, so moving it is an MSRV raise after 1.0. | (b): stay on thiserror in 1.x and migrate in config-lib 2.0, batched with its next planned breaking change. |
| J-2 | The pre-1.0 review (ROADMAP 0.9), before the API is frozen. | Review `docs/API.md` and `docs/GENERATED-CODE.md` as they stand at the last 0.9 release candidate. |
| J-3 | The go for 1.0.0. | Only after every line below is done or carries a reason you accept. |

## Definition of done, line by line

| Line (`PLAN.md` 16) | State |
|---|---|
| 1. P0 to P2 items implemented, documented, tested | Done at 0.5; re-checked in the 0.9 audit. |
| 2. Every FR and NFR has a named test (`tests/TRACEABILITY.md`) | Done in 0.9: every requirement is mapped. NFR-2, NFR-4, and NFR-5 are checked by measurement and NFR-13 by review, not by CI. |
| 3. E-1 to E-100 pass | Done; the 0.9 edge-case review re-runs them and the cases added since. |
| 4. T1 to T17 attack tests, costs stated | Done at 0.7; they pass on every push. The 0.9 review of the run-time code changed since 0.7 (`short_file`, path display, the code checks) found D-081, fixed. |
| 5. Fuzz, 24 hours per target, no open finding | Done at 0.7 (D-066). The one new run-time parser since, `short_file`, has a property test (`prop_short_file_is_a_suffix_and_keeps_relative_paths`) in place of a fuzz target. |
| 6. loom, Miri, 8-hour soak | Done at 0.7; Miri also passes over 0.8. |
| 7. Counting-allocator proofs | Done. |
| 8. `docs/PERFORMANCE.md`; build and facade 20% faster or a reviewed reason | Partly: the misses below are reviewed (D-060) and open for the 0.9 tuning pass. |
| 9. Zero runtime dependencies; deny and audit clean | Done; checked by CI on every push. |
| 10. `unsafe` only in `src/facade/slot.rs` | Done; five lines, checked by `tools/invariants.py` (D-080). |
| 11. CI green, every platform and mode | Done for every release so far. |
| 12. Documents complete, no banned words | Re-checked in the 0.9 audit. |
| 13. L2 passes the shared conformance suite; L3 reads codes through the probe | Open: waits on L2 and L3 (ROADMAP 0.9). |
| 14. Migrated crates green on the release candidate | page-db, the internal consumer's two crates, kraken, noml-rust, and mod-cli have draft PRs; config-lib waits on J-1. |
| 15. Release routine for 1.0.0 | After J-3. |

## Open items from the 0.x series

| Item | Source | State |
|---|---|---|
| `Code` `Display` 24 ns against a 20 ns target | D-060, D-084 | Reduced in 0.9.1 (about 10%). Kept: `write!` of a short `&str` is 10.5 ns on its own, and the safe `from_utf8` check (4.5 ns) stays, since `unsafe` stays in one module. |
| `warn()` after first delivery, 9.5 ns against 5 | D-060, D-084 | Kept: a cheaper key could deliver a site twice or drop a warning on a collision; `warn!` (0.5 ns) is the fast form. |
| `report` filtered, 5.9 ns against 3 | D-060, D-084 | Kept: the cost is the panic guard around the error's own code (D-062); `report_with` is 0.64 ns. |
| `identify` generated, 26 ns against 10; `report` to a no-op, 33 ns against 10 | D-060 | Kept unless a faster validation keeps the same checks: a hostile probe is validated once. |
| `log::warn!` comparison at level `Off` | D-060 | Kept: both are one load and a branch; the 20% target cannot be met. |
| Incremental rebuild slower than thiserror on one crate | D-060 | Reduced in 0.9: a plain enum expands in one pass, 28% faster on 500 variants (D-082); a per-variant `const` row was slower (D-077). Still slower than thiserror; types with metadata keep five passes. |
| 500-variant rebuild +11% from path display | D-067 | Kept for thiserror parity; re-measured with the tuning pass. |
| Early severity read drops a malformed low-severity probe | D-063 | Kept: severity filtering is not a security boundary (`docs/SECURITY.md`). |
| Codes of a compiled-out type still count in its block's duplicate check | D-072 | Kept: errs towards a false duplicate, never a missed one. |
| A crate past 1.0 takes define-err as a public dependency | D-070 | Policy recorded; the semver trick is carried out at 1.0. |
| kraken's minimum Rust rises to 1.81; its existing lint findings | 19.4, D-076 | The owner's call, stated in the PR. |
| Source size: 5,507 lines of code against a 5,000 target | NFR-14, D-080, D-087, D-088 | Open: under the 6,000 at which OD-6 is reopened; reviewed in the 1.0 audit. |
