use super::to_value;
use crate::value::borrowed::{Object, Value};
use crate::{stry, Error, ErrorType, Result, StaticNode};
use serde_ext::ser::{
    self, Serialize, SerializeMap as SerializeMapTrait, SerializeSeq as SerializeSeqTrait,
};
type Impossible<T> = ser::Impossible<T, Error>;

impl<'value> Serialize for Value<'value> {
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

pub struct Serializer {}
impl Default for Serializer {
    fn default() -> Self {
        Self {}
    }
}

impl serde::Serializer for Serializer {
    type Ok = Value<'static>;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value<'static>> {
        Ok(Value::Static(StaticNode::Bool(value)))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value<'static>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value<'static>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value<'static>> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Value<'static>> {
        Ok(Value::Static(StaticNode::I64(value)))
    }

    #[cfg(feature = "arbitrary_precision")]
    serde_if_integer128! {
        fn serialize_i128(self, value: i128) -> Result<Value<'static>> {
            Ok(Value::Number(value.into()))
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value<'static>> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value<'static>> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value<'static>> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    fn serialize_u64(self, value: u64) -> Result<Value<'static>> {
        Ok(Value::Static(StaticNode::I64(value as i64)))
    }

    #[cfg(feature = "arbitrary_precision")]
    serde_if_integer128! {
        fn serialize_u128(self, value: u128) -> Result<Value<'static>> {
            Ok(Value::Number(value.into()))
        }
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value<'static>> {
        self.serialize_f64(f64::from(value))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value<'static>> {
        Ok(Value::Static(StaticNode::F64(value)))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value<'static>> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value<'static>> {
        Ok(Value::from(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value<'static>> {
        Ok(value.iter().cloned().collect())
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value<'static>> {
        Ok(Value::Static(StaticNode::Null))
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value<'static>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value<'static>> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value<'static>>
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
    ) -> Result<Value<'static>>
    where
        T: Serialize,
    {
        let mut values = Object::with_capacity(1);
        values.insert(variant.into(), stry!(to_value(&value)));
        Ok(Value::from(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value<'static>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Value<'static>>
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
            name: variant.to_owned(),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap::Map {
            map: Object::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: variant.to_owned(),
            map: Object::new(),
        })
    }
}

pub struct SerializeVec {
    vec: Vec<Value<'static>>,
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value<'static>>,
}

pub enum SerializeMap {
    Map {
        map: Object<'static>,
        next_key: Option<String>,
    },
}

pub struct SerializeStructVariant {
    name: String,
    map: Object<'static>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value<'static>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(stry!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'static>> {
        Ok(Value::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value<'static>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'static>> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value<'static>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'static>> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value<'static>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(stry!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'static>> {
        let mut object = Object::with_capacity(1);

        object.insert(self.name.into(), Value::Array(self.vec));

        Ok(Value::from(object))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value<'static>;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        match *self {
            Self::Map {
                ref mut next_key, ..
            } => {
                *next_key = Some(stry!(key.serialize(MapKeySerializer {})));
                Ok(())
            }
            #[cfg(feature = "arbitrary_precision")]
            Self::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            Self::RawValue { .. } => unreachable!(),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        match *self {
            Self::Map {
                ref mut map,
                ref mut next_key,
            } => {
                let key = next_key.take();
                // Panic because this indicates a bug in the program rather than an
                // expected failure.
                let key = key.expect("serialize_value called before serialize_key");
                map.insert(key.into(), stry!(to_value(&value)));
                Ok(())
            }
            #[cfg(feature = "arbitrary_precision")]
            Self::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            Self::RawValue { .. } => unreachable!(),
        }
    }

    fn end(self) -> Result<Value<'static>> {
        match self {
            Self::Map { map, .. } => Ok(Value::from(map)),
            #[cfg(feature = "arbitrary_precision")]
            Self::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            Self::RawValue { .. } => unreachable!(),
        }
    }
}

struct MapKeySerializer {}

fn key_must_be_a_string() -> Error {
    Error::generic(ErrorType::KeyMustBeAString)
}

impl serde_ext::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String>;
    type SerializeTuple = Impossible<String>;
    type SerializeTupleStruct = Impossible<String>;
    type SerializeTupleVariant = Impossible<String>;
    type SerializeMap = Impossible<String>;
    type SerializeStruct = Impossible<String>;
    type SerializeStructVariant = Impossible<String>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(variant.to_owned())
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
        Ok(value.to_owned())
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

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value<'static>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        match *self {
            Self::Map { .. } => {
                stry!(serde::ser::SerializeMap::serialize_key(self, key));
                serde::ser::SerializeMap::serialize_value(self, value)
            }
            #[cfg(feature = "arbitrary_precision")]
            Self::Number { ref mut out_value } => {
                if key == ::number::TOKEN {
                    *out_value = Some(value.serialize(NumberValueEmitter)?);
                    Ok(())
                } else {
                    Err(invalid_number())
                }
            }
            #[cfg(feature = "raw_value")]
            Self::RawValue { ref mut out_value } => {
                if key == ::raw::TOKEN {
                    *out_value = Some(value.serialize(RawValueEmitter)?);
                    Ok(())
                } else {
                    Err(invalid_raw_value())
                }
            }
        }
    }

    fn end(self) -> Result<Value<'static>> {
        match self {
            Self::Map { .. } => serde::ser::SerializeMap::end(self),
            #[cfg(feature = "arbitrary_precision")]
            Self::Number { out_value, .. } => Ok(out_value.expect("number value was not emitted")),
            #[cfg(feature = "raw_value")]
            Self::RawValue { out_value, .. } => Ok(out_value.expect("raw value was not emitted")),
        }
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value<'static>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.map.insert(key.into(), stry!(to_value(&value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'static>> {
        let mut object = Object::with_capacity(1);

        object.insert(self.name.into(), Value::from(self.map));

        Ok(Value::from(object))
    }
}

#[cfg(test)]
mod test {
    use crate::serde::from_slice;
    /*
    use crate::{
        owned::to_value, owned::Object, owned::Value, to_borrowed_value, to_owned_value,
        Deserializer,
    };
    use halfbrown::HashMap;
    use proptest::prelude::*;
    */
    use serde::{Deserialize, Serialize};
    use serde_json;
    /*
    skipped due to https://github.com/simd-lite/simd-json/issues/65
    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    enum Enum {
        Opt1,
        Opt2,
    }
    impl std::default::Default for Enum {
        fn default() -> Self {
            Self::Opt1
        }
    }
    */
    #[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
    struct Map {
        key: u32,
    }
    #[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
    struct Obj {
        v_i128: i128,
        v_i64: i64,
        v_i32: i32,
        v_i16: i16,
        v_i8: i8,
        v_u128: u128,
        v_u64: u64,
        v_usize: usize,
        v_u32: u32,
        v_u66: u16,
        v_u8: u8,
        v_bool: bool,
        v_str: String,
        //v_enum: Enum,
        v_map: Map,
        v_arr: Vec<usize>,
        v_null: (),
    }
    #[test]
    fn from_slice_to_object() {
        let o = Obj::default();
        let vec = serde_json::to_vec(&o).expect("to_vec");
        let vec2 = crate::serde::to_vec(&o).expect("to_vec");
        assert_eq!(vec, vec2);
        let mut vec1 = vec.clone();
        let mut vec2 = vec.clone();

        println!("{}", serde_json::to_string_pretty(&o).expect("json"));
        let de: Obj = from_slice(&mut vec1).expect("from_slice");
        assert_eq!(o, de);
        let val = crate::to_owned_value(&mut vec2).expect("to_owned_value");

        let vec3 = serde_json::to_vec(&val).expect("to_vec");
        assert_eq!(vec, vec3);
    }

    use proptest::prelude::*;
    prop_compose! {
      fn obj_case()(
        v_i128 in any::<i64>().prop_map(i128::from),
        v_i64 in any::<i64>(),
        v_i32 in any::<i32>(),
        v_i16 in any::<i16>(),
        v_i8 in any::<i8>(),
        v_u128 in any::<u64>().prop_map(u128::from),
        v_u64 in any::<u64>(),
        v_usize in any::<u32>().prop_map(|v| v as usize),
        v_u32 in any::<u32>(),
        v_u66 in any::<u16>(),
        v_u8 in any::<u8>(),
        v_bool in any::<bool>(),
        v_str in ".*",
        ) -> Obj {
         Obj {
            v_i128,
            v_i64,
            v_i32,
            v_i16,
            v_i8,
            v_u128,
            v_u64,
            v_usize,
            v_u32,
            v_u66,
            v_u8,
            v_bool,
            v_str,
            ..Obj::default()
        }
      }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            .. ProptestConfig::default()
        })]

        #[test]
        fn prop_deserialize_obj(obj in obj_case()) {
            let mut vec = serde_json::to_vec(&obj).expect("to_vec");
            let vec1 = vec.clone();
            let vec2 = vec.clone();
            println!("{}", serde_json::to_string_pretty(&obj).expect("json"));
            let de: Obj = from_slice(&mut vec).expect("from_slice");
            prop_assert_eq!(&obj, &de);

            let borroed: crate::BorrowedValue = serde_json::from_slice(& vec1).expect("from_slice");
            let owned: crate::OwnedValue = serde_json::from_slice(& vec2).expect("from_slice");
            prop_assert_eq!(&borroed, &owned);

            let de: Obj = Obj::deserialize(borroed).expect("deserialize");
            prop_assert_eq!(&obj, &de);
            let de: Obj = Obj::deserialize(owned).expect("deserialize");
            prop_assert_eq!(&obj, &de);


        }
    }
}
