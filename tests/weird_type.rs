#![cfg(feature = "serde")]

use serde::{Deserialize, Serialize};

mod field_serde {
    use serde::{
        de::{Deserializer, Error as DeError, Visitor},
        ser::Serializer,
    };
    use std::fmt::{Formatter, Result as FmtResult};

    struct WeirdTypeVisitor;

    impl<'de> Visitor<'de> for WeirdTypeVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
            f.write_str("null")
        }

        // Works with `serde_json` but not `simd-json`.
        fn visit_none<E: DeError>(self) -> Result<Self::Value, E> {
            Ok(true)
        }

        // Works with `simd-json` but not `serde_json`.
        fn visit_unit<E: DeError>(self) -> Result<Self::Value, E> {
            Ok(true)
        }
    }

    // Clippy will say this bool can be taken by value, but we need it to be
    // passed by reference because that's what serde does.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(_: &bool, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
        deserializer.deserialize_option(WeirdTypeVisitor)
    }
}

// Clippy situation is the same as above.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !value
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct WeirdType {
    // `serde::ser::SerializeStruct` exposes a `skip_field` method but
    // `serde::ser::Serializer` doesn't, so this "skip_serializing_if" is
    // present since the behavior can't be deferred to
    // `field_serde::serialize`.
    #[serde(default, skip_serializing_if = "is_false", with = "field_serde")]
    pub field: bool,
}

const MISSING_EXPECTED: WeirdType = WeirdType { field: false };
const MISSING_INPUT: &str = r#"{}"#;
const PRESENT_EXPECTED: WeirdType = WeirdType { field: true };
const PRESENT_INPUT: &str = r#"{
        "field": null
    }"#;

#[test]
fn test_serde_json() {
    assert_eq!(
        MISSING_EXPECTED,
        serde_json::from_str(MISSING_INPUT).unwrap()
    );
    assert_eq!(
        PRESENT_EXPECTED,
        serde_json::from_str(PRESENT_INPUT).unwrap()
    );
}

#[test]
fn test_simd_json() {
    unsafe {
        assert_eq!(
            MISSING_EXPECTED,
            simd_json::from_str(&mut MISSING_INPUT.to_owned()).unwrap()
        );
        assert_eq!(
            PRESENT_EXPECTED,
            simd_json::from_str(&mut PRESENT_INPUT.to_owned()).unwrap()
        );
    }
}
