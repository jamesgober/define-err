use define_err::{define_errors, Location};

define_errors! {
    pub enum E {
        A { #[location] first: Location, #[location] second: Location } => "a",
    }
}

fn main() {}
