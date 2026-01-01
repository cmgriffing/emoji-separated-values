//! ESV Parser implementation
//!
//! Parses ESV (Emoji Separated Values) data following RFC 4180 conventions
//! adapted for emoji separators.

use crate::error::EsvError;
use crate::validate_separator;
use crate::EsvDocument;
use crate::DEFAULT_SEPARATOR;

/// Parser for ESV data
#[derive(Debug, Clone)]
pub struct EsvParser {
    separator: char,
    has_headers: bool,
    strict_field_count: bool,
}

impl Default for EsvParser {
    fn default() -> Self {
        Self::new()
    }
}

impl EsvParser {
    /// Create a new parser with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            separator: DEFAULT_SEPARATOR,
            has_headers: false,
            strict_field_count: false,
        }
    }

    /// Set a custom emoji separator
    ///
    /// Note: The separator will be validated when `parse()` is called.
    /// Only emoji characters are allowed as separators.
    #[must_use]
    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// Specify whether the first row should be treated as headers
    #[must_use]
    pub fn with_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    /// Enable strict field count validation (all rows must have same number of fields)
    #[must_use]
    pub fn with_strict_field_count(mut self, strict: bool) -> Self {
        self.strict_field_count = strict;
        self
    }

    /// Parse ESV data from a string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The separator is not an emoji character
    /// - A quoted field is not properly closed
    /// - An unexpected character appears after a closing quote
    /// - Field counts are inconsistent (when strict mode is enabled)
    pub fn parse(&self, input: &str) -> Result<EsvDocument, EsvError> {
        // Validate separator is an emoji
        validate_separator(self.separator)?;

        if input.is_empty() {
            return Ok(EsvDocument::new(vec![]));
        }

        let mut records = Vec::new();
        let mut chars = input.chars().peekable();
        let mut line_num = 1;
        let mut expected_field_count: Option<usize> = None;

        loop {
            let (record, ended_at_eof) = self.parse_record(&mut chars, &mut line_num)?;

            // Validate field count if strict mode is enabled
            if self.strict_field_count {
                match expected_field_count {
                    None => expected_field_count = Some(record.len()),
                    Some(expected) if record.len() != expected => {
                        return Err(EsvError::InconsistentFieldCount {
                            expected,
                            found: record.len(),
                            line: line_num,
                        });
                    }
                    _ => {}
                }
            }

            // Don't add empty records at the end (trailing newline)
            let is_trailing_empty =
                ended_at_eof && (record.is_empty() || (record.len() == 1 && record[0].is_empty()));
            if !is_trailing_empty {
                records.push(record);
            }

            if ended_at_eof {
                break;
            }
        }

        // Handle headers if specified
        if self.has_headers && !records.is_empty() {
            let headers = records.remove(0);
            Ok(EsvDocument::with_headers(headers, records))
        } else {
            Ok(EsvDocument::new(records))
        }
    }

    /// Parse a single record (line) from the input
    fn parse_record(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line_num: &mut usize,
    ) -> Result<(Vec<String>, bool), EsvError> {
        let mut fields = Vec::new();
        let mut column = 1;

        loop {
            let (field, terminator) = self.parse_field(chars, *line_num, &mut column)?;
            fields.push(field);

            match terminator {
                FieldTerminator::Separator => {
                    // Continue to next field
                }
                FieldTerminator::LineBreak => {
                    *line_num += 1;
                    return Ok((fields, false));
                }
                FieldTerminator::Eof => {
                    return Ok((fields, true));
                }
            }
        }
    }

    /// Parse a single field from the input
    fn parse_field(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line_num: usize,
        column: &mut usize,
    ) -> Result<(String, FieldTerminator), EsvError> {
        let start_column = *column;

        // Check if field is quoted
        if chars.peek() == Some(&'"') {
            chars.next(); // consume opening quote
            *column += 1;
            self.parse_quoted_field(chars, line_num, start_column, column)
        } else {
            self.parse_unquoted_field(chars, column)
        }
    }

    /// Parse a quoted field (handles escaped quotes and embedded separators/newlines)
    fn parse_quoted_field(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        line_num: usize,
        start_column: usize,
        column: &mut usize,
    ) -> Result<(String, FieldTerminator), EsvError> {
        let mut field = String::new();

        loop {
            match chars.next() {
                Some('"') => {
                    *column += 1;
                    // Check if this is an escaped quote or end of field
                    if chars.peek() == Some(&'"') {
                        // Escaped quote - add single quote to field
                        chars.next();
                        *column += 1;
                        field.push('"');
                    } else {
                        // End of quoted field - check what follows
                        return match chars.peek() {
                            Some(&c) if c == self.separator => {
                                chars.next();
                                *column += 1;
                                Ok((field, FieldTerminator::Separator))
                            }
                            Some('\r') => {
                                chars.next();
                                *column += 1;
                                if chars.peek() == Some(&'\n') {
                                    chars.next();
                                }
                                Ok((field, FieldTerminator::LineBreak))
                            }
                            Some('\n') => {
                                chars.next();
                                Ok((field, FieldTerminator::LineBreak))
                            }
                            None => Ok((field, FieldTerminator::Eof)),
                            Some(&c) => Err(EsvError::UnexpectedCharAfterQuote {
                                line: line_num,
                                column: *column,
                                found: c,
                            }),
                        };
                    }
                }
                Some('\r') => {
                    *column = 1;
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    field.push('\n');
                }
                Some('\n') => {
                    *column = 1;
                    field.push('\n');
                }
                Some(c) => {
                    *column += 1;
                    field.push(c);
                }
                None => {
                    return Err(EsvError::UnclosedQuote {
                        line: line_num,
                        column: start_column,
                    });
                }
            }
        }
    }

    /// Parse an unquoted field
    fn parse_unquoted_field(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        column: &mut usize,
    ) -> Result<(String, FieldTerminator), EsvError> {
        let mut field = String::new();

        loop {
            match chars.peek() {
                Some(&c) if c == self.separator => {
                    chars.next();
                    *column += 1;
                    return Ok((field, FieldTerminator::Separator));
                }
                Some('\r') => {
                    chars.next();
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    return Ok((field, FieldTerminator::LineBreak));
                }
                Some('\n') => {
                    chars.next();
                    return Ok((field, FieldTerminator::LineBreak));
                }
                Some(&c) => {
                    chars.next();
                    *column += 1;
                    field.push(c);
                }
                None => {
                    return Ok((field, FieldTerminator::Eof));
                }
            }
        }
    }
}

/// What terminated a field
#[derive(Debug, Clone, Copy, PartialEq)]
enum FieldTerminator {
    Separator,
    LineBreak,
    Eof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_record() {
        let parser = EsvParser::new();
        let doc = parser.parse("aaa🔥bbb🔥ccc").unwrap();
        assert_eq!(doc.records, vec![vec!["aaa", "bbb", "ccc"]]);
    }

    #[test]
    fn test_parse_multiple_records() {
        let parser = EsvParser::new();
        let doc = parser.parse("aaa🔥bbb🔥ccc\nzzz🔥yyy🔥xxx").unwrap();
        assert_eq!(doc.records.len(), 2);
        assert_eq!(doc.records[0], vec!["aaa", "bbb", "ccc"]);
        assert_eq!(doc.records[1], vec!["zzz", "yyy", "xxx"]);
    }

    #[test]
    fn test_parse_with_trailing_newline() {
        let parser = EsvParser::new();
        let doc = parser.parse("aaa🔥bbb🔥ccc\n").unwrap();
        assert_eq!(doc.records.len(), 1);
        assert_eq!(doc.records[0], vec!["aaa", "bbb", "ccc"]);
    }

    #[test]
    fn test_parse_quoted_fields() {
        let parser = EsvParser::new();
        let doc = parser.parse("\"aaa\"🔥\"bbb\"🔥\"ccc\"").unwrap();
        assert_eq!(doc.records, vec![vec!["aaa", "bbb", "ccc"]]);
    }

    #[test]
    fn test_parse_quoted_with_separator() {
        let parser = EsvParser::new();
        let doc = parser.parse("\"a🔥a\"🔥bbb").unwrap();
        assert_eq!(doc.records, vec![vec!["a🔥a", "bbb"]]);
    }

    #[test]
    fn test_parse_quoted_with_newline() {
        let parser = EsvParser::new();
        let doc = parser.parse("\"a\nb\"🔥ccc").unwrap();
        assert_eq!(doc.records, vec![vec!["a\nb", "ccc"]]);
    }

    #[test]
    fn test_parse_escaped_quotes() {
        let parser = EsvParser::new();
        let doc = parser.parse("\"a\"\"b\"🔥ccc").unwrap();
        assert_eq!(doc.records, vec![vec!["a\"b", "ccc"]]);
    }

    #[test]
    fn test_parse_empty_fields() {
        let parser = EsvParser::new();
        let doc = parser.parse("🔥🔥").unwrap();
        assert_eq!(doc.records, vec![vec!["", "", ""]]);
    }

    #[test]
    fn test_parse_with_headers() {
        let parser = EsvParser::new().with_headers(true);
        let doc = parser.parse("name🔥age\nAlice🔥30").unwrap();
        assert_eq!(
            doc.headers,
            Some(vec!["name".to_string(), "age".to_string()])
        );
        assert_eq!(doc.records, vec![vec!["Alice", "30"]]);
    }

    #[test]
    fn test_parse_custom_separator() {
        let parser = EsvParser::new().with_separator('😀');
        let doc = parser.parse("aaa😀bbb😀ccc").unwrap();
        assert_eq!(doc.records, vec![vec!["aaa", "bbb", "ccc"]]);
    }

    #[test]
    fn test_parse_crlf_line_endings() {
        let parser = EsvParser::new();
        let doc = parser.parse("aaa🔥bbb\r\nccc🔥ddd").unwrap();
        assert_eq!(doc.records.len(), 2);
        assert_eq!(doc.records[0], vec!["aaa", "bbb"]);
        assert_eq!(doc.records[1], vec!["ccc", "ddd"]);
    }

    #[test]
    fn test_parse_unclosed_quote_error() {
        let parser = EsvParser::new();
        let result = parser.parse("\"unclosed");
        assert!(matches!(result, Err(EsvError::UnclosedQuote { .. })));
    }

    #[test]
    fn test_parse_unexpected_char_after_quote() {
        let parser = EsvParser::new();
        let result = parser.parse("\"field\"x🔥other");
        assert!(matches!(
            result,
            Err(EsvError::UnexpectedCharAfterQuote { .. })
        ));
    }

    #[test]
    fn test_parse_strict_field_count() {
        let parser = EsvParser::new().with_strict_field_count(true);
        let result = parser.parse("a🔥b🔥c\nd🔥e");
        assert!(matches!(
            result,
            Err(EsvError::InconsistentFieldCount { .. })
        ));
    }

    #[test]
    fn test_parse_empty_input() {
        let parser = EsvParser::new();
        let doc = parser.parse("").unwrap();
        assert!(doc.records.is_empty());
    }

    #[test]
    fn test_parse_single_field() {
        let parser = EsvParser::new();
        let doc = parser.parse("single").unwrap();
        assert_eq!(doc.records, vec![vec!["single"]]);
    }

    #[test]
    fn test_parse_unicode_content() {
        let parser = EsvParser::new();
        let doc = parser.parse("héllo🔥wörld🔥日本語").unwrap();
        assert_eq!(doc.records, vec![vec!["héllo", "wörld", "日本語"]]);
    }

    #[test]
    fn test_parse_mixed_quoted_unquoted() {
        let parser = EsvParser::new();
        let doc = parser
            .parse("\"quoted\"🔥unquoted🔥\"also quoted\"")
            .unwrap();
        assert_eq!(doc.records, vec![vec!["quoted", "unquoted", "also quoted"]]);
    }
}
