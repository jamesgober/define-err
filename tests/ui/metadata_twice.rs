use define_err::define_errors;

define_errors! {
    pub enum E {
        #[category(Internal)]
        #[category(Unavailable)]
        A => "a",
    }
}

fn main() {}
