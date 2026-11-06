use define_err::define_errors;

define_errors! {
    pub enum E {
        A { #[from] #[source] error: std::io::Error } => "a",
    }
}

fn main() {}
