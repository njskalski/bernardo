use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::sync::RwLock;

use crossbeam_channel::{Receiver, Sender};
use log::{debug, error};
use lsp_types::{CompletionResponse, CompletionTextEdit, GotoDefinitionResponse, Location, LocationLink, Position, SymbolKind};

use crate::fs::path::SPath;
use crate::lsp_client::lsp_client::LspWrapper;
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_write_error::LspWriteError;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::promise::promise::Promise;
use crate::tsw::lang_id::LangId;
use crate::w7e::navcomp_group::NavCompTickSender;
use crate::w7e::navcomp_provider::{
    Completion, CompletionAction, CompletionsPromise, FormattingPromise, NavCompProvider, StupidSubstituteMessage,
    SymbolContextActionsPromise, SymbolType, SymbolUsage, SymbolUsagesPromise,
};
use crate::{unpack_or_e, unpack_unit_e};

/*
TODO I am silently ignoring errors here. I guess that if NavComp fails it should get re-started.
TODO (same as above) Use NavCompRes everywhere.
 */

impl From<Position> for StupidCursor {
    fn from(loc: Position) -> Self {
        StupidCursor {
            char_idx_0b: loc.character,
            line_0b: loc.line,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LspError {
    IOError(LspIOError),
    // ProtocolError
}

pub struct NavCompProviderLsp {
    lsp: RwLock<LspWrapper>,
    todo_tick_sender: NavCompTickSender,
    triggers: Vec<String>,
    read_error_channel: (Sender<LspReadError>, Receiver<LspReadError>),

    //
    crashed: RwLock<bool>,
}

impl NavCompProviderLsp {
    // TODO add errors
    pub fn new(lsp_path: PathBuf, workspace_root: PathBuf, language: LangId, tick_sender: NavCompTickSender) -> Option<Self> {
        let error_channel = crossbeam_channel::unbounded::<LspReadError>();

        if let Some(mut lsp) = LspWrapper::new(lsp_path, workspace_root, language, tick_sender.clone(), error_channel.0.clone()) {
            match lsp.initialize() {
                Ok(lsp_answer) => {
                    debug!("lsp initialization success: {:?}", lsp_answer);

                    Some(NavCompProviderLsp {
                        lsp: RwLock::new(lsp),
                        todo_tick_sender: tick_sender,
                        // TODO this will get lang specific
                        triggers: vec![".".to_string(), "::".to_string()],
                        read_error_channel: error_channel,
                        crashed: RwLock::new(false),
                    })
                }
                Err(lsp_err) => {
                    error!("lsp error {:?}", lsp_err);
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn eat_write_error(&self, error: LspWriteError) {
        error!("LSP: marking as crashed, failed write: {:?}", error);
        self.crashed.try_write().map(|mut lock| *lock = true).unwrap_or_else(|_| {
            error!("failed to acquire lock for crashed field. This is super weird, it shouldn't be even shared.");
        });
    }
}

// I am reusing SymbolUsage to NOT write a new widget while implementing go-to-definition.
// This might be changed later.
fn location_to_symbol_usage(loc: Location) -> SymbolUsage {
    SymbolUsage {
        path: loc.uri.to_string(),
        stupid_range: (
            StupidCursor {
                char_idx_0b: loc.range.start.character,
                line_0b: loc.range.start.line,
            },
            StupidCursor {
                char_idx_0b: loc.range.end.character,
                line_0b: loc.range.end.line,
            },
        ),
    }
}

fn location_link_to_symbol_usage(loc: LocationLink) -> SymbolUsage {
    SymbolUsage {
        path: loc.target_uri.to_string(),
        stupid_range: (
            StupidCursor {
                char_idx_0b: loc.target_range.start.character,
                line_0b: loc.target_range.start.line,
            },
            StupidCursor {
                char_idx_0b: loc.target_range.end.character,
                line_0b: loc.target_range.end.line,
            },
        ),
    }
}

impl NavCompProvider for NavCompProviderLsp {
    fn file_open_for_edition(&self, path: &SPath, file_contents: ropey::Rope) {
        let url = unpack_unit_e!(path.to_url().ok(), "failed to convert spath [{}] to url", path);
        let mut lock = unpack_unit_e!(self.lsp.try_write().ok(), "failed acquiring lock",);

        lock.text_document_did_open(url, file_contents.to_string());
    }

    fn submit_edit_event(&self, path: &SPath, file_contents: ropey::Rope) {
        let url = unpack_unit_e!(path.to_url().ok(), "failed to convert spath [{}] to url", path);
        let mut lock = unpack_unit_e!(self.lsp.try_write().ok(), "failed acquiring lock",);

        lock.text_document_did_change(url, file_contents.to_string());
    }

    fn completions(&self, path: SPath, cursor: StupidCursor, _trigger: Option<String>) -> Option<CompletionsPromise> {
        let url = unpack_or_e!(path.to_url().ok(), None, "failed to convert spath [{}] to url", path);
        let mut lock = unpack_or_e!(self.lsp.try_write().ok(), None, "failed acquiring lock");

        match lock.text_document_completion(url, cursor, true /* TODO */, None /* TODO */) {
            Ok(resp) => {
                let new_promise = resp.map(|cop| {
                    match cop {
                        None => vec![],
                        Some(comps) => {
                            match comps {
                                CompletionResponse::Array(arr) => arr.into_iter().map(translate_completion_item).collect(),
                                CompletionResponse::List(list) => {
                                    // TODO is complete ignored
                                    list.items.into_iter().map(translate_completion_item).collect()
                                }
                            }
                        }
                    }
                });

                Some(Box::new(new_promise))
            }
            Err(e) => {
                self.eat_write_error(e);
                None
            }
        }
    }

    fn completion_triggers(&self, _path: &SPath) -> &Vec<String> {
        &self.triggers
    }

    // fn todo_get_symbol_at(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolPromise> {
    //     let url = unpack_or_e!(path.to_url().ok(), None, "failed to convert spath [{}] to url",
    // path);     let mut lock = unpack_or_e!(self.lsp.try_write().ok(), None, "failed acquiring
    // lock");
    //
    //     match lock.text_document_document_symbol(url) {
    //         Ok(resp) => {
    //             let new_promise = resp.map(|response| {
    //                 let mut symbol_op: Option<NavCompSymbol> = None;
    //                 response.map(|symbol| {
    //                     match symbol {
    //                         DocumentSymbolResponse::Flat(v) => {
    //                             v.last().map(|f| {
    //                                 symbol_op = Some(NavCompSymbol {
    //                                     symbol_type: f.kind.into(),
    //                                     // range: f.location.range,
    //                                     stupid_range: (f.location.range.start.into(),
    // f.location.range.end.into()),                                 })
    //                             });
    //                         }
    //                         DocumentSymbolResponse::Nested(v) => {
    //                             v.last().map(|f| {
    //                                 symbol_op = Some(NavCompSymbol {
    //                                     symbol_type: f.kind.into(),
    //                                     stupid_range: (f.range.start.into(), f.range.end.into()),
    //                                 })
    //                             });
    //                         }
    //                     }
    //                 });
    //                 symbol_op
    //             });
    //
    //             Some(Box::new(new_promise))
    //         }
    //         Err(e) => {
    //             self.eat_write_error(e);
    //             None
    //         }
    //     }
    // }

    fn get_symbol_usages(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolUsagesPromise> {
        let url = unpack_or_e!(path.to_url().ok(), None, "failed to convert spath [{}] to url", path);
        let mut lock = unpack_or_e!(self.lsp.try_write().ok(), None, "failed acquiring lock");

        match lock.text_document_references(url, cursor) {
            Ok(resp) => {
                let new_promise = resp.map(|response| match response {
                    None => Vec::new(),
                    Some(items) => items
                        .into_iter()
                        .map(|loc| SymbolUsage {
                            path: loc.uri.to_string(),
                            stupid_range: (
                                StupidCursor {
                                    char_idx_0b: loc.range.start.character,
                                    line_0b: loc.range.start.line,
                                },
                                StupidCursor {
                                    char_idx_0b: loc.range.end.character,
                                    line_0b: loc.range.end.line,
                                },
                            ),
                        })
                        .collect(),
                });

                Some(Box::new(new_promise))
            }
            Err(e) => {
                self.eat_write_error(e);
                None
            }
        }
    }

    fn go_to_definition(&self, path: &SPath, cursor: StupidCursor) -> Option<SymbolUsagesPromise> {
        let url = unpack_or_e!(path.to_url().ok(), None, "failed to convert spath [{}] to url", path);
        let mut lock = unpack_or_e!(self.lsp.try_write().ok(), None, "failed acquiring lock");

        match lock.text_document_goto_definition(url, cursor) {
            Ok(resp) => {
                let new_promise = resp.map(|response| match response {
                    None => Vec::new(),
                    Some(items) => {
                        let mut locations: Vec<SymbolUsage> = Vec::default();

                        match items {
                            GotoDefinitionResponse::Scalar(item) => {
                                locations.push(location_to_symbol_usage(item));
                            }
                            GotoDefinitionResponse::Array(links) => {
                                for link in links {
                                    locations.push(location_to_symbol_usage(link));
                                }
                            }
                            GotoDefinitionResponse::Link(links) => {
                                locations = links.into_iter().map(location_link_to_symbol_usage).collect();
                            }
                        }

                        locations
                    }
                });

                Some(Box::new(new_promise))
            }
            Err(e) => {
                self.eat_write_error(e);
                None
            }
        }
    }

    fn todo_reformat(&self, path: &SPath) -> Option<FormattingPromise> {
        let url = unpack_or_e!(path.to_url().ok(), None, "failed to convert spath [{}] to url", path);
        let mut lock = unpack_or_e!(self.lsp.try_write().ok(), None, "failed acquiring lock");

        match lock.text_document_formatting(url) {
            Ok(resp) => {
                let new_promise = resp.map(|response| match response {
                    None => None,
                    Some(vte) => {
                        let stupid_edits: Vec<StupidSubstituteMessage> = vte
                            .into_iter()
                            .map(|te| StupidSubstituteMessage {
                                substitute: te.new_text,
                                stupid_range: (
                                    StupidCursor {
                                        char_idx_0b: te.range.start.character,
                                        line_0b: te.range.start.line,
                                    },
                                    StupidCursor {
                                        char_idx_0b: te.range.end.character,
                                        line_0b: te.range.end.line,
                                    },
                                ),
                            })
                            .collect();

                        Some(stupid_edits)
                    }
                });

                Some(Box::new(new_promise))
            }
            Err(e) => {
                self.eat_write_error(e);
                None
            }
        }
    }

    fn file_closed(&self, path: &SPath) {
        let url = unpack_unit_e!(path.to_url().ok(), "failed to convert spath [{}] to url", path);
        let mut lock = unpack_unit_e!(self.lsp.try_write().ok(), "failed acquiring lock",);
        lock.text_document_did_close(url);
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
        action: CompletionAction::Insert(
            i.text_edit
                .map(|c| match c {
                    CompletionTextEdit::Edit(e) => e.new_text,
                    CompletionTextEdit::InsertAndReplace(e) => e.new_text,
                })
                .unwrap_or("".to_string()),
        ),
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
