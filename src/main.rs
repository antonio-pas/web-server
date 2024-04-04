#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use std::sync::Arc;

use http::*;
use once_cell::sync::Lazy;
use parse::*;
use regex::Regex;
use serde::Serialize;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
  path_matcher: Regex,
  handler: H,
}
impl<H: Handler<Request>> Route<H> {
  fn new(method: RequestMethod, path: &str, handler: H) -> Self {
    Self {
      method,
      path_matcher: Regex::new(&format!("^{}$", path)).unwrap(),
      handler,
    }
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
      .find(|r| r.path_matcher.is_match(path))
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
  fn call(&mut self, request: Request) -> impl std::future::Future<Output = Self::Response> + Send;
}
type MyError = Box<dyn std::error::Error + Send + Sync + 'static>;
impl IntoResponse for MyError {
  fn into_response(self) -> Response {
    format!("{self}").into_response()
  }
}
macro_rules! path {
  ($method:ident $url:expr) => {
    (&RequestMethod::$method, $url)
  };
}
struct MyHandler {}
impl<F, R> Handler<Request> for F
where
  F: Fn(Request) -> R + Send,
  R: Send,
{
  type Response = R;
  async fn call(&mut self, request: Request) -> Self::Response {
    (self)(request)
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
impl Handler<Request> for MyHandler {
  type Response = Result<Response, MyError>;
  async fn call(&mut self, request: Request) -> Self::Response {
    if request.uri().path().starts_with("/public/") {
      let path = request.uri().path().strip_prefix("/").unwrap();
      let metadata = fs::metadata(path).await?;
      let mut buffer = vec![0; metadata.len() as usize];
      let mut file = fs::OpenOptions::new().read(true).open(path).await?;
      file.read(&mut buffer).await?;
      let mut content_type = "text/plain";
      if (path.ends_with(".html")) {
        content_type = "text/html";
      } else if (path.ends_with(".js")) {
        content_type = "text/javascript";
      }
      let response = Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .body(buffer)?;
      return Ok(response);
    }
    let response = match (request.method(), request.uri().path()) {
      path!(Get "/") => {
        let store = STORE.lock().await;
        let mut items = String::new();
        for item in store.iter() {
          items.push_str(&format!("<li>{}</li>", item));
        }
        let html = tokio::fs::read_to_string("public/index.html")
          .await
          .unwrap();
        Response::builder()
          .status(200)
          .header("Content-Type", "text/html")
          .header("Content-Length", &html.len().to_string())
          .body(html)?
      }
      path!(Get "/api/all") => {
        let serialized = {
          let store = STORE.lock().await;
          serde_json::to_string(&store.to_vec())?
        };
        Response::builder().body(serialized)?
      }
      path!(Post "/api/add") => {
        let string = String::from_utf8(request.body().get_bytes().to_vec())?;
        let serialized: String = serde_json::from_str(&string)?;
        {
          let mut store = STORE.lock().await;
          store.push(serialized);
        }
        Response::builder().status(201).body(())?
      }
      _ => Response::builder().status(404).body("not found")?,
    };
    Ok(response)
  }
}
struct Server {
  addr: &'static str,
}
impl Server {
  pub fn new(addr: &'static str) -> Self {
    Self { addr }
  }
  pub async fn listen<H>(&self, handler: H)
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
      let Ok(request) = parse_request(&mut stream).await else {
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
fn not_found(_: Request) -> Response {
  Response::builder().status(404).body("Not found").unwrap()
}
fn this_handler(req: Request) -> Response {
  Response::builder().body("helo").unwrap()
}
#[tokio::main]
async fn main() {
  let router = Router::builder(not_found as fn(Request) -> Response)
    .get("/", this_handler)
    .build();
  let my_handler = RouterHandler::new(router);
  Server::new("127.0.0.1:8000").listen(my_handler).await;
}
