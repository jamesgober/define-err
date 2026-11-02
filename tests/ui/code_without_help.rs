use define_err::define_errors;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(1)]
        A => "a",
    }
}

fn main() {}
