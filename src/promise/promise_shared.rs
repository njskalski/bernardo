// use std::sync::{Arc, LockResult, RwLock};
//
// use log::error;
//
// use crate::promise::promise::Promise;
//
// #[derive(Clone, Debug)]
// pub struct PromiseShared<A, P: Promise<A>> {
//     arc_lock: Arc<RwLock<P>>,
//
// }
//
// impl<A, P: Promise<A>> Promise<A> for PromiseShared<A, P> {
//     fn wait(&mut self) -> bool {
//         match self.arc_lock.write() {
//             Ok(mut lock) => {
//                 lock.wait()
//             }
//             Err(e) => {
//                 error!("shared promise lock poisoned: {}", e);
//                 false
//             }
//         }
//     }
//
//     fn update(&mut self) -> bool {
//         match self.arc_lock.write() {
//             Ok(mut lock) => {
//                 lock.update()
//             }
//             Err(e) => {
//                 error!("shared promise lock poisoned: {}", e);
//                 false
//             }
//         }
//     }
//
//     fn read(&self) -> Option<&A> {
//         match self.arc_lock.read() {
//             Ok(mut lock) => {
//                 lock.read()
//             }
//             Err(e) => {
//                 error!("shared promise lock poisoned: {}", e);
//                 false
//             }
//         }
//     }
//
//     fn is_broken(&self) -> bool {
//         match self.arc_lock.read() {
//             Ok(mut lock) => {
//                 lock.is_broken()
//             }
//             Err(e) => {
//                 error!("shared promise lock poisoned: {}", e);
//                 // poisoned lock -> broken promise
//                 true
//             }
//         }
//     }
//
//     fn take(self) -> Option<A> {
//         // hmm, shared take would require "emptied" promise state...
//         todo!()
//     }
// }