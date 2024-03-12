use super::*;

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
  inner: Result<Response, HttpError>,
}
impl ResponseBuilder {
  pub fn new() -> Self {
    Self {
      inner: Ok(Response::default()),
    }
  }
  pub fn status<T>(self, status: T) -> Self
  where
    T: TryInto<StatusCode>,
    T::Error: Into<HttpError>,
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
