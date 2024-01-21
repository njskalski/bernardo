use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::time;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use log::{debug, error, warn};
use lsp_types::request::Request;
use serde_json::Value;

use crate::lsp_client::lsp_read_error::LspReadError;
use crate::promise::promise::{Promise, PromiseState, UpdateResult};

pub struct LSPPromise<R: Request> {
    //Invariant: never item and error are set in the same time. They can be both empty though.
    receiver: Receiver<jsonrpc_core::Value>,
    item: Option<R::Result>,
    err: Option<LspReadError>,
    error_sink: Sender<LspReadError>,
}

impl<R: Request> LSPPromise<R> {
    pub fn new(receiver: Receiver<jsonrpc_core::Value>, error_sink: Sender<LspReadError>) -> Self {
        LSPPromise {
            receiver,
            item: None,
            err: None,
            error_sink,
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
                self.err = Some(LspReadError::DeError(err.to_string()));
                if let Err(e) = self.error_sink.try_send(self.err.as_ref().unwrap().clone()) {
                    error!("failed sending LSP Error [{:?}] to sink, due [{:?}]", self.err, e);
                }
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

    fn wait(&mut self, how_long: Option<Duration>) -> PromiseState {
        if self.state() == PromiseState::Broken {
            error!("wait on broken promise {:?}", self);
            return PromiseState::Broken;
        }

        if self.state() == PromiseState::Ready {
            warn!("rather unexpected second wait on promise {:?}", self);
            return PromiseState::Ready;
        }

        let _timeout = false;
        let (timeout, value_op): (bool, Option<Value>) = if let Some(duration) = how_long {
            match self.receiver.recv_deadline(time::Instant::now().add(duration)) {
                Ok(value) => (false, Some(value)),
                Err(e) => match e {
                    RecvTimeoutError::Timeout => (true, None),
                    RecvTimeoutError::Disconnected => (false, None),
                },
            }
        } else {
            (false, self.receiver.recv().ok())
        };

        assert!((timeout && value_op.is_some()) == false);

        if timeout {
            PromiseState::Unresolved
        } else {
            if let Some(value) = value_op {
                self.set_from_value(value)
            } else {
                self.err = Some(LspReadError::BrokenChannel);
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
                Err(e) => match e {
                    TryRecvError::Empty => UpdateResult {
                        state: PromiseState::Unresolved,
                        has_changed: false,
                    },
                    TryRecvError::Disconnected => {
                        warn!("promise {:?} broken", self);
                        self.err = Some(LspReadError::BrokenChannel);
                        if let Err(e) = self.error_sink.try_send(self.err.as_ref().unwrap().clone()) {
                            error!("failed sending LSP Error [{:?}] to sink, due [{:?}]", self.err, e);
                        }

                        UpdateResult {
                            state: PromiseState::Unresolved,
                            has_changed: true,
                        }
                    }
                },
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
