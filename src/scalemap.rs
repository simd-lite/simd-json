use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use hashbrown::HashMap;

const VEC_LOWER_LIMIT: usize = 32;
const VEC_LIMIT_UPPER: usize = 64;

#[derive(Clone, Debug)]
pub enum ScaleMap<K, V>
where
    K: Eq + Hash,
{
    Map(HashMap<K, V>),
    Vec(VecMap<K, V>),
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
            dbg!("map");
            ScaleMap::Map(HashMap::with_capacity(capacity))
        } else {
            ScaleMap::Vec(VecMap::with_capacity(capacity))
        }
    }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self {
            ScaleMap::Map(m) => m.insert(k, v),
            ScaleMap::Vec(m) => m.insert(k, v),
        }
    }

    #[inline]
    pub fn insert_nocheck(&mut self, k: K, v: V)  {
        match self {
            ScaleMap::Map(m) => {m.insert(k, v);},
            ScaleMap::Vec(m) => m.insert_nocheck(k, v),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            ScaleMap::Map(m) => m.len(),
            ScaleMap::Vec(v) => v.len(),
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
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        match self {
            ScaleMap::Map(m) => Iter::Map(m.iter()),
            ScaleMap::Vec(m) => Iter::Vec(m.iter()),
        }
    }
}

pub enum Iter<'a, K, V> {
    Map(hashbrown::hash_map::Iter<'a, K, V>),
    Vec(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Map(m) => m.size_hint(),
            Iter::Vec(m) => m.size_hint(),
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
    pub fn insert_nocheck(&mut self, k: K, mut v: V) {
        self.v.push((k, v));
    }

    #[inline]
    pub fn insert_(&mut self, k: K, mut v: V) -> Option<V> {
        /*
        for (ak, av) in self.v.iter_mut() {
        if k == *ak {
        std::mem::swap(av, &mut v);
        return Some(v);
    }
    }
         */
        self.v.push((k, v));
        None
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
    pub fn len(&self) -> usize {
        self.v.len()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
        self.v.iter()
    }
}

/*

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.inner.next().map(|x| unsafe {
            let r = x.as_ref();
            (&r.0, &r.1)
        })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
*/
