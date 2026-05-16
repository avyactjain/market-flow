//! Async line-by-line reader that turns an NDJSON file into a [`Stream`] of [`MarketEvent`]s.

use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use tokio_stream::wrappers::LinesStream;

use crate::model::{error::MarketFlowError, market_event::MarketEvent};

/// A [`Stream`] over market events read from a single NDJSON file.
///
/// Construct with [`MarketEventStream::new`] or [`crate::init_market_event_stream`],
/// then poll with [`futures::StreamExt::next`].
pub struct MarketEventStream {
    line_stream: LinesStream<BufReader<File>>,
}

impl MarketEventStream {
    /// Open `path` and prepare to stream one event per line.
    pub async fn new(path: &str) -> Result<Self, MarketFlowError> {
        let file = File::open(path).await?;
        let line_stream = LinesStream::new(BufReader::new(file).lines());
        Ok(Self { line_stream })
    }
}

impl Stream for MarketEventStream {
    type Item = Result<MarketEvent, MarketFlowError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.line_stream).poll_next(cx) {
            Poll::Ready(Some(line)) => Poll::Ready(Some(parse_line(line))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

fn parse_line(line: Result<String, std::io::Error>) -> Result<MarketEvent, MarketFlowError> {
    let line = line?;
    Ok(serde_json::from_str(&line)?)
}
