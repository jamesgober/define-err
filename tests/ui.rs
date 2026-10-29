//! Compile-fail tests: every misuse of `define_errors!` fails with a message
//! that names the problem and the fix (FR-28, NFR-12).
//!
//! Compiler messages change between releases, so the expected output in
//! `tests/ui/*.stderr` is pinned to one toolchain. CI runs this test in its
//! own job on that toolchain with `DEFINE_ERR_UI=1`; elsewhere it is skipped.

#[test]
fn test_compile_errors_name_the_problem_and_the_fix() {
    if std::env::var_os("DEFINE_ERR_UI").is_none() {
        eprintln!(
            "skipped: set DEFINE_ERR_UI=1 on the pinned toolchain to run the compile-fail tests"
        );
        return;
    }
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
