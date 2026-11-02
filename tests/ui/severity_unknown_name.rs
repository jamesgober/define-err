use define_err::define_errors;

define_errors! {
    pub enum E {
        #[severity(LOUD)]
        A => "a",
    }
}

fn main() {}
