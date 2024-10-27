use std::marker::PhantomData;
use std::mem;
use std::time::Duration;

use log::{debug, error};

use crate::promise::promise::{Promise, PromiseState, UpdateResult};

pub struct MappedPromise<A, P: Promise<A> + Sized, B, F: FnOnce(A) -> B> {
    // if broken, everything will be set to None
    // if resolved, value will be set to Some
    // if unresolved, value will be None, and parent and mapper are expected to be Some
    parent: Option<P>,
    value: Option<B>,
    mapper: Option<F>,
    _phantom: PhantomData<A>,
}

impl<A, P: Promise<A> + Sized, B, F: FnOnce(A) -> B> MappedPromise<A, P, B, F> {
    pub fn new(parent: P, mapper: F) -> Self {
        MappedPromise {
            parent: Some(parent),
            value: None,
            mapper: Some(mapper),
            _phantom: Default::default(),
        }
    }

    fn break_promise(&mut self) {
        self.parent = None;
        self.mapper = None;
        self.value = None;
    }

    // assumes parent resolved OR broke
    fn execute_mapping(&mut self) -> PromiseState {
        // paranoia asserts

        debug_assert!(self.parent.is_some());
        debug_assert!(self.parent.as_ref().unwrap().state() != PromiseState::Unresolved);
        debug_assert!(self.mapper.is_some());

        if !self.parent.as_ref().map(|p| p.state() != PromiseState::Unresolved).unwrap_or(false) {
            error!("executing mapping on unresolved parent! Breaking promise.");
            self.break_promise();
            return PromiseState::Broken;
        }
        if self.mapper.is_none() {
            self.break_promise();
            return PromiseState::Broken;
        }

        let mut parent = None;
        mem::swap(&mut self.parent, &mut parent);
        let parent = parent.unwrap();

        match parent.state() {
            PromiseState::Unresolved => {
                // this is a paranoid failsafe
                error!("executing mapping on unresolved parent! Breaking promise.");
                self.break_promise();
                PromiseState::Broken
            }
            PromiseState::Ready => {
                let value_op = parent.take();
                debug_assert!(value_op.is_some());
                let value = value_op.unwrap();

                debug_assert!(self.mapper.is_some());
                let mut mapper = None;
                mem::swap(&mut self.mapper, &mut mapper);
                let new_value = (mapper.unwrap())(value);
                self.value = Some(new_value);
                self.parent = None;
                PromiseState::Ready
            }
            PromiseState::Broken => {
                self.break_promise();
                PromiseState::Broken
            }
        }
    }
}

impl<A, P: Promise<A>, B, F: FnOnce(A) -> B> Promise<B> for MappedPromise<A, P, B, F> {
    fn state(&self) -> PromiseState {
        if self.value.is_some() {
            debug_assert!(self.mapper.is_none() && self.parent.is_none(), "failed cleanup on resolve");
            return PromiseState::Ready;
        }

        if self.parent.is_some() {
            debug_assert!(self.value.is_none());
            debug_assert!(self.mapper.is_some());
            return PromiseState::Unresolved;
        }

        debug_assert!(self.mapper.is_none());
        debug_assert!(self.parent.is_none());
        debug_assert!(self.value.is_none());
        PromiseState::Broken
    }

    fn wait(&mut self, how_long: Option<Duration>) -> PromiseState {
        if self.state() == PromiseState::Ready {
            debug!("rather unexpected wait on done");
            return PromiseState::Ready;
        }

        if self.state() == PromiseState::Broken {
            debug!("waiting on broken promise");
            return PromiseState::Broken;
        }

        match self.parent.as_mut().unwrap().wait(how_long) {
            PromiseState::Unresolved => PromiseState::Unresolved,
            _ => self.execute_mapping(),
        }
    }

    fn update(&mut self) -> UpdateResult {
        match self.state() {
            PromiseState::Ready => {
                debug!("rather unexpected update after ready");
                UpdateResult {
                    state: PromiseState::Ready,
                    has_changed: false,
                }
            }
            PromiseState::Broken => {
                debug!("rather unexpected update of broken promise");
                UpdateResult {
                    state: PromiseState::Broken,
                    has_changed: false,
                }
            }
            PromiseState::Unresolved => {
                if self.parent.as_mut().unwrap().update().state.is_resolved() {
                    let state = self.execute_mapping();
                    UpdateResult { state, has_changed: true }
                } else {
                    UpdateResult {
                        state: PromiseState::Unresolved,
                        has_changed: false,
                    }
                }
            }
        }
    }

    fn read(&self) -> Option<&B> {
        self.value.as_ref()
    }

    fn take(self) -> Option<B> {
        self.value
    }
}
