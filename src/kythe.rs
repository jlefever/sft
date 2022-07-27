#![allow(dead_code)]
use crate::data_structures::{EdgeSet, FactBook, KindedEdgeSet, NodeHolder, NodeId};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{BufRead, BufWriter, Write};
use std::{
    collections::HashSet,
    hash::Hash,
    io::{BufReader, Read},
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
    pub corpus: Option<String>,
    pub language: Option<String>,
    pub path: Option<String>,
    pub root: Option<String>,
    pub signature: Option<String>,
}

pub fn read_lines<R: Read>(reader: &mut BufReader<R>, read: &mut dyn FnMut(&str) -> ()) {
    let mut buffer = String::new();

    while let Ok(n_bytes_read) = reader.read_line(&mut buffer) {
        if n_bytes_read == 0 {
            break;
        };

        read(&buffer);
        buffer.clear();
    }
}

pub fn filter_lines<R: Read, W: Write>(
    reader: &mut BufReader<R>,
    writer: &mut BufWriter<W>,
    allow_line: &mut dyn FnMut(&str) -> bool,
) {
    read_lines(reader, &mut |line: &str| {
        if allow_line(line) {
            writer.write(line.as_bytes()).unwrap();
        };
    });
}

pub fn load_graph<R: Read>(reader: &mut BufReader<R>) -> KytheGraph<Ticket> {
    let mut graph = KytheGraph::new();

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

    read_lines(reader, &mut |line: &str| {
        let entry: EntryDto = serde_json::from_str(line).unwrap();
        let src_id = graph.add_ticket(entry.src);

        match entry.tgt {
            Some(tgt) => {
                let tgt_id = graph.add_ticket(tgt);
                let edge_kind = entry.edge_kind.unwrap();
                graph.add_edge(edge_kind, src_id, tgt_id);
            }
            None => {
                let fact_name = entry.fact_name;
                let fact_value = entry.fact_value.unwrap_or_default();
                graph.add_fact(src_id, fact_name, fact_value);
            }
        }
    });

    return graph;
}
