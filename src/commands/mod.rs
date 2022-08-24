pub mod exclude;
pub mod display;
pub mod dsm;
pub mod format;

pub trait CliCommand {
    fn execute(&self) -> ();
}