// this entire class is to be removed when I fix the async
use std::fmt::{Debug, Formatter};
use std::time::Duration;

use crate::promise::streaming_promise_map::MappedStreamingPromise;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StreamingPromiseState {
    Streaming,
    Finished,
    Broken,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UpdateResult {
    pub state: StreamingPromiseState,
    pub has_changed: bool,
}

pub trait StreamingPromise<T> {
    fn state(&self) -> StreamingPromiseState;

    // Blocks current thread until promise is finished, broken or deadline is met.
    // Double drain is *not* an error.
    fn drain(&mut self, how_long: Option<Duration>) -> StreamingPromiseState;

    /*
    TODO I am not sure I want to have read/update separated. If a list displays a promise result,
     right now it can't get the size in min_size, because update is mut and can happen earliest on
     layout. This is a violation of invariant in min_size.
     */

    // Non-blocking wait. Returns promise state and information, whether this call caused update or not.
    fn update(&mut self) -> UpdateResult;

    /*
    Returns value, provided it was retrieved before. Does *not* change state, so it can return None
    *even if Promise is ready to be determined*.

     */
    fn read(&self) -> &Vec<T>;

    fn map<B, F: Fn(&T) -> B>(self, mapper: F) -> MappedStreamingPromise<T, Self, B, F>
    where
        Self: Sized,
        B: Clone,
    {
        MappedStreamingPromise::new(self, mapper)
    }

    fn boxed(self) -> Box<dyn StreamingPromise<T>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<A> Debug for dyn StreamingPromise<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "[{:?} streaming promise of \"{}\"]", self.state(), std::any::type_name::<A>());
    }
}
