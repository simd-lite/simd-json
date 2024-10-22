// A small test to show that ordered-float does allow us to "do stuff" Eq-wise
// with values when we opt for an Eq-compatible representation of floats.
// Easiest way is simply to construct a type that derives Eq and include
// simd Values in it, construct it, and compare it! This won't even compile
// if we have got it wrong
#[cfg(feature = "ordered-float")]
use simd_json::{BorrowedValue, OwnedValue};

#[cfg(feature = "ordered-float")]
#[test]
fn test_values_as_hashmap_keys() {
    #[derive(Eq, PartialEq, Debug)]
    struct AnEqType {
        owned_value: OwnedValue,
        borrowed_value: BorrowedValue<'static>,
    }
    let an_eq_type = AnEqType {
        owned_value: OwnedValue::from("an-owned-value"),
        borrowed_value: BorrowedValue::from("a-borrowed-value"),
    };

    assert_eq!(
        an_eq_type,
        AnEqType {
            owned_value: OwnedValue::from("an-owned-value"),
            borrowed_value: BorrowedValue::from("a-borrowed-value")
        }
    );
}
