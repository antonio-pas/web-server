#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use http::*;
use parse::*;
use prelude::*;
use tokio::fs;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net;
use tree::Tree;

struct Server {
  router: Router,
}
impl Server {
  pub async fn new(router: Router) -> Result<Self> {
    Ok(Self { router })
  }
  pub async fn listen<A: net::ToSocketAddrs>(&self, a: A) -> Result<()> {
    let listener = net::TcpListener::bind(a).await?;
    loop {
      match listener.accept().await {
        Ok((stream, _)) => {
          let result = self.handle_connection(stream).await;
          if let Err(e) = result {
            eprintln!("error: {}", e);
          }
        }
        Err(e) => eprintln!("error: {}", e),
      }
    }
    Ok(())
  }
  async fn handle_connection(&self, mut stream: net::TcpStream) -> Result<()> {
    let request = parse_request(&mut stream).await?;
    println!(
      "{:?} request at {:?} with body {:?}",
      request.method(),
      request.url(),
      request.body()
    );
    let response = match self
      .router
      .get(request.url().to_string(), *request.method())
    {
      Some(endpoint) => endpoint(&request),
      None => Response::from_plain_text(ResponseCode::NotFound, "404 not found"),
    };
    stream.write_all(&response.as_bytes()).await?;
    Ok(())
  }
}
#[derive(Eq, Hash, PartialEq)]
struct RouteKey {
  path: String,
  method: RequestMethod,
}
struct Router {
  routes: HashMap<RouteKey, Box<dyn Fn(&Request) -> Response>>,
}
impl Router {
  fn new() -> Self {
    Self {
      routes: HashMap::new(),
    }
  }
  fn get(&self, path: String, method: RequestMethod) -> Option<&Box<dyn Fn(&Request) -> Response>> {
    self.routes.get(&RouteKey { path, method })
  }
  fn add_get<F>(&mut self, path: &str, endpoint: F)
  where
    F: Fn(&Request) -> Response + 'static,
  {
    self.routes.insert(
      RouteKey {
        path: path.to_string(),
        method: RequestMethod::Get,
      },
      Box::new(endpoint),
    );
  }
  fn add_post<F>(&mut self, path: &str, endpoint: F)
  where
    F: Fn(&Request) -> Response + 'static,
  {
    self.routes.insert(
      RouteKey {
        path: path.to_string(),
        method: RequestMethod::Post,
      },
      Box::new(endpoint),
    );
  }
}
fn test(req: &Request) -> Response {
  Response::from_html_ok("<h1>hi</h1>")
}
fn post(req: &Request) -> Response {
  Response::from_plain_text(ResponseCode::Ok, "whats up")
}
#[tokio::main]
async fn main() -> Result<()> {
  let mut router = Router::new();
  router.add_get("/", test);
  router.add_post("/add", post);
  Server::new(router).await?.listen("127.0.0.1:8000").await
}
