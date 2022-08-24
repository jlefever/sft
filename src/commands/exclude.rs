use crate::io::Entry;
use crate::io::EntryLineReader;
use crate::io::Ticket;
use crate::io::Writer;

use log;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::fs;

use std::path::Path;
use std::{path::PathBuf, time::Instant};

use super::CliCommand;

/// Exclude entries that meet the supplied conditions.
///
/// Reads a stream of newline-delimited entries in and writes each entry back
/// out unless it meets one of the supplied conditions to be excluded. Each
/// entry is either a node or an edge. Most of these options only concern edges,
/// but some nodes may also be excluded if they cannot possibly be involved in
/// any of the remaining edges. Use --keep-nodes to disable this behavior.
///
/// Some options ask for a "pathlist". A pathlist is a text file containing a
/// newline-delimited list of paths.
///
/// For more info on Kythe's entry format, see https://kythe.io/docs/kythe-storage.html.
///
/// On Windows, it is recommended to use --input/--output rather than
/// stdin/stdout for both performance reasons and compatibility reasons (Windows
/// console does not support UTF-8).
#[derive(clap::Args)]
pub struct CliExcludeCommand {
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

    /// Exclude an edge if either the source OR the target lack a "path"
    /// property.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "nilpath",
        long,
        display_order = 4
    )]
    if_any_nilpathed: bool,

    /// Exclude an edge if both the source AND the target lack a "path"
    /// property.
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

    /// Only include an edge if both the source AND the target path matches a
    /// given glob pattern.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "path",
        value_name = "GLOB_PATTERN",
        short = 'p',
        long,
        display_order = 18
    )]
    by_path: Option<String>,

    // /// Only include an edge if either the source OR the target path matches a
    // /// given glob pattern.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "path",
    //     value_name = "GLOB_PATTERN",
    //     long,
    //     display_order = 19
    // )]
    // by_any_path: Option<String>,

    // /// Only include an edge if both the source AND the target path matches a
    // /// given glob pattern.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "path",
    //     value_name = "GLOB_PATTERN",
    //     long,
    //     display_order = 20
    // )]
    // by_all_path: Option<String>,

    // /// Only include an edge if the source path matches a given glob pattern.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "path",
    //     value_name = "GLOB_PATTERN",
    //     long,
    //     display_order = 21
    // )]
    // by_src_path: Option<String>,

    // /// Only include an edge if the target path matches a given glob pattern.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "path",
    //     value_name = "GLOB_PATTERN",
    //     long,
    //     display_order = 22
    // )]
    // by_tgt_path: Option<String>,
    /// Only include an edge if both the source AND the target path is found
    /// verbatim in the provided pathlist.
    #[clap(
        help_heading = "EXCLUDE OPTIONS",
        group = "pathlist",
        value_name = "PATHLIST_PATH",
        short = 'l',
        long,
        display_order = 23
    )]
    by_pathlist: Option<String>,

    // /// Only include an edge if either the source OR the target path is found
    // /// verbatim in the provided pathlist.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "pathlist",
    //     value_name = "PATHLIST_PATH",
    //     long,
    //     display_order = 24
    // )]
    // by_any_pathlist: Option<String>,

    // /// Only include an edge if both the source AND the target path is found
    // /// verbatim in the provided pathlist.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "pathlist",
    //     value_name = "PATHLIST_PATH",
    //     long,
    //     display_order = 25
    // )]
    // by_all_pathlist: Option<String>,

    // /// Only include an edge if the source path is found verbatim in the
    // /// provided pathlist.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "pathlist",
    //     value_name = "PATHLIST_PATH",
    //     long,
    //     display_order = 26
    // )]
    // by_src_pathlist: Option<String>,

    // /// Only include an edge if the target path is found verbatim in the
    // /// provided pathlist.
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "pathlist",
    //     value_name = "PATHLIST_PATH",
    //     long,
    //     display_order = 27
    // )]
    // by_tgt_pathlist: Option<String>,

    // /// Exclude an entry (node or edge) if the fact name matches a given glob
    // /// pattern. (TODO)
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "factname",
    //     value_name = "GLOB_PATTERN",
    //     short = 'f',
    //     long,
    //     display_order = 28
    // )]
    // by_factname: Option<String>,

    // /// Exclude an edge if the fact name matches a given glob pattern. (TODO)
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "factname",
    //     value_name = "GLOB_PATTERN",
    //     long,
    //     display_order = 29
    // )]
    // by_edge_factname: Option<String>,

    // /// Exclude a node if the fact name matches a given glob pattern. (TODO)
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     group = "factname",
    //     value_name = "GLOB_PATTERN",
    //     long,
    //     display_order = 30
    // )]
    // by_node_factname: Option<String>,

    // /// Exclude an edge if the edge kind matches a given glob pattern. (TODO)
    // #[clap(
    //     help_heading = "EXCLUDE OPTIONS",
    //     value_name = "GLOB_PATTERN",
    //     short = 'e',
    //     long,
    //     display_order = 31
    // )]
    // by_edgekind: Option<String>,
    /// Do not remove any nodes unless explicitly requested (e.g. with
    /// --by-node-factname).
    #[clap(help_heading = "MISC", short = 'k', long, display_order = 33)]
    keep_nodes: bool,
}

impl CliCommand for CliExcludeCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let input = self.input.as_ref().map(PathBuf::as_path);
        let output = self.output.as_ref().map(PathBuf::as_path);
        let mut writer = Writer::open(output)?;

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

        if let Some(pattern) = &self.by_path {
            let matcher = globset::Glob::new(pattern)?.compile_matcher();
            let ticket_rule = Box::new(PathPatternBasedExclusion::new(matcher));
            let rule =
                TickedBasedExclusion::new(EdgeExclusionKind::Any, ticket_rule, self.keep_nodes);
            rules.push(Box::new(rule));
        }

        if let Some(pathlist) = &self.by_pathlist {
            log::debug!("Loading pathlist {}...", pathlist);
            match fs::read_to_string(pathlist) {
                Err(_) => log::error!("Failed to read pathlist {}", pathlist),
                Ok(text) => {
                    let rule = PathListBasedExclusion::new(text.lines().map(String::from));
                    let rule = Box::new(rule);
                    let rule =
                        TickedBasedExclusion::new(EdgeExclusionKind::Any, rule, self.keep_nodes);
                    rules.push(Box::new(rule));
                }
            }
        }

        log::debug!(
            "Found the following {} exclusion rule(s) on the command line:",
            rules.len()
        );
        for rule in &rules {
            log::debug!("{:#?}", rule);
        }
        log::info!("Starting exclusion process...");

        let start = Instant::now();
        let mut num_lines = 0u128;
        let mut num_excluded = 0u128;

        'outer: for (line, entry) in EntryLineReader::open(input)? {
            num_lines = num_lines + 1;

            for rule in &rules {
                if rule.is_excluded(&entry) {
                    num_excluded += 1;
                    continue 'outer;
                }
            }

            writer.write(line.as_bytes())?;
        }

        log::info!(
            "Excluded {} out of {} entries in {} secs.",
            num_excluded,
            num_lines,
            start.elapsed().as_secs_f32()
        );

        Ok(())
    }
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

trait Exclusion: Debug {
    fn is_excluded(&self, entry: &Entry) -> bool;
}

#[allow(dead_code)]
#[derive(Debug)]
enum FactExclusionKind {
    Both,
    Edge,
    Node,
}

#[allow(dead_code)]
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

#[derive(Debug)]
struct FactBasedExclusion {
    kind: FactExclusionKind,
    matcher: globset::GlobMatcher,
}

#[allow(dead_code)]
impl FactBasedExclusion {
    fn new(kind: FactExclusionKind, matcher: globset::GlobMatcher) -> Self {
        Self { kind, matcher }
    }
}

impl Exclusion for FactBasedExclusion {
    fn is_excluded(&self, entry: &Entry) -> bool {
        match entry {
            Entry::Edge { fact_name, .. } => match self.kind {
                FactExclusionKind::Node => false,
                _ => self.matcher.is_match(fact_name),
            },
            Entry::Node { fact_name, .. } => match self.kind {
                FactExclusionKind::Edge => false,
                _ => self.matcher.is_match(fact_name),
            },
        }
    }
}

#[derive(Debug)]
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

trait TicketExclusion: Debug {
    fn is_excluded(&self, ticket: &Ticket) -> bool;
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
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

#[derive(Debug)]
struct PathPatternBasedExclusion {
    matcher: globset::GlobMatcher,
}

impl PathPatternBasedExclusion {
    fn new(matcher: globset::GlobMatcher) -> Self {
        Self { matcher }
    }
}

impl TicketExclusion for PathPatternBasedExclusion {
    fn is_excluded(&self, ticket: &Ticket) -> bool {
        match &ticket.path {
            None => false,
            Some(path) => !self.matcher.is_match(Path::new(path)),
        }
    }
}

struct PathListBasedExclusion {
    paths: HashSet<String>,
}

impl Debug for PathListBasedExclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathListBasedExclusion")
            .field("paths", &self.paths.len())
            .finish()
    }
}

impl PathListBasedExclusion {
    fn new(paths: impl Iterator<Item = String>) -> Self {
        Self {
            paths: paths.collect(),
        }
    }
}

impl TicketExclusion for PathListBasedExclusion {
    fn is_excluded(&self, ticket: &Ticket) -> bool {
        match &ticket.path {
            None => false,
            Some(path) => !self.paths.contains(path),
        }
    }
}
