use std::ops::Add;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvError, RecvTimeoutError, TryRecvError};
use log::warn;
use streaming_iterator::{IntoStreamingIterator, StreamingIterator};

use crate::promise::streaming_promise::{StreamingPromise, StreamingPromiseState, UpdateResult};

/*
This promise cannot be "broken", it's always finished or streaming.
 */
pub struct WrappedMspcReceiver<A> {
    receiver: Receiver<A>,
    cached: Vec<A>,
    status: StreamingPromiseState,
}

impl<A> StreamingPromise<A> for WrappedMspcReceiver<A> {
    fn state(&self) -> StreamingPromiseState {
        self.status
    }

    // Wait without deadline is blocking until the end of stream.
    fn drain(&mut self, how_long: Option<Duration>) -> StreamingPromiseState {
        if self.status != StreamingPromiseState::Streaming {
            // Statuses Finished and Broken are final.
            return self.status;
        }

        if let Some(duration) = how_long {
            let deadline = Instant::now().add(duration);

            // we break that loop by timeout or by disconnection (finished)
            loop {
                match self.receiver.recv_deadline(deadline) {
                    Ok(item) => {
                        self.cached.push(item);
                    }
                    Err(error) => match error {
                        RecvTimeoutError::Timeout => {
                            return StreamingPromiseState::Streaming;
                        }
                        RecvTimeoutError::Disconnected => {
                            self.status = StreamingPromiseState::Finished;
                            return self.status;
                        }
                    },
                }
            }
        } else {
            loop {
                match self.receiver.recv() {
                    Ok(item) => {
                        self.cached.push(item);
                    }
                    Err(error) => match error {
                        RecvError => {
                            self.status = StreamingPromiseState::Finished;
                            return self.status;
                        }
                    },
                }
            }
        }
    }

    fn update(&mut self) -> UpdateResult {
        if self.status != StreamingPromiseState::Streaming {
            return UpdateResult {
                state: self.status,
                has_changed: false,
            };
        }

        let old_state = self.status;
        let mut changed = false;

        loop {
            match self.receiver.try_recv() {
                Ok(item) => {
                    self.cached.push(item);
                    changed = true;
                }
                Err(error) => match error {
                    TryRecvError::Empty => {
                        break;
                    }
                    TryRecvError::Disconnected => {
                        self.status = StreamingPromiseState::Finished;
                        break;
                    }
                },
            }
        }

        UpdateResult {
            state: self.status,
            has_changed: changed || (old_state != self.status),
        }
    }

    fn read(&self) -> Box<dyn Iterator<Item = &A> + '_> {
        let items: Vec<&A> = self.cached.iter().collect();
        Box::new(items.into_iter())
    }

    fn take(self) -> Vec<A> {
        if self.status != StreamingPromiseState::Finished {
            warn!("warning, taking result of non-finished stream");
        }

        self.cached
    }
}
