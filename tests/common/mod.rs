//! A reporter that keeps what it receives, for the facade's test files. Each
//! test file is its own process, so each has its own write-once slot; tests in
//! one file share it and take `serial()` so the global settings do not race.

#![allow(dead_code)]

use std::sync::{Mutex, MutexGuard};

use define_err::{Category, Code, Event, Kinds, Reporter, Severity, Verb, WarnMode};

#[derive(Clone, Debug)]
pub struct Seen {
    pub verb: Verb,
    pub kinds: Kinds,
    pub severity: Severity,
    pub code: Option<Code>,
    pub category: Category,
    pub message: String,
    pub type_name: Option<String>,
    pub file: &'static str,
    pub line: u32,
}

pub struct Recorder {
    events: Mutex<Vec<Seen>>,
}

pub static RECORDER: Recorder = Recorder {
    events: Mutex::new(Vec::new()),
};

impl Recorder {
    pub fn take(&self) -> Vec<Seen> {
        std::mem::take(&mut *self.events.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

impl Reporter for Recorder {
    fn receive(&self, event: &Event<'_>) {
        let message = match (event.error(), event.message()) {
            (Some(error), _) => error.to_string(),
            (None, Some(message)) => message.to_string(),
            (None, None) => String::new(),
        };
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(Seen {
                verb: event.verb(),
                kinds: event.kinds(),
                severity: event.severity(),
                code: event.code(),
                category: event.category(),
                message,
                type_name: event.type_name().map(String::from),
                file: event.location().file(),
                line: event.location().line(),
            });
    }
}

static SERIAL: Mutex<()> = Mutex::new(());

/// Takes the file's lock, installs the recorder, and resets every setting.
pub fn serial() -> MutexGuard<'static, ()> {
    let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let _ = define_err::set_reporter(&RECORDER);
    define_err::set_warn_mode(WarnMode::Once);
    define_err::set_min_severity(None);
    define_err::set_kinds_mask(Kinds::ALL);
    let _ = RECORDER.take();
    guard
}
