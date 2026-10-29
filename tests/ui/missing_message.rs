use define_err::define_errors;

define_errors! {
    pub enum E {
        A { x: u8 },
    }
}

fn main() {}
