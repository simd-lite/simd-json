use super::Value;
use crate::BorrowedValue;
use float_cmp::approx_eq;

impl PartialEq<BorrowedValue<'_>> for Value {
    fn eq(&self, other: &BorrowedValue<'_>) -> bool {
        #[allow(clippy::default_trait_access)]
        match (self, other) {
            (Self::Null, BorrowedValue::Null) => true,
            (Self::Bool(v1), BorrowedValue::Bool(v2)) => v1.eq(v2),
            (Self::I64(v1), BorrowedValue::I64(v2)) => v1.eq(v2),
            (Self::F64(v1), BorrowedValue::F64(v2)) => approx_eq!(f64, *v1, *v2),
            (Self::String(v1), BorrowedValue::String(v2)) => v1.eq(v2),
            (Self::Array(v1), BorrowedValue::Array(v2)) => v1.eq(v2),
            (Self::Object(v1), BorrowedValue::Object(v2)) => {
                if v1.len() != v2.len() {
                    return false;
                }
                v1.iter()
                    .all(|(key, value)| v2.get(key.as_str()).map_or(false, |v| value == v))
            }
            _ => false,
        }
    }
}

impl PartialEq for Value {
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

impl PartialEq<()> for Value {
    fn eq(&self, _other: &()) -> bool {
        if let Self::Null = self {
            true
        } else {
            false
        }
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        if let Self::Bool(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::String(s) => s == other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::String(s) => s == other,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        match self {
            Self::String(s) => s == other,
            _ => false,
        }
    }
}

impl PartialEq<i8> for Value {
    fn eq(&self, other: &i8) -> bool {
        match self {
            Self::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i16> for Value {
    fn eq(&self, other: &i16) -> bool {
        match self {
            Self::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        match self {
            Self::I64(i) => i == &i64::from(*other),
            _ => false,
        }
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Self::I64(i) => i == other,
            _ => false,
        }
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, other: &f32) -> bool {
        match self {
            Self::F64(f) => f == &f64::from(*other),
            _ => false,
        }
    }
}
impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Self::F64(f) => f == other,
            _ => false,
        }
    }
}
