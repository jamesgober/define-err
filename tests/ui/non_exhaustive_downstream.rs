// FR-6: a generated enum is `#[non_exhaustive]`, so a crate that uses it
// must keep a wildcard arm.

fn main() {
    let error = define_err_fixture::Open::A;
    match error {
        define_err_fixture::Open::A => {}
        define_err_fixture::Open::B => {}
    }
}
