use serde::ser;
use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

//#[derive(Clone)]
pub enum MaybeBorrowedString<'a> {
    B(&'a str),
    O(String),
}

impl<'a> Clone for MaybeBorrowedString<'a> {
    fn clone(&self) -> Self {
        match self {
            MaybeBorrowedString::B(s) => MaybeBorrowedString::O(s.to_owned()),
            MaybeBorrowedString::O(s) => MaybeBorrowedString::O(s.clone()),
        }
    }
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

impl<'a> ser::Serialize for MaybeBorrowedString<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            MaybeBorrowedString::O(s) => serializer.serialize_str(&s),
            MaybeBorrowedString::B(s) => serializer.serialize_str(s),
        }
    }
}

impl<'a> Hash for MaybeBorrowedString<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s: &str = self.borrow();
        s.hash(state)
    }
}
