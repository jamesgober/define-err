use define_err::define_errors;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(4_294_967_295, help = "x")]
        A => "a",
    }
}

fn main() {}
