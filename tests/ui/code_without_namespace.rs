use define_err::define_errors;

define_errors! {
    pub enum E {
        #[code(1, help = "x")]
        A => "a",
    }
}

fn main() {}
