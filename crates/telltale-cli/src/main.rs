//! The Telltale `tt` CLI that uses Telltale's main components to parse, interpret, and compile
//! Telltale programs

use clap::Parser;

mod args;

use args::Args;

use crate::args::{CompileCommand, RunCommand};

pub fn main() {
    let args = Args::parse();
    match &args.cmd {
        args::Command::Run(run_cmd) => run(run_cmd),
        args::Command::Compile(compile_cmd) => compile(compile_cmd),
    }
}

fn run(cmd: &RunCommand) {
    unimplemented!("Run is not yet implemented")
}

fn compile(cmd: &CompileCommand) {
    unimplemented!("Compile is not yet implemented")
}
