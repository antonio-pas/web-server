use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net;

use crate::http::*;

#[derive(Debug, thiserror::Error)]
pub enum ParseHttpRequestError {
  #[error(transparent)]
  IO(#[from] tokio::io::Error),

  #[error("bad header: {0}")]
  BadHeader(String),

  #[error("bad request line: {0}")]
  BadRequestLine(String)
}

type Result<T> = std::result::Result<T, ParseHttpRequestError>;

fn parse_header(text: &str) -> Result<(String, String)> {
  let (key, value) = text.split_once(':').ok_or(ParseHttpRequestError::BadHeader(text.into()))?;
  let key = key.to_string();
  let value = value.trim_start().trim_end_matches("\r\n").to_string();
  Ok((key, value))
}
fn parse_request_line(text: &str) -> Result<(RequestMethod, String)> {
  let mut request_line_parts = text.splitn(3, ' ').into_iter().map(|s| s.to_string());
  let method: RequestMethod = request_line_parts
    .next()
    .ok_or(ParseHttpRequestError::BadRequestLine(text.into()))?
    .try_into().map_err(|_| ParseHttpRequestError::BadRequestLine(text.into()))?;
  let url = request_line_parts.next().ok_or(ParseHttpRequestError::BadRequestLine(text.into()))?;
  Ok((method, url))
}

pub async fn parse_request(stream: &mut net::TcpStream) -> Result<Request> {
  let mut buf_reader = BufReader::new(stream);
  let mut request_line = String::new();
  buf_reader.read_line(&mut request_line).await?;
  let (method, url) = parse_request_line(&request_line)?;
  let mut headers = Headers::new();
  loop {
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?;
    if line == "\r\n" {
      break;
    }
    let header = parse_header(&line)?;
    headers.insert(header.0, header.1);
  }
  let body = if let Some(len) = headers.get("Content-Length") {
    let len = len.parse().map_err(|_| ParseHttpRequestError::BadHeader("Content-Length is not number".into()))?;
    let mut body = [0u8; 1024];
    buf_reader.read(&mut body).await?;
    let slice = &body[..len];
    slice.to_vec()
  } else {
    vec![]
  };
  let request = Request::new(method, url, headers, body.into());
  Ok(request)
}
