# market-flow

> **0.1.0 — early release.** Use at your own risk. The API and on-disk format may change in patch releases until 1.0.

A small Rust library that reads **newline-delimited JSON (NDJSON)** market data and exposes it as an async stream of typed events. Handy for replaying recorded feeds, building order-book logic, or wiring market data into Tokio pipelines without hand-rolling parsers.

[![crates.io](https://img.shields.io/crates/v/market-flow.svg)](https://crates.io/crates/market-flow)
[![docs.rs](https://docs.rs/market-flow/badge.svg)](https://docs.rs/market-flow)

Each line in the input file is one JSON object: a snapshot, depth update, or trade.

## Input format

Lines look like this (see `src/data/input.ndjson` for a full sample):

```json
{"ts":1000000000,"symbol":"BTC-USDT","type":"snapshot","bids":[[10000000,5],[9999900,8]],"asks":[[10000100,4]]}
{"ts":1000000100,"symbol":"BTC-USDT","type":"depth_batch","bids":[[10000000,7]],"asks":[[10000100,3]]}
{"ts":1000000200,"symbol":"BTC-USDT","type":"trade","side":"buy","price":10000100,"qty":1}
```

| Field | Notes |
|-------|--------|
| `type` | `snapshot`, `depth_batch`, or `trade` |
| `ts` | Milliseconds since Unix epoch |
| `symbol` | Instrument id, e.g. `BTC-USDT` |
| `bids` / `asks` | Arrays of `[price, qty]` (integer JSON numbers) |
| `side` | `buy` or `sell` (trades only) |

Prices and quantities deserialize into [`rust_decimal::Decimal`](https://docs.rs/rust_decimal).

## Dependencies

Consumers need **Tokio** (async runtime) and **futures** (for `StreamExt::next`) in addition to this crate:

```toml
market-flow = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures = "0.3"
```

## Example usage

```rust
use futures::StreamExt;
use market_flow::{
    init_market_event_stream,
    model::market_event::MarketEvent,
    MarketFlowError,
};

#[tokio::main]
async fn main() -> Result<(), MarketFlowError> {
    let mut stream = init_market_event_stream("src/data/input.ndjson").await?;

    while let Some(result) = stream.next().await {
        match result? {
            MarketEvent::Snapshot(book) => {
                println!("snapshot {}: {} bid levels", book.symbol, book.bids.len());
            }
            MarketEvent::DepthBatch(book) => {
                println!("depth {}: {} bid updates", book.symbol, book.bids.len());
            }
            MarketEvent::Trade(trade) => {
                println!("trade {} @ {}", trade.symbol, trade.price);
            }
            MarketEvent::Unknown => {}
        }
    }

    Ok(())
}
```

## Public API

### Crate root (`market_flow`)

| Item | Description |
|------|-------------|
| [`init_market_event_stream`](https://docs.rs/market-flow) | Open an NDJSON file and return a stream |
| [`MarketEventStream`](https://docs.rs/market-flow) | Async `Stream` of parsed events |
| [`model`](https://docs.rs/market-flow/market_flow/model/index.html) | Event types and errors |

### `market_flow::model::market_event`

| Type | Description |
|------|-------------|
| `MarketEvent` | `Snapshot`, `DepthBatch`, `Trade`, or `Unknown` |
| `OrderbookEvent` | Timestamp, symbol, bids, asks |
| `OrderbookLevel` | `price`, `qty` |
| `TradeEvent` | Timestamp, symbol, side, price, qty |
| `Side` | `Buy` or `Sell` |

All model fields are public. Types implement `Deserialize` (for JSON), `Debug`, `Clone`, and common comparison traits.

### `market_flow::model::error`

| Type | Description |
|------|-------------|
| `MarketFlowError` | `IOError` or `DeserializationError` |

### Stream contract

- **Item type:** `Result<MarketEvent, MarketFlowError>`
- **End of file:** stream yields `None`
- **Bad line:** one `Err` per failed line; the stream continues on the next poll (I/O errors aside)

You can also call `MarketEventStream::new(path).await` directly; it behaves the same as `init_market_event_stream`.

## Development

With [just](https://github.com/casey/just) installed:

```bash
just build    # compile
just run      # stream src/data/input.ndjson
just fmt      # apply rustfmt
just lint     # fmt --check + clippy
just audit    # cargo audit (install: cargo install cargo-audit)
just test     # unit + doc tests
just bench    # Criterion benchmarks
```

Or directly:

```bash
cargo test
cargo doc --open
```

## Benchmarking

We use [Criterion](https://github.com/bheisler/criterion.rs) to measure two layers:

| Group | What it measures |
|-------|------------------|
| `parse_line` | `serde_json` deserialization only (snapshot, trade, all 10 fixture lines) |
| `stream_file` | End-to-end: open `input.ndjson`, async line read + parse, drain the stream |

```bash
just bench
# or: cargo bench

# Quick smoke run (few iterations):
cargo bench -- --test

# One benchmark only:
cargo bench -- parse_line
```

HTML reports land under `target/criterion/report/index.html` (open in a browser).

**What to benchmark next** as the project grows:

- Larger NDJSON files (throughput in MB/s)
- Order-book update logic built on top of the stream
- Compare `SmolStr` vs `String` for symbols
- `spawn_blocking` vs Tokio async I/O if you add a sync path

For stable numbers, run benches on a quiet machine with `cargo bench -- --noplot` in CI, or save baselines with Criterion’s `cargo bench -- --save-baseline main` and compare with `--baseline main`.

## Publishing (maintainers)

```bash
cargo publish --dry-run   # verify package contents
cargo publish             # after `cargo login` and creating the GitHub repo
```

Tag releases as `v0.1.0` on GitHub to match `CHANGELOG.md`.

Update `repository` / `homepage` in `Cargo.toml` if your fork lives at a different URL.

## License

Licensed under the [MIT License](LICENSE-MIT).
