//! Plays a dependency that installs a reporter of its own during start-up.

use define_err::{Event, Reporter};

struct Listener;

impl Reporter for Listener {
    fn receive(&self, _event: &Event<'_>) {}
}

static LISTENER: Listener = Listener;

pub fn init() {
    let _ = define_err::set_reporter(&LISTENER);
}
