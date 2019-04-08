mod serde;
use core::borrow::Borrow;
use core::hash::Hash;
use hashbrown::HashMap as HashBrown;
use std::default::Default;
use std::iter::IntoIterator;
use std::ops::Index;

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
    pub fn len(&self) -> usize {
        match self {
            HashMap::Map(m) => m.len(),
            HashMap::Vec(v) => v.len(),
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

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        match self {
            HashMap::Map(m) => Iter::Map(m.iter()),
            HashMap::Vec(m) => Iter::Vec(m.iter()),
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

#[derive(Debug, PartialEq, Clone)]
pub struct VecMap<K, V> {
    v: Vec<(K, V)>,
}

impl<K, V> VecMap<K, V>
where
    K: Eq,
{
    #[inline]
    fn new() -> Self {
        Self { v: Vec::new() }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            v: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn insert(&mut self, k: K, mut v: V) -> Option<V> {
        for (ak, av) in self.v.iter_mut() {
            if k == *ak {
                std::mem::swap(av, &mut v);
                return Some(v);
            }
        }
        self.v.push((k, v));
        None
    }

    #[inline]
    pub fn insert_nocheck(&mut self, k: K, v: V) {
        self.v.push((k, v));
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        for (ak, v) in &self.v {
            if k == ak.borrow() {
                return Some(&v);
            }
        }
        None
    }

    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        for (ak, v) in &mut self.v {
            if k.eq((*ak).borrow()) {
                return Some(v);
            }
        }
        None
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.v.len()
    }
    #[inline]
    pub fn clear(&mut self) {
        self.v.clear()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
        self.v.iter()
    }
}

impl<K, V> IntoIterator for VecMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    #[inline]
    fn into_iter(self) -> std::vec::IntoIter<(K, V)> {
        self.v.into_iter()
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
}
