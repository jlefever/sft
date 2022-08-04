use crate::collections::{FactBook, KindedEdgeBag, NodeId, NodeKeeper};
use serde::{Deserialize, Serialize};
use serde_json;
use std::hash::Hash;
use std::io::{self, BufRead};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Ticket {
    pub corpus: Option<String>,
    pub language: Option<String>,
    pub path: Option<String>,
    pub root: Option<String>,
    pub signature: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Entry {
    Edge {
        #[serde(rename = "source")]
        src: Ticket,
        #[serde(rename = "target")]
        tgt: Ticket,
        edge_kind: String,
        fact_name: String,
        fact_value: Option<String>,
    },
    Node {
        #[serde(rename = "source")]
        src: Ticket,
        fact_name: String,
        fact_value: Option<String>,
    },
}

impl Entry {
    pub fn from_json(json: &String) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

pub struct KytheGraph {
    pub nodes: NodeKeeper<Ticket>,
    pub edges: KindedEdgeBag<String, NodeId>,
    pub facts: FactBook<NodeId>,
}

impl KytheGraph {
    pub fn new() -> Self {
        Self {
            nodes: NodeKeeper::new(),
            edges: KindedEdgeBag::new(),
            facts: FactBook::new(),
        }
    }

    pub fn insert_node(&mut self, ticket: Ticket) -> NodeId {
        self.nodes.insert(ticket)
    }

    pub fn insert_edge(&mut self, edge_kind: String, src: NodeId, tgt: NodeId) {
        self.edges.insert(edge_kind, src, tgt);
    }

    pub fn insert_fact(&mut self, id: NodeId, name: String, value: String) {
        self.facts.insert(id, name, value);
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&Ticket> {
        self.nodes.get(id)
    }

    #[allow(dead_code)]
    pub fn get_fact(&self, id: &NodeId, name: &str) -> Option<&String> {
        self.facts.get(id, name)
    }
}

pub fn load_kythe_graph(reader: &mut io::BufReader<Box<dyn io::Read>>) -> KytheGraph {
    let mut buf = String::new();
    let mut graph = KytheGraph::new();

    while let Ok(n_bytes_read) = reader.read_line(&mut buf) {
        if n_bytes_read == 0 {
            break;
        };

        let entry = Entry::from_json(&buf).unwrap();

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

        buf.clear();
    }

    return graph;
}

// pub static KE_REF_CALL: &str = "/kythe/edge/ref/call";
// pub static KE_DEFINES: &str = "/kythe/edge/defines";
// pub static KE_DEFINES_BINDING: &str = "/kythe/edge/defines/binding";
// pub static KE_DEFINES_IMPLICIT: &str = "/kythe/edge/defines/implicit";
// pub static KN_KIND: &str = "/kythe/node/kind";
