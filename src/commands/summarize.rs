use itertools::Itertools;

use crate::io::{EntryReader, Writer};
use crate::ir::{AnchorKind, EntityGraph, NodeKind, Pos, RawGraph, SpecGraph, EdgeKind};

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use super::CliCommand;

/// Produce...
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than
/// stdin/stdout for both performance reasons and compatibility reasons (Windows
/// console does not support UTF-8).
#[derive(clap::Args)]
pub struct CliSummarizeCommand {
    /// Path of the file to read entries from. If ommitted, read from stdin.
    #[clap(short = 'i', value_name = "PATH", long, display_order = 1)]
    input: Option<PathBuf>,
    /// Path of the file to write to. If ommitted, write to stdout.
    #[clap(short = 'o', value_name = "PATH", long, display_order = 2)]
    output: Option<PathBuf>,
}

impl CliCommand for CliSummarizeCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let input = self.input.as_ref().map(PathBuf::as_path);
        let output = self.output.as_ref().map(PathBuf::as_path);
        let mut writer = Writer::open(output)?;

        // Load graph
        let reader = EntryReader::open(input)?;
        let raw_graph = RawGraph::try_from(reader)?;
        let spec_graph = SpecGraph::try_from(raw_graph)?;
        let entity_graph = EntityGraph::try_from(spec_graph)?;

        // 
        // let map = HashMap::new();

        // for dep in &entity_graph.deps {}

        // Sort
        let mut entities = entity_graph.entities.into_values().collect_vec();
        entities.sort();
        let mut deps = entity_graph.deps;
        deps.sort();

        // Output
        for entity in entities {
            let str = serde_json::to_string(&entity)?;
            writer.write_fmt(format_args!("{}\n", str))?;
        }

        for dep in deps {
            let str = serde_json::to_string(&dep)?;
            writer.write_fmt(format_args!("{}\n", str))?;
        }

        Ok(())
    }
}

fn to_nodekind_str(kind: NodeKind) -> String {
    let canon = match kind {
        NodeKind::Anchor(AnchorKind::Explicit(_)) => {
            NodeKind::Anchor(AnchorKind::Explicit(Pos::default()))
        },
        NodeKind::Constant(_) => NodeKind::Constant(String::default()),
        NodeKind::Doc(_) => NodeKind::Doc(String::default()),
        NodeKind::File(_) => NodeKind::File(String::default()),
        NodeKind::Lookup(_) => NodeKind::Lookup(String::default()),
        _ => kind,
    };

    format!("{:?}", canon)
}

fn to_edgekind_str(kind: EdgeKind) -> String {
    let canon = match kind {
        EdgeKind::Param(_) => EdgeKind::Param(0),
        _ => kind
    };

    format!("{:?}", canon)
}