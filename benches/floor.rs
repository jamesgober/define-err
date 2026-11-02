//! The fastest of many short batches of each hot path. A criterion mean moves
//! with whatever else the machine runs; the floor is what the code costs when
//! nothing competes, so this is what the regression baseline compares
//! (dev/PLAN.md 11.2). Results: docs/PERFORMANCE.md.

use std::{fmt::Write as _, hint::black_box, time::Instant};

use define_err::{define_errors, identify, prelude::*, Event, Kinds, Reporter, Severity};
use log::{LevelFilter, Log, Metadata, Record};

const BATCHES: u32 = 4_000;
const ITERATIONS: u32 = 1_000;

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

fn floor(name: &str, mut work: impl FnMut()) {
    let mut best = f64::INFINITY;
    for _ in 0..BATCHES {
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            work();
        }
        best = best.min(start.elapsed().as_secs_f64());
    }
    println!("{name:<34}{:>9.2} ns", best * 1e9 / f64::from(ITERATIONS));
}

fn warn_macro_site() {
    define_err::warn!("cache file is stale");
}

fn warn_function_site() {
    define_err::warn("cache file is stale");
}

fn main() {
    let err = DbError::Timeout { after_ms: 5 };
    let foreign = std::fmt::Error;
    let code = DbError::Timeout { after_ms: 5 }.code().unwrap();
    let raw = code.as_u64();
    let mut out = String::with_capacity(32);

    floor("empty_loop", || {
        let _ = black_box(1);
    });
    floor("code_display", || {
        out.clear();
        write!(out, "{}", black_box(code)).unwrap();
    });
    floor("code_parse", || {
        let _ = black_box(Code::parse(black_box("DB-0001")));
    });
    floor("code_from_u64", || {
        let _ = black_box(Code::from_u64(black_box(raw)));
    });
    floor("error_info_code", || {
        let _ = black_box(black_box(&err).code());
    });
    floor("identify_generated", || {
        let _ = black_box(identify(black_box(&err)).map(|id| id.severity));
    });
    floor("identify_foreign", || {
        let _ = black_box(identify(black_box(&foreign)).map(|id| id.severity));
    });

    floor("not_installed_report", || {
        define_err::report(black_box(&err))
    });
    floor("not_installed_notice", || {
        define_err::notice(black_box("text"))
    });
    floor("not_installed_notice_macro", || {
        define_err::notice!("text {}", black_box(1));
    });
    floor("not_installed_warn", || define_err::warn(black_box("text")));
    floor("not_installed_warn_macro", || {
        define_err::warn!("text {}", black_box(1));
    });
    floor("not_installed_log_warn", || {
        log::warn!("text {}", black_box(1))
    });

    define_err::set_reporter(&NO_OP).unwrap();
    log::set_logger(&NO_OP).unwrap();
    log::set_max_level(LevelFilter::Trace);
    warn_macro_site();
    warn_function_site();
    floor("warn_macro_after_first", warn_macro_site);
    floor("warn_after_first", warn_function_site);

    define_err::set_min_severity(Some(Severity::FATAL));
    log::set_max_level(LevelFilter::Error);
    floor("filtered_report", || define_err::report(black_box(&err)));
    floor("filtered_report_with", || {
        define_err::report_with(black_box(&err), Severity::DEBUG, Kinds::ERROR);
    });
    floor("filtered_notice", || define_err::notice(black_box("text")));
    floor("filtered_log_warn", || log::warn!("text {}", black_box(1)));
    define_err::set_min_severity(None);
    log::set_max_level(LevelFilter::Trace);

    floor("delivered_report", || define_err::report(black_box(&err)));
    floor("delivered_report_foreign", || {
        define_err::report(black_box(&foreign))
    });
    floor("delivered_notice", || define_err::notice(black_box("text")));
    floor("delivered_log_error", || {
        log::error!("text {}", black_box(1))
    });
}
