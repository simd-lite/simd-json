use super::Value;
use crate::OwnedValue;
use float_cmp::approx_eq;

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::default_trait_access)]
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(v1), Self::Bool(v2)) => v1.eq(v2),
            (Self::I64(v1), Self::I64(v2)) => v1.eq(v2),
            (Self::F64(v1), Self::F64(v2)) => approx_eq!(f64, *v1, *v2),
            (Self::String(v1), Self::String(v2)) => v1.eq(v2),
            (Self::Array(v1), Self::Array(v2)) => v1.eq(v2),
            (Self::Object(v1), Self::Object(v2)) => v1.eq(v2),
            _ => false,
        }
    }
}

impl<'a> PartialEq<OwnedValue> for Value<'a> {
    fn eq(&self, other: &OwnedValue) -> bool {
        // We only need to implement this once
        other.eq(self)
    }
}
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
            Value::I64(i) => i == other,
            _ => false,
        }
    }
}

impl<'a> PartialEq<u8> for Value<'a> {
    fn eq(&self, other: &u8) -> bool {
        match self {
            Self::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl<'a> PartialEq<u16> for Value<'a> {
    fn eq(&self, other: &u16) -> bool {
        match self {
            Self::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl<'a> PartialEq<u32> for Value<'a> {
    fn eq(&self, other: &u32) -> bool {
        match self {
            Self::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl<'a> PartialEq<u64> for Value<'a> {
    fn eq(&self, other: &u64) -> bool {
        use std::convert::TryFrom;
        match self {
            Self::I64(i) => i64::try_from(*other).map(|o| *i == o).unwrap_or(false),
            _ => false,
        }
    }
}

impl<'a> PartialEq<usize> for Value<'a> {
    fn eq(&self, other: &usize) -> bool {
        use std::convert::TryFrom;
        match self {
            Self::I64(i) => i64::try_from(*other).map(|o| *i == o).unwrap_or(false),
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
            Value::F64(f) => f == other,
            _ => false,
        }
    }
}
