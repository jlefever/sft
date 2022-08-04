use std::{
    collections::HashMap,
    io::{self, BufRead},
};

use crate::{
    collections::{KindedEdgeBag, NodeId, NodeKeeper},
    kythe,
};

pub struct Dv8Graph {
    nodes: NodeKeeper<String>,
    edges: KindedEdgeBag<String, NodeId>,
}

impl Dv8Graph {
    pub fn new() -> Self {
        Self {
            nodes: NodeKeeper::new(),
            edges: KindedEdgeBag::new(),
        }
    }

    pub fn insert_var(&mut self, filename: String) -> NodeId {
        self.nodes.insert(filename)
    }

    pub fn insert_dep(&mut self, edge_kind: String, src: NodeId, tgt: NodeId) {
        self.edges.insert(edge_kind, src, tgt);
    }
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct Dv8Matrix {
    #[serde(rename = "schemaVersion")]
    schema_version: &'static str,

    #[serde(rename = "name")]
    name: Option<String>,

    #[serde(rename = "variables")]
    vars: Vec<String>,

    #[serde(rename = "cells")]
    cells: Vec<Dv8Cell>,
}

impl Dv8Matrix {
    fn new(vars: Vec<String>, cells: Vec<Dv8Cell>) -> Self {
        Self {
            schema_version: "1.0",
            name: None,
            vars,
            cells,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
}

impl From<Dv8Graph> for Dv8Matrix {
    fn from(graph: Dv8Graph) -> Self {
        to_matrix(graph)
    }
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct Dv8Cell {
    #[serde(rename = "src")]
    src: usize,

    #[serde(rename = "dest")]
    tgt: usize,

    #[serde(rename = "values")]
    values: HashMap<&'static str, usize>,
}

impl Dv8Cell {
    fn new(src: usize, tgt: usize, values: HashMap<&'static str, usize>) -> Self {
        Self { src, tgt, values }
    }
}

fn to_vars(keeper: NodeKeeper<String>) -> Vec<String> {
    let mut node_pairs: Vec<(NodeId, String)> = keeper.into_iter().collect();
    node_pairs.sort_by(|&(a_id, _), &(b_id, _)| a_id.cmp(&b_id));

    // Confirm that there are no gaps in node ids
    if let Some(last) = node_pairs.last() {
        assert!(usize::from(last.0) == node_pairs.len() - 1);
    }

    node_pairs.into_iter().map(|(_, node)| node).collect()
}

fn to_dv8_edge_kind(edge_kind: &String) -> Option<&'static str> {
    let edge_kind = edge_kind.strip_prefix("/kythe/edge/")?;

    match edge_kind {
        "ref" => Some("Use"),
        "ref/call" => Some("Call"),
        "ref/call/implicit" => Some("Call"),
        "ref/expands" => Some("Use"),
        "ref/init" => Some("Create"),
        "ref/init/implicit" => Some("Create"),
        "ref/imports" => Some("Import"),
        "ref/id" => Some("Use"),
        "ref/implicit" => Some("Use"),
        "ref/includes" => Some("Include"),
        "ref/queries" => Some("Use"),
        "extends/private" => Some("Extend"),
        "extends/public" => Some("Extend"),
        "overrides" => Some("ImplLink"),
        "overrides/root" => Some("ImplLink"),
        "undefines" => Some("Use"),
        "childof" => Some("Contain"),
        "childof/context" => Some("Contain"),
        "completedby" => Some("Contain"),
        _ => match edge_kind.starts_with("param.") {
            true => Some("Parameter"),
            false => None,
        },
    }
}

fn to_cells(edges: KindedEdgeBag<String, NodeId>, indices: Vec<usize>) -> Vec<Dv8Cell> {
    let mut pair_map: HashMap<(usize, usize), HashMap<&'static str, usize>> = HashMap::new();

    for (kind, &src, &tgt, &count) in edges.iter() {
        let kind = to_dv8_edge_kind(kind);

        if kind.is_none() {
            continue;
        }

        let new_src = *indices.get(usize::from(src)).unwrap();
        let new_tgt = *indices.get(usize::from(tgt)).unwrap();

        pair_map
            .entry((new_src, new_tgt))
            .or_default()
            .insert(kind.unwrap(), count);
    }

    pair_map
        .into_iter()
        .map(|((src, tgt), values)| Dv8Cell::new(src, tgt, values))
        .collect()
}

fn argsort<T: Ord>(data: &[T]) -> Vec<usize> {
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &data[i]);
    indices
}

fn to_matrix(graph: Dv8Graph) -> Dv8Matrix {
    let mut vars = to_vars(graph.nodes);
    let indices = argsort(&vars);
    vars.sort();
    Dv8Matrix::new(vars, to_cells(graph.edges, indices))
}

pub fn load_dv8_graph(reader: &mut io::BufReader<Box<dyn io::Read>>) -> Dv8Graph {
    let mut buf = String::new();
    let mut graph = Dv8Graph::new();

    while let Ok(n_bytes_read) = reader.read_line(&mut buf) {
        if n_bytes_read == 0 {
            break;
        };

        let entry = kythe::Entry::from_json(&buf).unwrap();

        if let kythe::Entry::Edge {
            src,
            tgt,
            edge_kind,
            ..
        } = entry
        {
            if let Some(src_path) = src.path && let Some(tgt_path) = tgt.path {
                let src_id = graph.insert_var(src_path);
                let tgt_id = graph.insert_var(tgt_path);
                graph.insert_dep(edge_kind, src_id, tgt_id);
            }
        }

        buf.clear();
    }

    return graph;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut graph = Dv8Graph::new();
        let tgt = graph.insert_var("src/Provider.java".to_owned());
        let src = graph.insert_var("src/Client.java".to_owned());
        graph.insert_dep("/kythe/edge/ref/call".to_owned(), src, tgt);

        let mut matrix = Dv8Matrix::from(graph);
        matrix.set_name("my-test".to_owned());

        let serialized = serde_json::to_string_pretty(&matrix).unwrap();
        println!("{}", serialized);
    }
}
