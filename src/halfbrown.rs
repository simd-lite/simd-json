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
//!
//! Note: Most of the documentation is taken from
//! rusts hashmap.rs and should be considered under
//! their copyright.

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
    /// Creates an empty `HashMap`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_hson::HashMap;
    /// let mut map: HashMap<&str, i32> = HashMap::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        HashMap::Vec(VecMap::new())
    }

    /// Creates an empty `HashMap` with the specified capacity.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    /// let mut map: HashMap<&str, i32> = HashMap::with_capacity(10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity > VEC_LIMIT_UPPER {
            HashMap::Map(HashBrown::with_capacity(capacity))
        } else {
            HashMap::Vec(VecMap::with_capacity(capacity))
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash,
{
    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the `HashMap<K, V>` might be able to hold
    /// more, but is guaranteed to be able to hold at least this many.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    /// let map: HashMap<i32, i32> = HashMap::with_capacity(100);
    /// assert!(map.capacity() >= 100);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            HashMap::Map(m) => m.capacity(),
            HashMap::Vec(m) => m.capacity(),
            HashMap::None => unreachable!(),
        }
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for key in map.keys() {
    ///     println!("{}", key);
    /// }
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    /// }
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    ///
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values_mut() {
    ///     *val = *val + 10;
    /// }
    ///
    /// for val in map.values() {
    ///     println!("{}", val);
    /// }
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        match self {
            HashMap::Map(m) => Iter::Map(m.iter()),
            HashMap::Vec(m) => Iter::Vec(m.iter()),
            HashMap::None => unreachable!(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// for (key, val) in &map {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        match self {
            HashMap::Map(m) => IterMut::Map(m.iter_mut()),
            HashMap::Vec(m) => IterMut::Vec(m.iter_mut()),
            HashMap::None => unreachable!(),
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            HashMap::Map(m) => m.len(),
            HashMap::Vec(m) => m.len(),
            HashMap::None => unreachable!(),
        }
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            HashMap::Map(m) => m.is_empty(),
            HashMap::Vec(m) => m.is_empty(),
            HashMap::None => unreachable!(),
        }
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the
    /// allocated memory for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    ///
    /// for (k, v) in a.drain().take(1) {
    ///     assert!(k == 1 || k == 2);
    ///     assert!(v == "a" || v == "b");
    /// }
    ///
    /// assert!(a.is_empty());
    /// ```
    #[inline]
    pub fn drain(&mut self) -> Drain<K, V> {
        match self {
            HashMap::Map(m) => Drain::Map(m.drain()),
            HashMap::Vec(m) => Drain::Vec(m.drain()),
            HashMap::None => unreachable!(),
        }
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut a = HashMap::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        match self {
            HashMap::Map(m) => m.clear(),
            HashMap::Vec(m) => m.clear(),
            HashMap::None => unreachable!(),
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash,
    // Hasher
{
    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `HashMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows [`usize`].
    ///
    /// [`usize`]: ../../std/primitive.usize.html
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    /// let mut map: HashMap<&str, i32> = HashMap::new();
    /// map.reserve(10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        match self {
            HashMap::Map(m) => m.reserve(additional),
            HashMap::Vec(m) => m.reserve(additional),
            HashMap::None => unreachable!(),
        }
    }

    /*
    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the given `HashMap<K,V>`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(try_reserve)]
    /// use simd_json::HashMap;
    /// let mut map: HashMap<&str, isize> = HashMap::new();
    /// map.try_reserve(10).expect("why is the test harness OOMing on 10 bytes?");
    /// ```
    #[unstable(feature = "try_reserve", reason = "new API", issue = "48043")]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
        match self {
            HashMap::Map(m) => m.try_reserve(additional),
            HashMap::Vec(m) => m.try_reserve(additional),
            HashMap::None => unreachable!(),
        }
    }
     */

    /// Shrinks the capacity of the map as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map: HashMap<i32, i32> = HashMap::with_capacity(100);
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// assert!(map.capacity() >= 100);
    /// map.shrink_to_fit();
    /// assert!(map.capacity() >= 2);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        match self {
            HashMap::Map(m) => m.shrink_to_fit(),
            HashMap::Vec(m) => m.shrink_to_fit(),
            HashMap::None => unreachable!(),
        }
    }

    /* TODO
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut letters = HashMap::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     let counter = letters.entry(ch).or_insert(0);
    ///     *counter += 1;
    /// }
    ///
    /// assert_eq!(letters[&'s'], 2);
    /// assert_eq!(letters[&'t'], 3);
    /// assert_eq!(letters[&'u'], 1);
    /// assert_eq!(letters.get(&'y'), None);
    /// ```
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        match self {
            HashMap::Map(m) => m.entry(k),
            HashMap::Vec(m) => m.entry(v),
            HashMap::None => unreachable!(),
        }
    }
    */

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// `    #[inline]
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

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            HashMap::Map(m) => m.contains_key(k),
            HashMap::Vec(m) => m.contains_key(k),
            HashMap::None => unreachable!(),
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```

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

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [module-level
    /// documentation] for more.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    /// [module-level documentation]: index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
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

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// [`Eq`]: ../../std/cmp/trait.Eq.html
    /// [`Hash`]: ../../std/hash/trait.Hash.html
    ///
    /// # Examples
    ///
    /// ```
    /// use simd_json::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
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
    pub fn insert_nocheck(&mut self, k: K, v: V) {
        match self {
            HashMap::Map(m) => {
                m.insert(k, v);
            }
            HashMap::Vec(m) => m.insert_nocheck(k, v),
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

pub enum Drain<'a, K, V> {
    Map(hashbrown::hash_map::Drain<'a, K, V>),
    Vec(std::vec::Drain<'a, (K, V)>),
}

impl<'a, K, V> Iterator for Drain<'a, K, V> {
    type Item = (K, V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Drain::Map(m) => m.next(),
            Drain::Vec(m) => m.next(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Drain::Map(m) => m.size_hint(),
            Drain::Vec(m) => m.size_hint(),
        }
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
