use super::Value;

impl<'a> PartialEq<()> for Value<'a> {
    fn eq(&self, _other: &()) -> bool {
        if let Value::Null = self {
            true
        } else {
            false
        }
    }
}

impl<'a> PartialEq<bool> for Value<'a> {
    fn eq(&self, other: &bool) -> bool {
        if let Value::Bool(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl<'a> PartialEq<str> for Value<'a> {
    fn eq(&self, other: &str) -> bool {
        match self {
            Value::String(s) => s == other,
            _ => false,
        }
    }
}

impl<'a> PartialEq<&str> for Value<'a> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Value::String(s) => s == other,
            _ => false,
        }
    }
}

impl<'a> PartialEq<String> for Value<'a> {
    fn eq(&self, other: &String) -> bool {
        match self {
            Value::String(s) => s == other,
            _ => false,
        }
    }
}

impl<'a> PartialEq<i8> for Value<'a> {
    fn eq(&self, other: &i8) -> bool {
        match self {
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl<'a> PartialEq<i16> for Value<'a> {
    fn eq(&self, other: &i16) -> bool {
        match self {
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl<'a> PartialEq<i32> for Value<'a> {
    fn eq(&self, other: &i32) -> bool {
        match self {
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl<'a> PartialEq<i64> for Value<'a> {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Value::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl<'a> PartialEq<f32> for Value<'a> {
    fn eq(&self, other: &f32) -> bool {
        match self {
            Value::F64(f) => f == &f64::from(*other),
            _ => false,
        }
    }
}
impl<'a> PartialEq<f64> for Value<'a> {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Value::F64(f) => f == &f64::from(*other),
            _ => false,
        }
    }
}
