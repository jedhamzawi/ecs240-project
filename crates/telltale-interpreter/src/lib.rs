//! The Telltale interpreter that directly executes a [`Telltale`] program

use telltale_core::Telltale;

#[derive(Debug, Default)]
pub struct TelltaleInterpreter {}

impl TelltaleInterpreter {
    pub fn interpret(&mut self, _prog: &Telltale) {
        unimplemented!("Interpreter is not yet implemented")
    }
}
