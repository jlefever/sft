use bimap::BiHashMap;
use std::{collections::HashMap, hash::Hash};

#[derive(Copy, Clone, Debug, Default, Ord, Eq, Hash, PartialEq, PartialOrd)]
pub struct ItemId(usize);

impl From<ItemId> for usize {
    fn from(id: ItemId) -> Self {
        id.0
    }
}

impl From<ItemId> for String {
    fn from(id: ItemId) -> Self {
        id.0.to_string()
    }
}

pub struct IdMap<N> {
    nodes: BiHashMap<ItemId, N>,
}

impl<T: Eq + Hash> IdMap<T> {
    pub fn new() -> Self {
        Self {
            nodes: BiHashMap::new(),
        }
    }

    pub fn insert(&mut self, node: T) -> ItemId {
        if let Some(node_id) = self.nodes.get_by_right(&node) {
            *node_id
        } else {
            let node_id = ItemId(self.nodes.len());
            self.nodes.insert(node_id, node);
            node_id
        }
    }

    pub fn get_item(&self, id: &ItemId) -> Option<&T> {
        self.nodes.get_by_left(id)
    }

    pub fn get_id(&self, node: &T) -> Option<&ItemId> {
        self.nodes.get_by_right(node)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ItemId, &T)> {
        self.nodes.iter()
    }
}

impl<'a, N: Eq + Hash> IntoIterator for &'a IdMap<N> {
    type Item = (&'a ItemId, &'a N);
    type IntoIter = bimap::hash::Iter<'a, ItemId, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl<N: Eq + Hash> IntoIterator for IdMap<N> {
    type Item = (ItemId, N);
    type IntoIter = bimap::hash::IntoIter<ItemId, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct EdgeBag<N> {
    outgoing: HashMap<N, HashMap<N, usize>>,
    incoming: HashMap<N, HashMap<N, usize>>,
}

impl<N: Copy + Eq + Hash> EdgeBag<N> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        }
    }

    pub fn insert(&mut self, src: N, tgt: N) -> usize {
        // Outgoing
        let inner = self.outgoing.entry(src).or_default();
        let count = inner.entry(tgt).or_default();
        *count += 1;

        // Incoming
        let inner = self.incoming.entry(tgt).or_default();
        let count = inner.entry(src).or_default();
        *count += 1;

        *count
    }

    pub fn outgoing(&self, src: &N) -> impl Iterator<Item = (N, usize)> + '_ {
        self.outgoing
            .get(src)
            .map(|inner| inner.iter().map(move |(tgt, count)| (*tgt, *count)))
            .into_iter()
            .flatten()
    }

    pub fn incoming(&self, tgt: &N) -> impl Iterator<Item = (N, usize)> + '_ {
        self.incoming
            .get(tgt)
            .map(|inner| inner.iter().map(move |(src, count)| (*src, *count)))
            .into_iter()
            .flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item = (N, N, usize)> + '_ {
        self.outgoing
            .iter()
            .flat_map(|(src, tgts)| tgts.iter().map(move |(tgt, count)| (*src, *tgt, *count)))
    }
}

#[derive(Debug, Default)]
pub struct KindedEdgeBag<K, N> {
    bags: HashMap<K, EdgeBag<N>>,
}

impl<K, N> KindedEdgeBag<K, N>
where
    K: Copy + Eq + Hash,
    N: Copy + Default + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            bags: HashMap::new(),
        }
    }

    pub fn insert(&mut self, kind: K, src: N, tgt: N) -> usize {
        self.bags.entry(kind).or_default().insert(src, tgt)
    }

    pub fn outgoing(&self, kind: &K, src: &N) -> impl Iterator<Item = (N, usize)> + '_ {
        self.bags.get(&kind).map(|m| m.outgoing(src)).into_iter().flatten()
    }

    pub fn incoming(&self, kind: &K, tgt: &N) -> impl Iterator<Item = (N, usize)> + '_ {
        self.bags.get(&kind).map(|m| m.incoming(tgt)).into_iter().flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, N, N, usize)> + '_ {
        self.bags.iter().flat_map(|(kind, edge_set)| {
            edge_set
                .iter()
                .map(move |(src, tgt, count)| (*kind, src, tgt, count))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test() {
        let mut bag: EdgeBag<usize> = EdgeBag::new();

        bag.insert(3, 4);

        let set: HashSet<(usize, usize, usize)> = bag.iter().collect();
        assert!(set.contains(&(3, 4, 1)));
    }
}
