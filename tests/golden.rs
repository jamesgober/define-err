//! The frozen tables, checked against the golden files written at the design
//! lock: code packing, probe strings, categories, severity names, and kind
//! bits (dev/PLAN.md NFR-10; frozen at 1.0).

use std::{error::Error as _, str::FromStr};

use define_err::{define_errors, identify, Category, Code, Kinds, Severity};

/// Data lines of a golden file: comment lines start with `#`.
fn rows(text: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| line.split('\t').collect())
}

#[test]
fn test_golden_codes_pack_parse_and_display() {
    for row in rows(include_str!("golden/codes.txt")) {
        let [text, namespace, number, width, field, raw, identity] = row[..] else {
            panic!("bad row {row:?}");
        };
        let raw = u64::from_str_radix(raw, 16).unwrap();
        let identity = u64::from_str_radix(identity, 16).unwrap();
        let code = Code::from_u64(raw).unwrap_or_else(|| panic!("{text} does not decode"));
        assert_eq!(code.to_string(), text);
        assert_eq!(code.as_u64(), raw, "{text}");
        assert_eq!(code.identity(), identity, "{text}");
        assert_eq!(code.number(), number.parse::<u32>().unwrap());
        assert_eq!(&*code.namespace().as_str(), namespace);
        assert_eq!(code.namespace().width(), width.parse::<u8>().unwrap());
        assert_eq!(raw >> 32, u64::from_str_radix(field, 16).unwrap());
        assert_eq!(Code::parse(text), Ok(code));
        // Parsing records the digits written as the width, so compare identities.
        assert_eq!(Code::from_str(text).unwrap().identity(), identity);
    }
}

#[test]
fn test_golden_categories_tables() {
    let mut seen = 0;
    for row in rows(include_str!("golden/categories.txt")) {
        let [_, name, grpc, http, exit, transient, advice, severity_name, severity, action, public] =
            row[..]
        else {
            panic!("bad row {row:?}");
        };
        let category = Category::parse(name).unwrap();
        assert_eq!(category.name(), name);
        assert_eq!(category.grpc_code().to_string(), grpc);
        assert_eq!(category.http_status().to_string(), http);
        assert_eq!(category.exit_code().to_string(), exit);
        assert_eq!(category.is_transient(), transient == "yes", "{name}");
        assert_eq!(format!("{:?}", category.retry_advice()), advice, "{name}");
        assert_eq!(
            category.default_severity().get().to_string(),
            severity,
            "{name}"
        );
        assert_eq!(
            category.default_severity().to_string(),
            severity_name,
            "{name}"
        );
        assert_eq!(format!("{:?}", category.default_action()), action, "{name}");
        assert_eq!(category.default_public_message(), public, "{name}");
        seen += 1;
    }
    assert_eq!(seen, Category::ALL.len());
}

#[test]
fn test_golden_severity_names() {
    for row in rows(include_str!("golden/severity.txt")) {
        let [value, otel, label, display, band] = row[..] else {
            panic!("bad row {row:?}");
        };
        let severity = Severity::new(value.parse().unwrap()).unwrap();
        assert_eq!(severity.otel_name(), otel);
        assert_eq!(severity.label().unwrap_or("-"), label);
        assert_eq!(severity.to_string(), display);
        assert_eq!(format!("{:?}", severity.band()), band);
    }
}

#[test]
fn test_golden_kind_bits() {
    let named = [
        ("ERROR", Kinds::ERROR),
        ("WARNING", Kinds::WARNING),
        ("NOTICE", Kinds::NOTICE),
        ("DEPRECATED", Kinds::DEPRECATED),
        ("SECURITY", Kinds::SECURITY),
        ("FUTURE", Kinds::FUTURE),
        ("RUNTIME", Kinds::RUNTIME),
        ("RESOURCE", Kinds::RESOURCE),
        ("CONFIG", Kinds::CONFIG),
        ("USER", Kinds::USER),
        ("PANIC", Kinds::PANIC),
        ("AUDIT", Kinds::AUDIT),
        ("TRIGGER", Kinds::TRIGGER),
        ("PLATFORM", Kinds::PLATFORM),
        ("PROJECT", Kinds::PROJECT),
        ("ALL", Kinds::ALL),
        ("NONE", Kinds::NONE),
    ];
    for row in rows(include_str!("golden/kinds.txt")) {
        let [bit, constant, mask] = row[..] else {
            panic!("bad row {row:?}");
        };
        let mask = u64::from_str_radix(mask, 16).unwrap();
        match (bit, constant) {
            ("project(0)", "-") => assert_eq!(Kinds::project::<0>().bits(), mask),
            ("project(31)", "-") => assert_eq!(Kinds::project::<31>().bits(), mask),
            (_, name) => {
                let kinds = named.iter().find(|(n, _)| *n == name).unwrap().1;
                assert_eq!(kinds.bits(), mask, "{name}");
                if let Ok(bit) = bit.parse::<u32>() {
                    assert_eq!(mask, 1 << bit);
                    assert_eq!(kinds.to_string(), name);
                }
            }
        }
    }
}

pub const BILLING: Kinds = Kinds::project::<0>();

define_errors! {
    namespace = "DB";

    pub enum DbError {
        #[code(1, help = "Raise pool.size or lower the load.")]
        #[category(DeadlineExceeded)]
        #[public("The database is busy. Try again shortly.")]
        Timeout => "query timed out",

        #[code(2, help = "Restore the page from a replica, then run a scrub.")]
        #[category(DataLoss)]
        #[severity(CRITICAL)]
        #[kinds(ERROR | SECURITY)]
        Corrupt => "page failed its checksum",

        #[code(10, help = "Raise the tenant quota or wait for the window to reset.")]
        #[category(ResourceExhausted)]
        #[kinds(BILLING)]
        Quota => "tenant is over its quota",
    }
}

define_errors! {
    namespace = "DIS", width = 5;

    pub enum DistroError {
        #[code(1, help = "Fix the named field in distro.noml.")]
        #[category(InvalidArgument)]
        Config => "configuration error",
    }
}

define_errors! {
    pub enum PageError {
        NotFound => "page not found",
    }

    pub enum AppError {
        Other(std::fmt::Error) => transparent,
    }

    pub struct Cancelled => "operation cancelled";
}

#[test]
fn test_golden_probe_strings() {
    let cases: [(&str, &(dyn std::error::Error + 'static)); 7] = [
        ("DbError::Timeout", &DbError::Timeout),
        ("DbError::Corrupt", &DbError::Corrupt),
        ("DbError::Quota", &DbError::Quota),
        ("DistroError::Config", &DistroError::Config),
        ("PageError::NotFound", &PageError::NotFound),
        ("AppError::Other", &AppError::Other(std::fmt::Error)),
        ("Cancelled", &Cancelled),
    ];
    let golden: Vec<(String, String)> = rows(include_str!("golden/probe.txt"))
        .map(|row| {
            let probe = row[1].replace("\\0", "\0");
            (probe.split('|').nth(1).unwrap().to_string(), probe)
        })
        .collect();
    assert_eq!(golden.len(), cases.len());
    for (name, error) in cases {
        let (_, expected) = golden.iter().find(|(n, _)| n == name).unwrap();
        #[allow(deprecated)]
        let probe = error.description();
        assert_eq!(probe, expected, "{name}");
        let identity = identify(error).unwrap();
        assert_eq!(identity.type_name, name);
    }
    assert!(DbError::Timeout.source().is_none());
}
