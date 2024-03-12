#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use http::*;
use parse::*;
use tokio::io::AsyncWriteExt;
use tokio::net;

type TempError = Box<dyn std::error::Error>;

trait Handler {
  type Error;
  async fn call(&mut self, request: Request) -> Result<Response, Self::Error>;
}
struct MyHandler {}
impl Handler for MyHandler {
  type Error = StatusCode;
  async fn call(&mut self, request: Request) -> Result<Response, Self::Error> {
    let response = match (request.method(), request.url().as_str()) {
      (&RequestMethod::Get, "/") => Response::builder().status(200).body("Hello")?,
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
  pub async fn listen<H: Handler>(&self, mut handler: H) {
    let listener = net::TcpListener::bind(self.addr)
      .await
      .expect("couldn't bind TCP listener");
    loop {
      let Ok((mut stream, _)) = listener.accept().await else {
        eprintln!("couldn't accept stream from the listener");
        continue;
      };
      let Ok(request) = parse_request(&mut stream).await else {
        eprintln!("error while parsing request");
        continue;
      };
      let response = handler.call(request).await.unwrap();
      if let Err(e) = stream.write_all(&response.as_bytes()).await {
        eprintln!("error while writing to stream: {e}");
        continue;
      }
    }
  }
}
#[tokio::main]
async fn main() {
  let my_handler = MyHandler {};
  Server::new("127.0.0.1:8000").listen(my_handler).await;
}
