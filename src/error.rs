use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    BadKeyType,
    EarlyEnd,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedBoolean,
    ExpectedEnum,
    ExpectedFloat,
    ExpectedInteger,
    ExpectedMap,
    ExpectedObjectColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedNull,
    ExpectedNumber,
    ExpectedSigned,
    ExpectedString,
    ExpectedUnsigned,
    InternalError,
    InvalidEscape,
    InvalidExponent,
    InvalidNumber,
    InvalidUTF8,
    InvalidUnicodeEscape,
    InvlaidUnicodeCodepoint,
    KeyMustBeAString,
    NoStructure,
    Parser,
    EOF,
    Serde(String),
    Syntax,
    TrailingCharacters,
    UnexpectedCharacter,
    UnexpectedEnd,
    UnterminatedString,
    ExpectedArrayContent,
    ExpectedObjectContent,
    ExpectedObjectKey,
    Overflow,
}

#[derive(Debug, PartialEq)]
pub struct Error {
    structural: usize,
    index: usize,
    character: char,
    error: ErrorType,
}

impl Error {
    pub fn new(structural: usize, index: usize, character: char, error: ErrorType) -> Self {
        Self {
            structural,
            index,
            character,
            error,
        }
    }
    pub fn generic(t: ErrorType) -> Self {
        Self {
            structural: 0,
            index: 0,
            character: '💩', //this is the poop emoji
            error: t,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} at chracter {} ('{}')",
            self.error, self.index, self.character
        )
    }
}
