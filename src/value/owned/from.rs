use super::{MaybeBorrowedString, Value};

impl From<&str> for MaybeBorrowedString {
    fn from(v: &str) -> Self {
        MaybeBorrowedString::O(v.to_owned())
    }
}

impl From<String> for MaybeBorrowedString {
    fn from(v: String) -> Self {
        MaybeBorrowedString::O(v)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(MaybeBorrowedString::O(s.to_owned()))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(MaybeBorrowedString::O(s))
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

/********* i_ **********/
impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Value::I64(i as i64)
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Value::I64(i as i64)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::I64(i as i64)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::I64(i as i64)
    }
}

impl From<&i8> for Value {
    fn from(i: &i8) -> Self {
        Value::I64(*i as i64)
    }
}

impl From<&i16> for Value {
    fn from(i: &i16) -> Self {
        Value::I64(*i as i64)
    }
}

impl From<&i32> for Value {
    fn from(i: &i32) -> Self {
        Value::I64(*i as i64)
    }
}

impl From<&i64> for Value {
    fn from(i: &i64) -> Self {
        Value::I64(*i as i64)
    }
}

/********* f_ **********/
impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::F64(f as f64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::F64(f as f64)
    }
}

impl From<&f32> for Value {
    fn from(f: &f32) -> Self {
        Value::F64(*f as f64)
    }
}

impl From<&f64> for Value {
    fn from(f: &f64) -> Self {
        Value::F64(*f as f64)
    }
}
