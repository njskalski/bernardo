use std::marker::PhantomData;
use std::mem;

use json::value;
use log::{debug, error, warn};

use crate::primitives::promise::Promise;

pub struct MappedPromise<A, P: Promise<A> + Sized, B, F: FnOnce(A) -> B> {
    // if broken, parent is set to None
    parent: Option<P>,
    value: Option<B>,
    mapper: Option<F>,
    _phantomDate: PhantomData<A>,
}

impl<A, P: Promise<A> + Sized, B, F: FnOnce(A) -> B> MappedPromise<A, P, B, F> {
    pub fn new(parent: P, mapper: F) -> Self {
        MappedPromise {
            parent: Some(parent),
            value: None,
            mapper: Some(mapper),
            _phantomDate: Default::default(),
        }
    }

    // assumes parent succeeded
    fn map_parent(&mut self) {
        let mut parent = None;
        mem::swap(&mut self.parent, &mut parent);
        debug_assert!(parent.is_some());
        let value_op = parent.unwrap().take();
        debug_assert!(value_op.is_some());
        let value = value_op.unwrap();

        let mut mapper = None;
        mem::swap(&mut self.mapper, &mut mapper);
        let new_value = (mapper.unwrap())(value);
        self.value = Some(new_value)
    }
}

impl<A, P: Promise<A>, B, F: FnOnce(A) -> B> Promise<B> for MappedPromise<A, P, B, F> {
    fn wait(&mut self) -> bool {
        if self.value.is_some() {
            debug!("rather unexpected wait on done");
            return true;
        }

        if self.parent.is_none() {
            debug!("waiting on broken promise");
            return false;
        }

        if self.mapper.is_none() {
            error!("can't wait, mapper is none. Breaking promise.");
            self.parent = None;
            return false;
        }

        if self.parent.as_mut().unwrap().wait() {
            self.map_parent();
            true
        } else {
            self.parent = None;
            self.mapper = None;
            false
        }
    }

    fn update(&mut self) -> bool {
        if self.value.is_some() {
            debug!("rather unexpected update on done");
            return true;
        }

        if self.parent.is_none() {
            debug!("updating broken promise");
            return false;
        }

        if self.mapper.is_none() {
            error!("can't update, mapper is none. Breaking promise.");
            self.parent = None;
            return false;
        }

        let parent_done = {
            let parent = self.parent.as_mut().unwrap();
            if parent.is_broken() {
                debug!("propagating broken promise");
                self.parent = None;
                self.mapper = None;
                return false;
            };
            parent.update()
        };

        if !parent_done {
            false
        } else {
            self.map_parent();
            true
        }
    }

    fn read(&self) -> Option<&B> {
        self.value.as_ref()
    }

    fn is_broken(&self) -> bool {
        self.value.is_none() && self.parent.is_none()
    }

    fn take(self) -> Option<B> {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use crate::primitives::promise::{DonePromise, Promise};
    use crate::primitives::promise_map::MappedPromise;

    struct MockPromise<A> {
        op: Option<A>,
        done: bool,
        //done + None = broken
        on_wait: Option<A>,
        //this will be moved on wait to op
        num_updates: usize, // when it hits zero, promise resolves to on_wait
    }

    impl<A> Promise<A> for MockPromise<A> {
        fn wait(&mut self) -> bool {
            mem::swap(&mut self.op, &mut self.on_wait);
            self.done = true;
            self.op.is_some()
        }

        fn update(&mut self) -> bool {
            if self.num_updates > 0 {
                self.num_updates -= 1;
            } else {
                // this is stupid but it's a mock.
                self.wait();
            }

            self.op.is_some()
        }

        fn read(&self) -> Option<&A> {
            self.op.as_ref()
        }

        fn is_broken(&self) -> bool {
            self.done && self.op.is_none()
        }

        fn take(mut self) -> Option<A> {
            if self.done {
                self.op.take()
            } else {
                None
            }
        }
    }

    #[test]
    fn map_good() {
        let done = DonePromise::new(Some(1));
        let mut mapped = MappedPromise::new(done, |a| a + 1);

        assert_eq!(mapped.is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), true);
        assert_eq!(mapped.read(), Some(&2));
    }

    #[test]
    fn map_broken() {
        let mock: MockPromise<i32> = MockPromise {
            op: None,
            done: false,
            on_wait: None,
            num_updates: 100,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.wait(), false);
        assert_eq!(mapped.wait(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.is_broken(), true);
    }

    #[test]
    fn map_successful_wait() {
        let mock: MockPromise<i32> = MockPromise {
            op: None,
            done: false,
            on_wait: Some(3),
            num_updates: 100,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.wait(), true);
        assert_eq!(mapped.wait(), true);
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.is_broken(), false);
        assert_eq!(mapped.take(), Some(4));
    }

    #[test]
    fn map_successful_update() {
        let mock: MockPromise<i32> = MockPromise {
            op: None,
            done: false,
            on_wait: Some(3),
            num_updates: 2,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), true);
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.wait(), true);
        assert_eq!(mapped.wait(), true);
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.read(), Some(&4));
        assert_eq!(mapped.is_broken(), false);
        assert_eq!(mapped.take(), Some(4));
    }

    #[test]
    fn map_broken_update() {
        let mock: MockPromise<i32> = MockPromise {
            op: None,
            done: false,
            on_wait: None,
            num_updates: 2,
        };
        let mut mapped = MappedPromise::new(mock, |a| a + 1);

        assert_eq!(mapped.is_broken(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.update(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.wait(), false);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.read(), None);
        assert_eq!(mapped.is_broken(), true);
        assert_eq!(mapped.is_broken(), true);
        assert_eq!(mapped.take(), None);
    }
}