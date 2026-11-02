use define_err::define_errors;

define_errors! {
    pub enum E {
        A(#[cfg(any())] u8) => "a",
    }
}

fn main() {}
