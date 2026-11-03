//! Every verb in the three states: not installed, filtered, delivered, against
//! `log` where it has a counterpart (dev/PLAN.md 11.1, 11.2). The states run in
//! this order, because the slots are write-once. Results: docs/PERFORMANCE.md.

use std::{
    hint::black_box,
    sync::Barrier,
    thread,
    time::{Duration, Instant},
};

use criterion::{criterion_group, criterion_main, Criterion};
use define_err::{define_errors, Event, Kinds, Reporter, Severity};
use log::{LevelFilter, Log, Metadata, Record};

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, help = "Raise pool.size.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",
    }
}

struct NoOp;

impl Reporter for NoOp {
    fn receive(&self, event: &Event<'_>) {
        let _ = black_box(event.severity());
    }
}

impl Log for NoOp {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        let _ = black_box(record.level());
    }

    fn flush(&self) {}
}

static NO_OP: NoOp = NoOp;

fn warn_macro_site() {
    define_err::warn!("cache file is stale");
}

fn warn_function_site() {
    define_err::warn("cache file is stale");
}

fn not_installed(c: &mut Criterion) {
    let err = DbError::Timeout { after_ms: 5 };
    let mut group = c.benchmark_group("not_installed");
    group.bench_function("report", |b| b.iter(|| define_err::report(black_box(&err))));
    group.bench_function("notice", |b| {
        b.iter(|| define_err::notice(black_box("text")))
    });
    group.bench_function("notice_macro", |b| {
        b.iter(|| define_err::notice!("text {}", black_box(1)))
    });
    group.bench_function("warn", |b| b.iter(|| define_err::warn(black_box("text"))));
    group.bench_function("warn_macro", |b| {
        b.iter(|| define_err::warn!("text {}", black_box(1)))
    });
    group.bench_function("trigger", |b| {
        b.iter(|| define_err::trigger(Kinds::SECURITY, black_box("text")))
    });
    group.bench_function("trigger_macro", |b| {
        b.iter(|| define_err::trigger!(Kinds::SECURITY, "text {}", black_box(1)))
    });
    group.bench_function("deprecated", |b| {
        b.iter(|| define_err::deprecated(black_box("old_call")))
    });
    group.bench_function("deprecated_macro", |b| {
        b.iter(|| define_err::deprecated!("old_call"))
    });
    group.bench_function("soft", |b| {
        b.iter(|| define_err::soft(black_box(DbError::Timeout { after_ms: 5 })))
    });
    group.bench_function("log_warn_max_level_off", |b| {
        b.iter(|| log::warn!("text {}", black_box(1)))
    });
    group.finish();
}

fn after_first_delivery(c: &mut Criterion) {
    warn_macro_site();
    warn_function_site();
    let mut group = c.benchmark_group("warn_after_first_delivery");
    group.bench_function("warn_macro", |b| b.iter(warn_macro_site));
    group.bench_function("warn", |b| b.iter(warn_function_site));
    group.finish();
}

fn filtered(c: &mut Criterion) {
    let err = DbError::Timeout { after_ms: 5 };
    define_err::set_min_severity(Some(Severity::FATAL));
    let mut group = c.benchmark_group("filtered");
    group.bench_function("report", |b| b.iter(|| define_err::report(black_box(&err))));
    group.bench_function("report_with", |b| {
        b.iter(|| define_err::report_with(black_box(&err), Severity::DEBUG, Kinds::ERROR))
    });
    group.bench_function("notice", |b| {
        b.iter(|| define_err::notice(black_box("text")))
    });
    group.bench_function("trigger", |b| {
        b.iter(|| define_err::trigger(Kinds::SECURITY, black_box("text")))
    });
    group.bench_function("soft", |b| {
        b.iter(|| define_err::soft(black_box(DbError::Timeout { after_ms: 5 })))
    });
    log::set_max_level(LevelFilter::Error);
    group.bench_function("log_warn_filtered", |b| {
        b.iter(|| log::warn!("text {}", black_box(1)))
    });
    group.finish();
    define_err::set_min_severity(None);
}

fn delivered(c: &mut Criterion) {
    let err = DbError::Timeout { after_ms: 5 };
    let foreign = std::fmt::Error;
    let mut group = c.benchmark_group("delivered");
    group.bench_function("report", |b| b.iter(|| define_err::report(black_box(&err))));
    group.bench_function("report_foreign", |b| {
        b.iter(|| define_err::report(black_box(&foreign)))
    });
    group.bench_function("notice", |b| {
        b.iter(|| define_err::notice(black_box("text")))
    });
    group.bench_function("notice_macro", |b| {
        b.iter(|| define_err::notice!("text {}", black_box(1)))
    });
    group.bench_function("trigger", |b| {
        b.iter(|| define_err::trigger(Kinds::SECURITY, black_box("text")))
    });
    group.bench_function("soft", |b| {
        b.iter(|| define_err::soft(black_box(DbError::Timeout { after_ms: 5 })))
    });
    group.bench_function("log_error", |b| {
        b.iter(|| log::error!("text {}", black_box(1)))
    });
    group.finish();
}

/// Sixteen threads hammering one call site.
fn contended(c: &mut Criterion) {
    let mut group = c.benchmark_group("sixteen_threads_one_site");
    group.bench_function("warn_macro", |b| {
        b.iter_custom(|iters| {
            let barrier = Barrier::new(16);
            let total: Duration = thread::scope(|scope| {
                let workers: Vec<_> = (0..16)
                    .map(|_| {
                        scope.spawn(|| {
                            let _ = barrier.wait();
                            let start = Instant::now();
                            for _ in 0..iters {
                                warn_macro_site();
                            }
                            start.elapsed()
                        })
                    })
                    .collect();
                workers
                    .into_iter()
                    .map(|worker| worker.join().unwrap())
                    .sum()
            });
            total / 16
        })
    });
    group.finish();
}

/// The order is the point: nothing is installed for the first group.
fn all_states(c: &mut Criterion) {
    not_installed(c);
    define_err::set_reporter(&NO_OP).unwrap();
    log::set_logger(&NO_OP).unwrap();
    log::set_max_level(LevelFilter::Trace);
    after_first_delivery(c);
    filtered(c);
    log::set_max_level(LevelFilter::Trace);
    delivered(c);
    contended(c);
}

criterion_group!(benches, all_states);
criterion_main!(benches);
