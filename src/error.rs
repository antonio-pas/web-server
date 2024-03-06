// Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  IO(#[from] tokio::io::Error),

  #[error(transparent)]
  Utf8(#[from] std::string::FromUtf8Error),

  #[error("invalid request method")]
  InvalidMethod,

  #[error("invalid request header")]
  InvalidHeader,

  #[error("invalid request line")]
  InvalidRequestLine,
}
