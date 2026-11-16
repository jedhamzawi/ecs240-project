//! Args to the `tt` CLI

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Run and interpret a Telltale program directly from a
    /// source file
    Run(RunCommand),
    /// Compile a Telltale source file into target Rust
    Compile(CompileCommand),
}

#[derive(Debug, Clone, Parser)]
pub struct RunCommand {
    /// The path to the Telltale source file (`.tt`) to interpret
    pub source_file: PathBuf,
}

#[derive(Debug, Clone, Parser)]
pub struct CompileCommand {
    /// The path to the Telltale source file (`.tt`) to compile
    pub source_file: PathBuf,
    /// The path to the output
    ///
    /// Default is [`source_file`].rs
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}
