mod data_structures;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json;

use clap::{Parser, Subcommand};

use unwrap::unwrap;

use crate::data_structures::KytheGraph;

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
struct Ticket {
    corpus: Option<String>,
    language: Option<String>,
    path: Option<String>,
    root: Option<String>,
    signature: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
enum KEntry {
    KEdge {
        #[serde(rename = "source")]
        src: Ticket,
        #[serde(rename = "target")]
        tgt: Ticket,
        #[serde(rename = "edge_kind")]
        edge_kind: String,
        fact_name: String,
        fact_value: Option<String>,
    },
    KNode {
        #[serde(rename = "source")]
        src: Ticket,
        fact_name: String,
        fact_value: Option<String>,
    },
}

fn decode(text: &str) -> String {
    let bytes = base64::decode(text).unwrap();
    return String::from(std::str::from_utf8(&bytes).unwrap());
}

static KE_REF_CALL: &str = "/kythe/edge/ref/call";
static KE_DEFINES: &str = "/kythe/edge/defines";
static KE_DEFINES_BINDING: &str = "/kythe/edge/defines/binding";
static KE_DEFINES_IMPLICIT: &str = "/kythe/edge/defines/implicit";
static KN_KIND: &str = "/kythe/node/kind";

fn process(entries: &Path) {
    let file = File::open(entries).unwrap();
    let mut node_count = 0;
    let mut edge_count = 0;
    let mut graph = KytheGraph::new();

    let start = Instant::now();

    for line in BufReader::new(file).lines() {
        let text = line.unwrap();
        let parse_res: serde_json::Result<KEntry> = serde_json::from_str(&text);
        let entry = Box::new(unwrap!(parse_res, "Error while parsing {}", &text));

        match *entry {
            KEntry::KNode {
                src,
                fact_name,
                fact_value,
            } => {
                node_count += 1;
                if fact_value.is_some() {
                    let src_id = graph.add_ticket(src);
                    graph.add_fact(src_id, fact_name, fact_value.unwrap());
                }
            }
            KEntry::KEdge {
                src,
                tgt,
                edge_kind,
                ..
            } => {
                edge_count += 1;
                let src_id = graph.add_ticket(src);
                let tgt_id = graph.add_ticket(tgt);
                graph.add_edge(edge_kind, src_id, tgt_id);
            }
        }
    }

    let elapsed = start.elapsed();

    let calls = graph.get_edge_set(KE_REF_CALL);
    let n_calls = calls.map(|x| x.len()).unwrap_or_default();

    println!(
        "Found {} nodes and {} edges in {} secs!",
        node_count,
        edge_count,
        elapsed.as_secs_f32()
    );
    println!("Found {} function calls in the KytheGraph.", n_calls);

    let pairs = calls.map(|x| x.to_pairs()).unwrap_or_default();

    // let bindings = graph.get_edge_set(KE_DEFINES_BINDING).unwrap();

    for (src_id, tgt_id) in pairs {
        let src = graph.get_ticket(&src_id).unwrap();
        let tgt = graph.get_ticket(&tgt_id).unwrap();

        let src_kind = decode(graph.get_fact(&src_id, KN_KIND).unwrap());
        let tgt_kind = decode(graph.get_fact(&src_id, KN_KIND).unwrap());

        // let src_bindings = bindings.outgoing(&src_id);
        // let tgt_bindings = bindings.outgoing(&tgt_id);

        println!("({}) {:?}", src_kind, src);
        for (kind, src, tgt) in graph.get_all_outgoing(&src_id) {
            let ticket = graph.get_ticket(&src).unwrap();
            let ticket_kind = decode(graph.get_fact(&src, KN_KIND).unwrap_or(""));
            println!("\t({}, {}) {:?}", kind, ticket_kind, ticket);
        }
        println!("calls");
        println!("({}) {:?}", tgt_kind, tgt);
        for (kind, src, tgt) in graph.get_all_outgoing(&tgt_id) {
            let ticket = graph.get_ticket(&src).unwrap();
            let ticket_kind = decode(graph.get_fact(&src, KN_KIND).unwrap_or(""));
            println!("\t({}, {}) {:?}", kind, ticket_kind, ticket);
        }
        println!("");
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Process { entries } => process(entries.as_path()),
    }
}
