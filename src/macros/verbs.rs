//! The verb macros. They expand at the call site, so an event carries the
//! caller's location, and nothing in their arguments is evaluated when the
//! event could not be delivered.

/// [`notice()`](crate::notice()) with a format string.
///
/// The format string must be a literal. Arguments are evaluated only when a
/// reporter is installed.
///
/// # Examples
///
/// ```
/// let elapsed = 12;
/// define_err::notice!("cache rebuilt from the journal in {elapsed} ms");
/// ```
#[macro_export]
macro_rules! notice {
    ($($arg:tt)+) => {
        if $crate::is_installed() {
            $crate::__private::notice_args($crate::Kinds::NONE, ::core::format_args!($($arg)+), $crate::here());
        }
    };
}

/// [`warn`](crate::warn()) with a format string: once per call site, exact for
/// any number of sites.
///
/// The format string must be a literal. After a site has delivered, its
/// arguments are no longer evaluated. The name matches `log::warn!` and
/// `tracing::warn!`; call it by path where those are used.
///
/// # Examples
///
/// ```
/// fn parse_port(text: &str) -> u16 {
///     text.parse().unwrap_or_else(|_| {
///         define_err::warn!("port {text:?} is not a number; using 8080");
///         8080
///     })
/// }
///
/// assert_eq!(parse_port("x"), 8080);
/// ```
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {{
        static __DEFINE_ERR_SITE: $crate::__private::Site = $crate::__private::Site::new();
        if $crate::__private::warn_ready(&__DEFINE_ERR_SITE) {
            $crate::__private::warn_site(
                &__DEFINE_ERR_SITE,
                $crate::Kinds::NONE,
                ::core::format_args!($($arg)+),
                $crate::here(),
            );
        }
    }};
}

/// [`trigger`](crate::trigger()) with a format string.
///
/// # Examples
///
/// ```
/// use define_err::Kinds;
///
/// let failed_logins = 21;
/// define_err::trigger!(Kinds::SECURITY, "{failed_logins} failed logins in one minute");
/// ```
#[macro_export]
macro_rules! trigger {
    ($kinds:expr, $($arg:tt)+) => {
        if $crate::is_installed() {
            $crate::__private::trigger_args($kinds, ::core::format_args!($($arg)+), $crate::here());
        }
    };
}

/// [`deprecated`](crate::deprecated()) with a version and a replacement: once
/// per call site. The message is "`item` is deprecated since 2.1.0; use
/// pool.size instead".
///
/// # Examples
///
/// ```
/// define_err::deprecated!("pool.max", since = "2.1.0", replacement = "pool.size");
/// ```
#[macro_export]
macro_rules! deprecated {
    ($item:literal $(, since = $since:literal)? $(, replacement = $replacement:literal)? $(,)?) => {{
        static __DEFINE_ERR_SITE: $crate::__private::Site = $crate::__private::Site::new();
        if $crate::__private::warn_ready(&__DEFINE_ERR_SITE) {
            $crate::__private::deprecated_site(
                &__DEFINE_ERR_SITE,
                ::core::format_args!(
                    "{}",
                    ::core::concat!(
                        "`", $item, "` is deprecated"
                        $(, " since ", $since)?
                        $(, "; use ", $replacement, " instead")?
                    )
                ),
                $crate::here(),
            );
        }
    }};
}
