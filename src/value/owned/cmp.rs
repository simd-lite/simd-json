use super::Value;
use crate::{BorrowedValue, ValueTrait};

use float_cmp::approx_eq;

#[allow(clippy::cast_sign_loss)]
impl PartialEq<BorrowedValue<'_>> for Value {
    fn eq(&self, other: &BorrowedValue<'_>) -> bool {
        #[allow(clippy::default_trait_access)]
        match (self, other) {
            (Self::Null, BorrowedValue::Null) => true,
            (Self::Bool(v1), BorrowedValue::Bool(v2)) => v1.eq(v2),
            (Self::I64(v1), BorrowedValue::I64(v2)) => v1.eq(v2),
            (Self::U64(v1), BorrowedValue::U64(v2)) => v1.eq(v2),
            // NOTE: We swap v1 and v2 here to avoid having to juggle ref's
            (Self::U64(v1), BorrowedValue::I64(v2)) if *v2 >= 0 => (*v2 as u64).eq(v1),
            (Self::I64(v1), BorrowedValue::U64(v2)) if *v1 >= 0 => (*v1 as u64).eq(v2),
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

#[allow(clippy::cast_sign_loss)]
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::default_trait_access)]
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(v1), Self::Bool(v2)) => v1.eq(v2),
            (Self::I64(v1), Self::I64(v2)) => v1.eq(v2),
            (Self::U64(v1), Self::U64(v2)) => v1.eq(v2),
            // NOTE: We swap v1 and v2 here to avoid having to juggle ref's
            (Self::U64(v1), Self::I64(v2)) if *v2 >= 0 => (*v2 as u64).eq(v1),
            (Self::I64(v1), Self::U64(v2)) if *v1 >= 0 => (*v1 as u64).eq(v2),
            (Self::F64(v1), Self::F64(v2)) => approx_eq!(f64, *v1, *v2),
            (Self::String(v1), Self::String(v2)) => v1.eq(v2),
            (Self::Array(v1), Self::Array(v2)) => v1.eq(v2),
            (Self::Object(v1), Self::Object(v2)) => v1.eq(v2),
            _ => false,
        }
    }
}

impl<T> PartialEq<&T> for Value
where
    Value: PartialEq<T>,
{
    fn eq(&self, other: &&T) -> bool {
        self == *other
    }
}

impl PartialEq<()> for Value {
    fn eq(&self, _other: &()) -> bool {
        self.is_null()
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        self.as_bool().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().map(|t| t.eq(*other)).unwrap_or_default()
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i8> for Value {
    fn eq(&self, other: &i8) -> bool {
        self.as_i8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i16> for Value {
    fn eq(&self, other: &i16) -> bool {
        self.as_i16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        self.as_i32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, other: &i64) -> bool {
        self.as_i64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i128> for Value {
    fn eq(&self, other: &i128) -> bool {
        self.as_i128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u8> for Value {
    fn eq(&self, other: &u8) -> bool {
        self.as_u8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u16> for Value {
    fn eq(&self, other: &u16) -> bool {
        self.as_u16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u32> for Value {
    fn eq(&self, other: &u32) -> bool {
        self.as_u32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u64> for Value {
    fn eq(&self, other: &u64) -> bool {
        self.as_u64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<usize> for Value {
    fn eq(&self, other: &usize) -> bool {
        self.as_usize().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u128> for Value {
    fn eq(&self, other: &u128) -> bool {
        self.as_u128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, other: &f32) -> bool {
        self.as_f32().map(|t| t.eq(other)).unwrap_or_default()
    }
}
impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        self.as_f64().map(|t| t.eq(other)).unwrap_or_default()
    }
}
