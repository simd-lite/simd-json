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
            b'n' => {
                stry!(self.parse_null());
                visitor.visit_unit()
            }
            b't' => visitor.visit_bool(stry!(self.parse_true())),
            b'f' => visitor.visit_bool(stry!(self.parse_false())),
            b'-' => match stry!(self.parse_number(true)) {
                Number::F64(n) => visitor.visit_f64(n),
                Number::I64(n) => visitor.visit_i64(n),
            },
            b'0'...b'9' => match stry!(self.parse_number(false)) {
                Number::F64(n) => visitor.visit_f64(n),
                Number::I64(n) => visitor.visit_i64(n),
            },
            b'"' => {
                // We don't do the short string optimisation as serde requires
                // additional checks
                visitor.visit_borrowed_str(stry!(self.parse_str_()))
            }

            b'[' => visitor.visit_seq(CommaSeparated::new(&mut self)),
            b'{' => visitor.visit_map(CommaSeparated::new(&mut self)),
            _c => Err(self.error(ErrorType::UnexpectedCharacter)),
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
            b't' => visitor.visit_bool(stry!(self.parse_true())),
            b'f' => visitor.visit_bool(stry!(self.parse_false())),
            _c => Err(self.error(ErrorType::ExpectedBoolean)),
        }
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if stry!(self.next()) != b'"' {
            return Err(self.error(ErrorType::ExpectedString));
        }
        if let Some(next) = self.structural_indexes.get(self.idx + 1) {
            if *next as usize - self.iidx < 32 {
                return visitor.visit_borrowed_str(stry!(self.parse_short_str_()));
            }
        }
        visitor.visit_borrowed_str(stry!(self.parse_str_()))
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if stry!(self.next()) != b'"' {
            return Err(self.error(ErrorType::ExpectedString));
        }
        if let Some(next) = self.structural_indexes.get(self.idx + 1) {
            if *next as usize - self.iidx < 32 {
                return visitor.visit_str(stry!(self.parse_short_str_()));
            }
        }
        visitor.visit_str(stry!(self.parse_str_()))
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
        if stry!(self.peek()) == b'n' {
            self.skip();
            stry!(self.parse_null());
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
        if stry!(self.next()) != b'n' {
            return Err(self.error(ErrorType::ExpectedNull));
        }
        stry!(self.parse_null());
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
        // Parse the opening bracket of the sequence.
        if stry!(self.next()) == b'[' {
            // Give the visitor access to each element of the sequence.
            visitor.visit_seq(CommaSeparated::new(&mut self))
        } else {
            Err(self.error(ErrorType::ExpectedArray))
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
        // Parse the opening bracket of the sequence.
        if stry!(self.next()) == b'{' {
            // Give the visitor access to each element of the sequence.
            visitor.visit_map(CommaSeparated::new(&mut self))
        } else {
            Err(self.error(ErrorType::ExpectedMap))
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
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated { first: true, de }
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
        let peek = match stry!(self.de.peek()) {
            b']' => {
                self.de.skip();
                return Ok(None);
            }
            b',' if !self.first => stry!(self.de.next()),
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(self.de.error(ErrorType::ExpectedArrayComma));
                }
            }
        };
        match peek {
            b']' => Err(self.de.error(ErrorType::ExpectedArrayComma)),
            _ => Ok(Some(stry!(seed.deserialize(&mut *self.de)))),
        }
    }
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.de.count_elements())
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
        let peek = match stry!(self.de.peek()) {
            b'}' => {
                self.de.skip();
                return Ok(None);
            }
            b',' if !self.first => {
                self.de.skip();
                stry!(self.de.peek())
            }
            b => {
                if self.first {
                    self.first = false;
                    b
                } else {
                    return Err(self.de.error(ErrorType::ExpectedArrayComma));
                }
            }
        };

        match peek {
            b'"' => seed.deserialize(&mut *self.de).map(Some),
            b'}' => Err(self.de.error(ErrorType::ExpectedArrayComma)), //Err(self.de.peek_error(ErrorCode::TrailingComma)),
            _ => Err(self.de.error(ErrorType::ExpectedString)), // TODO: Err(self.de.peek_error(ErrorCode::KeyMustBeAString)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let c = stry!(self.de.next());
        if c != b':' {
            return Err(self.de.error(ErrorType::ExpectedMapColon));
        }
        seed.deserialize(&mut *self.de)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn size_hint(&self) -> Option<usize> {
        Some(self.de.count_elements())
    }
}
