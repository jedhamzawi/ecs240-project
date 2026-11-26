//! The Telltale interpreter that takes a [`Telltale`] program as input and outputs
//! a string of valid Rust that is semantically equivalent to the program.
//!
//! That is, for every program in the set of valid triples `(INPUT, prog, OUTPUT)`
//! in the semantics for Telltale, the compiler must produce target Rust that,
//! when compiled by `rustc` and executed, the resulting program will
//! have the same input/output pairs as the interpreter.

use telltale_core::Telltale;

#[derive(Debug, Default)]
pub struct TelltaleCompiler {}

impl TelltaleCompiler {
    pub fn compile(&mut self, _prog: &Telltale) -> String {
        unimplemented!("Compiler is not yet implemented")
    }
}
