use std::fmt::format;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::{CommandFactory, Parser};
use colored::Colorize;
use sled::Db;
use tokio::join;
use tokio::process::Command;

use rand::Rng;

use clap_verbosity_flag::{InfoLevel, Verbosity};

use glob::glob;

use anyhow::{Context, Result};

use tokio::task::JoinSet;

use itertools::Itertools;

///
#[derive(clap::Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(flatten, help_heading = "LOG OPTIONS")]
    verbose: Verbosity<InfoLevel>,

    #[clap(subcommand)]
    command: CliSubCommand,
}

#[derive(clap::Subcommand)]
enum CliSubCommand {
    Index(CliIndexCommand),
    Dump(CliDumpCommand),
}

/// Index every `.kzip` in the current directory and write the entries to
/// a database.
///
/// This database is a directory managed by sled (http://sled.rs/) and functions similarly to a B-tree. As a consequence, duplicate entries will be automatically filtered out.
///
/// Instead of indexing every `.kzip` in the current directory, an alternative
/// glob pattern may be provided. Notice that this glob pattern is not (well,
/// shouldn't be) expanded by your shell. Rather, the pattern is passed-in
/// verbatim. This is to overcome a limitation in most shells on the maximum
/// number of arguments that can be passed to an executable.
#[derive(clap::Args)]
struct CliIndexCommand {
    /// Path to a Kythe indexer
    #[clap(value_parser)]
    indexer: PathBuf,

    /// Path to database directory. Will append entries if already exists.
    #[clap(value_parser)]
    db: PathBuf,

    /// Glob pattern used to select files to index
    #[clap(value_parser, default_value_t = String::from("*.kzip"))]
    glob_pattern: String,

    /// Number of Kythe indexer processes to _attempt_ to run at one time
    #[clap(short, long)]
    batch_size: usize,
}

/// Write out the contents of a cache file created with `index`
#[derive(clap::Args)]
struct CliDumpCommand {
    /// The cache_db created with `index`
    #[clap(value_parser)]
    db: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new().filter_level(cli.verbose.log_level_filter()).init();

    match cli.command {
        CliSubCommand::Index(args) => index(args).await,
        CliSubCommand::Dump(args) => dump(args).await,
    }
}

async fn index(args: CliIndexCommand) -> Result<()> {
    // Open database
    let mut db = sled::open(&args.db).context("Failed to open database")?;
    if sled::Db::was_recovered(&db) {
        log::info!("Connected to existing database `{}`", &args.db.to_string_lossy());
    } else {
        log::info!("Created new database `{}`", &args.db.to_string_lossy());
    }

    // Collect files
    log::info!("Searching for files that match `{}`...", &args.glob_pattern);
    let start = Instant::now();
    let files = collect_files(&args.glob_pattern)?;
    let elapsed = start.elapsed().as_secs_f32();
    log::info!("Found {} files in {} secs", files.len(), elapsed);

    let n_batches = div_ceil(files.len(), args.batch_size);
    log::info!("Breaking into {} batches of at most {} files each...", n_batches, args.batch_size);

    // Launch subprocess for each file
    let mut rng = rand::thread_rng();

    let batches = &files.into_iter().chunks(args.batch_size);
    let batches = batches.into_iter().enumerate();

    for (i, batch) in batches {
        let files = batch.collect_vec();

        let start = files.first().unwrap().to_string_lossy();
        let end = files.last().unwrap().to_string_lossy();

        log::info!(
            "Starting batch ({} / {})...\n{}\n{}",
            i + 1,
            n_batches,
            format!("From: {}", start).dimmed(),
            format!("To:   {}", end).dimmed()
        );

        let start = Instant::now();
        process_files(&mut db, files, &mut rng).await.context("Failed to run batch")?;
        log::info!("Completed batch in {} secs", start.elapsed().as_secs_f32());
    }

    Ok(())
}

async fn process_files<R: Rng>(db: &mut Db, files: Vec<PathBuf>, rng: &mut R) -> Result<()> {
    let mut join_set = JoinSet::new();

    for file in files {
        log::debug!("Starting process for `{}`...", file.to_string_lossy());
        join_set.spawn(dummy_cmd(rng).output());
    }

    while let Some(res) = join_set.join_next().await {
        let output = res
            .context("Failed to join tasks...")?
            .context("Encountered error running process...")?;

        log::debug!("Collected {} bytes from stdout", output.stdout.len());

        // store_entries(db, output.stdout)?;

        // TODO: log stderr as warn or debug or error?
        // I think the indexer prints log messages to stderr
    }

    Ok(())
}

fn store_entries(db: &mut Db, bytes: Vec<u8>) -> Result<()> {
    todo!();
}

fn collect_files(glob_pattern: &String) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in glob(glob_pattern).context("Failed to read glob pattern")? {
        match entry {
            Ok(path) => paths.push(path),
            Err(ref err) => {
                log::warn!("Failed to read `{}`", err.path().to_string_lossy());
                log::debug!("{}", err);
            }
        }
    }

    paths.sort();
    Ok(paths)
}

async fn dump(args: CliDumpCommand) -> Result<()> {
    Ok(())
}

#[tokio::main]
async fn main2() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();

    let start = Instant::now();

    println!("Staring A...");
    let fut_a = dummy_cmd(&mut rng).output();

    println!("Staring B...");
    let fut_b = dummy_cmd(&mut rng).output();

    println!("Staring C...");
    let fut_c = dummy_cmd(&mut rng).output();

    let (res_a, res_b, res_c) = join!(fut_a, fut_b, fut_c);

    println!("---A---\n{}", String::from_utf8(res_a?.stdout)?);
    println!("---B---\n{}", String::from_utf8(res_b?.stdout)?);
    println!("---C---\n{}", String::from_utf8(res_c?.stdout)?);

    let elapsed = start.elapsed();
    println!("Completed in {} secs.", elapsed.as_secs_f32());

    Ok(())
}

fn dummy_cmd<R: Rng>(rng: &mut R) -> Command {
    let mut command = Command::new("ping");
    command.arg(gen_local_ip_str(rng));
    command
}

fn gen_local_ip_str<R: Rng>(rng: &mut R) -> String {
    let block1 = rng.gen::<u8>();
    let block2 = rng.gen::<u8>();
    let block3 = rng.gen::<u8>();
    format!("127.{}.{}.{}", block1, block2, block3)
}

fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}
