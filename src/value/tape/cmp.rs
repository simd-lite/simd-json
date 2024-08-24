use std::borrow::Borrow;

use value_trait::{base::ValueAsScalar, derived::TypedScalarValue};

use super::Value;

#[allow(clippy::cast_sign_loss, clippy::default_trait_access)]
impl<'tape, 'input> PartialEq for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'tape, 'input, T> PartialEq<&T> for Value<'tape, 'input>
where
    Value<'tape, 'input>: PartialEq<T>,
{
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &&T) -> bool {
        self == *other
    }
}

impl<'tape, 'input> PartialEq<()> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, _other: &()) -> bool {
        self.is_null()
    }
}

impl<'tape, 'input> PartialEq<bool> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &bool) -> bool {
        self.as_bool().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<str> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &str) -> bool {
        self.as_str().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<&str> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl<'tape, 'input> PartialEq<String> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &String) -> bool {
        self.as_str().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<i8> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i8) -> bool {
        self.as_i8().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<i16> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i16) -> bool {
        self.as_i16().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<i32> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i32) -> bool {
        self.as_i32().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<i64> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i64) -> bool {
        self.as_i64().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<i128> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &i128) -> bool {
        self.as_i128().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<u8> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u8) -> bool {
        self.as_u8().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<u16> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u16) -> bool {
        self.as_u16().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<u32> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u32) -> bool {
        self.as_u32().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<u64> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u64) -> bool {
        self.as_u64().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<usize> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &usize) -> bool {
        self.as_usize().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<u128> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &u128) -> bool {
        self.as_u128().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<f32> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &f32) -> bool {
        self.as_f32().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input> PartialEq<f64> for Value<'tape, 'input> {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[must_use]
    fn eq(&self, other: &f64) -> bool {
        self.as_f64().is_some_and(|t| t.eq(other))
    }
}

impl<'tape, 'input, K, T, S> PartialEq<std::collections::HashMap<K, T, S>> for Value<'tape, 'input>
where
    K: Borrow<str> + std::hash::Hash + Eq,
    for<'i> T: PartialEq<Value<'i, 'input>>,
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
        for (key, value) in &object {
            if !other.get(key).map_or(false, |v| v == &value) {
                return false;
            }
        }
        true
    }
}
