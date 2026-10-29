//! Conformance with thiserror: every case `define_errors!` can express is
//! written both ways, and the `Display` output and the `source()` chain must
//! be byte-identical (dev/PLAN.md 12.1).

use std::{error::Error, fmt::Write as _};

/// `Display` of an error followed by every message in its `source()` chain.
fn chain(err: &(dyn Error + 'static)) -> String {
    let mut out = err.to_string();
    let mut next = err.source();
    while let Some(cause) = next {
        let _ = write!(out, " | {cause}");
        next = cause.source();
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: i32,
    y: i32,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

mod ours {
    use super::Point;
    use define_err::define_errors;

    define_errors! {
        pub enum Basic {
            Unit => "unit variant",
            Named { a: u8, b: String } => "named {a} and {b}",
            Specs { hex: u32, width: u16, prec: f32, dbg: &'static str } =>
                "hex {hex:#x} {hex:08X} width [{width:>5}] [{width:<5}] [{width:^5}] prec {prec:.2} dbg {dbg:?}",
            Tuple(first: u8, second: Point) => "tuple {first} {second}",
            Braces { n: u8 } => "{{literal}} {n} {{}}",
            Extra { n: u8 } => ("extra {} {n} {named}", n + 1, named = "x"),
            Expr { p: Point } => ("x is {}", p.x),
            Empty => "",
        }

        pub enum Sources {
            Io(#[from] source: std::io::Error) => "io failed: {source}",
            Named { #[source] cause: std::num::ParseIntError } => "parse failed",
            Implicit { source: std::fmt::Error } => "implicit source",
            Optional { #[source] cause: Option<std::io::Error> } => "optional",
            Nested(#[from] inner: Basic) => "nested",
        }

        pub enum Transparent {
            Inner(#[from] inner: Sources) => transparent,
        }

        pub struct Unit => "unit struct";
        pub struct Braced { pub line: u32 } => "braced {line}";
        pub struct Tupled(pub source: std::io::Error) => "tupled";
    }
}

mod theirs {
    use super::Point;

    #[derive(Debug, thiserror::Error)]
    pub enum Basic {
        #[error("unit variant")]
        Unit,
        #[error("named {a} and {b}")]
        Named { a: u8, b: String },
        #[error(
            "hex {hex:#x} {hex:08X} width [{width:>5}] [{width:<5}] [{width:^5}] prec {prec:.2} dbg {dbg:?}"
        )]
        Specs {
            hex: u32,
            width: u16,
            prec: f32,
            dbg: &'static str,
        },
        #[error("tuple {0} {1}")]
        Tuple(u8, Point),
        #[error("{{literal}} {n} {{}}")]
        Braces { n: u8 },
        #[error("extra {} {n} {named}", .n + 1, named = "x")]
        Extra { n: u8 },
        #[error("x is {}", .p.x)]
        Expr { p: Point },
        #[error("")]
        Empty,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Sources {
        #[error("io failed: {0}")]
        Io(#[from] std::io::Error),
        #[error("parse failed")]
        Named {
            #[source]
            cause: std::num::ParseIntError,
        },
        #[error("implicit source")]
        Implicit { source: std::fmt::Error },
        #[error("optional")]
        Optional {
            #[source]
            cause: Option<std::io::Error>,
        },
        #[error("nested")]
        Nested(#[from] Basic),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Transparent {
        #[error(transparent)]
        Inner(#[from] Sources),
    }

    #[derive(Debug, thiserror::Error)]
    #[error("unit struct")]
    pub struct Unit;

    #[derive(Debug, thiserror::Error)]
    #[error("braced {line}")]
    pub struct Braced {
        pub line: u32,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("tupled")]
    pub struct Tupled(#[source] pub std::io::Error);
}

const P: Point = Point { x: -3, y: 4 };

macro_rules! same {
    ($ours:expr, $theirs:expr) => {{
        let ours = $ours;
        let theirs = $theirs;
        assert_eq!(chain(&ours), chain(&theirs));
    }};
}

#[test]
fn test_conformance_display_forms() {
    use ours::Basic as O;
    use theirs::Basic as T;
    same!(O::Unit, T::Unit);
    same!(
        O::Named {
            a: 1,
            b: "two".into()
        },
        T::Named {
            a: 1,
            b: "two".into()
        }
    );
    same!(
        O::Specs {
            hex: 255,
            width: 7,
            prec: 1.23456,
            dbg: "q\"t"
        },
        T::Specs {
            hex: 255,
            width: 7,
            prec: 1.23456,
            dbg: "q\"t"
        }
    );
    same!(O::Tuple(9, P), T::Tuple(9, P));
    same!(O::Braces { n: 5 }, T::Braces { n: 5 });
    same!(O::Extra { n: 5 }, T::Extra { n: 5 });
    same!(O::Expr { p: P }, T::Expr { p: P });
    same!(O::Empty, T::Empty);
}

#[test]
fn test_conformance_sources() {
    use ours::Sources as O;
    use theirs::Sources as T;
    same!(
        O::from(std::io::Error::other("disk")),
        T::from(std::io::Error::other("disk"))
    );
    same!(
        O::Named {
            cause: "x".parse::<u8>().unwrap_err()
        },
        T::Named {
            cause: "x".parse::<u8>().unwrap_err()
        }
    );
    same!(
        O::Implicit {
            source: std::fmt::Error
        },
        T::Implicit {
            source: std::fmt::Error
        }
    );
    same!(O::Optional { cause: None }, T::Optional { cause: None });
    same!(
        O::Optional {
            cause: Some(std::io::Error::other("late"))
        },
        T::Optional {
            cause: Some(std::io::Error::other("late"))
        }
    );
    same!(
        O::from(ours::Basic::Named {
            a: 1,
            b: "x".into()
        }),
        T::from(theirs::Basic::Named {
            a: 1,
            b: "x".into()
        })
    );
}

#[test]
fn test_conformance_transparent() {
    use ours::Transparent as O;
    use theirs::Transparent as T;
    same!(
        O::from(ours::Sources::from(std::io::Error::other("deep"))),
        T::from(theirs::Sources::from(std::io::Error::other("deep")))
    );
}

#[test]
fn test_conformance_structs() {
    same!(ours::Unit, theirs::Unit);
    same!(ours::Braced { line: 3 }, theirs::Braced { line: 3 });
    same!(
        ours::Tupled(std::io::Error::other("t")),
        theirs::Tupled(std::io::Error::other("t"))
    );
}
