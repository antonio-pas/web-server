use super::*;

#[derive(Debug)]
pub struct Request {
  method: RequestMethod,
  uri: Uri,
  headers: Headers,
  body: Body,
}

impl Request {
  pub fn new(method: RequestMethod, uri: Uri, headers: Headers, body: Body) -> Self {
    Self {
      method,
      uri,
      headers,
      body,
    }
  }

  pub fn method(&self) -> &RequestMethod {
    &self.method
  }

  pub fn uri(&self) -> &Uri {
    &self.uri
  }

  pub fn headers(&self) -> &Headers {
    &self.headers
  }

  pub fn body(&self) -> &Body {
    &self.body
  }
}
