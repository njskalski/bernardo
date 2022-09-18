use std::fmt::{Debug, Formatter};

use log::{error, warn};

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

    // Blocks current thread until promise is delivered or broken.
    // Double wait *is not* an error.
    // Returns true iff value became available, false if promise is broken.
    fn wait(&mut self) -> PromiseState;

    // Non-blocking wait. Returns whether value is available, false if promise is broken OR unresolved.
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

    fn map<B, F: FnOnce(T) -> B>(self, mapper: F) -> MappedPromise<T, Self, B, F> where Self: Sized {
        MappedPromise::new(self, mapper)
    }
}

pub struct ResolvedPromise<A> {
    op: Option<A>,
}

impl<A> ResolvedPromise<A> {
    pub fn new(op: Option<A>) -> Self {
        ResolvedPromise {
            op
        }
    }
}

impl<A> Promise<A> for ResolvedPromise<A> {
    fn wait(&mut self) -> PromiseState {
        self.state()
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