//! `report_record` and the creation hook, with a reporter installed.

#![cfg(feature = "alloc")]

mod common;

use std::sync::Mutex;

use common::{serial, RECORDER};
use define_err::{
    Annotate, Category, CreationHook, CreationInfo, ErrorRecord, Event, Kinds, Reporter, Severity,
    Slot, Verb,
};

#[test]
fn test_report_record_uses_the_records_facts() {
    let _serial = serial();
    let mut record = ErrorRecord::new(
        None,
        Severity::CRITICAL,
        Category::Unavailable,
        Kinds::ERROR,
    );
    assert!(record.push_chain(&"while syncing", None, "context"));
    define_err::report_record(&record);
    let events = RECORDER.take();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].verb, Verb::Report);
    assert_eq!(events[0].severity, Severity::CRITICAL);
    assert_eq!(events[0].category, Category::Unavailable);
    assert_eq!(events[0].type_name.as_deref(), Some("context"));
}

#[test]
fn test_report_record_is_filtered() {
    let _serial = serial();
    define_err::set_min_severity(Some(Severity::FATAL));
    let record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    define_err::report_record(&record);
    assert!(RECORDER.take().is_empty());
}

struct RecordKeeper(Mutex<Option<ErrorRecord>>);

impl Reporter for RecordKeeper {
    fn receive(&self, event: &Event<'_>) {
        *self.0.lock().unwrap() = event.record().cloned();
    }
}

#[test]
fn test_event_carries_the_record() {
    let keeper = RecordKeeper(Mutex::new(None));
    let record = ErrorRecord::new(None, Severity::ERROR, Category::Internal, Kinds::ERROR);
    keeper.receive(&Event::new(Verb::Report, define_err::here()).with_record(&record));
    assert_eq!(keeper.0.lock().unwrap().as_ref(), Some(&record));
}

struct Scope;

impl CreationHook for Scope {
    fn on_create(&self, info: &CreationInfo<'_>, out: &mut dyn Annotate) {
        out.field("type", &info.type_name());
        out.kind(Kinds::SECURITY);
    }
}

#[derive(Default)]
struct Collected {
    fields: Vec<(&'static str, String)>,
    kinds: Option<Kinds>,
}

impl Annotate for Collected {
    fn field(&mut self, key: &'static str, value: &dyn core::fmt::Display) {
        self.fields.push((key, value.to_string()));
    }

    fn kind(&mut self, kinds: Kinds) {
        self.kinds = Some(kinds);
    }
}

static SCOPE: Scope = Scope;

#[test]
fn test_creation_hook_slot_is_write_once_and_independent() {
    let _serial = serial();
    assert!(define_err::creation_hook().is_none());
    define_err::set_creation_hook(&SCOPE).unwrap();
    let err = define_err::set_creation_hook(&SCOPE).unwrap_err();
    assert_eq!(err.slot, Slot::CreationHook);
    assert!(err
        .to_string()
        .starts_with("a creation hook is already installed"));
    assert!(define_err::is_installed());

    let hook = define_err::creation_hook().unwrap();
    let mut out = Collected::default();
    let info = CreationInfo::new(define_err::here(), None, "app::Error");
    assert_eq!(info.code(), None);
    assert_eq!(info.location().file(), file!());
    hook.on_create(&info, &mut out);
    assert_eq!(out.fields, [("type", "app::Error".to_string())]);
    assert_eq!(out.kinds, Some(Kinds::SECURITY));
}
