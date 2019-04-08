use crate::{MaybeBorrowedString, Value};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

impl<'a> Serialize for Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Null => serializer.serialize_unit(),
            Value::F64(f) => serializer.serialize_f64(*f),
            Value::I64(i) => serializer.serialize_i64(*i),
            Value::String(MaybeBorrowedString::B(s)) => serializer.serialize_str(s),
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

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value, Error> {
        Ok(Value::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value, Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value, Error> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Value, Error> {
        Ok(Value::I64(value.into()))
    }

    #[cfg(feature = "arbitrary_precision")]
    serde_if_integer128! {
        fn serialize_i128(self, value: i128) -> Result<Value, Error> {
            Ok(Value::Number(value.into()))
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value, Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value, Error> {
        Ok(Value::Number(value.into()))
    }

    #[cfg(feature = "arbitrary_precision")]
    serde_if_integer128! {
        fn serialize_u128(self, value: u128) -> Result<Value, Error> {
            Ok(Value::Number(value.into()))
        }
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value, Error> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value, Error> {
        Ok(Number::from_f64(value).map_or(Value::Null, Value::Number))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value, Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value, Error> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value, Error> {
        let vec = value.iter().map(|&b| Value::Number(b.into())).collect();
        Ok(Value::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value, Error> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, Error>
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
    ) -> Result<Value, Error>
    where
        T: Serialize,
    {
        let mut values = Map::new();
        values.insert(String::from(variant), try!(to_value(&value)));
        Ok(Value::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Value, Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(SerializeMap::Map {
            map: Map::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
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
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: Map::new(),
        })
    }
}

pub struct SerializeVec<'a> {
    vec: Vec<Value<'a>>,
}

pub struct SerializeTupleVariant<'a> {
    name: String,
    vec: Vec<Value<'a>>,
}

pub enum SerializeMap<'a> {
    Map {
        map: Map<String, Value<'a>>,
        next_key: Option<String>,
    },
}

pub struct SerializeStructVariant<'a> {
    name: String,
    map: Map<String, Value<'a>>,
}
