use crate::{Map, MaybeBorrowedString, Number, Value};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

#[cfg(not(feature = "no-borrow"))]
impl<'de> Deserialize<'de> for Value<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Value<'de>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(feature = "no-borrow")]
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    #[cfg(not(feature = "no-borrow"))]
    type Value = Value<'de>;

    #[cfg(feature = "no-borrow")]
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an JSONesque value")
    }

    /****************** unit ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    /****************** bool ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    /****************** Option ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    /****************** enum ******************/
    /*
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where
        A: EnumAccess<'de>,
    {
    }
     */

    /****************** i64 ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::I64(value as i64)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::I64(value as i64)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::I64(value as i64)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::I64(value)))
    }

    /****************** u64 ******************/

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::I64(value as i64)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::I64(value as i64)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::I64(value as i64)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use std::i64;
        if value <= i64::MAX as u64 {
            Ok(Value::Number(Number::I64(value as i64)))
        } else {
            Err(E::custom(format!("Integer out of range: {}", value)))
        }
    }

    /****************** f64 ******************/

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::F64(value as f64)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::F64(value)))
    }

    /****************** stringy stuff ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_char<E>(self, _value: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        unimplemented!()
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::from(value))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(MaybeBorrowedString::O(value.to_owned())))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(MaybeBorrowedString::O(value)))
    }

    /****************** byte stuff ******************/

    /*
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_str<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
    'a: 'de
        E: de::Error,
    {
      Ok(Value::String(value))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_string<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
      Ok(Value::String(&value))
    }
     */
    /****************** nexted stuff ******************/

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let size = map.size_hint().unwrap_or_default();

        let mut m = Map::with_capacity(size);
        while let Some(k) = map.next_key()? {
            let v = map.next_value()?;
            m.insert(k, v);
        }
        Ok(Value::Object(m))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let size = seq.size_hint().unwrap_or_default();

        let mut v = Vec::with_capacity(size);
        while let Some(e) = seq.next_element()? {
            v.push(e);
        }
        Ok(Value::Array(v))
    }
}
