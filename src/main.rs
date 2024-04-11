#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use error::*;
use http::*;
use once_cell::sync::Lazy;
use parse::*;
use regex::Regex;
use serde::Serialize;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net;
use tokio::sync::Mutex;
static STORE: Lazy<Mutex<Vec<String>>> =
  Lazy::new(|| Mutex::new(vec![String::from("wash the dishes")]));
struct Route {
  method: RequestMethod,
  regex: Regex,
  handler: Box<dyn Handler<Request, Response = Response> + Send>,
}
type Params = std::collections::HashMap<String, String>;
impl Route {
  fn new<H: Handler<Request, Response = Response> + Send + 'static>(method: RequestMethod, path: &str, handler: H) -> Self {
    let new_scheme = path
      .split('/')
      .map(|s| {
        if let Some(s) = s.strip_prefix(":") {
          format!("(?P<{s}>[^/]+)")
        } else {
          s.to_string()
        }
      })
      .map(|s| s.replace("*", "[\\w]+"))
      .collect::<Vec<_>>()
      .join("/");
    Self {
      method,
      regex: Regex::new(&format!("^{}$", new_scheme)).unwrap(),
      handler: Box::new(handler),
    }
  }
  fn matches(&self, path: &str) -> Option<Params> {
    let caps = self.regex.captures(path)?;
    let map = self
      .regex
      .capture_names()
      .flatten()
      .filter_map(|name| Some((name.to_string(), caps.name(name)?.as_str().to_string())))
      .collect::<Params>();
    Some(map)
  }
}
struct Router {
  routes: Vec<Route>,
  not_found: Box<dyn Handler<Request, Response = Response> + Send>
}
impl Router {
  fn new<H: Handler<Request, Response = Response> + Send + 'static>(routes: Vec<Route>, not_found: H) -> Self {
    Self { routes, not_found: Box::new(not_found) }
  }
  fn builder<H: Handler<Request, Response = Response> + Send + 'static>(not_found: H) -> RouterBuilder {
    RouterBuilder::new(not_found)
  }
  fn get_handler(&mut self, method: &RequestMethod, path: &str) -> Option<&mut (dyn Handler<Request, Response = Response> + Send + 'static)> {
    self
      .routes
      .iter_mut()
      .filter(|r| &r.method == method)
      .find(|r| r.matches(path).is_some())
      .map(move |r| &mut *r.handler)
  }
}
struct RouterBuilder {
  routes: Vec<Route>,
  not_found: Box<dyn Handler<Request, Response = Response> + Send>,
}
impl RouterBuilder {
  fn new<H: Handler<Request, Response = Response> + Send + 'static>(not_found: H) -> Self {
    Self {
      routes: vec![],
      not_found: Box::new(not_found),
    }
  }
  fn get<H: Handler<Request, Response = Response> + Send + 'static>(mut self, path: &str, handler: H) -> Self {
    let route = Route::new(RequestMethod::Get, path, handler);
    self.routes.push(route);
    self
  }
  fn build(mut self) -> Router {
    Router {
      routes: self.routes,
      not_found: self.not_found,
    }
  }
}
#[async_trait]
trait Handler<Request: Send> {
  type Response: Send;
  async fn call(&mut self, request: Request) -> Self::Response;
}
#[async_trait]
impl<F, R, Response> Handler<Request> for F
where
  F: FnMut(Request) -> R + Send,
  R: Future<Output = Response> + Send,
  Response: Send,
{
  type Response = Response;
  async fn call(&mut self, request: Request) -> Self::Response {
    (self)(request).await
  }
}
struct RouterHandler {
  router: Router,
}
impl RouterHandler {
  fn new(router: Router) -> Self {
    Self { router }
  }
}
#[async_trait]
impl Handler<Request> for RouterHandler where Request: Send {
  type Response = Response;
  async fn call(&mut self, request: Request) -> Self::Response {
    match self
      .router
      .get_handler(request.method(), request.uri().path())
    {
      Some(mut handler) => handler.call(request).await,
      None => self.router.not_found.call(request).await,
    }
  }
}

struct Server {
  addr: &'static str,
}
impl Server {
  pub fn new(addr: &'static str) -> Self {
    Self { addr }
  }
  pub async fn listen<H>(&self, handler: H) -> Result<(), Error>
  where
    H: Handler<Request> + Send + 'static,
    H::Response: IntoResponse,
  {
    let listener = net::TcpListener::bind(self.addr)
      .await
      .expect("couldn't bind TCP listener");
    let handler = Arc::new(Mutex::new(handler));
    loop {
      let Ok((mut stream, _)) = listener.accept().await else {
        eprintln!("couldn't accept stream from the listener");
        continue;
      };
      let mut reader = BufReader::new(&mut stream);
      let mut buf = [0u8; 2048];
      let Ok(n) = reader.read(&mut buf).await else {
        eprintln!("error reading");
        continue;
      };
      let input = buf[0..n].to_vec();
      let mut parser = Parser::new(input);

      let Ok(request) = parser.parse() else {
        eprintln!("parsing error");
        continue;
      };
      let handler = handler.clone();
      tokio::task::spawn(async move {
        let mut handler = handler.lock().await;
        let response = handler.call(request).await.into_response();
        if let Err(e) = stream.write_all(&response.as_bytes()).await {
          eprintln!("error while writing to stream: {e}");
        }
      });
    }
  }
}
fn server_error() -> Response {
  Response::builder()
    .status(500)
    .body("500 Internal Server Error")
    .unwrap()
}
async fn not_found(_: Request) -> Response {
  Response::builder().status(404).body("Not found").unwrap()
}
async fn this_handler(req: Request) -> Response {
  Response::builder().body("helo").unwrap()
}
async fn public_handler(req: Request) -> Response {
  let path = req.uri().path().strip_prefix("/public/").unwrap();
  Response::builder().body("this is public").unwrap()
}
#[tokio::main]
async fn main() {
  let router = Router::builder(not_found)
    .get("/", this_handler)
    .build();
  let my_handler = RouterHandler::new(router);
  Server::new("127.0.0.1:8000")
    .listen(my_handler)
    .await
    .expect("error");
}
