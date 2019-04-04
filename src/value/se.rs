use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap};
use crate::{Value, Number};

impl<'a> Serialize for Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Null => serializer.serialize_unit(),
            Value::Number(n) => match n {
                Number::F64(f) => serializer.serialize_f64(*f),
                Number::I64(i) => serializer.serialize_i64(*i),
            }
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for e in v {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Value::Map(m) => {
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
mod test{
    use serde_json;
    use crate::{Value, Number, Map};

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
        let v = Value::Number(Number::F64(1.0));
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "1.0")
    }

    #[test]
    fn int() {
        let v = Value::Number(Number::I64(42));
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "42")
    }


    #[test]
    fn arr() {
        let v = Value::Array(vec![Value::Number(Number::I64(42)), Value::Number(Number::I64(23))]);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "[42,23]")
    }
    #[test]
    fn map() {
        let mut m = Map::new();
        m.insert("a",Value::Number(Number::I64(42)));
        m.insert("b",Value::Number(Number::I64(23)));
        let v = Value::Map(m);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, r#"{"a":42,"b":23}"#)
    }
}
