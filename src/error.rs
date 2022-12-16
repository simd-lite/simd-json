use std::fmt;

use value_trait::ValueType;

/// Error types encountered while parsing
#[derive(Debug)]
pub enum ErrorType {
    /// A specific type was expected but another one encountered.
    Unexpected(Option<ValueType>, Option<ValueType>),
    /// Simd-json only supports inputs of up to
    /// 4GB in size.
    InputTooLarge,
    /// The key of a map isn't a string
    BadKeyType,
    /// Expected an array
    ExpectedArray,
    /// Expected a `,` in an array
    ExpectedArrayComma,
    /// expected an boolean
    ExpectedBoolean,
    /// Expected an enum
    ExpectedEnum,
    /// Expected a float
    ExpectedFloat,
    /// Expected an integer
    ExpectedInteger,
    /// Expected a map
    ExpectedMap,
    /// Expected an `:` to separate key and value in an object
    ExpectedObjectColon,
    /// Expected a `,` in an object
    ExpectedMapComma,
    /// Expected the object to end
    ExpectedMapEnd,
    /// Expected a null
    ExpectedNull,
    /// Expected a number
    ExpectedNumber,
    /// Expected a signed number
    ExpectedSigned,
    /// Expected a string
    ExpectedString,
    /// Expected an unsigned number
    ExpectedUnsigned,
    /// Internal error
    InternalError,
    /// Invalid escape sequence
    InvalidEscape,
    /// Invalid exponent in a floating point number
    InvalidExponent,
    /// Invalid number
    InvalidNumber,
    /// Invalid UTF8 codepoint
    InvalidUtf8,
    /// Invalid Unicode escape sequence
    InvalidUnicodeEscape,
    /// Invalid Unicode codepoint
    InvalidUnicodeCodepoint,
    /// Object Key isn't a string
    KeyMustBeAString,
    /// Non structural character
    NoStructure,
    /// Parser Error
    Parser,
    /// Early End Of File
    Eof,
    /// Generic serde error
    Serde(String),
    /// Generic syntax error
    Syntax,
    /// Trailing data
    TrailingData,
    /// Unexpected character
    UnexpectedCharacter,
    /// Unterminated string
    UnterminatedString,
    /// Expected Array elements
    ExpectedArrayContent,
    /// Expected Object elements
    ExpectedObjectContent,
    /// Expected Object Key
    ExpectedObjectKey,
    /// Overflow of a limited buffer
    Overflow,
    /// IO error
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::generic(ErrorType::Io(e))
    }
}

#[cfg(not(tarpaulin_include))]
impl PartialEq for ErrorType {
    #[must_use]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(_), Self::Io(_))
            | (Self::BadKeyType, Self::BadKeyType)
            | (Self::ExpectedArray, Self::ExpectedArray)
            | (Self::ExpectedArrayComma, Self::ExpectedArrayComma)
            | (Self::ExpectedBoolean, Self::ExpectedBoolean)
            | (Self::ExpectedEnum, Self::ExpectedEnum)
            | (Self::ExpectedFloat, Self::ExpectedFloat)
            | (Self::ExpectedInteger, Self::ExpectedInteger)
            | (Self::ExpectedMap, Self::ExpectedMap)
            | (Self::ExpectedObjectColon, Self::ExpectedObjectColon)
            | (Self::ExpectedMapComma, Self::ExpectedMapComma)
            | (Self::ExpectedMapEnd, Self::ExpectedMapEnd)
            | (Self::ExpectedNull, Self::ExpectedNull)
            | (Self::ExpectedNumber, Self::ExpectedNumber)
            | (Self::ExpectedSigned, Self::ExpectedSigned)
            | (Self::ExpectedString, Self::ExpectedString)
            | (Self::ExpectedUnsigned, Self::ExpectedUnsigned)
            | (Self::InternalError, Self::InternalError)
            | (Self::InvalidEscape, Self::InvalidEscape)
            | (Self::InvalidExponent, Self::InvalidExponent)
            | (Self::InvalidNumber, Self::InvalidNumber)
            | (Self::InvalidUtf8, Self::InvalidUtf8)
            | (Self::InvalidUnicodeEscape, Self::InvalidUnicodeEscape)
            | (Self::InvalidUnicodeCodepoint, Self::InvalidUnicodeCodepoint)
            | (Self::KeyMustBeAString, Self::KeyMustBeAString)
            | (Self::NoStructure, Self::NoStructure)
            | (Self::Parser, Self::Parser)
            | (Self::Eof, Self::Eof)
            | (Self::Syntax, Self::Syntax)
            | (Self::TrailingData, Self::TrailingData)
            | (Self::UnexpectedCharacter, Self::UnexpectedCharacter)
            | (Self::UnterminatedString, Self::UnterminatedString)
            | (Self::ExpectedArrayContent, Self::ExpectedArrayContent)
            | (Self::ExpectedObjectContent, Self::ExpectedObjectContent)
            | (Self::ExpectedObjectKey, Self::ExpectedObjectKey)
            | (Self::Overflow, Self::Overflow) => true,
            (Self::Serde(s1), Self::Serde(s2)) => s1 == s2,
            _ => false,
        }
    }
}
/// Parser error
#[derive(Debug, PartialEq)]
pub struct Error {
    /// Byte index it was encountered at
    index: usize,
    /// Current character
    character: Option<char>,
    /// Type of error
    error: ErrorType,
}

impl Error {
    pub(crate) fn new(index: usize, character: Option<char>, error: ErrorType) -> Self {
        Self {
            index,
            character,
            error,
        }
    }
    pub(crate) fn new_c(index: usize, character: char, error: ErrorType) -> Self {
        Self::new(index, Some(character), error)
    }

    /// Create a generic error
    #[must_use = "Error creation"]
    pub fn generic(t: ErrorType) -> Self {
        Self {
            index: 0,
            character: None,
            error: t,
        }
    }
}
impl std::error::Error for Error {}

#[cfg(not(tarpaulin_include))]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = self.character {
            write!(f, "{:?} at character {} ('{c}')", self.error, self.index)
        } else {
            write!(f, "{:?} at character {}", self.error, self.index)
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    }
}

#[cfg(test)]
mod test {
    use super::{Error, ErrorType};
    #[test]
    fn fmt() {
        let e = Error::generic(ErrorType::InternalError);
        assert_eq!(e.to_string(), "InternalError at character 0");
    }
}
