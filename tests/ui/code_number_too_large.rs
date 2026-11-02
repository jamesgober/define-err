use define_err::define_errors;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(16_777_216, help = "x")]
        A => "a",
    }
}

fn main() {}
