pub mod exclude;
pub mod display;

pub trait CliCommand {
    fn execute(&self) -> ();
}