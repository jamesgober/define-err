use define_err::define_errors;

define_errors! {
    namespace = "DB";
    docs = "https://docs.hivedb.com/errors";
    pub enum E {
        #[code(1, help = "x")]
        A => "a",
    }
}

fn main() {}
