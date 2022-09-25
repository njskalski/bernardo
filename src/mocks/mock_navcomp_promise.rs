use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, TryRecvError};
use log::debug;

use crate::promise::promise::{Promise, PromiseState, UpdateResult};
use crate::w7e::navcomp_provider::Completion;

pub struct MockNavCompPromise<T: Send + 'static> {
    receiver: Receiver<T>,
    item: Option<T>,
    done: bool,
}

impl<T: Send + 'static> MockNavCompPromise<T> {
    const DEFAULT_DELAY: Duration = Duration::from_millis(600);

    pub fn new_succ(value: T) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded::<T>(1);

        thread::spawn(move || {
            thread::sleep(Self::DEFAULT_DELAY);
            sender.send(value).unwrap();
        });

        MockNavCompPromise {
            receiver,
            item: None,
            done: false,
        }
    }

    pub fn new_broken() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded::<T>(1);

        thread::spawn(move || {
            thread::sleep(Self::DEFAULT_DELAY);
            //bs call so it's taken
            debug!("breaking promise {}", sender.is_full());
        });

        MockNavCompPromise {
            receiver,
            item: None,
            done: false,
        }
    }
}

impl<T: Send + 'static> Promise<T> for MockNavCompPromise<T> {
    fn state(&self) -> PromiseState {
        if self.done {
            if self.item.is_some() {
                PromiseState::Ready
            } else {
                PromiseState::Broken
            }
        } else {
            PromiseState::Unresolved
        }
    }

    fn wait(&mut self) -> PromiseState {
        match self.receiver.recv() {
            Ok(value) => {
                self.item = Some(value);
                self.done = true;
                PromiseState::Ready
            }
            Err(e) => {
                self.done = true;
                PromiseState::Broken
            }
        }
    }

    fn update(&mut self) -> UpdateResult {
        if self.done {
            return if self.item.is_some() {
                UpdateResult {
                    state: PromiseState::Ready,
                    has_changed: false,
                }
            } else {
                UpdateResult {
                    state: PromiseState::Broken,
                    has_changed: false,
                }
            };
        }

        return match self.receiver.try_recv() {
            Ok(value) => {
                self.item = Some(value);
                self.done = true;
                UpdateResult {
                    state: PromiseState::Ready,
                    has_changed: true,
                }
            }
            Err(e) => {
                match e {
                    TryRecvError::Empty => {
                        UpdateResult {
                            state: PromiseState::Unresolved,
                            has_changed: false,
                        }
                    }
                    TryRecvError::Disconnected => {
                        self.done = true;
                        UpdateResult {
                            state: PromiseState::Broken,
                            has_changed: true,
                        }
                    }
                }
            }
        };
    }

    fn read(&self) -> Option<&T> {
        self.item.as_ref()
    }

    fn take(mut self) -> Option<T> {
        self.item.take()
    }
}