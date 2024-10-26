use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::promise::streaming_promise::{StreamingPromise, StreamingPromiseState, UpdateResult};

struct MockStreamingPromise {
    status: StreamingPromiseState,
    items: Vec<i32>,
    has_changed: bool,
}

impl StreamingPromise<i32> for Arc<RwLock<MockStreamingPromise>> {
    fn state(&self) -> StreamingPromiseState {
        RwLock::read(self).unwrap().status
    }

    fn drain(&mut self, how_long: Option<Duration>) -> StreamingPromiseState {
        RwLock::read(self).unwrap().status
    }

    fn update(&mut self) -> UpdateResult {
        let item = RwLock::read(self).unwrap();

        UpdateResult {
            state: item.status,
            has_changed: item.has_changed,
        }
    }

    fn read(&self) -> Box<dyn Iterator<Item = &i32> + '_> {
        todo!()
    }

    fn take(self) -> Vec<i32> {
        let mut item = self.write().unwrap();
        let mut nananana: Vec<i32> = vec![];
        std::mem::swap(&mut item.items, &mut nananana);
        nananana
    }
}
