use std::fmt;
use std::io;

use crate::http::IntoResponse;
use crate::http::Response;

#[derive(Debug)]
pub struct Error {
  kind: Kind,
  cause: Option<Box<dyn std::error::Error>>,
}

#[derive(Debug)]
pub enum Kind {
  Parse(Parse),
  UnsupportedVersion,
  Io,
}

impl Error {
  pub fn new(kind: Kind) -> Self {
    Self { kind, cause: None }
  }
  pub fn with<E: Into<Box<dyn std::error::Error>>>(mut self, err: E) -> Self {
    self.cause = Some(err.into());
    self
  }
  pub fn new_io(err: io::Error) -> Self {
    Self::new(Kind::Io).with(err)
  }
}

#[derive(Debug)]
pub enum Parse {
  Method,
  Uri,
  Header,
  Version,
}

impl From<Parse> for Error {
  fn from(err: Parse) -> Self {
    Self::new(Kind::Parse(err))
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    self.cause.as_deref()
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self.kind {
        Kind::Parse(Parse::Uri) => "invalid uri parsed",
        Kind::Parse(Parse::Header) => "invalid header parsed",
        Kind::Parse(Parse::Method) => "invalid method parsed",
        Kind::Parse(Parse::Version) => "invalid version parsed",
        Kind::UnsupportedVersion => "unsupported version",
        Kind::Io => "io error",
      }
    )
  }
}

impl IntoResponse for Error {
  fn into_response(self) -> crate::http::Response {
    match self.kind {
      Kind::Parse(_) => Response::builder().status(400).body("Sorry! Bad request.").unwrap(),
      Kind::UnsupportedVersion => Response::builder().status(505).body("HTTP version not supported.").unwrap(),
      Kind::Io => Response::builder().status(500).body("Sorry! Internal server error.").unwrap()
    }
  }
}
