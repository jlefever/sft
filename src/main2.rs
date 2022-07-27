#![allow(dead_code)]
mod data_structures;
mod kythe;
mod path_filtering;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::{Parser, Subcommand};

use base64;
use path_filtering::{EntryPathFilter, PathedStrategy, PatternList, UnpathedStrategy};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Output some statistics
    Stats {
        /// The input entries file
        #[clap(value_parser)]
        entries: PathBuf,
    },
    Filter {
        /// The input entries file
        #[clap(value_parser)]
        entries: PathBuf,

        /// The output entries file
        #[clap(value_parser)]
        output: PathBuf,

        /// Exclude entries that have a "path" key. Note: this makes any patterns provided redundent.
        #[clap(long)]
        exclude_pathed: bool,

        /// Exclude entries that do not have a "path" key
        #[clap(long)]
        exclude_unpathed: bool,

        /// The patterns to filter on
        #[clap(value_parser)]
        patterns: Vec<String>,
    },
}

fn decode(text: &str) -> String {
    let bytes = base64::decode(text).unwrap();
    return String::from(std::str::from_utf8(&bytes).unwrap());
}

fn stats(entries: &Path) {
    let start = Instant::now();
    let mut reader = BufReader::new(File::open(entries).unwrap());
    let graph = kythe::load_graph(&mut reader);
    let elapsed = start.elapsed();

    let calls = graph.get_edge_set(kythe::KE_REF_CALL);
    let n_calls = calls.map(|x| x.len()).unwrap_or_default();

    println!("Finished loading in {} secs!", elapsed.as_secs_f32());
    println!("Found {} function calls in the KytheGraph.", n_calls);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Stats { entries } => stats(entries.as_path()),
        Commands::Filter {
            entries,
            output,
            patterns,
            exclude_pathed,
            exclude_unpathed,
        } => {
            let mut reader = BufReader::new(File::open(entries).unwrap());
            let mut writer = BufWriter::new(File::create(output).unwrap());
            let pattern_list = PatternList::new(patterns).unwrap();
            let pathed_strat = PathedStrategy::Only(pattern_list);
            let unpathed_strat = UnpathedStrategy::Exclude;
            let entry_path_filter = EntryPathFilter::new(pathed_strat, unpathed_strat);
            kythe::filter_lines(&mut reader, &mut writer, &mut |line: &str| {
                entry_path_filter.is_valid_line(line)
            });
        }
    }
}
