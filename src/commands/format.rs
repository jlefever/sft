use itertools::Itertools;

use crate::io::{EntryReader, open_bufwriter};
use crate::ir::{EntityGraph, RawGraph, SpecGraph};

use std::error::Error;
use std::io::Write;
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
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let reader = EntryReader::open(self.input.clone())?;
        let raw_graph = RawGraph::try_from(reader)?;
        let spec_graph = SpecGraph::try_from(raw_graph)?;
        let entity_graph = EntityGraph::try_from(spec_graph)?;

        // Sort
        let mut entities = entity_graph.entities.into_values().collect_vec();
        entities.sort();
        let mut deps = entity_graph.deps;
        deps.sort();

        // Output
        let mut writer = open_bufwriter(self.output.clone())?;

        for entity in entities {
            write!(writer, "{}\n", serde_json::to_string(&entity)?)?;
        }

        for dep in deps {
            write!(writer, "{}\n", serde_json::to_string(&dep)?)?;
        }

        Ok(())
    }
}
