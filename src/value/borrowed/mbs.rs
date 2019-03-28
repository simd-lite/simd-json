use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

#[derive(Clone)]
pub enum MaybeBorrowedString<'a> {
    B(&'a str),
    O(String),
}

impl<'a> Borrow<str> for MaybeBorrowedString<'a> {
    fn borrow(&self) -> &str {
        match self {
            MaybeBorrowedString::B(s) => s,
            MaybeBorrowedString::O(s) => &s,
        }
    }
}

impl<'a> Deref for MaybeBorrowedString<'a> {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            MaybeBorrowedString::B(s) => s,
            MaybeBorrowedString::O(s) => &s,
        }
    }
}

impl<'a> fmt::Display for MaybeBorrowedString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::B(s) => write!(f, "{}", s),
            MaybeBorrowedString::O(s) => write!(f, "{}", s),
        }
    }
}

impl<'a> fmt::Debug for MaybeBorrowedString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::B(s) => write!(f, "{:?}", s),
            MaybeBorrowedString::O(s) => write!(f, "{:?}", s),
        }
    }
}
