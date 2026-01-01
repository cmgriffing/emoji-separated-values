//! Error types for ESV parsing and serialization

use std::fmt;

/// Errors that can occur during ESV parsing or serialization
#[derive(Debug, Clone, PartialEq)]
pub enum EsvError {
    /// Unclosed quoted field
    UnclosedQuote { line: usize, column: usize },
    /// Unexpected character after closing quote
    UnexpectedCharAfterQuote {
        line: usize,
        column: usize,
        found: char,
    },
    /// Inconsistent field count across records
    InconsistentFieldCount {
        expected: usize,
        found: usize,
        line: usize,
    },
    /// Empty input
    EmptyInput,
    /// Invalid UTF-8 in input
    InvalidUtf8,
}

impl fmt::Display for EsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EsvError::UnclosedQuote { line, column } => {
                write!(f, "unclosed quote at line {line}, column {column}")
            }
            EsvError::UnexpectedCharAfterQuote {
                line,
                column,
                found,
            } => {
                write!(
                    f,
                    "unexpected character '{found}' after closing quote at line {line}, column {column}"
                )
            }
            EsvError::InconsistentFieldCount {
                expected,
                found,
                line,
            } => {
                write!(
                    f,
                    "inconsistent field count at line {line}: expected {expected} fields, found {found}"
                )
            }
            EsvError::EmptyInput => write!(f, "empty input"),
            EsvError::InvalidUtf8 => write!(f, "invalid UTF-8 in input"),
        }
    }
}

impl std::error::Error for EsvError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EsvError::UnclosedQuote { line: 1, column: 5 };
        assert_eq!(err.to_string(), "unclosed quote at line 1, column 5");

        let err = EsvError::UnexpectedCharAfterQuote {
            line: 2,
            column: 10,
            found: 'x',
        };
        assert_eq!(
            err.to_string(),
            "unexpected character 'x' after closing quote at line 2, column 10"
        );

        let err = EsvError::InconsistentFieldCount {
            expected: 3,
            found: 2,
            line: 5,
        };
        assert_eq!(
            err.to_string(),
            "inconsistent field count at line 5: expected 3 fields, found 2"
        );

        let err = EsvError::EmptyInput;
        assert_eq!(err.to_string(), "empty input");

        let err = EsvError::InvalidUtf8;
        assert_eq!(err.to_string(), "invalid UTF-8 in input");
    }
}
