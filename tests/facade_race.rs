//! E-67: threads race to install; exactly one wins.

use std::{sync::Barrier, thread};

use define_err::{Event, Reporter};

struct Quiet;

impl Reporter for Quiet {
    fn receive(&self, _event: &Event<'_>) {}
}

static REPORTER: Quiet = Quiet;

#[test]
fn test_one_of_sixteen_installers_wins() {
    let barrier = Barrier::new(16);
    let wins = thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    let _ = barrier.wait();
                    define_err::set_reporter(&REPORTER).is_ok()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|&won| won)
            .count()
    });
    assert_eq!(wins, 1);
    assert!(define_err::is_installed());
    assert!(define_err::reporter_installed_at().is_some());
}
