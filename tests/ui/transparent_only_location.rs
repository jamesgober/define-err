use define_err::define_errors;

use define_err::Location;

define_errors! {
    pub enum E {
        A { #[location] at: Location } => transparent,
    }
}

fn main() {}
