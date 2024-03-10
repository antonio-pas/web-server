#![allow(unused)]
mod error;
mod http;
mod parse;
mod prelude;
mod tree;

use std::future::Future;

use http::*;
use parse::*;
use prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net;

trait Handler {
  async fn call(&mut self, request: Request) -> Result<Response>;
}
struct MyHandler {}
impl Handler for MyHandler {
  async fn call(&mut self, request: Request) -> Result<Response> {
    if request.url().as_str() == "/" {
      Ok(Response::from_html_ok("<h1>Home</h1>"))
    } else {
      Ok(Response::from_plain_text(StatusCode::NotFound, "404 not found"))
    }
  }
}
struct Server {
  addr: &'static str
}
impl Server {
  pub fn new(addr: &'static str) -> Self {
    Self { addr }
  }
  pub async fn listen<H: Handler>(&self, mut handler: H) -> Result<()> {
    let listener = net::TcpListener::bind(self.addr).await?;
    loop {
      let Ok((mut stream, _)) = listener.accept().await else { eprintln!("error"); continue; };
      let result = {
        let request = parse_request(&mut stream).await?;
        let response = handler.call(request).await?;
        stream.write_all(&response.as_bytes()).await?;
        Ok::<(), Error>(())
      }.inspect_err(|e| eprintln!("{e}"));
    }
  }
}
#[tokio::main]
async fn main() -> Result<()> {
  let my_handler = MyHandler {};
  Server::new("127.0.0.1:8000").listen(my_handler).await
}
