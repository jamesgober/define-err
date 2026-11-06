use define_err::define_errors;

define_errors! {
    pub enum E {
        #[kinds("error")]
        A => "a",
    }
}

fn main() {}
