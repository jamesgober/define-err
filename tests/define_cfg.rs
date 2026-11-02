//! `#[cfg]` on variants and fields (DEF-24, E-27). The conditions test this
//! file's own `cfg(all())` (always on) and `cfg(any())` (always off), the same
//! way a user's feature would behave on and off.

use define_err::prelude::*;

define_errors! {
    namespace = "NET";

    pub enum NetError {
        Refused => "connection refused",

        /// Fast path: `#[cfg]` right after the doc comments.
        #[cfg(all())]
        #[code(1, name = TLS, help = "Check the certificate.")]
        Tls { alert: u8 } => "tls alert {alert}",

        #[cfg(any())]
        #[code(2, name = OFF, help = "Never compiled.")]
        Off { value: NotDefinedAnywhere } => "off {value}",

        // The same number again: the duplicate check follows the cfg.
        #[cfg(any())]
        #[code(1, help = "Never compiled.")]
        Twin => "twin",

        // General path: `#[cfg]` after a metadata attribute, and stacked.
        #[code(3, help = "Retry.")]
        #[cfg(all())]
        #[cfg(not(any()))]
        Stacked => "stacked",

        #[category(Internal)]
        #[cfg(any())]
        Gone(#[from] inner: NotDefinedAnywhere) => "gone",

        Wrapped { keep: u8, #[cfg(any())] off: NotDefinedAnywhere, #[cfg(all())] on: u8 } => "wrapped {keep} {on}",
    }

    pub struct Plain {
        pub keep: u8,
        #[cfg(any())]
        pub off: NotDefinedAnywhere,
    } => "plain {keep}";

    /// Every variant compiled out: the type still has every impl.
    pub enum Empty {
        #[cfg(any())]
        Off => "off",
    }
}

#[test]
fn test_variants_follow_their_cfg() {
    assert_eq!(NetError::Refused.to_string(), "connection refused");
    assert_eq!(NetError::Tls { alert: 40 }.to_string(), "tls alert 40");
    assert_eq!(NetError::Stacked.to_string(), "stacked");
    assert_eq!(NetError::Tls { alert: 40 }.code(), Some(NetError::TLS));
}

#[test]
fn test_catalog_and_checks_follow_the_cfg() {
    let numbers: Vec<u32> = NetError::CATALOG
        .iter()
        .map(|info| info.code.number())
        .collect();
    assert_eq!(numbers, [1, 3]);
}

#[test]
fn test_fields_follow_their_cfg() {
    let err = NetError::Wrapped { keep: 1, on: 2 };
    assert_eq!(err.to_string(), "wrapped 1 2");
    assert_eq!(Plain { keep: 3 }.to_string(), "plain 3");
}

#[test]
fn test_enum_with_every_variant_off_is_a_type() {
    fn assert_info<T: ErrorInfo + std::error::Error>() {}
    assert_info::<Empty>();
    assert_eq!(core::mem::size_of::<Empty>(), 0);
}
