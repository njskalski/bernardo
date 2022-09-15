use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{FutureExt, task, TryFuture};

pub struct WrappedFuture<T: Sized> {
    future: Option<Box<dyn Future<Output=T> + Unpin>>,
    item: Option<T>,
}

impl<T: Sized> WrappedFuture<T> {
    /*
    Returns whether poll CHANGED STATE, not whether it resolved.
     */
    pub fn poll(&mut self) -> bool where T: Sized {
        if self.item.is_none() {
            let noop_waker = task::noop_waker();
            let mut cx = Context::from_waker(&noop_waker);

            let mut this = self.future.take().unwrap();

            match Pin::new(&mut this).poll(&mut cx) {
                Poll::Ready(x) => {
                    self.item = Some(x);
                    self.item.as_ref();
                    true
                }
                Poll::Pending => {
                    self.future = Some(this);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn read(&self) -> Option<&T> {
        self.item.as_ref()
    }
}

impl<T> WrappedFuture<T> {
    pub fn new(f: Box<dyn Future<Output=T> + Unpin>) -> Self {
        Self {
            future: Some(f),
            item: None,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::future::ready;

    use crate::experiments::wrapped_future::WrappedFuture;
    use crate::widget::stupid_tree::get_stupid_tree;
    use crate::widgets::tree_view::tree_it::TreeIt;
    use crate::widgets::tree_view::tree_view_node::TreeViewNode;

    #[test]
    fn tree_it_test_1() {
        let f1 = Box::new(ready(3));
        let mut wrapped = WrappedFuture::new(f1);

        assert_eq!(wrapped.poll(), Some(&3));
        assert_eq!(wrapped.poll(), Some(&3));
    }
}
