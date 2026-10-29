use define_err::define_errors;

define_errors! {
    pub enum E {
        A { #[serde(skip)] x: u8 } => "a {x}",
    }
}

fn main() {}
