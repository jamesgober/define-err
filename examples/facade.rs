//! A small reporter, and the verbs any library can call.
//!
//! ```text
//! cargo run --example facade
//! ```

use define_err::{define_errors, ErrorRecord, Event, Kinds, Reporter, Severity};

define_errors! {
    namespace = "APP";

    pub enum AppError {
        #[code(1, help = "Check the path.")]
        #[category(NotFound)]
        Missing { path: String } => "{path} does not exist",

        Io(#[from] source: std::io::Error) => "i/o failed",
    }
}

/// Writes one line per event. A real reporter queues instead of writing: it
/// must return in microseconds and never block.
struct StderrReporter;

impl Reporter for StderrReporter {
    fn receive(&self, event: &Event<'_>) {
        let at = event.location();
        let code = event
            .code()
            .map(|code| code.to_string())
            .unwrap_or_default();
        match (event.error(), event.message()) {
            (Some(_), _) => {
                // A record is owned, bounded, and safe to keep or send.
                let record = ErrorRecord::from_event(event);
                eprintln!(
                    "{:<8} {code} {}:{} {record}",
                    event.severity(),
                    at.file(),
                    at.line()
                );
            }
            (None, Some(message)) => eprintln!(
                "{:<8} {}:{} {message}",
                event.severity(),
                at.file(),
                at.line()
            ),
            (None, None) => {}
        }
    }
}

static REPORTER: StderrReporter = StderrReporter;

fn parse_port(text: &str) -> u16 {
    text.parse().unwrap_or_else(|_| {
        define_err::warn!("port {text:?} is not a number; using 8080"); // once, however often this runs
        8080
    })
}

fn main() {
    // First thing in `main`, and a failure is fatal: see docs/FACADE.md.
    if let Err(err) = define_err::set_reporter(&REPORTER) {
        eprintln!("{err}");
        std::process::exit(70);
    }

    define_err::notice("started");
    let _ = (parse_port("x"), parse_port("y"));
    define_err::deprecated!("pool.max", since = "2.1.0", replacement = "pool.size");
    define_err::trigger!(Kinds::SECURITY, "{} failed logins in one minute", 21);

    let err = AppError::Missing {
        path: "/etc/app.conf".into(),
    };
    define_err::report(&err);
    define_err::report_with(&err, Severity::DEBUG, Kinds::ERROR); // routine here

    let io = std::fs::read("/does/not/exist")
        .map_err(AppError::from)
        .unwrap_err();
    define_err::report(&io);
}
