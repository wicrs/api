#[derive(Debug)]
pub enum Error {
    WICRSError(wicrs_server::error::Error),
    ReqwestClientBuild,
    SerializeFailed,
    Connection,
    UnexpectedResponse,
    LoginNotComplete,
    TokenExpired,
}

impl From<wicrs_server::error::Error> for Error {
    fn from(err: wicrs_server::error::Error) -> Self {
        Self::WICRSError(err)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
