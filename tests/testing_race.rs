//! Threads race into their first `capture`: every one runs its closure. The
//! window is a few instructions and a process has one first install, so the
//! test re-runs itself in child processes.

#![cfg(feature = "testing")]

use std::{env, process::Command, sync::Barrier, thread};

use define_err::testing::capture;

const CHILD: &str = "DEFINE_ERR_CAPTURE_RACE_CHILD";

fn race() {
    let barrier = Barrier::new(16);
    thread::scope(|scope| {
        for _ in 0..16 {
            let _ = scope.spawn(|| {
                let _ = barrier.wait();
                let ((), events) = capture(|| define_err::notice("mine")).unwrap();
                assert_eq!(events.len(), 1);
            });
        }
    });
}

#[test]
fn test_every_first_capture_runs() {
    if cfg!(miri) || env::var_os(CHILD).is_some() {
        race();
        return;
    }
    let exe = env::current_exe().unwrap();
    for _ in 0..200 {
        let status = Command::new(&exe)
            .args(["--exact", "test_every_first_capture_runs"])
            .env(CHILD, "1")
            .output()
            .unwrap()
            .status;
        assert!(status.success());
    }
}
