use dot_writer::Attributes;
use dot_writer::DotWriter;

use crate::kythe;
use crate::util;

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use super::CliCommand;

/// Produce a DOT file that can be rendered with Graphviz.
///
/// Reads a stream of newline-delimited entries in and writes out a DOT file. It is recommended
/// to use the `exclude` subcommand to filter down the graph to a legible size.
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than stdin/stdout for both
/// performance reasons and compatibility reasons (Windows console does not support UTF-8).
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
        let mut input = util::create_input(self.input.as_ref()).unwrap();
        let mut output = util::create_output(self.output.as_ref()).unwrap();

        let start = Instant::now();
        let graph = kythe::load_graph(&mut input);
        log::debug!("Loaded graph in {} secs.", start.elapsed().as_secs_f32());

        let mut output_bytes: Vec<u8> = Vec::new();
        {
            let mut writer = DotWriter::from(&mut output_bytes);
            let mut digraph = writer.digraph();

            for (kind, src, tgt) in graph.edges.iter() {
                let src_name = String::from(src);
                let tgt_name = String::from(tgt);

                let src_ticket = graph.get_ticket(&src).unwrap();
                let tgt_ticket = graph.get_ticket(&tgt).unwrap();

                let src_path = src_ticket.path.as_ref().unwrap();
                let tgt_path = tgt_ticket.path.as_ref().unwrap();

                let src_label = format!("{} ({})", String::from(src), src_path);
                let tgt_label = format!("{} ({})", String::from(tgt), tgt_path);

                {
                    let mut src_node = digraph.node_named(&src_name);
                    src_node.set_label(&src_label);
                }

                {
                    let mut tgt_node = digraph.node_named(&tgt_name);
                    tgt_node.set_label(&tgt_label);
                }

                let edge_label = kind.strip_prefix("/kythe/edge/").unwrap();
                digraph.edge(&src_name, &tgt_name).attributes().set_label(edge_label);
            }
        }

        output.write_all(&output_bytes).unwrap();
    }
}
