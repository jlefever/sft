use std::fs;
use std::io;

use std::path::PathBuf;

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
