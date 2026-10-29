//! The emitter: canonical variants in, items out (`docs/GENERATED-CODE.md`).
//!
//! Canonical input, produced by both parser paths:
//!
//! ```text
//! @enum [type attributes] (vis) Name [ (
//!     [variant attributes] Variant
//!     { { [field attributes] [field vis] role name : Type } ... }     // braced, or
//!     ( { [field attributes] [field vis] role name : Type } ... )     // tuple, or nothing (unit)
//!     => message
//! ) ... ]
//! @struct [type attributes] (vis) Name { fields } => message           // or ( fields ), or nothing
//! ```
//!
//! `role` is `from`, `source`, `location`, or `plain`. Every field has a name:
//! unnamed tuple fields were named by the parser. Every generated arm binds
//! all fields by reference (`ref name`), so the per-variant helpers below can
//! use any of them.

/// Emits the items for one type from its canonical form.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_emit {
    (@enum [$($tattr:tt)*] ($vis:vis) $name:ident [ $(
        ( [$($vattr:tt)*] $vn:ident
          $({ $( { [$($fattr:tt)*] [$($fvis:tt)*] $frole:ident $fname:ident : $fty:ty } )* })?
          $(( $( { [$($xattr:tt)*] [$($xvis:tt)*] $xrole:ident $xname:ident : $xty:ty } )* ))?
          => $msg:tt )
    )* ]) => {
        $($tattr)*
        $vis enum $name { $(
            $($vattr)*
            $vn
            $({ $( $($fattr)* $fname: $fty, )* })?
            $(( $( $($xattr)* $xty, )* ))?,
        )* }

        $crate::__define_err_emit! { @impls $name [ $(
            ( [Self::$vn] [::core::concat!(::core::stringify!($name), "::", ::core::stringify!($vn))]
              $({ $( { $frole $fname : $fty } )* })?
              $(( $( { $xrole $xname : $xty } )* ))?
              => $msg )
        )* ] }
    };

    (@struct [$($tattr:tt)*] ($vis:vis) $name:ident
        { $( { [$($fattr:tt)*] [$($fvis:tt)*] $frole:ident $fname:ident : $fty:ty } )* } => $msg:tt) => {
        $($tattr)*
        $vis struct $name { $( $($fattr)* $($fvis)* $fname: $fty, )* }

        $crate::__define_err_emit! { @impls $name [
            ( [Self] [::core::stringify!($name)] { $( { $frole $fname : $fty } )* } => $msg )
        ] }
    };
    (@struct [$($tattr:tt)*] ($vis:vis) $name:ident
        ( $( { [$($fattr:tt)*] [$($fvis:tt)*] $frole:ident $fname:ident : $fty:ty } )* ) => $msg:tt) => {
        $($tattr)*
        $vis struct $name ( $( $($fattr)* $($fvis)* $fty, )* );

        $crate::__define_err_emit! { @impls $name [
            ( [Self] [::core::stringify!($name)] ( $( { $frole $fname : $fty } )* ) => $msg )
        ] }
    };
    (@struct [$($tattr:tt)*] ($vis:vis) $name:ident => $msg:tt) => {
        $($tattr)*
        $vis struct $name;

        $crate::__define_err_emit! { @impls $name [ ( [Self] [::core::stringify!($name)] => $msg ) ] }
    };

    (@impls $name:ident [ $(
        ( [$($path:tt)*] [$($tname:tt)*]
          $({ $( { $frole:ident $fname:ident : $fty:ty } )* })?
          $(( $( { $xrole:ident $xname:ident : $xty:ty } )* ))?
          => $msg:tt )
    )* ]) => {
        #[allow(deprecated)]
        impl ::core::fmt::Display for $name {
            fn fmt(&self, __define_err_f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_fmt!(__define_err_f; $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }
        }

        #[allow(deprecated)]
        impl ::core::error::Error for $name {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_source!($msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }
        }

        #[allow(deprecated)]
        impl $crate::ErrorInfo for $name {
            fn location(&self) -> ::core::option::Option<$crate::Location> {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_location!($msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn type_name(&self) -> &'static str {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_type_name!($msg; [$($tname)*]; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }
        }

        $(
            $crate::__define_err_from! {
                $name [$($path)*]
                $( braced $( $frole $fname : $fty ; )* )?
                $( tuple $( $xrole $xname : $xty ; )* )?
            }
        )*
    };

    ($($input:tt)*) => {
        ::core::compile_error! {
            "define_errors!: a field must be `name: Type` with at most one of `#[from]`, `#[source]`, `#[location]` before it"
        }
    };
}

/// The body of one `Display` arm.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_fmt {
    ($f:ident; $lit:literal; $($roles:tt)*) => {
        ::core::write!($f, $lit)
    };
    ($f:ident; ($lit:literal $(, $($args:tt)*)?); $($roles:tt)*) => {
        ::core::write!($f, $lit $(, $($args)*)?)
    };
    ($f:ident; transparent; $($roles:tt)*) => {
        $crate::__define_err_transparent!(@fmt [$f]; $($roles)*)
    };
    ($f:ident; $($other:tt)*) => {
        ::core::compile_error! {
            "define_errors!: after `=>` write a string literal (`\"page {page} not found\"`), a format string with arguments in parentheses (`(\"{}\", value.format())`), or `transparent`"
        }
    };
}

/// The body of one `source()` arm.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_source {
    (@find [$($found:ident)*] from $n:ident $($rest:tt)*) => {
        $crate::__define_err_source!(@find [$($found)* $n] $($rest)*)
    };
    (@find [$($found:ident)*] source $n:ident $($rest:tt)*) => {
        $crate::__define_err_source!(@find [$($found)* $n] $($rest)*)
    };
    // A field named `source` is the source without an attribute. The name is
    // passed twice so the found name is the user's own token (hygiene).
    (@find [$($found:ident)*] plain $n:ident $($rest:tt)*) => {
        $crate::__define_err_source!(@named [$($found)*] [$n] $n $($rest)*)
    };
    (@named [$($found:ident)*] [$n:ident] source $($rest:tt)*) => {
        $crate::__define_err_source!(@find [$($found)* $n] $($rest)*)
    };
    (@named [$($found:ident)*] [$n:ident] $other:ident $($rest:tt)*) => {
        $crate::__define_err_source!(@find [$($found)*] $($rest)*)
    };
    (@find [$($found:ident)*] $role:ident $n:ident $($rest:tt)*) => {
        $crate::__define_err_source!(@find [$($found)*] $($rest)*)
    };
    (@find []) => {
        ::core::option::Option::None
    };
    (@find [$n:ident]) => {
        $crate::__private::source_of!($n)
    };
    (@find [$a:ident $b:ident $($more:ident)*]) => {
        ::core::compile_error! {
            "define_errors!: a variant has more than one source field; keep one `#[source]`, `#[from]`, or field named `source`"
        }
    };
    (transparent; $($roles:tt)*) => {
        $crate::__define_err_transparent!(@source []; $($roles)*)
    };
    ($msg:tt; $($roles:tt)*) => {
        $crate::__define_err_source!(@find [] $($roles)*)
    };
}

/// The body of one `ErrorInfo::location` arm.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_location {
    (@find [$($found:ident)*] location $n:ident $($rest:tt)*) => {
        $crate::__define_err_location!(@find [$($found)* $n] $($rest)*)
    };
    (@find [$($found:ident)*] $role:ident $n:ident $($rest:tt)*) => {
        $crate::__define_err_location!(@find [$($found)*] $($rest)*)
    };
    (@find []) => {
        ::core::option::Option::None
    };
    (@find [$n:ident]) => {
        ::core::option::Option::Some(*$n)
    };
    (@find [$a:ident $b:ident $($more:ident)*]) => {
        ::core::compile_error! { "define_errors!: a variant has more than one `#[location]` field" }
    };
    (transparent; $($roles:tt)*) => {
        $crate::__define_err_transparent!(@location []; $($roles)*)
    };
    ($msg:tt; $($roles:tt)*) => {
        $crate::__define_err_location!(@find [] $($roles)*)
    };
}

/// The body of one `ErrorInfo::type_name` arm.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_type_name {
    (transparent; [$($tname:tt)*]; $($roles:tt)*) => {
        $crate::__define_err_transparent!(@type_name [$($tname)*]; $($roles)*)
    };
    ($msg:tt; [$($tname:tt)*]; $($roles:tt)*) => {
        $($tname)*
    };
}

/// `transparent` variants: find the one field (besides an optional
/// `#[location]` field) and forward to it.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_transparent {
    (@go fmt [$f:ident] $n:ident [$($loc:ident)?]) => {
        ::core::fmt::Display::fmt($n, $f)
    };
    (@go source [] $n:ident [$($loc:ident)?]) => {
        $crate::__private::forward_source!($n)
    };
    (@go location [] $n:ident [$($loc:ident)?]) => {{
        let inner: ::core::option::Option<$crate::Location> = match $crate::__private::info_of!($n) {
            ::core::option::Option::Some(info) => $crate::ErrorInfo::location(info),
            ::core::option::Option::None => ::core::option::Option::None,
        };
        inner $( .or(::core::option::Option::Some(*$loc)) )?
    }};
    (@default source []) => { ::core::option::Option::None };
    (@default location []) => { ::core::option::Option::None };
    (@default type_name [$($tname:tt)*]) => { $($tname)* };
    (@go type_name [$($tname:tt)*] $n:ident [$($loc:ident)?]) => {
        match $crate::__private::info_of!($n) {
            ::core::option::Option::Some(info) => $crate::ErrorInfo::type_name(info),
            ::core::option::Option::None => $($tname)*,
        }
    };

    // Shape errors are reported once, by the `Display` arm; the other
    // methods expand to their defaults so the user sees one message.
    (@fmt [$($ctx:tt)*]; location $a:ident location $b:ident $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a variant has more than one `#[location]` field" }
    };
    (@$what:ident [$($ctx:tt)*]; location $a:ident location $b:ident $($rest:tt)*) => {
        $crate::__define_err_transparent!(@default $what [$($ctx)*])
    };
    (@$what:ident [$($ctx:tt)*]; location $loc:ident $role:ident $n:ident) => {
        $crate::__define_err_transparent!(@go $what [$($ctx)*] $n [$loc])
    };
    (@$what:ident [$($ctx:tt)*]; $role:ident $n:ident location $loc:ident) => {
        $crate::__define_err_transparent!(@go $what [$($ctx)*] $n [$loc])
    };
    (@fmt [$($ctx:tt)*]; location $loc:ident) => {
        ::core::compile_error! { "define_errors!: `transparent` needs a field to forward to besides the `#[location]` field" }
    };
    (@$what:ident [$($ctx:tt)*]; location $loc:ident) => {
        $crate::__define_err_transparent!(@default $what [$($ctx)*])
    };
    (@$what:ident [$($ctx:tt)*]; $role:ident $n:ident) => {
        $crate::__define_err_transparent!(@go $what [$($ctx)*] $n [])
    };
    (@fmt [$($ctx:tt)*]; $($roles:tt)*) => {
        ::core::compile_error! { "define_errors!: `transparent` needs exactly one field, plus an optional `#[location]` field" }
    };
    (@$what:ident [$($ctx:tt)*]; $($roles:tt)*) => {
        $crate::__define_err_transparent!(@default $what [$($ctx)*])
    };
}

/// The `From` impl for a variant with a `#[from]` field, if it has one.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_from {
    // Start: `kind` is `braced` or `tuple`; a unit variant has neither.
    ($name:ident [$($path:tt)*]) => {};
    ($name:ident [$($path:tt)*] $kind:ident $($fields:tt)*) => {
        $crate::__define_err_from! { @scan $name [$($path)*] __define_err_value $kind [] [] [] $($fields)* }
    };

    // @scan name [path] value kind [from field] [constructor arguments] [other fields]
    // `value` is the parameter name, created once above so every step's tokens
    // share its hygiene.
    (@scan $name:ident [$($path:tt)*] $v:ident braced [] [$($init:tt)*] [$($other:tt)*] from $n:ident : $t:ty ; $($rest:tt)*) => {
        $crate::__define_err_from! { @scan $name [$($path)*] $v braced [$t] [$($init)* $n: $v,] [$($other)*] $($rest)* }
    };
    (@scan $name:ident [$($path:tt)*] $v:ident tuple [] [$($init:tt)*] [$($other:tt)*] from $n:ident : $t:ty ; $($rest:tt)*) => {
        $crate::__define_err_from! { @scan $name [$($path)*] $v tuple [$t] [$($init)* $v,] [$($other)*] $($rest)* }
    };
    (@scan $name:ident [$($path:tt)*] $v:ident $kind:ident [$($from:tt)+] [$($init:tt)*] [$($other:tt)*] from $n:ident : $t:ty ; $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a variant has more than one `#[from]` field" }
    };
    (@scan $name:ident [$($path:tt)*] $v:ident braced [$($from:tt)*] [$($init:tt)*] [$($other:tt)*] location $n:ident : $t:ty ; $($rest:tt)*) => {
        $crate::__define_err_from! { @scan $name [$($path)*] $v braced [$($from)*] [$($init)* $n: ::core::panic::Location::caller(),] [$($other)*] $($rest)* }
    };
    (@scan $name:ident [$($path:tt)*] $v:ident tuple [$($from:tt)*] [$($init:tt)*] [$($other:tt)*] location $n:ident : $t:ty ; $($rest:tt)*) => {
        $crate::__define_err_from! { @scan $name [$($path)*] $v tuple [$($from)*] [$($init)* ::core::panic::Location::caller(),] [$($other)*] $($rest)* }
    };
    (@scan $name:ident [$($path:tt)*] $v:ident $kind:ident [$($from:tt)*] [$($init:tt)*] [$($other:tt)*] $role:ident $n:ident : $t:ty ; $($rest:tt)*) => {
        $crate::__define_err_from! { @scan $name [$($path)*] $v $kind [$($from)*] [$($init)*] [$($other)* $n] $($rest)* }
    };

    // Done.
    (@scan $name:ident [$($path:tt)*] $v:ident $kind:ident [] [$($init:tt)*] [$($other:tt)*]) => {};
    (@scan $name:ident [$($path:tt)*] $v:ident braced [$t:ty] [$($init:tt)*] []) => {
        #[allow(deprecated)]
        impl ::core::convert::From<$t> for $name {
            #[track_caller]
            #[inline]
            fn from($v: $t) -> Self {
                $($path)* { $($init)* }
            }
        }
    };
    (@scan $name:ident [$($path:tt)*] $v:ident tuple [$t:ty] [$($init:tt)*] []) => {
        #[allow(deprecated)]
        impl ::core::convert::From<$t> for $name {
            #[track_caller]
            #[inline]
            fn from($v: $t) -> Self {
                $($path)* ( $($init)* )
            }
        }
    };
    (@scan $name:ident [$($path:tt)*] $v:ident $kind:ident [$t:ty] [$($init:tt)*] [$($other:tt)+]) => {
        ::core::compile_error! {
            "define_errors!: a `#[from]` variant holds the source and, optionally, one `#[location]` field; move other fields to a variant without `#[from]`"
        }
    };
}
