use define_err::define_errors;

define_errors! {
    pub enum E {
        A(first: std::io::Error, second: std::fmt::Error) => transparent,
    }
}

fn main() {}
