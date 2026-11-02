use define_err::define_errors;

define_errors! {
    pub enum E {
        #[kinds(ERROR | BILLING)]
        A => "a",
    }
}

fn main() {}
