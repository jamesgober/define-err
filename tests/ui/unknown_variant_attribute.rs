use define_err::define_errors;

define_errors! {
    pub enum E {
        #[serde(rename = "x")]
        A => "a",
    }
}

fn main() {}
