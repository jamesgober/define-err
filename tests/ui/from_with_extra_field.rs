use define_err::define_errors;

define_errors! {
    pub enum E {
        A { #[from] source: std::io::Error, path: String } => "a {path}",
    }
}

fn main() {}
