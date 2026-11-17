//! The parser for the Telltale language
//!
//! Takes an input string of possible Telltale syntax and outputs a Telltale program if the string is valid

use telltale_core::Telltale;

mod error;

pub use error::TelltaleParseError;

#[derive(Default)]
pub struct TelltaleParser {}

impl TelltaleParser {
    pub fn parse(&self, raw: &str) -> Result<Telltale, TelltaleParseError> {
        if raw.is_empty() {
            return Err(TelltaleParseError::EmptyInput);
        }
        unimplemented!("Parser is not yet implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_prog() {
        let parser = TelltaleParser::default();
        let raw = "fn main() {}";
        let telltale_prog = parser.parse(raw);
        assert!(
            telltale_prog
                .as_ref()
                .is_ok_and(|prog| prog == &Telltale::default())
        );
    }

    #[test]
    fn empty_input() {
        let parser = TelltaleParser::default();
        let raw = "";
        let telltale_prog = parser.parse(raw);
        assert!(telltale_prog.is_err_and(|err| err == TelltaleParseError::EmptyInput));
    }
}
