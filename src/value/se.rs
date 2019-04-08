use crate::{MaybeBorrowedString, Value};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

#[cfg(not(feature = "no-borrow"))]
#[cfg(feature = "no-borrow")]
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Null => serializer.serialize_unit(),
            Value::F64(f) => serializer.serialize_f64(*f),
            Value::I64(i) => serializer.serialize_i64(*i),
            Value::String(MaybeBorrowedString::O(s)) => serializer.serialize_str(&s),
            Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for e in v {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Value::Object(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Map, Value};
    use serde_json;

    #[test]
    fn null() {
        let v = Value::Null;
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "null")
    }

    #[test]
    fn bool_true() {
        let v = Value::Bool(true);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "true")
    }

    #[test]
    fn bool_false() {
        let v = Value::Bool(false);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "false")
    }

    #[test]
    fn float() {
        let v = Value::F64(1.0);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "1.0")
    }

    #[test]
    fn int() {
        let v = Value::I64(42);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "42")
    }

    #[test]
    fn arr() {
        let v = Value::Array(vec![Value::I64(42), Value::I64(23)]);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "[42,23]")
    }
    #[test]
    fn map() {
        let mut m = Map::new();
        m.insert("a".into(), Value::from(42));
        m.insert("b".into(), Value::from(23));
        let v = Value::Object(m);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, r#"{"a":42,"b":23}"#)
    }
}
