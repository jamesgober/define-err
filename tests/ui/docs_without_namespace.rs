use define_err::define_errors;

define_errors! {
    docs = "https://docs.example.com/{code}";
    pub enum E {
        A => "a",
    }
}

fn main() {}
