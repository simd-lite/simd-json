use crate::*;
use serde_ext::ser;
use std::io::Write;
use std::result::Result;
use value_trait::generator::*;

macro_rules! iotry {
    ($e:expr) => {
        match $e {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(err) => {
                return ::std::result::Result::Err(Error::generic(ErrorType::IO(err)))
            }
        }
    };
}
macro_rules! qtry {
    ($e:expr) => {
        if let ::std::result::Result::Err(err) = $e {
            return ::std::result::Result::Err(err);
        }
    };
}

macro_rules! iomap {
    ($e:expr) => {
        ($e).map_err(|err| Error::generic(ErrorType::IO(err)))
    };
}

/// Write a value to a vector
#[inline]
pub fn to_vec<T>(to: &T) -> crate::Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let v = Vec::with_capacity(512);
    let mut s = Serializer(v);
    qtry!(to.serialize(&mut s));
    Ok(s.0)
}

/// Write a value to a string
#[inline]
pub fn to_string<T>(to: &T) -> crate::Result<String>
where
    T: ser::Serialize,
{
    to_vec(to).map(|v| unsafe { String::from_utf8_unchecked(v) })
}

/// Write a value to a string
#[inline]
pub fn to_writer<T, W>(writer: W, to: &T) -> crate::Result<()>
where
    T: ser::Serialize,
    W: Write,
{
    let mut s = Serializer(writer);
    to.serialize(&mut s)
}
struct Serializer<W: Write>(W);

impl<'w, W> BaseGenerator for Serializer<W>
where
    W: Write,
{
    type T = W;
    #[inline]
    fn get_writer(&mut self) -> &mut Self::T {
        &mut self.0
    }
    #[inline]
    fn write_min(&mut self, _slice: &[u8], min: u8) -> std::io::Result<()> {
        self.0.write_all(&[min])
    }
}
struct SerializeSeq<'s, W: Write + 's> {
    s: &'s mut Serializer<W>,
    first: bool,
}
impl<'s, W> ser::SerializeSeq for SerializeSeq<'s, W>
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
        } else {
            iotry!(s.write(b","));
        }
        value.serialize(&mut **s)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        iomap!(self.s.write(b"]"))
    }
}

impl<'s, W> ser::SerializeTuple for SerializeSeq<'s, W>
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
        } else {
            iotry!(s.write(b","));
        }
        value.serialize(&mut **s)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        iomap!(self.s.write(b"]"))
    }
}

impl<'s, W> ser::SerializeTupleStruct for SerializeSeq<'s, W>
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
        } else {
            iotry!(s.write(b","));
        }
        value.serialize(&mut **s)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        iomap!(self.s.write(b"]"))
    }
}

impl<'s, W> ser::SerializeTupleVariant for SerializeSeq<'s, W>
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
        } else {
            iotry!(s.write(b","));
        }
        value.serialize(&mut **s)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        iomap!(self.s.write(b"}"))
    }
}

struct SerializeMap<'s, W: Write + 's> {
    s: &'s mut Serializer<W>,
    first: bool,
}

impl<'s, W> ser::SerializeMap for SerializeMap<'s, W>
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
        } = *self;
        if *first {
            *first = false;
        } else {
            iotry!(s.write(b","));
        }
        qtry!(key.serialize(&mut **s));
        iomap!(s.write(b":"))
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
        iomap!(self.s.write(b"}"))
    }
}

impl<'s, W> ser::SerializeStruct for SerializeMap<'s, W>
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
        } = *self;
        if *first {
            *first = false;
        } else {
            iotry!(s.write(b","));
        }
        iotry!(s.write_string(key));
        iotry!(s.write(b":"));
        value.serialize(&mut **s)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        iomap!(self.s.write(b"}"))
    }
}

impl<'s, W> ser::SerializeStructVariant for SerializeMap<'s, W>
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
        } = *self;
        if *first {
            *first = false;
        } else {
            iotry!(s.write(b","));
        }
        iotry!(s.write_string(key));
        iotry!(s.write(b":"));
        value.serialize(&mut **s)
    }
    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        iomap!(self.s.write(b"}"))
    }
}

impl<'w, W> ser::Serializer for &'w mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeSeq<'w, W>;
    type SerializeTuple = SerializeSeq<'w, W>;
    type SerializeTupleStruct = SerializeSeq<'w, W>;
    type SerializeTupleVariant = SerializeSeq<'w, W>;
    type SerializeMap = SerializeMap<'w, W>;
    type SerializeStruct = SerializeMap<'w, W>;
    type SerializeStructVariant = SerializeMap<'w, W>;
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
        iomap!(self.write_int(v as i64))
    }
    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v as i64))
    }
    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v as i64))
    }
    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int(v as i64))
    }
    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_int128(v))
    }
    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_uint(v as u64))
    }
    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_uint(v as u64))
    }
    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_uint(v as u64))
    }
    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_uint(v as u64))
    }
    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_uint128(v))
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_float(v as f64))
    }
    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.write_float(v)?;
        Ok(())
    }
    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        // taken from: https://docs.serde.rs/src/serde_json/ser.rs.html#213
        let mut buf = [0; 4];
        iomap!(self.write_string(v.encode_utf8(&mut buf)))
    }
    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        iomap!(self.write_string(v))
    }
    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        iotry!(self.write(b"["));
        if let Some((first, rest)) = v.split_first() {
            iotry!(self.write_uint(*first as u64));
            for v in rest {
                iotry!(self.write(b","));
                iotry!(self.write_uint(*v as u64));
            }
        }
        iomap!(self.write(b"]"))
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
        iomap!(self.write_string(variant))
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
        iotry!(self.write(b"{"));
        iotry!(self.write_string(variant));
        iotry!(self.write(b":"));
        qtry!(value.serialize(&mut *self));
        iomap!(self.write(b"}"))
    }
    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if len == Some(0) {
            iotry!(self.write(b"["));
            Ok(SerializeSeq {
                s: self,
                first: true,
            })
        } else {
            iotry!(self.write(b"["));
            Ok(SerializeSeq {
                s: self,
                first: true,
            })
        }
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
        iotry!(self.write(b"{"));
        iotry!(self.write_string(variant));
        iotry!(self.write(b":"));
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if len == Some(0) {
            iotry!(self.write(b"{"));
            Ok(SerializeMap {
                s: self,
                first: true,
            })
        } else {
            iotry!(self.write(b"{"));
            Ok(SerializeMap {
                s: self,
                first: true,
            })
        }
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
        iotry!(self.write(b"{"));
        iotry!(self.write_string(variant));
        iotry!(self.write(b":"));
        self.serialize_map(Some(len))
    }
}
