use crate::data_structures::{EdgeSet, FactBook, KindedEdgeSet, NodeHolder, NodeId};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    collections::HashSet,
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct TicketId(NodeId);

pub struct KytheGraph<TTicket: Eq + Hash> {
    nodes: NodeHolder<TTicket>,
    edges: KindedEdgeSet<TicketId>,
    facts: FactBook<TicketId>,
}

impl<TTicket: Eq + Hash> KytheGraph<TTicket> {
    pub fn new() -> Self {
        Self {
            nodes: NodeHolder::new(),
            edges: KindedEdgeSet::new(),
            facts: FactBook::new(),
        }
    }

    pub fn add_ticket(&mut self, ticket: TTicket) -> TicketId {
        TicketId(self.nodes.add(ticket))
    }

    pub fn add_edge(&mut self, edge_kind: String, src: TicketId, tgt: TicketId) {
        self.edges.add(edge_kind, src, tgt);
    }

    pub fn add_fact(&mut self, id: TicketId, name: String, value: String) {
        self.facts.add(id, name, value);
    }

    pub fn get_ticket(&self, id: &TicketId) -> Option<&TTicket> {
        self.nodes.get(&id.0)
    }

    pub fn get_fact(&self, id: &TicketId, name: &str) -> Option<&str> {
        self.facts.get(id, name)
    }

    pub fn get_edge_set(&self, edge_kind: &str) -> Option<&EdgeSet<TicketId>> {
        self.edges.get_edge_set(edge_kind)
    }

    pub fn get_all_outgoing(&self, src: &TicketId) -> HashSet<(&str, TicketId, TicketId)> {
        self.edges.all_outgoing(src)
    }
}

pub static KE_REF_CALL: &str = "/kythe/edge/ref/call";
pub static KE_DEFINES: &str = "/kythe/edge/defines";
pub static KE_DEFINES_BINDING: &str = "/kythe/edge/defines/binding";
pub static KE_DEFINES_IMPLICIT: &str = "/kythe/edge/defines/implicit";
pub static KN_KIND: &str = "/kythe/node/kind";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Ticket {
    corpus: Option<String>,
    language: Option<String>,
    path: Option<String>,
    root: Option<String>,
    signature: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct EntryDto {
    #[serde(rename = "source")]
    src: Ticket,
    #[serde(rename = "target")]
    tgt: Option<Ticket>,
    edge_kind: Option<String>,
    fact_name: String,
    fact_value: Option<String>,
}

pub fn load_graph(entries: &Path) -> Option<KytheGraph<Ticket>> {
    let reader = BufReader::new(File::open(entries).ok()?);
    let mut graph = KytheGraph::new();

    for line in reader.lines() {
        let entry: EntryDto = serde_json::from_str(&line.ok()?).ok()?;

        if entry.tgt.is_some() {
            let src_id = graph.add_ticket(entry.src);
            let tgt_id = graph.add_ticket(entry.tgt?);
            graph.add_edge(entry.edge_kind?, src_id, tgt_id);
        } else {
            let src_id = graph.add_ticket(entry.src);
            let fact_name = entry.fact_name;
            let fact_value = entry.fact_value.unwrap_or_default();
            graph.add_fact(src_id, fact_name, fact_value);
        }
    }

    return Some(graph);
}
