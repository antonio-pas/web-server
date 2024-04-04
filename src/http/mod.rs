// HTTP request and response structures

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
  #[error("invalid response code: {0}")]
  InvalidResponseCode(u16),
  #[error("invalid header: {0}")]
  InvalidHeader(String),
  #[error("invalid request line: {0}")]
  InvalidRequestLine(String),
  #[error("header of wrong type, {0} should not be {1}")]
  InvalidHeaderValue(String, String),
}

mod common;
pub use common::Body;
pub use common::Headers;

mod uri;
pub use uri::Uri;

mod code;
pub use code::StatusCode;

mod method;
pub use method::RequestMethod;

mod request;
pub use request::Request;

mod response;
pub use response::IntoResponse;
pub use response::Response;
pub use response::ResponseBuilder;
