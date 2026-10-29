use define_err::define_errors;

define_errors! {
    pub enum E {
        A(u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) => "a",
    }
}

fn main() {}
