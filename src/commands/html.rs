use itertools::Itertools;
use tinytemplate::TinyTemplate;

use crate::io::{EntryReader, Writer};
use crate::ir::{AnchorKind, EntityGraph, NodeKind, RawGraph, SpecGraph};

use std::error::Error;
use std::fmt::Write;
use std::path::PathBuf;

use super::CliCommand;

/// Produce an HTML view of Kythe data for debugging purposes.
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than
/// stdin/stdout for both performance reasons and compatibility reasons (Windows
/// console does not support UTF-8).
#[derive(clap::Args)]
pub struct CliHtmlCommand {
    /// Path of the file to read entries from. If ommitted, read from stdin.
    #[clap(short = 'i', value_name = "PATH", long, display_order = 1)]
    input: Option<PathBuf>,
    /// Path of the file to write to. If ommitted, write to stdout.
    #[clap(short = 'o', value_name = "PATH", long, display_order = 2)]
    output: Option<PathBuf>,
}

impl CliCommand for CliHtmlCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let input = self.input.as_ref().map(PathBuf::as_path);
        let output = self.output.as_ref().map(PathBuf::as_path);
        let mut writer = Writer::open(output)?;

        // Load graph
        let reader = EntryReader::open(input)?;
        let raw_graph = RawGraph::try_from(reader)?;
        let spec_graph = SpecGraph::try_from(raw_graph)?;
        // let entity_graph = EntityGraph::try_from(spec_graph)?;

        // ???
        let mut files = spec_graph.iter_nodes().map(|n| (&n.file_key, n)).into_group_map();
        let mut file_ctxs = Vec::new();

        for (file_key, nodes) in &mut files {
            nodes.sort();

            let text = spec_graph.get_file_text(&file_key).unwrap();
            let mut out = String::new();
            let mut prev_end = 0;

            for node in nodes {
                match &node.kind {
                    NodeKind::Anchor(AnchorKind::Explicit(pos)) => {
                        log::trace!("{:?}", pos);
                        let preceeding = match text.get(prev_end..pos.start) {
                            Some(x) => x,
                            None => continue,
                        };
                        let value = match text.get(pos.start..pos.end) {
                            Some(x) => x,
                            None => continue,
                        };
                        out.push_str(preceeding);
                        write!(out, "<span title=\"{}\">{}</span>", node.index.0, value)?;
                        prev_end = pos.end;
                    }
                    _ => (),
                }
            }

            out.push_str(&text[prev_end..]);
            let name = file_key.path.as_ref().unwrap().clone();
            file_ctxs.push(FileCtx { name, text: out });
        }

        // Templating
        let mut tt = TinyTemplate::new();
        tt.add_template("root", ROOT_TEMPLATE)?;
        tt.add_template("file", FILE_TEMPLATE)?;
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        let context = RootCtx { files: file_ctxs };
        let rendered = tt.render("root", &context)?;
        writer.write(rendered.as_bytes())?;

        Ok(())

        // // Pull first file
        // let mut entities = entity_graph.entities.into_values().collect_vec();
        // entities.sort();
        // let text = entities.iter().find_map(|entity| match &entity.kind {
        //     NodeKind::File(text) => Some(text),
        //     _ => None
        // })?.to_string();

        // // Write
        // let mut tt = TinyTemplate::new();
        // tt.add_template("hello", TEMPLATE)?;
        // let context = Context { text };
        // let rendered = tt.render("hello", &context)?;
        // writer.write(rendered.as_bytes())?;
    }
}

#[derive(serde::Serialize)]
struct RootCtx {
    files: Vec<FileCtx>,
}

static ROOT_TEMPLATE: &'static str = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>HTML 5 Boilerplate</title>

        <style>
        </style>
    </head>
    <body>
    {{ for file in files }}
        {{ call file with file }}
    {{ endfor }}
    </body>
    </html>
"#;

#[derive(serde::Serialize)]
struct FileCtx {
    name: String,
    text: String,
}

static FILE_TEMPLATE: &'static str = r#"
<div>
    <h1>{name}</h1>
    <pre>{text}</pre>
</div>
"#;

