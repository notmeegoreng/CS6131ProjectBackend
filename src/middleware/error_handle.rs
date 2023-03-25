use tide;

use crate::State;

pub(crate) struct ErrorHandleMiddleware;

#[tide::utils::async_trait]
impl tide::Middleware<State> for ErrorHandleMiddleware {
    async fn handle(&self, request: crate::Request, next: tide::Next<'_, State>) -> tide::Result {
        let resp = next.run(request).await;
        match resp.error() {
            Some(e) => tide::log::error!("Error {:?} occurred with response {:?}", e, resp),
            None => ()
        }
        Ok(resp)
    }
}