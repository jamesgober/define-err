use define_err::define_errors;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(1, help = "x")]
        A => "a",
        #[code(1, help = "y")]
        B => "b",
    }
}

fn main() {}
