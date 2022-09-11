use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{FutureExt, pin_mut, task, TryFuture};

pub struct WrappedFuture<T: Sized> {
    future: Option<Box<dyn Future<Output=T> + Unpin>>,
    item: Option<T>,
}

impl<T: Sized> WrappedFuture<T> {
    pub fn poll(&mut self) -> Option<&T> where T: Sized {
        let noop_waker = task::noop_waker();
        let mut cx = Context::from_waker(&noop_waker);

        let mut this = self.future.take().unwrap();

        match Pin::new(&mut this).poll(&mut cx) {
            Poll::Ready(x) => {
                self.item = Some(x);
                self.item.as_ref()
            }
            Poll::Pending => {
                self.future = Some(this);
                None
            }
        }
    }
}