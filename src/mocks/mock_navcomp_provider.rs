use std::fmt::{Debug, Formatter};
use std::time::Duration;

use crossbeam_channel::{Receiver, select, Sender};
use lazy_static::lazy_static;
use log::{debug, error};

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::mocks::mock_navcomp_provider::MockNavCompEvent::FileOpened;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider::{CompletionsPromise, NavCompProvider};

lazy_static! {
    static ref pair: (Sender<MockNavCompEvent>, Receiver<MockNavCompEvent>) = crossbeam_channel::unbounded();
}

pub struct MockCompletionMatcher {
    pub path: Option<SPath>,
}

#[derive(Clone, Debug)]
pub enum MockNavCompEvent {
    FileOpened(SPath, String),
}

pub struct MockNavCompProvider {
    pub completions: Vec<MockCompletionMatcher>,

    event_sender: Sender<MockNavCompEvent>,
}

impl MockNavCompProvider {
    pub fn new() -> Self {
        MockNavCompProvider {
            completions: Default::default(),
            event_sender: pair.0.clone(),
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
}

impl NavCompProvider for MockNavCompProvider {
    fn file_open_for_edition(&self, path: &SPath, file_contents: String) {
        self.event_sender.send(MockNavCompEvent::FileOpened(path.clone(), file_contents)).unwrap()
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: String) {
        todo!()
    }

    fn completions(&self, path: SPath, cursor: LspTextCursor, trigger: Option<String>) -> Option<CompletionsPromise> {
        todo!()
    }

    fn completion_triggers(&self, path: &SPath) -> &Vec<String> {
        todo!()
    }

    fn file_closed(&self, path: &SPath) {
        todo!()
    }

    fn todo_navcomp_sender(&self) -> &NavCompTickSender {
        todo!()
    }
}

impl Debug for MockNavCompProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[MockNavCompProvider]")
    }
}