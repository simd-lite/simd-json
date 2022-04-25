// A lot of this logic is a re-implementation or copy of serde_json::Value
use super::super::shared::MapKeyDeserializer;
use crate::value::owned::{Object, Value};
use crate::StaticNode;
use crate::{cow::Cow, ErrorType};
use crate::{stry, Error};
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;
use serde_ext::de::{EnumAccess, IntoDeserializer, VariantAccess};
use std::{fmt, slice};

impl<'de> de::Deserializer<'de> for Value {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Static(StaticNode::Null) => visitor.visit_unit(),
            Self::Static(StaticNode::Bool(b)) => visitor.visit_bool(b),
            Self::Static(StaticNode::I64(n)) => visitor.visit_i64(n),
            #[cfg(feature = "128bit")]
            Self::Static(StaticNode::I128(n)) => visitor.visit_i128(n),
            Self::Static(StaticNode::U64(n)) => visitor.visit_u64(n),
            #[cfg(feature = "128bit")]
            Self::Static(StaticNode::U128(n)) => visitor.visit_u128(n),
            Self::Static(StaticNode::F64(n)) => visitor.visit_f64(n),
            Self::String(s) => visitor.visit_string(s),
            Self::Array(a) => visit_array(a, visitor),
            Self::Object(o) => visit_object(o, visitor),
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
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        // FIXME: better error
                        return Err(crate::Deserializer::error(ErrorType::ExpectedString));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    // FIXME: better error
                    return Err(crate::Deserializer::error(ErrorType::ExpectedString));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            _other => {
                return Err(crate::Deserializer::error(ErrorType::ExpectedMap));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
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
            Self::Array(a) => visit_array(a, visitor),
            Self::Object(o) => visit_object(o, visitor),
            _ => Err(crate::Deserializer::error(ErrorType::ExpectedMap)),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct seq tuple
            tuple_struct map identifier ignored_any
    }
}

fn visit_array<'de, V>(array: Vec<Value>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer::new(array);
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

fn visit_object<'de, V>(object: Box<Object>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = ObjectDeserializer::new(object);
    let map = stry!(visitor.visit_map(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}
struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        Self {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
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

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
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

struct ObjectDeserializer {
    iter: <Object as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl ObjectDeserializer {
    #[allow(clippy::boxed_local, clippy::needless_pass_by_value)]
    fn new(map: Box<Object>) -> Self {
        Self {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for ObjectDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::from(key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
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

impl<'de> serde::Deserializer<'de> for ObjectDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
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

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSONesque value")
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
        Ok(Value::String(value.to_owned()))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value))
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
        while let Some(k) = map.next_key()? {
            let v = map.next_value()?;
            m.insert(k, v);
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

fn visit_array_ref<'de, V>(array: &'de [Value], visitor: V) -> Result<V::Value, Error>
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

fn visit_object_ref<'de, V>(object: &'de Object, visitor: V) -> Result<V::Value, Error>
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

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(crate::Deserializer::error(ErrorType::ExpectedMap))
            // None => Err(serde::de::Error::invalid_type(
            //     Unexpected::UnitVariant,
            //     &"newtype variant",
            // )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Array(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                     visit_array(v, visitor)
                }
            }
            // FIXME
            Some(_) | None => Err(crate::Deserializer::error(ErrorType::ExpectedMap))
            // Some(other) => Err(serde::de::Error::invalid_type(
            //     other.unexpected(),
            //     &"tuple variant",
            // )),
            // None => Err(serde::de::Error::invalid_type(
            //     Unexpected::UnitVariant,
            //     &"tuple variant",
            // )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Object(v)) => visit_object(v, visitor),
            // FIXME
            Some(_) | None => Err(crate::Deserializer::error(ErrorType::ExpectedMap))

            // Some(other) => Err(serde::de::Error::invalid_type(
            //     other.unexpected(),
            //     &"struct variant",
            // )),
            // None => Err(serde::de::Error::invalid_type(
            //     Unexpected::UnitVariant,
            //     &"struct variant",
            // )),
        }
    }
}

impl<'de> de::Deserializer<'de> for &'de Value {
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
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
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

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::Object(value) => {
                let mut iter = value.iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        // FIXME: better error
                        return Err(crate::Deserializer::error(ErrorType::ExpectedString));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    // FIXME: better error
                    return Err(crate::Deserializer::error(ErrorType::ExpectedString));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            _other => {
                return Err(crate::Deserializer::error(ErrorType::ExpectedMap));
            }
        };

        visitor.visit_enum(EnumRefDeserializer { variant, value })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct seq tuple
            tuple_struct map identifier ignored_any
    }
}

struct EnumRefDeserializer<'de> {
    variant: &'de str,
    value: Option<&'de Value>,
}

impl<'de> EnumAccess<'de> for EnumRefDeserializer<'de> {
    type Error = Error;
    type Variant = VariantRefDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantRefDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}
struct VariantRefDeserializer<'de> {
    value: Option<&'de Value>,
}

impl<'de> VariantAccess<'de> for VariantRefDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(crate::Deserializer::error(ErrorType::ExpectedMap))
            // None => Err(serde::de::Error::invalid_type(
            //     Unexpected::UnitVariant,
            //     &"newtype variant",
            // )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Array(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                     visit_array_ref(v, visitor)
                }
            }
            // FIXME
            Some(_) | None => Err(crate::Deserializer::error(ErrorType::ExpectedMap))
            // Some(other) => Err(serde::de::Error::invalid_type(
            //     other.unexpected(),
            //     &"tuple variant",
            // )),
            // None => Err(serde::de::Error::invalid_type(
            //     Unexpected::UnitVariant,
            //     &"tuple variant",
            // )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Object(v)) => visit_object_ref(v, visitor),
            // FIXME
            Some(_) | None => Err(crate::Deserializer::error(ErrorType::ExpectedMap))
            // Some(other) => Err(serde::de::Error::invalid_type(
            //     other.unexpected(),
            //     &"struct variant",
            // )),
            // None => Err(serde::de::Error::invalid_type(
            //     Unexpected::UnitVariant,
            //     &"struct variant",
            // )),
        }
    }
}
struct SeqRefDeserializer<'de> {
    iter: slice::Iter<'de, Value>,
}

impl<'de> SeqRefDeserializer<'de> {
    fn new(slice: &'de [Value]) -> Self {
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
    iter: <&'de Object as IntoIterator>::IntoIter,
    value: Option<&'de Value>,
}

impl<'de> MapRefDeserializer<'de> {
    fn new(map: &'de Object) -> Self {
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
        let result: Result<Person, _> = crate::to_owned_value(unsafe { raw_json.as_bytes_mut() })
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
            r#"{"name":"bob","middle_name": "frank", "friends":[], "pos": [1,2]}"#.to_string();
        let result: Result<Person, _> = crate::to_owned_value(unsafe { raw_json.as_bytes_mut() })
            .and_then(super::super::from_value);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize() {
        use halfbrown::{hashmap, HashMap};
        #[derive(serde::Deserialize, Debug, PartialEq)]
        #[serde(rename_all = "lowercase")]
        pub enum Rotate {
            Left,
            Right,
            Up,
            Down,
        }
        #[derive(serde::Deserialize, Debug, PartialEq)]
        pub struct Point {
            pub x: i64,
            pub y: i64,
            pub z: f64,
            pub rotate: Rotate,
        }
        #[derive(serde::Deserialize, Debug, PartialEq)]
        pub struct Person {
            pub name: String,
            pub middle_name: Option<String>,
            pub friends: Vec<String>,
            pub pos: Point,
            pub age: u64,
        }
        #[derive(serde::Deserialize, Debug, PartialEq)]
        pub struct TestStruct {
            pub key: HashMap<String, String>,
            pub vec: Vec<Vec<Option<u8>>>,
        }

        let mut raw_json =
            r#"{"name":"bob","middle_name": "frank", "friends":[], "pos": [-1, 2, -3.25, "up"], "age": 123}"#.to_string();
        let serde_result: Person = serde_json::from_str(&raw_json).expect("serde_json::from_str");
        let value =
            crate::to_owned_value(unsafe { raw_json.as_bytes_mut() }).expect("to_owned_value");
        let result: Person = super::super::from_refvalue(&value).expect("from_refvalue");
        let expected = Person {
            name: "bob".to_string(),
            middle_name: Some("frank".to_string()),
            friends: Vec::new(),
            pos: Point {
                x: -1,
                y: 2,
                z: -3.25_f64,
                rotate: Rotate::Up,
            },
            age: 123,
        };
        assert_eq!(result, expected);
        assert_eq!(result, serde_result);

        let mut raw_json = r#"{"key":{"subkey": "value"}, "vec":[[null], [1]]}"#.to_string();
        let value =
            crate::to_owned_value(unsafe { raw_json.as_bytes_mut() }).expect("to_owned_value");
        let result: TestStruct = super::super::from_refvalue(&value).expect("from_refvalue");
        let expected = TestStruct {
            key: hashmap!("subkey".to_string() => "value".to_string()),
            vec: vec![vec![None], vec![Some(1)]],
        };
        assert_eq!(result, expected);
    }

    #[cfg(feature = "128bit")]
    #[test]
    fn deserialize_128bit() {
        let value = i64::MIN as i128 - 1;
        let int128 = crate::OwnedValue::Static(crate::StaticNode::I128(value));
        let res: i128 = super::super::from_refvalue(&int128).expect("from_refvalue");
        assert_eq!(value, res);

        let value = u64::MAX as u128;
        let int128 = crate::OwnedValue::Static(crate::StaticNode::U128(value));
        let res: u128 = super::super::from_refvalue(&int128).expect("from_refvalue");
        assert_eq!(value, res);
    }
}
