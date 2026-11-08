use define_err::define_errors;

define_errors! {
    pub enum E {
        A(#[from] a: std::io::Error, #[from] b: core::fmt::Error) => "a",
    }
}

fn main() {}
