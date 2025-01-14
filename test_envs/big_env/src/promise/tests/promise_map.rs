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
            UpdateResult {
                state: self.state(),
                has_changed: true,
            }
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

    assert!(!mapped.state().is_broken());
    assert_eq!(mapped.state(), PromiseState::Unresolved);
    assert_eq!(mapped.read(), None);
    assert_eq!(
        mapped.update(),
        UpdateResult {
            state: PromiseState::Ready,
            has_changed: true,
        }
    );
    assert_eq!(
        mapped.update(),
        UpdateResult {
            state: PromiseState::Ready,
            has_changed: false,
        }
    );
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

    assert!(!mapped.state().is_broken());
    assert_eq!(mapped.read(), None);
    assert_eq!(mapped.read(), None);
    assert_eq!(
        mapped.update(),
        UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        }
    );
    assert_eq!(
        mapped.update(),
        UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        }
    );
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

    assert!(!mapped.state().is_broken());
    assert_eq!(mapped.state(), PromiseState::Unresolved);
    assert_eq!(mapped.read(), None);
    assert_eq!(
        mapped.update(),
        UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        }
    );
    assert_eq!(
        mapped.update(),
        UpdateResult {
            state: PromiseState::Unresolved,
            has_changed: false,
        }
    );
    assert_eq!(mapped.wait(None), PromiseState::Ready);
    assert_eq!(mapped.wait(None), PromiseState::Ready);
    assert_eq!(mapped.read(), Some(&4));
    assert_eq!(mapped.read(), Some(&4));
    assert!(!mapped.state().is_broken());
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

    assert!(!mapped.state().is_broken());
    assert_eq!(mapped.read(), None);
    assert!(!mapped.update().has_changed);
    assert!(!mapped.update().has_changed);
    assert_eq!(mapped.read(), None);
    assert!(mapped.update().has_changed);
    assert!(!mapped.update().has_changed);
    assert_eq!(mapped.read(), Some(&4));
    assert_eq!(mapped.wait(None), PromiseState::Ready);
    assert_eq!(mapped.wait(None), PromiseState::Ready);
    assert_eq!(mapped.read(), Some(&4));
    assert_eq!(mapped.read(), Some(&4));
    assert!(!mapped.state().is_broken());
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

    assert!(!mapped.state().is_broken());
    assert_eq!(mapped.read(), None);
    assert!(!mapped.update().has_changed);
    assert!(!mapped.update().has_changed);
    assert_eq!(mapped.read(), None);
    assert!(mapped.update().has_changed,);
    assert_eq!(mapped.read(), None);
    assert_eq!(mapped.wait(None), PromiseState::Broken);
    assert_eq!(mapped.read(), None);
    assert_eq!(mapped.read(), None);
    assert!(mapped.state().is_broken());
    assert_eq!(mapped.state(), PromiseState::Broken);
    assert_eq!(mapped.take(), None);
}
