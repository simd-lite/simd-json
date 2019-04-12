use serde::ser;
use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Clone)]
pub enum MaybeBorrowedString {
    O(String),
}

impl Borrow<str> for MaybeBorrowedString {
    #[inline]
    fn borrow(&self) -> &str {
        match self {
            MaybeBorrowedString::O(s) => &s,
        }
    }
}

impl Borrow<String> for MaybeBorrowedString {
    #[inline]
    fn borrow(&self) -> &String {
        match self {
            MaybeBorrowedString::O(s) => &s,
        }
    }
}

impl Deref for MaybeBorrowedString {
    type Target = str;
    #[inline]
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

impl ser::Serialize for MaybeBorrowedString {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            MaybeBorrowedString::O(s) => serializer.serialize_str(&s),
        }
    }
}

impl Hash for MaybeBorrowedString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s: &str = self.borrow();
        s.hash(state)
    }
}
