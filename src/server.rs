use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net;
use tokio::sync::Mutex;

use crate::error::*;
use crate::http::*;
use crate::Handler;
use crate::Parser;
pub struct Server {
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
