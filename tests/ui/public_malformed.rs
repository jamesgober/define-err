use define_err::define_errors;

define_errors! {
    pub enum E {
        #[public(busy)]
        A => "a",
    }
}

fn main() {}
