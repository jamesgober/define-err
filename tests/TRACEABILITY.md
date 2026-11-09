# Requirement to test map

Each requirement of `dev/PLAN.md` section 5 and the tests that check it (definition of done, line 2). `tests/ui/*` cases run in the CI `ui` job (pinned toolchain, `DEFINE_ERR_UI=1`); `tests/loom.rs` runs under `--cfg loom` in the `loom` job; `tools/invariants.py` runs in the `security` job.

| Requirement | Checked by | Note |
|---|---|---|
| FR-1 plain type generates exactly its items | `define_basic.rs::test_debug_is_derived`, `define_meta.rs::test_plain_arm_answers_as_the_general_arm_does`, `alloc_free.rs::test_construction_display_source_info_allocate_nothing` | |
| FR-2 compile-time messages, named and tuple bindings | `define_basic.rs::test_display_named_fields_with_format_specs`, `define_basic.rs::test_display_tuple_bindings_and_unnamed_fields`, `define_basic.rs::test_display_extra_arguments_and_field_expressions` | |
| FR-3 `#[from]`, `#[track_caller]`, location | `define_basic.rs::test_from_conversion_through_question_mark_records_line`, `define_structs.rs::test_struct_from_fills_location`, `ui/from_with_extra_field.rs` | |
| FR-4 sources, `Option<E>`, boxed | `define_basic.rs::test_source_attribute_and_named_source`, `boxed_sources.rs::test_boxed_sources_return_the_inner_error`, `src/private.rs::test_source_of_option_none_and_some` | |
| FR-5 `transparent` | `define_basic.rs::test_transparent_forwards_display_and_source`, `conformance.rs::test_conformance_transparent` | |
| FR-6 `#[non_exhaustive]` unless `#[exhaustive]` | `ui/non_exhaustive_downstream.rs`, `define_basic.rs::test_non_exhaustive_binds_downstream_and_exhaustive_opts_out`, `define_structs.rs::test_structs_are_not_non_exhaustive` | From a second crate, `tests/fixture` (D-079). |
| FR-7 optional metadata costs nothing when absent | `define_meta.rs::test_plain_arm_answers_as_the_general_arm_does`, `traits.rs::test_size_and_alignment_equal_hand_written_and_thiserror`, `alloc_free.rs::test_codes_metadata_and_probe_allocate_nothing` | Generated code size is measured as build time (`docs/PERFORMANCE.md`), not by a test. |
| FR-8 `Code` layout, text, width | `src/code.rs::test_code_parse_valid_forms_equal_by_identity`, `golden.rs::test_golden_codes_pack_parse_and_display`, `properties.rs::prop_code_round_trips_through_u64_and_text` | |
| FR-9 duplicate codes are a compile error | `ui/duplicate_code.rs`, `ui/retired_code_reused.rs`, `ui/duplicate_code_across_types.rs`, `src/private.rs::test_check_block_finds_a_duplicate_across_blocks` | Blocks of more than 2,048 codes by unit test (D-088). |
| FR-10 a code needs help | `ui/code_without_help.rs`, `ui/codes_without_help.rs` | |
| FR-11 `CATALOG` | `define_meta.rs::test_catalog_entries_titles_docs_and_retired`, `define_meta.rs::test_code_info_matches_catalog` | |
| FR-12 deprecated codes | `define_meta.rs::test_deprecated_code_state_and_constant`, `ui/deprecated_code_used.rs` | |
| FR-13 `Severity` | `src/severity.rs::test_severity_display_label_or_otel_name`, `ui/severity_out_of_range.rs`, `golden.rs::test_golden_severity_names` | |
| FR-14 `Category` tables | `golden.rs::test_golden_categories_tables`, `define_meta.rs::test_categories_and_their_defaults`, `src/category.rs::test_category_grpc_round_trip` | |
| FR-15 `Kinds` bits | `src/kinds.rs::test_kinds_project_bits`, `ui/project_kind_out_of_range.rs`, `golden.rs::test_golden_kind_bits` | |
| FR-16 retry metadata | `define_meta.rs::test_retry_metadata`, `src/category.rs::test_category_never_retries_non_idempotent` | |
| FR-17 `Secret<T>` | `src/secret.rs::test_secret_display_plain_redacted`, `attack.rs::prop_secret_hides_its_value_under_every_format_spec`, `ui/secret_no_deref.rs` | |
| FR-18 `set_reporter` once | `facade.rs::test_second_install_names_the_first`, `attack_hijack.rs::test_hijacking_install_is_named`, `loom.rs::loom_loser_location` | |
| FR-19 verbs and macros, `#[track_caller]` | `facade.rs::test_track_caller_helpers`, `facade.rs::test_macros_report_the_outermost_call_site`, `facade.rs::test_notice_macro_and_extra_kinds` | |
| FR-20 nothing installed: one load | `facade_empty.rs::test_nothing_installed_then_first_install`, `alloc_free.rs::test_verbs_allocate_nothing` | The single load is shown by `tools/asm` (NFR-4). |
| FR-21 warn once per site, modes | `facade.rs::test_warn_in_a_loop_delivers_once`, `facade.rs::test_mode_always_then_once`, `facade.rs::test_mode_ignore`, `facade_empty.rs::test_nothing_installed_then_first_install` | |
| FR-22 `soft` | `facade.rs::test_soft_delivers_a_warning_that_carries_the_error`, `facade.rs::test_mode_error_makes_soft_fail` | |
| FR-23 no reentry, panics contained | `facade_guards.rs::test_reentrant_verbs_are_dropped_and_panics_are_contained`, `attack_panic.rs::test_panicking_probe_never_reaches_the_caller` | |
| FR-24 `ErrorRecord` | `record.rs::test_record_is_send_sync_clone`, `record.rs::test_total_limit_bounds_all_text`, `record.rs::test_setters` | |
| FR-25 `identify` | `define_meta.rs::test_identify_reads_every_generated_variant`, `define_meta.rs::test_innermost_code_walks_the_chain`, `edge_codes.rs::test_e89_innermost_code_depth_limit` | |
| FR-26 four feature modes, hosted and embedded | CI `test` job (four modes), CI `no-std` job, `src/lib.rs::test_core_only_build_has_no_alloc` | Embedded targets are built, not run. |
| FR-27 absolute paths only | `no_prelude.rs::test_generated_code_without_prelude` | |
| FR-28 compile errors name the fix | `ui.rs::test_compile_errors_name_the_problem_and_the_fix` (every case in `tests/ui`) | |
| NFR-1 zero runtime dependencies | `tools/invariants.py` (`cargo tree`, four modes), CI `security` job (`cargo deny check`) | |
| NFR-2 same size and `Display` speed as hand-written | `traits.rs::test_size_and_alignment_equal_hand_written_and_thiserror`, `benches/definitions.rs` | Speed by benchmark, not in CI. |
| NFR-3 no allocation outside the record | `alloc_free.rs::test_construction_display_source_info_allocate_nothing`, `alloc_free.rs::test_verbs_allocate_nothing`, `alloc_free.rs::test_record_allocation_count` | |
| NFR-4 facade with nothing installed | `benches/facade.rs`, `benches/floor.rs`, `tools/asm` | By benchmark and assembly, not in CI. |
| NFR-5 clean build faster than thiserror | `tools/build-time/compare.py` | By measurement, not in CI. |
| NFR-6 500 variants at the default `recursion_limit` | `big_enum.rs::test_big_enum_first_and_last_variants` (canonical forms), `big_enum_any_order.rs::test_any_order_enum_first_and_last_variants` (attributes in any order, unnamed tuple fields) | |
| NFR-7 no run-time panic | `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, ...))]` in `src/lib.rs`, CI `test` job (Clippy, `-D warnings`) | |
| NFR-8 `unsafe` in one module, under 40 lines | `#![deny(unsafe_code)]` in `src/lib.rs`, `tools/invariants.py`, CI `miri` and `loom` jobs | |
| NFR-9 `Send + Sync` | `traits.rs::test_generated_types_are_send_sync_unpin`, `traits.rs::test_auto_traits_follow_fields`, `record.rs::test_record_is_send_sync_clone` | |
| NFR-10 deterministic formats | `golden.rs::test_golden_codes_pack_parse_and_display`, `golden.rs::test_golden_categories_tables`, `golden.rs::test_golden_probe_strings` | |
| NFR-11 `no_std` targets | CI `no-std` job (thumbv7em, thumbv6m, riscv32imac, wasm32) | |
| NFR-12 a compile-fail case per error path | `tools/invariants.py`, CI `ui` job | |
| NFR-13 API fits one page per module | the 1.0 audit (`docs/STABILITY-1.0.md`) | A review, not a test. |
| NFR-14 source size | `tools/invariants.py` | 5,507 lines of code, over the 5,000 target (D-080, D-087, D-088). |
