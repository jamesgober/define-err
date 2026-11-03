//! Attack tests for the threats that need no reporter (`dev/PLAN.md` 10.1).
//! `docs/SECURITY.md` maps each threat to the tests that hold it.

use std::{error::Error, fmt};

use define_err::{
    catalog, define_errors, identify, Category, Code, ErrorInfo, Kinds, Secret, Severity,
};
use proptest::prelude::*;

const MARKER: &str = "s3cr3t-shard-17";

define_errors! {
    namespace = "AT";

    pub enum Leaky {
        #[code(1, help = "Check the store.")]
        #[public("The store is unavailable.")]
        Stated { path: String } => "open {path} failed",

        #[code(2, help = "Check the store.")]
        #[category(Internal)]
        Defaulted { path: String } => "read {path} failed",

        #[code(3, help = "Check the store.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt => "page failed its checksum",

        Uncoded { path: String } => "write {path} failed",

        Through(#[from] inner: Inner) => transparent,
    }

    pub enum Inner {
        Plain { path: String } => "sync {path} failed",
    }

    pub enum Outer {
        #[public("Try again later.")]
        Wrapped(#[source] inner: Leaky) => "wrapped: {inner}",
    }
}

/// T1: no variant's public message carries what its `Display` carries.
#[test]
fn test_public_message_never_contains_display_text() {
    let path = String::from(MARKER);
    let variants = [
        Leaky::Stated { path: path.clone() },
        Leaky::Defaulted { path: path.clone() },
        Leaky::Uncoded { path: path.clone() },
        Leaky::Through(Inner::Plain { path }),
    ];
    for error in &variants {
        assert!(error.to_string().contains(MARKER));
        assert!(!error.public_message().contains(MARKER), "{error:?}");
    }
    assert_eq!(variants[0].public_message(), "The store is unavailable.");
}

/// T1: the same through the probe, where a record and a sink read it.
#[test]
fn test_probe_never_carries_display_text() {
    let error = Leaky::Defaulted {
        path: String::from(MARKER),
    };
    let dynamic: &(dyn Error + 'static) = &error;
    let identity = identify(dynamic).unwrap();
    assert!(!identity.type_name.contains(MARKER));
    assert!(!identity.public.unwrap_or_default().contains(MARKER));
}

proptest! {
    // Miri runs this about ten thousand times slower.
    #![proptest_config(ProptestConfig::with_cases(if cfg!(miri) { 4 } else { 256 }))]

    /// T2: no format spec shows the value. The value has no lowercase
    /// letters, so it cannot match part of `[redacted]` by chance.
    #[test]
    fn prop_secret_hides_its_value_under_every_format_spec(
        value in "[A-Z0-9]{4,32}",
        width in 0_usize..64,
        precision in 0_usize..16,
    ) {
        let secret = Secret::new(value.clone());
        for text in [
            format!("{secret}"),
            format!("{secret:?}"),
            format!("{secret:#?}"),
            format!("{secret:>40}"),
            format!("{secret:<width$}"),
            format!("{secret:^width$}"),
            format!("{secret:*^9}"),
            format!("{secret:.3}"),
            format!("{secret:.precision$}"),
            format!("{secret:>width$.precision$}"),
            format!("{secret:#>width$?}"),
            format!("{:?}", Some(&secret)),
            format!("{:#?}", (&secret, &secret)),
        ] {
            prop_assert!(!text.contains(&value), "{}", text);
        }
    }

    /// T4: a kinds value from the wire has no way to put a control byte in text.
    #[test]
    fn prop_kinds_text_has_no_control_bytes(bits in any::<u64>()) {
        let kinds = Kinds::from_bits(bits);
        let debug = format!("{kinds:?}");
        for text in [kinds.to_string(), debug] {
            prop_assert!(text.bytes().all(|byte| byte >= 0x20 && byte != 0x7f));
        }
    }

    /// T4 and T12: the same for a code, from any input a peer can send.
    #[test]
    fn prop_code_text_has_no_control_bytes(raw in any::<u64>()) {
        if let Some(code) = Code::from_u64(raw) {
            prop_assert!(code.to_string().bytes().all(|byte| byte.is_ascii_graphic()));
        }
    }
}

/// T4: every severity and category, whose text a sink trusts.
#[test]
fn test_severity_and_category_text_has_no_control_bytes() {
    for number in 1..=24 {
        let severity = Severity::new(number).unwrap();
        assert!(severity.to_string().bytes().all(|b| b.is_ascii_graphic()));
    }
    for category in Category::ALL {
        assert!(category.name().bytes().all(|b| b.is_ascii_graphic()));
        let public = category.default_public_message();
        assert!(public.bytes().all(|b| (0x20..0x7f).contains(&b)));
    }
}

/// A foreign error that states any probe it likes.
#[derive(Debug)]
struct Forged(String);

impl fmt::Display for Forged {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("forged")
    }
}

impl Error for Forged {
    fn description(&self) -> &str {
        &self.0
    }
}

/// The probe text of a real variant.
#[allow(deprecated)]
fn genuine() -> String {
    Leaky::Corrupt.description().to_owned()
}

/// T11: a copy of a real probe is believed: codes identify, they do not
/// authenticate. `identify` alone cannot and does not claim otherwise.
#[test]
fn test_copied_probe_is_accepted_as_stated() {
    let forged = Forged(genuine());
    let identity = identify(&forged).unwrap();
    assert_eq!(identity.code, Some(Leaky::Corrupt.code().unwrap()));
    assert!(identity.kinds.contains(Kinds::SECURITY));
}

/// T11: the check `docs/SECURITY.md` gives the error service. A forger who
/// claims a code under another name is caught by the catalog.
#[test]
fn test_catalog_detects_a_claimed_code_under_another_name() {
    let catalogs = [Leaky::CATALOG];
    let text = genuine().replace("Leaky::Corrupt", "Evil::Claim");
    let forged = Forged(text);
    let identity = identify(&forged).unwrap();
    let entry = catalog::find(&catalogs, identity.code.unwrap()).unwrap();
    assert_ne!(entry.type_name, identity.type_name);

    // The real one agrees with its catalog entry.
    let dynamic: &(dyn Error + 'static) = &Leaky::Corrupt;
    let real = identify(dynamic).unwrap();
    let entry = catalog::find(&catalogs, real.code.unwrap()).unwrap();
    assert_eq!(entry.type_name, real.type_name);
}

/// T11: one bad byte anywhere in the header, and the claim is refused.
#[test]
fn test_every_header_byte_is_validated() {
    let text = genuine();
    assert!(identify(&Forged(text.clone())).is_some());
    for position in 0..43 {
        let mut bytes = text.clone().into_bytes();
        bytes[position] = b'g';
        let forged = Forged(String::from_utf8(bytes).unwrap());
        assert!(identify(&forged).is_none(), "byte {position} was not read");
    }
}

/// T11: fields that are valid hex and invalid values.
#[test]
fn test_out_of_range_fields_are_refused() {
    let text = genuine();
    let with = |range: std::ops::Range<usize>, value: &str| {
        let mut forged = text.clone();
        forged.replace_range(range, value);
        identify(&Forged(forged)).is_none()
    };
    assert!(with(4..20, "ffffffffffffffff"), "code layout");
    assert!(with(20..22, "00"), "severity 0");
    assert!(with(20..22, "19"), "severity 25");
    assert!(with(22..24, "ff"), "category");
    assert!(with(40..42, "04"), "flags");
    assert!(with(43..44, " "), "a space in the name");
}

/// T11: a probe far longer than any real one is refused without a scan.
#[test]
fn test_oversized_probe_is_refused() {
    let mut text = genuine();
    text.push_str(&"A".repeat(1 << 16));
    assert!(identify(&Forged(text)).is_none());
}

/// T15: inner text stays in the chain and out of `public`.
#[cfg(feature = "alloc")]
#[test]
fn test_record_public_never_takes_inner_display_text() {
    use define_err::ErrorRecord;

    let inner = Leaky::Defaulted {
        path: String::from(MARKER),
    };
    let outer = Outer::Wrapped(inner);
    let record = ErrorRecord::from_error(&outer);
    assert!(!record.public.contains(MARKER));
    assert!(record
        .chain
        .iter()
        .any(|entry| entry.message.contains(MARKER)));
    assert!(!record.public.is_empty());
}

/// T15: no probe at all, and `public` is still the category's sentence.
#[cfg(feature = "alloc")]
#[test]
fn test_record_of_a_foreign_chain_uses_the_category_sentence() {
    use define_err::ErrorRecord;

    #[derive(Debug)]
    struct Foreign;
    impl fmt::Display for Foreign {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(MARKER)
        }
    }
    impl Error for Foreign {}

    let record = ErrorRecord::from_error(&Foreign);
    assert_eq!(record.public, Category::Unknown.default_public_message());
}

/// T4: a forged log line is kept verbatim and marked.
#[cfg(feature = "alloc")]
#[test]
fn test_record_marks_a_forged_log_line() {
    use define_err::ErrorRecord;

    let error = Leaky::Uncoded {
        path: String::from("x\n2026-11-03 INFO login ok"),
    };
    let record = ErrorRecord::from_error(&error);
    assert!(record.chain[0].has_control);
    assert!(record.chain[0].message.contains('\n'));
    assert!(!record.to_string().contains('\n'));
}
