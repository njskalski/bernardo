use log::{error, warn};

use crate::primitives::promise_map::MappedPromise;

pub trait Promise<T> {
    // Blocks current thread until promise is delivered or broken.
    // Double wait *is not* an error.
    // Returns true iff value became available, false if promise is broken.
    fn wait(&mut self) -> bool;

    // Non-blocking wait. Returns whether value is available, false if promise is broken OR unresolved.
    fn update(&mut self) -> bool;

    /*
    Returns value, provided it was retrieved before. Does *not* change state, so it can return None
    *even if Promise is ready to be determined*.
     */
    fn read(&self) -> Option<&T>;

    /*
    Returns true, iff there is no chance that it will ever resolve successfully.
     */
    fn is_broken(&self) -> bool;

    /*
    Immediately consumes promise returning value inside. It *does not* poll for message, so it's
    *not* an equivalent of Future.now_or_never().
     */
    fn take(self) -> Option<T>;

    fn map<B, F: FnOnce(T) -> B>(self, mapper: F) -> MappedPromise<T, Self, B, F> where Self: Sized {
        MappedPromise::new(self, mapper)
    }
}

pub struct DonePromise<A> {
    op: Option<A>,
}

impl<A> DonePromise<A> {
    pub fn new(op: Option<A>) -> Self {
        DonePromise {
            op
        }
    }
}

impl<A> Promise<A> for DonePromise<A> {
    fn wait(&mut self) -> bool {
        self.op.is_some()
    }

    fn update(&mut self) -> bool {
        self.op.is_some()
    }

    fn read(&self) -> Option<&A> {
        self.op.as_ref()
    }

    fn is_broken(&self) -> bool {
        self.op.is_none()
    }

    fn take(mut self) -> Option<A> {
        self.op.take()
    }
}
