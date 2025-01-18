use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{select, Receiver, RecvError, RecvTimeoutError, SendError, TryRecvError};
use log::debug;

use crate::fs::path::SPath;
use crate::primitives::tree::filter_policy::FilterPolicy;
use crate::primitives::tree::lazy_tree_it::LazyTreeIterator;
use crate::primitives::tree::tree_node::{FilterRef, TreeItFilter, TreeNode};
use crate::promise::streaming_promise::{StreamingPromise, StreamingPromiseState, UpdateResult};
use crate::widgets::spath_tree_view_node::FileTreeNode;

pub struct FsAsyncTreeIt {
    root: FileTreeNode,
    receiver: Option<Receiver<(u16, FileTreeNode)>>,
    handle: JoinHandle<()>,
    cache: Vec<(u16, FileTreeNode)>,
}

impl FsAsyncTreeIt {
    pub fn new(
        root: FileTreeNode,
        filter_op: Option<(FilterRef<FileTreeNode>, FilterPolicy)>,
        expanded_op: Option<HashSet<SPath>>,
    ) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<(u16, FileTreeNode)>();
        let root_copy = root.clone();

        let handle = std::thread::spawn(move || {
            let filter_op = if let Some(pair) = filter_op.as_ref() {
                Some(pair.clone())
            } else {
                None
            };

            let mut tree_it: LazyTreeIterator<SPath, FileTreeNode> = LazyTreeIterator::new(root_copy, filter_op);
            if let Some(expanded) = expanded_op.as_ref() {
                tree_it = tree_it.with_expanded(expanded)
            }

            for item in tree_it {
                if let Err(e) = sender.send(item) {
                    debug!("closing writer");
                    break;
                }
            }
        });

        // warmup
        let mut cache: Vec<(u16, FileTreeNode)> = vec![];

        // TODO add quota
        'warmup: while cache.len() < 50 {
            select! {
                recv(receiver) -> item => {
                    match item {
                        Ok(item) => cache.push(item),
                        Err(e) => break 'warmup,
                    }
                }
                default(Duration::from_millis(50)) => {
                    break 'warmup;
                }
            }
        }

        FsAsyncTreeIt {
            root,
            receiver: Some(receiver),
            handle,
            cache,
        }
    }
}

impl StreamingPromise<(u16, FileTreeNode)> for FsAsyncTreeIt {
    fn state(&self) -> StreamingPromiseState {
        if self.receiver.is_some() {
            StreamingPromiseState::Streaming
        } else {
            StreamingPromiseState::Finished
        }
    }

    fn drain(&mut self, how_long: Option<Duration>) -> StreamingPromiseState {
        if let Some(channel) = self.receiver.as_mut() {
            loop {
                match channel.recv() {
                    Ok(item) => self.cache.push(item),
                    Err(_) => return StreamingPromiseState::Finished,
                }
            }
        } else {
            StreamingPromiseState::Finished
        }
    }

    fn update(&mut self) -> UpdateResult {
        let old_state = self.state();

        if let Some(channel) = self.receiver.as_mut() {
            loop {
                match channel.try_recv() {
                    Ok(rec) => {
                        self.cache.push(rec);
                    }
                    Err(e) => {
                        return match e {
                            TryRecvError::Empty => {
                                let new_state = StreamingPromiseState::Streaming;
                                return UpdateResult {
                                    state: new_state,
                                    has_changed: old_state != new_state,
                                };
                            }
                            TryRecvError::Disconnected => {
                                let new_state = StreamingPromiseState::Finished;
                                self.receiver = None;
                                return UpdateResult {
                                    state: new_state,
                                    has_changed: old_state != new_state,
                                };
                            }
                        }
                    }
                }
            }
        } else {
            UpdateResult {
                state: StreamingPromiseState::Finished,
                has_changed: false,
            }
        }
    }

    fn read(&self) -> &Vec<(u16, FileTreeNode)> {
        &self.cache
    }
}
