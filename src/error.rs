use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    WICRSError(#[from] wicrs_server::error::Error),
    #[error(transparent)]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("websocket connection closed")]
    WsClosed,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
