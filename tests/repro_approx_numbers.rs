// Valid JSON numbers the default parser accepts as floats; `approx-number-parsing` must too.
fn parse(s: &str) -> simd_json::OwnedValue {
    let mut b = s.as_bytes().to_vec();
    simd_json::to_owned_value(&mut b).unwrap_or_else(|e| panic!("{s}: {e:?}"))
}

#[test]
fn repro_approx_integer_part_above_u64_max() {
    assert_eq!(
        parse("18446744073709551616.5"),
        simd_json::OwnedValue::from(18446744073709551616.5_f64)
    );
    assert_eq!(
        parse("-18446744073709551619.0"),
        simd_json::OwnedValue::from(-18446744073709551619.0_f64)
    );
    assert_eq!(
        parse("18446744073709551616e0"),
        simd_json::OwnedValue::from(18446744073709551616.0_f64)
    );
}

#[test]
fn repro_approx_zero_exponent_is_float() {
    assert_eq!(parse("1e0"), simd_json::OwnedValue::from(1.0_f64));
    assert_eq!(parse("123E-0"), simd_json::OwnedValue::from(123.0_f64));
    assert_eq!(
        parse("12345678901234567e0"),
        simd_json::OwnedValue::from(12345678901234567.0_f64)
    );
}
