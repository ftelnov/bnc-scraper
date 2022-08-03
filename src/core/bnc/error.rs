use reqwest::Error;
use thiserror::Error;

/// Errors that BNC fetch part can return.
#[derive(Error, Debug)]
pub enum BncError {
    #[error("Reqwest crate could not proceed with given data. Origin error: {}", .0)]
    RequestError(reqwest::Error),

    #[error("Serialization framework was unable to process entity. Possibly some binance entity is malformed. Origin serde error: {}", .0)]
    SerdeError(serde_json::Error),

    #[error("Interaction with WS module failed. Origin error: {}", .0)]
    WsError(tokio_tungstenite::tungstenite::Error),

    #[error("Could not send thread's data to the thread's master.")]
    DataTransmitError,

    #[error("Data was rejected by predicate. Possibly some conditions were unmet.")]
    DataRejected,
}

pub type BncResult<T> = Result<T, BncError>;

impl From<reqwest::Error> for BncError {
    fn from(err: Error) -> Self {
        Self::RequestError(err)
    }
}

impl From<serde_json::Error> for BncError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for BncError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WsError(err)
    }
}
