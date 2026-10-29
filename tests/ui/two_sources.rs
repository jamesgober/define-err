use define_err::define_errors;

define_errors! {
    pub enum E {
        A { #[source] first: std::io::Error, #[source] second: std::fmt::Error } => "a",
    }
}

fn main() {}
