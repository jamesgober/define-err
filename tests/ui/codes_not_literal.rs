use define_err::define_errors;

const N: u32 = 1;

// Two codes without a number literal report that, and not a duplicate.
define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(N, help = "a")]
        A => "a",
        #[code(N, help = "b")]
        B => "b",
    }
}

fn main() {}
