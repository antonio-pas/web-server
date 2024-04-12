use async_trait::async_trait;

use super::*;
use crate::http::*;
use crate::Handler;

pub struct RouterHandler {
  router: Router,
}
impl RouterHandler {
  pub fn new(router: Router) -> Self {
    Self { router }
  }
}
#[async_trait]
impl Handler<Request> for RouterHandler
where
  Request: Send,
{
  type Response = Response;
  async fn call(&mut self, request: Request) -> Self::Response {
    match self
      .router
      .get_handler(request.method(), request.uri().path())
    {
      Some(mut handler) => handler.call(request).await,
      None => self.router.not_found().call(request).await,
    }
  }
}
