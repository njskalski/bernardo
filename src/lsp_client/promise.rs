use std::fmt::{Debug, Formatter};

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use log::{debug, error, warn};
use lsp_types::request::Request;

use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::promise::promise::{Promise, PromiseState, UpdateResult};
use crate::w7e::navcomp_provider_lsp::LspError;

pub struct LSPPromise<R: Request> {
    //Invariant: never item and error are set in the same time. They can be both empty though.
    receiver: Receiver<jsonrpc_core::Value>,
    item: Option<R::Result>,
    // Error can occur twofold: first, pipe was broken. Second, deserialization failed. Both of these result in broken promise.
    err: Option<LspReadError>,
}

impl<R: Request> LSPPromise<R> {
    pub fn new(receiver: Receiver<jsonrpc_core::Value>, error_sink: Sender<LspReadError>) -> Self {
        LSPPromise {
            receiver,
            item: None,
            err: None,
        }
    }

    // returns whether value is available
    fn set_from_value(&mut self, value: jsonrpc_core::Value) -> PromiseState {
        match serde_json::from_value::<R::Result>(value) {
            Ok(item) => {
                self.item = Some(item);
                PromiseState::Ready
            }
            Err(err) => {
                error!("failed deserializing to {}", std::any::type_name::<R::Result>());
                self.err = Some(err.into());
                PromiseState::Broken
            }
        }
    }

    pub fn err(&self) -> Option<&LspReadError> {
        self.err.as_ref()
    }
}

impl<R: Request> Promise<R::Result> for LSPPromise<R> {
    fn state(&self) -> PromiseState {
        debug_assert!(!(self.err.is_some() && self.item.is_some()), "both error and item set");

        if self.err.is_some() {
            return PromiseState::Broken;
        }
        if self.item.is_some() {
            return PromiseState::Ready;
        }

        PromiseState::Unresolved
    }

    fn wait(&mut self) -> PromiseState {
        if self.state() == PromiseState::Broken {
            error!("wait on broken promise {:?}", self);
            return PromiseState::Broken;
        }

        if self.state() == PromiseState::Ready {
            warn!("rather unexpected second wait on promise {:?}", self);
            return PromiseState::Ready;
        }

        match self.receiver.recv() {
            Ok(value) => {
                self.set_from_value(value) // ready or broken
            }
            Err(err) => {
                error!("broken on wait: {:?}", self);
                self.err = Some(err.into());
                PromiseState::Broken
            }
        }
    }

    fn update(&mut self) -> UpdateResult {
        if self.state() == PromiseState::Broken {
            debug!("update on broken promise {:?}", self);
            return UpdateResult {
                state: PromiseState::Broken,
                has_changed: false,
            };
        }

        if self.item.is_none() {
            match self.receiver.try_recv() {
                Ok(value) => {
                    let state = self.set_from_value(value);
                    UpdateResult {
                        state: state,
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
                            warn!("promise {:?} broken", self);
                            self.err = Some(LspReadError::BrokenChannel);
                            UpdateResult {
                                state: PromiseState::Unresolved,
                                has_changed: true,
                            }
                        }
                    }
                }
            }
        } else {
            UpdateResult {
                state: PromiseState::Ready,
                has_changed: false,
            }
        }
    }

    fn read(&self) -> Option<&R::Result> {
        self.item.as_ref()
    }

    fn take(mut self) -> Option<R::Result> {
        self.update();
        self.item.take()
    }
}

// TODO remove!
impl<R: Request> Debug for LSPPromise<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?} lsp promise of {}]", self.state(), std::any::type_name::<R>())
    }
}