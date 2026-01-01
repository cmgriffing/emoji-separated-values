//! Benchmarks for ESV parsing and serialization

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use esv_core::{parse, parse_with_headers, serialize, EsvDocument, EsvParser, EsvSerializer};

fn generate_simple_esv(rows: usize, cols: usize) -> String {
    let mut result = String::new();
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 {
                result.push('🔥');
            }
            result.push_str(&format!("value_{row}_{col}"));
        }
        result.push('\n');
    }
    result
}

fn generate_esv_with_headers(rows: usize, cols: usize) -> String {
    let mut result = String::new();
    // Header row
    for col in 0..cols {
        if col > 0 {
            result.push('🔥');
        }
        result.push_str(&format!("column_{col}"));
    }
    result.push('\n');
    // Data rows
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 {
                result.push('🔥');
            }
            result.push_str(&format!("value_{row}_{col}"));
        }
        result.push('\n');
    }
    result
}

fn generate_esv_with_quotes(rows: usize, cols: usize) -> String {
    let mut result = String::new();
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 {
                result.push('🔥');
            }
            // Every other field is quoted, some with special characters
            if col % 2 == 0 {
                result.push_str(&format!("\"value with \"\"quotes\"\" {row}_{col}\""));
            } else {
                result.push_str(&format!("simple_{row}_{col}"));
            }
        }
        result.push('\n');
    }
    result
}

fn generate_esv_with_newlines(rows: usize, cols: usize) -> String {
    let mut result = String::new();
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 {
                result.push('🔥');
            }
            // Some fields contain newlines
            if col % 3 == 0 {
                result.push_str(&format!("\"line1\nline2\nvalue_{row}_{col}\""));
            } else {
                result.push_str(&format!("value_{row}_{col}"));
            }
        }
        result.push('\n');
    }
    result
}

fn generate_records(rows: usize, cols: usize) -> Vec<Vec<String>> {
    (0..rows)
        .map(|row| {
            (0..cols)
                .map(|col| format!("value_{row}_{col}"))
                .collect()
        })
        .collect()
}

fn bench_parse_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_simple");

    for size in [10, 100, 1000].iter() {
        let input = generate_simple_esv(*size, 5);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| parse(black_box(input)));
        });
    }

    group.finish();
}

fn bench_parse_with_headers(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_with_headers");

    for size in [10, 100, 1000].iter() {
        let input = generate_esv_with_headers(*size, 5);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| parse_with_headers(black_box(input)));
        });
    }

    group.finish();
}

fn bench_parse_quoted(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_quoted");

    for size in [10, 100, 1000].iter() {
        let input = generate_esv_with_quotes(*size, 5);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| parse(black_box(input)));
        });
    }

    group.finish();
}

fn bench_parse_with_newlines(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_with_newlines");

    for size in [10, 100, 1000].iter() {
        let input = generate_esv_with_newlines(*size, 5);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| parse(black_box(input)));
        });
    }

    group.finish();
}

fn bench_parse_custom_separator(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_custom_separator");

    for size in [10, 100, 1000].iter() {
        // Generate with custom separator
        let input = generate_simple_esv(*size, 5).replace('🔥', "⭐");
        let parser = EsvParser::new().with_separator('⭐');
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| parser.parse(black_box(input)));
        });
    }

    group.finish();
}

fn bench_parse_strict_mode(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_strict_mode");

    for size in [10, 100, 1000].iter() {
        let input = generate_simple_esv(*size, 5);
        let parser = EsvParser::new().with_strict_field_count(true);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| parser.parse(black_box(input)));
        });
    }

    group.finish();
}

fn bench_serialize_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_simple");

    for size in [10, 100, 1000].iter() {
        let records = generate_records(*size, 5);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &records, |b, records| {
            b.iter(|| serialize(black_box(records)));
        });
    }

    group.finish();
}

fn bench_serialize_with_headers(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_with_headers");

    for size in [10, 100, 1000].iter() {
        let headers: Vec<String> = (0..5).map(|i| format!("column_{i}")).collect();
        let records = generate_records(*size, 5);
        let doc = EsvDocument::with_headers(headers, records);
        let serializer = EsvSerializer::new();
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &doc, |b, doc| {
            b.iter(|| serializer.serialize(black_box(doc)));
        });
    }

    group.finish();
}

fn bench_serialize_always_quote(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_always_quote");

    for size in [10, 100, 1000].iter() {
        let records = generate_records(*size, 5);
        let doc = EsvDocument::new(records);
        let serializer = EsvSerializer::new().with_always_quote(true);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &doc, |b, doc| {
            b.iter(|| serializer.serialize(black_box(doc)));
        });
    }

    group.finish();
}

fn bench_serialize_needs_quoting(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_needs_quoting");

    for size in [10, 100, 1000].iter() {
        // Generate records that need quoting
        let records: Vec<Vec<String>> = (0..*size)
            .map(|row| {
                (0..5)
                    .map(|col| {
                        if col % 2 == 0 {
                            format!("value with \"quotes\" {row}_{col}")
                        } else if col % 3 == 0 {
                            format!("value🔥with🔥separator_{row}_{col}")
                        } else {
                            format!("line1\nline2_{row}_{col}")
                        }
                    })
                    .collect()
            })
            .collect();
        let doc = EsvDocument::new(records);
        let serializer = EsvSerializer::new();
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &doc, |b, doc| {
            b.iter(|| serializer.serialize(black_box(doc)));
        });
    }

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");

    for size in [10, 100, 1000].iter() {
        let input = generate_simple_esv(*size, 5);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                let doc = parse(black_box(input)).unwrap();
                EsvSerializer::new().serialize(&doc)
            });
        });
    }

    group.finish();
}

fn bench_wide_records(c: &mut Criterion) {
    let mut group = c.benchmark_group("wide_records");

    for cols in [10, 50, 100].iter() {
        let input = generate_simple_esv(100, *cols);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parse", cols),
            &input,
            |b, input| {
                b.iter(|| parse(black_box(input)));
            },
        );
    }

    for cols in [10, 50, 100].iter() {
        let records = generate_records(100, *cols);
        group.throughput(Throughput::Elements(100));
        group.bench_with_input(
            BenchmarkId::new("serialize", cols),
            &records,
            |b, records| {
                b.iter(|| serialize(black_box(records)));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_with_headers,
    bench_parse_quoted,
    bench_parse_with_newlines,
    bench_parse_custom_separator,
    bench_parse_strict_mode,
    bench_serialize_simple,
    bench_serialize_with_headers,
    bench_serialize_always_quote,
    bench_serialize_needs_quoting,
    bench_roundtrip,
    bench_wide_records,
);

criterion_main!(benches);
