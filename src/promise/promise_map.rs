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
        if !self.mapper.is_some() {
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
            PromiseState::Unresolved => {
                PromiseState::Unresolved
            }
            _ => self.execute_mapping()
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
                    UpdateResult {
                        state,
                        has_changed: true,
                    }
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

#[cfg(test)]
mod tests {
    use std::mem;
    use std::time::Duration;

    use crate::promise::promise::{Promise, PromiseState, ResolvedPromise, UpdateResult};
    use crate::promise::promise_map::MappedPromise;

    struct MockPromise<A> {
        //done + value == None => broken
        value: Option<A>,
        done: bool,

        // value to be set to
        will_become: Option<A>,
        //this will be moved on wait to op
        num_updates: usize, // when it hits zero, promise resolves to on_wait
    }

    impl<A> Promise<A> for MockPromise<A> {
        fn state(&self) -> PromiseState {
            if self.done {
                if self.value.is_some() {
                    PromiseState::Ready
                } else {
                    PromiseState::Broken
                }
            } else {
                PromiseState::Unresolved
            }
        }

        fn wait(&mut self, _how_long: Option<Duration>) -> PromiseState {
            mem::swap(&mut self.value, &mut self.will_become);
            self.done = true;
            self.state()
        }

        fn update(&mut self) -> UpdateResult {
            if self.num_updates > 0 {
                self.num_updates -= 1;
                UpdateResult {
                    state: PromiseState::Unresolved,
                    has_changed: false,
                }
            } else {
                // this is stupid but it's a mock.
                self.wait(None);
                UpdateResult { state: self.state(), has_changed: true }
            }
        }

        fn read(&self) -> Option<&A> {
            self.value.as_ref()
        }

        fn take(mut self) -> Option<A> {
            if self.done {
                self.value.take()
            } else {
                None
            }
        }
    }

    #[test]
    fn map_good() {
        let done = ResolvedPromise::new(Some(1));
        let mut mapped = MappedPromise::new(done, |a| a + 1);

        assert_eq!(mapped.state().is_broken(), false);
        assert_eq!(mapped.state(), PromiseState::Unresolved);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), UpdateResult {
            state: PromiseState::Ready,
            has_changed: true,
        });
        assert_eq!(mapped.update(), UpdateResult {
            state: PromiseState::Ready,
            has_changed: false,
        });
        assert_eq!(mapped.read(), Some(&2));
    }

    #[test]
    fn map_broken() {
        let mock: MockPromise<i32> = MockPromise {
            value: None,
            done: false,
            will_become: None,
            num_updates: 100,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.state().is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        });
        assert_eq!(mapped.update(), UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        });
        assert_eq!(mapped.wait(None), PromiseState::Broken);
        assert_eq!(mapped.wait(None), PromiseState::Broken);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.state(), PromiseState::Broken);
    }

    #[test]
    fn map_successful_wait() {
        let mock: MockPromise<i32> = MockPromise {
            value: None,
            done: false,
            will_become: Some(3),
            num_updates: 100,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.state().is_broken(), false);
        assert_eq!(mapped.state(), PromiseState::Unresolved);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        });
        assert_eq!(mapped.update(), UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        });
        assert_eq!(mapped.wait(None), PromiseState::Ready);
        assert_eq!(mapped.wait(None), PromiseState::Ready);
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.state().is_broken(), false);
        assert_eq!(mapped.take(), Some(4));
    }

    #[test]
    fn map_successful_update() {
        let mock: MockPromise<i32> = MockPromise {
            value: None,
            done: false,
            will_become: Some(3),
            num_updates: 2,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.state().is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update().has_changed, false);
        assert_eq!(mapped.update().has_changed, false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update().has_changed, true);
        assert_eq!(mapped.update().has_changed, false);
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.wait(None), PromiseState::Ready);
        assert_eq!(mapped.wait(None), PromiseState::Ready);
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.state().is_broken(), false);
        assert_eq!(mapped.state(), PromiseState::Ready);
        assert_eq!(mapped.take(), Some(4));
    }

    #[test]
    fn map_broken_update() {
        let mock: MockPromise<i32> = MockPromise {
            value: None,
            done: false,
            will_become: None,
            num_updates: 2,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.state().is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update().has_changed, false);
        assert_eq!(mapped.update().has_changed, false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update().has_changed, true);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.wait(None), PromiseState::Broken);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.state().is_broken(), true);
        assert_eq!(mapped.state(), PromiseState::Broken);
        assert_eq!(mapped.take(), None);
    }
}