use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

#[derive(Clone)]
pub enum MaybeBorrowedString {
    O(String),
}

impl Borrow<str> for MaybeBorrowedString {
    fn borrow(&self) -> &str {
        match self {
            MaybeBorrowedString::O(s) => &s,
        }
    }
}

impl Deref for MaybeBorrowedString {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            MaybeBorrowedString::O(s) => &s,
        }
    }
}

impl fmt::Display for MaybeBorrowedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::O(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for MaybeBorrowedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeBorrowedString::O(s) => write!(f, "{:?}", s),
        }
    }
}
