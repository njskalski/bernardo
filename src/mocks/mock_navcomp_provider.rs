use std::fmt::{Debug, Formatter};
use std::sync::{LockResult, RwLock, RwLockWriteGuard};
use std::time::Duration;

use crossbeam_channel::{Receiver, select, Sender};
use lazy_static::lazy_static;
use log::{debug, error};

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::mocks::mock_navcomp_promise::MockNavCompPromise;
use crate::mocks::mock_navcomp_provider::MockNavCompEvent::{FileOpened, FileUpdated};
use crate::promise::promise::Promise;
use crate::w7e::navcomp_group::{NavCompTick, NavCompTickSender};
use crate::w7e::navcomp_provider::{Completion, CompletionsPromise, NavCompProvider};
use crate::w7e::navcomp_provider_lsp::NavCompProviderLsp;

lazy_static! {
    static ref pair: (Sender<MockNavCompEvent>, Receiver<MockNavCompEvent>) = crossbeam_channel::unbounded();
    static ref completions_behind_lock : RwLock<Vec<MockCompletionMatcher>> = Default::default();
}

pub struct MockCompletionMatcher {
    // None matches all
    pub path: Option<SPath>,
    pub answer: Option<Vec<Completion>>,
}

#[derive(Clone, Debug)]
pub enum MockNavCompEvent {
    FileOpened(SPath, String),
    FileUpdated(SPath, String),
}

pub struct MockNavCompProvider {
    triggers: Vec<String>,
    event_sender: Sender<MockNavCompEvent>,
    navcomp_tick_server: Sender<NavCompTick>,
}

impl MockNavCompProvider {
    pub fn new(navcomp_tick_server: Sender<NavCompTick>) -> Self {
        MockNavCompProvider {
            event_sender: pair.0.clone(),
            triggers: vec![".".to_string(), "::".to_string()],
            navcomp_tick_server,
        }
    }

    pub fn recvr() -> &'static Receiver<MockNavCompEvent> {
        &pair.1
    }
}

pub struct MockNavCompProviderPilot {
    recvr: Receiver<MockNavCompEvent>,
}

impl MockNavCompProviderPilot {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);

    pub fn new() -> Self {
        MockNavCompProviderPilot {
            recvr: pair.1.clone()
        }
    }

    pub fn wait_for_load(&self, requested_path: &SPath) -> Option<String> {
        loop {
            select! {
                recv(self.recvr) -> msg_res => {
                    match msg_res {
                        Ok(msg) => {
                            match msg {
                                FileOpened(opened_path, contents) if &opened_path == requested_path => {
                                    return Some(contents);
                                }
                                other => {
                                    debug!("received {:?}", other);
                                    continue;
                                }
                            }
                        },
                        Err(e) => {
                            error!("failed retrieving msg");
                            return None;
                        }
                    }
                },
                default(Self::DEFAULT_TIMEOUT) => {
                    return None;
                }
            }
        }
    }

    pub fn completions(&self) -> Option<RwLockWriteGuard<Vec<MockCompletionMatcher>>> {
        match completions_behind_lock.write() {
            Ok(lock) => Some(lock),
            Err(e) => {
                error!("failed acquiring competions lock: {:?}", e);
                None
            }
        }
    }
}

impl NavCompProvider for MockNavCompProvider {
    fn file_open_for_edition(&self, path: &SPath, file_contents: String) {
        self.event_sender.send(MockNavCompEvent::FileOpened(path.clone(), file_contents)).unwrap()
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: String) {
        self.event_sender.send(MockNavCompEvent::FileUpdated(path.clone(), file_contents)).unwrap()
    }

    fn completions(&self, path: SPath, cursor: LspTextCursor, trigger: Option<String>) -> Option<CompletionsPromise> {
        match completions_behind_lock.read() {
            Err(e) => {
                error!("failed acquiring lock on completions: {:?}", e);
                None
            }
            Ok(completions) => {
                completions
                    .iter()
                    .find(|c| c.path.as_ref().map(|spath| *spath == path).unwrap_or(true))
                    .map(|c| {
                        match c.answer.as_ref() {
                            None => {
                                Box::new(
                                    MockNavCompPromise::<Vec<Completion>>::new_broken(self.navcomp_tick_server.clone())
                                ) as Box<dyn Promise<Vec<Completion>> + 'static>
                            }
                            Some(value) => {
                                Box::new(
                                    MockNavCompPromise::new_succ(self.navcomp_tick_server.clone(), value.clone())
                                ) as Box<dyn Promise<Vec<Completion>> + 'static>
                            }
                        }
                    })
            }
        }
    }

    fn completion_triggers(&self, path: &SPath) -> &Vec<String> {
        &self.triggers
    }

    fn file_closed(&self, path: &SPath) {}

    fn todo_navcomp_sender(&self) -> &NavCompTickSender {
        &self.navcomp_tick_server
    }
}

impl Debug for MockNavCompProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[MockNavCompProvider]")
    }
}