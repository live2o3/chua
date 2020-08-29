use thiserror::Error as TError;

#[derive(TError, Debug)]
pub enum ChuaError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Send(#[from] futures_channel::mpsc::SendError),

    #[error(transparent)]
    Canceled(#[from] futures_channel::oneshot::Canceled),

    #[cfg(target_arch = "wasm32")]
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    Other(String),
}

impl From<String> for ChuaError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for ChuaError {
    fn from(s: &str) -> Self {
        Self::Other(s.into())
    }
}

pub type ChuaResult<T> = Result<T, ChuaError>;
