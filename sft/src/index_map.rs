use crate::prelude::*;

use std::hash::Hash;

pub struct IndexMap<T: Eq + Hash> {
    inner: bimap::BiMap<T, Idx>,
}

impl<T: Eq + Hash> IndexMap<T> {
    pub fn new() -> Self {
        Self { inner: bimap::BiMap::new() }
    }

    pub fn from<I: IntoIterator<Item = T>>(items: I) -> Self {
        let mut index_map = Self::new();

        for item in items {
            index_map.put(item);
        }

        index_map
    }

    pub fn put(&mut self, item: T) -> Idx {
        let idx = self.inner.len();

        match self.inner.insert_no_overwrite(item, idx) {
            Ok(_) => idx,
            Err((_, idx)) => idx,
        }
    }

    pub fn get(&self, idx: Idx) -> Option<&T> {
        self.inner.get_by_right(&idx)
    }

    pub fn get_idx(&self, item: &T) -> Option<Idx> {
        self.inner.get_by_left(item).cloned()
    }

    pub fn map<'a, I: IntoIterator<Item = &'a T> + 'a>(
        &'a self,
        iter: I,
    ) -> impl Iterator<Item = Idx> + '_ {
        iter.into_iter().map(|item| self.get_idx(&item).unwrap())
    }

    pub fn map_pairs<'a, I: IntoIterator<Item = (&'a T, &'a T)> + 'a>(
        &'a self,
        pairs: I,
    ) -> impl Iterator<Item = (Idx, Idx)> + '_ {
        pairs.into_iter().map(|(a, b)| (self.get_idx(&a).unwrap(), self.get_idx(&b).unwrap()))
    }

    pub fn map_indices<'a, I: IntoIterator<Item = Idx> + 'a>(
        &'a self,
        iter: I,
    ) -> impl Iterator<Item = &'a T> + '_ {
        iter.into_iter().map(|idx| self.get(idx).unwrap())
    }
}
