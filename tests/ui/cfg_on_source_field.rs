use define_err::define_errors;

define_errors! {
    pub enum E {
        A { #[cfg(any())] #[source] cause: std::fmt::Error } => "a",
    }
}

fn main() {}
