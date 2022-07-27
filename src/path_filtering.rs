use crate::kythe::Ticket;
use glob::{Pattern, PatternError};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct MiniEntryDto {
    #[serde(rename = "source")]
    src: Ticket,
    #[serde(rename = "target")]
    tgt: Option<Ticket>,
}

pub struct PatternList {
    patterns: Vec<Pattern>,
}

impl PatternList {
    pub fn new(patterns: &Vec<String>) -> Result<Self, PatternError> {
        let mut compiled_patterns: Vec<Pattern> = Vec::with_capacity(patterns.len());

        for pattern in patterns {
            compiled_patterns.push(Pattern::new(&pattern)?);
        }

        return Ok(Self {
            patterns: compiled_patterns,
        });
    }

    pub fn matches(&self, str: &str) -> bool {
        self.patterns.iter().any(|p| p.matches(str))
    }
}

pub enum PathedStrategy {
    Exclude,
    Include,
    Only(PatternList),
}

pub enum UnpathedStrategy {
    Exclude,
    Include,
}

pub struct EntryPathFilter {
    pathed_strat: PathedStrategy,
    unpathed_strat: UnpathedStrategy,
}

impl EntryPathFilter {
    pub fn new(pathed_strat: PathedStrategy, unpathed_strat: UnpathedStrategy) -> Self {
        Self {
            pathed_strat,
            unpathed_strat,
        }
    }

    pub fn is_valid_line(&self, line: &str) -> bool {
        self.is_valid_entry(&serde_json::from_str(line).unwrap())
    }

    pub fn is_valid_entry(&self, entry: &MiniEntryDto) -> bool {
        self.has_valid_tgt(entry) && self.has_valid_src(entry)
    }

    fn has_valid_tgt(&self, entry: &MiniEntryDto) -> bool {
        match &entry.tgt {
            Some(tgt) => self.is_valid_path(tgt.path.as_ref()),
            None => true,
        }
    }

    fn has_valid_src(&self, entry: &MiniEntryDto) -> bool {
        self.is_valid_path(entry.src.path.as_ref())
    }

    fn is_valid_path(&self, path: Option<&String>) -> bool {
        match path {
            Some(text) => match &self.pathed_strat {
                PathedStrategy::Exclude => false,
                PathedStrategy::Include => true,
                PathedStrategy::Only(patterns) => patterns.matches(text),
            },
            None => match &self.unpathed_strat {
                UnpathedStrategy::Exclude => false,
                UnpathedStrategy::Include => true,
            },
        }
    }
}
