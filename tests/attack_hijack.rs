//! T7: a dependency installs first. The application's own install fails, and
//! the error names the file that took the slot. One test, because the slot
//! is write-once.

#[path = "attack_support/dependency.rs"]
mod dependency;

use define_err::{Event, Reporter};

struct Application;

impl Reporter for Application {
    fn receive(&self, _event: &Event<'_>) {
        panic!("the application's reporter was never installed");
    }
}

static APPLICATION: Application = Application;

#[test]
fn test_hijacking_install_is_named() {
    dependency::init();
    let error = define_err::set_reporter(&APPLICATION).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("dependency.rs"), "{text}");

    let at = define_err::reporter_installed_at().unwrap();
    assert!(at.file().ends_with("dependency.rs"));
    assert_eq!(error.installed_at, Some(at));
}
