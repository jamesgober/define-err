use define_err::define_errors;

define_errors! {
    #[derive(Debug)]
    pub enum E {
        A => "a",
    }
}

fn main() {}
