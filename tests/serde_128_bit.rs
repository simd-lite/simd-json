#[cfg(all(feature = "serde_impl", feature = "128bit"))]
#[test]
#[ignore] // https://github.com/serde-rs/serde/issues/1717
fn lgostash_int_bug() {
    use serde::Deserialize;
    use simd_json::serde::from_slice;
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    pub enum RawMessageMetricValue {
        Boolean(bool),
        Float(f64),
        Long(i128),
        Integer(i64),
        String(String),
    }
    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Message {
        pub name: String,
        pub value: RawMessageMetricValue,
    }

    let mut d = String::from(
        r#"
{
"name": "max_unsafe_auto_id_timestamp",
"value": -9223372036854776000
}
"#,
    );
    let mut d = unsafe { d.as_bytes_mut() };
    let v_serde: Message = serde_json::from_slice(d).expect("serde_json");
    let v_simd: Message = from_slice(&mut d).expect("simd_json");
    assert_eq!(v_simd, v_serde);
}

#[cfg(all(feature = "serde_impl", feature = "128bit"))]
#[test]
fn lgostash_int_bug2() {
    use serde::Deserialize;
    use simd_json::serde::from_slice;
    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Message {
        pub name: String,
        pub value: i128,
    }

    let mut d = String::from(
        r#"
{
"name": "max_unsafe_auto_id_timestamp",
"value": -9223372036854776000
}
"#,
    );
    let mut d = unsafe { d.as_bytes_mut() };
    let v_serde: Message = serde_json::from_slice(d).expect("serde_json");
    let v_simd: Message = from_slice(&mut d).expect("simd_json");
    assert_eq!(v_simd, v_serde);
}
