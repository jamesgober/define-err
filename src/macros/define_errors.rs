//! `define_errors!`: the entry point and the parser.
//!
//! The parser has two paths that end in the same emitter (`emit.rs`):
//!
//! - **Fast path.** One arm matches a whole enum written in the canonical
//!   order (on variants: doc comments, then the metadata attributes in the
//!   order `code`, `category`, `severity`, `kinds`, `public`, `action`,
//!   `retryable`, `retry_after`; on fields: doc comments, then at most one
//!   marker right before the name; tuple fields named) and hands every
//!   variant to the emitter through repetitions. It does not recurse, so an
//!   enum of any size compiles at the default `recursion_limit`.
//! - **General path.** Anything else (lint attributes, attributes in other
//!   orders, unnamed tuple fields, structs) is normalised one variant at a
//!   time. A variant already in canonical form costs one step; an irregular
//!   one costs one step per attribute and field.
//!
//! Matching rules that shape the patterns: a `tt`, `ident`, or `vis` fragment
//! may never compete with a literal token at the same position (rustc reports
//! "local ambiguity"), and a `ty` fragment may only be followed by `,`, `=>`,
//! `:`, `;`, `=`, `|`, `>`, `{`, `[`, `as`, or `where`. Every slot below is
//! therefore led by a literal token (`#[doc`, `#[code`, `#[from]`, ...), and
//! field markers capture the field name, with the type in one shared tail.
//!
//! Every type of a block is emitted in turn, and each passes the block's
//! context to the next: `[[namespace] [docs] [retired] [retired, for this
//! type's catalog] [codes so far] [rest of the block]]`. After the last type
//! the block emits its compile-time checks over every code it defined.
//!
//! A variant's metadata travels as eight slots, each empty or holding the
//! attribute's argument: `[code] [category] [severity] [kinds] [public]
//! [action] [retryable] [retry_after]`.

/// Declares error types: enums and structs with `Display`, `core::error::Error`
/// with `source()`, `From` conversions, `Debug`, and `ErrorInfo`, plus
/// optional stable codes, a compile-time catalog, and metadata.
///
/// One line per variant: `Name { field: Type } => "message"`. No proc macro,
/// no build dependency, and no run-time cost: building a value runs no code.
/// The complete grammar is in `docs/API.md`; what the macro expands to is in
/// `docs/GENERATED-CODE.md`.
///
/// # Variants and messages
///
/// ```
/// use define_err::define_errors;
///
/// define_errors! {
///     /// Errors from the page store.
///     #[derive(Clone, PartialEq, Eq)]
///     pub enum PageError {
///         NotFound { page: u64 } => "page {page} not found",
///         BadMagic { found: u32, expected: u32 } =>
///             "bad page magic: found {found:#010x}, expected {expected:#010x}",
///         ShortRead(page: u64, got: usize) => "short read on page {page}: got {got} bytes",
///         InvalidPageSize { size: usize } => ("invalid page size {size}: at most {max}", max = 65_536),
///         InvalidSuperblock => "page 0 is not a valid allocator superblock",
///     }
/// }
///
/// assert_eq!(PageError::NotFound { page: 7 }.to_string(), "page 7 not found");
/// assert_eq!(PageError::ShortRead(7, 12).to_string(), "short read on page 7: got 12 bytes");
/// ```
///
/// Tuple fields get a binding name (`ShortRead(page: u64, got: usize)`); the
/// variant is still a tuple variant. A field the message does not use may stay
/// unnamed (`Closed(u64)`).
///
/// # Sources, conversions, and transparent wrapping
///
/// ```
/// use define_err::{define_errors, Location};
///
/// define_errors! {
///     pub enum LoadError {
///         Io(#[from] source: std::io::Error, #[location] at: Location) => "cannot read the file: {source}",
///         Parse { line: u32, #[source] cause: core::num::ParseIntError } => "bad number on line {line}",
///         Format(#[from] std::fmt::Error) => transparent,
///     }
/// }
///
/// fn read(path: &str) -> Result<String, LoadError> {
///     Ok(std::fs::read_to_string(path)?)            // LoadError::Io, located at this line
/// }
///
/// assert!(read("/does/not/exist").is_err());
/// ```
///
/// A field named `source`, or marked `#[source]` or `#[from]`, is returned by
/// `Error::source()`. `#[from]` generates `From<FieldType>`; its `from` is
/// `#[track_caller]`, so a `#[location]` field records the line of the `?`.
/// `=> transparent` forwards `Display`, `source()`, and `ErrorInfo` to the
/// only field. Boxed sources (`Box<dyn Error + Send + Sync>` and the other
/// two forms) need this crate's `alloc` feature, which `std` turns on.
///
/// # Codes and metadata
///
/// ```
/// use define_err::prelude::*;
///
/// define_errors! {
///     namespace = "DB";
///     docs = "https://docs.hivedb.com/errors/{code}";
///
///     pub enum DbError {
///         #[code(1, name = TIMEOUT, help = "Raise pool.size or lower the load.")]
///         #[category(DeadlineExceeded)]
///         #[public("The database is busy. Try again shortly.")]
///         #[retry_after(secs = 2)]
///         Timeout { after_ms: u64 } => "query timed out after {after_ms} ms",
///
///         #[code(2, help = "Restore the page from a replica, then run a scrub.")]
///         #[category(DataLoss)]
///         #[kinds(ERROR | SECURITY)]
///         Corrupt { page: u64 } => "page {page} failed its checksum",
///     }
/// }
///
/// let err = DbError::Timeout { after_ms: 5000 };
/// assert_eq!(err.code(), Some(DbError::TIMEOUT));
/// assert_eq!(DbError::TIMEOUT.to_string(), "DB-0001");
/// assert_eq!(err.category().http_status(), 504);
/// assert_eq!(err.public_message(), "The database is busy. Try again shortly.");
/// assert_eq!(DbError::CATALOG.len(), 2);
/// ```
///
/// # Structs and type markers
///
/// ```
/// use define_err::define_errors;
///
/// define_errors! {
///     /// A syntax error in a NOML file.
///     pub struct SyntaxError { pub line: u32, pub column: u32 } => "syntax error at {line}:{column}";
///     pub struct Timeout(pub u64) => "operation timed out";
///     pub struct Cancelled => "operation cancelled";
///
///     #[exhaustive]
///     pub enum Parity {
///         Odd => "value is odd",
///         Even => "value is even",
///     }
/// }
///
/// assert_eq!(SyntaxError { line: 3, column: 9 }.to_string(), "syntax error at 3:9");
/// ```
///
/// Enums get `#[non_exhaustive]` unless marked `#[exhaustive]`; every type gets
/// `#[derive(Debug)]` unless marked `#[manual_debug]`. Other attributes on the
/// type are passed through. Metadata attributes go on variants, or on the
/// struct itself.
#[macro_export]
macro_rules! define_errors {
    ($($input:tt)*) => {
        $crate::__define_err_block! { @header [] [] [] $($input)* }
    };
}

/// Reads a block's header (`namespace`, `docs`, `retired`, in any order),
/// splits the block into types, and emits the block's checks at the end.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_block {
    (@header [] [$($docs:tt)*] [$($ret:tt)*] namespace = $ns:literal $(, width = $w:literal)? ; $($rest:tt)*) => {
        $crate::__define_err_block! { @header [$ns $($w)?] [$($docs)*] [$($ret)*] $($rest)* }
    };
    (@header [$($ns:tt)+] [$($docs:tt)*] [$($ret:tt)*] namespace = $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `namespace` is written twice in one block" }
    };
    (@header [$($ns:tt)*] [] [$($ret:tt)*] docs = $docs:literal ; $($rest:tt)*) => {
        $crate::__define_err_block! { @header [$($ns)*] [$docs] [$($ret)*] $($rest)* }
    };
    (@header [$($ns:tt)*] [$($docs:tt)+] [$($ret:tt)*] docs = $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `docs` is written twice in one block" }
    };
    (@header [$($ns:tt)*] [$($docs:tt)*] [] retired = [ $($r:literal),* $(,)? ] ; $($rest:tt)*) => {
        $crate::__define_err_block! { @header [$($ns)*] [$($docs)*] [retired $($r)*] $($rest)* }
    };
    (@header [$($ns:tt)*] [$($docs:tt)*] [$($ret:tt)+] retired = $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `retired` is written twice in one block" }
    };
    (@header [$($ns:tt)*] [$($docs:tt)*] [$(retired $($r:literal)*)?] $($rest:tt)*) => {
        $crate::__define_err_block! { @types [[$($ns)*] [$($docs)*] [$($($r)*)?] [$($($r)*)?] []] $($rest)* }
    };

    // No namespace: `docs` and `retired` mean nothing without one.
    (@types [[] [$($docs:tt)+] [$($rall:tt)*] [$($rf:tt)*] [$($acc:tt)*]]) => {
        ::core::compile_error! { "define_errors!: `docs = \"...\";` needs a `namespace = \"...\";` line in the same block" }
    };
    (@types [[] [] [$($rall:tt)+] [$($rf:tt)*] [$($acc:tt)*]]) => {
        ::core::compile_error! { "define_errors!: `retired = [...];` needs a `namespace = \"...\";` line in the same block" }
    };
    (@types [[] [] [] [$($rf:tt)*] [$($acc:tt)*]]) => {};

    // The block's checks, after its last type.
    (@types [[$ns:literal $($w:literal)?] [$($docs:literal)?] [$($rall:literal)*] [$($rf:tt)*] [$( { $code:tt } )*]]) => {
        const _: () = {
            $crate::__private::check_block(
                ::core::concat!("define_errors!: duplicate code number in namespace ", $ns),
                &[ $( $crate::__define_err_code!(@number $code) ),* ],
                &[ $( $rall ),* ],
                &[ $( $crate::__define_err_code!(@replacement $code) ),* ],
            );
            $( $crate::__private::check_docs($docs); )?
            #[deny(unreachable_patterns)]
            match 0_u32 {
                $( $crate::__define_err_code!(@number $code) => {} )*
                $( $rall => {} )*
                _ => {}
            }
        };
    };

    // The next type.
    (@types [$($ctx:tt)*] $($rest:tt)+) => {
        $crate::__define_err_type! {
            @attrs [$($ctx)*] [] [#[derive(::core::fmt::Debug)]] [#[non_exhaustive]] [] [[] [] [] [] [] [] [] []] $($rest)+
        }
    };
}

/// Reads one type's outer attributes, then hands the type to the enum or
/// struct parser together with the block context and the rest of the block.
///
/// State: `@attrs [block context] [passed-through attributes] [Debug derive,
/// unless #[manual_debug]] [#[non_exhaustive], unless #[exhaustive] or already
/// written] [`exhaustive` when #[exhaustive] was seen] [metadata slots, for
/// structs]`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_type {
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [] [$($meta:tt)*] #[exhaustive] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [] [exhaustive] [$($meta)*] $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)+] [$($meta:tt)*] #[exhaustive] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `#[exhaustive]` is written twice on one type" }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[manual_debug] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($ctx)*] [$($acc)*] [] [$($ne)*] [$($ex)*] [$($meta)*] $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[non_exhaustive] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($ctx)*] [$($acc)* #[non_exhaustive]] [$($debug)*] [] [$($ex)*] [$($meta)*] $($rest)* }
    };
    // Metadata attributes (meaningful on structs; an enum with them is an
    // error, reported when the enum is recognised).
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[code $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set code $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[category $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set category $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[severity $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set severity $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[kinds $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set kinds $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[public $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set public $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[action $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set action $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[retryable $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set retryable $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[retry_after $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set retry_after $($a)*; [$($meta)*];
            __define_err_type @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*]; $($rest)* }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] #[$($attr:tt)*] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($ctx)*] [$($acc)* #[$($attr)*]] [$($debug)*] [$($ne)*] [$($ex)*] [$($meta)*] $($rest)* }
    };

    // An enum: try the fast path, then the general path. Metadata belongs on
    // its variants.
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [[] [] [] [] [] [] [] []]
        $vis:vis enum $name:ident { $($body:tt)* } $($rest:tt)*) => {
        $crate::__define_err_enum! { @start [$($ctx)* [$($rest)*]] [$($acc)* $($debug)* $($ne)*] ($vis) $name { $($body)* } }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*]
        $vis:vis enum $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: metadata attributes (`#[code]`, `#[category]`, ...) go on the enum's variants, not on the enum" }
    };

    // Structs: braced, tuple, unit. `#[non_exhaustive]` is never added.
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [exhaustive] [$($meta:tt)*] $vis:vis struct $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `#[exhaustive]` applies only to enums; a struct is never made `#[non_exhaustive]`" }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [] [$($meta:tt)*]
        $vis:vis struct $name:ident { $($fields:tt)* } => $msg:tt ; $($rest:tt)*) => {
        $crate::__define_err_fields! {
            @f braced [struct [$($ctx)* [$($rest)*]] [$($acc)* $($debug)*] [$($meta)*] ($vis) $name braced $msg] [] [] [] []
            [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [] [$($meta:tt)*]
        $vis:vis struct $name:ident ( $($fields:tt)* ) => $msg:tt ; $($rest:tt)*) => {
        $crate::__define_err_fields! {
            @f tuple [struct [$($ctx)* [$($rest)*]] [$($acc)* $($debug)*] [$($meta)*] ($vis) $name tuple $msg] [] [] [] []
            [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [] [$($meta:tt)*]
        $vis:vis struct $name:ident => $msg:tt ; $($rest:tt)*) => {
        $crate::__define_err_emit! { @struct [$($ctx)* [$($rest)*]] [$($acc)* $($debug)*] [$($meta)*] ($vis) $name => $msg }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*]
        $vis:vis struct $name:ident $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: a struct needs `=> \"message\";` after its fields, for example `pub struct Timeout(pub u64) => \"operation timed out\";`"
        }
    };
    (@attrs [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] [$($meta:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: expected an `enum` or a `struct` definition, for example `pub enum MyError { NotFound { id: u64 } => \"item {id} not found\" }`"
        }
    };
}

/// Sets one of the eight metadata slots, then returns to the caller:
/// `@set key argument; [slots]; caller-macro @state [state...]; rest`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_meta_slot {
    (@set code $a:tt; [[] $($s:tt)*]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [[$a] $($s)*] $($state)* }
    };
    (@set category $a:tt; [$s1:tt [] $($s:tt)*]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 [$a] $($s)*] $($state)* }
    };
    (@set severity $a:tt; [$s1:tt $s2:tt [] $($s:tt)*]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 $s2 [$a] $($s)*] $($state)* }
    };
    (@set kinds $a:tt; [$s1:tt $s2:tt $s3:tt [] $($s:tt)*]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 $s2 $s3 [$a] $($s)*] $($state)* }
    };
    (@set public $a:tt; [$s1:tt $s2:tt $s3:tt $s4:tt [] $($s:tt)*]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 $s2 $s3 $s4 [$a] $($s)*] $($state)* }
    };
    (@set action $a:tt; [$s1:tt $s2:tt $s3:tt $s4:tt $s5:tt [] $($s:tt)*]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 $s2 $s3 $s4 $s5 [$a] $($s)*] $($state)* }
    };
    (@set retryable; [$s1:tt $s2:tt $s3:tt $s4:tt $s5:tt $s6:tt [] $s8:tt]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 $s2 $s3 $s4 $s5 $s6 [()] $s8] $($state)* }
    };
    (@set retryable $a:tt; [$s1:tt $s2:tt $s3:tt $s4:tt $s5:tt $s6:tt [] $s8:tt]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 $s2 $s3 $s4 $s5 $s6 [($a)] $s8] $($state)* }
    };
    (@set retry_after $a:tt; [$s1:tt $s2:tt $s3:tt $s4:tt $s5:tt $s6:tt $s7:tt []]; $m:ident @$st:ident $($state:tt)*) => {
        $crate::__define_err_meta_slot! { @back $m @$st [$s1 $s2 $s3 $s4 $s5 $s6 $s7 [$a]] $($state)* }
    };
    (@set code; $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `#[code]` needs a number and help text, for example `#[code(1, help = \"Raise pool.size.\")]`" }
    };
    (@set $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: a metadata attribute is written twice or without its argument; each of `#[code]`, `#[category]`, `#[severity]`, `#[kinds]`, `#[public]`, `#[action]`, `#[retryable]`, `#[retry_after]` is written at most once"
        }
    };

    // Back to the caller, with the slots in the position it keeps them.
    (@back __define_err_type @attrs [$($meta:tt)*] [$($ctx:tt)*] [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*]; $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($ctx)*] [$($acc)*] [$($debug)*] [$($ne)*] [$($ex)*] [$($meta)*] $($rest)* }
    };
    (@back __define_err_enum @v [$($meta:tt)*] [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*]; $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*] [$($meta)*] $($rest)* }
    };
}

/// Parses an enum body.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_enum {
    // Fast path: the whole enum in canonical form, in one non-recursive arm.
    (@start [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident { $(
        $(#[doc $($doc:tt)*])*
        $(#[code $code:tt])?
        $(#[category $cat:tt])?
        $(#[severity $sev:tt])?
        $(#[kinds $kinds:tt])?
        $(#[public $public:tt])?
        $(#[action $action:tt])?
        $(#[retryable $($retry:tt)?])?
        $(#[retry_after $ra:tt])?
        $vn:ident
        $({ $(
            $(#[doc $($bdoc:tt)*])*
            $(#[from] $bfrom:ident)?
            $(#[source] $bsrc:ident)?
            $(#[location] $bloc:ident)?
            $(#[retry_after] $bra:ident)?
            $($bplain:ident)?
            : $bty:ty
        ),* $(,)? })?
        $(( $(
            $(#[doc $($tdoc:tt)*])*
            $(#[from] $tfrom:ident)?
            $(#[source] $tsrc:ident)?
            $(#[location] $tloc:ident)?
            $(#[retry_after] $tra:ident)?
            $($tplain:ident)?
            : $tty:ty
        ),* $(,)? ))?
        => $msg:tt
    ),* $(,)? }) => {
        $crate::__define_err_emit! { @enum [$($ctx)*] [$($tattr)*] ($vis) $name [ $(
            ( [$(#[doc $($doc)*])*]
              [$($code)?] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?] [$( ( $($retry)? ) )?] [$($ra)?]
              $vn
              $({ $( { [$(#[doc $($bdoc)*])*] []
                  $(from $bfrom)? $(source $bsrc)? $(location $bloc)? $(retry_after $bra)? $(plain $bplain)? : $bty } )* })?
              $(( $( { [$(#[doc $($tdoc)*])*] []
                  $(from $tfrom)? $(source $tsrc)? $(location $tloc)? $(retry_after $tra)? $(plain $tplain)? : $tty } )* ))?
              => $msg )
        )* ] }
    };

    // General path.
    (@start [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident { $($body:tt)* }) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [] [] [[] [] [] [] [] [] [] []] $($body)* }
    };

    // All variants read: emit.
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [] [[] [] [] [] [] [] [] []]) => {
        $crate::__define_err_emit! { @enum [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*]) => {
        ::core::compile_error! { "define_errors!: attributes at the end of an enum belong to no variant" }
    };

    // One variant already in canonical form: one step.
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [] [[] [] [] [] [] [] [] []]
        $(#[doc $($doc:tt)*])*
        $(#[code $code:tt])?
        $(#[category $cat:tt])?
        $(#[severity $sev:tt])?
        $(#[kinds $kinds:tt])?
        $(#[public $public:tt])?
        $(#[action $action:tt])?
        $(#[retryable $($retry:tt)?])?
        $(#[retry_after $ra:tt])?
        $vn:ident
        $({ $(
            $(#[doc $($bdoc:tt)*])*
            $(#[from] $bfrom:ident)?
            $(#[source] $bsrc:ident)?
            $(#[location] $bloc:ident)?
            $(#[retry_after] $bra:ident)?
            $($bplain:ident)?
            : $bty:ty
        ),* $(,)? })?
        $(( $(
            $(#[doc $($tdoc:tt)*])*
            $(#[from] $tfrom:ident)?
            $(#[source] $tsrc:ident)?
            $(#[location] $tloc:ident)?
            $(#[retry_after] $tra:ident)?
            $($tplain:ident)?
            : $tty:ty
        ),* $(,)? ))?
        => $msg:tt
        $(, $($rest:tt)*)?
    ) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*
            ( [$(#[doc $($doc)*])*]
              [$($code)?] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?] [$( ( $($retry)? ) )?] [$($ra)?]
              $vn
              $({ $( { [$(#[doc $($bdoc)*])*] []
                  $(from $bfrom)? $(source $bsrc)? $(location $bloc)? $(retry_after $bra)? $(plain $bplain)? : $bty } )* })?
              $(( $( { [$(#[doc $($tdoc)*])*] []
                  $(from $tfrom)? $(source $tsrc)? $(location $tloc)? $(retry_after $tra)? $(plain $tplain)? : $tty } )* ))?
              => $msg )
        ] [] [[] [] [] [] [] [] [] []] $($($rest)*)? }
    };

    // Variant attributes, in any order: passed through, or metadata.
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[doc $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[doc $($a)*]] [$($meta)*] $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[allow $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[allow $($a)*]] [$($meta)*] $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[warn $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[warn $($a)*]] [$($meta)*] $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[deny $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[deny $($a)*]] [$($meta)*] $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[forbid $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[forbid $($a)*]] [$($meta)*] $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[expect $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[expect $($a)*]] [$($meta)*] $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[deprecated $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[deprecated $($a)*]] [$($meta)*] $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[code $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set code $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[category $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set category $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[severity $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set severity $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[kinds $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set kinds $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[public $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set public $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[action $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set action $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[retryable $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set retryable $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[retry_after $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_meta_slot! { @set retry_after $($a)*; [$($meta)*]; __define_err_enum @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*]; $($rest)* }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] #[$($a:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: unknown attribute on a variant; valid here are doc comments, `#[allow]`, `#[warn]`, `#[deny]`, `#[forbid]`, `#[expect]`, `#[deprecated]`, `#[code]`, `#[category]`, `#[severity]`, `#[kinds]`, `#[public]`, `#[action]`, `#[retryable]`, and `#[retry_after]`"
        }
    };

    // Variant bodies.
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*]
        $vn:ident { $($fields:tt)* } => $msg:tt $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! {
            @f braced [enum [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*] [$($meta)*] $vn braced $msg [$($($rest)*)?]]
            [] [] [] []
            [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*]
        $vn:ident ( $($fields:tt)* ) => $msg:tt $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! {
            @f tuple [enum [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*] [$($meta)*] $vn tuple $msg [$($($rest)*)?]]
            [] [] [] []
            [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*]
        $vn:ident => $msg:tt $(, $($rest:tt)*)?) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)* ( [$($va)*] $($meta)* $vn => $msg )] [] [[] [] [] [] [] [] [] []] $($($rest)*)? }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] $vn:ident $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: every variant needs `=> \"message\"` after its fields, for example `NotFound { id: u64 } => \"item {id} not found\"`"
        }
    };
    (@v [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: expected a variant: `Name => \"message\"`, `Name { field: Type } => \"message\"`, or `Name(binding: Type) => \"message\"`"
        }
    };

    // Back from the field parser with one variant's canonical fields.
    (@done [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] $vn:ident braced $msg:tt [$($rest:tt)*] [$($fields:tt)*]) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)* ( [$($va)*] $($meta)* $vn { $($fields)* } => $msg )] [] [[] [] [] [] [] [] [] []] $($rest)* }
    };
    (@done [$($ctx:tt)*] [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] [$($meta:tt)*] $vn:ident tuple $msg:tt [$($rest:tt)*] [$($fields:tt)*]) => {
        $crate::__define_err_enum! { @v [$($ctx)*] [$($tattr)*] ($vis) $name [$($acc)* ( [$($va)*] $($meta)* $vn ( $($fields)* ) => $msg )] [] [[] [] [] [] [] [] [] []] $($rest)* }
    };
}

/// Parses the fields of one variant or struct into canonical fields
/// `{ [attributes] [visibility] role name : Type }`, then returns them to the
/// caller named in the continuation (`enum ...` or `struct ...`).
///
/// State: `@f kind [continuation] [done fields] [pending attributes] [role]
/// [unused] [names for unnamed fields]`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_fields {
    // Field attributes, in any order.
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)*] [] [$($pool:tt)*] #[doc $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_fields! { @f $kind [$($cont)*] [$($done)*] [$($fa)* #[doc $($a)*]] [$($role)*] [] [$($pool)*] $($rest)* }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [] [] [$($pool:tt)*] #[from] $($rest:tt)*) => {
        $crate::__define_err_fields! { @f $kind [$($cont)*] [$($done)*] [$($fa)*] [from] [] [$($pool)*] $($rest)* }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [] [] [$($pool:tt)*] #[source] $($rest:tt)*) => {
        $crate::__define_err_fields! { @f $kind [$($cont)*] [$($done)*] [$($fa)*] [source] [] [$($pool)*] $($rest)* }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [] [] [$($pool:tt)*] #[location] $($rest:tt)*) => {
        $crate::__define_err_fields! { @f $kind [$($cont)*] [$($done)*] [$($fa)*] [location] [] [$($pool)*] $($rest)* }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [] [] [$($pool:tt)*] #[retry_after] $($rest:tt)*) => {
        $crate::__define_err_fields! { @f $kind [$($cont)*] [$($done)*] [$($fa)*] [retry_after] [] [$($pool)*] $($rest)* }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)+] [] [$($pool:tt)*] #[from] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a field has more than one of `#[from]`, `#[source]`, `#[location]`, `#[retry_after]`; `#[from]` already implies `#[source]`" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)+] [] [$($pool:tt)*] #[source] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a field has more than one of `#[from]`, `#[source]`, `#[location]`, `#[retry_after]`; `#[from]` already implies `#[source]`" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)+] [] [$($pool:tt)*] #[location] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a field has more than one of `#[from]`, `#[source]`, `#[location]`, `#[retry_after]`" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)+] [] [$($pool:tt)*] #[retry_after] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a field has more than one of `#[from]`, `#[source]`, `#[location]`, `#[retry_after]`" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)*] [] [$($pool:tt)*] #[$($a:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: unknown attribute on a field; valid here are doc comments, `#[from]`, `#[source]`, `#[location]`, and `#[retry_after]`"
        }
    };

    // A named field: `name: Type` (braced), `binding: Type` (tuple). An
    // unmarked field is `plain`.
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [] [] [$($pool:tt)*]
        $fv:vis $n:ident : $t:ty $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! { @f $kind [$($cont)*] [$($done)* { [$($fa)*] [$fv] plain $n : $t }] [] [] [] [$($pool)*] $($($rest)*)? }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$role:ident] [] [$($pool:tt)*]
        $fv:vis $n:ident : $t:ty $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! { @f $kind [$($cont)*] [$($done)* { [$($fa)*] [$fv] $role $n : $t }] [] [] [] [$($pool)*] $($($rest)*)? }
    };

    // An unnamed tuple field: sources, locations, and retry delays get a
    // fixed name (a variant has at most one of each); other fields take the
    // next of the 16 names the caller passed in.
    (@f tuple [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [from] [] [$($pool:tt)*]
        $fv:vis $t:ty $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! { @f tuple [$($cont)*] [$($done)* { [$($fa)*] [$fv] from __define_err_source : $t }] [] [] [] [$($pool)*] $($($rest)*)? }
    };
    (@f tuple [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [source] [] [$($pool:tt)*]
        $fv:vis $t:ty $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! { @f tuple [$($cont)*] [$($done)* { [$($fa)*] [$fv] source __define_err_source : $t }] [] [] [] [$($pool)*] $($($rest)*)? }
    };
    (@f tuple [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [location] [] [$($pool:tt)*]
        $fv:vis $t:ty $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! { @f tuple [$($cont)*] [$($done)* { [$($fa)*] [$fv] location __define_err_location : $t }] [] [] [] [$($pool)*] $($($rest)*)? }
    };
    (@f tuple [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [retry_after] [] [$($pool:tt)*]
        $fv:vis $t:ty $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! { @f tuple [$($cont)*] [$($done)* { [$($fa)*] [$fv] retry_after __define_err_retry_after : $t }] [] [] [] [$($pool)*] $($($rest)*)? }
    };
    (@f tuple [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [] [] [$next:ident $($pool:tt)*]
        $fv:vis $t:ty $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! { @f tuple [$($cont)*] [$($done)* { [$($fa)*] [$fv] plain $next : $t }] [] [] [] [$($pool)*] $($($rest)*)? }
    };
    (@f tuple [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [] [] []
        $($rest:tt)+) => {
        ::core::compile_error! { "define_errors!: a tuple variant has more than 16 unnamed fields; name them (`binding: Type`) or use a braced variant" }
    };

    // All fields read: return to the caller.
    (@f $kind:ident [enum $($cont:tt)*] [$($done:tt)*] [] [] [] [$($pool:tt)*]) => {
        $crate::__define_err_enum! { @done $($cont)* [$($done)*] }
    };
    (@f $kind:ident [struct [$($ctx:tt)*] [$($tattr:tt)*] [$($meta:tt)*] ($vis:vis) $name:ident braced $msg:tt] [$($done:tt)*] [] [] [] [$($pool:tt)*]) => {
        $crate::__define_err_emit! { @struct [$($ctx)*] [$($tattr)*] [$($meta)*] ($vis) $name { $($done)* } => $msg }
    };
    (@f $kind:ident [struct [$($ctx:tt)*] [$($tattr:tt)*] [$($meta:tt)*] ($vis:vis) $name:ident tuple $msg:tt] [$($done:tt)*] [] [] [] [$($pool:tt)*]) => {
        $crate::__define_err_emit! { @struct [$($ctx)*] [$($tattr)*] [$($meta)*] ($vis) $name ( $($done)* ) => $msg }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)*] [] [$($pool:tt)*]) => {
        ::core::compile_error! { "define_errors!: attributes after the last field belong to no field" }
    };
    (@f braced [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)*] [] [$($pool:tt)*] $($rest:tt)+) => {
        ::core::compile_error! { "define_errors!: expected a field `name: Type` in a braced variant or struct" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)*] [] [$($pool:tt)*] $($rest:tt)+) => {
        ::core::compile_error! { "define_errors!: expected a tuple field `binding: Type` or `Type`" }
    };
}
