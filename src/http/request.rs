use super::*;

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
