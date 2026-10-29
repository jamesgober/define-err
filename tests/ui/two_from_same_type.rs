use define_err::define_errors;

define_errors! {
    pub enum E {
        A(#[from] std::io::Error) => "a",
        B(#[from] std::io::Error) => "b",
    }
}

fn main() {}
