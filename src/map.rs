use fnv::FnvHashMap;
use std::{borrow::Borrow, fmt::Debug, hash::Hash};

#[derive(Clone, Debug)]
pub struct Map<K, V>
where
    K: Clone + Hash + Eq + Debug,
    V: Clone + Debug,
{
    inner: FnvHashMap<K, V>,
}

impl<K, V> Map<K, V>
where
    K: Clone + Eq + Hash + Debug,
    V: Clone + Debug,
{
    #[inline]
    pub fn new() -> Self {
        Map {
            inner: FnvHashMap::default(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get(k)
    }

    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get_mut(k)
    }

    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.contains_key(k)
    }

    #[inline]
    pub fn optimize(&mut self) -> &Self {
        self.inner.shrink_to_fit();
        self
    }

    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.inner.insert(k, v)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a K, &'a V)> + 'a> {
        Box::new(self.inner.iter())
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = (&'a K, &'a mut V)> + 'a> {
        Box::new(self.inner.iter_mut())
    }
}

impl<K, V> Default for Map<K, V>
where
    K: Clone + Eq + Hash + Debug,
    V: Clone + Debug,
{
    fn default() -> Self {
        Map::new()
    }
}
