#![deny(deprecated)]

use define_err::define_errors;

define_errors! {
    namespace = "DB";
    pub enum E {
        #[code(4, name = OLD, help = "Use DB-0010.", deprecated = "2.1.0", replacement = 10)]
        Old => "old",
        #[code(10, help = "New.")]
        New => "new",
    }
}

fn main() {
    let _ = E::OLD;
}
