use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    WICRSError(#[from] wicrs_server::ApiError),
    #[error("failed to build the reqwest HTTP client")]
    ReqwestClientBuild,
    #[error("unable to connect to server or server did not respond")]
    Connection,
    #[error("the server responded in an unexpected way")]
    UnexpectedResponse,
    #[error("the login steps must be completed before building the client")]
    LoginNotComplete,
    #[error("the given authentication token has expired")]
    TokenExpired,

}

pub type Result<T, E = Error> = std::result::Result<T, E>;
