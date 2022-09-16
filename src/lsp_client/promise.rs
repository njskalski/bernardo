use std::cell::RefCell;
use std::fmt::{Debug, Formatter};

use crossbeam_channel::{Receiver, RecvError, TryRecvError};
use log::{error, warn};
use lsp_types::request::Request;
use serde_json::Error;

use crate::lsp_client::lsp_read_error::LspReadError;

pub trait Promise<T> {
    // Returns true if wait succeded, false if not
    fn wait(&mut self) -> bool;

    // Non-blocking wait. Returns whether *state changed*, not whether value is available.
    fn update(&mut self) -> bool;

    /*
    Returns value, provided it was retrieved before. Does *not* change state, so if you want an
    equivalent to Future.poll(), it's {Promise.update(); Promise.read()}
     */
    fn read(&self) -> Option<&T>;

    /*
    Returns true, iff there is no chance that it will ever resolve successfully.
     */
    fn is_broken(&self) -> bool;

    /*
    Returns true iff value is available to read.
     */
    fn is_delivered(&self) -> bool;

    /*
    Consumes promise returning value inside. It *does not* poll for message, so it's *not* an
    equivalent of Future.now_or_never().
     */
    fn take(self) -> Option<T>;
}

pub struct LSPPromise<R: Request> {
    receiver: Receiver<jsonrpc_core::Value>,
    item: Option<R::Result>,
    err: Option<LspReadError>,
}

impl<R: Request> LSPPromise<R> {
    pub fn new(receiver: Receiver<jsonrpc_core::Value>) -> Self {
        LSPPromise {
            receiver,
            item: None,
            err: None,
        }
    }

    fn set_from_value(&mut self, value: jsonrpc_core::Value) -> bool {
        match serde_json::from_value::<R::Result>(value) {
            Ok(item) => {
                self.item = Some(item);
                true
            }
            Err(err) => {
                error!("failed deserializing to {}", std::any::type_name::<R::Result>());
                self.err = Some(err.into());
                false
            }
        }
    }
}

impl<R: Request> Promise<R::Result> for LSPPromise<R> {
    fn wait(&mut self) -> bool {
        if self.is_broken() {
            error!("wait on broken promise {:?}", self);
            return false;
        }
        if self.is_delivered() {
            warn!("rather unexpected wait on {:?}", self);
            return true;
        }

        match self.receiver.recv() {
            Ok(value) => {
                self.set_from_value(value)
            }
            Err(err) => {
                error!("broken on wait: {:?}", self);
                self.err = Some(err.into());
                false
            }
        }
    }

    fn update(&mut self) -> bool {
        if self.is_broken() {
            return false;
        }

        if self.item.is_none() {
            match self.receiver.try_recv() {
                Ok(value) => {
                    self.set_from_value(value)
                }
                Err(e) => {
                    match e {
                        TryRecvError::Empty => {
                            false
                        }
                        TryRecvError::Disconnected => {
                            warn!("promise {:?} broken", self);
                            self.err = Some(LspReadError::BrokenChannel);
                            false
                        }
                    }
                }
            }
        } else {
            false
        }
    }

    fn read(&self) -> Option<&R::Result> {
        self.item.as_ref()
    }

    fn is_broken(&self) -> bool {
        self.err.is_some()
    }

    fn is_delivered(&self) -> bool {
        self.item.is_some()
    }

    fn take(mut self) -> Option<R::Result> {
        self.update();
        self.item.take()
    }
}

impl<R: Request> Debug for LSPPromise<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_broken() {
            write!(f, "[broken promise of {}]", std::any::type_name::<R>())
        } else {
            match &self.item {
                None => write!(f, "[undelivered promise of {}]", std::any::type_name::<R>()),
                Some(_) => write!(f, "[delivered promise of {}]", std::any::type_name::<R>()),
            }
        }
    }
}
