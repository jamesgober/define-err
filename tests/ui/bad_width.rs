use define_err::define_errors;

define_errors! {
    namespace = "DB", width = 9;
    pub enum E {
        #[code(1, help = "x")]
        A => "a",
    }
}

fn main() {}
