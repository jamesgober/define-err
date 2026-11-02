use define_err::define_errors;

define_errors! {
    pub enum E {
        #[category(Broken)]
        A => "a",
    }
}

fn main() {}
