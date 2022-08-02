mod commands;
mod collections;
mod kythe;
mod util;

use clap::{Parser, Subcommand};
use commands::CliCommand;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Show all logging messages.
    #[clap(short = 'v', long)]
    verbose: bool,

    /// Silence all logging messages.
    #[clap(short = 'q', long)]
    quiet: bool,

    #[clap(subcommand)]
    command: Option<CliSubCommand>,
}

#[derive(Subcommand)]
enum CliSubCommand {
    Display(commands::display::CliDisplayCommand),
    Exclude(commands::exclude::CliExcludeCommand),
}

fn main() {
    let cli = Cli::parse();

    let verbosity = match cli.verbose {
        true => stderrlog::LogLevelNum::Trace,
        false => stderrlog::LogLevelNum::Info,
    };

    stderrlog::new()
        .module(module_path!())
        .quiet(cli.quiet)
        .verbosity(verbosity)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .unwrap();

    match cli.command {
        None => std::process::exit(0),
        Some(command) => match command {
            CliSubCommand::Exclude(com) => com.execute(),
            CliSubCommand::Display(com) => com.execute(),
        },
    }
}
