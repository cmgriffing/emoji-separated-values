//! ESV (Emoji Separated Values) Core Library
//!
//! This library provides functionality for parsing and serializing ESV data,
//! following a format similar to RFC 4180 for CSV but using emoji as field separators.
//!
//! # ESV Format Definition
//!
//! Based on RFC 4180 for CSV, adapted for emoji separators:
//!
//! 1. Each record is located on a separate line, delimited by a line break (CRLF or LF).
//! 2. The last record in the file may or may not have an ending line break.
//! 3. There may be an optional header line appearing as the first line of the file.
//! 4. Within the header and each record, fields are separated by the emoji separator.
//! 5. Each field may or may not be enclosed in double quotes.
//! 6. Fields containing line breaks, double quotes, or the emoji separator should be
//!    enclosed in double quotes.
//! 7. Double quotes inside a field must be escaped by preceding with another double quote.
//!
//! # Default Separator
//!
//! The default emoji separator is 🔥 (fire emoji, U+1F525).

mod error;
mod parser;
mod serializer;

pub use error::EsvError;
pub use parser::EsvParser;
pub use serializer::EsvSerializer;
pub use serializer::LineEnding;

/// Default emoji separator (fire emoji 🔥)
pub const DEFAULT_SEPARATOR: char = '🔥';

/// Represents a parsed ESV document
#[derive(Debug, Clone, PartialEq)]
pub struct EsvDocument {
    /// Optional header row
    pub headers: Option<Vec<String>>,
    /// Data records
    pub records: Vec<Vec<String>>,
}

impl EsvDocument {
    /// Create a new ESV document without headers
    #[must_use]
    pub fn new(records: Vec<Vec<String>>) -> Self {
        Self {
            headers: None,
            records,
        }
    }

    /// Create a new ESV document with headers
    #[must_use]
    pub fn with_headers(headers: Vec<String>, records: Vec<Vec<String>>) -> Self {
        Self {
            headers: Some(headers),
            records,
        }
    }

    /// Returns the number of records (excluding headers)
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if there are no records
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the number of fields per record (based on first record or headers)
    pub fn field_count(&self) -> Option<usize> {
        self.headers
            .as_ref()
            .map(Vec::len)
            .or_else(|| self.records.first().map(Vec::len))
    }
}

/// Parse ESV data from a string using the default separator
///
/// # Errors
///
/// Returns an error if the input contains invalid ESV syntax (unclosed quotes,
/// unexpected characters after quotes, etc.)
pub fn parse(input: &str) -> Result<EsvDocument, EsvError> {
    EsvParser::new().parse(input)
}

/// Parse ESV data from a string, treating the first row as headers
///
/// # Errors
///
/// Returns an error if the input contains invalid ESV syntax (unclosed quotes,
/// unexpected characters after quotes, etc.)
pub fn parse_with_headers(input: &str) -> Result<EsvDocument, EsvError> {
    EsvParser::new().with_headers(true).parse(input)
}

/// Serialize records to ESV format using the default separator
#[must_use]
pub fn serialize(records: &[Vec<String>]) -> String {
    EsvSerializer::new().serialize(&EsvDocument::new(records.to_vec()))
}

/// Serialize records with headers to ESV format using the default separator
#[must_use]
pub fn serialize_with_headers(headers: &[String], records: &[Vec<String>]) -> String {
    EsvSerializer::new().serialize(&EsvDocument::with_headers(
        headers.to_vec(),
        records.to_vec(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = "aaa🔥bbb🔥ccc\nzzz🔥yyy🔥xxx";
        let doc = parse(input).unwrap();
        assert_eq!(doc.records.len(), 2);
        assert_eq!(doc.records[0], vec!["aaa", "bbb", "ccc"]);
        assert_eq!(doc.records[1], vec!["zzz", "yyy", "xxx"]);
    }

    #[test]
    fn test_parse_with_headers() {
        let input = "name🔥age🔥city\nAlice🔥30🔥NYC\nBob🔥25🔥LA";
        let doc = parse_with_headers(input).unwrap();
        assert_eq!(
            doc.headers,
            Some(vec![
                "name".to_string(),
                "age".to_string(),
                "city".to_string()
            ])
        );
        assert_eq!(doc.records.len(), 2);
    }

    #[test]
    fn test_serialize_simple() {
        let records = vec![
            vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string()],
            vec!["zzz".to_string(), "yyy".to_string(), "xxx".to_string()],
        ];
        let output = serialize(&records);
        assert_eq!(output, "aaa🔥bbb🔥ccc\nzzz🔥yyy🔥xxx\n");
    }

    #[test]
    fn test_serialize_with_headers() {
        let headers = vec!["name".to_string(), "age".to_string()];
        let records = vec![vec!["Alice".to_string(), "30".to_string()]];
        let output = serialize_with_headers(&headers, &records);
        assert_eq!(output, "name🔥age\nAlice🔥30\n");
    }

    #[test]
    fn test_roundtrip() {
        let original = "field1🔥field2🔥field3\nvalue1🔥value2🔥value3\n";
        let doc = parse(original).unwrap();
        let serialized = EsvSerializer::new().serialize(&doc);
        let reparsed = parse(&serialized).unwrap();
        assert_eq!(doc, reparsed);
    }

    #[test]
    fn test_esv_document_methods() {
        let doc = EsvDocument::with_headers(
            vec!["a".to_string(), "b".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
        );
        assert_eq!(doc.len(), 1);
        assert!(!doc.is_empty());
        assert_eq!(doc.field_count(), Some(2));
    }
}
