//! Property tests (dev/PLAN.md 12.3): round trips, orderings, set laws, and
//! "never panics" for every parser and decoder.

use define_err::{identify, Category, Code, Kinds, Namespace, Secret, Severity};
use proptest::prelude::*;

/// Any valid namespace text: a letter, then up to five letters or digits.
fn namespace_text() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9]{0,5}"
}

fn any_code() -> impl Strategy<Value = Code> {
    (namespace_text(), 1_u8..=8, 0_u32..=Code::MAX_NUMBER).prop_map(|(text, width, number)| {
        Namespace::try_with_width(&text, width)
            .unwrap()
            .code(number)
            .unwrap()
    })
}

proptest! {
    #[test]
    fn prop_code_round_trips_through_u64_and_text(code in any_code()) {
        prop_assert_eq!(Code::from_u64(code.as_u64()), Some(code));
        prop_assert_eq!(Code::from_u64(code.as_u64()).unwrap().as_u64(), code.as_u64());
        prop_assert_eq!(Code::parse(&code.to_string()), Ok(code));
    }

    #[test]
    fn prop_code_order_is_namespace_then_number(a in any_code(), b in any_code()) {
        let key = |c: Code| (c.namespace().as_str().to_string(), c.number());
        prop_assert_eq!(a.cmp(&b), key(a).cmp(&key(b)));
    }

    #[test]
    fn prop_code_equality_ignores_width(text in namespace_text(), number in 0_u32..1000, w1 in 4_u8..=8, w2 in 4_u8..=8) {
        let a = Namespace::try_with_width(&text, w1).unwrap().code(number).unwrap();
        let b = Namespace::try_with_width(&text, w2).unwrap().code(number).unwrap();
        prop_assert_eq!(a, b);
        prop_assert_eq!(a.identity(), b.identity());
    }

    #[test]
    fn prop_code_parse_never_panics(text in ".{0,40}") {
        let _ = Code::parse(&text);
    }

    #[test]
    fn prop_code_from_u64_never_panics(raw in any::<u64>()) {
        if let Some(code) = Code::from_u64(raw) {
            prop_assert_eq!(code.as_u64(), raw);
            prop_assert_eq!(Code::parse(&code.to_string()).map(Code::identity), Ok(code.identity()));
        }
    }

    #[test]
    fn prop_severity_round_trips(value in 1_u8..=24) {
        let severity = Severity::new(value).unwrap();
        prop_assert_eq!(Severity::parse(&severity.to_string()), Some(severity));
        prop_assert_eq!(Severity::parse(severity.otel_name()), Some(severity));
    }

    #[test]
    fn prop_severity_new_accepts_exactly_one_to_24(value in any::<u8>()) {
        prop_assert_eq!(Severity::new(value).is_some(), (1..=24).contains(&value));
    }

    #[test]
    fn prop_severity_parse_never_panics(text in ".{0,16}") {
        let _ = Severity::parse(&text);
    }

    #[test]
    fn prop_kinds_set_laws(a in any::<u64>(), b in any::<u64>(), c in any::<u64>()) {
        let (a, b, c) = (Kinds::from_bits(a), Kinds::from_bits(b), Kinds::from_bits(c));
        prop_assert_eq!(a | b, b | a);
        prop_assert!(a.contains(a));
        prop_assert!((a | b).contains(a));
        if a.contains(b) && b.contains(c) {
            prop_assert!(a.contains(c));
        }
        prop_assert_eq!(a.union(b), a | b);
        prop_assert_eq!((a - b) & b, Kinds::NONE);
        prop_assert_eq!(a.iter().fold(Kinds::NONE, |acc, k| acc | k), a);
    }

    #[test]
    fn prop_project_kinds_never_touch_platform_bits(bit in 0_u8..32) {
        prop_assert!(!Kinds::try_project(bit).unwrap().intersects(Kinds::PLATFORM));
    }

    #[test]
    fn prop_category_parse_never_panics(text in ".{0,40}") {
        let _ = Category::parse(&text);
    }

    #[test]
    fn prop_identify_never_panics_on_any_description(text in ".{0,200}") {
        let error = Described(text);
        let _ = identify(&error);
    }

    #[test]
    fn prop_secret_never_reveals_its_value(value in "[a-z]{4,20}", width in 0_usize..40) {
        let secret = Secret::new(value.clone());
        for text in [
            format!("{secret}"),
            format!("{secret:?}"),
            format!("{secret:#?}"),
            format!("{secret:>width$}"),
            format!("{secret:.3}"),
            format!("{secret:^width$}"),
        ] {
            prop_assert!(!text.contains(&value));
        }
    }
}

/// A foreign error whose `description()` is any text, to feed the decoder.
#[derive(Debug)]
struct Described(String);

impl std::fmt::Display for Described {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("described")
    }
}

impl std::error::Error for Described {
    fn description(&self) -> &str {
        &self.0
    }
}

#[test]
fn test_category_round_trips() {
    for category in Category::ALL {
        assert_eq!(Category::from_grpc(category.grpc_code()), Some(category));
        assert_eq!(Category::parse(category.name()), Some(category));
        assert_eq!(
            Category::parse(&category.name().to_uppercase()),
            Some(category)
        );
    }
}
