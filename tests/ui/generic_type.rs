use define_err::define_errors;

define_errors! {
    pub enum E<T> {
        Bad(inner: T) => "bad",
    }
}

define_errors! {
    pub struct S<'a> {
        name: &'a str,
    } => "bad {name}";
}

fn main() {}
