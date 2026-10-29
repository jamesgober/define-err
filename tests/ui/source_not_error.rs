use define_err::define_errors;

define_errors! {
    pub enum E {
        A { source: String } => "a",
    }
}

fn main() {}
