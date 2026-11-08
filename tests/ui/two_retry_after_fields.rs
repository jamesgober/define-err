use define_err::define_errors;

define_errors! {
    pub enum E {
        A { #[retry_after] a: core::time::Duration, #[retry_after] b: core::time::Duration } => "a",
    }
}

fn main() {}
