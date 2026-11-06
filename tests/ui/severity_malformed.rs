use define_err::define_errors;

define_errors! {
    pub enum E {
        #[severity(1 + 2)]
        A => "a",
    }
}

fn main() {}
