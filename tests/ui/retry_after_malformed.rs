use define_err::define_errors;

define_errors! {
    pub enum E {
        #[retry_after(minutes = 1)]
        A => "a",
    }
}

fn main() {}
