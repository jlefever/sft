mod data_structures;
mod kythe;

use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::{Parser, Subcommand};

use crate::kythe::load_graph;

use base64;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process entries
    Process {
        /// The input entries file
        #[clap(value_parser)]
        entries: PathBuf,
    },
}

fn decode(text: &str) -> String {
    let bytes = base64::decode(text).unwrap();
    return String::from(std::str::from_utf8(&bytes).unwrap());
}

fn process(entries: &Path) {
    let start = Instant::now();
    let graph = load_graph(entries).unwrap();
    let elapsed = start.elapsed();

    let calls = graph.get_edge_set(kythe::KE_REF_CALL);
    let n_calls = calls.map(|x| x.len()).unwrap_or_default();

    println!("Finished loading in {} secs!", elapsed.as_secs_f32());
    println!("Found {} function calls in the KytheGraph.", n_calls);

    // let pairs = calls.map(|x| x.to_pairs()).unwrap_or_default();

    // let bindings = graph.get_edge_set(KE_DEFINES_BINDING).unwrap();

    // for (src_id, tgt_id) in pairs {
    //     let src = graph.get_ticket(&src_id).unwrap();
    //     let tgt = graph.get_ticket(&tgt_id).unwrap();

    //     let src_kind = decode(graph.get_fact(&src_id, KN_KIND).unwrap());
    //     let tgt_kind = decode(graph.get_fact(&src_id, KN_KIND).unwrap());

    //     // let src_bindings = bindings.outgoing(&src_id);
    //     // let tgt_bindings = bindings.outgoing(&tgt_id);

    //     println!("({}) {:?}", src_kind, src);
    //     for (kind, src, tgt) in graph.get_all_outgoing(&src_id) {
    //         let ticket = graph.get_ticket(&src).unwrap();
    //         let ticket_kind = decode(graph.get_fact(&src, KN_KIND).unwrap_or(""));
    //         println!("\t({}, {}) {:?}", kind, ticket_kind, ticket);
    //     }
    //     println!("calls");
    //     println!("({}) {:?}", tgt_kind, tgt);
    //     for (kind, src, tgt) in graph.get_all_outgoing(&tgt_id) {
    //         let ticket = graph.get_ticket(&src).unwrap();
    //         let ticket_kind = decode(graph.get_fact(&src, KN_KIND).unwrap_or(""));
    //         println!("\t({}, {}) {:?}", kind, ticket_kind, ticket);
    //     }
    //     println!("");
    // }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Process { entries } => process(entries.as_path()),
    }
}
