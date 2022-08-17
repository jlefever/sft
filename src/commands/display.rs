use dot_writer::Attributes;
use dot_writer::DotWriter;

use crate::io::EntryReader;
use crate::io::Writer;
use crate::kythe::EdgeKind;
use crate::kythe::NodeIndex;
use crate::kythe::RawKGraph;
use crate::kythe::KGraph;

use std::collections::HashSet;
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

        let start = Instant::now();
        let reader = EntryReader::open(input).unwrap();
        let raw_graph = RawKGraph::try_from(reader).unwrap();
        log::debug!("Loaded raw graph in {} secs.", start.elapsed().as_secs_f32());
        let start = Instant::now();
        let graph = KGraph::try_from(raw_graph).unwrap();
        log::debug!("Loaded graph in {} secs.", start.elapsed().as_secs_f32());

        let mut output_bytes: Vec<u8> = Vec::new();
        {
            let mut dot_writer = DotWriter::from(&mut output_bytes);
            let mut digraph = dot_writer.digraph();

            let mut nodes_used = HashSet::new();

            for (kind, src, tgt, count) in graph.iter() {
                match kind {
                    // EdgeKind::Ref => (),
                    EdgeKind::RefCall => (),
                    // EdgeKind::RefCallImplicit => (),
                    // EdgeKind::RefId => (),
                    _ => continue,
                };

                let src = match graph.get_parent(src) {
                    Some(parent) => parent,
                    None => {
                        continue;
                    },
                };

                nodes_used.insert(src);
                nodes_used.insert(tgt);

                digraph
                    .edge(src.0.to_string(), tgt.0.to_string())
                    .attributes()
                    .set_label(&format!("{:?} ({})", kind, count));
            }

            for index in nodes_used {
                let mut node = digraph.node_named(index.0.to_string());
                node.set_label(&create_label(&graph, index));
            }
        }

        writer.write(&output_bytes).unwrap();
    }
}

fn create_label(graph: &KGraph, index: &NodeIndex) -> String {
    let node = graph.get_node(index).unwrap();
    let path = node.file_key.path.as_ref().map(String::as_str).unwrap_or_default();

    let node_str = format!("{:?}", node.kind).replace("\"", "'");

    match graph.get_name_bn(node) {
        Some(name) => format!("{} '{}' ({}) [{}]", index.0, name, node_str, path),
        None => format!("{} ({}) [{}]", index.0, node_str, path),
    }
}