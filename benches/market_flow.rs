//! Criterion benchmarks for JSON parsing and NDJSON streaming.
//!
//! Run: `cargo bench` or `just bench`
//! Report: `target/criterion/report/index.html`

use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use futures::StreamExt;
use market_flow::{init_market_event_stream, model::market_event::MarketEvent};
use serde_json::from_str;

const FIXTURE: &str = include_str!("../src/data/input.ndjson");

fn fixture_lines() -> Vec<&'static str> {
    FIXTURE.lines().filter(|line| !line.is_empty()).collect()
}

fn parse_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_line");
    group.throughput(Throughput::Bytes(FIXTURE.len() as u64));

    let snapshot = fixture_lines()[0];
    let trade = fixture_lines()[2];

    group.bench_with_input(
        BenchmarkId::new("market_event", "snapshot"),
        &snapshot,
        |b, line| {
            b.iter(|| black_box(from_str::<MarketEvent>(line).unwrap()));
        },
    );

    group.bench_with_input(
        BenchmarkId::new("market_event", "trade"),
        &trade,
        |b, line| {
            b.iter(|| black_box(from_str::<MarketEvent>(line).unwrap()));
        },
    );

    group.bench_function("market_event_all_lines", |b| {
        let lines = fixture_lines();
        b.iter(|| {
            for line in &lines {
                black_box(from_str::<MarketEvent>(line).unwrap());
            }
        });
    });

    group.finish();
}

fn stream_group(c: &mut Criterion) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/data/input.ndjson");
    let path = path.to_str().unwrap().to_string();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    let mut group = c.benchmark_group("stream_file");
    group.throughput(Throughput::Bytes(FIXTURE.len() as u64));

    group.bench_function("init_and_drain", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut stream = init_market_event_stream(&path).await.expect("open fixture");
                let mut count = 0usize;
                while let Some(event) = stream.next().await {
                    black_box(event.expect("parse event"));
                    count += 1;
                }
                black_box(count);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, parse_group, stream_group);
criterion_main!(benches);
