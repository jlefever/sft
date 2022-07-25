use bimap::BiHashMap;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Default)]
pub struct EdgeSet<TNodeId: Copy + Default + Eq + Hash> {
    incoming: HashMap<TNodeId, HashSet<TNodeId>>,
    outgoing: HashMap<TNodeId, HashSet<TNodeId>>,
}

impl<TNodeId: Copy + Default + Eq + Hash> EdgeSet<TNodeId> {
    pub fn new() -> Self {
        Self {
            incoming: HashMap::new(),
            outgoing: HashMap::new(),
        }
    }

    pub fn add(&mut self, src: TNodeId, tgt: TNodeId) {
        self.incoming.entry(tgt).or_default().insert(src);
        self.outgoing.entry(src).or_default().insert(tgt);
    }

    pub fn incoming(&self, tgt: &TNodeId) -> Option<&HashSet<TNodeId>> {
        self.incoming.get(tgt)
    }

    pub fn outgoing(&self, src: &TNodeId) -> Option<&HashSet<TNodeId>> {
        self.outgoing.get(src)
    }

    pub fn to_pairs(&self) -> Vec<(TNodeId, TNodeId)> {
        let mut pairs = Vec::new();

        for (src, outgoing) in &self.outgoing {
            for tgt in outgoing {
                pairs.push((*src, *tgt));
            }
        }

        return pairs;
    }

    pub fn contains(&self, src: &TNodeId, tgt: &TNodeId) -> bool {
        self.outgoing(src)
            .map(|x| x.contains(tgt))
            .unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        let mut total = 0;

        for (_, map) in &self.incoming {
            total += map.len();
        }

        return total;
    }
}

pub struct KindedEdgeSet<TNodeId: Copy + Default + Eq + Hash> {
    edge_sets: HashMap<String, EdgeSet<TNodeId>>,
}

impl<TNodeId: Copy + Default + Eq + Hash> KindedEdgeSet<TNodeId> {
    pub fn new() -> Self {
        Self {
            edge_sets: HashMap::new(),
        }
    }

    pub fn add(&mut self, edge_kind: String, src: TNodeId, tgt: TNodeId) {
        self.edge_sets.entry(edge_kind).or_default().add(src, tgt);
    }

    pub fn incoming(&self, edge_kind: &str, tgt: &TNodeId) -> Option<&HashSet<TNodeId>> {
        self.edge_sets.get(edge_kind).and_then(|x| x.incoming(tgt))
    }

    pub fn outgoing(&self, edge_kind: &str, src: &TNodeId) -> Option<&HashSet<TNodeId>> {
        self.edge_sets.get(edge_kind).and_then(|x| x.outgoing(src))
    }

    pub fn all_outgoing(&self, src: &TNodeId) -> HashSet<(&str, TNodeId, TNodeId)> {
        let mut out = HashSet::new();

        for (edge_kind, edge_set) in &self.edge_sets {
            if let Some(outgoing) = edge_set.outgoing(src) {
                for tgt in outgoing {
                    out.insert((edge_kind.as_str(), *src, *tgt));
                }
            }
        }

        return out;
    }

    pub fn contains(&self, edge_kind: &str, src: &TNodeId, tgt: &TNodeId) -> bool {
        self.edge_sets
            .get(edge_kind)
            .map(|x| x.contains(src, tgt))
            .unwrap_or_default()
    }

    pub fn get_edge_set(&self, edge_kind: &str) -> Option<&EdgeSet<TNodeId>> {
        return self.edge_sets.get(edge_kind)
    }

    pub fn between(&self, src: &TNodeId, tgt: &TNodeId) -> HashSet<&str> {
        let mut out = HashSet::new();
        for (edge_kind, edge_set) in &self.edge_sets {
            if edge_set.contains(src, tgt) {
                out.insert(edge_kind.as_str());
            }
        }
        return out;
    }
}

pub struct FactBook<TNodeId: Eq + Hash> {
    facts: HashMap<String, HashMap<TNodeId, String>>,
}

impl<TNodeId: Eq + Hash> FactBook<TNodeId> {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }

    pub fn add(&mut self, node: TNodeId, name: String, value: String) {
        self.facts.entry(name).or_default().insert(node, value);
    }

    pub fn get(&self, node: &TNodeId, name: &str) -> Option<&str> {
        self.facts.get(name)?.get(node).map(|x| x.as_str())
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct NodeId(usize);

pub struct NodeHolder<TNode: Eq + Hash> {
    nodes: BiHashMap<TNode, NodeId>,
}

impl<TNode: Eq + Hash> NodeHolder<TNode> {
    pub fn new() -> Self {
        Self {
            nodes: BiHashMap::new(),
        }
    }

    pub fn add(&mut self, node: TNode) -> NodeId {
        if let Some(node_id) = self.nodes.get_by_left(&node) {
            *node_id
        } else {
            let node_id = NodeId(self.nodes.len());
            self.nodes.insert(node, node_id);
            node_id
        }
    }

    pub fn get(&self, id: &NodeId) -> Option<&TNode> {
        self.nodes.get_by_right(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_set() {
        let mut set = EdgeSet::new();
        set.add(4, 5);
        set.add(4, 6);
        set.add(20, 6);

        assert!(set.incoming(&5).unwrap().contains(&4));
        assert!(set.incoming(&5).unwrap().len() == 1);

        assert!(set.incoming(&6).unwrap().contains(&4));
        assert!(set.incoming(&6).unwrap().contains(&20));
        assert!(set.incoming(&6).unwrap().len() == 2);

        assert!(set.incoming(&4).is_none());
        assert!(set.incoming(&2).is_none());

        assert!(set.outgoing(&4).unwrap().contains(&5));
        assert!(set.outgoing(&4).unwrap().contains(&6));
        assert!(set.outgoing(&4).unwrap().len() == 2);

        assert!(set.outgoing(&20).unwrap().contains(&6));
        assert!(set.outgoing(&20).unwrap().len() == 1);

        assert!(set.outgoing(&6).is_none());
        assert!(set.outgoing(&2).is_none());

        assert!(set.contains(&4, &5));
        assert!(!set.contains(&4, &7));
    }

    #[test]
    fn test_kinded_edge_set() {
        let mut set = KindedEdgeSet::new();
        set.add("call".to_string(), 4, 5);
        set.add("call".to_string(), 4, 6);
        set.add("bind".to_string(), 4, 5);

        assert!(set.incoming("call", &5).unwrap().contains(&4));
        assert!(set.incoming("call", &5).unwrap().len() == 1);

        assert!(set.incoming("bind", &5).unwrap().contains(&4));
        assert!(set.incoming("bind", &5).unwrap().len() == 1);

        assert!(set.outgoing("call", &4).unwrap().contains(&5));
        assert!(set.outgoing("call", &4).unwrap().contains(&6));
        assert!(set.outgoing("call", &4).unwrap().len() == 2);

        assert!(set.outgoing("bind", &4).unwrap().contains(&5));
        assert!(set.outgoing("bind", &4).unwrap().len() == 1);

        assert!(!set.outgoing("bind", &4).unwrap().contains(&6));

        assert!(set.contains("call", &4, &6));
        assert!(!set.contains("bind", &4, &6));

        assert!(set.between(&4, &5).contains("call"));
        assert!(set.between(&4, &5).contains("bind"));
        assert!(set.between(&4, &5).len() == 2);
    }
}
