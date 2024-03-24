#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use std::sync::Arc;

use http::*;
use parse::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;
use tokio::sync::Mutex;
use tokio::fs;

trait Handler<Request> {
  type Response: IntoResponse + Send;
  fn call(&mut self, request: Request) -> impl std::future::Future<Output = Self::Response> + Send;
}
type MyError = Box<dyn std::error::Error + Send + Sync + 'static>;
impl IntoResponse for MyError {
  fn into_response(self) -> Response {
    format!("{self}").into_response()
  }
}
struct MyHandler {}
impl Handler<Request> for MyHandler {
  type Response = Result<Response, MyError>;
  async fn call(&mut self, request: Request) -> Self::Response {
    if request.url().starts_with("/public/") {
      let path = request.url().strip_prefix("/").unwrap();
      let metadata = fs::metadata(path).await?;
      let mut buffer = vec![0; metadata.len() as usize];
      let mut file = fs::OpenOptions::new().read(true).open(path).await?;
      file.read(&mut buffer).await?;
      return Ok(Body::from(buffer).into_response());
    }
    let response = match (request.method(), request.url().as_str()) {
      (&RequestMethod::Get, "/") => Response::builder().status(200).body("Hello").unwrap(),
      _ => Response::builder().status(404).body("not found").unwrap(),
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
        eprintln!("error while parsing request");
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
#[tokio::main]
async fn main() {
  let my_handler = MyHandler {};
  Server::new("127.0.0.1:8000").listen(my_handler).await;
}
