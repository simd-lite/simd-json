use super::Value;
use crate::{prelude::*, BorrowedValue};

#[allow(clippy::cast_sign_loss, clippy::default_trait_access)]
impl PartialEq<BorrowedValue<'_>> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &BorrowedValue<'_>) -> bool {
        match (self, other) {
            (Self::Static(s1), BorrowedValue::Static(s2)) => s1 == s2,
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

#[allow(clippy::cast_sign_loss, clippy::default_trait_access)]
impl PartialEq for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(s1), Self::Static(s2)) => s1.eq(s2),
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
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &&T) -> bool {
        self == *other
    }
}

impl PartialEq<()> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, _other: &()) -> bool {
        self.is_null()
    }
}

impl PartialEq<bool> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &bool) -> bool {
        self.as_bool().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<str> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &str) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<&str> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<String> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &String) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i8> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i8) -> bool {
        self.as_i8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i16> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i16) -> bool {
        self.as_i16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i32> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i32) -> bool {
        self.as_i32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i64> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i64) -> bool {
        self.as_i64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i128> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i128) -> bool {
        self.as_i128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u8> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u8) -> bool {
        self.as_u8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u16> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u16) -> bool {
        self.as_u16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u32> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u32) -> bool {
        self.as_u32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u64> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u64) -> bool {
        self.as_u64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<usize> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &usize) -> bool {
        self.as_usize().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u128> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u128) -> bool {
        self.as_u128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<f32> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &f32) -> bool {
        self.as_f32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<f64> for Value {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &f64) -> bool {
        self.as_f64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<T> PartialEq<&[T]> for Value
where
    Value: PartialEq<T>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &&[T]) -> bool {
        self.as_array().map(|t| t.eq(other)).unwrap_or_default()
    }
}
impl<K, T, S> PartialEq<std::collections::HashMap<K, T, S>> for Value
where
    K: AsRef<str> + std::hash::Hash + Eq,
    Value: PartialEq<T>,
    S: std::hash::BuildHasher,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &std::collections::HashMap<K, T, S>) -> bool {
        self.as_object().map_or(false, |object| {
            object.len() == other.len()
                && other
                    .iter()
                    .all(|(key, value)| object.get(key.as_ref()).map_or(false, |v| *v == *value))
        })
    }
}
