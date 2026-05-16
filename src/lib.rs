//! Read NDJSON market-data files as an async stream of [`model::market_event::MarketEvent`] values.
//!
//! # Quick start
//!
//! ```no_run
//! use futures::StreamExt;
//! use market_flow::{init_market_event_stream, model::market_event::MarketEvent};
//!
//! # async fn example() -> Result<(), market_flow::MarketFlowError> {
//! let mut stream = init_market_event_stream("events.ndjson").await?;
//! while let Some(result) = stream.next().await {
//!     let event: MarketEvent = result?;
//!     // handle event
//! }
//! # Ok(())
//! # }
//! ```

pub mod model;

mod market_event_stream;

pub use market_event_stream::MarketEventStream;
pub use model::error::MarketFlowError;
pub use model::market_event::{MarketEvent, OrderbookEvent, OrderbookLevel, Side, TradeEvent};

/// Open an NDJSON file at `file` and return a stream of market events.
///
/// Each non-empty line is parsed as one [`model::market_event::MarketEvent`]. Requires a Tokio runtime
/// (see [`MarketEventStream::new`]).
pub async fn init_market_event_stream(file: &str) -> Result<MarketEventStream, MarketFlowError> {
    MarketEventStream::new(file).await
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use futures::StreamExt;
    use rust_decimal::Decimal;

    use crate::model::market_event::{MarketEvent, Side};

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/data/input.ndjson")
    }

    #[tokio::test]
    async fn streams_all_events_from_input_ndjson() {
        let mut stream = crate::init_market_event_stream(fixture_path().to_str().unwrap())
            .await
            .expect("open fixture");

        let mut events = Vec::new();
        while let Some(result) = stream.next().await {
            events.push(result.expect("parse event"));
        }

        assert_eq!(events.len(), 10);

        let MarketEvent::Snapshot(first) = &events[0] else {
            panic!("expected snapshot");
        };
        assert_eq!(first.symbol.as_str(), "BTC-USDT");
        assert_eq!(first.bids.len(), 3);
        assert_eq!(first.bids[0].price, Decimal::from(10_000_000));

        let MarketEvent::Trade(third) = &events[2] else {
            panic!("expected trade");
        };
        assert_eq!(third.side, Side::Buy);
        assert_eq!(third.price, Decimal::from(10_000_100));

        assert!(matches!(events[9], MarketEvent::DepthBatch(_)));
    }
}
