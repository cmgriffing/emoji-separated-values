//! ESV CLI Library
//!
//! This module provides the command-line interface functionality for working with
//! ESV (Emoji Separated Values) files.

use std::fs;
use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use esv_core::{EsvDocument, EsvParser, EsvSerializer, LineEnding, DEFAULT_SEPARATOR};

/// ESV (Emoji Separated Values) command-line tool
#[derive(Parser, Debug)]
#[command(name = "esv")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Parse ESV data and output as JSON
    Parse(ParseArgs),

    /// Serialize JSON data to ESV format
    Serialize(SerializeArgs),

    /// Validate ESV data
    Validate(ValidateArgs),

    /// Display information about ESV format
    Info(InfoArgs),
}

#[derive(Args, Debug)]
pub struct ParseArgs {
    /// Input file (use - for stdin)
    #[arg(default_value = "-")]
    pub input: String,

    /// Output file (use - for stdout)
    #[arg(short, long, default_value = "-")]
    pub output: String,

    /// Treat first row as headers
    #[arg(short = 'H', long)]
    pub headers: bool,

    /// Custom emoji separator
    #[arg(short, long)]
    pub separator: Option<char>,

    /// Enable strict field count validation
    #[arg(long)]
    pub strict: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Args, Debug)]
pub struct SerializeArgs {
    /// Input JSON file (use - for stdin)
    #[arg(default_value = "-")]
    pub input: String,

    /// Output file (use - for stdout)
    #[arg(short, long, default_value = "-")]
    pub output: String,

    /// Custom emoji separator
    #[arg(short, long)]
    pub separator: Option<char>,

    /// Always quote all fields
    #[arg(long)]
    pub always_quote: bool,

    /// Line ending style
    #[arg(long, value_enum, default_value = "lf")]
    pub line_ending: LineEndingArg,
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Input file (use - for stdin)
    #[arg(default_value = "-")]
    pub input: String,

    /// Custom emoji separator
    #[arg(short, long)]
    pub separator: Option<char>,

    /// Enable strict field count validation
    #[arg(long)]
    pub strict: bool,

    /// Treat first row as headers
    #[arg(short = 'H', long)]
    pub headers: bool,
}

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Show default separator
    #[arg(long)]
    pub separator: bool,

    /// Show format specification
    #[arg(long)]
    pub spec: bool,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    /// JSON output
    #[default]
    Json,
    /// Pretty-printed JSON
    JsonPretty,
    /// Simple text output (one field per line, records separated by blank lines)
    Text,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LineEndingArg {
    /// Unix-style (LF)
    Lf,
    /// Windows-style (CRLF)
    Crlf,
}

impl Cli {
    /// Run the CLI application
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input file cannot be read
    /// - Output file cannot be written
    /// - ESV parsing fails
    /// - JSON parsing fails
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::Parse(args) => run_parse(args),
            Commands::Serialize(args) => run_serialize(args),
            Commands::Validate(args) => run_validate(args),
            Commands::Info(args) => {
                run_info(args);
                Ok(())
            }
        }
    }
}

fn run_parse(args: &ParseArgs) -> Result<()> {
    let input = read_input(&args.input)?;

    let mut parser = EsvParser::new();
    if let Some(sep) = args.separator {
        parser = parser.with_separator(sep);
    }
    if args.headers {
        parser = parser.with_headers(true);
    }
    if args.strict {
        parser = parser.with_strict_field_count(true);
    }

    let doc = parser
        .parse(&input)
        .context("Failed to parse ESV input")?;

    let output = match args.format {
        OutputFormat::Json => format_as_json(&doc, false)?,
        OutputFormat::JsonPretty => format_as_json(&doc, true)?,
        OutputFormat::Text => format_as_text(&doc),
    };

    write_output(&args.output, &output)?;
    Ok(())
}

fn run_serialize(args: &SerializeArgs) -> Result<()> {
    let input = read_input(&args.input)?;

    let doc: EsvDocument = parse_json_input(&input)?;

    let mut serializer = EsvSerializer::new();
    if let Some(sep) = args.separator {
        serializer = serializer.with_separator(sep);
    }
    if args.always_quote {
        serializer = serializer.with_always_quote(true);
    }
    serializer = serializer.with_line_ending(match args.line_ending {
        LineEndingArg::Lf => LineEnding::Lf,
        LineEndingArg::Crlf => LineEnding::Crlf,
    });

    let output = serializer.serialize(&doc);
    write_output(&args.output, &output)?;
    Ok(())
}

fn run_validate(args: &ValidateArgs) -> Result<()> {
    let input = read_input(&args.input)?;

    let mut parser = EsvParser::new();
    if let Some(sep) = args.separator {
        parser = parser.with_separator(sep);
    }
    if args.headers {
        parser = parser.with_headers(true);
    }
    if args.strict {
        parser = parser.with_strict_field_count(true);
    }

    match parser.parse(&input) {
        Ok(doc) => {
            let record_count = doc.len();
            let field_count = doc.field_count().unwrap_or(0);
            let has_headers = doc.headers.is_some();

            println!("✅ Valid ESV");
            println!("   Records: {record_count}");
            println!("   Fields per record: {field_count}");
            println!("   Has headers: {has_headers}");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Invalid ESV: {e}");
            std::process::exit(1);
        }
    }
}

fn run_info(args: &InfoArgs) {
    if args.separator {
        println!(
            "Default separator: {DEFAULT_SEPARATOR} (U+{:04X})",
            DEFAULT_SEPARATOR as u32
        );
        return;
    }

    if args.spec {
        println!("ESV (Emoji Separated Values) Format Specification");
        println!("=================================================");
        println!();
        println!("Based on RFC 4180 for CSV, adapted for emoji separators:");
        println!();
        println!("1. Each record is on a separate line, delimited by LF or CRLF.");
        println!("2. The last record may or may not have an ending line break.");
        println!("3. An optional header line may appear as the first line.");
        println!("4. Fields are separated by the emoji separator (default: {DEFAULT_SEPARATOR}).");
        println!("5. Fields may or may not be enclosed in double quotes.");
        println!("6. Fields containing line breaks, quotes, or the separator");
        println!("   should be enclosed in double quotes.");
        println!("7. Double quotes inside a field are escaped by doubling (\"\").");
        return;
    }

    // Default: show both
    println!("ESV (Emoji Separated Values)");
    println!(
        "Default separator: {DEFAULT_SEPARATOR} (U+{:04X})",
        DEFAULT_SEPARATOR as u32
    );
    println!();
    println!("Use --spec for format specification");
    println!("Use --help for available commands");
}

// Helper functions

fn read_input(path: &str) -> Result<String> {
    if path == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        Ok(buffer)
    } else {
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {path}"))
    }
}

fn write_output(path: &str, content: &str) -> Result<()> {
    if path == "-" {
        io::stdout()
            .write_all(content.as_bytes())
            .context("Failed to write to stdout")?;
    } else {
        fs::write(path, content).with_context(|| format!("Failed to write file: {path}"))?;
    }
    Ok(())
}

fn format_as_json(doc: &EsvDocument, pretty: bool) -> Result<String> {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<Vec<String>>,
        records: Vec<Vec<String>>,
    }

    let output = JsonOutput {
        headers: doc.headers.clone(),
        records: doc.records.clone(),
    };

    if pretty {
        serde_json::to_string_pretty(&output).context("Failed to serialize to JSON")
    } else {
        serde_json::to_string(&output).context("Failed to serialize to JSON")
    }
}

fn format_as_text(doc: &EsvDocument) -> String {
    use std::fmt::Write;

    let mut output = String::new();

    if let Some(headers) = &doc.headers {
        output.push_str("# Headers\n");
        for header in headers {
            output.push_str(header);
            output.push('\n');
        }
        output.push('\n');
    }

    for (i, record) in doc.records.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        let _ = writeln!(output, "# Record {}", i + 1);
        for field in record {
            output.push_str(field);
            output.push('\n');
        }
    }

    output
}

fn parse_json_input(input: &str) -> Result<EsvDocument> {
    #[derive(serde::Deserialize)]
    struct JsonInput {
        headers: Option<Vec<String>>,
        records: Vec<Vec<String>>,
    }

    let parsed: JsonInput = serde_json::from_str(input).context("Failed to parse JSON input")?;

    Ok(match parsed.headers {
        Some(headers) => EsvDocument::with_headers(headers, parsed.records),
        None => EsvDocument::new(parsed.records),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_as_json() {
        let doc = EsvDocument::new(vec![vec!["a".to_string(), "b".to_string()]]);
        let json = format_as_json(&doc, false).unwrap();
        assert_eq!(json, r#"{"records":[["a","b"]]}"#);
    }

    #[test]
    fn test_format_as_json_with_headers() {
        let doc = EsvDocument::with_headers(
            vec!["x".to_string(), "y".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
        );
        let json = format_as_json(&doc, false).unwrap();
        assert_eq!(json, r#"{"headers":["x","y"],"records":[["1","2"]]}"#);
    }

    #[test]
    fn test_parse_json_input() {
        let input = r#"{"records":[["a","b"],["c","d"]]}"#;
        let doc = parse_json_input(input).unwrap();
        assert_eq!(doc.records.len(), 2);
        assert!(doc.headers.is_none());
    }

    #[test]
    fn test_parse_json_input_with_headers() {
        let input = r#"{"headers":["x","y"],"records":[["1","2"]]}"#;
        let doc = parse_json_input(input).unwrap();
        assert_eq!(doc.headers, Some(vec!["x".to_string(), "y".to_string()]));
        assert_eq!(doc.records.len(), 1);
    }
}
