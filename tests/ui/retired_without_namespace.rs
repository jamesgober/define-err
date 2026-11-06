use define_err::define_errors;

define_errors! {
    retired = [1];
    pub enum E {
        A => "a",
    }
}

fn main() {}
