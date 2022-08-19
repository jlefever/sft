// use std::{collections::HashMap, io, path::Path};

// use crate::{
//     collections::{KindedEdgeBag, ItemId, IdMap},
//     io::{Entry, EntryReader},
// };

// pub struct Dv8Graph {
//     nodes: IdMap<String>,
//     edges: KindedEdgeBag<String, ItemId>,
// }

// impl Dv8Graph {
//     pub fn new() -> Self {
//         Self {
//             nodes: IdMap::new(),
//             edges: KindedEdgeBag::new(),
//         }
//     }

//     pub fn open(path: Option<&Path>) -> io::Result<Self> {
//         Ok(Self::from(EntryReader::open(path)?))
//     }

//     pub fn insert_var(&mut self, filename: String) -> ItemId {
//         self.nodes.insert(filename)
//     }

//     pub fn insert_dep(&mut self, edge_kind: String, src: ItemId, tgt: ItemId) {
//         self.edges.insert(edge_kind, src, tgt);
//     }
// }

// impl From<EntryReader> for Dv8Graph {
//     fn from(reader: EntryReader) -> Self {
//         let mut graph = Dv8Graph::new();

//         for entry in reader {
//             match entry {
//                 Entry::Edge { src, tgt, edge_kind, .. } => {
//                     if let Some(src_path) = src.path && let Some(tgt_path) = tgt.path {
//                         let src_id = graph.insert_var(src_path);
//                         let tgt_id = graph.insert_var(tgt_path);
//                         graph.insert_dep(edge_kind, src_id, tgt_id);
//                     }
//                 },
//                 _ => ()
//             }
//         }

//         return graph;
//     }
// }

// #[derive(serde::Serialize, Debug, PartialEq, Eq)]
// pub struct Dv8Matrix {
//     #[serde(rename = "schemaVersion")]
//     schema_version: &'static str,

//     #[serde(rename = "name")]
//     name: Option<String>,

//     #[serde(rename = "variables")]
//     vars: Vec<String>,

//     #[serde(rename = "cells")]
//     cells: Vec<Dv8Cell>,
// }

// impl Dv8Matrix {
//     fn new(vars: Vec<String>, cells: Vec<Dv8Cell>) -> Self {
//         Self {
//             schema_version: "1.0",
//             name: None,
//             vars,
//             cells,
//         }
//     }

//     pub fn set_name(&mut self, name: String) {
//         self.name = Some(name);
//     }
// }

// impl From<Dv8Graph> for Dv8Matrix {
//     fn from(graph: Dv8Graph) -> Self {
//         to_matrix(graph)
//     }
// }

// #[derive(serde::Serialize, Debug, PartialEq, Eq)]
// pub struct Dv8Cell {
//     #[serde(rename = "src")]
//     src: usize,

//     #[serde(rename = "dest")]
//     tgt: usize,

//     #[serde(rename = "values")]
//     values: HashMap<&'static str, usize>,
// }

// impl Dv8Cell {
//     fn new(src: usize, tgt: usize, values: HashMap<&'static str, usize>) -> Self {
//         Self { src, tgt, values }
//     }
// }

// fn to_vars(keeper: IdMap<String>) -> Vec<String> {
//     let mut node_pairs: Vec<(ItemId, String)> = keeper.into_iter().collect();
//     node_pairs.sort_by(|&(a_id, _), &(b_id, _)| a_id.cmp(&b_id));

//     // Confirm that there are no gaps in node ids
//     if let Some(last) = node_pairs.last() {
//         assert!(usize::from(last.0) == node_pairs.len() - 1);
//     }

//     node_pairs.into_iter().map(|(_, node)| node).collect()
// }

// fn to_dv8_edge_kind(edge_kind: &String) -> Option<&'static str> {
//     let edge_kind = edge_kind.strip_prefix("/kythe/edge/")?;

//     match edge_kind {
//         "ref" => Some("Use"),
//         "ref/call" => Some("Call"),
//         "ref/call/implicit" => Some("Call"),
//         "ref/expands" => Some("Use"),
//         "ref/init" => Some("Create"),
//         "ref/init/implicit" => Some("Create"),
//         "ref/imports" => Some("Import"),
//         "ref/id" => Some("Use"),
//         "ref/implicit" => Some("Use"),
//         "ref/includes" => Some("Include"),
//         "ref/queries" => Some("Use"),
//         "extends/private" => Some("Extend"),
//         "extends/public" => Some("Extend"),
//         "overrides" => Some("ImplLink"),
//         "overrides/root" => Some("ImplLink"),
//         "undefines" => Some("Use"),
//         "satisfies" => Some("Implement"),
//         "extends" => Some("Extend"),
//         "childof" => Some("Contain"),
//         "childof/context" => Some("Contain"),
//         // "completedby" => Some("Contain"),
//         _ => match edge_kind.starts_with("param.") {
//             true => Some("Parameter"),
//             false => None,
//         },
//     }
// }

// fn to_cells(edges: KindedEdgeBag<String, ItemId>, indices: Vec<usize>) -> Vec<Dv8Cell> {
//     let mut pair_map: HashMap<(usize, usize), HashMap<&'static str, usize>> = HashMap::new();

//     for (kind, &src, &tgt, &count) in edges.iter() {
//         let kind = to_dv8_edge_kind(kind);

//         if kind.is_none() {
//             continue;
//         }

//         let new_src = *indices.get(usize::from(src)).unwrap();
//         let new_tgt = *indices.get(usize::from(tgt)).unwrap();

//         pair_map
//             .entry((new_src, new_tgt))
//             .or_default()
//             .insert(kind.unwrap(), count);
//     }

//     pair_map
//         .into_iter()
//         .map(|((src, tgt), values)| Dv8Cell::new(src, tgt, values))
//         .collect()
// }

// fn argsort<T: Ord>(data: &[T]) -> Vec<usize> {
//     let mut indices = (0..data.len()).collect::<Vec<_>>();
//     indices.sort_by_key(|&i| &data[i]);
//     indices
// }

// fn to_matrix(graph: Dv8Graph) -> Dv8Matrix {
//     let mut vars = to_vars(graph.nodes);
//     let indices = argsort(&vars);
//     vars.sort();
//     Dv8Matrix::new(vars, to_cells(graph.edges, indices))
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test() {
//         let mut graph = Dv8Graph::new();
//         let tgt = graph.insert_var("src/Provider.java".to_owned());
//         let src = graph.insert_var("src/Client.java".to_owned());
//         graph.insert_dep("/kythe/edge/ref/call".to_owned(), src, tgt);

//         let mut matrix = Dv8Matrix::from(graph);
//         matrix.set_name("my-test".to_owned());

//         let serialized = serde_json::to_string_pretty(&matrix).unwrap();
//         println!("{}", serialized);
//     }
// }
