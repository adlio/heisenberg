//! Custom futures for performance optimization

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Custom future that avoids boxing for better performance
pub struct HeisenbergFuture<F> {
    inner: F,
}

impl<F> HeisenbergFuture<F> {
    /// Create a new HeisenbergFuture wrapping the given future
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F, T, E> Future for HeisenbergFuture<F>
where
    F: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We're not moving the inner future
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        inner.poll(cx)
    }
}
