use define_err::define_errors;

define_errors! {
    namespace = "DB";
    retired = [1];
    retired = [2];
    pub enum E {
        A => "a",
    }
}

fn main() {}
