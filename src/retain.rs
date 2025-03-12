use crate::Watch;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_layer::{layer_fn, Layer};
use tower_service::Service;

/// Holds a drain::Watch for as long as a request is pending.
#[derive(Clone, Debug)]
pub struct Retain<S> {
    inner: S,
    drain: Watch,
}

// === impl Retain ===

impl<S> Retain<S> {
    pub fn new(drain: Watch, inner: S) -> Self {
        Self { drain, inner }
    }

    pub fn layer(drain: Watch) -> impl Layer<S, Service = Self> + Clone {
        layer_fn(move |inner| Self::new(drain.clone(), inner))
    }
}

impl<Req, S> Service<Req> for Retain<S>
where
    S: Service<Req>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send + 'static>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: Req) -> Self::Future {
        let call = self.inner.call(req);
        let drain = self.drain.clone();
        Box::pin(drain.ignore_signaled().release_after(call))
    }
}
