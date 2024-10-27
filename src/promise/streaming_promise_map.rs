use std::marker::PhantomData;
use std::time::Duration;

use crate::promise::streaming_promise::{StreamingPromise, StreamingPromiseState, UpdateResult};

pub struct MappedStreamingPromise<OldType, InternalPromise: StreamingPromise<OldType>, NewType: Clone, Mutator: Fn(&OldType) -> NewType> {
    internal: Option<InternalPromise>,
    status: StreamingPromiseState,
    mutator: Mutator,
    cached: Vec<NewType>,
    _phantom: PhantomData<OldType>,
}

impl<OldType, InternalPromise: StreamingPromise<OldType>, NewType: Clone, Mutator: Fn(&OldType) -> NewType>
    MappedStreamingPromise<OldType, InternalPromise, NewType, Mutator>
{
    pub fn new(internal: InternalPromise, mutator: Mutator) -> Self {
        MappedStreamingPromise {
            internal: Some(internal),
            status: StreamingPromiseState::Streaming,
            mutator,
            cached: vec![],
            _phantom: Default::default(),
        }
    }

    fn partial_drain(&mut self, how_long: Option<Duration>) {
        if self.status == StreamingPromiseState::Streaming {
            if let Some(internal) = self.internal.as_mut() {
                let state = internal.drain(how_long);

                for item in internal.read().iter().skip(self.cached.len()) {
                    self.cached.push((self.mutator)(item));
                }

                match state {
                    StreamingPromiseState::Streaming => {
                        self.status = StreamingPromiseState::Streaming;
                    }
                    StreamingPromiseState::Finished => {
                        self.status = StreamingPromiseState::Finished;
                        self.internal = None;
                    }
                    StreamingPromiseState::Broken => {
                        self.status = StreamingPromiseState::Broken;
                        self.internal = None;
                    }
                }
            }
        }
    }
}

impl<OldType, InternalPromise: StreamingPromise<OldType>, NewType: Clone, Mutator: Fn(&OldType) -> NewType> StreamingPromise<NewType>
    for MappedStreamingPromise<OldType, InternalPromise, NewType, Mutator>
{
    fn state(&self) -> StreamingPromiseState {
        self.status
    }

    fn drain(&mut self, how_long: Option<Duration>) -> StreamingPromiseState {
        if self.status != StreamingPromiseState::Streaming {
            return self.status;
        }

        self.partial_drain(how_long);

        self.status
    }

    fn update(&mut self) -> UpdateResult {
        if self.status != StreamingPromiseState::Streaming {
            return UpdateResult {
                state: self.status,
                has_changed: false,
            };
        }

        let old_len = self.cached.len();
        let old_status = self.status;

        if let Some(internal) = self.internal.as_mut() {
            let ur = internal.update();

            for item in internal.read().iter().skip(self.cached.len()) {
                self.cached.push((self.mutator)(item));
            }

            match ur.state {
                StreamingPromiseState::Streaming => {}
                StreamingPromiseState::Finished => {
                    self.internal = None;
                    self.status = StreamingPromiseState::Finished;
                }
                StreamingPromiseState::Broken => {
                    self.internal = None;
                    self.status = StreamingPromiseState::Broken;
                }
            }
        }

        UpdateResult {
            state: self.status,
            has_changed: (self.cached.len() != old_len) || (self.status != old_status),
        }
    }

    fn read(&self) -> &Vec<NewType> {
        &self.cached
    }
}
