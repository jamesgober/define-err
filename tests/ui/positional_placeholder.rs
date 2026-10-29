use define_err::define_errors;

define_errors! {
    pub enum E {
        A(u8) => "value {0}",
    }
}

fn main() {}
