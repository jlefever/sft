pub mod display;
pub mod dsm;
pub mod exclude;
pub mod format;
pub mod edgekinds;

pub trait CliCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>>;
}
