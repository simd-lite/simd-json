use halfbrown::HashMap as Halfbrown;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

/// A JSON Object
pub trait Object {
    /// The key in the objects
    type Key;
    /// The values in the object
    type Element;

    /// Gets a ref to a value based on a key, returns `None` if the
    /// current Value isn't an Object or doesn't contain the key
    /// it was asked for.
    #[must_use]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq;

    /// Gets the value of a key as a mutable reference.
    #[must_use]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq;

    /// Inserts a value
    #[must_use]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq;

    /// Removes a value from the object
    #[must_use]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q>,
        Q: Hash + Eq;

    /// Iterates over the key value paris
    #[must_use]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i>;
}

impl<MapK, MapE> Object for Halfbrown<MapK, MapE>
where
    MapK: Hash + Eq,
{
    type Key = MapK;
    type Element = MapE;

    #[inline]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        Halfbrown::get(self, k)
    }

    #[inline]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        Halfbrown::get_mut(self, k)
    }

    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq,
    {
        Halfbrown::insert(self, k.into(), v.into())
    }

    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        Halfbrown::remove(self, k)
    }

    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i> {
        Box::new(Halfbrown::iter(self))
    }
}

impl<MapK, MapE, S: ::std::hash::BuildHasher> Object for HashMap<MapK, MapE, S>
where
    MapK: Hash + Eq,
{
    type Key = MapK;
    type Element = MapE;

    #[inline]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        HashMap::get(self, k)
    }

    #[inline]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        HashMap::get_mut(self, k)
    }

    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq,
    {
        HashMap::insert(self, k.into(), v.into())
    }

    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        HashMap::remove(self, k)
    }

    #[inline]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i> {
        Box::new(HashMap::iter(self))
    }
}
