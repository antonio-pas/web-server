#![allow(unused)]
mod error;
mod handler;
mod http;
mod parse;
mod prelude;
mod routing;
mod server;
mod tree;

use std::{fs, path::Path};

use async_trait::async_trait;
use error::*;
use handler::*;
use http::*;
use parse::*;
use routing::*;
use server::*;

async fn not_found(_: Request) -> Response {
  Response::builder().status(404).body("not found!").unwrap()
}
async fn handler(req: Request) -> Response {
  Response::builder().body("welcome to my home page").unwrap()
}
#[tokio::main]
async fn main() {
  let router = Router::builder(not_found).get("/", handler).build();
  let my_handler = RouterHandler::new(router);
  Server::new("127.0.0.1:8000")
    .listen(my_handler)
    .await
    .expect("error");
}
