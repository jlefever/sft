use dot_writer::{Attributes, DotWriter};

use crate::io::{EntryReader, Writer};
use crate::ir::{Dep, Entity, EntityGraph, SpecGraph, RawGraph, NodeKind};

use std::path::PathBuf;
use std::time::Instant;

use super::CliCommand;

/// Produce a DOT file that can be rendered with Graphviz.
///
/// Reads a stream of newline-delimited entries in and writes out a DOT file. It
/// is recommended to use the `exclude` subcommand to filter down the graph to a
/// legible size.
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than
/// stdin/stdout for both performance reasons and compatibility reasons (Windows
/// console does not support UTF-8).
#[derive(clap::Args)]
pub struct CliDisplayCommand {
    /// Path of the file to read entries from. If ommitted, read from stdin.
    #[clap(short = 'i', value_name = "PATH", long, display_order = 1)]
    input: Option<PathBuf>,
    /// Path of the file to write DOT file to. If ommitted, write to stdout.
    #[clap(short = 'o', value_name = "PATH", long, display_order = 2)]
    output: Option<PathBuf>,
}

impl CliCommand for CliDisplayCommand {
    fn execute(&self) {
        let input = self.input.as_ref().map(PathBuf::as_path);
        let output = self.output.as_ref().map(PathBuf::as_path);
        let mut writer = Writer::open(output).unwrap();

        // Load graph
        let start = Instant::now();
        let reader = EntryReader::open(input).unwrap();
        let graph = RawGraph::try_from(reader).unwrap();
        log::debug!("Loaded raw graph in {} secs.", start.elapsed().as_secs_f32());
        let start = Instant::now();
        let graph = SpecGraph::try_from(graph).unwrap();
        log::debug!("Loaded spec graph in {} secs.", start.elapsed().as_secs_f32());
        let graph = EntityGraph::try_from(graph).unwrap();

        println!("{:#?}", graph);

        // Setup graphviz stuff
        let mut output_bytes: Vec<u8> = Vec::new();
        {
            let mut dot_writer = DotWriter::from(&mut output_bytes);
            let mut digraph = dot_writer.digraph();
    
            // Add nodes to DOT graph
            for entity in graph.entities.values() {
                let mut node = digraph.node_named(entity.id.to_string());
                node.set_label(&to_node_label(entity));
            }
    
            // Add edges to DOT graph
            for dep in &graph.deps {
                let edge = digraph.edge(dep.src.to_string(), dep.tgt.to_string());
                edge.attributes().set_label(&to_edge_label(dep));
            }
        }

        // Write output
        writer.write(&output_bytes).unwrap();
    }
}

fn clean(text: String) -> String {
    text.replace("\"", "'")
}

fn to_node_label(entity: &Entity) -> String {
    let kind = match &entity.kind {
        NodeKind::Doc(text) => NodeKind::Doc(text[..18].to_string()),
        NodeKind::File(text) => NodeKind::File(text[..18].to_string()),
        kind => kind.clone(),
    };
    
    clean(format!("{}\n<{:?}>", entity.name, kind))
}

fn to_edge_label(dep: &Dep) -> String {
    clean(format!("{:?} ({})", dep.kind, dep.count))
}
