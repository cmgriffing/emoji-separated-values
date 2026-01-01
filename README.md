# Emoji Separated Values (ESV)

A Rust library and CLI tool for working with Emoji Separated Values (ESV), a data format similar to CSV but using emoji as field separators.

**Default separator:** 🔥 (fire emoji, U+1F525)

## Features

- **Core Library (`esv-core`)**: Parse and serialize ESV data
- **CLI Tool (`esv`)**: Command-line interface for working with ESV files
- **Custom Separators**: Use any emoji as a field separator
- **RFC 4180 Compatible**: Follows CSV conventions for quoting and escaping
- **Header Support**: Optional header row handling
- **Strict Mode**: Validate consistent field counts across records

## Installation

### From Source

```bash
cargo install --path src/cli
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
esv-core = { git = "https://github.com/cmgriffing/emoji-separated-values" }
```

## ESV Format Specification

Based on [RFC 4180](https://datatracker.ietf.org/doc/html/rfc4180) for CSV, adapted for emoji separators:

1. **Records**: Each record is located on a separate line, delimited by a line break (CRLF or LF).

2. **Trailing Line Break**: The last record in the file may or may not have an ending line break.

3. **Header Line**: There may be an optional header line appearing as the first line of the file with the same format as normal record lines.

4. **Field Separator**: Within the header and each record, fields are separated by the emoji separator (default: 🔥). Each line should contain the same number of fields throughout the file.

5. **Quoting**: Each field may or may not be enclosed in double quotes. If fields are not enclosed with double quotes, then double quotes may not appear inside the fields.

6. **Special Characters**: Fields containing line breaks (CRLF/LF), double quotes, or the emoji separator should be enclosed in double quotes.

7. **Escaping Quotes**: If double-quotes are used to enclose fields, then a double-quote appearing inside a field must be escaped by preceding it with another double quote (`""`).

### ABNF Grammar

```abnf
file = [header CRLF] record *(CRLF record) [CRLF]

header = name *(EMOJI name)

record = field *(EMOJI field)

name = field

field = (escaped / non-escaped)

escaped = DQUOTE *(TEXTDATA / EMOJI / CR / LF / 2DQUOTE) DQUOTE

non-escaped = *TEXTDATA

EMOJI = %x1F525  ; 🔥 (default, configurable)

CR = %x0D

DQUOTE = %x22

LF = %x0A

CRLF = CR LF

TEXTDATA = any character except EMOJI, CR, LF, or DQUOTE
```

### Example

```
name🔥age🔥city
Alice🔥30🔥New York
Bob🔥25🔥"Los Angeles"
"Charlie ""Chuck"""🔥35🔥"San Francisco"
```

## CLI Usage

### Parse ESV to JSON

```bash
# Parse from file
esv parse data.esv

# Parse from stdin
cat data.esv | esv parse

# Parse with headers
esv parse -H data.esv

# Pretty-print JSON output
esv parse --format json-pretty data.esv

# Use custom separator
esv parse --separator '🌟' data.esv

# Enable strict field count validation
esv parse --strict data.esv

# Output to file
esv parse data.esv -o output.json
```

### Serialize JSON to ESV

```bash
# Serialize from stdin
echo '{"records":[["a","b"],["c","d"]]}' | esv serialize

# Serialize with headers
echo '{"headers":["x","y"],"records":[["1","2"]]}' | esv serialize

# Use custom separator
esv serialize --separator '⭐' input.json

# Always quote all fields
esv serialize --always-quote input.json

# Use CRLF line endings
esv serialize --line-ending crlf input.json

# Output to file
esv serialize input.json -o output.esv
```

### Validate ESV Data

```bash
# Validate file
esv validate data.esv

# Validate with strict field count
esv validate --strict data.esv

# Validate with headers
esv validate -H data.esv
```

### Display Format Information

```bash
# Show general info
esv info

# Show default separator
esv info --separator

# Show format specification
esv info --spec
```

## Library Usage

### Basic Parsing

```rust
use esv_core::{parse, parse_with_headers};

// Parse ESV data
let input = "aaa🔥bbb🔥ccc\nzzz🔥yyy🔥xxx";
let doc = parse(input).unwrap();

assert_eq!(doc.records.len(), 2);
assert_eq!(doc.records[0], vec!["aaa", "bbb", "ccc"]);

// Parse with headers
let input = "name🔥age🔥city\nAlice🔥30🔥NYC";
let doc = parse_with_headers(input).unwrap();

assert_eq!(doc.headers, Some(vec!["name".into(), "age".into(), "city".into()]));
assert_eq!(doc.records.len(), 1);
```

### Basic Serialization

```rust
use esv_core::{serialize, serialize_with_headers};

// Serialize records
let records = vec![
    vec!["aaa".into(), "bbb".into(), "ccc".into()],
    vec!["zzz".into(), "yyy".into(), "xxx".into()],
];
let output = serialize(&records);
assert_eq!(output, "aaa🔥bbb🔥ccc\nzzz🔥yyy🔥xxx\n");

// Serialize with headers
let headers = vec!["name".into(), "age".into()];
let records = vec![vec!["Alice".into(), "30".into()]];
let output = serialize_with_headers(&headers, &records);
assert_eq!(output, "name🔥age\nAlice🔥30\n");
```

### Advanced Parser Configuration

```rust
use esv_core::EsvParser;

let parser = EsvParser::new()
    .with_separator('🌟')           // Custom separator
    .with_headers(true)              // Treat first row as headers
    .with_strict_field_count(true);  // Validate consistent field counts

let doc = parser.parse("a🌟b\n1🌟2").unwrap();
```

### Advanced Serializer Configuration

```rust
use esv_core::{EsvSerializer, EsvDocument, LineEnding};

let serializer = EsvSerializer::new()
    .with_separator('⭐')           // Custom separator
    .with_always_quote(true)         // Always quote fields
    .with_line_ending(LineEnding::Crlf);  // Windows-style line endings

let doc = EsvDocument::new(vec![
    vec!["hello".into(), "world".into()],
]);
let output = serializer.serialize(&doc);
```

### Working with EsvDocument

```rust
use esv_core::EsvDocument;

// Create document without headers
let doc = EsvDocument::new(vec![
    vec!["a".into(), "b".into()],
    vec!["c".into(), "d".into()],
]);

// Create document with headers
let doc = EsvDocument::with_headers(
    vec!["col1".into(), "col2".into()],
    vec![vec!["a".into(), "b".into()]],
);

// Document methods
assert_eq!(doc.len(), 1);           // Number of records
assert!(!doc.is_empty());           // Check if empty
assert_eq!(doc.field_count(), Some(2));  // Fields per record
```

### Error Handling

```rust
use esv_core::{parse, EsvError};

let result = parse("\"unclosed quote");
match result {
    Ok(doc) => println!("Parsed {} records", doc.len()),
    Err(EsvError::UnclosedQuote { line, column }) => {
        eprintln!("Unclosed quote at line {}, column {}", line, column);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## API Reference

### Types

- **`EsvDocument`**: Represents a parsed ESV document with optional headers and records
- **`EsvParser`**: Configurable parser for ESV data
- **`EsvSerializer`**: Configurable serializer for ESV data
- **`EsvError`**: Error type for parsing failures
- **`LineEnding`**: Enum for line ending style (`Lf` or `Crlf`)

### Constants

- **`DEFAULT_SEPARATOR`**: The default emoji separator (`🔥`, U+1F525)

### Functions

- **`parse(input: &str)`**: Parse ESV with default settings
- **`parse_with_headers(input: &str)`**: Parse ESV treating first row as headers
- **`serialize(records: &[Vec<String>])`**: Serialize records to ESV
- **`serialize_with_headers(headers: &[String], records: &[Vec<String>])`**: Serialize with headers

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for details.
