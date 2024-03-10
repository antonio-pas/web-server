// HTTP request and response structures
use crate::prelude::*;

#[derive(Eq, Hash, PartialEq, Debug, Copy, Clone)]
pub enum RequestMethod {
  Get,
  Post,
  Put,
  Patch,
  Delete,
}
#[derive(Debug)]
pub struct Url {
  // TODO: support anchors and query strings
  path_parts: Vec<String>,
}
pub type Headers = std::collections::HashMap<String, String>;
#[derive(Debug)]
pub struct Request {
  method: RequestMethod,
  url: String,
  headers: Headers,
  body: Vec<u8>,
}

impl Request {
  pub fn new(method: RequestMethod, url: String, headers: Headers, body: Vec<u8>) -> Self {
    Self {
      method,
      url,
      headers,
      body,
    }
  }

  pub fn method(&self) -> &RequestMethod {
    &self.method
  }

  pub fn url(&self) -> &String {
    &self.url
  }

  pub fn headers(&self) -> &Headers {
    &self.headers
  }

  pub fn body(&self) -> &[u8] {
    &self.body
  }
}

impl TryFrom<String> for RequestMethod {
  type Error = Error;
  fn try_from(value: String) -> Result<Self> {
    match value.as_str() {
      "GET" => Ok(RequestMethod::Get),
      "POST" => Ok(RequestMethod::Post),
      "PUT" => Ok(RequestMethod::Put),
      "DELETE" => Ok(RequestMethod::Delete),
      "PATCH" => Ok(RequestMethod::Patch),
      _ => Err(Error::InvalidMethod),
    }
  }
}

#[derive(Debug)]
pub enum StatusCode {
  Ok,
  Created,
  NotFound,
  Unauthorized,
  BadRequest,
}

impl ToString for StatusCode {
  fn to_string(&self) -> String {
    match self {
      Self::Ok => "200 OK",
      Self::Created => "201 Created",
      Self::NotFound => "404 Not Found",
      Self::Unauthorized => "403 Unauthorized",
      Self::BadRequest => "400 Bad Request",
    }
    .to_string()
  }
}
#[derive(Debug)]
pub struct Response {
  code: StatusCode,
  headers: Headers,
  body: Vec<u8>,
}

impl Response {
  pub fn new(code: StatusCode, headers: Headers, body: Vec<u8>) -> Self {
    Self {
      code,
      headers,
      body,
    }
  }
  pub fn from_plain_text(code: StatusCode, body: &str) -> Self {
    Self {
      code,
      headers: Headers::from([
        ("Content-Length".to_string(), body.len().to_string())]),
      body: body.bytes().collect(),
    }
  }
  pub fn from_html_ok(html: &str) -> Self {
    Self {
      code: StatusCode::Ok,
      headers: Headers::from([
        ("Content-Length".to_string(), html.len().to_string()),
        (
          "Content-Type".to_string(),
          "text/html; charset=utf-8".to_string(),
        ),
      ]),
      body: html.bytes().collect(),
    }
  }
  pub fn as_bytes(&self) -> Vec<u8> {
    let status_line = format!("HTTP/1.1 {}\r\n", self.code.to_string());
    let headers = self
      .headers
      .iter()
      .map(|(k, v)| format!("{k}: {v}"))
      .collect::<Vec<_>>()
      .join("\r\n");
    [
      status_line.as_bytes(),
      headers.as_bytes(),
      b"\r\n\r\n",
      &self.body,
      b"\r\n",
    ]
    .concat()
  }
}
