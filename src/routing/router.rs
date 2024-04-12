use regex::Regex;

use super::Route;
use crate::handler::*;
use crate::http::*;
pub struct Router {
  routes: Vec<Route>,
  not_found: Box<dyn Handler<Request, Response = Response> + Send>,
}
impl Router {
  pub fn new<H: Handler<Request, Response = Response> + Send + 'static>(
    routes: Vec<Route>,
    not_found: H,
  ) -> Self {
    Self {
      routes,
      not_found: Box::new(not_found),
    }
  }
  pub fn builder<H: Handler<Request, Response = Response> + Send + 'static>(
    not_found: H,
  ) -> RouterBuilder {
    RouterBuilder::new(not_found)
  }
  pub fn get_handler(
    &mut self,
    method: &RequestMethod,
    path: &str,
  ) -> Option<&mut (dyn Handler<Request, Response = Response> + Send)> {
    self
      .routes
      .iter_mut()
      .filter(|r| r.method() == method)
      .find(|r| r.matches(path).is_some())
      .map(move |r| r.handler())
  }

  pub fn not_found(&mut self) -> &mut(dyn Handler<Request, Response = Response> + Send) {
    &mut *self.not_found
  }
}
pub struct RouterBuilder {
  routes: Vec<Route>,
  not_found: Box<dyn Handler<Request, Response = Response> + Send>,
}
impl RouterBuilder {
  pub fn new<H: Handler<Request, Response = Response> + Send + 'static>(not_found: H) -> Self {
    Self {
      routes: vec![],
      not_found: Box::new(not_found),
    }
  }
  pub fn get<H: Handler<Request, Response = Response> + Send + 'static>(
    mut self,
    path: &str,
    handler: H,
  ) -> Self {
    let route = Route::new(RequestMethod::Get, path, handler);
    self.routes.push(route);
    self
  }
  pub fn build(mut self) -> Router {
    Router {
      routes: self.routes,
      not_found: self.not_found,
    }
  }
}
