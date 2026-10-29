use define_err::define_errors;

define_errors! {
    pub enum E {
        #[retry_after(secs = 1)]
        A { #[retry_after] wait: core::time::Duration } => "a",
    }
}

fn main() {}
