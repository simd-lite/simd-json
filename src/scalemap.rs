use core::borrow::Borrow;
use core::hash::Hash;
use hashbrown::HashMap;
use std::iter::IntoIterator;
use std::ops::Index;

//const VEC_LOWER_LIMIT: usize = 32;
const VEC_LIMIT_UPPER: usize = 64;

#[derive(Clone, Debug)]
pub enum ScaleMap<K, V>
where
    K: Eq + Hash,
{
    Map(HashMap<K, V>),
    Vec(VecMap<K, V>),
    None,
}

impl<K, Q: ?Sized, V> Index<&Q> for ScaleMap<K, V>
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

impl<K, V> ScaleMap<K, V>
where
    K: Eq + Hash,
{
    #[inline]
    pub fn new() -> Self {
        ScaleMap::Vec(VecMap::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity > VEC_LIMIT_UPPER {
            ScaleMap::Map(HashMap::with_capacity(capacity))
        } else {
            ScaleMap::Vec(VecMap::with_capacity(capacity))
        }
    }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self {
            ScaleMap::Map(m) => m.insert(k, v),
            ScaleMap::Vec(m) => {
                if m.len() >= VEC_LIMIT_UPPER {
                    let mut r;
                    *self = match std::mem::replace(self, ScaleMap::None) {
                        ScaleMap::Vec(m) => {
                            let mut m1: HashMap<K, V> = m.into_iter().collect();
                            r = m1.insert(k, v);
                            ScaleMap::Map(m1)
                        }
                        _ => unreachable!(),
                    };
                    r
                } else {
                    m.insert(k, v)
                }
            }
            ScaleMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        match self {
            ScaleMap::Map(m) => m.clear(),
            ScaleMap::Vec(m) => m.clear(),
            ScaleMap::None => unreachable!(),
        }
    }
    #[inline]
    pub fn insert_nocheck(&mut self, k: K, v: V) {
        match self {
            ScaleMap::Map(m) => {
                m.insert(k, v);
            }
            ScaleMap::Vec(m) => m.insert_nocheck(k, v),
            ScaleMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            ScaleMap::Map(m) => m.len(),
            ScaleMap::Vec(v) => v.len(),
            ScaleMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            ScaleMap::Map(m) => m.get(k),
            ScaleMap::Vec(m) => m.get(k),
            ScaleMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            ScaleMap::Map(m) => m.get_mut(k),
            ScaleMap::Vec(m) => m.get_mut(k),
            ScaleMap::None => unreachable!(),
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        match self {
            ScaleMap::Map(m) => Iter::Map(m.iter()),
            ScaleMap::Vec(m) => Iter::Vec(m.iter()),
            ScaleMap::None => unreachable!(),
        }
    }

    #[cfg(test)]
    fn is_map(&self) -> bool {
        match self {
            ScaleMap::Map(_m) => true,
            ScaleMap::Vec(_m) => false,
            ScaleMap::None => unreachable!(),
        }
    }

    #[cfg(test)]
    fn is_vec(&self) -> bool {
        match self {
            ScaleMap::Map(_m) => false,
            ScaleMap::Vec(_m) => true,
            ScaleMap::None => unreachable!(),
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

impl<K, V> IntoIterator for ScaleMap<K, V>
where
    K: Eq + Hash,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> IntoIter<K, V> {
        match self {
            ScaleMap::Map(m) => IntoIter::Map(m.into_iter()),
            ScaleMap::Vec(m) => IntoIter::Vec(m.into_iter()),
            ScaleMap::None => unreachable!(),
        }
    }
}

// Taken from hashbrown
impl<K, V> PartialEq for ScaleMap<K, V>
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
        let mut v = ScaleMap::new();
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
        let mut v: ScaleMap<String, u32> = ScaleMap::new();
        v.insert("hello".to_owned(), 42);
        assert_eq!(v["hello"], 42);
    }
}
