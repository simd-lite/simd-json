use std::borrow::Borrow;
use value_trait::{base::ValueAsScalar, derived::TypedScalarValue};

use super::Value;

impl<'borrow, 'tape, 'input> PartialEq<()> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, _other: &()) -> bool {
        self.is_null()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<bool> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &bool) -> bool {
        self.as_bool().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<str> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &str) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<&str> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl<'borrow, 'tape, 'input> PartialEq<String> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &String) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<i8> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i8) -> bool {
        self.as_i8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<i16> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i16) -> bool {
        self.as_i16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<i32> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i32) -> bool {
        self.as_i32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<i64> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i64) -> bool {
        self.as_i64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<i128> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i128) -> bool {
        self.as_i128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<u8> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u8) -> bool {
        self.as_u8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<u16> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u16) -> bool {
        self.as_u16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<u32> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u32) -> bool {
        self.as_u32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<u64> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u64) -> bool {
        self.as_u64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<usize> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &usize) -> bool {
        self.as_usize().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<u128> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u128) -> bool {
        self.as_u128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<f32> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &f32) -> bool {
        self.as_f32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input> PartialEq<f64> for Value<'borrow, 'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &f64) -> bool {
        self.as_f64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl<'borrow, 'tape, 'input, K, T, S> PartialEq<std::collections::HashMap<K, T, S>>
    for Value<'borrow, 'tape, 'input>
where
    K: Borrow<str> + std::hash::Hash + Eq,
    for<'b, 't, 'i> T: PartialEq<Value<'b, 't, 'i>>,
    S: std::hash::BuildHasher,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &std::collections::HashMap<K, T, S>) -> bool {
        let Some(object) = self.as_object() else {
            return false;
        };
        if object.len() != other.len() {
            return false;
        }
        for (key, value) in object.iter() {
            if !other.get(key).map_or(false, |v| v == &value) {
                return false;
            }
        }
        true
    }
}
