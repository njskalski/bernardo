use std::fmt::{Debug, Formatter};

use crossbeam_channel::{Receiver, Sender, unbounded};

use crate::fs::path::SPath;
use crate::lsp_client::helpers::LspTextCursor;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider::{CompletionsPromise, NavCompProvider};

pub struct MockCompletionMatcher {
    pub path: Option<SPath>,
}

pub enum MockNavCompEvent {
    FileOpened(SPath),
}

pub struct MockNavCompProvider {
    pub completions: Vec<MockCompletionMatcher>,

    event_sender: Sender<MockNavCompEvent>,
    event_receiver: Receiver<MockNavCompEvent>,
}

impl MockNavCompProvider {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = unbounded::<MockNavCompEvent>();

        MockNavCompProvider {
            completions: Default::default(),
            event_sender,
            event_receiver,
        }
    }
}


impl NavCompProvider for MockNavCompProvider {
    fn file_open_for_edition(&self, path: &SPath, file_contents: String) {
        self.event_sender.send(MockNavCompEvent::FileOpened(path.clone())).unwrap()
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