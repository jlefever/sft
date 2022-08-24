use itertools::Itertools;

use crate::io::{EntryReader, Writer};
use crate::ir::{EntityGraph, RawGraph, SpecGraph};

use std::path::PathBuf;

use super::CliCommand;

/// Produce "human-readable" JSON nodes and edges for debugging purposes.
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than
/// stdin/stdout for both performance reasons and compatibility reasons (Windows
/// console does not support UTF-8).
#[derive(clap::Args)]
pub struct CliFormatCommand {
    /// Path of the file to read entries from. If ommitted, read from stdin.
    #[clap(short = 'i', value_name = "PATH", long, display_order = 1)]
    input: Option<PathBuf>,
    /// Path of the file to write to. If ommitted, write to stdout.
    #[clap(short = 'o', value_name = "PATH", long, display_order = 2)]
    output: Option<PathBuf>,
}

impl CliCommand for CliFormatCommand {
    fn execute(&self) {
        let input = self.input.as_ref().map(PathBuf::as_path);
        let output = self.output.as_ref().map(PathBuf::as_path);
        let mut writer = Writer::open(output).unwrap();

        // Load graph
        let reader = EntryReader::open(input).unwrap();
        let raw_graph = RawGraph::try_from(reader).unwrap();
        let spec_graph = SpecGraph::try_from(raw_graph).unwrap();
        let entity_graph = EntityGraph::try_from(spec_graph).unwrap();

        // Sort
        let mut entities = entity_graph.entities.into_values().collect_vec();
        entities.sort();
        let mut deps = entity_graph.deps;
        deps.sort();

        // Output
        for entity in entities {
            let str = serde_json::to_string(&entity).unwrap();
            writer.write_fmt(format_args!("{}\n", str)).unwrap();
        }

        for dep in deps {
            let str = serde_json::to_string(&dep).unwrap();
            writer.write_fmt(format_args!("{}\n", str)).unwrap();
        }
    }
}
