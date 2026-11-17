//! Telltale parsing errors

use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TelltaleParseError {
    EmptyInput,
}

impl fmt::Display for TelltaleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "Telltale program is empty!"),
        }
    }
}

impl Error for TelltaleParseError {}
