use define_err::define_errors;

define_errors! {
    pub enum E {
        #[retryable(yes)]
        A => "a",
    }
}

fn main() {}
