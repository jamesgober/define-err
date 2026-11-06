use define_err::define_errors;

define_errors! {
    pub enum E {
        A { 42 } => "a",
    }
}

fn main() {}
