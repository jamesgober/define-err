use define_err::define_errors;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(1, help = "x", deprecated = "2.0", replacement = 5)]
        A => "a",
    }
}

fn main() {}
