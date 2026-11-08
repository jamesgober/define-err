use define_err::define_errors;

define_errors! {
    pub enum E {
        #[category("unavailable")]
        A => "a",
    }
}

fn main() {}
