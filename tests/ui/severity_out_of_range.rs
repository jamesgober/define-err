use define_err::define_errors;

define_errors! {
    pub enum E {
        #[severity(25)]
        A => "a",
    }
}

fn main() {}
