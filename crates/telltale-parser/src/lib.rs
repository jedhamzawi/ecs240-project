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
        let telltale_prog = parser.parse(raw).unwrap();
        // TODO: make sure we parsed the right thing
    }
}
