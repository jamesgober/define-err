//! Attack tests for the threats that need a reporter (`dev/PLAN.md` 10.1).

#![cfg(feature = "alloc")]

mod common;

use std::{error::Error, fmt, thread};

use common::{serial, RECORDER};
use define_err::{define_errors, ErrorInfo, ErrorRecord, Event, Kinds, Reporter, Severity};

define_errors! {
    namespace = "AT";

    pub enum Audit {
        #[code(1, help = "Review the audit log.")]
        #[category(PermissionDenied)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Breach => "login from a revoked key",
    }
}

/// A foreign error that states any probe it likes.
#[derive(Debug)]
struct Forged(String);

impl fmt::Display for Forged {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("forged")
    }
}

impl Error for Forged {
    fn description(&self) -> &str {
        &self.0
    }
}

#[allow(deprecated)]
fn probe_of(error: &Audit) -> String {
    error.description().to_owned()
}

/// T3: a template in user text stays text, in every text verb.
#[test]
fn test_user_text_is_never_a_template() {
    let _serial = serial();
    let text = "{password} {0} {:?} %s";
    define_err::notice(text);
    define_err::warn(text);
    define_err::trigger(Kinds::SECURITY, text);
    let events = RECORDER.take();
    assert_eq!(events.len(), 3);
    for event in events {
        assert_eq!(event.message, text);
    }
}

/// T6: one site, 64 threads, ten million calls each, one delivery.
#[test]
fn test_warn_flood_delivers_once() {
    fn site() {
        define_err::warn!("flood");
    }
    let _serial = serial();
    // Miri runs this about ten thousand times slower.
    let rounds = if cfg!(miri) { 4 } else { 10_000_000 };
    thread::scope(|scope| {
        for _ in 0..64 {
            let _ = scope.spawn(move || {
                for _ in 0..rounds {
                    site();
                }
            });
        }
    });
    assert_eq!(RECORDER.take().len(), 1);
}

/// T11: a forged probe claims `SECURITY`, and the reporter sees the claim
/// with the name it came under. `docs/SECURITY.md` says what to check.
#[test]
fn test_forged_security_claim_keeps_its_forged_name() {
    let _serial = serial();
    let forged = Forged(probe_of(&Audit::Breach).replace("Audit::Breach", "Evil::Claim"));
    define_err::report(&forged);
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    assert!(events[0].kinds.contains(Kinds::SECURITY));
    assert_eq!(events[0].type_name.as_deref(), Some("Evil::Claim"));
    assert_eq!(events[0].code, Audit::Breach.code());
}

/// T11: severity filtering is not a security boundary. A forged probe can
/// state a low severity whether or not its other fields are valid, so it can
/// drop itself; it cannot raise anything or reach past the reporter.
#[test]
fn test_forged_severity_can_only_drop_its_own_event() {
    let _serial = serial();
    define_err::set_min_severity(Some(Severity::WARNING));
    let mut valid = probe_of(&Audit::Breach);
    valid.replace_range(20..22, "01");
    let mut malformed = valid.clone();
    malformed.replace_range(4..8, "gggg");
    define_err::report(&Forged(valid));
    define_err::report(&Forged(malformed));
    assert!(RECORDER.take().is_empty());

    define_err::report(&Audit::Breach);
    assert_eq!(RECORDER.take().len(), 1);
}

/// An error that displays the text its `description()` returns, as std's
/// string errors do on Rust 1.81 to 1.89 (`io::Error::other(String)`,
/// `Box<dyn Error>::from(String)`).
#[derive(Debug)]
struct Spoken(String);

impl fmt::Display for Spoken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for Spoken {
    fn description(&self) -> &str {
        &self.0
    }
}

/// T11: outside text shaped like a probe, in a foreign error that displays
/// it or in std's string errors, is delivered as a foreign error: it claims
/// no code, no severity, and no `SECURITY`.
#[test]
fn test_spoken_probe_claims_nothing() {
    let _serial = serial();
    let text = probe_of(&Audit::Breach);
    define_err::report(&Spoken(text.clone()));
    define_err::report(&std::io::Error::other(text.clone()));
    let boxed: Box<dyn Error + Send + Sync> = text.into();
    define_err::report(&*boxed);
    let events = RECORDER.take();
    assert_eq!(events.len(), 3);
    for event in events {
        assert_eq!(event.code, None);
        assert_eq!(event.severity, Severity::ERROR);
        assert!(!event.kinds.contains(Kinds::SECURITY));
    }
}

/// T11: a malformed probe claims nothing. It is a foreign error.
#[test]
fn test_malformed_probe_loses_its_claims() {
    let _serial = serial();
    let mut text = probe_of(&Audit::Breach);
    text.replace_range(4..8, "gggg");
    define_err::report(&Forged(text));
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    assert!(!events[0].kinds.contains(Kinds::SECURITY));
    assert_eq!(events[0].code, None);
    assert_eq!(events[0].type_name, None);
}

/// T17: a record of an event keeps the location private.
#[test]
fn test_location_stays_out_of_public_text() {
    let event = Event::new(define_err::Verb::Report, define_err::here())
        .with_severity(Severity::ERROR)
        .with_error(&Audit::Breach);
    let record = ErrorRecord::from_event(&event);

    let path = file!();
    let directory = path.split(['/', '\\']).next().unwrap();
    assert!(record
        .trail
        .iter()
        .any(|entry| entry.file.contains("attack")));
    assert!(!record.public.contains(path));
    assert!(!record.public.contains(directory));
    assert!(!record.to_string().contains(path));
}

/// T7's boot rule needs a reporter that can be told apart.
struct Second;

impl Reporter for Second {
    fn receive(&self, _event: &Event<'_>) {}
}

static SECOND: Second = Second;

/// T7: the slot never changes hands after the first install.
#[test]
fn test_installed_reporter_cannot_be_replaced() {
    let _serial = serial();
    assert!(define_err::set_reporter(&SECOND).is_err());
    define_err::notice("still the first reporter");
    assert_eq!(RECORDER.take().len(), 1);
}
