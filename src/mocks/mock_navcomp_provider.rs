use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender};
use log::{debug, error};

use crate::fs::path::SPath;
use crate::mocks::mock_navcomp_promise::MockNavCompPromise;
use crate::mocks::mock_navcomp_provider::MockNavCompEvent::FileOpened;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::promise::promise::Promise;
use crate::unpack_or_e;
use crate::w7e::navcomp_group::{NavCompTick, NavCompTickSender};
use crate::w7e::navcomp_provider::{
    Completion, CompletionsPromise, FormattingPromise, NavCompProvider, NavCompSymbol, SymbolContextActionsPromise, SymbolType,
    SymbolUsage, SymbolUsagesPromise,
};

pub struct MockCompletionMatcher {
    // None matches all
    pub path: Option<SPath>,
    // None means "return broken promise"
    pub answer: Option<Vec<Completion>>,
}

pub struct MockSymbolMatcher {
    pub path: Option<SPath>,
    pub symbol: NavCompSymbol,
    // None means "return broken promise"
    pub usages: Option<Vec<SymbolUsage>>,
}

impl MockSymbolMatcher {
    pub fn matches(&self, path: Option<&SPath>, stupid_cursor: StupidCursor) -> bool {
        let path_matches = self.path.as_ref() == path;
        let range_matches = stupid_cursor.is_between(self.symbol.stupid_range.0, self.symbol.stupid_range.1);

        path_matches && range_matches
    }

    pub fn symbol_type(&self) -> &SymbolType {
        &self.symbol.symbol_type
    }

    pub fn symbol(&self) -> &NavCompSymbol {
        &self.symbol
    }
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
    completions: Arc<RwLock<Vec<MockCompletionMatcher>>>,
    symbols: Arc<RwLock<Vec<MockSymbolMatcher>>>,
}

impl MockNavCompProvider {
    pub fn new(
        navcomp_tick_server: Sender<NavCompTick>,
        event_sender: Sender<MockNavCompEvent>,
        completions: Arc<RwLock<Vec<MockCompletionMatcher>>>,
        symbols: Arc<RwLock<Vec<MockSymbolMatcher>>>,
    ) -> Self {
        MockNavCompProvider {
            event_sender,
            triggers: vec![".".to_string(), "::".to_string()],
            navcomp_tick_server,
            completions,
            symbols,
        }
    }
}

pub struct MockNavCompProviderPilot {
    recvr: Receiver<MockNavCompEvent>,
    completions: Arc<RwLock<Vec<MockCompletionMatcher>>>,
    symbols: Arc<RwLock<Vec<MockSymbolMatcher>>>,
}

impl MockNavCompProviderPilot {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);

    pub fn new(
        recvr: Receiver<MockNavCompEvent>,
        completions: Arc<RwLock<Vec<MockCompletionMatcher>>>,
        symbols: Arc<RwLock<Vec<MockSymbolMatcher>>>,
    ) -> Self {
        MockNavCompProviderPilot {
            recvr,
            completions,
            symbols,
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
                            error!("failed retrieving msg: {:?}", e);
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
        match self.completions.write() {
            Ok(lock) => Some(lock),
            Err(e) => {
                error!("failed acquiring competions lock: {:?}", e);
                None
            }
        }
    }

    pub fn symbols(&self) -> Option<RwLockWriteGuard<Vec<MockSymbolMatcher>>> {
        match self.symbols.write() {
            Ok(lock) => Some(lock),
            Err(e) => {
                error!("failed acquiring symbols lock: {:?}", e);
                None
            }
        }
    }
}

impl NavCompProvider for MockNavCompProvider {
    fn file_open_for_edition(&self, path: &SPath, file_contents: ropey::Rope) {
        self.event_sender
            .send(MockNavCompEvent::FileOpened(path.clone(), file_contents.to_string()))
            .unwrap()
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: ropey::Rope) {
        self.event_sender
            .send(MockNavCompEvent::FileUpdated(path.clone(), file_contents.to_string()))
            .unwrap()
    }

    fn completions(&self, path: SPath, _cursor: StupidCursor, _trigger: Option<String>) -> Option<CompletionsPromise> {
        let completions = unpack_or_e!(self.completions.read().ok(), None, "failed acquiring lock on completions");

        let res = completions
            .iter()
            .find(|c| c.path.as_ref().map(|spath| *spath == path).unwrap_or(true))
            .map(|c| match c.answer.as_ref() {
                None => {
                    debug!("returning broken completion promise");
                    Box::new(MockNavCompPromise::<Vec<Completion>>::new_broken(self.navcomp_tick_server.clone()))
                        as Box<dyn Promise<Vec<Completion>> + 'static>
                }
                Some(value) => {
                    debug!("returning successful completion promise");
                    Box::new(MockNavCompPromise::new_succ(self.navcomp_tick_server.clone(), value.clone()))
                        as Box<dyn Promise<Vec<Completion>> + 'static>
                }
            });

        if res.is_none() {
            debug!("no results for completion");
        }

        res
    }

    fn completion_triggers(&self, _path: &SPath) -> &Vec<String> {
        &self.triggers
    }

    fn todo_get_context_options(&self, _path: &SPath, _cursor: StupidCursor) -> Option<SymbolContextActionsPromise> {
        None
    }

    // fn todo_get_symbol_at(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolPromise> {
    //     let symbols = unpack_or_e!(self.symbols.read().ok(), None, "failed acquiring lock on
    // symbols");
    //
    //     let res = symbols
    //         .iter()
    //         .find(|candidate| candidate.matches(Some(path), cursor))
    //         .map(|c| {
    //             match c.usages.as_ref() {
    //                 None => {
    //                     debug!("returning broken symbol promise");
    //                     Box::new(
    // MockNavCompPromise::<Option<NavCompSymbol>>::new_broken(self.navcomp_tick_server.clone())
    //                     ) as SymbolPromise
    //                 }
    //                 Some(_) => {
    //                     debug!("returning successful symbol promise");
    //                     Box::new(
    //                         MockNavCompPromise::new_succ(self.navcomp_tick_server.clone(),
    // Some(c.symbol.clone()))                     ) as SymbolPromise
    //                 }
    //             }
    //         });
    //
    //     if res.is_none() {
    //         debug!("no results for symbol");
    //     }
    //
    //     res
    // }

    fn todo_get_symbol_usages(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolUsagesPromise> {
        let symbols = unpack_or_e!(self.symbols.read().ok(), None, "failed acquiring lock on symbols");

        let res = symbols
            .iter()
            .find(|candidate| candidate.matches(Some(path), cursor))
            .map(|c| match c.usages.as_ref() {
                None => {
                    debug!("returning broken symbol usages promise");
                    Box::new(MockNavCompPromise::<Vec<SymbolUsage>>::new_broken(self.navcomp_tick_server.clone())) as SymbolUsagesPromise
                }
                Some(usages) => {
                    debug!("returning successful symbol usages promise");
                    Box::new(MockNavCompPromise::new_succ(self.navcomp_tick_server.clone(), usages.clone())) as SymbolUsagesPromise
                }
            });

        if res.is_none() {
            debug!("no results for symbol usages");
        }

        res
    }

    fn todo_reformat(&self, _path: &SPath) -> Option<FormattingPromise> {
        todo!()
    }

    fn file_closed(&self, _path: &SPath) {}

    fn todo_navcomp_sender(&self) -> &NavCompTickSender {
        &self.navcomp_tick_server
    }

    fn todo_is_healthy(&self) -> bool {
        todo!()
    }
}

impl Debug for MockNavCompProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[MockNavCompProvider]")
    }
}
