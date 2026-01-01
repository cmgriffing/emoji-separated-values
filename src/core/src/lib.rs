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
//!
//! # Separator Validation
//!
//! The separator must be an emoji character. ASCII characters and other non-emoji
//! Unicode characters are not allowed. This ensures the format remains distinct from
//! CSV and other traditional delimited formats.

mod error;
mod parser;
mod serializer;

pub use error::EsvError;
pub use parser::EsvParser;
pub use serializer::EsvSerializer;
pub use serializer::LineEnding;

/// Default emoji separator (fire emoji 🔥)
pub const DEFAULT_SEPARATOR: char = '🔥';

/// Check if a character is an emoji
///
/// This function checks if a character falls within common emoji Unicode ranges.
/// It covers:
/// - Miscellaneous Symbols and Pictographs (U+1F300-U+1F5FF)
/// - Emoticons (U+1F600-U+1F64F)
/// - Transport and Map Symbols (U+1F680-U+1F6FF)
/// - Supplemental Symbols and Pictographs (U+1F900-U+1F9FF)
/// - Symbols and Pictographs Extended-A (U+1FA00-U+1FA6F)
/// - Symbols and Pictographs Extended-B (U+1FA70-U+1FAFF)
/// - Dingbats (U+2700-U+27BF)
/// - Miscellaneous Symbols (U+2600-U+26FF)
/// - Miscellaneous Symbols and Arrows (U+2B00-U+2BFF)
/// - Various other emoji ranges
#[must_use]
pub fn is_emoji(c: char) -> bool {
    let code = c as u32;

    // Common emoji ranges
    matches!(
        code,
        // Miscellaneous Symbols and Pictographs
        0x1F300..=0x1F5FF |
        // Emoticons
        0x1F600..=0x1F64F |
        // Transport and Map Symbols
        0x1F680..=0x1F6FF |
        // Supplemental Symbols and Pictographs
        0x1F900..=0x1F9FF |
        // Symbols and Pictographs Extended-A
        0x1FA00..=0x1FA6F |
        // Symbols and Pictographs Extended-B
        0x1FA70..=0x1FAFF |
        // Dingbats (includes ❤ at U+2764)
        0x2700..=0x27BF |
        // Miscellaneous Symbols (includes ☀, ☁, etc.)
        0x2600..=0x26FF |
        // Miscellaneous Symbols and Arrows (includes ⭐ at U+2B50)
        0x2B00..=0x2BFF |
        // Enclosed Alphanumeric Supplement (some emoji)
        0x1F100..=0x1F1FF |
        // Mahjong Tiles
        0x1F000..=0x1F02F |
        // Domino Tiles
        0x1F030..=0x1F09F |
        // Playing Cards
        0x1F0A0..=0x1F0FF |
        // Miscellaneous Technical (some emoji like ⌚)
        0x2300..=0x23FF |
        // Arrows (some are emoji)
        0x2190..=0x21FF |
        // CJK Symbols (some emoji)
        0x3000..=0x303F |
        // Enclosed CJK Letters and Months
        0x3200..=0x32FF |
        // Geometric Shapes (some emoji)
        0x25A0..=0x25FF |
        // Box Drawing and Block Elements (some used as emoji)
        0x2580..=0x259F
    )
}

/// Validate that a separator is an emoji
///
/// # Errors
///
/// Returns `EsvError::InvalidSeparator` if the character is not an emoji.
pub fn validate_separator(separator: char) -> Result<(), EsvError> {
    if is_emoji(separator) {
        Ok(())
    } else {
        Err(EsvError::InvalidSeparator { separator })
    }
}

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

    // Emoji validation tests
    #[test]
    fn test_is_emoji_common_emoji() {
        // Fire emoji (default separator)
        assert!(is_emoji('🔥'));
        // Smiley face
        assert!(is_emoji('😀'));
        // Heart
        assert!(is_emoji('❤'));
        // Star
        assert!(is_emoji('⭐'));
        // Rocket
        assert!(is_emoji('🚀'));
        // Pizza
        assert!(is_emoji('🍕'));
        // Thumbs up
        assert!(is_emoji('👍'));
    }

    #[test]
    fn test_is_emoji_ascii_not_emoji() {
        // ASCII letters
        assert!(!is_emoji('a'));
        assert!(!is_emoji('Z'));
        // ASCII digits
        assert!(!is_emoji('0'));
        assert!(!is_emoji('9'));
        // Common CSV separators
        assert!(!is_emoji(','));
        assert!(!is_emoji(';'));
        assert!(!is_emoji('\t'));
        assert!(!is_emoji('|'));
        // Other ASCII
        assert!(!is_emoji(' '));
        assert!(!is_emoji('\n'));
        assert!(!is_emoji('"'));
    }

    #[test]
    fn test_is_emoji_non_emoji_unicode() {
        // Regular Unicode letters (not emoji)
        assert!(!is_emoji('é'));
        assert!(!is_emoji('ñ'));
        assert!(!is_emoji('日'));
        assert!(!is_emoji('本'));
        // Currency symbols
        assert!(!is_emoji('€'));
        assert!(!is_emoji('£'));
    }

    #[test]
    fn test_validate_separator_valid() {
        assert!(validate_separator('🔥').is_ok());
        assert!(validate_separator('😀').is_ok());
        assert!(validate_separator('🚀').is_ok());
        assert!(validate_separator('⭐').is_ok());
    }

    #[test]
    fn test_validate_separator_invalid() {
        let result = validate_separator(',');
        assert!(matches!(
            result,
            Err(EsvError::InvalidSeparator { separator: ',' })
        ));

        let result = validate_separator('\t');
        assert!(matches!(
            result,
            Err(EsvError::InvalidSeparator { separator: '\t' })
        ));

        let result = validate_separator('|');
        assert!(matches!(
            result,
            Err(EsvError::InvalidSeparator { separator: '|' })
        ));
    }

    #[test]
    fn test_parser_rejects_ascii_separator() {
        let parser = EsvParser::new().with_separator(',');
        let result = parser.parse("a,b,c");
        assert!(matches!(
            result,
            Err(EsvError::InvalidSeparator { separator: ',' })
        ));
    }

    #[test]
    fn test_parser_accepts_emoji_separator() {
        let parser = EsvParser::new().with_separator('😀');
        let result = parser.parse("a😀b😀c");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.records[0], vec!["a", "b", "c"]);
    }

    #[test]
    fn test_serializer_rejects_ascii_separator() {
        let serializer = EsvSerializer::new().with_separator(',');
        let doc = EsvDocument::new(vec![vec!["a".to_string(), "b".to_string()]]);
        let result = serializer.try_serialize(&doc);
        assert!(matches!(
            result,
            Err(EsvError::InvalidSeparator { separator: ',' })
        ));
    }

    #[test]
    fn test_serializer_accepts_emoji_separator() {
        let serializer = EsvSerializer::new().with_separator('😀');
        let doc = EsvDocument::new(vec![vec!["a".to_string(), "b".to_string()]]);
        let result = serializer.try_serialize(&doc);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "a😀b\n");
    }
}
