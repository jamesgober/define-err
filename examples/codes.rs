//! Stable codes, categories, and the catalog.
//!
//! ```text
//! cargo run --example codes
//! ```

use define_err::{prelude::*, CodeState};

define_errors! {
    namespace = "DB";
    docs = "https://docs.example.com/errors/{code}";
    retired = [9];

    pub enum DbError {
        #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        #[retry_after(secs = 2)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt { page: u64 } => "page {page} failed its checksum",
    }
}

fn main() {
    let err = DbError::Timeout { after_ms: 5000 };
    println!("{} ({err})", err.code().unwrap());
    println!(
        "HTTP {}, gRPC {}, exit {}",
        err.category().http_status(),
        err.category().grpc_code(),
        err.category().exit_code()
    );
    println!("client sees: {}", err.public_message());
    println!("retry after: {:?}", err.retry_after());

    println!();
    for entry in DbError::CATALOG {
        match entry.url() {
            Some(url) if entry.state == CodeState::Active => {
                println!("{}  {}  {url}", entry.code, entry.help)
            }
            _ => println!("{}  ({})", entry.code, entry.title),
        }
    }

    // Any library can read the same facts through `&dyn Error`.
    let boxed: Box<dyn std::error::Error> = Box::new(DbError::Corrupt { page: 7 });
    let id = define_err::identify(&*boxed).unwrap();
    println!(
        "\n{} is {} at severity {}",
        id.type_name, id.category, id.severity
    );
}
