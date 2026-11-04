use define_err::define_errors;

// Two invalid codes report what is wrong with each, and not a duplicate.
define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(1)]
        A => "a",
        #[code(3)]
        B => "b",
    }
}

fn main() {}
