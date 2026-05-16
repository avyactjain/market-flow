//! Stream an NDJSON file and print each event (debug format).
//!
//! ```bash
//! cargo run --example read_feed -- src/data/input.ndjson
//! ```

use futures::StreamExt;
use market_flow::init_market_event_stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "src/data/input.ndjson".into());

    let mut stream = init_market_event_stream(&path).await?;

    while let Some(event) = stream.next().await {
        println!("{:?}", event?);
    }

    Ok(())
}
