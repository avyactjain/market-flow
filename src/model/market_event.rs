//! Typed market events deserialized from NDJSON lines.
//!
//! Each line is a JSON object with a `type` field. Order book levels are encoded
//! as `[price, qty]` arrays; timestamps are millisecond Unix epochs.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use smol_str::SmolStr;

/// One record from the market-data feed.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MarketEvent {
    /// Full order book for a symbol.
    Snapshot(OrderbookEvent),
    /// Incremental bid/ask updates.
    #[serde(rename = "depth_batch")]
    DepthBatch(OrderbookEvent),
    /// A single executed trade.
    Trade(TradeEvent),
    /// Unrecognized or placeholder event type.
    Unknown,
}

/// Shared payload for [`MarketEvent::Snapshot`] and [`MarketEvent::DepthBatch`].
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderbookEvent {
    /// Event time (UTC).
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    /// Instrument identifier, e.g. `BTC-USDT`.
    pub symbol: SmolStr,
    /// Bid levels as `(price, qty)` pairs.
    pub bids: Vec<OrderbookLevel>,
    /// Ask levels as `(price, qty)` pairs.
    pub asks: Vec<OrderbookLevel>,
}

/// One price level on the book.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderbookLevel {
    pub price: Decimal,
    pub qty: Decimal,
}

/// A trade print.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradeEvent {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    pub symbol: SmolStr,
    pub side: Side,
    pub price: Decimal,
    pub qty: Decimal,
}

/// Aggressor side for a trade.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn parse(json: &str) -> MarketEvent {
        serde_json::from_str(json).expect("valid market event json")
    }

    #[test]
    fn deserializes_snapshot() {
        let json = r#"{"ts":1000000000,"symbol":"BTC-USDT","type":"snapshot","bids":[[10000000,5],[9999900,8]],"asks":[[10000100,4]]}"#;
        let event = parse(json);

        let MarketEvent::Snapshot(book) = event else {
            panic!("expected snapshot variant");
        };
        assert_eq!(book.ts, Utc.timestamp_millis_opt(1_000_000_000).unwrap());
        assert_eq!(book.symbol.as_str(), "BTC-USDT");
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.bids[0].price, Decimal::from(10_000_000));
        assert_eq!(book.bids[0].qty, Decimal::from(5));
        assert_eq!(book.asks[0].price, Decimal::from(10_000_100));
        assert_eq!(book.asks[0].qty, Decimal::from(4));
    }

    #[test]
    fn deserializes_depth_batch() {
        let json = r#"{"ts":1000000100,"symbol":"BTC-USDT","type":"depth_batch","bids":[[10000000,7]],"asks":[[10000100,3]]}"#;
        let event = parse(json);

        let MarketEvent::DepthBatch(book) = event else {
            panic!("expected depth_batch variant");
        };
        assert_eq!(book.symbol.as_str(), "BTC-USDT");
        assert_eq!(book.bids[0].qty, Decimal::from(7));
    }

    #[test]
    fn deserializes_trade() {
        let json = r#"{"ts":1000000200,"symbol":"BTC-USDT","type":"trade","side":"buy","price":10000100,"qty":1}"#;
        let event = parse(json);

        let MarketEvent::Trade(trade) = event else {
            panic!("expected trade variant");
        };
        assert_eq!(trade.side, Side::Buy);
        assert_eq!(trade.price, Decimal::from(10_000_100));
        assert_eq!(trade.qty, Decimal::from(1));
    }

    #[test]
    fn deserializes_all_fixture_lines() {
        let fixture = include_str!("../data/input.ndjson");
        let events: Vec<MarketEvent> = fixture
            .lines()
            .filter(|line| !line.is_empty())
            .map(parse)
            .collect();

        assert_eq!(events.len(), 10);
        assert!(matches!(events[0], MarketEvent::Snapshot(_)));
        assert!(matches!(events[2], MarketEvent::Trade(_)));
    }
}
