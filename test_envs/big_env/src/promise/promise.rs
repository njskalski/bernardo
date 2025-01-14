use std::fmt::{Debug, Formatter};
use std::time::Duration;

use crate::promise::promise_map::MappedPromise;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PromiseState {
    Unresolved,
    Ready,
    Broken,
}

impl PromiseState {
    #[inline]
    pub fn is_ready(&self) -> bool {
        *self == PromiseState::Ready
    }

    #[inline]
    pub fn is_broken(&self) -> bool {
        *self == PromiseState::Broken
    }

    #[inline]
    pub fn is_resolved(&self) -> bool {
        *self == PromiseState::Ready || *self == PromiseState::Broken
    }

    #[inline]
    pub fn is_unresolved(&self) -> bool {
        *self == PromiseState::Unresolved
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UpdateResult {
    pub state: PromiseState,
    pub has_changed: bool,
}

pub trait Promise<T> {
    fn state(&self) -> PromiseState;

    // Blocks current thread until promise is delivered or broken or deadline is exceeded.
    // Double wait *is not* an error.
    // If how_long is exceeded, it returns Unresolved
    fn wait(&mut self, how_long: Option<Duration>) -> PromiseState;

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
    fn read(&self) -> Option<&T>;

    /*
    Immediately consumes promise returning value inside. It *does not* poll for message, so it's
    *not* an equivalent of Future.now_or_never().
     */
    fn take(self) -> Option<T>;

    fn map<B, F: FnOnce(T) -> B>(self, mapper: F) -> MappedPromise<T, Self, B, F>
    where
        Self: Sized,
    {
        MappedPromise::new(self, mapper)
    }

    fn boxed(self) -> Box<dyn Promise<T>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub struct ResolvedPromise<A> {
    op: Option<A>,
}

impl<A> ResolvedPromise<A> {
    pub fn new(op: Option<A>) -> Self {
        ResolvedPromise { op }
    }
}

impl<A> Promise<A> for ResolvedPromise<A> {
    fn wait(&mut self, _how_long: Option<Duration>) -> PromiseState {
        let result = self.state();
        debug_assert!(result != PromiseState::Unresolved);
        result
    }

    fn update(&mut self) -> UpdateResult {
        UpdateResult {
            state: self.state(),
            has_changed: false,
        }
    }

    fn read(&self) -> Option<&A> {
        self.op.as_ref()
    }

    fn state(&self) -> PromiseState {
        if self.op.is_some() {
            PromiseState::Ready
        } else {
            PromiseState::Broken
        }
    }

    fn take(mut self) -> Option<A> {
        self.op.take()
    }
}

impl<A> Debug for dyn Promise<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "[{:?} promise of \"{}\"]", self.state(), std::any::type_name::<A>());
    }
}
