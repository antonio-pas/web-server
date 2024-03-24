#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use std::fmt::Pointer;
use std::sync::Arc;

use http::*;
use parse::*;
use tokio::io::AsyncWriteExt;
use tokio::net;
use tokio::sync::Mutex;

trait Handler {
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
impl Handler for MyHandler {
  type Response = Result<Response, MyError>;
  async fn call(&mut self, request: Request) -> Self::Response {
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
  pub async fn listen<H: Handler>(&self, mut handler: H)
  where
    H: Handler + Send + 'static,
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
