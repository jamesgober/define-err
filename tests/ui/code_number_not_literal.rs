use define_err::define_errors;

const N: u32 = 1;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(N, help = "x")]
        A => "a",
    }
}

fn main() {}
