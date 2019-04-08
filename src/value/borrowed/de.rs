use super::{MaybeBorrowedString, Value};
use crate::{Error, Result};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;

impl<'de, 'a> de::Deserializer<'de> for Value<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::I64(n) => visitor.visit_i64(n),
            Value::F64(n) => visitor.visit_f64(n),
            Value::String(s) => match s {
                MaybeBorrowedString::B(s) => visitor.visit_borrowed_str(s),
                MaybeBorrowedString::O(s) => visitor.visit_string(s),
            },
            Value::Array(a) => visitor.visit_seq(Array(a.iter())),
            Value::Object(o) => visitor.visit_map(Object {
                i: o.iter(),
                v: Value::Null,
            }),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}

struct Array<'a, 'de>(std::slice::Iter<'de, Value<'a>>);

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for Array<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(v) = self.0.next() {
            seed.deserialize(*v).map(Some)
        } else {
            Ok(None)
        }
    }
}
struct Object<'a, 'de> {
    i: crate::halfbrown::Iter<'de, &'a str, Value<'a>>,
    v: Value<'a>,
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for Object<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((k, v)) = self.i.next() {
            self.v = *v;
            seed.deserialize(Value::from(*v)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.v)
    }
}
