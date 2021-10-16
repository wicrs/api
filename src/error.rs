use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    WICRSError(#[from] wicrs_server::error::ApiError),
    #[error(transparent)]
    Tungstenite(#[from] tungstenite::Error),
    #[cfg(feature = "use-tokio")]
    #[error(transparent)]
    TokioTungstenite(#[from] tokio_tungstenite::tungstenite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unable to send command result to executor")]
    TokioMpscSend,
    #[error("websocket connection closed")]
    WsClosed,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
