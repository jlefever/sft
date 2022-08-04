pub mod exclude;
pub mod display;
pub mod dsm;

pub trait CliCommand {
    fn execute(&self) -> ();
}