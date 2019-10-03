use std::fmt;

/// Error types encountered while parsing
#[derive(Debug, PartialEq)]
pub enum ErrorType {
    /// The key of a map isn't a string
    BadKeyType,
    /// The data ended early
    EarlyEnd,
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
    /// Expected an `:` to seperate key and value in an object
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
    /// Inbalid UTF8 codepoint
    InvalidUTF8,
    /// Invalid Unicode escape sequence
    InvalidUnicodeEscape,
    /// Inbalid Unicode codepoint
    InvlaidUnicodeCodepoint,
    /// Object Key isn't a string
    KeyMustBeAString,
    /// Non structural character
    NoStructure,
    /// Parser Erropr
    Parser,
    /// Early End Of File
    EOF,
    /// Generic serde error
    Serde(String),
    /// Generic syntax error
    Syntax,
    /// Training characters
    TrailingCharacters,
    /// Unexpected character
    UnexpectedCharacter,
    /// Unexpected end
    UnexpectedEnd,
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
}

/// Parser error
#[derive(Debug, PartialEq)]
pub struct Error {
    /// Strucutral the error was encountered at
    structural: usize,
    /// Byte index it was encountered at
    index: usize,
    /// Current character
    character: char,
    /// Tyep of error
    error: ErrorType,
}

impl Error {
    pub(crate) fn new(structural: usize, index: usize, character: char, error: ErrorType) -> Self {
        Self {
            structural,
            index,
            character,
            error,
        }
    }
    pub(crate) fn generic(t: ErrorType) -> Self {
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
