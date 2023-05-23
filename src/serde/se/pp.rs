use crate::{serde_ext, stry, Error, ErrorType};
use serde_ext::ser;
use std::io::Write;
use std::result::Result;
use std::str;
use value_trait::generator::BaseGenerator;

macro_rules! iomap {
    ($e:expr) => {
        ($e).map_err(|err| Error::generic(ErrorType::Io(err)))
    };
}

/// Write a value to a vector
/// # Errors
/// when the data can not be written
#[inline]
pub fn to_vec_pretty<T>(to: &T) -> crate::Result<Vec<u8>>
where
    T: ser::Serialize + ?Sized,
{
    let v = Vec::with_capacity(512);
    let mut s = PrettySerializer::new(v);
    to.serialize(&mut s).map(|_| s.writer)
}

/// Write a value to a string
///
/// # Errors
/// when the data can not be written
#[inline]
pub fn to_string_pretty<T>(to: &T) -> crate::Result<String>
where
    T: ser::Serialize + ?Sized,
{
    to_vec_pretty(to).map(|v| unsafe { String::from_utf8_unchecked(v) })
}

/// Write a value to a string
/// # Errors
/// when the data can not be written
#[inline]
pub fn to_writer_pretty<T, W>(writer: W, to: &T) -> crate::Result<()>
where
    T: ser::Serialize + ?Sized,
    W: Write,
{
    let mut s = PrettySerializer::new(writer);
    to.serialize(&mut s)
}
struct PrettySerializer<W: Write> {
    writer: W,
    dent: u32,
}
impl<W: Write> PrettySerializer<W> {
    fn new(writer: W) -> Self {
        Self { writer, dent: 0 }
    }
}

impl<W> BaseGenerator for PrettySerializer<W>
where
    W: Write,
{
    type T = W;
    #[inline]
    fn get_writer(&mut self) -> &mut Self::T {
        &mut self.writer
    }
    #[inline]
    fn write_min(&mut self, _slice: &[u8], min: u8) -> std::io::Result<()> {
        self.writer.write_all(&[min])
    }
    #[inline]
    fn new_line(&mut self) -> std::io::Result<()> {
        self.write_char(b'\n').and_then(|_| match self.dent {
            0 => Ok(()),
            1 => self.get_writer().write_all(b"  "),
            2 => self.get_writer().write_all(b"    "),
            3 => self.get_writer().write_all(b"      "),
            4 => self.get_writer().write_all(b"        "),
            5 => self.get_writer().write_all(b"          "),
            6 => self.get_writer().write_all(b"            "),
            7 => self.get_writer().write_all(b"              "),
            8 => self.get_writer().write_all(b"                "),
            9 => self.get_writer().write_all(b"                  "),
            _ => {
                for _ in 0..(self.dent * 2) {
                    stry!(self.get_writer().write_all(b" "));
                }
                Ok(())
            }
        })
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}
struct SerializeSeq<'serializer, W: Write + 'serializer> {
    s: &'serializer mut PrettySerializer<W>,
    first: bool,
}
impl<'serializer, W> ser::SerializeSeq for SerializeSeq<'serializer, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeSeq {
            ref mut s,
            ref mut first,
            ..
        } = *self;
        if *first {
            *first = false;
            iomap!(s.new_line()).and_then(|_| value.serialize(&mut **s))
        } else {
            iomap!(s.write(b",").and_then(|_| s.new_line())).and_then(|_| value.serialize(&mut **s))
        }
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.first {
            Ok(())
        } else {
            self.s.dedent();
            iomap!(self.s.new_line().and_then(|_| self.s.write(b"]")))
        }
    }
}

impl<'serializer, W> ser::SerializeTuple for SerializeSeq<'serializer, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeSeq {
            ref mut s,
            ref mut first,
        } = *self;
        if *first {
            *first = false;
            iomap!(s.new_line()).and_then(|_| value.serialize(&mut **s))
        } else {
            iomap!(s.write(b",").and_then(|_| s.new_line())).and_then(|_| value.serialize(&mut **s))
        }
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.first {
            Ok(())
        } else {
            self.s.dedent();
            iomap!(self.s.new_line().and_then(|_| self.s.write(b"]")))
        }
    }
}

impl<'serializer, W> ser::SerializeTupleStruct for SerializeSeq<'serializer, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeSeq {
            ref mut s,
            ref mut first,
        } = *self;
        if *first {
            *first = false;
            iomap!(s.new_line()).and_then(|_| value.serialize(&mut **s))
        } else {
            iomap!(s.write(b",").and_then(|_| s.new_line())).and_then(|_| value.serialize(&mut **s))
        }
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.first {
            Ok(())
        } else {
            self.s.dedent();
            iomap!(self.s.new_line().and_then(|_| self.s.write(b"]")))
        }
    }
}

impl<'serializer, W> ser::SerializeTupleVariant for SerializeSeq<'serializer, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeSeq {
            ref mut s,
            ref mut first,
        } = *self;
        if *first {
            *first = false;
            iomap!(s.new_line()).and_then(|_| value.serialize(&mut **s))
        } else {
            iomap!(s.write(b",").and_then(|_| s.new_line())).and_then(|_| value.serialize(&mut **s))
        }
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.first {
            Ok(())
        } else {
            self.s.dedent();
            iomap!(self.s.new_line().and_then(|_| self.s.write(b"}")))
        }
    }
}

struct SerializeMap<'serializer, W: Write + 'serializer> {
    s: &'serializer mut PrettySerializer<W>,
    first: bool,
}

impl<'serializer, W> ser::SerializeMap for SerializeMap<'serializer, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeMap {
            ref mut s,
            ref mut first,
            ..
        } = *self;

        if *first {
            *first = false;
            iomap!(s.new_line())
                .and_then(|_| key.serialize(&mut **s))
                .and_then(|_| iomap!(s.write(b": ")))
        } else {
            iomap!(s.write(b",").and_then(|_| s.new_line()))
                .and_then(|_| key.serialize(&mut **s))
                .and_then(|_| iomap!(s.write(b": ")))
        }
    }
    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeMap { ref mut s, .. } = *self;
        value.serialize(&mut **s)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.first {
            Ok(())
        } else {
            self.s.dedent();
            iomap!(self.s.new_line().and_then(|_| self.s.write(b"}")))
        }
    }
}

impl<'serializer, W> ser::SerializeStruct for SerializeMap<'serializer, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeMap {
            ref mut s,
            ref mut first,
            ..
        } = *self;
        if *first {
            *first = false;
            iomap!(s
                .new_line()
                .and_then(|_| s.write_simple_string(key))
                .and_then(|_| s.write(b": ")))
            .and_then(|_| value.serialize(&mut **s))
        } else {
            iomap!(s
                .write(b",")
                .and_then(|_| s.write_simple_string(key))
                .and_then(|_| s.write(b": ")))
            .and_then(|_| value.serialize(&mut **s))
        }
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.first {
            Ok(())
        } else {
            self.s.dedent();
            iomap!(self.s.new_line().and_then(|_| self.s.write(b"}")))
        }
    }
}

struct SerializeStructVariant<'serializer, W: Write + 'serializer> {
    s: &'serializer mut PrettySerializer<W>,
    first: bool,
}

impl<'serializer, W> ser::SerializeStructVariant for SerializeStructVariant<'serializer, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde_ext::Serialize,
    {
        let SerializeStructVariant {
            ref mut s,
            ref mut first,
            ..
        } = *self;
        if *first {
            *first = false;
            s.indent();
            iomap!(s
                .new_line()
                .and_then(|_| s.write_simple_string(key))
                .and_then(|_| s.write(b": ")))
            .and_then(|_| value.serialize(&mut **s))
        } else {
            iomap!(s
                .write(b",")
                .and_then(|_| s.write_simple_string(key))
                .and_then(|_| s.write(b": ")))
            .and_then(|_| value.serialize(&mut **s))
        }
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.s.dedent();
        iomap!(self.s.new_line().and_then(|_| self.s.write(b"}"))).and_then(move |_| {
            if self.first {
                Ok(())
            } else {
                self.s.dedent();

                iomap!(self.s.new_line().and_then(|_| self.s.write(b"}")))
            }
        })
    }
}

impl<'writer, W> ser::Serializer for &'writer mut PrettySerializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeSeq<'writer, W>;
    type SerializeTuple = SerializeSeq<'writer, W>;
    type SerializeTupleStruct = SerializeSeq<'writer, W>;
    type SerializeTupleVariant = SerializeSeq<'writer, W>;
    type SerializeMap = SerializeMap<'writer, W>;
    type SerializeStruct = SerializeMap<'writer, W>;
    type SerializeStructVariant = SerializeStructVariant<'writer, W>;
    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            iomap!(self.write(b"true"))
        } else {
            iomap!(self.write(b"false"))
        }
    }
    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }
    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v))
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_float(f64::from(v)))
    }
    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_float(v))
    }
    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        // taken from: https://docs.serde.rs/src/serde_json/ser.rs.html#213
        let mut buf = [0; 4];
        iomap!(self.write_simple_string(v.encode_utf8(&mut buf)))
    }
    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_string(v))
    }
    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write(b"[").and_then(|_| {
            if let Some((first, rest)) = v.split_first() {
                self.indent();
                self.new_line().and_then(|_| {
                    self.write_int(*first).and_then(|_| {
                        for v in rest {
                            self.write(b",").and_then(|_| self.write_int(*v))?;
                        }
                        self.dedent();
                        self.new_line().and_then(|_| self.write(b"]"))
                    })
                })
            } else {
                self.write(b"]")
            }
        }))
    }
    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }
    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde_ext::Serialize,
    {
        value.serialize(self)
    }
    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write(b"null"))
    }
    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }
    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_simple_string(variant))
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde_ext::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde_ext::Serialize,
    {
        iomap!(self
            .write(b"{")
            .and_then(|_| self.write_simple_string(variant))
            .and_then(|_| self.write(b": ")))
        .and_then(|_| value.serialize(&mut *self))
        .and_then(|_| iomap!(self.write(b"}")))
    }
    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if len == Some(0) {
            iomap!(self.write(b"[]"))
        } else {
            self.indent();
            iomap!(self.write(b"["))
        }
        .map(move |_| SerializeSeq {
            s: self,
            first: true,
        })
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.indent();
        iomap!(self
            .write(b"{")
            .and_then(|_| self.new_line())
            .and_then(|_| self.write_simple_string(variant))
            .and_then(|_| self.write(b": ")))
        .and_then(move |_| self.serialize_seq(Some(len)))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if len == Some(0) {
            iomap!(self.write(b"{}"))
        } else {
            self.indent();
            iomap!(self.write(b"{"))
        }
        .map(move |_| SerializeMap {
            s: self,
            first: true,
        })
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.indent();
        iomap!(self
            .write(b"{")
            .and_then(|_| self.new_line())
            .and_then(|_| self.write_simple_string(variant))
            .and_then(|_| self.write(b": ")))
        .and_then(move |_| {
            if len == 0 {
                iomap!(self.write(b"{}"))
            } else {
                iomap!(self.write(b"{"))
            }
            .map(move |_| SerializeStructVariant {
                s: self,
                first: true,
            })
        })
    }
}

#[cfg(test)]
mod test {
    use crate::from_slice;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::{OwnedValue as Value, StaticNode};
    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    #[cfg(not(feature = "128bit"))]
    #[cfg(not(target_arch = "wasm32"))]
    fn arb_json_value() -> BoxedStrategy<Value> {
        let leaf = prop_oneof![
            Just(Value::Static(StaticNode::Null)),
            any::<bool>().prop_map(Value::from),
            //(-1.0e306f64..1.0e306f64).prop_map(Value::from), // damn you float!
            any::<i8>().prop_map(Value::from),
            any::<i16>().prop_map(Value::from),
            any::<i32>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            any::<u8>().prop_map(Value::from),
            any::<u16>().prop_map(Value::from),
            any::<u32>().prop_map(Value::from),
            any::<u64>().prop_map(Value::from),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::from),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(Value::from),
                ]
            },
        )
        .boxed()
    }

    #[cfg(feature = "128bit")]
    #[cfg(not(target_arch = "wasm32"))]
    fn arb_json_value() -> BoxedStrategy<Value> {
        let leaf = prop_oneof![
            Just(Value::Static(StaticNode::Null)),
            any::<bool>().prop_map(Value::from),
            //(-1.0e306f64..1.0e306f64).prop_map(Value::from), // damn you float!
            any::<i8>().prop_map(Value::from),
            any::<i16>().prop_map(Value::from),
            any::<i32>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            any::<u8>().prop_map(Value::from),
            any::<u16>().prop_map(Value::from),
            any::<u32>().prop_map(Value::from),
            any::<u64>().prop_map(Value::from),
            any::<i128>().prop_map(Value::from),
            any::<u128>().prop_map(Value::from),
            ".*".prop_map(Value::from),
        ];
        leaf.prop_recursive(
            8,   // 8 levels deep
            256, // Shoot for maximum size of 256 nodes
            10,  // We put up to 10 items per collection
            |inner| {
                prop_oneof![
                    // Take the inner strategy and make the two recursive cases.
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::from),
                    prop::collection::hash_map(".*", inner, 0..10).prop_map(Value::from),
                ]
            },
        )
        .boxed()
    }

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #![proptest_config(ProptestConfig {
            // Setting both fork and timeout is redundant since timeout implies
            // fork, but both are shown for clarity.
            // Disabled for code coverage, enable to track bugs
            // fork: true,
            .. ProptestConfig::default()
        })]

        #[test]
        fn prop_json_encode_decode(val in arb_json_value()) {
            let mut encoded = crate::to_vec_pretty(&val).expect("to_vec_pretty");
            println!("{}", String::from_utf8_lossy(&encoded.clone()));
            let res: Value = crate::from_slice(encoded.as_mut_slice()).expect("can't convert");
            assert_eq!(val, res);
        }
        #[test]
        fn prop_serd_compat(val in arb_json_value()) {
            let simd = crate::to_string_pretty(&val).expect("to_string_pretty");
            let serde = serde_json::to_string_pretty(&val).expect("to_string_pretty");
            assert_eq!(simd, serde);
        }
    }

    #[test]
    fn prettyfy() {
        let v = crate::json!({"key1":{}, "key2":[], "key3":[1,{"key4":null}]});
        let mut res = crate::to_vec_pretty(&v).expect("encoding failed");
        let v2: Value = from_slice(&mut res).expect("generated bad json");
        assert_eq!(v, v2);
    }
}
