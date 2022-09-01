use itertools::Itertools;

use crate::io::{open_bufwriter, EntryReader};
use crate::ir::{AnchorKind, EdgeKind, NodeIndex, NodeKind, RawGraph, SpecGraph};

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use tabled::{Style, Table, Tabled};

use super::CliCommand;

/// Produce a table of edge kinds and frequencies
///
/// The n(*) columns are produced using the following steps:
///     - Place each edge with the same (source_kind, edge_kind, target_kind) in
///       the same bucket
///     - Group the edges of each bucket according to either its source or
///       target
///     - Count the size of each group
///     - Count the number of times each group size occurs
///
/// So n(2)=3 means a group size of 2 occured 3 times. If we passed "--count-by
/// source" in, this means there are 3 nodes with the same node kind and each
/// one has 2 outgoing edges all with the same edge kind and target kind.
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than
/// stdin/stdout for both performance reasons and compatibility reasons (Windows
/// console does not support UTF-8).
#[derive(clap::Args)]
#[clap(verbatim_doc_comment)]
pub struct CliEdgeKindsCommand {
    /// Path of the file to read entries from. If ommitted, read from stdin.
    #[clap(short = 'i', value_name = "PATH", long, display_order = 1)]
    input: Option<PathBuf>,
    /// Path of the file to write to. If ommitted, write to stdout.
    #[clap(short = 'o', value_name = "PATH", long, display_order = 2)]
    output: Option<PathBuf>,
    /// Group edges by this endpoint, then count.
    #[clap(short = 'c', value_name = "ENDPOINT", long, arg_enum, value_parser)]
    count_by: CountBy,
}

#[derive(Clone, clap::ValueEnum)]
pub enum CountBy {
    Source,
    Target,
}

impl CliCommand for CliEdgeKindsCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        // Load graph
        let reader = EntryReader::open(self.input.clone())?;
        let raw_graph = RawGraph::try_from(reader)?;
        let graph = SpecGraph::try_from(raw_graph)?;

        // Select count by
        let count_by = match self.count_by {
            CountBy::Source => |e: &Edge| e.src,
            CountBy::Target => |e: &Edge| e.tgt,
        };

        // Map edges into an `Edge` set
        let edges: HashSet<Edge> = graph.iter().map_into().collect();

        // Group edges by their `TotalEdgeKind`
        let edges: HashMap<TotalEdgeKind, Vec<Edge>> =
            edges.into_iter().into_group_map_by(|e| TotalEdgeKind::from_graph_edge(&graph, e));

        // Subgroup the edges of each group according to their source or target, then
        // count the size of each subgroup. Store these counts as `Vec<usize>`.
        let edges: HashMap<TotalEdgeKind, Vec<usize>> = edges
            .into_iter()
            .map(|(kind, edges)| {
                (
                    kind,
                    edges
                        .into_iter()
                        .into_group_map_by(count_by)
                        .into_values()
                        .map(|edges| edges.len())
                        .collect_vec(),
                )
            })
            .collect();

        // Count the number of times each subgroup size occurs
        let edges: HashMap<TotalEdgeKind, HashMap<usize, usize>> =
            edges.into_iter().map(|(kind, edges)| (kind, edges.into_iter().counts())).collect();

        // Map these counts to a table and write it out
        let mut rows: Vec<Row> = edges.into_iter().map(Row::from_pair).collect();
        rows.sort();
        let table = Table::new(rows).with(Style::psql()).to_string();
        open_bufwriter(self.output.clone())?.write_all(table.as_bytes())?;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Edge {
    src: NodeIndex,
    tgt: NodeIndex,
    kind: EdgeKind,
}

impl Edge {
    fn new(src: NodeIndex, tgt: NodeIndex, kind: EdgeKind) -> Self {
        Self { src, tgt, kind }
    }

    fn from_quad(quad: (EdgeKind, NodeIndex, NodeIndex, usize)) -> Self {
        let (kind, src, tgt, count) = quad;

        if count != 1 {
            log::warn!("found edge ({:?}) with non-singular count", quad);
        }

        Self::new(src, tgt, kind)
    }
}

impl From<(EdgeKind, NodeIndex, NodeIndex, usize)> for Edge {
    fn from(quad: (EdgeKind, NodeIndex, NodeIndex, usize)) -> Self {
        Self::from_quad(quad)
    }
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TotalEdgeKind {
    src: String,
    tgt: String,
    edge: String,
}

impl TotalEdgeKind {
    fn new(src: String, tgt: String, edge: String) -> Self {
        Self { src, tgt, edge }
    }

    fn from_graph_edge(graph: &SpecGraph, edge: &Edge) -> Self {
        let src = to_nodekind_str(&graph.get_node(edge.src).kind);
        let tgt = to_nodekind_str(&graph.get_node(edge.tgt).kind);
        let edge = to_edgekind_str(&edge.kind);

        Self::new(src, tgt, edge)
    }
}

fn to_nodekind_str(kind: &NodeKind) -> String {
    match kind {
        NodeKind::Anchor(AnchorKind::Explicit(_)) => "Anchor(Explicit(...))".to_owned(),
        NodeKind::Constant(_) => "Constant(...)".to_owned(),
        NodeKind::Doc(_) => "Doc(...)".to_owned(),
        NodeKind::File(_) => "File(...)".to_owned(),
        NodeKind::Lookup(_) => "Lookup(...)".to_owned(),
        _ => format!("{:?}", kind),
    }
}

fn to_edgekind_str(kind: &EdgeKind) -> String {
    match kind {
        EdgeKind::Param(_) => "Param(...)".to_owned(),
        _ => format!("{:?}", kind),
    }
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Tabled)]
struct Row {
    #[tabled(rename = "Source Kind")]
    src: String,

    #[tabled(rename = "Edge Kind")]
    edge: String,

    #[tabled(rename = "Target Kind")]
    tgt: String,

    #[tabled(rename = "n(1)")]
    n_1: usize,

    #[tabled(rename = "n(2)")]
    n_2: usize,

    #[tabled(rename = "n(3)")]
    n_3: usize,

    #[tabled(rename = "n(4)")]
    n_4: usize,

    #[tabled(rename = "n(5+)")]
    n_5_plus: usize,
}

impl Row {
    fn from_pair((kind, counts): (TotalEdgeKind, HashMap<usize, usize>)) -> Self {
        let mut row = Self {
            src: kind.src,
            tgt: kind.tgt,
            edge: kind.edge,
            n_1: 0,
            n_2: 0,
            n_3: 0,
            n_4: 0,
            n_5_plus: 0,
        };

        for (n, count) in counts {
            match n {
                0 => panic!(),
                1 => row.n_1 += count,
                2 => row.n_2 += count,
                3 => row.n_3 += count,
                4 => row.n_4 += count,
                _ => row.n_5_plus += count,
            }
        }

        row
    }
}
