#![allow(dead_code)]
use std::fmt::{Debug, Display};
use std::path::PathBuf;

use clap::{ArgAction, Command};

use clap::{arg, command, value_parser};

#[derive(Debug)]
enum EdgeOperator {
    Any,
    All,
    Src,
    Tgt,
}

impl EdgeOperator {
    fn to_str(&self) -> &'static str {
        match self {
            EdgeOperator::Any => "any",
            EdgeOperator::All => "all",
            EdgeOperator::Src => "src",
            EdgeOperator::Tgt => "tgt",
        }
    }
}

impl Display for EdgeOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PathKind {
    NilPath,
    RelPath,
    AbsPath,
}

impl PathKind {
    fn to_str(&self) -> &'static str {
        match self {
            PathKind::NilPath => "nilpath",
            PathKind::RelPath => "relpath",
            PathKind::AbsPath => "abspath",
        }
    }
}

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([name] "Optional name to operate on"))
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                -d --debug "Turn debugging information on"
            )
            .action(ArgAction::Count),
        )
        .subcommand(
            Command::new("test")
                .about("does testing things")
                .arg(arg!(-l --list "lists test values").action(ArgAction::SetTrue)),
        )
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = matches.get_one::<String>("name") {
        println!("Value for name: {}", name);
    }

    if let Some(config_path) = matches.get_one::<PathBuf>("config") {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match matches
        .get_one::<u8>("debug")
        .expect("Count's are defaulted")
    {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    if let Some(matches) = matches.subcommand_matches("test") {
        // "$ myapp test" was run
        if *matches.get_one::<bool>("list").expect("defaulted by clap") {
            // "$ myapp test -l" was run
            println!("Printing testing lists...");
        } else {
            println!("Not printing testing lists...");
        }
    }

    // Continued program logic goes here...
}
