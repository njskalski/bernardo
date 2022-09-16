use std::{process, thread};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{ChildStderr, ChildStdout, Stdio};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, select, Sender};
use log::{debug, error, warn};
use lsp_types::{CompletionContext, CompletionResponse, CompletionTriggerKind, Position, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentPositionParams, Url, VersionedTextDocumentIdentifier};
use lsp_types::request::Completion;
use serde_json::Value;

use crate::lsp_client::debug_helpers::lsp_debug_save;
use crate::lsp_client::helpers::LspTextCursor;
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_notification::LspServerNotification;
use crate::lsp_client::lsp_read::read_lsp;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_write::{internal_send_notification, internal_send_notification_no_params, internal_send_request};
use crate::lsp_client::lsp_write_error::LspWriteError;
use crate::lsp_client::promise::{LSPPromise, Promise};
use crate::tsw::lang_id::LangId;
use crate::w7e::navcomp_group::{NavCompTick, NavCompTickSender};

const DEFAULT_RESPONSE_PREALLOCATION_SIZE: usize = 4192;

/*
Number of LanguageServerProtocol messages we don't know what to with do kept in memory.
 */
const DEFAULT_MAX_UNPROCESSED_MSGS: usize = 24;

// I use ID == String, because i32 might be small, and i64 is safe, so I send i64 as string and so I store it.
// LSP defines id integer as i32, while jsonrpc_core as u64.
pub struct CallInfo {
    pub method: &'static str,
    pub sender: Sender<jsonrpc_core::Value>,
}

pub type IdToCallInfo = HashMap<String, CallInfo>;

/*
Represents a single LSP server connection
 */
pub struct LspWrapper {
    server_path: PathBuf,
    workspace_root_path: PathBuf,
    language: LangId,
    child: process::Child,

    //TODO the common state should probably be merged to avoid concurrency issues. It's not like I
    // will be sending multiple edit events concurrently.
    ids: Arc<RwLock<IdToCallInfo>>,
    file_versions: Arc<RwLock<HashMap<Url, i32>>>,

    curr_id: u64,
    reader_handle: JoinHandle<Result<(), LspReadError>>,
    logger_handle: JoinHandle<Result<(), ()>>,

    /*
    to be copied to all features, indicate "ready"
     */
    tick_sender: NavCompTickSender,
}

// pub type LspWrapperRef = Arc<RwLock<LspWrapper>>;

impl LspWrapper {
    /*
    This spawns a reader thread that awaits server's stdout/stderr and pipes messages.

     */
    pub fn new(lsp_path: PathBuf,
               workspace_root: PathBuf,
               tick_sender: NavCompTickSender) -> Option<LspWrapper> {
        debug!("starting LspWrapper for directory {:?}", &workspace_root);
        let mut child = process::Command::new(lsp_path.as_os_str())
            // .args(&["--cli"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .ok()?;

        let stdout = match child.stdout.take() {
            None => {
                error!("failed acquiring stdout");
                return None;
            }
            Some(o) => BufReader::new(o)
        };

        let stderr = match child.stderr.take() {
            None => {
                error!("failed acquiring stderr");
                return None;
            }
            Some(e) => BufReader::new(e)
        };

        let (notification_sender, notification_receiver) = crossbeam_channel::unbounded::<LspServerNotification>();
        let ids = Arc::new(RwLock::new(IdToCallInfo::default()));
        let file_versions = Arc::new(RwLock::new(HashMap::default()));

        let reader_identifier: String = format!("{}-{}", lsp_path.file_name().map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                error!("failed to unwrap filename");
                "noname".to_string()
            }), process::id());

        let reader_identifier2 = reader_identifier.clone();

        let ids_clone = ids.clone();
        let reader_handle: JoinHandle<Result<(), LspReadError>> = thread::spawn(move || {
            Self::reader_thread(
                reader_identifier,
                ids_clone,
                notification_sender,
                stdout,
            )
        });

        let logger_handle: JoinHandle<Result<(), ()>> = thread::spawn(|| {
            Self::logger_thread(
                reader_identifier2,
                stderr,
                notification_receiver,
            )
        });

        Some(
            LspWrapper {
                server_path: lsp_path,
                workspace_root_path: workspace_root,
                language: LangId::RUST,
                child,
                ids,
                file_versions,
                curr_id: 1,
                reader_handle,
                logger_handle,
                tick_sender,
            }
        )
    }

    fn send_message<R: lsp_types::request::Request>(&mut self, params: R::Params) -> Result<LSPPromise<R>, LspIOError> where <R as lsp_types::request::Request>::Result: std::marker::Send {
        let new_id = format!("{}", self.curr_id);
        self.curr_id += 1;

        let (sender, receiver) = crossbeam_channel::bounded::<jsonrpc_core::Value>(1);

        if self.ids.write().map_err(|poison_error| {
            LspWriteError::LockError(poison_error.to_string())
        })?.insert(new_id.clone(), CallInfo {
            method: R::METHOD,
            sender: sender,
        }).is_some() {
            // TODO this is a reuse of id, super unlikely
            warn!("id reuse, not handled properly");
        }

        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_request::<R, _>(stdin, new_id.clone(), params)?;
            Ok(LSPPromise::<R>::new(receiver))
        } else {
            Err(LspIOError::Write(LspWriteError::BrokenPipe.into()))
        }
    }

    fn send_notification_no_params<N: lsp_types::notification::Notification>(&mut self) -> Result<(), LspWriteError> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_notification_no_params::<N, _>(stdin)?;
            Ok(())
        } else {
            Err(LspWriteError::BrokenPipe.into())
        }
    }

    fn send_notification<N: lsp_types::notification::Notification>(&mut self, params: N::Params) -> Result<(), LspWriteError> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_notification::<N, _>(stdin, params)?;
            Ok(())
        } else {
            Err(LspWriteError::BrokenPipe.into())
        }
    }

    pub fn initialize(&mut self) -> Result<lsp_types::InitializeResult, LspIOError> {
        let pid = std::process::id();

        let abs_path = self.workspace_root_path.to_str().unwrap(); // TODO should be absolute //TODO unwrap

        let root_url = Url::parse(&format!("file:///{}", abs_path)).unwrap(); //TODO unwrap
        let root_uri = Some(root_url.clone());

        let _trace = if cfg!(debug_assertions) {
            lsp_types::TraceValue::Verbose
        } else {
            lsp_types::TraceValue::Messages
        };

        let workspace = lsp_types::WorkspaceFolder {
            uri: root_url,
            name: "".to_string(),
        };

        let mut result = self.send_message::<lsp_types::request::Initialize>(lsp_types::InitializeParams {
            process_id: Some(pid),
            // process_id: None,
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities: lsp_types::ClientCapabilities {
                workspace: Some(lsp_types::WorkspaceClientCapabilities {
                    apply_edit: None,
                    workspace_edit: None,
                    did_change_configuration: None,
                    did_change_watched_files: None,
                    symbol: None,
                    execute_command: None,
                    workspace_folders: None,
                    configuration: None,
                    semantic_tokens: None,
                    code_lens: None,
                    file_operations: None,
                }),
                text_document: Some(lsp_types::TextDocumentClientCapabilities {
                    synchronization: None,
                    completion: None,
                    hover: None,
                    signature_help: None,
                    references: None,
                    document_highlight: None,
                    document_symbol: None,
                    formatting: None,
                    range_formatting: None,
                    on_type_formatting: None,
                    declaration: None,
                    definition: None,
                    type_definition: None,
                    implementation: None,
                    code_action: None,
                    code_lens: None,
                    document_link: None,
                    color_provider: None,
                    rename: None,
                    publish_diagnostics: None,
                    folding_range: None,
                    selection_range: None,
                    linked_editing_range: None,
                    call_hierarchy: None,
                    semantic_tokens: None,
                    moniker: None,
                }),
                window: None,
                general: None,
                experimental: None,
            },
            trace: None,
            workspace_folders: Some(vec![workspace]),
            client_info: None,
            // I specifically refuse to support any locale other than US English. Not sorry.
            locale: None,
        })?;

        //before returning I will send syn-ack as protocol demands.
        if result.wait() {
            self.send_notification_no_params::<lsp_types::notification::Initialized>()?;
            Ok(result.take().unwrap())
        } else {
            // OK maybe I should put errors in promises?
            Err(LspIOError::Write(LspWriteError::IoError("failed init".to_string())))
        }
    }

    pub fn text_document_did_open(&mut self, url: Url, text: String) -> Result<(), LspWriteError> {
        {
            let mut lock = self.file_versions.write()?;
            if let Some(old_id) = lock.get(&url) {
                warn!("expected document {:?} version to be 0, is {}", &url, old_id);
            } else {
                lock.insert(url.clone(), 0);
            }
        }

        self.send_notification::<lsp_types::notification::DidOpenTextDocument>(
            lsp_types::DidOpenTextDocumentParams {
                text_document: lsp_types::TextDocumentItem {
                    uri: url,
                    language_id: self.language.to_lsp_lang_id_string().to_owned(),
                    version: 1,
                    text,
                }
            }
        )
    }

    /*
    This is a non-incremental variant of text_document_did_change
     */
    pub fn text_document_did_change(&mut self, url: Url, full_text: String) -> Result<(), LspWriteError> {
        let version = {
            let mut lock = self.file_versions.write()?;
            if let Some(old_id) = lock.get(&url).map(|i| *i) {
                // debug!("updating document {} from {} to {}", &url, old_id, old_id+1);
                lock.insert(url.clone(), old_id + 1);
                old_id + 1
            } else {
                error!("failed to find document version for {:?} - was document opened?", &url);
                // TODO add error for "no document version found"
                return Ok(());
            }
        };

        self.send_notification::<lsp_types::notification::DidChangeTextDocument>(
            lsp_types::DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier { uri: url, version },
                content_changes: vec![
                    TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: full_text,
                    }
                ],
            }
        )
    }

    pub fn text_document_completion(&mut self,
                                    url: Url,
                                    cursor: LspTextCursor,
                                    /*
                                    just typing or ctrl-space?
                                     */
                                    automatic: bool,
                                    /*
                                    '.' or '::' or other thing like that
                                     */
                                    trigger_character: Option<String>,
    ) -> Result<LSPPromise<Completion>, LspIOError> {
        self.send_message::<lsp_types::request::Completion>(
            lsp_types::CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: url },
                    position: Position {
                        line: cursor.row,
                        character: cursor.col,
                    },
                },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
                context: Some(CompletionContext {
                    trigger_kind: if automatic {
                        CompletionTriggerKind::TRIGGER_CHARACTER
                    } else {
                        CompletionTriggerKind::INVOKED
                    },
                    trigger_character,
                }),
            }
        )
    }

    pub fn wait(&self) -> &JoinHandle<Result<(), LspReadError>> {
        &self.reader_handle
    }

    pub fn reader_thread(
        // used for debugging
        identifier: String,
        id_to_name: Arc<RwLock<IdToCallInfo>>,
        notification_sender: Sender<LspServerNotification>,
        mut stdout: BufReader<ChildStdout>,
    ) -> Result<(), LspReadError> {
        let mut num: usize = 0;
        loop {
            num += 1;
            match read_lsp(
                &identifier,
                &mut num,
                &mut stdout,
                &id_to_name,
                &notification_sender,
            ) {
                Ok(_) => {}
                Err(LspReadError::UnmatchedId { id, method }) => {
                    warn!("unmatched id {} (method {})", id, method);
                }
                Err(e) => {
                    debug!("terminating lsp_reader thread because {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    /*
    This thread just dries the pipes of whatever we have no good receiver for: so stderr and
    LspNotifications that I do not handle in any specific way yet.
     */
    pub fn logger_thread(
        identifier: String,
        stderr_pipe: BufReader<ChildStderr>,
        mut notification_receiver: Receiver<LspServerNotification>,
    ) -> Result<(), ()> {
        //TODO dry the other channel too before quitting

        let mut stderr_lines = stderr_pipe.lines();
        let stderr_path = PathBuf::from(identifier).join("stderr.txt");

        let mut more_lines = true;
        let mut notification_channel_open = true;
        // loop {
        //     select! {
        //         stderr_lines.next_line() -> line_res => {
        //             match line_res {
        //                 Ok(line_op) => {
        //                     if let Some(line) = line_op {
        //                         //error!("Lspx{:?}: {}", &stderr_path, &line);
        //                         lsp_debug_save(stderr_path.clone(), format!("{}\n", line)).await;
        //                     } else {
        //                         warn!("no more lines in LSP stderr_pipe.");
        //                         more_lines = false;
        //                     }
        //                 }
        //                 Err(e) => {
        //                     error!("stderr_pipe read error: {}", e);
        //                 }
        //             }
        //         },
        //         notification_op = notification_receiver.recv(), if notification_channel_open => {
        //             match notification_op {
        //                 Some(_notification) => {
        //                     //debug!("received LSP notification:\n---\n{:?}\n---\n", notification);
        //                     // debug!("received LSP notification");
        //                 }
        //                 None => {
        //                     debug!("notification channel closed.");
        //                     notification_channel_open = false;
        //                 }
        //             }
        //         },
        //         else => { break; },
        //     }
        // }

        debug!("closing logger thread");

        Ok(())
    }
}

impl Debug for LspWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LspWrapper({:?})", &self.server_path)
    }
}