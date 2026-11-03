//! `ErrorRecord` from a three-level chain (dev/PLAN.md 11.1). Results:
//! docs/PERFORMANCE.md.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use define_err::{define_errors, ErrorRecord};

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, help = "Raise pool.size.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",
    }

    pub enum StoreError {
        Write(#[from] inner: DbError) => "cannot write the page",
    }

    pub enum AppError {
        Save(#[from] inner: StoreError) => "cannot save the document",
    }
}

fn record(c: &mut Criterion) {
    let single = DbError::Timeout { after_ms: 5000 };
    let chain = AppError::from(StoreError::from(DbError::Timeout { after_ms: 5000 }));
    let mut group = c.benchmark_group("record");
    group.bench_function("from_error_one_level", |b| {
        b.iter(|| ErrorRecord::from_error(black_box(&single)))
    });
    group.bench_function("from_error_three_levels", |b| {
        b.iter(|| ErrorRecord::from_error(black_box(&chain)))
    });
    let record = ErrorRecord::from_error(&chain);
    group.bench_function("clone", |b| b.iter(|| black_box(&record).clone()));
    group.finish();
}

criterion_group!(benches, record);
criterion_main!(benches);
