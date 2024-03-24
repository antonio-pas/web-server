use super::*;

#[derive(Debug, Default)]
pub enum StatusCode {
  #[default]
  Ok,
  Created,
  NotFound,
  Unauthorized,
  BadRequest,
  InternalServerError,
}

impl ToString for StatusCode {
  fn to_string(&self) -> String {
    match self {
      Self::Ok => "200 OK",
      Self::Created => "201 Created",
      Self::NotFound => "404 Not Found",
      Self::Unauthorized => "403 Unauthorized",
      Self::BadRequest => "400 Bad Request",
      Self::InternalServerError => "500 Internal Server Error",
    }
    .to_string()
  }
}
impl TryFrom<u16> for StatusCode {
  type Error = HttpError;
  fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
    match value {
      200 => Ok(Self::Ok),
      201 => Ok(Self::Created),
      404 => Ok(Self::NotFound),
      403 => Ok(Self::Unauthorized),
      400 => Ok(Self::BadRequest),
      500 => Ok(Self::InternalServerError),
      _ => Err(HttpError::InvalidResponseCode(value)),
    }
  }
}
