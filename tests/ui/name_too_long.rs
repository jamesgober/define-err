use define_err::define_errors;

// `T::` plus a 126-byte variant name: 129 bytes, one more than a probe carries.
define_errors! {
    pub enum T {
        Vaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa => "long",
    }
}

fn main() {}
