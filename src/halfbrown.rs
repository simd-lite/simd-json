//! Halfbrown is a hashmap implementation that provides
//! high performance for both small and large maps by
//! dymaically switching between different backend.
//!
//! The basic idea is that hash maps are expensive to
//! insert and lookup for small numbers of entries
//! but effective for lager numbers.
//!
//! So for smaller maps, we picked 64 entries as a rule
//! of thumb, we simply store data in a list of tuples.
//! Looking those up and iterating over them is still
//! faster then hasing strings on every lookup.
//!
//! Once we pass the 64 entires we transition the
//! backend to a HashBrown hashmap.

mod macros;
mod vecmap;
use core::borrow::Borrow;
use core::hash::Hash;
use hashbrown::HashMap as HashBrown;
use std::default::Default;
use std::iter::{FromIterator, IntoIterator};
use std::ops::Index;
use vecmap::VecMap;

//const VEC_LOWER_LIMIT: usize = 32;
const VEC_LIMIT_UPPER: usize = 64;

#[derive(Clone, Debug)]
pub enum HashMap<K, V>
where
    K: Eq + Hash,
{
    Map(HashBrown<K, V>),
    Vec(VecMap<K, V>),
    None,
}

impl<K, V> Default for HashMap<K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn default() -> Self {
        HashMap::Vec(VecMap::new())
    }
}

impl<K, Q: ?Sized, V> Index<&Q> for HashMap<K, V>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash,
{
    #[inline]
    pub fn new() -> Self {
        HashMap::Vec(VecMap::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity > VEC_LIMIT_UPPER {
            HashMap::Map(HashBrown::with_capacity(capacity))
        } else {
            HashMap::Vec(VecMap::with_capacity(capacity))
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            HashMap::Map(m) => m.capacity(),
            HashMap::Vec(m) => m.capacity(),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        match self {
            HashMap::Map(m) => Iter::Map(m.iter()),
            HashMap::Vec(m) => Iter::Vec(m.iter()),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        match self {
            HashMap::Map(m) => IterMut::Map(m.iter_mut()),
            HashMap::Vec(m) => IterMut::Vec(m.iter_mut()),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            HashMap::Map(m) => m.len(),
            HashMap::Vec(m) => m.len(),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            HashMap::Map(m) => m.is_empty(),
            HashMap::Vec(m) => m.is_empty(),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self {
            HashMap::Map(m) => m.insert(k, v),
            HashMap::Vec(m) => {
                if m.len() >= VEC_LIMIT_UPPER {
                    let mut r;
                    *self = match std::mem::replace(self, HashMap::None) {
                        HashMap::Vec(m) => {
                            let mut m1: HashBrown<K, V> = m.into_iter().collect();
                            r = m1.insert(k, v);
                            HashMap::Map(m1)
                        }
                        _ => unreachable!(),
                    };
                    r
                } else {
                    m.insert(k, v)
                }
            }
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            HashMap::Map(m) => m.remove(k),
            HashMap::Vec(m) => m.remove(k),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        match self {
            HashMap::Map(m) => m.clear(),
            HashMap::Vec(m) => m.clear(),
            HashMap::None => unreachable!(),
        }
    }
    #[inline]
    pub fn insert_nocheck(&mut self, k: K, v: V) {
        match self {
            HashMap::Map(m) => {
                m.insert(k, v);
            }
            HashMap::Vec(m) => m.insert_nocheck(k, v),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            HashMap::Map(m) => m.get(k),
            HashMap::Vec(m) => m.get(k),
            HashMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            HashMap::Map(m) => m.get_mut(k),
            HashMap::Vec(m) => m.get_mut(k),
            HashMap::None => unreachable!(),
        }
    }

    #[cfg(test)]
    fn is_map(&self) -> bool {
        match self {
            HashMap::Map(_m) => true,
            HashMap::Vec(_m) => false,
            HashMap::None => unreachable!(),
        }
    }

    #[cfg(test)]
    fn is_vec(&self) -> bool {
        match self {
            HashMap::Map(_m) => false,
            HashMap::Vec(_m) => true,
            HashMap::None => unreachable!(),
        }
    }
}

pub enum Iter<'a, K, V> {
    Map(hashbrown::hash_map::Iter<'a, K, V>),
    Vec(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Map(m) => m.next(),
            Iter::Vec(m) => {
                if let Some((k, v)) = m.next() {
                    Some((&k, &v))
                } else {
                    None
                }
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Map(m) => m.size_hint(),
            Iter::Vec(m) => m.size_hint(),
        }
    }
}

pub enum IntoIter<K, V> {
    Map(hashbrown::hash_map::IntoIter<K, V>),
    Vec(std::vec::IntoIter<(K, V)>),
}
impl<K, V> IntoIter<K, V> {
    pub fn len(&self) -> usize {
        match self {
            IntoIter::Map(i) => i.len(),
            IntoIter::Vec(i) => i.len(),
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Map(m) => m.next(),
            IntoIter::Vec(m) => m.next(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            IntoIter::Map(m) => m.size_hint(),
            IntoIter::Vec(m) => m.size_hint(),
        }
    }
}

impl<K, V> IntoIterator for HashMap<K, V>
where
    K: Eq + Hash,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> IntoIter<K, V> {
        match self {
            HashMap::Map(m) => IntoIter::Map(m.into_iter()),
            HashMap::Vec(m) => IntoIter::Vec(m.into_iter()),
            HashMap::None => unreachable!(),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V>
where
    K: Eq + Hash,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let mut map = Self::with_capacity(iter.size_hint().0);
        iter.for_each(|(k, v)| {
            map.insert(k, v);
        });
        map
    }
}

// Taken from hashbrown
impl<K, V> PartialEq for HashMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

//#[derive(Clone)]
pub enum IterMut<'a, K, V> {
    Map(hashbrown::hash_map::IterMut<'a, K, V>),
    Vec(std::slice::IterMut<'a, (K, V)>),
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        match self {
            IterMut::Map(m) => m.next(),
            IterMut::Vec(m) => m.next().map(|(k, v)| (k as &K, v)),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            IterMut::Map(m) => m.size_hint(),
            IterMut::Vec(m) => m.size_hint(),
        }
    }
}

//#[derive(Clone)]
pub struct Keys<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<(&'a K)> {
        self.inner.next().map(|(k, _)| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

//#[derive(Clone)]
pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}
impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<(&'a V)> {
        self.inner.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

//#[derive(Clone)]
pub struct ValuesMut<'a, K, V> {
    inner: IterMut<'a, K, V>,
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<(&'a mut V)> {
        self.inner.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scale_up() {
        let mut v = HashMap::new();
        assert!(v.is_vec());
        for i in 1..65 {
            // 64 entries
            v.insert(i, i);
            assert!(v.is_vec());
        }
        v.insert(65, 65);
        assert!(v.is_map());
    }

    #[test]
    fn str_key() {
        let mut v: HashMap<String, u32> = HashMap::new();
        v.insert("hello".to_owned(), 42);
        assert_eq!(v["hello"], 42);
    }

    #[test]
    fn add_remove() {
        let mut v = HashMap::new();
        v.insert(1, 1);
        v.insert(2, 2);
        v.insert(3, 3);
        assert_eq!(v.get(&1), Some(&1));
        assert_eq!(v.get(&2), Some(&2));
        assert_eq!(v.get(&3), Some(&3));
        v.remove(&2);
        assert_eq!(v.get(&1), Some(&1));
        assert_eq!(v.get(&2), None);
        assert_eq!(v.get(&3), Some(&3));
    }

}
