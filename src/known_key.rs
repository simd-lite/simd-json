use crate::{MutableValue, Value as ValueTrait, ValueType};
use halfbrown::RawEntryMut;
use std::borrow::{Borrow, Cow};
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

#[cfg_attr(tarpaulin, skip)]
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
    pub fn key(&self) -> &Cow<'key, str> {
        &self.key
    }

    /// Looks up this key in a `Value`, returns None if the
    /// key wasn't present or `target` isn't an object
    ///
    /// ```rust
    /// use simd_json::*;
    /// let object = json!({
    ///   "answer": 42,
    ///   "key": 7
    /// });
    /// let known_key = KnownKey::from("answer");
    /// assert_eq!(known_key.lookup(&object).unwrap(), &42);
    /// ```
    #[inline]
    pub fn lookup<'borrow, 'value, V>(&self, target: &'borrow V) -> Option<&'borrow V>
    where
        'key: 'value,
        'value: 'borrow,
        V: ValueTrait + 'value,
        V::Key: Hash + Eq + Borrow<str>,
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
    /// let mut object = json!({
    ///   "answer": 23,
    ///   "key": 7
    /// });
    /// let known_key = KnownKey::from("answer");
    ///
    /// assert_eq!(object["answer"], 23);
    ///
    /// if let Some(answer) = known_key.lookup_mut(&mut object) {
    ///   *answer = OwnedValue::from(42);
    /// }
    ///
    /// assert_eq!(object["answer"], 42);
    /// ```
    #[inline]
    pub fn lookup_mut<'borrow, 'value, V>(&self, target: &'borrow mut V) -> Option<&'borrow mut V>
    where
        'key: 'value,
        'value: 'borrow,
        V: MutableValue + 'value,
        <V as MutableValue>::Key: Hash + Eq + Borrow<str>,
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
    ///
    /// ```rust
    /// use simd_json::*;
    /// let mut object = json!({
    ///   "answer": 23,
    ///   "key": 7
    /// });
    /// let known_key = KnownKey::from("answer");
    ///
    /// assert_eq!(object["answer"], 23);
    ///
    /// if let Ok(answer) = known_key.lookup_or_insert_mut(&mut object, || 17.into()) {
    ///   assert_eq!(*answer, 23);
    ///   *answer = OwnedValue::from(42);
    /// }
    ///
    /// assert_eq!(object["answer"], 42);
    ///
    /// let known_key2 = KnownKey::from("also the answer");
    /// if let Ok(answer) = known_key2.lookup_or_insert_mut(&mut object, || 8.into()) {
    ///   assert_eq!(*answer, 8);
    ///   *answer = OwnedValue::from(42);
    /// }
    ///
    /// assert_eq!(object["also the answer"], 42);
    /// ```
    #[inline]
    pub fn lookup_or_insert_mut<'borrow, 'value, V, F>(
        &self,
        target: &'borrow mut V,
        with: F,
    ) -> Result<&'borrow mut V, Error>
    where
        'key: 'value,
        'value: 'borrow,
        V: ValueTrait + MutableValue + 'value,
        <V as MutableValue>::Key: Hash + Eq + Borrow<str> + From<Cow<'key, str>>,
        F: FnOnce() -> V,
    {
        if !target.is_object() {
            return Err(Error::NotAnObject(target.value_type()));
        }
        target
            .as_object_mut()
            .map(|m| {
                m.raw_entry_mut()
                    .from_key_hashed_nocheck(self.hash, &self.key)
                    .or_insert_with(|| (self.key.clone().into(), with()))
                    .1
            })
            .ok_or(Error::NotAnObject(ValueType::Null))
    }

    /// Inserts a value key into  `Value`, returns None if the
    /// key wasn't present otherwise Some(`old value`).
    /// Errors if `target` isn't an object
    ///
    /// ```rust
    /// use simd_json::*;
    /// let mut object = json!({
    ///   "answer": 23,
    ///   "key": 7
    /// });
    /// let known_key = KnownKey::from("answer");
    ///
    /// assert_eq!(object["answer"], 23);
    ///
    /// assert!(known_key.insert(&mut object, OwnedValue::from(42)).is_ok());
    ///
    /// assert_eq!(object["answer"], 42);
    ///
    /// let known_key2 = KnownKey::from("also the answer");
    ///
    /// assert!(known_key2.insert(&mut object, OwnedValue::from(42)).is_ok());
    ///
    /// assert_eq!(object["also the answer"], 42);
    /// ```
    #[inline]
    pub fn insert<'borrow, 'value, V>(
        &self,
        target: &'borrow mut V,
        value: V,
    ) -> Result<Option<V>, Error>
    where
        'key: 'value,
        'value: 'borrow,
        V: MutableValue + 'value,
        <V as MutableValue>::Key: Hash + Eq + Borrow<str> + From<Cow<'key, str>>,
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
                    e.insert_hashed_nocheck(self.hash, self.key.clone().into(), value);
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
    use crate::borrowed::*;
    use crate::{BorrowedValue, Value as ValueTrait, ValueBuilder};

    #[test]
    fn known_key() {
        use std::borrow::Cow;
        let mut o = Object::new();
        o.insert("key".into(), 1.into());
        let key1 = KnownKey::from(Cow::Borrowed("key"));
        let key2 = KnownKey::from(Cow::Borrowed("cake"));

        let mut v = BorrowedValue::from(o);

        assert!(key1.lookup(&BorrowedValue::null()).is_none());
        assert!(key2.lookup(&BorrowedValue::null()).is_none());
        assert!(key1.lookup(&v).is_some());
        assert!(key2.lookup(&v).is_none());
        assert!(key1.lookup_mut(&mut v).is_some());
        assert!(key2.lookup_mut(&mut v).is_none());
    }

    #[test]
    fn known_key_insert() {
        use std::borrow::Cow;
        let mut o = Object::new();
        o.insert("key".into(), 1.into());
        let key1 = KnownKey::from(Cow::Borrowed("key"));
        let key2 = KnownKey::from(Cow::Borrowed("cake"));

        let mut v = BorrowedValue::from(o);

        let mut v1 = BorrowedValue::null();
        assert!(key1.insert(&mut v1, 2.into()).is_err());
        assert!(key2.insert(&mut v1, 2.into()).is_err());
        assert_eq!(key1.insert(&mut v, 2.into()).unwrap(), Some(1.into()));
        assert_eq!(key2.insert(&mut v, 3.into()).unwrap(), None);
        assert_eq!(v["key"], 2);
        assert_eq!(v["cake"], 3);
    }

    #[test]
    fn lookup_or_insert_mut() {
        use std::borrow::Cow;
        let mut o = Object::new();
        o.insert("key".into(), 1.into());
        let key1 = KnownKey::from(Cow::Borrowed("key"));
        let key2 = KnownKey::from(Cow::Borrowed("cake"));

        let mut v = BorrowedValue::from(o);

        let mut v1 = BorrowedValue::null();
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
        use std::borrow::Cow;
        let mut o = Object::with_capacity(128);
        assert!(o.is_map());
        let key1 = KnownKey::from(Cow::Borrowed("key"));
        let key2 = KnownKey::from(Cow::Borrowed("cake"));

        o.insert("key".into(), 1.into());
        let v = BorrowedValue::from(o);

        assert!(key1.lookup(&BorrowedValue::null()).is_none());
        assert!(key2.lookup(&BorrowedValue::null()).is_none());
        assert!(key1.lookup(&v).is_some());
        assert!(key2.lookup(&v).is_none());
    }

    #[test]
    fn known_key_insert_map() {
        use std::borrow::Cow;
        let mut o = Object::with_capacity(128);
        o.insert("key".into(), 1.into());
        let key1 = KnownKey::from(Cow::Borrowed("key"));
        let key2 = KnownKey::from(Cow::Borrowed("cake"));

        let mut v = BorrowedValue::from(o);

        let mut v1 = BorrowedValue::null();
        assert!(key1.insert(&mut v1, 2.into()).is_err());
        assert!(key2.insert(&mut v1, 2.into()).is_err());
        assert_eq!(key1.insert(&mut v, 2.into()).unwrap(), Some(1.into()));
        assert_eq!(key2.insert(&mut v, 3.into()).unwrap(), None);
        assert_eq!(v["key"], 2);
        assert_eq!(v["cake"], 3);
    }

    #[test]
    fn known_key_get_key() {
        use std::borrow::Cow;
        let mut o = Object::with_capacity(128);
        o.insert("snot".into(), 1.into());
        let key1 = KnownKey::from(Cow::Borrowed("snot"));

        assert_eq!(key1.key(), "snot");
    }
}
