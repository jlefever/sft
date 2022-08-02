use std::{
    collections::{hash_map, hash_set, HashMap, HashSet},
    iter::FlatMap,
};

struct EdgeSet {
    incoming: HashMap<usize, HashSet<usize>>,
    outgoing: HashMap<usize, HashSet<usize>>,
}

impl EdgeSet {
    pub fn new() -> Self {
        Self {
            incoming: HashMap::new(),
            outgoing: HashMap::new(),
        }
    }

    pub fn add(&mut self, src: usize, tgt: usize) {
        self.incoming.entry(tgt).or_default().insert(src);
        self.outgoing.entry(src).or_default().insert(tgt);
    }

    pub fn incoming(&self, tgt: &usize) -> Option<&HashSet<usize>> {
        self.incoming.get(tgt)
    }

    pub fn outgoing(&self, src: &usize) -> Option<&HashSet<usize>> {
        self.outgoing.get(src)
    }

    fn iter(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.outgoing
            .iter()
            .flat_map(|(src, tgts)| tgts.iter().map(move |tgt| (*src, *tgt)))
    }
}

fn main() {
    let mut edges = EdgeSet::new();
    edges.add(4, 5);
    edges.add(4, 6);
    edges.add(20, 6);

    for t in edges.iter() {
        println!("{:?}", t);
    }
}
