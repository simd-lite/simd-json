// A lot of this logic is a re-implementation or copy of serde_json::Value
use super::super::shared::MapKeyDeserializer;
use crate::value::borrowed::{Object, Value};
use crate::Error;
use crate::StaticNode;
use crate::{cow::Cow, stry, ErrorType};
use serde_ext::de::{
    self, Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor,
};
use serde_ext::forward_to_deserialize_any;
use std::{fmt, slice};

impl<'de> de::Deserializer<'de> for Value<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Static(StaticNode::Null) => visitor.visit_unit(),
            Value::Static(StaticNode::Bool(b)) => visitor.visit_bool(b),
            Self::Static(StaticNode::I64(n)) => visitor.visit_i64(n),
            #[cfg(feature = "128bit")]
            Self::Static(StaticNode::I128(n)) => visitor.visit_i128(n),
            Self::Static(StaticNode::U64(n)) => visitor.visit_u64(n),
            #[cfg(feature = "128bit")]
            Self::Static(StaticNode::U128(n)) => visitor.visit_u128(n),
            Value::Static(StaticNode::F64(n)) => visitor.visit_f64(n),
            #[cfg(feature = "beef")]
            Value::String(s) => {
                if s.is_borrowed() {
                    visitor.visit_borrowed_str(s.unwrap_borrowed())
                } else {
                    visitor.visit_string(s.into_owned())
                }
            }
            #[cfg(not(feature = "beef"))]
            Value::String(s) => match s {
                Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
                Cow::Owned(s) => visitor.visit_string(s),
            },

            Value::Array(a) => visitor.visit_seq(Array(a.iter())),
            Value::Object(o) => visitor.visit_map(ObjectAccess {
                i: o.iter(),
                v: &Value::Static(StaticNode::Null),
            }),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if self == Self::Static(StaticNode::Null) {
            visitor.visit_unit()
        } else {
            visitor.visit_some(self)
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            // Give the visitor access to each element of the sequence.
            Value::Array(a) => visitor.visit_seq(Array(a.iter())),
            Value::Object(o) => visitor.visit_map(ObjectAccess {
                i: o.iter(),
                v: &Value::Static(StaticNode::Null),
            }),
            _ => Err(crate::Deserializer::error(ErrorType::ExpectedMap)),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map enum identifier ignored_any
    }
}

struct Array<'de, 'value: 'de>(std::slice::Iter<'de, Value<'value>>);

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'value> SeqAccess<'de> for Array<'value, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        //TODO: This is ugly
        self.0
            .next()
            .map_or(Ok(None), |v| seed.deserialize(v.clone()).map(Some))
    }
}

struct ObjectAccess<'de, 'value: 'de> {
    i: halfbrown::Iter<'de, Cow<'value, str>, Value<'value>>,
    v: &'de Value<'value>,
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'value> MapAccess<'de> for ObjectAccess<'value, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((k, v)) = self.i.next() {
            self.v = v;
            seed.deserialize(Value::String(k.clone())).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        //TODO: This is ugly
        seed.deserialize(self.v.clone())
    }
}

impl<'de> Deserialize<'de> for Value<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Value<'de>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an JSONesque value")
    }

    /****************** unit ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Static(StaticNode::Null))
    }

    /****************** bool ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Static(StaticNode::Bool(value)))
    }

    /****************** Option ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Static(StaticNode::Null))
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
        Ok(Value::Static(StaticNode::I64(i64::from(value))))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::I64(i64::from(value))))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::I64(i64::from(value))))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::I64(value)))
    }

    #[cfg(feature = "128bit")]
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::I128(value)))
    }

    /****************** u64 ******************/

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::U64(u64::from(value))))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::U64(u64::from(value))))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::U64(u64::from(value))))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::U64(value)))
    }

    #[cfg(feature = "128bit")]
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::U128(value)))
    }

    /****************** f64 ******************/

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::F64(f64::from(value))))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Static(StaticNode::F64(value)))
    }

    /****************** stringy stuff ******************/
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::from(value.to_string()))
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
        Ok(Value::String(value.to_owned().into()))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value.into()))
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

        let mut m = Object::with_capacity(size);
        while let Some(k) = map.next_key::<&str>()? {
            let v = map.next_value()?;
            m.insert(k.into(), v);
        }
        Ok(Value::from(m))
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

fn visit_array_ref<'de, V>(array: &'de [Value<'de>], visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqRefDeserializer::new(array);
    let seq = stry!(visitor.visit_seq(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_object_ref<'de, V>(object: &'de Object<'de>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapRefDeserializer::new(object);
    let map = stry!(visitor.visit_map(&mut deserializer));
    if deserializer.iter.next().is_none() {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

impl<'de> de::Deserializer<'de> for &'de Value<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Static(StaticNode::Null) => visitor.visit_unit(),
            Value::Static(StaticNode::Bool(b)) => visitor.visit_bool(*b),
            Value::Static(StaticNode::I64(n)) => visitor.visit_i64(*n),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::I128(n)) => visitor.visit_i128(*n),
            Value::Static(StaticNode::U64(n)) => visitor.visit_u64(*n),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::U128(n)) => visitor.visit_u128(*n),
            Value::Static(StaticNode::F64(n)) => visitor.visit_f64(*n),
            Value::String(ref s) => visitor.visit_borrowed_str(s),
            Value::Array(ref a) => visit_array_ref(a, visitor),
            Value::Object(ref o) => visit_object_ref(o, visitor),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if self == &Value::Static(StaticNode::Null) {
            visitor.visit_unit()
        } else {
            visitor.visit_some(self)
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            // Give the visitor access to each element of the sequence.
            Value::Array(ref a) => visit_array_ref(a, visitor),
            Value::Object(ref o) => visit_object_ref(o, visitor),
            _ => Err(crate::Deserializer::error(ErrorType::ExpectedMap)),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map enum identifier ignored_any
    }
}

struct SeqRefDeserializer<'de> {
    iter: slice::Iter<'de, Value<'de>>,
}

impl<'de> SeqRefDeserializer<'de> {
    fn new(slice: &'de [Value<'de>]) -> Self {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> serde::Deserializer<'de> for SeqRefDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = stry!(visitor.visit_seq(&mut self));
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(serde::de::Error::invalid_length(
                    len,
                    &"fewer elements in array",
                ))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapRefDeserializer<'de> {
    iter: <&'de Object<'de> as IntoIterator>::IntoIter,
    value: Option<&'de Value<'de>>,
}

impl<'de> MapRefDeserializer<'de> {
    fn new(map: &'de Object<'de>) -> Self {
        MapRefDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapRefDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::from(&**key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapRefDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn option_field_absent_owned() {
        #[derive(serde::Deserialize, Debug)]
        pub struct Person {
            pub name: String,
            pub middle_name: Option<String>,
            pub friends: Vec<String>,
        }
        let mut raw_json = r#"{"name":"bob","friends":[]}"#.to_string();
        let result: Result<Person, _> =
            crate::to_borrowed_value(unsafe { raw_json.as_bytes_mut() })
                .and_then(super::super::from_value);
        assert!(result.is_ok());
    }
    #[test]
    fn option_field_present_owned() {
        #[derive(serde::Deserialize, Debug)]
        pub struct Point {
            pub x: u64,
            pub y: u64,
        }
        #[derive(serde::Deserialize, Debug)]
        pub struct Person {
            pub name: String,
            pub middle_name: Option<String>,
            pub friends: Vec<String>,
            pub pos: Point,
        }

        let mut raw_json =
            r#"{"name":"bob","middle_name": "frank", "friends":[], "pos":[0,1]}"#.to_string();
        let result: Result<Person, _> =
            crate::to_borrowed_value(unsafe { raw_json.as_bytes_mut() })
                .and_then(super::super::from_value);
        assert!(result.is_ok());
    }
}
