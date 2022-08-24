use std::fmt::Arguments;
use std::fs;
use std::io;

use std::io::BufRead;
use std::path::Path;

pub struct Writer(io::BufWriter<Box<dyn io::Write>>);

impl Writer {
    pub fn open(path: Option<&Path>) -> io::Result<Self> {
        Ok(Self(io::BufWriter::new(match path {
            None => Box::new(io::stdout().lock()),
            Some(path) => Box::new(fs::File::create(path)?),
        })))
    }

    pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::Write::write(&mut self.0, buf)
    }

    pub fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        io::Write::write_fmt(&mut self.0, fmt)
    }
}

pub struct Reader(io::BufReader<Box<dyn io::Read>>);

impl Reader {
    fn open(path: Option<&Path>) -> io::Result<Self> {
        Ok(Self(io::BufReader::new(match path {
            None => Box::new(io::stdin().lock()),
            Some(path) => Box::new(fs::File::open(path)?),
        })))
    }
}

pub struct EntryReader(Reader);

impl EntryReader {
    pub fn open(path: Option<&Path>) -> io::Result<Self> {
        Ok(Self(Reader::open(path)?))
    }
}

impl IntoIterator for EntryReader {
    type Item = Entry;
    type IntoIter = EntryIter;

    fn into_iter(self) -> Self::IntoIter {
        EntryIter {
            reader: self.0,
            buffer: String::new(),
        }
    }
}

pub struct EntryIter {
    reader: Reader,
    buffer: String,
}

impl Iterator for EntryIter {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.0.read_line(&mut self.buffer).unwrap() {
            0 => None,
            _ => {
                let entry = Entry::from_json(&self.buffer).unwrap();
                self.buffer.clear();
                Some(entry)
            }
        }
    }
}

pub struct EntryLineReader(Reader);

impl EntryLineReader {
    pub fn open(path: Option<&Path>) -> io::Result<Self> {
        Ok(Self(Reader::open(path)?))
    }
}

impl IntoIterator for EntryLineReader {
    type Item = (String, Entry);
    type IntoIter = EntryLineIter;

    fn into_iter(self) -> Self::IntoIter {
        EntryLineIter {
            reader: self.0,
            buffer: String::new(),
        }
    }
}

pub struct EntryLineIter {
    reader: Reader,
    buffer: String,
}

impl Iterator for EntryLineIter {
    type Item = (String, Entry);

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.0.read_line(&mut self.buffer).unwrap() {
            0 => None,
            _ => {
                let entry = Entry::from_json(&self.buffer).unwrap();
                let line = self.buffer.clone();
                self.buffer.clear();
                Some((line, entry))
            }
        }
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
pub enum Entry {
    Edge {
        #[serde(rename = "source")]
        src: Ticket,
        #[serde(rename = "target")]
        tgt: Ticket,
        edge_kind: String,
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

impl Entry {
    pub fn from_json(json: &String) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}
