use std::path::Path;

use crate::collections::{FactBook, KindedEdgeBag, NodeId, NodeKeeper};
use crate::io::{Entry, EntryReader, Ticket};

pub struct RawKytheGraph {
    pub nodes: NodeKeeper<Ticket>,
    pub edges: KindedEdgeBag<String, NodeId>,
    pub facts: FactBook<NodeId>,
}

impl RawKytheGraph {
    pub fn new() -> Self {
        Self {
            nodes: NodeKeeper::new(),
            edges: KindedEdgeBag::new(),
            facts: FactBook::new(),
        }
    }

    pub fn open(path: Option<&Path>) -> std::io::Result<Self> {
        Ok(Self::from(EntryReader::open(path)?))
    }

    pub fn insert_node(&mut self, ticket: Ticket) -> NodeId {
        self.nodes.insert(ticket)
    }

    pub fn insert_edge(&mut self, edge_kind: String, src: NodeId, tgt: NodeId) {
        self.edges.insert(edge_kind, src, tgt);
    }

    pub fn insert_fact(&mut self, id: NodeId, name: String, value: String) {
        let decoded = base64::decode(value).unwrap();
        let value = String::from_utf8_lossy(&decoded).to_string();
        self.facts.insert(id, name, value);
    }

    pub fn get_node_id(&self, ticket: &Ticket) -> Option<&NodeId> {
        self.nodes.get_id(ticket)
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Ticket> {
        self.nodes.get_node(id)
    }

    pub fn get_fact(&self, id: &NodeId, name: &str) -> Option<&String> {
        self.facts.get(id, name)
    }
}

impl From<EntryReader> for RawKytheGraph {
    fn from(reader: EntryReader) -> Self {
        let mut graph = RawKytheGraph::new();

        for entry in reader {
            match entry {
                Entry::Edge {
                    src,
                    tgt,
                    edge_kind,
                    ..
                } => {
                    let src_id = graph.insert_node(src);
                    let tgt_id = graph.insert_node(tgt);
                    graph.insert_edge(edge_kind, src_id, tgt_id);
                }
                Entry::Node {
                    src,
                    fact_name,
                    fact_value,
                } => {
                    let src_id = graph.insert_node(src);
                    graph.insert_fact(src_id, fact_name, fact_value.unwrap_or_default());
                }
            }
        }

        return graph;
    }
}
