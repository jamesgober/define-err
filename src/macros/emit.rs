//! The emitter: canonical variants in, items out (`docs/GENERATED-CODE.md`).
//!
//! Canonical input, produced by both parser paths:
//!
//! ```text
//! @enum [block context] [type attributes] (vis) Name [ (
//!     [variant attributes] [code] [category] [severity] [kinds] [public] [action] [retryable] [retry_after]
//!     Variant
//!     { { [field attributes] [field vis] role name : Type } ... }     // braced, or
//!     ( { [field attributes] [field vis] role name : Type } ... )     // tuple, or nothing (unit)
//!     => message
//! ) ... ]
//! @struct [block context] [type attributes] [metadata slots] (vis) Name { fields } => message
//! ```
//!
//! `role` is `from`, `source`, `location`, `retry_after`, or `plain`. Every
//! field has a name: unnamed tuple fields were named by the parser. Every
//! generated arm binds all fields by reference (`ref name`) on `match *self`,
//! so the per-variant helpers below can use any of them.

/// Emits the items for one type from its canonical form, then hands the rest
/// of the block back to `__define_err_block!`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_emit {
    (@enum [[$($ns:tt)*] [$($docs:tt)*] [$($rall:tt)*] [$($rf:tt)*] [$($acc:tt)*] [$($rest:tt)*]]
        [$($tattr:tt)*] ($vis:vis) $name:ident [ $(
        ( [$($vattr:tt)*]
          [$($code:tt)?] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] [$($retry:tt)?] [$($ra:tt)?]
          $vn:ident
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

        $crate::__define_err_emit! { @impls [[$($ns)*] [$($docs)*] [$($rf)*]] ($vis) $name [ $(
            ( [Self::$vn] [::core::concat!(::core::stringify!($name), "::", ::core::stringify!($vn))] [::core::stringify!($vn)]
              [$($code)?] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?] [$($retry)?] [$($ra)?]
              $({ $( { $frole $fname : $fty } )* })?
              $(( $( { $xrole $xname : $xty } )* ))?
              => $msg )
        )* ] }

        $crate::__define_err_block! {
            @types [[$($ns)*] [$($docs)*] [$($rall)*] [] [$($acc)* $( $( { $code } )? )*]] $($rest)*
        }
    };

    (@struct [[$($ns:tt)*] [$($docs:tt)*] [$($rall:tt)*] [$($rf:tt)*] [$($acc:tt)*] [$($rest:tt)*]]
        [$($tattr:tt)*]
        [[$($code:tt)?] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] [$($retry:tt)?] [$($ra:tt)?]]
        ($vis:vis) $name:ident
        $({ $( { [$($fattr:tt)*] [$($fvis:tt)*] $frole:ident $fname:ident : $fty:ty } )* })?
        $(( $( { [$($xattr:tt)*] [$($xvis:tt)*] $xrole:ident $xname:ident : $xty:ty } )* ))?
        => $msg:tt) => {
        $crate::__define_err_emit! { @struct_def [$($tattr)*] ($vis) $name
            $({ $( { [$($fattr)*] [$($fvis)*] $fname : $fty } )* })?
            $(( $( { [$($xattr)*] [$($xvis)*] $xname : $xty } )* ))?
        }

        $crate::__define_err_emit! { @impls [[$($ns)*] [$($docs)*] [$($rf)*]] ($vis) $name [
            ( [Self] [::core::stringify!($name)] [::core::stringify!($name)]
              [$($code)?] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?] [$($retry)?] [$($ra)?]
              $({ $( { $frole $fname : $fty } )* })?
              $(( $( { $xrole $xname : $xty } )* ))?
              => $msg )
        ] }

        $crate::__define_err_block! {
            @types [[$($ns)*] [$($docs)*] [$($rall)*] [] [$($acc)* $( { $code } )?]] $($rest)*
        }
    };

    (@struct_def [$($tattr:tt)*] ($vis:vis) $name:ident { $( { [$($fattr:tt)*] [$($fvis:tt)*] $fname:ident : $fty:ty } )* }) => {
        $($tattr)*
        $vis struct $name { $( $($fattr)* $($fvis)* $fname: $fty, )* }
    };
    (@struct_def [$($tattr:tt)*] ($vis:vis) $name:ident ( $( { [$($fattr:tt)*] [$($fvis:tt)*] $fname:ident : $fty:ty } )* )) => {
        $($tattr)*
        $vis struct $name ( $( $($fattr)* $($fvis)* $fty, )* );
    };
    (@struct_def [$($tattr:tt)*] ($vis:vis) $name:ident) => {
        $($tattr)*
        $vis struct $name;
    };

    (@impls [$ns:tt $docs:tt $rf:tt] ($vis:vis) $name:ident [ $(
        ( [$($path:tt)*] [$($tname:tt)*] [$($title:tt)*]
          [$($code:tt)?] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] [$($retry:tt)?] [$($ra:tt)?]
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

            fn description(&self) -> &str {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_probe!(
                            $ns [$($code)?] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($tname)*];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?
                        ),
                )* }
            }
        }

        #[allow(deprecated)]
        impl $crate::ErrorInfo for $name {
            fn code(&self) -> ::core::option::Option<$crate::Code> {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(code [$crate::__define_err_code!(@option $ns [$($code)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn code_info(&self) -> ::core::option::Option<&'static $crate::CodeInfo> {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(code_info [$crate::__define_err_code!(@info
                                $ns $docs [$($tname)*] [$($title)*] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?]
                                [$($code)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn category(&self) -> $crate::Category {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(category [$crate::__define_err_value!(@category [$($cat)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn severity(&self) -> $crate::Severity {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(severity [$crate::__define_err_value!(@severity [$($sev)?] [$($cat)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn kinds(&self) -> $crate::Kinds {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(kinds [$crate::__define_err_value!(@kinds [$($kinds)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn public_message(&self) -> &'static str {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(public_message [$crate::__define_err_value!(@public [$($public)?] [$($cat)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn help(&self) -> ::core::option::Option<&'static str> {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(help [$crate::__define_err_code!(@help [$($code)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn action(&self) -> $crate::Action {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(action [$crate::__define_err_value!(@action [$($action)?] [$($cat)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn is_retryable(&self) -> bool {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(is_retryable [$crate::__define_err_value!(@retryable [$($retry)?] [$($cat)?])];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

            fn retry_after(&self) -> ::core::option::Option<::core::time::Duration> {
                #[allow(unused_variables)]
                match *self { $(
                    $($path)* $({ $( ref $fname, )* })? $(( $( ref $xname, )* ))? =>
                        $crate::__define_err_forward!(retry_after
                            [$crate::__define_err_value!(@retry_after [$($ra)?] []; $($( $frole $fname )*)? $($( $xrole $xname )*)?)];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
                )* }
            }

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
                        $crate::__define_err_forward!(type_name [$($tname)*];
                            $msg; $($( $frole $fname )*)? $($( $xrole $xname )*)?),
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

        #[allow(deprecated)]
        impl $name {
            $( $crate::__define_err_code!(@const ($vis) $ns [$($code)?]); )*

            $crate::__define_err_catalog! { $ns $docs $rf ($vis) [ $(
                ( [$($tname)*] [$($title)*] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?] [$($code)?] )
            )* ] }
        }
    };

    ($($input:tt)*) => {
        ::core::compile_error! {
            "define_errors!: a field must be `name: Type` with at most one of `#[from]`, `#[source]`, `#[location]`, `#[retry_after]` before it"
        }
    };
}

/// A type's `CATALOG`, when its block has a namespace.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_catalog {
    ([] [$($docs:tt)*] [$($rf:tt)*] ($vis:vis) [$($entries:tt)*]) => {};
    ($ns:tt $docs:tt [$($rf:literal)*] ($vis:vis) [ $(
        ( [$($tname:tt)*] [$($title:tt)*] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] [$($code:tt)?] )
    )* ]) => {
        /// The catalog entries of this type: one per code, plus the block's
        /// retired numbers on the block's first type. Generated by
        /// `define_errors!`.
        $vis const CATALOG: &'static [$crate::CodeInfo] = {
            const PARTS: &[&[$crate::CodeInfo]] = &[
                $( $crate::__define_err_code!(@part
                    $ns $docs [$($tname)*] [$($title)*] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?]
                    [$($code)?]), )*
                $( &[$crate::__private::retired_code($crate::__define_err_ns!($ns), $rf)], )*
            ];
            const LEN: usize = $crate::__private::catalog_len(PARTS);
            const ALL: [$crate::CodeInfo; LEN] = $crate::__private::catalog::<LEN>(PARTS);
            &ALL
        };
    };
}

/// The block's namespace as a `const` expression.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_ns {
    ([$ns:literal]) => {
        $crate::__private::namespace($ns, 4)
    };
    ([$ns:literal $width:literal]) => {
        $crate::__private::namespace($ns, $width)
    };
    ([]) => {
        ::core::compile_error! { "define_errors!: `#[code]` needs a `namespace = \"...\";` line at the top of the define_errors! block" }
    };
}

/// Everything read from a `#[code(...)]` argument.
///
/// Grammar: `(n [, name = IDENT], help = "..." [, title = "..."]
/// [, deprecated = "since" [, replacement = n]])`, in this order.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_code {
    // Operations on a parsed code: [n] [name] [help] [title] [since] [replacement].
    (@do number [] [$n:literal] [$($cname:ident)?] [$help:literal] [$($title:literal)?] [$($since:literal)?] [$($rep:literal)?]) => {
        $n
    };
    (@do replacement [] [$n:literal] [$($cname:ident)?] [$help:literal] [$($title:literal)?] [$($since:literal)?] [$($rep:literal)?]) => {
        &[ $( ($n, $rep) )? ]
    };
    (@do value [$($ns:tt)*] [$n:literal] [$($cname:ident)?] [$help:literal] [$($title:literal)?] [$($since:literal)?] [$($rep:literal)?]) => {{
        const __DEFINE_ERR_CODE: $crate::Code = $crate::__private::code($crate::__define_err_ns!([$($ns)*]), $n);
        __DEFINE_ERR_CODE
    }};
    (@do help [] [$n:literal] [$($cname:ident)?] [$help:literal] [$($title:literal)?] [$($since:literal)?] [$($rep:literal)?]) => {
        $help
    };
    (@do entry [[$($ns:tt)*] [$($docs:literal)?] [$($tname:tt)*] [$($tdefault:tt)*] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?]]
        [$n:literal] [$($cname:ident)?] [$help:literal] [$($title:literal)?] [$($since:literal)?] [$($rep:literal)?]) => {
        $crate::CodeInfo::new(
            $crate::__private::code($crate::__define_err_ns!([$($ns)*]), $n),
            $crate::__define_err_code!(@title [$($title)?] [$($tdefault)*]),
            $help,
        )
        .with_category($crate::__define_err_value!(@category [$($cat)?]))
        .with_severity($crate::__define_err_value!(@severity [$($sev)?] [$($cat)?]))
        .with_kinds($crate::__define_err_value!(@kinds [$($kinds)?]))
        .with_action($crate::__define_err_value!(@action [$($action)?] [$($cat)?]))
        .with_public($crate::__define_err_value!(@public [$($public)?] [$($cat)?]))
        .with_type_name($($tname)*)
        $( .with_docs($docs) )?
        .with_state($crate::__define_err_code!(@state [$($ns)*] [$($since)?] [$($rep)?]))
    };
    (@do const [($vis:vis) [$ns:literal $($w:literal)?]]
        [$n:literal] [] [$help:literal] [$($title:literal)?] [$($since:literal)?] [$($rep:literal)?]) => {};
    (@do const [($vis:vis) [$ns:literal $($w:literal)?]]
        [$n:literal] [$cname:ident] [$help:literal] [$($title:literal)?] [$($since:literal)?] [$($rep:literal)?]) => {
        #[doc = ::core::concat!("The code number ", ::core::stringify!($n), " in namespace ", $ns, ". Generated by `define_errors!`.")]
        $( #[deprecated(since = $since, note = "this code is deprecated; its catalog entry names the replacement")] )?
        $vis const $cname: $crate::Code = $crate::__define_err_code!(@do value [$ns $($w)?] [$n] [] [$help] [] [] []);
    };

    (@title [$title:literal] [$($default:tt)*]) => { $title };
    (@title [] [$($default:tt)*]) => { $($default)* };
    (@state [$($ns:tt)*] [] [$($rep:literal)?]) => { $crate::CodeState::Active };
    (@state [$($ns:tt)*] [$since:literal] []) => {
        $crate::CodeState::Deprecated { since: $since, replacement: ::core::option::Option::None }
    };
    (@state [$($ns:tt)*] [$since:literal] [$rep:literal]) => {
        $crate::CodeState::Deprecated {
            since: $since,
            replacement: ::core::option::Option::Some($crate::__private::code($crate::__define_err_ns!([$($ns)*]), $rep)),
        }
    };

    // Slot forms: a variant's code slot is `[]` or `[(...)]`.
    (@option [$($ns:tt)*] []) => { ::core::option::Option::None };
    (@option [$($ns:tt)*] [$code:tt]) => {
        ::core::option::Option::Some($crate::__define_err_code!(@parse value [$($ns)*] $code))
    };
    (@raw [$($ns:tt)*] []) => { ::core::option::Option::None };
    (@raw [$($ns:tt)*] [$code:tt]) => {
        ::core::option::Option::Some($crate::Code::as_u64($crate::__define_err_code!(@parse value [$($ns)*] $code)))
    };
    (@help []) => { ::core::option::Option::None };
    (@help [$code:tt]) => { ::core::option::Option::Some($crate::__define_err_code!(@parse help [] $code)) };
    (@info [$($ns:tt)*] [$($docs:tt)*] [$($tname:tt)*] [$($title:tt)*] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] []) => {
        ::core::option::Option::None
    };
    (@info [$($ns:tt)*] [$($docs:tt)*] [$($tname:tt)*] [$($title:tt)*] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] [$code:tt]) => {{
        const __DEFINE_ERR_ENTRY: $crate::CodeInfo = $crate::__define_err_code!(@parse entry
            [[$($ns)*] [$($docs)*] [$($tname)*] [$($title)*] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?]] $code);
        ::core::option::Option::Some(&__DEFINE_ERR_ENTRY)
    }};
    (@part [$($ns:tt)*] [$($docs:tt)*] [$($tname:tt)*] [$($title:tt)*] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] []) => {
        &[]
    };
    (@part [$($ns:tt)*] [$($docs:tt)*] [$($tname:tt)*] [$($title:tt)*] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($action:tt)?] [$code:tt]) => {
        &[$crate::__define_err_code!(@parse entry
            [[$($ns)*] [$($docs)*] [$($tname)*] [$($title)*] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($action)?]] $code)]
    };
    (@const ($vis:vis) [$($ns:tt)*] []) => {};
    (@const ($vis:vis) [$($ns:tt)+] [$code:tt]) => {
        $crate::__define_err_code!(@parse const [($vis) [$($ns)+]] $code);
    };
    (@const ($vis:vis) [] [$code:tt]) => {
        ::core::compile_error! { "define_errors!: `#[code]` needs a `namespace = \"...\";` line at the top of the define_errors! block" }
    };

    // Number and replacement, from the block's checks.
    (@number $code:tt) => { $crate::__define_err_code!(@parse number [] $code) };
    (@replacement $code:tt) => { $crate::__define_err_code!(@parse replacement [] $code) };

    // Parsing the argument.
    (@parse $op:ident [$($args:tt)*]
        ($n:literal $(, name = $cname:ident)? , help = $help:literal $(, title = $title:literal)?
            $(, deprecated = $since:literal $(, replacement = $rep:literal)?)? $(,)?)) => {
        $crate::__define_err_code! { @do $op [$($args)*] [$n] [$($cname)?] [$help] [$($title)?] [$($since)?] [$($($rep)?)?] }
    };
    (@parse $op:ident [$($args:tt)*] ($n:literal $(, name = $cname:ident)? $(,)?)) => {
        ::core::compile_error! { "define_errors!: `#[code]` needs `help = \"...\"`: every code ships with its explanation" }
    };
    (@parse $op:ident [$($args:tt)*] ($n:literal $($rest:tt)*)) => {
        ::core::compile_error! {
            "define_errors!: write `#[code(n, name = NAME, help = \"...\", title = \"...\", deprecated = \"since\", replacement = n)]` in this order; only the number and `help` are required"
        }
    };
    (@parse $op:ident [$($args:tt)*] $($rest:tt)*) => {
        ::core::compile_error! { "define_errors!: a code number must be an integer literal from 0 to 16777215, for example `#[code(42, help = \"...\")]`" }
    };
}

/// One metadata value of a variant, from its slots; the category's default
/// when the variant does not say.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_value {
    (@category []) => { $crate::Category::Unknown };
    (@category [($name:ident)]) => { $crate::Category::$name };
    (@category [$($x:tt)*]) => {
        ::core::compile_error! { "define_errors!: `#[category]` takes one category name, for example `#[category(Unavailable)]`" }
    };

    (@severity [] [$($cat:tt)?]) => { $crate::Category::default_severity($crate::__define_err_value!(@category [$($cat)?])) };
    (@severity [($n:literal)] [$($cat:tt)?]) => { $crate::__private::severity::<$n>() };
    (@severity [($name:ident)] [$($cat:tt)?]) => {{
        #[allow(unused_imports)]
        use $crate::__private::named_levels::*;
        $name
    }};
    (@severity [$($x:tt)*] [$($cat:tt)?]) => {
        ::core::compile_error! { "define_errors!: `#[severity]` takes a name (`CRITICAL`) or a number from 1 to 24" }
    };

    (@kinds []) => { $crate::Kinds::ERROR };
    (@kinds [($($kind:path)|+)]) => {{
        #[allow(unused_imports)]
        use $crate::__private::platform_kinds::*;
        $crate::Kinds::NONE $( .union($kind) )+
    }};
    (@kinds [$($x:tt)*]) => {
        ::core::compile_error! { "define_errors!: `#[kinds]` takes kinds joined by `|`, for example `#[kinds(ERROR | SECURITY)]`" }
    };

    (@public [] [$($cat:tt)?]) => { $crate::Category::default_public_message($crate::__define_err_value!(@category [$($cat)?])) };
    (@public [($text:literal)] [$($cat:tt)?]) => { $text };
    (@public [$($x:tt)*] [$($cat:tt)?]) => {
        ::core::compile_error! { "define_errors!: `#[public]` takes one string, for example `#[public(\"The service is busy.\")]`" }
    };
    (@public_flag []) => { 0_u8 };
    (@public_flag [$x:tt]) => { 1_u8 };

    (@action [] [$($cat:tt)?]) => { $crate::Category::default_action($crate::__define_err_value!(@category [$($cat)?])) };
    (@action [($name:ident)] [$($cat:tt)?]) => { $crate::Action::$name };
    (@action [$($x:tt)*] [$($cat:tt)?]) => {
        ::core::compile_error! { "define_errors!: `#[action]` takes one action name, for example `#[action(FixConfig)]`" }
    };

    (@retryable [] [$($cat:tt)?]) => { $crate::Category::is_transient($crate::__define_err_value!(@category [$($cat)?])) };
    (@retryable [()] [$($cat:tt)?]) => { true };
    (@retryable [(($value:literal))] [$($cat:tt)?]) => { $value };
    (@retryable [$($x:tt)*] [$($cat:tt)?]) => {
        ::core::compile_error! { "define_errors!: write `#[retryable]`, `#[retryable(true)]`, or `#[retryable(false)]`" }
    };

    // `retry_after`: a `#[retry_after]` field, or the static form, not both.
    (@retry_after [$($ra:tt)?] [$($found:ident)*]; retry_after $n:ident $($rest:tt)*) => {
        $crate::__define_err_value!(@retry_after [$($ra)?] [$($found)* $n]; $($rest)*)
    };
    (@retry_after [$($ra:tt)?] [$($found:ident)*]; $role:ident $n:ident $($rest:tt)*) => {
        $crate::__define_err_value!(@retry_after [$($ra)?] [$($found)*]; $($rest)*)
    };
    (@retry_after [] [];) => { ::core::option::Option::None };
    (@retry_after [] [$n:ident];) => { $crate::__private::retry_after_of!($n) };
    (@retry_after [(secs = $value:literal)] [];) => {
        ::core::option::Option::Some(::core::time::Duration::from_secs($value))
    };
    (@retry_after [(ms = $value:literal)] [];) => {
        ::core::option::Option::Some(::core::time::Duration::from_millis($value))
    };
    (@retry_after [$($ra:tt)?] [$a:ident $b:ident $($more:ident)*];) => {
        ::core::compile_error! { "define_errors!: a variant has more than one `#[retry_after]` field" }
    };
    (@retry_after [$ra:tt] [$n:ident];) => {
        ::core::compile_error! { "define_errors!: a variant has both `#[retry_after(...)]` and a `#[retry_after]` field; keep one" }
    };
    (@retry_after [$ra:tt] [];) => {
        ::core::compile_error! { "define_errors!: write `#[retry_after(secs = 2)]` or `#[retry_after(ms = 500)]`" }
    };
}

/// One `ErrorInfo` method's arm: the variant's own value, or for a
/// `transparent` variant the inner value's, falling back to the own value
/// when the inner value has no `ErrorInfo`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_forward {
    ($method:ident [$($own:tt)*]; transparent; $($roles:tt)*) => {
        $crate::__define_err_transparent!(@forward [$method [$($own)*]]; $($roles)*)
    };
    ($method:ident [$($own:tt)*]; $msg:tt; $($roles:tt)*) => {
        $($own)*
    };
}

/// The identity probe of one variant (`description()`).
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_probe {
    ([$($ns:tt)*] [$($code:tt)?] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($tname:tt)*]; transparent; $($roles:tt)*) => {
        $crate::__define_err_transparent!(@probe [
            $crate::__define_err_probe!(@build [$($ns)*] [$($code)?] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($tname)*] 2_u8)
        ]; $($roles)*)
    };
    ([$($ns:tt)*] [$($code:tt)?] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($tname:tt)*]; $msg:tt; $($roles:tt)*) => {
        $crate::__define_err_probe!(@build [$($ns)*] [$($code)?] [$($cat)?] [$($sev)?] [$($kinds)?] [$($public)?] [$($tname)*] 0_u8)
    };
    (@build [$($ns:tt)*] [$($code:tt)?] [$($cat:tt)?] [$($sev:tt)?] [$($kinds:tt)?] [$($public:tt)?] [$($tname:tt)*] $flag:expr) => {{
        const __DEFINE_ERR_PROBE: &$crate::__private::ProbeBuf = &$crate::__private::ProbeBuf::new(
            $crate::__define_err_code!(@raw [$($ns)*] [$($code)?]),
            $crate::Severity::get($crate::__define_err_value!(@severity [$($sev)?] [$($cat)?])),
            $crate::Category::grpc_code($crate::__define_err_value!(@category [$($cat)?])),
            $crate::Kinds::bits($crate::__define_err_value!(@kinds [$($kinds)?])),
            $crate::__define_err_value!(@public_flag [$($public)?]) | $flag,
            $($tname)*,
        );
        const __DEFINE_ERR_TEXT: &str = __DEFINE_ERR_PROBE.as_str();
        __DEFINE_ERR_TEXT
    }};
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
    (@go forward [$method:ident [$($own:tt)*]] $n:ident [$($loc:ident)?]) => {
        match $crate::__private::info_of!($n) {
            ::core::option::Option::Some(info) => $crate::ErrorInfo::$method(info),
            ::core::option::Option::None => $($own)*,
        }
    };
    (@go probe [$($own:tt)*] $n:ident [$($loc:ident)?]) => {
        $crate::__private::forward_probe($crate::__private::description_of!($n), $($own)*)
    };
    (@default source []) => { ::core::option::Option::None };
    (@default location []) => { ::core::option::Option::None };
    (@default forward [$method:ident [$($own:tt)*]]) => { $($own)* };
    (@default probe [$($own:tt)*]) => { $($own)* };

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
