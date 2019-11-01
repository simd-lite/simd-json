/// simdjson-rs integrates with serde, this module holds this integration.
/// note that when parsing to a dom you should use the functions in
/// `to_owned_value` or `to_borrowed_value` as they provide much
/// better performance.
///
/// However if have to use serde for other readons or are psrsing
/// directly to structs this is th4 place to go.
///
mod de;
mod value;
pub use self::value::*;
use crate::numberparse::Number;
use crate::stage2::CharType;
use crate::{stry, Deserializer, Error, ErrorType, Result};
use crate::{BorrowedValue, OwnedValue};
use serde_ext::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::fmt;

type ConvertResult<T> = std::result::Result<T, SerdeConversionError>;

/// Error while converting from or to serde values
#[derive(Debug)]
pub enum SerdeConversionError {
    /// Serde can not reflect NAN or Infinity
    NanOrInfinity,
    /// Something horrible went wrong, please open a ticket at <https://simd-json.rs>
    Oops,
}
impl std::fmt::Display for SerdeConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SerdeConversionError::*;
        match self {
            NanOrInfinity => write!(f, "JSON can not represent NAN or Infinity values"),
            Oops => write!(
                f,
                "Unreachable code is reachable, oops - please open a bug with simdjson-rs"
            ),
        }
    }
}

impl std::error::Error for SerdeConversionError {}

/// parses a byte slice using a serde deserializer.
/// note that the slice will be rewritten in the process.
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
#[cfg_attr(not(feature = "no-inline"), inline(always))]
pub fn from_str<'a, T>(s: &'a mut str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = stry!(Deserializer::from_slice(unsafe { s.as_bytes_mut() }));

    T::deserialize(&mut deserializer)
}

impl std::error::Error for Error {}

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
    fn next(&mut self) -> Result<(CharType, u32)> {
        self.idx += 1;
        self.structural_indexes
            .get(self.idx)
            .copied()
            .ok_or_else(|| self.error(ErrorType::Syntax))
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn peek(&self) -> Result<CharType> {
        if let Some((c, _)) = self.structural_indexes.get(self.idx + 1) {
            Ok(*c)
        } else {
            Err(self.error(ErrorType::UnexpectedEnd))
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn parse_signed(&mut self) -> Result<i64> {
        match self.next_() {
            (CharType::NegNum, idx) => match stry!(self.parse_number(idx as usize, true)) {
                Number::I64(n) => Ok(n),
                _ => Err(self.error(ErrorType::ExpectedSigned)),
            },
            (CharType::PosNum, idx) => match stry!(self.parse_number(idx as usize, false)) {
                Number::I64(n) => Ok(n),
                Number::U64(n) => n
                    .try_into()
                    .map_err(|_| self.error(ErrorType::ExpectedSigned)),
                _ => Err(self.error(ErrorType::ExpectedSigned)),
            },
            _ => Err(self.error(ErrorType::ExpectedSigned)),
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_sign_loss)]
    fn parse_unsigned(&mut self) -> Result<u64> {
        match self.next_() {
            (CharType::PosNum, idx) => match stry!(self.parse_number(idx as usize, false)) {
                Number::I64(n) => Ok(n as u64),
                Number::U64(n) => Ok(n as u64),
                _ => Err(self.error(ErrorType::ExpectedUnsigned)),
            },
            _ => Err(self.error(ErrorType::ExpectedUnsigned)),
        }
    }
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    #[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]
    fn parse_double(&mut self) -> Result<f64> {
        match self.next_() {
            (CharType::NegNum, idx) => match stry!(self.parse_number(idx as usize, true)) {
                Number::F64(n) => Ok(n),
                Number::I64(n) => Ok(n as f64),
                Number::U64(n) => Ok(n as f64),
            },
            (CharType::PosNum, idx) => match stry!(self.parse_number(idx as usize, false)) {
                Number::F64(n) => Ok(n),
                Number::I64(n) => Ok(n as f64),
                Number::U64(n) => Ok(n as f64),
            },
            _ => Err(self.error(ErrorType::ExpectedFloat)),
        }
    }
}

impl TryFrom<serde_json::Value> for OwnedValue {
    type Error = SerdeConversionError;
    fn try_from(item: serde_json::Value) -> ConvertResult<Self> {
        use serde_json::Value;
        Ok(match item {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(b),
            Value::Number(b) => {
                if let Some(n) = b.as_i64() {
                    Self::I64(n)
                } else if let Some(n) = b.as_u64() {
                    Self::U64(n)
                } else if let Some(n) = b.as_f64() {
                    Self::F64(n)
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
            Self::Null => Value::Null,
            Self::Bool(b) => Value::Bool(b),
            Self::I64(n) => Value::Number(n.into()),
            Self::U64(n) => Value::Number(n.into()),
            Self::F64(n) => {
                if let Some(n) = serde_json::Number::from_f64(n) {
                    Value::Number(n)
                } else {
                    return Err(SerdeConversionError::NanOrInfinity);
                }
            }
            Self::String(b) => Value::String(b.to_string()),
            Self::Array(a) => Value::Array(
                a.into_iter()
                    .map(|v| v.try_into())
                    .collect::<ConvertResult<Vec<Value>>>()?,
            ),
            Self::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| Ok((k.to_string(), v.try_into()?)))
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
            Value::Null => Ok(BorrowedValue::Null),
            Value::Bool(b) => Ok(BorrowedValue::from(b)),
            Value::Number(b) => {
                if let Some(n) = b.as_i64() {
                    Ok(Self::from(n))
                } else if let Some(n) = b.as_u64() {
                    Ok(Self::from(n))
                } else if let Some(n) = b.as_f64() {
                    Ok(Self::from(n))
                } else {
                    Err(SerdeConversionError::Oops)
                }
            }
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
            BorrowedValue::Null => Value::Null,
            BorrowedValue::Bool(b) => Value::Bool(b),
            BorrowedValue::I64(n) => Value::Number(n.into()),
            BorrowedValue::U64(n) => Value::Number(n.into()),
            BorrowedValue::F64(n) => {
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
    #![allow(clippy::result_unwrap_used)]
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
}
