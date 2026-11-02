//! Codes, the catalog, `ErrorInfo`, and the probe, against error-forge's
//! string codes and locked registry (dev/PLAN.md 11.1, 11.2). Results:
//! docs/PERFORMANCE.md.

use std::{fmt::Write as _, hint::black_box};

use criterion::{criterion_group, criterion_main, Criterion};
use define_err::{catalog, define_errors, identify, prelude::*};
use error_forge::registry::ErrorRegistry;

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, name = TIMEOUT, help = "Raise pool.size.")]
        #[category(DeadlineExceeded)]
        #[severity(WARNING)]
        #[kinds(ERROR | SECURITY)]
        Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",

        #[code(2, help = "Restore the page.")]
        #[category(DataLoss)]
        Corrupt { page: u64 } => "page {page} failed its checksum",
    }
}

// Its expansion checks a `serde` feature this crate does not have.
#[allow(unexpected_cfgs)]
mod forge {
    error_forge::define_errors! {
        pub enum ForgeDbError {
            #[error(display = "query timed out after {after_ms} ms", after_ms)]
            #[kind(Timeout, retryable = true, status = 504)]
            Timeout { after_ms: u64 },
        }
    }
}

fn code_text(c: &mut Criterion) {
    let code = DbError::TIMEOUT;
    let mut out = String::with_capacity(32);
    let mut group = c.benchmark_group("code_display");
    group.bench_function("define_err", |b| {
        b.iter(|| {
            out.clear();
            write!(out, "{}", black_box(code)).unwrap();
            black_box(out.len())
        })
    });
    group.bench_function("error_forge_format_string", |b| {
        b.iter(|| {
            let (namespace, number) = (black_box("DB"), black_box(1_u32));
            black_box(format!("{namespace}-{number:04}"))
        })
    });
    group.finish();

    let mut group = c.benchmark_group("code_parse");
    group.bench_function("define_err", |b| {
        b.iter(|| Code::parse(black_box("DB-0001")))
    });
    group.finish();

    let raw = code.as_u64();
    c.bench_function("code_from_u64", |b| {
        b.iter(|| Code::from_u64(black_box(raw)))
    });
}

fn lookup(c: &mut Criterion) {
    let registry = ErrorRegistry::global();
    registry
        .register_code("DB-0001".into(), "Raise pool.size.".into(), None, true)
        .unwrap();
    let key = String::from("DB-0001");
    let catalogs = [DbError::CATALOG];
    let err = DbError::Timeout { after_ms: 5 };
    let mut group = c.benchmark_group("catalog_lookup");
    group.bench_function("define_err_code_info", |b| {
        b.iter(|| black_box(&err).code_info())
    });
    group.bench_function("define_err_catalog_find", |b| {
        b.iter(|| catalog::find(black_box(&catalogs), black_box(DbError::TIMEOUT)))
    });
    group.bench_function("error_forge_registry", |b| {
        b.iter(|| registry.get_code_info(black_box(&key)))
    });
    group.finish();
}

fn info(c: &mut Criterion) {
    let err = DbError::Timeout { after_ms: 5 };
    let forge = forge::ForgeDbError::timeout(5);
    let mut group = c.benchmark_group("error_info");
    group.bench_function("code", |b| b.iter(|| black_box(&err).code()));
    group.bench_function("category", |b| b.iter(|| black_box(&err).category()));
    group.bench_function("severity", |b| b.iter(|| black_box(&err).severity()));
    group.bench_function("kinds", |b| b.iter(|| black_box(&err).kinds()));
    group.bench_function("is_retryable", |b| {
        b.iter(|| black_box(&err).is_retryable())
    });
    group.bench_function("error_forge_kind", |b| b.iter(|| black_box(&forge).kind()));
    group.bench_function("error_forge_is_retryable", |b| {
        b.iter(|| black_box(&forge).is_retryable())
    });
    group.finish();
}

fn probe(c: &mut Criterion) {
    let err = DbError::Timeout { after_ms: 5 };
    let foreign = std::fmt::Error;
    let mut group = c.benchmark_group("identify");
    // One field: passing a whole `Identity` through `black_box` measures a copy.
    group.bench_function("generated", |b| {
        b.iter(|| identify(black_box(&err)).map(|id| id.severity))
    });
    group.bench_function("foreign", |b| {
        b.iter(|| identify(black_box(&foreign)).map(|id| id.severity))
    });
    group.finish();
}

criterion_group!(benches, code_text, lookup, info, probe);
criterion_main!(benches);
