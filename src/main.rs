use log;
use std::fs;
use std::io;

use std::str::FromStr;
use std::{path::PathBuf, time::Instant};

use clap::{Args, Parser, Subcommand};
use glob::Pattern;

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
    Exclude(ExcludeCommand),
}

/// Exclude entries that meet the supplied conditions.
///
/// Reads a stream of newline-delimited entries in and writes each entry back out unless it
/// meets one of the supplied conditions to be excluded. Each entry is either a node or an
/// edge. Most of these options only concern edges, but some nodes may also be excluded if they
/// cannot possibly be involved in any of the remaining edges. Use --keep-nodes to disable this
/// behavior.
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than stdin/stdout for both
/// performance reasons and compatibility reasons (Windows console does not support UTF-8).
#[derive(Args)]
struct ExcludeCommand {
    /// Path of the file to read entries from. If ommitted, read from stdin.
    #[clap(short = 'i', value_name = "PATH", long, display_order = 1)]
    input: Option<PathBuf>,
    /// Path of the file to write entries to. If ommitted, write to stdout.
    #[clap(short = 'o', value_name = "PATH", long, display_order = 2)]
    output: Option<PathBuf>,

    /// Alias for --if-any-nilpathed.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "nilpath",
        short = 'n',
        long,
        display_order = 3
    )]
    if_nilpathed: bool,
    /// Exclude an edge if either the source OR the target lack a "path" property.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "nilpath",
        long,
        display_order = 4
    )]
    if_any_nilpathed: bool,
    /// Exclude an edge if both the source AND the target lack a "path" property.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "nilpath",
        long,
        display_order = 5
    )]
    if_all_nilpathed: bool,
    /// Exclude an edge if the source lacks a "path" property.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "nilpath",
        long,
        display_order = 6
    )]
    if_src_nilpathed: bool,
    /// Exclude an edge if the target lacks a "path" property.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "nilpath",
        long,
        display_order = 7
    )]
    if_tgt_nilpathed: bool,

    /// Alias for --if-any-abspathed.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "abspath",
        short = 'a',
        long,
        display_order = 8
    )]
    if_abspathed: bool,
    /// Exclude an edge if either the source OR the target use an absolute path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "abspath",
        long,
        display_order = 9
    )]
    if_any_abspathed: bool,
    /// Exclude an edge if both the source AND the target use an absolute path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "abspath",
        long,
        display_order = 10
    )]
    if_all_abspathed: bool,
    /// Exclude an edge if the source uses an absolute path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "abspath",
        long,
        display_order = 11
    )]
    if_src_abspathed: bool,
    /// Exclude an edge if the target uses an absolute path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "abspath",
        long,
        display_order = 12
    )]
    if_tgt_abspathed: bool,

    /// Alias for --if-any-relpathed.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "relpath",
        short = 'r',
        long,
        display_order = 13
    )]
    if_relpathed: bool,
    /// Exclude an edge if either the source OR the target use a relative path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "relpath",
        long,
        display_order = 14
    )]
    if_any_relpathed: bool,
    /// Exclude an edge if both the source AND the target use a relative path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "relpath",
        long,
        display_order = 15
    )]
    if_all_relpathed: bool,
    /// Exclude an edge if the source uses a relative path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "relpath",
        long,
        display_order = 16
    )]
    if_src_relpathed: bool,
    /// Exclude an edge if the target uses a relative path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "relpath",
        long,
        display_order = 17
    )]
    if_tgt_relpathed: bool,

    /// Alias for --by-any-path.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "path",
        value_name = "GLOB_PATTERN",
        short = 'p',
        long,
        display_order = 18
    )]
    by_path: Option<String>,
    /// Exclude an edge if either the source OR the target path matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "path",
        value_name = "GLOB_PATTERN",
        long,
        display_order = 19
    )]
    by_any_path: Option<String>,
    /// Exclude an edge if both the source AND the target path matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "path",
        value_name = "GLOB_PATTERN",
        long,
        display_order = 20
    )]
    by_all_path: Option<String>,
    /// Exclude an edge if the source path matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "path",
        value_name = "GLOB_PATTERN",
        long,
        display_order = 21
    )]
    by_src_path: Option<String>,
    /// Exclude an edge if the target path matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "path",
        value_name = "GLOB_PATTERN",
        long,
        display_order = 22
    )]
    by_tgt_path: Option<String>,

    /// Exclude an entry (node or edge) if the fact name matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "factname",
        value_name = "GLOB_PATTERN",
        short = 'f',
        long,
        display_order = 23
    )]
    by_factname: Option<String>,
    /// Exclude an edge if the fact name matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "factname",
        value_name = "GLOB_PATTERN",
        long,
        display_order = 24
    )]
    by_edge_factname: Option<String>,
    /// Exclude a node if the fact name matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "factname",
        value_name = "GLOB_PATTERN",
        long,
        display_order = 25
    )]
    by_node_factname: Option<String>,

    /// Exclude an edge if the edge kind matches a given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        value_name = "GLOB_PATTERN",
        short = 'e',
        long,
        display_order = 26
    )]
    by_edgekind: Option<String>,

    /// Do not remove any nodes unless explicitly requested (e.g. with --by-node-factname).
    #[clap(help_heading = "MISC", short = 'k', long, display_order = 27)]
    keep_nodes: bool,
}

impl ExcludeCommand {
    fn execute(&self) {
        let mut input = create_input(self.input.as_ref()).unwrap();
        let mut output = create_output(self.output.as_ref()).unwrap();

        let mut rules: Vec<Box<dyn Exclusion>> = Vec::new();

        let mut push_path_kind_exclusion =
            |exclusion_kind: Option<EdgeExclusionKind>, path_kind: PathKind| {
                if let Some(exclusion_kind) = exclusion_kind {
                    let ticket_rule = Box::new(PathKindBasedExclusion::new(path_kind));
                    let rule =
                        TickedBasedExclusion::new(exclusion_kind, ticket_rule, self.keep_nodes);
                    rules.push(Box::new(rule));
                };
            };

        let nilpath_kind = EdgeExclusionKind::from_bools(
            self.if_any_nilpathed || self.if_nilpathed,
            self.if_all_nilpathed,
            self.if_src_nilpathed,
            self.if_tgt_nilpathed,
        );

        push_path_kind_exclusion(nilpath_kind, PathKind::NilPathed);

        let abspath_kind = EdgeExclusionKind::from_bools(
            self.if_any_abspathed || self.if_abspathed,
            self.if_all_abspathed,
            self.if_src_abspathed,
            self.if_tgt_abspathed,
        );

        push_path_kind_exclusion(abspath_kind, PathKind::AbsPathed);

        let relpath_kind = EdgeExclusionKind::from_bools(
            self.if_any_relpathed || self.if_relpathed,
            self.if_all_relpathed,
            self.if_src_relpathed,
            self.if_tgt_relpathed,
        );

        push_path_kind_exclusion(relpath_kind, PathKind::RelPathed);

        let pathmatch_kind = EdgeExclusionKind::from_bools(
            self.by_any_path.is_some() || self.by_path.is_some(),
            self.by_all_path.is_some(),
            self.by_src_path.is_some(),
            self.by_tgt_path.is_some(),
        );

        let pathmatch_pattern = collapse(vec![
            self.by_path.as_ref(),
            self.by_any_path.as_ref(),
            self.by_all_path.as_ref(),
            self.by_src_path.as_ref(),
            self.by_tgt_path.as_ref(),
        ]);

        if let Some(exclusion_kind) = pathmatch_kind {
            let pattern = Pattern::from_str(pathmatch_pattern.unwrap()).unwrap();
            let ticket_rule = Box::new(PathPatternBasedExclusion::new(pattern));
            let rule = TickedBasedExclusion::new(exclusion_kind, ticket_rule, self.keep_nodes);
            rules.push(Box::new(rule));
        }

        log::debug!("Starting exclusion process with {} rule(s)...", rules.len());
        let start = Instant::now();

        let mut buf = String::new();
        let mut num_lines = 0u128;
        let mut num_excluded = 0u128;
        'outer: while io::BufRead::read_line(&mut input, &mut buf).unwrap() != 0 {
            num_lines = num_lines + 1;
            let entry: Entry = serde_json::from_str(&buf).unwrap();

            for rule in &rules {
                if rule.is_excluded(&entry) {
                    num_excluded += 1;
                    buf.clear();
                    continue 'outer;
                }
            }

            io::Write::write(&mut output, buf.as_bytes()).unwrap();
            buf.clear();
        }

        log::debug!(
            "Excluded {} out of {} entries in {} secs.",
            num_excluded,
            num_lines,
            start.elapsed().as_secs_f32()
        );
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Ticket {
    pub corpus: Option<String>,
    pub language: Option<String>,
    pub path: Option<String>,
    pub root: Option<String>,
    pub signature: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
enum Entry {
    Edge {
        #[serde(rename = "source")]
        src: Ticket,
        #[serde(rename = "target")]
        tgt: Ticket,
        edge_kind: Option<String>,
        fact_name: String,
        fact_value: Option<String>,
    },
    Node {
        #[serde(rename = "source")]
        src: Ticket,
        fact_name: String,
        fact_value: Option<String>,
    },
}

#[derive(Debug)]
enum EdgeExclusionKind {
    Any,
    All,
    Src,
    Tgt,
}

impl EdgeExclusionKind {
    fn from_bools(any: bool, all: bool, src: bool, tgt: bool) -> Option<Self> {
        match (any, all, src, tgt) {
            (false, false, false, false) => None,
            (true, false, false, false) => Some(Self::Any),
            (false, true, false, false) => Some(Self::All),
            (false, false, true, false) => Some(Self::Src),
            (false, false, false, true) => Some(Self::Tgt),
            _ => panic!(),
        }
    }
}

trait Exclusion {
    fn is_excluded(&self, entry: &Entry) -> bool;
}

enum FactExclusionKind {
    Both,
    Edge,
    Node,
}

impl FactExclusionKind {
    fn from_bools(both: bool, edge: bool, node: bool) -> Option<Self> {
        match (both, edge, node) {
            (false, false, false) => None,
            (true, false, false) => Some(Self::Both),
            (false, true, false) => Some(Self::Edge),
            (false, false, true) => Some(Self::Node),
            _ => panic!(),
        }
    }
}

struct FactBasedExclusion {
    kind: FactExclusionKind,
    pattern: Pattern,
}

impl FactBasedExclusion {
    fn new(kind: FactExclusionKind, pattern: Pattern) -> Self {
        Self { kind, pattern }
    }
}

impl Exclusion for FactBasedExclusion {
    fn is_excluded(&self, entry: &Entry) -> bool {
        match entry {
            Entry::Edge { fact_name, .. } => match self.kind {
                FactExclusionKind::Node => false,
                _ => self.pattern.matches(fact_name),
            },
            Entry::Node { fact_name, .. } => match self.kind {
                FactExclusionKind::Edge => false,
                _ => self.pattern.matches(fact_name),
            },
        }
    }
}

struct TickedBasedExclusion {
    kind: EdgeExclusionKind,
    ticket_rule: Box<dyn TicketExclusion>,
    keep_nodes: bool,
}

impl TickedBasedExclusion {
    fn new(
        kind: EdgeExclusionKind,
        ticket_rule: Box<dyn TicketExclusion>,
        keep_nodes: bool,
    ) -> Self {
        Self {
            kind,
            ticket_rule,
            keep_nodes,
        }
    }
}

impl Exclusion for TickedBasedExclusion {
    fn is_excluded(&self, entry: &Entry) -> bool {
        let is_excluded = |t: &Ticket| self.ticket_rule.is_excluded(t);

        match entry {
            Entry::Edge { src, tgt, .. } => match self.kind {
                EdgeExclusionKind::Any => is_excluded(src) || is_excluded(tgt),
                EdgeExclusionKind::All => is_excluded(src) && is_excluded(tgt),
                EdgeExclusionKind::Src => is_excluded(src),
                EdgeExclusionKind::Tgt => is_excluded(tgt),
            },
            Entry::Node { src, .. } => match self.kind {
                EdgeExclusionKind::Any => !self.keep_nodes && is_excluded(src),
                _ => false,
            },
        }
    }
}

trait TicketExclusion {
    fn is_excluded(&self, ticket: &Ticket) -> bool;
}

#[derive(PartialEq, Eq)]
enum PathKind {
    NilPathed,
    RelPathed,
    AbsPathed,
}

impl PathKind {
    fn of(path: Option<&String>) -> Self {
        match path {
            None => Self::NilPathed,
            Some(text) => match text.chars().next() {
                Some('/') => Self::AbsPathed,
                _ => Self::RelPathed,
            },
        }
    }
}

struct PathKindBasedExclusion {
    kind: PathKind,
}

impl PathKindBasedExclusion {
    fn new(kind: PathKind) -> Self {
        Self { kind }
    }
}

impl TicketExclusion for PathKindBasedExclusion {
    fn is_excluded(&self, ticket: &Ticket) -> bool {
        self.kind == PathKind::of(ticket.path.as_ref())
    }
}

struct PathPatternBasedExclusion {
    pattern: Pattern,
}

impl PathPatternBasedExclusion {
    fn new(pattern: Pattern) -> Self {
        Self { pattern }
    }
}

impl TicketExclusion for PathPatternBasedExclusion {
    fn is_excluded(&self, ticket: &Ticket) -> bool {
        match &ticket.path {
            None => false,
            Some(path) => self.pattern.matches(path),
        }
    }
}

fn create_input(path: Option<&PathBuf>) -> io::Result<io::BufReader<Box<dyn io::Read>>> {
    Ok(io::BufReader::new(match path {
        None => Box::new(io::stdin().lock()),
        Some(path) => Box::new(fs::File::open(path)?),
    }))
}

fn create_output(path: Option<&PathBuf>) -> io::Result<io::BufWriter<Box<dyn io::Write>>> {
    Ok(io::BufWriter::new(match path {
        None => Box::new(io::stdout().lock()),
        Some(path) => Box::new(fs::File::create(path)?),
    }))
}

fn collapse<T>(opts: Vec<Option<T>>) -> Option<T> {
    for opt in opts {
        if opt.is_some() {
            return opt;
        };
    }

    return None;
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
            CliSubCommand::Exclude(exclude) => exclude.execute(),
        },
    }
}
