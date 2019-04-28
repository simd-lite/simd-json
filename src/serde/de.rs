use crate::numberparse::Number;
use crate::*;
use serde_ext::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_ext::forward_to_deserialize_any;

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match stry!(self.next()) {
            (b'n', idx, _) => {
                stry!(self.parse_null(idx));
                visitor.visit_unit()
            }
            (b't', idx, _len) => visitor.visit_bool(stry!(self.parse_true(idx))),
            (b'f', idx, _len) => visitor.visit_bool(stry!(self.parse_false(idx))),
            (b'-', idx, _len) => match stry!(self.parse_number(idx, true)) {
                Number::F64(n) => visitor.visit_f64(n),
                Number::I64(n) => visitor.visit_i64(n),
            },
            (b'0'...b'9', idx, _len) => match stry!(self.parse_number(idx, false)) {
                Number::F64(n) => visitor.visit_f64(n),
                Number::I64(n) => visitor.visit_i64(n),
            },
            (b'"', idx, _len) => {
                // We don't do the short string optimisation as serde requires
                // additional checks
                visitor.visit_borrowed_str(stry!(self.parse_str_(idx)))
            }

            (b'[', _idx, len) => visitor.visit_seq(CommaSeparated::new(&mut self, len)),
            (b'{', _idx, len) => visitor.visit_map(CommaSeparated::new(&mut self, len)),
            (_c, idx, _len) => Err(self.error(idx, ErrorType::UnexpectedCharacter)),
        }
    }

    // Uses the `parse_bool` parsing function defined above to read the JSON
    // identifier `true` or `false` from the input.
    //
    // Parsing refers to looking at the input and deciding that it contains the
    // JSON value `true` or `false`.
    //
    // Deserialization refers to mapping that JSON value into Serde's data
    // model by invoking one of the `Visitor` methods. In the case of JSON and
    // bool that mapping is straightforward so the distinction may seem silly,
    // but in other cases Deserializers sometimes perform non-obvious mappings.
    // For example the TOML format has a Datetime type and Serde's data model
    // does not. In the `toml` crate, a Datetime in the input is deserialized by
    // mapping it to a Serde data model "struct" type with a special name and a
    // single field containing the Datetime represented as a string.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match stry!(self.next()) {
            (b't', idx, _) => visitor.visit_bool(stry!(self.parse_true(idx))),
            (b'f', idx, _) => visitor.visit_bool(stry!(self.parse_false(idx))),
            (_c, idx, _) => Err(self.error(idx, ErrorType::ExpectedBoolean)),
        }
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (c, idx, len) = stry!(self.next());
        if c != b'"' {
            return Err(self.error(idx, ErrorType::ExpectedString));
        }

        if len < 32 {
            return visitor.visit_borrowed_str(stry!(self.parse_short_str_(idx)));
        }
        visitor.visit_borrowed_str(stry!(self.parse_str_(idx)))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (c, idx, len) = stry!(self.next());
        if c != b'"' {
            return Err(self.error(idx, ErrorType::ExpectedString));
        }
        if len < 32 {
            return visitor.visit_str(stry!(self.parse_short_str_(idx)));
        }

        visitor.visit_str(stry!(self.parse_str_(idx)))
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: i64 = stry!(self.parse_signed());
        visitor.visit_i8(v as i8)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: i64 = stry!(self.parse_signed());
        visitor.visit_i16(v as i16)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: i64 = stry!(self.parse_signed());
        visitor.visit_i32(v as i32)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(stry!(self.parse_signed()))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: u64 = stry!(self.parse_unsigned());
        visitor.visit_u8(v as u8)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: u64 = stry!(self.parse_unsigned());
        visitor.visit_u16(v as u16)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: u64 = stry!(self.parse_unsigned());
        visitor.visit_u32(v as u32)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(stry!(self.parse_unsigned()))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v: f64 = stry!(self.parse_double());
        visitor.visit_f32(v as f32)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(stry!(self.parse_double()))
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (c, idx, _len) = stry!(self.peek());
        if c == b'n' {
            self.skip();
            stry!(self.parse_null(idx));
            //self.skip();
            visitor.visit_unit()
        } else {
            visitor.visit_some(self)
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (c, idx, _len) = stry!(self.next());
        if c != b'n' {
            return Err(self.error(idx, ErrorType::ExpectedNull));
        }
        stry!(self.parse_null(idx));
        visitor.visit_unit()
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (c, idx, len) = stry!(self.next());
        // Parse the opening bracket of the sequence.
        if c == b'[' {
            // Give the visitor access to each element of the sequence.
            visitor.visit_seq(CommaSeparated::new(&mut self, len))
        } else {
            Err(self.error(idx, ErrorType::ExpectedArray))
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let r = self.deserialize_seq(visitor);
        // tuples have a known length damn you serde ...
        self.skip();
        r
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (c, idx, len) = stry!(self.next());
        // Parse the opening bracket of the sequence.
        if c == b'{' {
            // Give the visitor access to each element of the sequence.
            visitor.visit_map(CommaSeparated::new(&mut self, len))
        } else {
            Err(self.error(idx, ErrorType::ExpectedMap))
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
            i128 u128 char
            bytes byte_buf
            enum identifier ignored_any
    }
}

// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: u32,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn new(de: &'a mut Deserializer<'de>, len: u32) -> Self {
        CommaSeparated {
            first: true,
            de,
            len,
        }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.len == 0 {
            self.de.skip();
            Ok(None)
        } else {
            if !self.first {
                self.de.skip();
            } else {
                self.first = false;
            }
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(|e| Some(e))
        }
        /*
        let peek = match stry!(self.de.peek()) {
            (b']', _idx, _len) => {
                self.de.skip();
                return Ok(None);
            }
            (b',', _idx, _len) if !self.first => stry!(self.de.next()),
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(self.de.error(b.1, ErrorType::ExpectedArrayComma));
                }
            }
        };
        match peek {
            (b']', idx, _len) => Err(self.de.error(idx, ErrorType::ExpectedArrayComma)),
            _ => Ok(Some(stry!(seed.deserialize(&mut *self.de)))),
        }
        */
    }
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len as usize)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.len == 0 {
            if self.first {
                self.de.skip();
            }
            Ok(None)
        } else {
            self.len -= 1;
            self.first = false;
            seed.deserialize(&mut *self.de).map(Some)
        }
        /*
        let peek = match stry!(self.de.peek()) {
            (b'}', _idx, _len) => {
                self.de.skip();
                return Ok(None);
            }
            (b',', _idx, _len) if !self.first => {
                self.de.skip();
                stry!(self.de.peek())
            }
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(self.de.error(b.1, ErrorType::ExpectedMapComma));
                }
            }
        };

        match peek {
            (b'"', _idx, _len) => seed.deserialize(&mut *self.de).map(Some),
            (b'}', idx, _len) => Err(self.de.error(idx, ErrorType::ExpectedMapComma)),
            (_, idx, _len) => Err(self.de.error(idx, ErrorType::ExpectedString)),
        }
        */
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        //let (c, idx, _len) = stry!(self.de.next());
        //if c != b':' {
        //return Err(self.de.error(idx, ErrorType::ExpectedMapColon));
        //}
        // Skip the ':'
        self.de.skip();
        // read the value
        let r = seed.deserialize(&mut *self.de);
        // skip the ','or '}'
        self.de.skip();
        r
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.len as usize)
    }
}
