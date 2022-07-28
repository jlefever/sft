use log;
use std::fmt::Debug;
use std::fs;
use std::io;

use std::str::FromStr;
use std::{path::PathBuf, time::Instant};

use clap::{Args, Parser, Subcommand};
use glob::Pattern;

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
pub enum Entry {
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

pub fn create_input(path: Option<&PathBuf>) -> io::Result<io::BufReader<Box<dyn io::Read>>> {
    Ok(io::BufReader::new(match path {
        None => Box::new(io::stdin().lock()),
        Some(path) => Box::new(fs::File::open(path)?),
    }))
}

pub fn create_output(path: Option<&PathBuf>) -> io::Result<io::BufWriter<Box<dyn io::Write>>> {
    Ok(io::BufWriter::new(match path {
        None => Box::new(io::stdout().lock()),
        Some(path) => Box::new(fs::File::create(path)?),
    }))
}

pub fn collapse<T>(opts: Vec<Option<T>>) -> Option<T> {
    for opt in opts {
        if opt.is_some() {
            return opt;
        };
    }

    return None;
}