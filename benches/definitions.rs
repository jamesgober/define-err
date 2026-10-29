//! Construction and `Display` of a generated error against the same type
//! written with thiserror and by hand (dev/PLAN.md 11.1: identical).

use std::{error::Error as _, fmt, fmt::Write as _, hint::black_box};

use criterion::{criterion_group, criterion_main, Criterion};
use define_err::define_errors;

define_errors! {
    pub enum Ours {
        NotFound { page: u64 } => "page {page} not found",
        Short(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
        Io(#[from] source: std::io::Error) => "page file i/o failed",
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Theirs {
    #[error("page {page} not found")]
    NotFound { page: u64 },
    #[error("short read on page {0}: got {1} bytes")]
    Short(u64, usize),
    #[error("page file i/o failed")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum Hand {
    NotFound { page: u64 },
    Short(u64, usize),
    Io(std::io::Error),
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hand::NotFound { page } => write!(f, "page {page} not found"),
            Hand::Short(page, got) => write!(f, "short read on page {page}: got {got} bytes"),
            Hand::Io(_) => f.write_str("page file i/o failed"),
        }
    }
}

impl std::error::Error for Hand {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Hand::Io(source) => Some(source),
            _ => None,
        }
    }
}

fn bench_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("display");
    let mut out = String::with_capacity(128);
    let ours = Ours::Short(7, 12);
    let theirs = Theirs::Short(7, 12);
    let hand = Hand::Short(7, 12);
    group.bench_function("define_err", |b| {
        b.iter(|| {
            out.clear();
            write!(out, "{}", black_box(&ours)).unwrap();
            black_box(out.len())
        })
    });
    group.bench_function("thiserror", |b| {
        b.iter(|| {
            out.clear();
            write!(out, "{}", black_box(&theirs)).unwrap();
            black_box(out.len())
        })
    });
    group.bench_function("hand_written", |b| {
        b.iter(|| {
            out.clear();
            write!(out, "{}", black_box(&hand)).unwrap();
            black_box(out.len())
        })
    });
    group.finish();
}

fn bench_construct_and_source(c: &mut Criterion) {
    let mut group = c.benchmark_group("construct_source");
    group.bench_function("define_err", |b| {
        b.iter(|| {
            let err = Ours::NotFound { page: black_box(7) };
            black_box(err.source().is_none())
        })
    });
    group.bench_function("thiserror", |b| {
        b.iter(|| {
            let err = Theirs::NotFound { page: black_box(7) };
            black_box(err.source().is_none())
        })
    });
    group.bench_function("hand_written", |b| {
        b.iter(|| {
            let err = Hand::NotFound { page: black_box(7) };
            black_box(err.source().is_none())
        })
    });
    group.finish();
}

criterion_group!(benches, bench_display, bench_construct_and_source);
criterion_main!(benches);
