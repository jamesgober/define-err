use define_err::define_errors;

define_errors! {
    pub enum E {
        A => "a",
        #[category(Internal)]
    }
}

fn main() {}
