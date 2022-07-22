use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

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

fn process(entries: &Path) {
    println!("{:?}", entries);
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Process { entries } => process(entries.as_path()),
    }
}
