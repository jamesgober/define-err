use define_err::define_errors;

define_errors! {
    #[manual_debug]
    pub enum E {
        A => "a",
    }
}

fn main() {}
