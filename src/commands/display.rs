use dot_writer::Attributes;
use dot_writer::DotWriter;

use base64;

use crate::collections::NodeId;
use crate::io::Ticket;
use crate::io::Writer;
use crate::kythe::RawKytheGraph;

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
        let graph = RawKytheGraph::open(input).unwrap();
        log::debug!("Loaded graph in {} secs.", start.elapsed().as_secs_f32());

        let mut output_bytes: Vec<u8> = Vec::new();
        {
            let mut writer = DotWriter::from(&mut output_bytes);
            let mut digraph = writer.digraph();

            for (kind, src, tgt, count) in graph.edges.iter() {
                let kind = kind.strip_prefix("/kythe/edge/").unwrap();

                if kind != "defines/binding" && kind != "ref/call" && kind != "childof" {
                    continue;
                }

                let src_name = String::from(*src);
                let tgt_name = String::from(*tgt);

                let src_label = create_label(&graph, src);
                let tgt_label = create_label(&graph, tgt);

                if src_label.is_none() || tgt_label.is_none() {
                    continue;
                }

                {
                    let mut src_node = digraph.node_named(&src_name);
                    src_node.set_label(&src_label.unwrap());
                }

                {
                    let mut tgt_node = digraph.node_named(&tgt_name);
                    tgt_node.set_label(&tgt_label.unwrap());
                }

                let edge_label = format!("{} ({})", kind, count);
                digraph
                    .edge(&src_name, &tgt_name)
                    .attributes()
                    .set_label(&edge_label);
            }
        }

        writer.write(&output_bytes).unwrap();
    }
}

// /kythe/text
// /kythe/loc/start
// /kythe/loc/end

fn file_of(anchor: &Ticket) -> Ticket {
    Ticket {
        corpus: anchor.corpus.clone(),
        language: None,
        path: anchor.path.clone(),
        root: anchor.root.clone(),
        signature: None,
    }
}

fn create_label(graph: &RawKytheGraph, id: &NodeId) -> Option<String> {
    let ticket = graph.get_node(id)?;
    let kind = graph.get_fact(id, "/kythe/node/kind")?;
    let path = ticket.path.as_ref()?;

    if kind != "anchor" {
        return Some(format!("{} ({}) [{}]", String::from(*id), kind, path));
    }

    let start: usize = graph.get_fact(id, "/kythe/loc/start")?.parse().unwrap();
    let end: usize = graph.get_fact(id, "/kythe/loc/end")?.parse().unwrap();

    let file_id = graph.get_node_id(&file_of(&ticket)).unwrap();
    let text = graph.get_fact(file_id, "/kythe/text").unwrap();
    let mut content = String::from_utf8(text.as_bytes()[start..end].to_vec()).unwrap();
    content = content.replace("//", "\\\\");
    content = content.replace("\"", "'");

    let content: String = content.chars().take_while(|&c| c != '\n').collect();

    return Some(format!(
        "{} ({}) '{}' [{}]",
        String::from(*id),
        kind,
        content,
        path
    ));
}

struct Anchor {
    start: usize,
    end: usize,
}

// fn lookup(graph: &KytheGraph, fact_name: &'static str) -> &String {

// }
