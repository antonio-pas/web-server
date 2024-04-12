use std::future::Future;

use async_trait::async_trait;
#[async_trait]
pub trait Handler<Request: Send> {
  type Response: Send;
  async fn call(&mut self, request: Request) -> Self::Response;
}
#[async_trait]
impl<F, Fut, Request, Response> Handler<Request> for F
where
  F: FnMut(Request) -> Fut + Send,
  Fut: Future<Output = Response> + Send,
  Request: Send + 'static,
  Response: Send,
{
  type Response = Response;
  async fn call(&mut self, request: Request) -> Self::Response {
    (self)(request).await
  }
}
