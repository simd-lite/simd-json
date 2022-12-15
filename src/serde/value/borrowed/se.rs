use super::to_value;
use crate::StaticNode;
use crate::{
    cow::Cow,
    stry,
    value::borrowed::{Object, Value},
    Error, ErrorType, Result,
};
use serde_ext::ser::{
    self, Serialize, SerializeMap as SerializeMapTrait, SerializeSeq as SerializeSeqTrait,
};
use std::marker::PhantomData;

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
            Value::String(s) => serializer.serialize_str(s),
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
                    let k: &str = k;
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

pub struct Serializer<'se> {
    marker: PhantomData<&'se u8>,
}

impl<'se> Default for Serializer<'se> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<'se> serde::Serializer for Serializer<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    type SerializeSeq = SerializeVec<'se>;
    type SerializeTuple = SerializeVec<'se>;
    type SerializeTupleStruct = SerializeVec<'se>;
    type SerializeTupleVariant = SerializeTupleVariant<'se>;
    type SerializeMap = SerializeMap<'se>;
    type SerializeStruct = SerializeMap<'se>;
    type SerializeStructVariant = SerializeStructVariant<'se>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value<'se>> {
        Ok(Value::Static(StaticNode::Bool(value)))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value<'se>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value<'se>> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value<'se>> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Value<'se>> {
        Ok(Value::Static(StaticNode::I64(value)))
    }

    #[cfg(feature = "128bit")]
    fn serialize_i128(self, value: i128) -> Result<Value<'se>> {
        Ok(Value::Static(StaticNode::I128(value)))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value<'se>> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value<'se>> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value<'se>> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value<'se>> {
        Ok(Value::Static(StaticNode::U64(value)))
    }

    #[cfg(feature = "128bit")]
    fn serialize_u128(self, value: u128) -> Result<Value<'se>> {
        Ok(Value::Static(StaticNode::U128(value)))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value<'se>> {
        self.serialize_f64(f64::from(value))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value<'se>> {
        Ok(Value::Static(StaticNode::F64(value)))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value<'se>> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value<'se>> {
        Ok(Value::from(value.to_owned()))
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<Value<'se>> {
        Ok(value.iter().copied().collect())
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value<'se>> {
        Ok(Value::Static(StaticNode::Null))
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value<'se>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value<'se>> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value<'se>>
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
    ) -> Result<Value<'se>>
    where
        T: Serialize,
    {
        let mut values = Object::with_capacity(1);
        let x = stry!(to_value(value));
        values.insert(variant.into(), x);
        Ok(Value::from(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value<'se>> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Value<'se>>
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
        Ok(SerializeMap {
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
            name: variant,
            map: Object::new(),
        })
    }
}

pub struct SerializeVec<'se> {
    vec: Vec<Value<'se>>,
}

pub struct SerializeTupleVariant<'se> {
    name: &'se str,
    vec: Vec<Value<'se>>,
}

pub struct SerializeMap<'se> {
    map: Object<'se>,
    next_key: Option<Cow<'se, str>>,
}

pub struct SerializeStructVariant<'se> {
    name: &'se str,
    map: Object<'se>,
}

impl<'se> serde::ser::SerializeSeq for SerializeVec<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(stry!(to_value(value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'se>> {
        Ok(Value::Array(self.vec))
    }
}

impl<'se> serde::ser::SerializeTuple for SerializeVec<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'se>> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'se> serde::ser::SerializeTupleStruct for SerializeVec<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value<'se>> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'se> serde::ser::SerializeTupleVariant for SerializeTupleVariant<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.vec.push(stry!(to_value(value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'se>> {
        let mut object = Object::with_capacity(1);
        object.insert(self.name.into(), Value::Array(self.vec));

        Ok(Value::Object(Box::new(object)))
    }
}

impl<'se> serde::ser::SerializeMap for SerializeMap<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.next_key = Some(stry!(key.serialize(MapKeySerializer {
            marker: PhantomData
        })));
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, stry!(to_value(value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'se>> {
        Ok(Value::Object(Box::new(self.map)))
    }
}

struct MapKeySerializer<'se> {
    marker: PhantomData<&'se u8>,
}

fn key_must_be_a_string() -> Error {
    Error::generic(ErrorType::KeyMustBeAString)
}

impl<'se> serde_ext::Serializer for MapKeySerializer<'se> {
    type Ok = Cow<'se, str>;
    type Error = Error;

    type SerializeSeq = Impossible<Cow<'se, str>>;
    type SerializeTuple = Impossible<Cow<'se, str>>;
    type SerializeTupleStruct = Impossible<Cow<'se, str>>;
    type SerializeTupleVariant = Impossible<Cow<'se, str>>;
    type SerializeMap = Impossible<Cow<'se, str>>;
    type SerializeStruct = Impossible<Cow<'se, str>>;
    type SerializeStructVariant = Impossible<Cow<'se, str>>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(Cow::from(variant))
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

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(value.to_string().into())
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        Err(key_must_be_a_string())
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s.into()
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        // TODO: we copy `value` here this is not idea but safe
        Ok(Cow::from(value.to_string()))
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

impl<'se> serde::ser::SerializeStruct for SerializeMap<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        stry!(serde::ser::SerializeMap::serialize_key(self, key));
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value<'se>> {
        serde::ser::SerializeMap::end(self)
    }
}

impl<'se> serde::ser::SerializeStructVariant for SerializeStructVariant<'se> {
    type Ok = Value<'se>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.map.insert(key.into(), stry!(to_value(value)));
        Ok(())
    }

    fn end(self) -> Result<Value<'se>> {
        let mut object = Object::with_capacity(1);

        object.insert(self.name.into(), self.map.into());

        Ok(Value::Object(Box::new(object)))
    }
}

#[cfg(test)]
mod test {
    use super::Value;
    use crate::serde::from_slice;
    use halfbrown::HashMap;
    use serde::{Deserialize, Serialize};
    use serde_json;
    use value_trait::StaticNode;

    #[test]
    fn null() {
        let v = Value::Static(crate::StaticNode::Null);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "null");
    }

    #[test]
    fn bool_true() {
        let v = Value::Static(StaticNode::Bool(true));
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "true");
    }

    #[test]
    fn bool_false() {
        let v = Value::Static(StaticNode::Bool(false));
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "false");
    }

    #[test]
    fn float() {
        let v = Value::Static(StaticNode::F64(1.0));
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "1.0");
    }

    #[test]
    fn stringlike() {
        let v = Value::from("snot".to_string());
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "\"snot\"");

        let v = Value::from("snot");
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "\"snot\"");
    }

    #[test]
    fn int() {
        let v = Value::Static(StaticNode::I64(42));
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "42");
    }

    #[test]
    fn arr() {
        let v = Value::Array(vec![
            Value::Static(StaticNode::I64(42)),
            Value::Static(StaticNode::I64(23)),
        ]);
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, "[42,23]");
    }

    #[test]
    fn map() {
        let mut m = HashMap::new();
        m.insert("a".into(), Value::from(42));
        m.insert("b".into(), Value::from(23));
        let v = Value::Object(Box::new(m));
        let s = serde_json::to_string(&v).expect("Failed to serialize");
        assert_eq!(s, r#"{"a":42,"b":23}"#);
    }

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
        v_u16: u16,
        v_u8: u8,
        v_bool: bool,
        v_str: String,
        v_char: char,
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
        let val = crate::to_borrowed_value(&mut vec2).expect("to_borrowed_value");

        let vec3 = serde_json::to_vec(&val).expect("to_vec");
        assert_eq!(vec, vec3);
    }

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;
    #[cfg(not(target_arch = "wasm32"))]
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
        v_u16 in any::<u16>(),
        v_u8 in any::<u8>(),
        v_bool in any::<bool>(),
        v_str in ".*",
        v_char in any::<char>(),
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
            v_u16,
            v_u8,
            v_bool,
            v_str,
            v_char,
            ..Obj::default()
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
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

            let borrowed: crate::BorrowedValue = serde_json::from_slice(& vec1).expect("from_slice");
            let owned: crate::OwnedValue = serde_json::from_slice(& vec2).expect("from_slice");
            prop_assert_eq!(&borrowed, &owned);

            let de: Obj = Obj::deserialize(borrowed).expect("deserialize");
            prop_assert_eq!(&obj, &de);
            let de: Obj = Obj::deserialize(owned).expect("deserialize");
            prop_assert_eq!(&obj, &de);


        }
    }
}
