use bimap::BiHashMap;
use std::{collections::HashMap, hash::Hash};

#[derive(Copy, Clone, Debug, Default, Ord, Eq, Hash, PartialEq, PartialOrd)]
pub struct NodeId(usize);

impl From<NodeId> for usize {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl From<NodeId> for String {
    fn from(id: NodeId) -> Self {
        id.0.to_string()
    }
}

pub struct NodeKeeper<N> {
    nodes: BiHashMap<NodeId, N>,
}

impl<N: Eq + Hash> NodeKeeper<N> {
    pub fn new() -> Self {
        Self {
            nodes: BiHashMap::new(),
        }
    }

    pub fn insert(&mut self, node: N) -> NodeId {
        if let Some(node_id) = self.nodes.get_by_right(&node) {
            *node_id
        } else {
            let node_id = NodeId(self.nodes.len());
            self.nodes.insert(node_id, node);
            node_id
        }
    }

    pub fn get(&self, id: &NodeId) -> Option<&N> {
        self.nodes.get_by_left(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &N)> {
        self.nodes.iter()
    }
}

impl<'a, N: Eq + Hash> IntoIterator for &'a NodeKeeper<N> {
    type Item = (&'a NodeId, &'a N);
    type IntoIter = bimap::hash::Iter<'a, NodeId, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl<N: Eq + Hash> IntoIterator for NodeKeeper<N> {
    type Item = (NodeId, N);
    type IntoIter = bimap::hash::IntoIter<NodeId, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct EdgeBag<N> {
    outgoing: HashMap<N, HashMap<N, usize>>,
}

impl<N: Eq + Hash> EdgeBag<N> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            outgoing: HashMap::new(),
        }
    }

    pub fn insert(&mut self, src: N, tgt: N) -> usize {
        let inner = self.outgoing.entry(src).or_default();
        let count = inner.entry(tgt).or_default();
        *count += 1;
        *count
    }

    pub fn iter(&self) -> impl Iterator<Item = (&N, &N, &usize)> + '_ {
        self.outgoing
            .iter()
            .flat_map(|(src, tgts)| tgts.iter().map(move |(tgt, count)| (src, tgt, count)))
    }
}

#[derive(Debug, Default)]
pub struct KindedEdgeBag<K, N> {
    bags: HashMap<K, EdgeBag<N>>,
}

impl<K, N> KindedEdgeBag<K, N>
where
    K: Default + Eq + Hash,
    N: Default + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            bags: HashMap::new(),
        }
    }

    pub fn insert(&mut self, kind: K, src: N, tgt: N) -> usize {
        self.bags.entry(kind).or_default().insert(src, tgt)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &N, &N, &usize)> + '_ {
        self.bags.iter().flat_map(|(kind, edge_set)| {
            edge_set
                .iter()
                .map(move |(src, tgt, count)| (kind, src, tgt, count))
        })
    }
}

pub struct FactBook<N> {
    facts: HashMap<String, HashMap<N, String>>,
}

impl<N: Eq + Hash> FactBook<N> {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }

    pub fn insert(&mut self, node: N, name: String, value: String) {
        self.facts.entry(name).or_default().insert(node, value);
    }

    pub fn get(&self, node: &N, name: &str) -> Option<&String> {
        self.facts.get(name)?.get(node)
    }
}

// Idea: Every pair (src, tgt) gets an EdgeData
// The exact type of EdgeData is specified by the client.
// Client can use it for count, (kind, count), etc.
// EdgeFreq is a type of EdgeData
// EdgeFreqByKind is another type of EdgeData

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test() {
        let mut bag: EdgeBag<usize> = EdgeBag::new();

        bag.insert(3, 4);

        let set: HashSet<(&usize, &usize, &usize)> = bag.iter().collect();
        assert!(set.contains(&(&3, &4, &1)));
    }
}
