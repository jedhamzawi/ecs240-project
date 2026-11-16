//! Telltale parsing errors

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TelltaleParseError {}

impl fmt::Display for TelltaleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: display errors for each case with info
        match self {
            _ => write!(f, "TelltaleParseError"),
        }
    }
}

impl Error for TelltaleParseError {}
