use define_err::define_errors;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(7, help = "x")]
        A => "a",
    }
    pub enum F {
        #[code(7, help = "y")]
        B => "b",
    }
}

fn main() {}
