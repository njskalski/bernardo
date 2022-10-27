use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::sync::{RwLock, RwLockWriteGuard};

use crossbeam_channel::{Receiver, Sender};
use log::{debug, error};
use lsp_types::{CompletionResponse, CompletionTextEdit, DocumentSymbolResponse, Location, Position, SymbolKind};
use lsp_types::request::DocumentSymbolRequest;

use crate::fs::path::SPath;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::promise::LSPPromise;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::promise::promise::Promise;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider::{Completion, CompletionAction, CompletionsPromise, NavCompProvider, Symbol, SymbolContextActionsPromise, SymbolPromise, SymbolType};

/*
TODO I am silently ignoring errors here. I guess that if NavComp fails it should get re-started.
TODO (same as above) Use NavCompRes everywhere.
 */

impl From<Position> for StupidCursor {
    fn from(loc: Position) -> Self {
        StupidCursor {
            char_idx: loc.character,
            line: loc.line,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LspError {
    IOError(LspIOError),
    // ProtocolError
}

pub struct NavCompProviderLsp {
    // TODO probably a RefCell would suffice
    lsp: RwLock<LspWrapper>,
    todo_tick_sender: NavCompTickSender,
    triggers: Vec<String>,

    error_channel: (Sender<LspError>, Receiver<LspError>),

}

impl NavCompProviderLsp {
    pub fn new(lsp: LspWrapper, tick_sender: NavCompTickSender) -> Self {
        NavCompProviderLsp {
            lsp: RwLock::new(lsp),
            todo_tick_sender: tick_sender,
            // TODO this will get lang specific
            triggers: vec![".".to_string(), "::".to_string()],
            error_channel: crossbeam_channel::unbounded(),
        }
    }

    pub fn get_url_and_lock(&self, path: &SPath) -> Option<(url::Url, RwLockWriteGuard<LspWrapper>)> {
        let url = match path.to_url() {
            Ok(url) => url,
            Err(e) => {
                error!("failed to convert spath [{}] to url", path);
                return None;
            }
        };

        match self.lsp.try_write() {
            Err(e) => {
                error!("failed acquiring write lock: {}", e);
                None
            }
            Ok(lock) => {
                Some((url, lock))
            }
        }
    }

    // pub fn map_err<R: lsp_types::request::Request>(promise: LSPPromise<R>, error_sink: Sender<LspError>) -> Box<dyn Promise<Option<R::Result>>>{
    //     promise.map(|result| {
    //         match result { }
    //     })
    // }
}


impl NavCompProvider for NavCompProviderLsp {
    fn file_open_for_edition(&self, path: &SPath, file_contents: ropey::Rope) {
        self.get_url_and_lock(path).map(|(url, mut lock)| {
            lock.text_document_did_open(url, file_contents.to_string());
        });
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: ropey::Rope) {
        self.get_url_and_lock(path).map(|(url, mut lock)| {
            lock.text_document_did_change(url, file_contents.to_string());
        });
    }

    fn completions(&self, path: SPath, cursor: StupidCursor, _trigger: Option<String>) -> Option<CompletionsPromise> {
        let error_sink = self.error_channel.0.clone();

        self.get_url_and_lock(&path).map(move |(url, mut lock)| {
            match lock.text_document_completion(url, cursor, true /*TODO*/, None /*TODO*/) {
                Err(e) => {
                    if error_sink.try_send(LspError::IOError(e)).is_err() {
                        error!("failed sending lsp error to sink")
                    };
                    None
                }
                Ok(resp) => Some(Box::new(resp.map(|cop| -> Vec<Completion> {
                    match cop {
                        None => vec![],
                        Some(comps) => {
                            match comps {
                                CompletionResponse::Array(arr) => {
                                    arr.into_iter().map(translate_completion_item).collect()
                                }
                                CompletionResponse::List(list) => {
                                    // TODO is complete ignored
                                    list.items.into_iter().map(translate_completion_item).collect()
                                }
                            }
                        }
                    }
                })) as Box<dyn Promise<Vec<Completion>> + 'static>),
            }
        }).flatten()
    }

    fn completion_triggers(&self, _path: &SPath) -> &Vec<String> {
        &self.triggers
    }

    fn todo_get_context_options(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolContextActionsPromise> {
        todo!()
    }

    fn todo_get_symbol_at(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolPromise> {
        let error_sink = self.error_channel.0.clone();

        self.get_url_and_lock(path).map(|(url, mut lock)| {
            match lock.text_document_document_symbol(url, cursor) {
                Err(e) => {
                    if error_sink.try_send(LspError::IOError(e)).is_err() {
                        error!("failed sending lsp error to sink")
                    };
                    None
                }
                Ok(p) => {
                    Some(Box::new(p.map(|response| {
                        let mut symbol_op: Option<Symbol> = None;

                        response.map(|symbol| {
                            match symbol {
                                DocumentSymbolResponse::Flat(v) => {
                                    v.first().map(|f| {
                                        symbol_op = Some(Symbol {
                                            symbol_type: f.kind.into(),
                                            // range: f.location.range,
                                            stupid_range: (f.location.range.start.into(), f.location.range.end.into()),
                                        })
                                    });
                                }
                                DocumentSymbolResponse::Nested(v) => {
                                    v.first().map(|f| {
                                        symbol_op = Some(Symbol {
                                            symbol_type: f.kind.into(),
                                            stupid_range: (f.range.start.into(), f.range.end.into()),
                                        })
                                    });
                                }
                            }
                        });
                        symbol_op
                    })) as Box<dyn Promise<Option<Symbol>> + 'static>)
                }
            }
        }).flatten()
    }

    fn file_closed(&self, path: &SPath) {
        self.get_url_and_lock(path).map(|(url, mut lock)| {
            lock.text_document_did_close(url);
        });
    }

    fn todo_navcomp_sender(&self) -> &NavCompTickSender {
        &self.todo_tick_sender
    }

    fn todo_is_healthy(&self) -> bool {
        //TODO
        true
    }
}

impl Debug for NavCompProviderLsp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NavComp({:?})", self.lsp)
    }
}

// TODO convert suggestions to CEMs?

fn translate_completion_item(i: lsp_types::CompletionItem) -> Completion {
    debug!("[{:?}]", i);
    Completion {
        key: i.label,
        desc: i.detail,
        action: CompletionAction::Insert(i.text_edit.map(|c|
            match c {
                CompletionTextEdit::Edit(e) => {
                    e.new_text
                }
                CompletionTextEdit::InsertAndReplace(e) => {
                    e.new_text
                }
            }
        ).unwrap_or("".to_string())),
    }
}

impl From<SymbolKind> for SymbolType {
    fn from(sk: SymbolKind) -> Self {
        match sk {
            SymbolKind::FILE => SymbolType::File,
            SymbolKind::MODULE => SymbolType::Module,
            SymbolKind::NAMESPACE => SymbolType::Namespace,
            SymbolKind::PACKAGE => SymbolType::Package,
            SymbolKind::CLASS => SymbolType::Class,
            SymbolKind::METHOD => SymbolType::Method,
            SymbolKind::PROPERTY => SymbolType::Property,
            SymbolKind::FIELD => SymbolType::Field,
            SymbolKind::CONSTRUCTOR => SymbolType::Constructor,
            SymbolKind::ENUM => SymbolType::Enum,
            SymbolKind::INTERFACE => SymbolType::Interface,
            SymbolKind::FUNCTION => SymbolType::Function,
            SymbolKind::VARIABLE => SymbolType::Variable,
            SymbolKind::CONSTANT => SymbolType::Constant,
            SymbolKind::STRING => SymbolType::String,
            SymbolKind::NUMBER => SymbolType::Number,
            SymbolKind::BOOLEAN => SymbolType::Boolean,
            SymbolKind::ARRAY => SymbolType::Array,
            SymbolKind::OBJECT => SymbolType::Object,
            SymbolKind::KEY => SymbolType::Key,
            SymbolKind::NULL => SymbolType::Null,
            SymbolKind::ENUM_MEMBER => SymbolType::EnumMember,
            SymbolKind::STRUCT => SymbolType::Struct,
            SymbolKind::EVENT => SymbolType::Event,
            SymbolKind::OPERATOR => SymbolType::Operator,
            SymbolKind::TYPE_PARAMETER => SymbolType::TypeParameter,
            _ => SymbolType::Unmapped(format!("{:?}", sk)),
        }
    }
}