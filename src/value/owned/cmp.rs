use super::Value;

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

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
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
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i16> for Value {
    fn eq(&self, other: &i16) -> bool {
        match self {
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        match self {
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, other: &f32) -> bool {
        match self {
            Value::F64(f) => f == &f64::from(*other),
            _ => false,
        }
    }
}
impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Value::F64(f) => f == &f64::from(*other),
            _ => false,
        }
    }
}
