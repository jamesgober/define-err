//! `define_errors!`: the entry point and the parser.
//!
//! The parser has two paths that end in the same emitter (`emit.rs`):
//!
//! - **Fast path.** One arm matches a whole enum written in the canonical
//!   order (doc comments first on variants and fields, field markers written
//!   right before the field name, tuple fields named) and hands every variant
//!   to the emitter through repetitions. It does not recurse, so an enum of any
//!   size compiles at the default `recursion_limit`.
//! - **General path.** Anything else (other variant attributes, attributes in
//!   other orders, unnamed tuple fields, structs) is normalised one variant at
//!   a time. A variant already in canonical form costs one step; an irregular
//!   one costs one step per attribute and field. Recursion depth therefore
//!   grows with the number of variants the fast path could not take.
//!
//! Matching rules that shape the patterns: a `tt`, `ident`, or `vis` fragment
//! may never compete with a literal token at the same position (rustc reports
//! "local ambiguity"), and a `ty` fragment may only be followed by `,`, `=>`,
//! `:`, `;`, `=`, `|`, `>`, `{`, `[`, `as`, or `where`. Every slot below is
//! therefore led by a literal token (`#[doc`, `#[from]`, ...), and field
//! markers capture the field name, with the type in one shared tail.

/// Declares error types: enums and structs with `Display`, `core::error::Error`
/// with `source()`, `From` conversions, `Debug`, and `ErrorInfo`.
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
/// type are passed through.
#[macro_export]
macro_rules! define_errors {
    ($($input:tt)*) => {
        $crate::__define_err_block! { $($input)* }
    };
}

/// Splits a block into its type definitions.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_block {
    () => {};
    ($($input:tt)+) => {
        $crate::__define_err_type! {
            @attrs [] [#[derive(::core::fmt::Debug)]] [#[non_exhaustive]] [] $($input)+
        }
    };
}

/// Reads one type's outer attributes, then hands the type to the enum or
/// struct parser and the rest of the block back to `__define_err_block!`.
///
/// State: `@attrs [passed-through attributes] [Debug derive, unless
/// #[manual_debug]] [#[non_exhaustive], unless #[exhaustive] or already
/// written] [`exhaustive` when #[exhaustive] was seen]`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_type {
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [] #[exhaustive] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($acc)*] [$($debug)*] [] [exhaustive] $($rest)* }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)+] #[exhaustive] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `#[exhaustive]` is written twice on one type" }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] #[manual_debug] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($acc)*] [] [$($ne)*] [$($ex)*] $($rest)* }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] #[non_exhaustive] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($acc)* #[non_exhaustive]] [$($debug)*] [] [$($ex)*] $($rest)* }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] #[$($attr:tt)*] $($rest:tt)*) => {
        $crate::__define_err_type! { @attrs [$($acc)* #[$($attr)*]] [$($debug)*] [$($ne)*] [$($ex)*] $($rest)* }
    };

    // An enum: try the fast path, then the general path.
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*]
        $vis:vis enum $name:ident { $($body:tt)* } $($rest:tt)*) => {
        $crate::__define_err_enum! { @start [$($acc)* $($debug)* $($ne)*] ($vis) $name { $($body)* } }
        $crate::__define_err_block! { $($rest)* }
    };

    // Structs: braced, tuple, unit. `#[non_exhaustive]` is never added.
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [exhaustive] $vis:vis struct $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: `#[exhaustive]` applies only to enums; a struct is never made `#[non_exhaustive]`" }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] []
        $vis:vis struct $name:ident { $($fields:tt)* } => $msg:tt ; $($rest:tt)*) => {
        $crate::__define_err_fields! {
            @f braced [struct [$($acc)* $($debug)*] ($vis) $name braced $msg] [] [] [] [] [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
        $crate::__define_err_block! { $($rest)* }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] []
        $vis:vis struct $name:ident ( $($fields:tt)* ) => $msg:tt ; $($rest:tt)*) => {
        $crate::__define_err_fields! {
            @f tuple [struct [$($acc)* $($debug)*] ($vis) $name tuple $msg] [] [] [] [] [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
        $crate::__define_err_block! { $($rest)* }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] []
        $vis:vis struct $name:ident => $msg:tt ; $($rest:tt)*) => {
        $crate::__define_err_emit! { @struct [$($acc)* $($debug)*] ($vis) $name => $msg }
        $crate::__define_err_block! { $($rest)* }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*]
        $vis:vis struct $name:ident $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: a struct needs `=> \"message\";` after its fields, for example `pub struct Timeout(pub u64) => \"operation timed out\";`"
        }
    };
    (@attrs [$($acc:tt)*] [$($debug:tt)*] [$($ne:tt)*] [$($ex:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: expected an `enum` or a `struct` definition, for example `pub enum MyError { NotFound { id: u64 } => \"item {id} not found\" }`"
        }
    };
}

/// Parses an enum body.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_enum {
    // Fast path: the whole enum in canonical form, in one non-recursive arm.
    (@start [$($tattr:tt)*] ($vis:vis) $name:ident { $(
        $(#[doc $($doc:tt)*])*
        $vn:ident
        $({ $(
            $(#[doc $($bdoc:tt)*])*
            $(#[from] $bfrom:ident)?
            $(#[source] $bsrc:ident)?
            $(#[location] $bloc:ident)?
            $($bplain:ident)?
            : $bty:ty
        ),* $(,)? })?
        $(( $(
            $(#[doc $($tdoc:tt)*])*
            $(#[from] $tfrom:ident)?
            $(#[source] $tsrc:ident)?
            $(#[location] $tloc:ident)?
            $($tplain:ident)?
            : $tty:ty
        ),* $(,)? ))?
        => $msg:tt
    ),* $(,)? }) => {
        $crate::__define_err_emit! { @enum [$($tattr)*] ($vis) $name [ $(
            ( [$(#[doc $($doc)*])*] $vn
              $({ $( { [$(#[doc $($bdoc)*])*] []
                  $(from $bfrom)? $(source $bsrc)? $(location $bloc)? $(plain $bplain)? : $bty } )* })?
              $(( $( { [$(#[doc $($tdoc)*])*] []
                  $(from $tfrom)? $(source $tsrc)? $(location $tloc)? $(plain $tplain)? : $tty } )* ))?
              => $msg )
        )* ] }
    };

    // General path.
    (@start [$($tattr:tt)*] ($vis:vis) $name:ident { $($body:tt)* }) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [] [] $($body)* }
    };

    // All variants read: emit.
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] []) => {
        $crate::__define_err_emit! { @enum [$($tattr)*] ($vis) $name [$($acc)*] }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)+]) => {
        ::core::compile_error! { "define_errors!: attributes at the end of an enum belong to no variant" }
    };

    // One variant already in canonical form: one step.
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] []
        $(#[doc $($doc:tt)*])*
        $vn:ident
        $({ $(
            $(#[doc $($bdoc:tt)*])*
            $(#[from] $bfrom:ident)?
            $(#[source] $bsrc:ident)?
            $(#[location] $bloc:ident)?
            $($bplain:ident)?
            : $bty:ty
        ),* $(,)? })?
        $(( $(
            $(#[doc $($tdoc:tt)*])*
            $(#[from] $tfrom:ident)?
            $(#[source] $tsrc:ident)?
            $(#[location] $tloc:ident)?
            $($tplain:ident)?
            : $tty:ty
        ),* $(,)? ))?
        => $msg:tt
        $(, $($rest:tt)*)?
    ) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*
            ( [$(#[doc $($doc)*])*] $vn
              $({ $( { [$(#[doc $($bdoc)*])*] []
                  $(from $bfrom)? $(source $bsrc)? $(location $bloc)? $(plain $bplain)? : $bty } )* })?
              $(( $( { [$(#[doc $($tdoc)*])*] []
                  $(from $tfrom)? $(source $tsrc)? $(location $tloc)? $(plain $tplain)? : $tty } )* ))?
              => $msg )
        ] [] $($($rest)*)? }
    };

    // Variant attributes, in any order.
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[doc $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[doc $($a)*]] $($rest)* }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[allow $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[allow $($a)*]] $($rest)* }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[warn $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[warn $($a)*]] $($rest)* }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[deny $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[deny $($a)*]] $($rest)* }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[forbid $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[forbid $($a)*]] $($rest)* }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[expect $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[expect $($a)*]] $($rest)* }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[deprecated $($a:tt)*] $($rest:tt)*) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)*] [$($va)* #[deprecated $($a)*]] $($rest)* }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] #[$($a:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: unknown attribute on a variant; valid here are doc comments, `#[allow]`, `#[warn]`, `#[deny]`, `#[forbid]`, `#[expect]`, and `#[deprecated]`"
        }
    };

    // Variant bodies.
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*]
        $vn:ident { $($fields:tt)* } => $msg:tt $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! {
            @f braced [enum [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*] $vn braced $msg [$($($rest)*)?]]
            [] [] [] [] [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*]
        $vn:ident ( $($fields:tt)* ) => $msg:tt $(, $($rest:tt)*)?) => {
        $crate::__define_err_fields! {
            @f tuple [enum [$($tattr)*] ($vis) $name [$($acc)*] [$($va)*] $vn tuple $msg [$($($rest)*)?]]
            [] [] [] [] [__define_err_0 __define_err_1 __define_err_2 __define_err_3 __define_err_4 __define_err_5 __define_err_6 __define_err_7 __define_err_8 __define_err_9 __define_err_10 __define_err_11 __define_err_12 __define_err_13 __define_err_14 __define_err_15] $($fields)*
        }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*]
        $vn:ident => $msg:tt $(, $($rest:tt)*)?) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)* ( [$($va)*] $vn => $msg )] [] $($($rest)*)? }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] $vn:ident $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: every variant needs `=> \"message\"` after its fields, for example `NotFound { id: u64 } => \"item {id} not found\"`"
        }
    };
    (@v [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: expected a variant: `Name => \"message\"`, `Name { field: Type } => \"message\"`, or `Name(binding: Type) => \"message\"`"
        }
    };

    // Back from the field parser with one variant's canonical fields.
    (@done [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] $vn:ident braced $msg:tt [$($rest:tt)*] [$($fields:tt)*]) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)* ( [$($va)*] $vn { $($fields)* } => $msg )] [] $($rest)* }
    };
    (@done [$($tattr:tt)*] ($vis:vis) $name:ident [$($acc:tt)*] [$($va:tt)*] $vn:ident tuple $msg:tt [$($rest:tt)*] [$($fields:tt)*]) => {
        $crate::__define_err_enum! { @v [$($tattr)*] ($vis) $name [$($acc)* ( [$($va)*] $vn ( $($fields)* ) => $msg )] [] $($rest)* }
    };
}

/// Parses the fields of one variant or struct into canonical fields
/// `{ [attributes] [visibility] role name : Type }`, then returns them to the
/// caller named in the continuation (`enum ...` or `struct ...`).
///
/// State: `@f kind [continuation] [done fields] [pending attributes] [role]
/// [pending visibility, unused] [unused names for unnamed fields]`.
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
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)+] [] [$($pool:tt)*] #[from] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a field has more than one of `#[from]`, `#[source]`, `#[location]`; `#[from]` already implies `#[source]`" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)+] [] [$($pool:tt)*] #[source] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a field has more than one of `#[from]`, `#[source]`, `#[location]`; `#[from]` already implies `#[source]`" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)+] [] [$($pool:tt)*] #[location] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a field has more than one of `#[from]`, `#[source]`, `#[location]`" }
    };
    (@f $kind:ident [$($cont:tt)*] [$($done:tt)*] [$($fa:tt)*] [$($role:tt)*] [] [$($pool:tt)*] #[$($a:tt)*] $($rest:tt)*) => {
        ::core::compile_error! {
            "define_errors!: unknown attribute on a field; valid here are doc comments, `#[from]`, `#[source]`, and `#[location]`"
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

    // An unnamed tuple field: sources and locations get a fixed name (a
    // variant has at most one of each); other fields take the next of the 16
    // names the caller passed in.
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
    (@f $kind:ident [struct [$($tattr:tt)*] ($vis:vis) $name:ident braced $msg:tt] [$($done:tt)*] [] [] [] [$($pool:tt)*]) => {
        $crate::__define_err_emit! { @struct [$($tattr)*] ($vis) $name { $($done)* } => $msg }
    };
    (@f $kind:ident [struct [$($tattr:tt)*] ($vis:vis) $name:ident tuple $msg:tt] [$($done:tt)*] [] [] [] [$($pool:tt)*]) => {
        $crate::__define_err_emit! { @struct [$($tattr)*] ($vis) $name ( $($done)* ) => $msg }
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
