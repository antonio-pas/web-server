// HTTP request and response structures

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
  #[error("invalid response code {0}")]
  InvalidResponseCode(u16),
}

mod common;
pub use common::Body;
pub use common::Headers;

mod code;
pub use code::StatusCode;

mod method;
pub use method::RequestMethod;

mod request;
pub use request::Request;

mod response;
pub use response::Response;
pub use response::ResponseBuilder;
