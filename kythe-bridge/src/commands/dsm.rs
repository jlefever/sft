// use crate::dv8::{Dv8Graph, Dv8Matrix};
// use crate::io::Writer;

// use std::path::PathBuf;
// use std::time::Instant;

// use super::CliCommand;

// /// Produce a JSON file that can be processed by DV8.
// ///
// /// Reads a stream of newline-delimited entries in and produces a file-level DSM
// /// (Design Structure Matrix) in a format suitable for DV8 (https://archdia.com/).
// ///
// /// On Windows, it is recommended to use --input/--output rather than
// /// stdin/stdout for both performance reasons and compatibility reasons (Windows
// /// console does not support UTF-8).
// #[derive(clap::Args)]
// pub struct CliDsmCommand {
//     /// Path of the file to read entries from. If ommitted, read from stdin.
//     #[clap(short = 'i', value_name = "PATH", long, display_order = 1)]
//     input: Option<PathBuf>,
//     /// Path of the file to write JSON file to. If ommitted, write to stdout.
//     #[clap(short = 'o', value_name = "PATH", long, display_order = 2)]
//     output: Option<PathBuf>,
//     /// Name of the output DSM. This is included in the JSON file.
//     #[clap(short = 'n', long, display_order = 3)]
//     name: String,
// }

// impl CliCommand for CliDsmCommand {
//     fn execute(&self) {
//         let input = self.input.as_ref().map(PathBuf::as_path);
//         let output = self.output.as_ref().map(PathBuf::as_path);
//         let mut writer = Writer::open(output).unwrap();

//         let start = Instant::now();
//         let graph = Dv8Graph::open(input).unwrap();
//         log::debug!("Loaded graph in {} secs.", start.elapsed().as_secs_f32());

//         let start = Instant::now();
//         let mut matrix = Dv8Matrix::from(graph);
//         matrix.set_name(self.name.clone());
//         log::debug!("Converted to DV8 matrix in {} secs.", start.elapsed().as_secs_f32());

//         let start = Instant::now();
//         let serialized = serde_json::to_string_pretty(&matrix).unwrap();
//         log::debug!("Serialized in {} secs.", start.elapsed().as_secs_f32());

//         writer.write(serialized.as_bytes()).unwrap();
//     }
// }
