use define_err::define_errors;

define_errors! {
    #[category(Internal)]
    pub enum E {
        A => "a",
    }
}

fn main() {}
