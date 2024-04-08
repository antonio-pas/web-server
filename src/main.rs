#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use std::future::Future;
use std::sync::Arc;

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
#[derive(Debug)]
struct Route<H>
where
  H: Handler<Request>,
{
  method: RequestMethod,
  regex: Regex,
  handler: H,
}
type Params = std::collections::HashMap<String, String>;
impl<H: Handler<Request>> Route<H> {
  fn new(method: RequestMethod, path: &str, handler: H) -> Self {
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
      handler,
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
#[derive(Debug)]
struct Router<H: Handler<Request>> {
  routes: Vec<Route<H>>,
  not_found: H,
}
impl<H: Handler<Request>> Router<H> {
  fn new(routes: Vec<Route<H>>, not_found: H) -> Self {
    Self { routes, not_found }
  }
  fn builder(not_found: H) -> RouterBuilder<H> {
    RouterBuilder::new(not_found)
  }
  fn get_handler(&mut self, method: &RequestMethod, path: &str) -> Option<&mut H> {
    self
      .routes
      .iter_mut()
      .filter(|r| &r.method == method)
      .find(|r| r.matches(path).is_some())
      .map(|r| &mut r.handler)
  }
}
#[derive(Debug)]
struct RouterBuilder<H: Handler<Request>> {
  routes: Vec<Route<H>>,
  not_found: H,
}
impl<H: Handler<Request>> RouterBuilder<H> {
  fn new(not_found: H) -> Self {
    Self {
      routes: vec![],
      not_found,
    }
  }
  fn get(mut self, path: &str, handler: H) -> Self {
    let route = Route::new(RequestMethod::Get, path, handler);
    self.routes.push(route);
    self
  }
  fn build(mut self) -> Router<H> {
    Router {
      routes: self.routes,
      not_found: self.not_found,
    }
  }
}
trait Handler<Request> {
  type Response: Send;
  fn call(&mut self, request: Request) -> impl Future<Output = Self::Response> + Send;
}
impl<F, R, Response> Handler<Request> for F
where
  F: Fn(Request) -> R + Send,
  R: Future<Output = Response> + Send,
  Response: Send,
{
  type Response = Response;
  async fn call(&mut self, request: Request) -> Self::Response {
    (self)(request).await
  }
}
struct RouterHandler<H: Handler<Request>> {
  router: Router<H>,
}
impl<H: Handler<Request>> RouterHandler<H> {
  fn new(router: Router<H>) -> Self {
    Self { router }
  }
}
impl<H: Handler<Request> + Send> Handler<Request> for RouterHandler<H> {
  type Response = <H as Handler<Request>>::Response;
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
type Function<R> where R: Future<Output = Request> = fn(Request) -> R;
#[tokio::main]
async fn main() {
  let router = Router::builder(not_found as fn(Request) -> impl Future<Output = Response>)
    .get("/", this_handler)
    .get("/public/*", public_handler)
    .build();
  let my_handler = RouterHandler::new(router);
  Server::new("127.0.0.1:8000")
    .listen(my_handler)
    .await
    .expect("error");
}
