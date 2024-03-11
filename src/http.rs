// HTTP request and response structures

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
  #[error("invalid response code {0}")]
  InvalidResponseCode(u16)
}

#[derive(Debug, Copy, Clone)]
pub enum RequestMethod {
  Get,
  Post,
  Put,
  Patch,
  Delete,
}
pub type Headers = std::collections::HashMap<String, String>;

#[derive(Debug, Default)]
pub struct Body {
  inner: Vec<u8>,
}
impl Body {
  pub fn new(inner: Vec<u8>) -> Self {
    Self { inner }
  }
  pub fn get_bytes(&self) -> &[u8] {
    &self.inner[..]
  }
}
impl From<Vec<u8>> for Body {
  fn from(value: Vec<u8>) -> Self {
    Self::new(value)
  }
}
impl From<String> for Body {
  fn from(value: String) -> Self {
    Self::new(value.into_bytes())
  }
}
impl From<&str> for Body {
  fn from(value: &str) -> Self {
    Self::new(value.bytes().collect::<Vec<_>>())
  }
}
impl From<()> for Body {
  fn from(value: ()) -> Self {
    Self::new(Vec::new())
  }
}
#[derive(Debug)]
pub struct Request {
  method: RequestMethod,
  url: String,
  headers: Headers,
  body: Body,
}

impl Request {
  pub fn new(method: RequestMethod, url: String, headers: Headers, body: Body) -> Self {
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

  pub fn body(&self) -> &Body {
    &self.body
  }
}

impl TryFrom<String> for RequestMethod {
  type Error = ();
  fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
    match value.as_str() {
      "GET" => Ok(RequestMethod::Get),
      "POST" => Ok(RequestMethod::Post),
      "PUT" => Ok(RequestMethod::Put),
      "DELETE" => Ok(RequestMethod::Delete),
      "PATCH" => Ok(RequestMethod::Patch),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Default)]
pub enum StatusCode {
  #[default]
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
impl TryFrom<u16> for StatusCode {
  type Error = HttpError;
  fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
    match value {
      200 => Ok(Self::Ok),
      201 => Ok(Self::Created),
      404 => Ok(Self::NotFound),
      403 => Ok(Self::Unauthorized),
      400 => Ok(Self::BadRequest),
      _ => Err(HttpError::InvalidResponseCode(value)),
    }
  }
}
#[derive(Debug, Default)]
pub struct Response {
  status_code: StatusCode,
  headers: Headers,
  body: Body,
}

impl Response {
  pub fn builder() -> ResponseBuilder {
    ResponseBuilder::new()
  }
  pub fn new(status_code: StatusCode, headers: Headers, body: Body) -> Self {
    Self {
      status_code,
      headers,
      body,
    }
  }
  pub fn from_plain_text(code: StatusCode, body: &str) -> Self {
    Self {
      status_code: code,
      headers: Headers::from([("Content-Length".to_string(), body.len().to_string())]),
      body: body.to_string().into(),
    }
  }
  pub fn from_html_ok(html: &str) -> Self {
    Self {
      status_code: StatusCode::Ok,
      headers: Headers::from([
        ("Content-Length".to_string(), html.len().to_string()),
        (
          "Content-Type".to_string(),
          "text/html; charset=utf-8".to_string(),
        ),
      ]),
      body: html.to_string().into(),
    }
  }
  pub fn as_bytes(&self) -> Vec<u8> {
    let status_line = format!("HTTP/1.1 {}\r\n", self.status_code.to_string());
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
      self.body.get_bytes(),
      b"\r\n",
    ]
    .concat()
  }
}

#[derive(Debug)]
pub struct ResponseBuilder {
  inner: Result<Response, HttpError>
}
impl ResponseBuilder  {
  pub fn new() -> Self {
    Self { inner: Ok(Response::default()) }
  }
  pub fn status<T>(self, status: T) -> Self 
    where
      T: TryInto<StatusCode>, 
      T::Error: Into<HttpError>
  {
    let inner = self.inner.and_then(|mut this| {
      this.status_code = status.try_into().map_err(|e| e.into())?;
      Ok(this)
    });
    Self { inner }
  }
  pub fn header(self, key: &str, value: &str) -> Self {
    let inner = self.inner.and_then(|mut this| {
      this.headers.insert(key.into(), value.into());
      Ok(this)
    });
    Self { inner }
  }
  pub fn body(mut self, body: impl Into<Body>) -> Result<Response, HttpError> {
    self.inner.as_mut().map(|mut this| this.body = body.into());
    self.inner
  }
}
