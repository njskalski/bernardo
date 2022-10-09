use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use log::{debug, error};

use crate::promise::promise::{Promise, PromiseState, UpdateResult};
use crate::tsw::lang_id::LangId;
use crate::w7e::navcomp_group::NavCompTick;

pub struct MockNavCompPromise<T: Send + 'static> {
    receiver: Receiver<T>,
    item: Option<T>,
    done: bool,
    join_handle: thread::JoinHandle<()>,
}

impl<T: Send + 'static> MockNavCompPromise<T> {
    const DEFAULT_DELAY: Duration = Duration::from_millis(100);

    pub fn new_succ(tick_sender: Sender<NavCompTick>, value: T) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded::<T>(1);

        let join_handle = thread::spawn(move || {
            thread::sleep(Self::DEFAULT_DELAY);
            if sender.send(value).is_ok() {
                tick_sender.send(NavCompTick::LspTick(LangId::RUST, 0)).unwrap();
                debug!("sent succ");
            } else {
                error!("succ mock DID NOT send");
            }
        });

        MockNavCompPromise {
            receiver,
            item: None,
            done: false,
            join_handle,
        }
    }

    pub fn new_broken(tick_sender: Sender<NavCompTick>) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded::<T>(1);

        let join_handle = thread::spawn(move || {
            thread::sleep(Self::DEFAULT_DELAY);
            //bs call so it's taken
            tick_sender.send(NavCompTick::LspTick(LangId::RUST, 0)).unwrap();
            debug!("breaking promise {}", sender.is_full());
        });

        MockNavCompPromise {
            receiver,
            item: None,
            done: false,
            join_handle,
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
            Err(_) => {
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