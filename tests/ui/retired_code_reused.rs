use define_err::define_errors;

define_errors! {
    namespace = "DB";
    retired = [9];
    pub enum E {
        #[code(9, help = "x")]
        A => "a",
    }
}

fn main() {}
