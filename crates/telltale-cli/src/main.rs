//! The Telltale `tt` CLI that uses Telltale's main components to parse, interpret, and compile
//! Telltale programs

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
// TODO: maybe remove anyhow and use our own error?
use anyhow::{Context, Error};

mod args;

use args::Args;
use telltale_compiler::TelltaleCompiler;
use telltale_core::Telltale;
use telltale_interpreter::TelltaleInterpreter;
use telltale_parser::TelltaleParser;

use crate::args::{CompileCommand, RunCommand};

pub fn main() -> Result<(), Error> {
    let args = Args::parse();
    match &args.cmd {
        args::Command::Run(run_cmd) => run(run_cmd),
        args::Command::Compile(compile_cmd) => compile(compile_cmd),
    }
}

fn parse(source_file: &Path) -> Result<Telltale, Error> {
    let parser = TelltaleParser::default();
    // TODO: maybe don't read to string? it's probably fine though
    let raw_source = std::fs::read_to_string(source_file)
        .with_context(|| format!("Error reading source file '{}'", source_file.display()))?;
    parser
        .parse(&raw_source)
        .with_context(|| format!("Error parsing source file '{}'", source_file.display()))
}

fn run(cmd: &RunCommand) -> Result<(), Error> {
    let program = parse(&cmd.source_file)?;
    let mut interpreter = TelltaleInterpreter::default();
    interpreter.interpret(&program);
    Ok(())
}

fn construct_output_path(source_path: &Path) -> Result<PathBuf, Error> {
    // get the current directory
    Ok(std::env::current_dir()
        .context("Error getting the current directory")?
        .join(
            // and default the output file name to the source file name with the `rs` extension
            source_path
                .file_name()
                .map(|file_name| Path::new(file_name).with_extension("rs"))
                .ok_or(Error::msg("Source file path is empty"))?,
        ))
}

fn compile(cmd: &CompileCommand) -> Result<(), Error> {
    let program = parse(&cmd.source_path)?;
    let mut compiler = TelltaleCompiler::default();
    let compiled = compiler.compile(&program);
    if cmd.stdout {
        let mut stdout = std::io::stdout();
        stdout
            .write_all(compiled.as_bytes())
            .context("Error writing to stdout")?;
    } else {
        // open our output file
        // TODO: should we open first? probably not; compiling could take a long time,
        // and we don't want to have the file lock for too long;
        // then again, we don't want to do all this work and then find that the
        // path was invalid...
        let output_path = match &cmd.output {
            Some(output_path) => output_path.clone(),
            None => construct_output_path(&cmd.source_path)
                .context("Error constructing default output path")?,
        };
        // make sure the source is not the same as the output;
        // basically nobody would want to overwrite their source to the output
        if output_path == cmd.source_path {
            return Err(Error::msg(format!(
                "the input file '{}' would be overwritten by the generated Rust file",
                cmd.source_path.display()
            )));
        }
        // create the output file
        let mut output_file = File::create(&output_path).with_context(|| {
            format!(
                "Error creating output Rust file '{}'",
                output_path.display()
            )
        })?;
        output_file
            .write_all(compiled.as_bytes())
            .with_context(|| {
                format!(
                    "Error writing to output Rust file '{}'",
                    output_path.display()
                )
            })?;
    }
    Ok(())
}
