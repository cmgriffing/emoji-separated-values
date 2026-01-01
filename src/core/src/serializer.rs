//! ESV Serializer implementation
//!
//! Serializes data to ESV (Emoji Separated Values) format following RFC 4180 conventions
//! adapted for emoji separators.

use crate::EsvDocument;
use crate::DEFAULT_SEPARATOR;

/// Serializer for ESV data
#[derive(Debug, Clone)]
pub struct EsvSerializer {
    separator: char,
    always_quote: bool,
    line_ending: LineEnding,
}

/// Line ending style for serialized output
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEnding {
    /// Unix-style line endings (LF)
    Lf,
    /// Windows-style line endings (CRLF)
    Crlf,
}

impl Default for EsvSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl EsvSerializer {
    /// Create a new serializer with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            separator: DEFAULT_SEPARATOR,
            always_quote: false,
            line_ending: LineEnding::Lf,
        }
    }

    /// Set a custom emoji separator
    #[must_use]
    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// Always quote all fields, even if not necessary
    #[must_use]
    pub fn with_always_quote(mut self, always_quote: bool) -> Self {
        self.always_quote = always_quote;
        self
    }

    /// Set the line ending style
    #[must_use]
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }

    /// Serialize an ESV document to a string
    #[must_use]
    pub fn serialize(&self, doc: &EsvDocument) -> String {
        let mut output = String::new();
        let line_ending = match self.line_ending {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        };

        // Serialize headers if present
        if let Some(headers) = &doc.headers {
            self.serialize_record(headers, &mut output);
            output.push_str(line_ending);
        }

        // Serialize records
        for record in &doc.records {
            self.serialize_record(record, &mut output);
            output.push_str(line_ending);
        }

        output
    }

    /// Serialize records without headers
    #[must_use]
    pub fn serialize_records(&self, records: &[Vec<String>]) -> String {
        self.serialize(&EsvDocument::new(records.to_vec()))
    }

    /// Serialize a single record (row)
    fn serialize_record(&self, record: &[String], output: &mut String) {
        for (i, field) in record.iter().enumerate() {
            if i > 0 {
                output.push(self.separator);
            }
            self.serialize_field(field, output);
        }
    }

    /// Serialize a single field, quoting if necessary
    fn serialize_field(&self, field: &str, output: &mut String) {
        let needs_quoting = self.always_quote || self.field_needs_quoting(field);

        if needs_quoting {
            output.push('"');
            for c in field.chars() {
                if c == '"' {
                    output.push_str("\"\"");
                } else {
                    output.push(c);
                }
            }
            output.push('"');
        } else {
            output.push_str(field);
        }
    }

    /// Check if a field needs to be quoted
    fn field_needs_quoting(&self, field: &str) -> bool {
        field
            .chars()
            .any(|c| c == self.separator || c == '"' || c == '\n' || c == '\r')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![vec![
            "aaa".to_string(),
            "bbb".to_string(),
            "ccc".to_string(),
        ]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "aaa🔥bbb🔥ccc\n");
    }

    #[test]
    fn test_serialize_multiple_records() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![
            vec!["aaa".to_string(), "bbb".to_string()],
            vec!["ccc".to_string(), "ddd".to_string()],
        ]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "aaa🔥bbb\nccc🔥ddd\n");
    }

    #[test]
    fn test_serialize_with_headers() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::with_headers(
            vec!["name".to_string(), "age".to_string()],
            vec![vec!["Alice".to_string(), "30".to_string()]],
        );
        let output = serializer.serialize(&doc);
        assert_eq!(output, "name🔥age\nAlice🔥30\n");
    }

    #[test]
    fn test_serialize_field_with_separator() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![vec!["a🔥b".to_string(), "ccc".to_string()]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "\"a🔥b\"🔥ccc\n");
    }

    #[test]
    fn test_serialize_field_with_newline() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![vec!["a\nb".to_string(), "ccc".to_string()]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "\"a\nb\"🔥ccc\n");
    }

    #[test]
    fn test_serialize_field_with_quotes() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![vec!["a\"b".to_string(), "ccc".to_string()]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "\"a\"\"b\"🔥ccc\n");
    }

    #[test]
    fn test_serialize_always_quote() {
        let serializer = EsvSerializer::new().with_always_quote(true);
        let doc = EsvDocument::new(vec![vec!["aaa".to_string(), "bbb".to_string()]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "\"aaa\"🔥\"bbb\"\n");
    }

    #[test]
    fn test_serialize_crlf_line_ending() {
        let serializer = EsvSerializer::new().with_line_ending(LineEnding::Crlf);
        let doc = EsvDocument::new(vec![vec!["aaa".to_string(), "bbb".to_string()]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "aaa🔥bbb\r\n");
    }

    #[test]
    fn test_serialize_custom_separator() {
        let serializer = EsvSerializer::new().with_separator('😀');
        let doc = EsvDocument::new(vec![vec!["aaa".to_string(), "bbb".to_string()]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "aaa😀bbb\n");
    }

    #[test]
    fn test_serialize_empty_fields() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![vec![String::new(), String::new(), String::new()]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "🔥🔥\n");
    }

    #[test]
    fn test_serialize_unicode_content() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![vec![
            "héllo".to_string(),
            "wörld".to_string(),
            "日本語".to_string(),
        ]]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "héllo🔥wörld🔥日本語\n");
    }

    #[test]
    fn test_serialize_records_helper() {
        let serializer = EsvSerializer::new();
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        let output = serializer.serialize_records(&records);
        assert_eq!(output, "a🔥b\nc🔥d\n");
    }

    #[test]
    fn test_serialize_empty_document() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![]);
        let output = serializer.serialize(&doc);
        assert_eq!(output, "");
    }

    #[test]
    fn test_serialize_complex_escaping() {
        let serializer = EsvSerializer::new();
        let doc = EsvDocument::new(vec![vec![
            "has \"quotes\" and 🔥 separator\nand newline".to_string()
        ]]);
        let output = serializer.serialize(&doc);
        assert_eq!(
            output,
            "\"has \"\"quotes\"\" and 🔥 separator\nand newline\"\n"
        );
    }
}
