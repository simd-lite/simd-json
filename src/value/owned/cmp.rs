use super::{MaybeBorrowedString, Value};
use crate::Number;
use std::cmp::{Ordering, PartialOrd};

impl PartialOrd for MaybeBorrowedString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_s = match self {
            MaybeBorrowedString::O(s) => s.as_str(),
        };
        let other_s = match other {
            MaybeBorrowedString::O(s) => s.as_str(),
        };

        Some(self_s.cmp(other_s))
    }
}

impl PartialEq for MaybeBorrowedString {
    fn eq(&self, other: &Self) -> bool {
        let self_s = match self {
            MaybeBorrowedString::O(s) => s.as_str(),
        };
        let other_s = match other {
            MaybeBorrowedString::O(s) => s.as_str(),
        };

        self_s == other_s
    }
}

impl PartialEq<str> for MaybeBorrowedString {
    fn eq(&self, other: &str) -> bool {
        match self {
            MaybeBorrowedString::O(s) => s == &other,
        }
    }
}

impl PartialEq<String> for MaybeBorrowedString {
    fn eq(&self, other: &String) -> bool {
        match self {
            MaybeBorrowedString::O(s) => s == other,
        }
    }
}

impl PartialEq<()> for Value {
    fn eq(&self, _other: &()) -> bool {
        if let Value::Null = self {
            true
        } else {
            false
        }
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        if let Value::Bool(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        match self {
            Value::String(s) => s == other,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        match self {
            Value::String(s) => s == other,
            _ => false,
        }
    }
}

impl PartialEq<i8> for Value {
    fn eq(&self, other: &i8) -> bool {
        match self {
            Value::Number(Number::I64(i)) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i16> for Value {
    fn eq(&self, other: &i16) -> bool {
        match self {
            Value::Number(Number::I64(i)) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        match self {
            Value::Number(Number::I64(i)) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Value::Number(Number::I64(i)) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, other: &f32) -> bool {
        match self {
            Value::Number(Number::F64(f)) => f == &f64::from(*other),
            _ => false,
        }
    }
}
impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Value::Number(Number::F64(f)) => f == &f64::from(*other),
            _ => false,
        }
    }
}
