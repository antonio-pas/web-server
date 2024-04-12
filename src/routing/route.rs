use regex::Regex;

use crate::handler::*;
use crate::http::*;
pub type Params = std::collections::HashMap<String, String>;

pub struct Route {
  method: RequestMethod,
  regex: Regex,
  handler: Box<dyn Handler<Request, Response = Response> + Send>,
}
impl Route {
  pub fn new<H: Handler<Request, Response = Response> + Send + 'static>(
    method: RequestMethod,
    path: &str,
    handler: H,
  ) -> Self {
    let new_scheme = path
      .split('/')
      .map(|s| {
        if let Some(s) = s.strip_prefix(":") {
          format!("(?P<{s}>[^/]+)")
        } else {
          s.to_string()
        }
      })
      .map(|s| s.replace("*", "[\\w.\\-_]+"))
      .collect::<Vec<_>>()
      .join("/");
    Self {
      method,
      regex: Regex::new(&format!("^{}$", new_scheme)).unwrap(),
      handler: Box::new(handler),
    }
  }
  pub fn matches(&self, path: &str) -> Option<Params> {
    let caps = self.regex.captures(path)?;
    let map = self
      .regex
      .capture_names()
      .flatten()
      .filter_map(|name| Some((name.to_string(), caps.name(name)?.as_str().to_string())))
      .collect::<Params>();
    Some(map)
  }
  pub fn method(&self) -> &RequestMethod {
    &self.method
  }

  pub fn handler(&mut self) -> &mut (dyn Handler<Request, Response = Response> + Send) {
    &mut *self.handler
  }
}
