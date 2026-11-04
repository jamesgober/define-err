//! Stress and soak (`dev/PLAN.md` 12.5): every verb from 64 threads against a
//! counting reporter, with exact counts and flat memory. A short pass runs in
//! the suite. The full runs are ignored; they run in release:
//!
//! ```text
//! cargo test --release --all-features --test stress -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `DEFINE_ERR_SOAK_SECS` sets the soak's length, 8 hours by default.

#![cfg(feature = "std")]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed},
        Mutex, MutexGuard,
    },
    thread,
    time::{Duration, Instant},
};

use define_err::{define_errors, ErrorRecord, Event, Kinds, Reporter, Severity, Verb};

const THREADS: u64 = 64;

/// Memory may rise this far past the first round: allocator and thread-exit
/// bookkeeping, not anything that scales with events.
const SLACK: usize = 256 * 1024;

struct Live;

static LIVE: AtomicUsize = AtomicUsize::new(0);

// SAFETY: forwards every call to `System` unchanged; the counter is an atomic.
unsafe impl GlobalAlloc for Live {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = LIVE.fetch_add(layout.size(), Relaxed);
        // SAFETY: the caller upholds `GlobalAlloc::alloc`'s contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = LIVE.fetch_sub(layout.size(), Relaxed);
        // SAFETY: the caller upholds `GlobalAlloc::dealloc`'s contract.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Live = Live;

define_errors! {
    namespace = "ST";

    pub enum Load {
        #[code(1, help = "None needed.")]
        #[category(Unavailable)]
        Busy { shard: u64 } => "shard {shard} is busy",
    }
}

/// One count per verb. A soft event also builds and drops a record, so a leak
/// in the owned path shows as growth.
#[derive(Default)]
struct Counts {
    report: AtomicU64,
    notice: AtomicU64,
    warn: AtomicU64,
    trigger: AtomicU64,
    deprecated: AtomicU64,
    soft: AtomicU64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Snapshot {
    report: u64,
    notice: u64,
    trigger: u64,
    soft: u64,
}

struct Counting(Counts);

impl Reporter for Counting {
    fn receive(&self, event: &Event<'_>) {
        let count = match event.verb() {
            Verb::Report => &self.0.report,
            Verb::Notice => &self.0.notice,
            Verb::Warn => &self.0.warn,
            Verb::Trigger => &self.0.trigger,
            Verb::Deprecated => &self.0.deprecated,
            Verb::Soft => {
                drop(ErrorRecord::from_event(event));
                &self.0.soft
            }
            _ => return,
        };
        let _ = count.fetch_add(1, Relaxed);
    }
}

static REPORTER: Counting = Counting(Counts {
    report: AtomicU64::new(0),
    notice: AtomicU64::new(0),
    warn: AtomicU64::new(0),
    trigger: AtomicU64::new(0),
    deprecated: AtomicU64::new(0),
    soft: AtomicU64::new(0),
});

static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let _ = define_err::set_reporter(&REPORTER);
    guard
}

fn snapshot() -> Snapshot {
    Snapshot {
        report: REPORTER.0.report.load(Relaxed),
        notice: REPORTER.0.notice.load(Relaxed),
        trigger: REPORTER.0.trigger.load(Relaxed),
        soft: REPORTER.0.soft.load(Relaxed),
    }
}

/// One call of every verb. `report` runs three ways, so it counts three times.
fn mix(shard: u64) {
    let error = Load::Busy { shard };
    define_err::report(&error);
    define_err::report_with(&error, Severity::ERROR, Kinds::ERROR);
    define_err::report_record(&ErrorRecord::new(
        None,
        Severity::ERROR,
        define_err::Category::Internal,
        Kinds::ERROR,
    ));
    define_err::notice("load");
    define_err::warn("one site, by function");
    define_err::warn!("one site, by macro");
    define_err::trigger(Kinds::RESOURCE, "load");
    define_err::deprecated("load.old_key");
    define_err::deprecated!("one site, by macro");
    let _ = define_err::soft(error);
}

/// `rounds` rounds of `calls` calls per thread, or until `until`. Returns the
/// rounds run and the highest memory seen after the first.
fn drive(rounds: u64, calls: u64, until: Option<Instant>) -> (u64, usize) {
    let before = snapshot();
    let mut baseline = None;
    let mut highest = 0;
    let mut done = 0;
    let expired = || until.is_some_and(|end| Instant::now() >= end);
    while done < rounds && !expired() {
        thread::scope(|scope| {
            for thread in 0..THREADS {
                let _ = scope.spawn(move || {
                    for call in 0..calls {
                        mix(thread << 32 | call);
                    }
                });
            }
        });
        done += 1;
        let live = LIVE.load(Relaxed);
        let first = *baseline.get_or_insert(live);
        highest = highest.max(live);
        assert!(
            live <= first + SLACK,
            "memory grew to {live} from {first} after {done} rounds"
        );
    }

    let events = done * THREADS * calls;
    assert_eq!(
        snapshot(),
        Snapshot {
            report: before.report + 3 * events,
            notice: before.notice + events,
            trigger: before.trigger + events,
            soft: before.soft + events,
        }
    );
    // Once per site, however many calls ran, and across every run.
    assert_eq!(REPORTER.0.warn.load(Relaxed), 2);
    assert_eq!(REPORTER.0.deprecated.load(Relaxed), 2);
    let stats = define_err::stats();
    assert_eq!(stats.reentrant_dropped, 0);
    assert_eq!(stats.reporter_panics, 0);
    assert_eq!(stats.site_table_overflow, 0);
    (done, highest)
}

#[test]
fn test_stress_short_pass() {
    let _serial = serial();
    // Miri runs this about ten thousand times slower.
    let calls = if cfg!(miri) { 1 } else { 1_000 };
    let _ = drive(3, calls, None);
}

/// 64 threads, every verb, ten million calls each.
#[test]
#[ignore = "minutes in release; run by hand"]
fn test_stress_ten_million_calls_per_thread() {
    let _serial = serial();
    let start = Instant::now();
    let _ = drive(1, 10_000_000, None);
    println!("stress: {:?}", start.elapsed());
}

#[test]
#[ignore = "hours; run by hand"]
fn test_soak() {
    let _serial = serial();
    let seconds = std::env::var("DEFINE_ERR_SOAK_SECS")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(8 * 60 * 60);
    let start = Instant::now();
    let until = start + Duration::from_secs(seconds);
    let (rounds, highest) = drive(u64::MAX, 100_000, Some(until));
    println!(
        "soak: {rounds} rounds in {:?}, {} verb calls per thread each round, highest live {highest} bytes",
        start.elapsed(),
        100_000
    );
}
