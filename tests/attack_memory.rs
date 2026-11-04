//! T5: a hostile error cannot make a record grow past its limits. A file of
//! its own: the peak is read from a global allocator, and one test at a time
//! must be the only one allocating.

#![cfg(feature = "alloc")]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    error::Error,
    fmt,
    mem::size_of,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

use define_err::{innermost_code, ChainEntry, ErrorRecord, RecordLimits};

struct Peak;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: forwards every call to `System` unchanged; the counters are atomics.
unsafe impl GlobalAlloc for Peak {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let live = LIVE.fetch_add(layout.size(), Relaxed) + layout.size();
        let _ = PEAK.fetch_max(live, Relaxed);
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
static GLOBAL: Peak = Peak;

/// `source()` returns itself, and `Display` offers a total of 1 GiB per link
/// from a 1 MiB buffer, ignoring every write error.
#[derive(Debug)]
struct Bomb {
    chunk: String,
    chunks: usize,
}

impl fmt::Display for Bomb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.chunks {
            let _ = f.write_str(&self.chunk);
        }
        Ok(())
    }
}

impl Error for Bomb {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

#[test]
fn test_cyclic_gigabyte_error_stays_inside_the_limits() {
    let limits = RecordLimits::DEFAULT;
    // Miri runs this about ten thousand times slower.
    let (bytes, chunks) = if cfg!(miri) {
        (4_096, 8)
    } else {
        (1 << 20, 1 << 10)
    };
    let bomb = Bomb {
        chunk: "x".repeat(bytes),
        chunks,
    };

    let before = LIVE.load(Relaxed);
    PEAK.store(before, Relaxed);
    let record = ErrorRecord::from_error_with(&bomb, &limits);
    let peak = PEAK.load(Relaxed) - before;

    assert!(record.truncated);
    assert!(record.chain.len() <= limits.max_chain);
    let text: usize = record.chain.iter().map(|entry| entry.message.len()).sum();
    assert!(text <= limits.max_total_bytes);

    // The text, and the vector of entries with at most one doubling of slack.
    // Each entry's strings may keep up to twice what they hold.
    let vector = limits.max_chain * 2 * size_of::<ChainEntry>();
    let allowed = 2 * limits.max_total_bytes + vector;
    assert!(peak <= allowed, "peak {peak} over {allowed}");
}

/// `innermost_code` stops on a cycle too.
#[test]
fn test_cyclic_error_ends_the_code_walk() {
    let bomb = Bomb {
        chunk: String::new(),
        chunks: 0,
    };
    assert!(innermost_code(&bomb).is_none());
}
