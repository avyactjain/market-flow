//! Errors returned when opening files or parsing NDJSON lines.

/// Something went wrong while reading or parsing market data.
#[derive(Debug)]
pub enum MarketFlowError {
    /// JSON on a line did not match the expected event shape.
    DeserializationError(serde_json::error::Error),
    /// The underlying file could not be read (missing path, permissions, etc.).
    IOError(std::io::Error),
}

impl From<std::io::Error> for MarketFlowError {
    fn from(error: std::io::Error) -> Self {
        MarketFlowError::IOError(error)
    }
}

impl From<serde_json::error::Error> for MarketFlowError {
    fn from(value: serde_json::error::Error) -> Self {
        MarketFlowError::DeserializationError(value)
    }
}

impl std::error::Error for MarketFlowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MarketFlowError::DeserializationError(e) => Some(e),
            MarketFlowError::IOError(e) => Some(e),
        }
    }
}

impl std::fmt::Display for MarketFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketFlowError::DeserializationError(e) => {
                write!(f, "failed to parse market event: {e}")
            }
            MarketFlowError::IOError(e) => write!(f, "market data I/O error: {e}"),
        }
    }
}
