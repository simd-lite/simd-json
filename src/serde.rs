/// simd-json integrates with serde, this module holds this integration.
/// note that when parsing to a dom you should use the functions in
/// `to_owned_value` or `to_borrowed_value` as they provide much
/// better performance.
///
/// However if have to use serde for other reasons or are parsing
/// directly to structs this is th4 place to go.
///
mod de;
mod se;
mod value;
pub use self::se::*;
pub use self::value::*;
use crate::{stry, Deserializer, Error, ErrorType, Result};
use crate::{BorrowedValue, OwnedValue};
use crate::{Node, StaticNode};
use serde::de::DeserializeOwned;
use serde_ext::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::io;
use value_trait::prelude::*;
type ConvertResult<T> = std::result::Result<T, SerdeConversionError>;

/// Error while converting from or to serde values
#[derive(Debug)]
pub enum SerdeConversionError {
    /// Serde can not reflect NAN or Infinity
    NanOrInfinity,
    /// The number is out of the 64 bit bound
    NumberOutOfBounds,
    /// Something horrible went wrong, please open a ticket at <https://simd-json.rs>
    Oops,
}
impl std::fmt::Display for SerdeConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SerdeConversionError::{NanOrInfinity, NumberOutOfBounds, Oops};
        match self {
            NanOrInfinity => write!(f, "JSON can not represent NAN or Infinity values"),
            NumberOutOfBounds => write!(f, "Serde can not represent 128 bit values"),
            Oops => write!(
                f,
                "Unreachable code is reachable, oops - please open a bug with simd-json"
            ),
        }
    }
}

impl std::error::Error for SerdeConversionError {}

/// parses a byte slice using a serde deserializer.
/// note that the slice will be rewritten in the process.
///
/// # Errors
///
/// Will return `Err` if `s` is invalid JSON.
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn from_slice<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(s));
    T::deserialize(&mut deserializer)
}
/// parses a str  using a serde deserializer.
/// note that the slice will be rewritten in the process and
/// might not remain a valid utf8 string in its entirety.
///
/// # Errors
///
/// Will return `Err` if `s` is invalid JSON.
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn from_str<'a, T>(s: &'a mut str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(unsafe { s.as_bytes_mut() }));

    T::deserialize(&mut deserializer)
}

/// parses a Reader using a serde deserializer.
///
/// # Errors
///
/// Will return `Err` if an IO error is encountred while reading
/// rdr or if the readers content is invalid JSON.
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn from_reader<R, T>(mut rdr: R) -> Result<T>
where
    R: io::Read,
    T: DeserializeOwned,
{
    let mut data = Vec::new();
    if let Err(e) = rdr.read_to_end(&mut data) {
        return Err(Error::generic(ErrorType::Io(e)));
    };
    let mut deserializer = stry!(Deserializer::from_slice(&mut data));
    T::deserialize(&mut deserializer)
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::generic(ErrorType::Serde(msg.to_string()))
    }
}

impl serde_ext::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::generic(ErrorType::Serde(msg.to_string()))
    }
}

// Functions purely used by serde
impl<'de> Deserializer<'de> {
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn next(&mut self) -> Result<Node<'de>> {
        self.idx += 1;
        self.tape
            .get(self.idx)
            .copied()
            .ok_or_else(|| Self::error(ErrorType::Syntax))
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn peek(&self) -> Result<Node> {
        self.tape
            .get(self.idx + 1)
            .copied()
            .ok_or_else(|| Self::error(ErrorType::UnexpectedEnd))
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_u8(&mut self) -> Result<u8> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_u8()
                .ok_or_else(|| Self::error(ErrorType::ExpectedUnsigned)),
            _ => Err(Self::error(ErrorType::ExpectedUnsigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_u16(&mut self) -> Result<u16> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_u16()
                .ok_or_else(|| Self::error(ErrorType::ExpectedUnsigned)),
            _ => Err(Self::error(ErrorType::ExpectedUnsigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_u32(&mut self) -> Result<u32> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_u32()
                .ok_or_else(|| Self::error(ErrorType::ExpectedUnsigned)),
            _ => Err(Self::error(ErrorType::ExpectedUnsigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_u64(&mut self) -> Result<u64> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_u64()
                .ok_or_else(|| Self::error(ErrorType::ExpectedUnsigned)),
            _ => Err(Self::error(ErrorType::ExpectedUnsigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_u128(&mut self) -> Result<u128> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_u128()
                .ok_or_else(|| Self::error(ErrorType::ExpectedUnsigned)),
            _ => Err(Self::error(ErrorType::ExpectedUnsigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_i8(&mut self) -> Result<i8> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_i8()
                .ok_or_else(|| Self::error(ErrorType::ExpectedSigned)),
            _ => Err(Self::error(ErrorType::ExpectedSigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_i16(&mut self) -> Result<i16> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_i16()
                .ok_or_else(|| Self::error(ErrorType::ExpectedSigned)),
            _ => Err(Self::error(ErrorType::ExpectedSigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_i32(&mut self) -> Result<i32> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_i32()
                .ok_or_else(|| Self::error(ErrorType::ExpectedSigned)),
            _ => Err(Self::error(ErrorType::ExpectedSigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_i64(&mut self) -> Result<i64> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_i64()
                .ok_or_else(|| Self::error(ErrorType::ExpectedSigned)),
            _ => Err(Self::error(ErrorType::ExpectedSigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_i128(&mut self) -> Result<i128> {
        match unsafe { self.next_() } {
            Node::Static(s) => s
                .as_i128()
                .ok_or_else(|| Self::error(ErrorType::ExpectedSigned)),
            _ => Err(Self::error(ErrorType::ExpectedSigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]
    fn parse_double(&mut self) -> Result<f64> {
        match unsafe { self.next_() } {
            Node::Static(StaticNode::F64(n)) => Ok(n),
            Node::Static(StaticNode::I64(n)) => Ok(n as f64),
            Node::Static(StaticNode::U64(n)) => Ok(n as f64),
            _ => Err(Self::error(ErrorType::ExpectedFloat)),
        }
    }
}

impl TryFrom<serde_json::Value> for OwnedValue {
    type Error = SerdeConversionError;
    fn try_from(item: serde_json::Value) -> ConvertResult<Self> {
        use serde_json::Value;
        Ok(match item {
            Value::Null => Self::Static(StaticNode::Null),
            Value::Bool(b) => Self::Static(StaticNode::Bool(b)),
            Value::Number(b) => {
                if let Some(n) = b.as_i64() {
                    Self::Static(StaticNode::I64(n))
                } else if let Some(n) = b.as_u64() {
                    Self::Static(StaticNode::U64(n))
                } else if let Some(n) = b.as_f64() {
                    Self::Static(StaticNode::F64(n))
                } else {
                    return Err(SerdeConversionError::Oops);
                }
            }
            Value::String(b) => Self::String(b),
            Value::Array(a) => a
                .into_iter()
                .map(Self::try_from)
                .collect::<ConvertResult<Self>>()?,
            Value::Object(o) => o
                .into_iter()
                .map(|(k, v)| Ok((k, Self::try_from(v)?)))
                .collect::<ConvertResult<Self>>()?,
        })
    }
}

impl TryInto<serde_json::Value> for OwnedValue {
    type Error = SerdeConversionError;
    fn try_into(self) -> ConvertResult<serde_json::Value> {
        use serde_json::Value;
        Ok(match self {
            Self::Static(StaticNode::Null) => Value::Null,
            Self::Static(StaticNode::Bool(b)) => Value::Bool(b),
            Self::Static(StaticNode::I64(n)) => Value::Number(n.into()),
            #[cfg(feature = "128bit")] // FIXME error for too large numbers
            Self::Static(StaticNode::I128(n)) => Value::Number(
                i64::try_from(n)
                    .map_err(|_| SerdeConversionError::NumberOutOfBounds)?
                    .into(),
            ),
            Self::Static(StaticNode::U64(n)) => Value::Number(n.into()),
            #[cfg(feature = "128bit")] // FIXME error for too large numbers
            Self::Static(StaticNode::U128(n)) => Value::Number(
                u64::try_from(n)
                    .map_err(|_| SerdeConversionError::NumberOutOfBounds)?
                    .into(),
            ),
            Self::Static(StaticNode::F64(n)) => {
                if let Some(n) = serde_json::Number::from_f64(n) {
                    Value::Number(n)
                } else {
                    return Err(SerdeConversionError::NanOrInfinity);
                }
            }
            Self::String(b) => Value::String(b),
            Self::Array(a) => Value::Array(
                a.into_iter()
                    .map(|v| v.try_into())
                    .collect::<ConvertResult<Vec<Value>>>()?,
            ),
            Self::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| Ok((k, v.try_into()?)))
                    .collect::<ConvertResult<serde_json::map::Map<String, Value>>>()?,
            ),
        })
    }
}

impl<'value> TryFrom<serde_json::Value> for BorrowedValue<'value> {
    type Error = SerdeConversionError;
    fn try_from(item: serde_json::Value) -> ConvertResult<Self> {
        use serde_json::Value;
        match item {
            Value::Null => Ok(BorrowedValue::from(())),
            Value::Bool(b) => Ok(BorrowedValue::from(b)),
            Value::Number(b) => match (b.as_i64(), b.as_u64(), b.as_f64()) {
                (Some(n), _, _) => Ok(Self::from(n)),
                (_, Some(n), _) => Ok(Self::from(n)),
                (_, _, Some(n)) => Ok(Self::from(n)),
                _ => Err(SerdeConversionError::Oops),
            },
            Value::String(b) => Ok(Self::String(b.into())),
            Value::Array(a) => a.into_iter().map(Self::try_from).collect(),
            Value::Object(o) => o
                .into_iter()
                .map(|(k, v)| Ok((k, Self::try_from(v)?)))
                .collect(),
        }
    }
}

impl<'value> TryInto<serde_json::Value> for BorrowedValue<'value> {
    type Error = SerdeConversionError;
    fn try_into(self) -> ConvertResult<serde_json::Value> {
        use serde_json::Value;
        Ok(match self {
            BorrowedValue::Static(StaticNode::Null) => Value::Null,
            BorrowedValue::Static(StaticNode::Bool(b)) => Value::Bool(b),
            BorrowedValue::Static(StaticNode::I64(n)) => Value::Number(n.into()),
            #[cfg(feature = "128bit")] // FIXME error for too large numbers
            BorrowedValue::Static(StaticNode::I128(n)) => Value::Number(
                i64::try_from(n)
                    .map_err(|_| SerdeConversionError::NumberOutOfBounds)?
                    .into(),
            ),
            BorrowedValue::Static(StaticNode::U64(n)) => Value::Number(n.into()),
            #[cfg(feature = "128bit")] // FIXME error for too large numbers
            BorrowedValue::Static(StaticNode::U128(n)) => Value::Number(
                u64::try_from(n)
                    .map_err(|_| SerdeConversionError::NumberOutOfBounds)?
                    .into(),
            ),
            BorrowedValue::Static(StaticNode::F64(n)) => {
                if let Some(n) = serde_json::Number::from_f64(n) {
                    Value::Number(n)
                } else {
                    return Err(SerdeConversionError::NanOrInfinity);
                }
            }
            BorrowedValue::String(b) => Value::String(b.to_string()),
            BorrowedValue::Array(a) => Value::Array(
                a.into_iter()
                    .map(|v| v.try_into())
                    .collect::<ConvertResult<Vec<Value>>>()?,
            ),
            BorrowedValue::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| Ok((k.to_string(), v.try_into()?)))
                    .collect::<ConvertResult<serde_json::map::Map<String, Value>>>()?,
            ),
        })
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use crate::{json, BorrowedValue, OwnedValue};
    use serde_json::{json as sjson, Value as SerdeValue};
    use std::convert::TryInto;
    #[test]
    fn convert_owned_value() {
        let v: OwnedValue = json!({
            "int": 42,
            "float": 7.2,
            "neg-int": -23,
            "string": "string",
            "bool": true,
            "null": null,
            "object": {
            "array": [42, 7, -23, false, null, {"key": "value"}],
            }
        });

        let s: SerdeValue = sjson!({
            "int": 42,
            "float": 7.2,
            "neg-int": -23,
            "string": "string",
            "bool": true,
            "null": null,
            "object": {
            "array": [42, 7, -23, false, null, {"key": "value"}],
            }
        });
        let s_c: SerdeValue = v.clone().try_into().unwrap();
        assert_eq!(s, s_c);
        let v_c: OwnedValue = s.try_into().unwrap();
        assert_eq!(v, v_c);
    }

    #[test]
    fn convert_borrowed_value() {
        let v: BorrowedValue = json!({
            "int": 42,
            "float": 7.2,
            "neg-int": -23,
            "string": "string",
            "bool": true,
            "null": null,
            "object": {
            "array": [42, 7, -23, false, null, {"key": "value"}],
            }
        })
        .into();

        let s: SerdeValue = sjson!({
            "int": 42,
            "float": 7.2,
            "neg-int": -23,
            "string": "string",
            "bool": true,
            "null": null,
            "object": {
            "array": [42, 7, -23, false, null, {"key": "value"}],
            }
        });
        let s_c: SerdeValue = v.clone().try_into().unwrap();
        assert_eq!(s, s_c);
        let v_c: BorrowedValue = s.try_into().unwrap();
        assert_eq!(v, v_c);
    }

    #[test]
    fn option_field_absent() {
        #[derive(serde::Deserialize, Debug)]
        pub struct Person {
            pub name: String,
            pub middle_name: Option<String>,
            pub friends: Vec<String>,
        }
        let mut raw_json = r#"{"name":"bob","friends":[]}"#.to_string();
        let result: Result<Person, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());
    }
    #[test]
    fn option_field_present() {
        #[derive(serde::Deserialize, Debug)]
        pub struct Person {
            pub name: String,
            pub middle_name: Option<String>,
            pub friends: Vec<String>,
        }
        let mut raw_json = r#"{"name":"bob","middle_name": "frank", "friends":[]}"#.to_string();
        let result: Result<Person, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());
    }

    #[test]
    fn convert_enum() {
        #[derive(serde::Deserialize, Debug)]
        #[serde(tag = "type")]
        enum Message {
            Request { id: usize, method: String },
            Response { id: String, result: String },
        }
        let mut raw_json = r#"{"type": "Request", "id": 1, "method": "..."}"#.to_string();
        let result: Result<Message, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());

        let mut raw_json = r#"{"type": "Response", "id": "1", "result": "..."}"#.to_string();
        let result: Result<Message, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());

        #[derive(serde::Deserialize, Debug)]
        #[serde(tag = "type", content = "v")]
        pub enum Color {
            Red(String), // TODO: If `content` flag is present, `Red` works and `Green` doesn't
            Green { v: bool },
            Blue,
        }
        let mut raw_json = r#"{"type": "Red", "v": "1"}"#.to_string();
        let result: Result<Color, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());

        let mut raw_json = r#"{"type": "Blue"}"#.to_string();
        let result: Result<Color, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());

        #[derive(serde::Deserialize, Debug)]
        #[serde(tag = "type")]
        pub enum Color1 {
            Red(String),
            Green { v: bool }, // TODO: If `content` flag is absent, `Green` works and `Red` doesn't
            Blue,
        }
        let mut raw_json = r#"{"type": "Green", "v": false}"#.to_string();
        let result: Result<Color1, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());

        let mut raw_json = r#"{"type": "Blue"}"#.to_string();
        let result: Result<Color1, _> = super::from_slice(unsafe { raw_json.as_bytes_mut() });
        assert!(result.is_ok());
    }

    #[derive(serde_ext::Deserialize)]
    pub struct Foo {
        #[allow(unused)]
        bar: Bar,
    }

    #[derive(serde_ext::Deserialize)]
    pub enum Bar {
        A,
    }

    #[test]
    fn object_simd_json() {
        let mut json = br#"{"bar":"A"}"#.to_vec();

        crate::from_slice::<Foo>(&mut json).unwrap();
    }

    #[test]
    fn simple_simd_json() {
        let mut json = br#""A""#.to_vec();

        assert!(crate::from_slice::<Bar>(&mut json).is_ok());
    }

    #[test]
    fn array_as_struct() {
        #[derive(serde_ext::Deserialize)]
        struct Point {
            x: u64,
            y: u64,
        }

        let mut json = br#"[1,2]"#.to_vec();

        let p: Point = serde_json::from_slice(&json).unwrap();
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);

        let p: Point = crate::from_slice(&mut json).unwrap();
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
    }
}
