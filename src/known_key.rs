use crate::cow::Cow;
use crate::prelude::*;
use crate::BorrowedValue as Value;
use halfbrown::RawEntryMut;
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};

/// Well known key that can be looked up in a `Value` faster.
/// It achives this by memorizing the hash.
#[derive(Debug, Clone, PartialEq)]
pub struct KnownKey<'key> {
    key: Cow<'key, str>,
    hash: u64,
}

/// Error for known keys
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// The target passed wasn't an object
    NotAnObject(ValueType),
}

#[cfg(not(tarpaulin_include))]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotAnObject(t) => write!(f, "Expected object but got {:?}", t),
        }
    }
}
impl std::error::Error for Error {}

impl<'key, S> From<S> for KnownKey<'key>
where
    Cow<'key, str>: From<S>,
{
    fn from(key: S) -> Self {
        let key = Cow::from(key);
        let hash_builder = halfbrown::DefaultHashBuilder::default();
        let mut hasher = hash_builder.build_hasher();
        key.hash(&mut hasher);
        Self {
            hash: hasher.finish(),
            key,
        }
    }
}

impl<'key> KnownKey<'key> {
    /// The known key
    #[inline]
    #[must_use]
    pub fn key(&self) -> &Cow<'key, str> {
        &self.key
    }

    /// Looks up this key in a `Value`, returns None if the
    /// key wasn't present or `target` isn't an object
    ///
    /// # Errors
    ///  * If target is not an object
    ///
    /// ```rust
    /// use simd_json::*;
    /// let object = json!({
    ///   "answer": 42,
    ///   "key": 7
    /// }).into();
    /// let known_key = KnownKey::from("answer");
    /// assert_eq!(known_key.lookup(&object).unwrap(), &42);
    /// ```
    #[inline]
    #[must_use]
    pub fn lookup<'target, 'value>(
        &self,
        target: &'target Value<'value>,
    ) -> Option<&'target Value<'value>>
    where
        'key: 'value,
        'value: 'target,
    {
        target
            .as_object()
            .and_then(|m| m.raw_entry().from_key_hashed_nocheck(self.hash, &self.key))
            .map(|kv| kv.1)
    }

    /// Looks up this key in a `Value`, returns None if the
    /// key wasn't present or `target` isn't an object
    ///
    /// ```rust
    /// use simd_json::*;
    /// let mut object: BorrowedValue = json!({
    ///   "answer": 23,
    ///   "key": 7
    /// }).into();
    /// let known_key = KnownKey::from("answer");
    ///
    /// assert_eq!(object["answer"], 23);
    ///
    /// if let Some(answer) = known_key.lookup_mut(&mut object) {
    ///   *answer = BorrowedValue::from(42);
    /// }
    ///
    /// assert_eq!(object["answer"], 42);
    /// ```
    #[inline]
    pub fn lookup_mut<'target, 'value>(
        &self,
        target: &'target mut Value<'value>,
    ) -> Option<&'target mut Value<'value>>
    where
        'key: 'value,
        'value: 'target,
    {
        target.as_object_mut().and_then(|m| {
            match m
                .raw_entry_mut()
                .from_key_hashed_nocheck(self.hash, &self.key)
            {
                RawEntryMut::Occupied(e) => Some(e.into_mut()),
                RawEntryMut::Vacant(_e) => None,
            }
        })
    }

    /// Looks up this key in a `Value`, inserts `with` when the key
    ///  when wasn't present returns None if the `target` isn't an object
    /// # Errors
    /// * if target is not a record
    ///
    /// ```rust
    /// use simd_json::*;
    /// let mut object: BorrowedValue = json!({
    ///   "answer": 23,
    ///   "key": 7
    /// }).into();
    /// let known_key = KnownKey::from("answer");
    ///
    /// assert_eq!(object["answer"], 23);
    ///
    /// if let Ok(answer) = known_key.lookup_or_insert_mut(&mut object, || 17.into()) {
    ///   assert_eq!(*answer, 23);
    ///   *answer = BorrowedValue::from(42);
    /// }
    ///
    /// assert_eq!(object["answer"], 42);
    ///
    /// let known_key2 = KnownKey::from("also the answer");
    /// if let Ok(answer) = known_key2.lookup_or_insert_mut(&mut object, || 8.into()) {
    ///   assert_eq!(*answer, 8);
    ///   *answer = BorrowedValue::from(42);
    /// }
    ///
    /// assert_eq!(object["also the answer"], 42);
    /// ```
    #[inline]
    pub fn lookup_or_insert_mut<'target, 'value, F>(
        &self,
        target: &'target mut Value<'value>,
        with: F,
    ) -> Result<&'target mut Value<'value>, Error>
    where
        'key: 'value,
        'value: 'target,
        F: FnOnce() -> Value<'value>,
    {
        if !target.is_object() {
            return Err(Error::NotAnObject(target.value_type()));
        }
        target
            .as_object_mut()
            .map(|m| {
                m.raw_entry_mut()
                    .from_key_hashed_nocheck(self.hash, &self.key)
                    .or_insert_with(|| (self.key.clone(), with()))
                    .1
            })
            .ok_or(Error::NotAnObject(ValueType::Null))
    }

    /// Inserts a value key into  `Value`, returns None if the
    /// key wasn't present otherwise Some(`old value`).
    /// # Errors
    ///   * if `target` isn't an object
    ///
    /// ```rust
    /// use simd_json::*;
    /// let mut object: BorrowedValue = json!({
    ///   "answer": 23,
    ///   "key": 7
    /// }).into();
    /// let known_key = KnownKey::from("answer");
    ///
    /// assert_eq!(object["answer"], 23);
    ///
    /// assert!(known_key.insert(&mut object, BorrowedValue::from(42)).is_ok());
    ///
    /// assert_eq!(object["answer"], 42);
    ///
    /// let known_key2 = KnownKey::from("also the answer");
    ///
    /// assert!(known_key2.insert(&mut object, BorrowedValue::from(42)).is_ok());
    ///
    /// assert_eq!(object["also the answer"], 42);
    ///
    /// ```
    #[inline]
    pub fn insert<'target, 'value>(
        &self,
        target: &'target mut Value<'value>,
        value: Value<'value>,
    ) -> Result<Option<Value<'value>>, Error>
    where
        'key: 'value,
        'value: 'target,
    {
        if !target.is_object() {
            return Err(Error::NotAnObject(target.value_type()));
        }

        Ok(target.as_object_mut().and_then(|m| {
            match m
                .raw_entry_mut()
                .from_key_hashed_nocheck(self.hash, &self.key)
            {
                RawEntryMut::Occupied(mut e) => Some(e.insert(value)),
                RawEntryMut::Vacant(e) => {
                    e.insert_hashed_nocheck(self.hash, self.key.clone(), value);
                    None
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unnecessary_operation, clippy::non_ascii_literal)]
    use super::*;

    #[test]
    fn known_key() {
        use crate::cow::Cow;
        let mut v = Value::object();
        v.insert("key", 1).unwrap();
        let key1 = KnownKey::from(Cow::from("key"));
        let key2 = KnownKey::from(Cow::from("cake"));

        assert!(key1.lookup(&Value::null()).is_none());
        assert!(key2.lookup(&Value::null()).is_none());
        assert!(key1.lookup(&v).is_some());
        assert!(key2.lookup(&v).is_none());
        assert!(key1.lookup_mut(&mut v).is_some());
        assert!(key2.lookup_mut(&mut v).is_none());
    }

    #[test]
    fn known_key_insert() {
        use crate::cow::Cow;
        let mut v = Value::object();
        v.insert("key", 1).unwrap();
        let key1 = KnownKey::from(Cow::from("key"));
        let key2 = KnownKey::from(Cow::from("cake"));

        let mut v1 = Value::null();
        assert!(key1.insert(&mut v1, 2.into()).is_err());
        assert!(key2.insert(&mut v1, 2.into()).is_err());
        assert_eq!(key1.insert(&mut v, 2.into()).unwrap(), Some(1.into()));
        assert_eq!(key2.insert(&mut v, 3.into()).unwrap(), None);
        assert_eq!(v["key"], 2);
        assert_eq!(v["cake"], 3);
    }

    #[test]
    fn lookup_or_insert_mut() {
        use crate::cow::Cow;
        let mut v = Value::object();
        v.insert("key", 1).unwrap();
        let key1 = KnownKey::from(Cow::from("key"));
        let key2 = KnownKey::from(Cow::from("cake"));

        let mut v1 = Value::null();
        assert!(key1.lookup_or_insert_mut(&mut v1, || 2.into()).is_err());
        assert!(key2.lookup_or_insert_mut(&mut v1, || 2.into()).is_err());

        {
            let r1 = key1.lookup_or_insert_mut(&mut v, || 2.into()).unwrap();
            assert_eq!(r1.as_u8(), Some(1));
        }
        {
            let r2 = key2.lookup_or_insert_mut(&mut v, || 3.into()).unwrap();
            assert_eq!(r2.as_u8(), Some(3));
        }
    }
    #[test]
    fn known_key_map() {
        use crate::cow::Cow;
        let mut v = Value::object_with_capacity(128);
        v.insert("key", 1).unwrap();
        let key1 = KnownKey::from(Cow::from("key"));
        let key2 = KnownKey::from(Cow::from("cake"));

        assert!(key1.lookup(&Value::null()).is_none());
        assert!(key2.lookup(&Value::null()).is_none());
        assert!(key1.lookup(&v).is_some());
        assert!(key2.lookup(&v).is_none());
    }

    #[test]
    fn known_key_insert_map() {
        use crate::cow::Cow;
        let mut v = Value::object_with_capacity(128);
        v.insert("key", 1).unwrap();
        let key1 = KnownKey::from(Cow::from("key"));
        let key2 = KnownKey::from(Cow::from("cake"));

        let mut v1 = Value::null();

        assert!(key1.insert(&mut v1, 2.into()).is_err());
        assert!(key2.insert(&mut v1, 2.into()).is_err());
        assert_eq!(key1.insert(&mut v, 2.into()).unwrap(), Some(1.into()));
        assert_eq!(key2.insert(&mut v, 3.into()).unwrap(), None);
        assert_eq!(v["key"], 2);
        assert_eq!(v["cake"], 3);
    }

    #[test]
    fn known_key_get_key() {
        let key1 = KnownKey::from("snot");

        assert_eq!(key1.key(), "snot");
    }
}
