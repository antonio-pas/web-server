#![allow(unused)]
mod error;
mod prelude;
mod tree;

use std::fmt;

use prelude::*;
use tokio::io::{self, AsyncWriteExt};
use tokio::io::AsyncReadExt;
use tokio::net;

struct Server {}
impl Server {
  pub async fn new() -> Result<Self> {
    Ok(Self {})
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
    let mut buf = [0u8; 1024];
    stream.read(&mut buf).await?;
    let s = String::from_utf8(buf.to_vec())?;
    println!("received connection: {}", s.lines().next().unwrap());
    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\nhello\n").await?;
    Ok(())
  }
}
#[tokio::main]
async fn main() -> Result<()> {
  Server::new().await?.listen("127.0.0.1:8000").await
}
