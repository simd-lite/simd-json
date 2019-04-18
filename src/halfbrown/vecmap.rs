use core::borrow::Borrow;

#[derive(Debug, PartialEq, Clone)]
pub struct VecMap<K, V> {
    v: Vec<(K, V)>,
}

impl<K, V> VecMap<K, V>
where
    K: Eq,
{
    #[inline]
    pub fn new() -> Self {
        Self { v: Vec::new() }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            v: Vec::with_capacity(capacity),
        }
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.v.capacity()
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
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        let mut i = 0;
        while i != self.v.len() {
            let (ak, _) = unsafe { self.v.get_unchecked(i) };
            if k == ak.borrow() {
                let (_, v) = self.v.swap_remove(i);
                return Some(v);
            }
            i += 1;
        }
        None
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
        self.v.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, (K, V)> {
        self.v.iter_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.v.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    #[inline]
    pub fn drain(&mut self) -> std::vec::Drain<(K, V)> {
        self.v.drain(..)
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.v.reserve(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.v.shrink_to_fit()
    }

    #[inline]
    pub fn insert_nocheck(&mut self, k: K, v: V) {
        self.v.push((k, v));
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        for (ak, v) in &self.v {
            if k == ak.borrow() {
                return Some(&v);
            }
        }
        None
    }

    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        for (ak, _v) in &self.v {
            if k == ak.borrow() {
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        for (ak, v) in &mut self.v {
            if k.eq((*ak).borrow()) {
                return Some(v);
            }
        }
        None
    }

    #[inline]
    pub fn clear(&mut self) {
        self.v.clear()
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
