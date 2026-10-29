//! Construction, `Display` into a fixed buffer, `source()`, the `ErrorInfo`
//! methods, `From`, and `Secret` formatting allocate nothing (NFR-3, 12.6).

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    error::Error as _,
    fmt::Write as _,
};

use define_err::{define_errors, prelude::*};

struct Counting;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

// SAFETY: forwards every call to `System` unchanged; the counter is a
// thread-local `Cell`, which never allocates once initialised.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        // SAFETY: the caller upholds `GlobalAlloc::alloc`'s contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the caller upholds `GlobalAlloc::dealloc`'s contract.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn allocations_during(f: impl FnOnce()) -> usize {
    let before = ALLOCATIONS.with(Cell::get);
    f();
    ALLOCATIONS.with(Cell::get) - before
}

/// A fixed buffer that never allocates.
struct Buffer {
    bytes: [u8; 256],
    len: usize,
}

impl std::fmt::Write for Buffer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let end = self.len + s.len();
        if end > self.bytes.len() {
            return Err(std::fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

define_errors! {
    pub enum Store {
        NotFound { page: u64, name: &'static str } => "page {page} of {name:?} not found",
        Io(#[from] source: std::fmt::Error, #[location] at: Location) => "i/o: {source}",
        Token { token: Secret<&'static str> } => "token {token:>12}",
        Wrapped(#[from] inner: Inner) => transparent,
    }

    pub struct Inner { pub code: u16 } => "inner {code}";
}

#[test]
fn test_construction_display_source_info_allocate_nothing() {
    let mut buffer = Buffer {
        bytes: [0; 256],
        len: 0,
    };
    let count = allocations_during(|| {
        let a = Store::NotFound {
            page: 7,
            name: "data",
        };
        let b: Store = std::fmt::Error.into();
        let c = Store::Token {
            token: Secret::new("s3cr3t"),
        };
        let d: Store = Inner { code: 3 }.into();
        for err in [&a, &b, &c, &d] {
            write!(buffer, "{err}|").unwrap();
            let _ = err.source();
            let _ = err.location();
            let _ = err.type_name();
        }
        write!(buffer, "{:?}", Secret::new("pw")).unwrap();
    });
    assert_eq!(count, 0);
    let text = std::str::from_utf8(&buffer.bytes[..buffer.len]).unwrap();
    assert_eq!(
        text,
        "page 7 of \"data\" not found|i/o: an error occurred when formatting an argument|token   [redacted]|inner 3|[redacted]"
    );
}
