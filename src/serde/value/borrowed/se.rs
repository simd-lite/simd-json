use crate::value::borrowed::Value;
use crate::StaticNode;
use serde_ext::ser::{
    self, Serialize, SerializeMap as SerializeMapTrait, SerializeSeq as SerializeSeqTrait,
};

impl<'a> Serialize for Value<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            Value::Static(StaticNode::Null) => serializer.serialize_unit(),
            Value::Static(StaticNode::Bool(b)) => serializer.serialize_bool(*b),
            Value::Static(StaticNode::F64(f)) => serializer.serialize_f64(*f),
            Value::Static(StaticNode::U64(i)) => serializer.serialize_u64(*i),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::U128(i)) => serializer.serialize_u128(*i),
            Value::Static(StaticNode::I64(i)) => serializer.serialize_i64(*i),
            #[cfg(feature = "128bit")]
            Value::Static(StaticNode::I128(i)) => serializer.serialize_i128(*i),
            Value::String(s) => serializer.serialize_str(&s),
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
                    let k: &str = &k;
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

/*
use super::{Map};
use crate::{stry, ErrorType, Result};
use std::marker::PhantomData;
use super::serde::to_value;

type Impossible<T> = ser::Impossible<T, Error>;

pub struct Serializer<'a> {
    marker: PhantomData<&'a u8>,
}
impl<'a> Default for Serializer<'a> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<'a> serde::Serializer for Serializer<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    type SerializeSeq = SerializeVec<'a>;
    type SerializeTuple = SerializeVec<'a>;
    type SerializeTupleStruct = SerializeVec<'a>;
    type SerializeTupleVariant = SerializeTupleVariant<'a>;
    type SerializeMap = SerializeMap<'a>;
    type SerializeStruct = SerializeMap<'a>;
    type SerializeStructVariant = SerializeStructVariant<'a>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value<'a>> {
        Ok(Value::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value<'a>> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value<'a>> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value<'a>> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Value<'a>> {
        Ok(Value::I64(value.into()))
    }

    #[cfg(feature = "arbitrary_precision")]
    serde_if_integer128! {
        fn serialize_i128(self, value: i128) -> Result<Value<'a>> {
            Ok(Value::Number(value.into()))
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value<'a>> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value<'a>> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value<'a>> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value<'a>> {
        Ok(Value::I64(value as i64))
    }

    #[cfg(feature = "arbitrary_precision")]
    serde_if_integer128! {
        fn serialize_u128(self, value: u128) -> Result<Value<'a>> {
            Ok(Value::Number(value.into()))
        }
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value<'a>> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value<'a>> {
        Ok(Value::F64(value))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value<'a>> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value<'a>> {
        Ok(Value::from(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value<'a>> {
        let vec = value.iter().map(|&b| Value::I64(b.into())).collect();
        Ok(Value::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value<'a>> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value<'a>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value<'a>> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value<'a>>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value<'a>>
    where
        T: Serialize,
    {
        let mut values = Map::new();
        values.insert(variant, stry!(to_value(&mut value)));
        Ok(Value::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value<'a>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Value<'a>>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            name: variant,
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap::Map {
            map: Map::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        match name {
            #[cfg(feature = "arbitrary_precision")]
            ::number::TOKEN => Ok(SerializeMap::Number { out_value: None }),
            #[cfg(feature = "raw_value")]
            ::raw::TOKEN => Ok(SerializeMap::RawValue { out_value: None }),
            _ => self.serialize_map(Some(len)),
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: variant,
            map: Map::new(),
        })
    }
}

pub struct SerializeVec<'a> {
    vec: Vec<Value<'a>>,
}

pub struct SerializeTupleVariant<'a> {
    name: &'a str,
    vec: Vec<Value<'a>>,
}

pub enum SerializeMap<'a> {
    Map {
        map: Map<'a>,
        next_key: Option<&'a str>,
    },
}

pub struct SerializeStructVariant<'a> {
    name: &'a str,
    map: Map<'a>,
}

impl<'a> serde::ser::SerializeSeq for SerializeVec<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(stry!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'a>> {
        Ok(Value::Array(self.vec))
    }
}

impl<'a> serde::ser::SerializeTuple for SerializeVec<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'a>> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a> serde::ser::SerializeTupleStruct for SerializeVec<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'a>> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a> serde::ser::SerializeTupleVariant for SerializeTupleVariant<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(stry!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'a>> {
        let mut object = Map::new();

        object.insert(&self.name, Value::Array(self.vec));

        Ok(Value::Object(object))
    }
}

impl<'a> serde::ser::SerializeMap for SerializeMap<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut next_key, ..
            } => {
                *next_key = Some(stry!(key.serialize(MapKeySerializer {
                    marker: PhantomData
                })));
                Ok(())
            }
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValue { .. } => unreachable!(),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut map,
                ref mut next_key,
            } => {
                let key = next_key.take();
                // Panic because this indicates a bug in the program rather than an
                // expected failure.
                let key = key.expect("serialize_value called before serialize_key");
                map.insert(key, stry!(to_value(&value)));
                Ok(())
            }
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValue { .. } => unreachable!(),
        }
    }

    fn end(self) -> Result<Value<'a>> {
        match self {
            SerializeMap::Map { map, .. } => Ok(Value::Object(map)),
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValue { .. } => unreachable!(),
        }
    }
}

struct MapKeySerializer<'a> {
    marker: PhantomData<&'a u8>,
}

fn key_must_be_a_string() -> Error {
    Error::generic(ErrorType::KeyMustBeAString)
}

impl<'a> serde_ext::Serializer for MapKeySerializer<'a> {
    type Ok = &'a str;
    type Error = Error;

    type SerializeSeq = Impossible<&'a str>;
    type SerializeTuple = Impossible<&'a str>;
    type SerializeTupleStruct = Impossible<&'a str>;
    type SerializeTupleVariant = Impossible<&'a str>;
    type SerializeMap = Impossible<&'a str>;
    type SerializeStruct = Impossible<&'a str>;
    type SerializeStructVariant = Impossible<&'a str>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_u8(self, _value: u8) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_u64(self, _value: u64) -> Result<Self::Ok> {
        //Ok(value.to_string())
        Err(key_must_be_a_string())
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        //Err(key_must_be_a_string())
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        //Err(key_must_be_a_string())
        Err(key_must_be_a_string())
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok> {
        // Ok({
        //     let mut s = String::new();
        //     s.push(value);
        //     s
        // })
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        // TODO: This is ugly
        //let s = value.to_owned();
        Ok(value)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }
}

impl<'a> serde::ser::SerializeStruct for SerializeMap<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        match *self {
            SerializeMap::Map { .. } => {
                stry!(serde::ser::SerializeMap::serialize_key(self, key));
                serde::ser::SerializeMap::serialize_value(self, value)
            }
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { ref mut out_value } => {
                if key == ::number::TOKEN {
                    *out_value = Some(value.serialize(NumberValueEmitter)?);
                    Ok(())
                } else {
                    Err(invalid_number())
                }
            }
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValue { ref mut out_value } => {
                if key == ::raw::TOKEN {
                    *out_value = Some(value.serialize(RawValueEmitter)?);
                    Ok(())
                } else {
                    Err(invalid_raw_value())
                }
            }
        }
    }

    fn end(self) -> Result<Value<'a>> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { out_value, .. } => {
                Ok(out_value.expect("number value was not emitted"))
            }
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValue { out_value, .. } => {
                Ok(out_value.expect("raw value was not emitted"))
            }
        }
    }
}

impl<'a> serde::ser::SerializeStructVariant for SerializeStructVariant<'a> {
    type Ok = Value<'a>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.map.insert(key, stry!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'a>> {
        let mut object = Map::new();

        object.insert(self.name, Value::Object(self.map));

        Ok(Value::Object(object))
    }
}

#[cfg(test)]
mod test {
    use super::{Map, Value};
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
*/
