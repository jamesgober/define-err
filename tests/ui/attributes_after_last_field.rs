use define_err::define_errors;

define_errors! {
    pub enum E {
        A { a: u32, #[source] } => "a",
    }
}

fn main() {}
