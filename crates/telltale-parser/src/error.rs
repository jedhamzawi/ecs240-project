//! Telltale parser errors

use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TelltaleParseError {
    // Empty source code
    EmptyInput,

    // Invalid character in source code
    UnexpectedCharacter(char),

    // Invalid or unexpected token
    UnexpectedToken,

    // Invalid integer literal
    InvalidNumber,

    // Program does not contain main()
    MissingMainFunction,

    // Duplicate function declaration
    DuplicateFunction(String),
}

// Display error messages
impl fmt::Display for TelltaleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelltaleParseError::EmptyInput => {
                write!(f, "Telltale program is empty!")
            }

            TelltaleParseError::UnexpectedCharacter(c) => {
                write!(f, "Unexpected character: '{}'", c)
            }

            TelltaleParseError::UnexpectedToken => {
                write!(f, "Unexpected token in Telltale program")
            }

            TelltaleParseError::InvalidNumber => {
                write!(f, "Invalid integer literal")
            }

            TelltaleParseError::MissingMainFunction => {
                write!(f, "Program must contain a main function")
            }

            TelltaleParseError::DuplicateFunction(name) => {
                write!(f, "Duplicate function declaration: {}", name)
            }
        }
    }
}

// Implement standard Rust Error trait
impl Error for TelltaleParseError {}
