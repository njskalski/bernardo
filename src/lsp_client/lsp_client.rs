use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{ChildStderr, ChildStdout, Stdio};
use std::sync::{Arc, LockResult, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{process, thread};

use crossbeam_channel::{Receiver, Sender};
use log::{debug, error, warn};
use lsp_types::{self, PublishDiagnosticsParams};
use url::Url;

use crate::lsp_client::debug_helpers::lsp_debug_save;
use crate::lsp_client::diagnostic;
use crate::lsp_client::diagnostic::{Diagnostic, DiagnosticSeverity};
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_notification::LspServerNotification;
use crate::lsp_client::lsp_read::read_lsp;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_write::{internal_send_notification, internal_send_notification_no_params, internal_send_request};
use crate::lsp_client::lsp_write_error::LspWriteError;
use crate::lsp_client::promise::LSPPromise;
use crate::primitives::stupid_cursor::StupidCursor;
use crate::promise::promise::{Promise, PromiseState};
use crate::tsw::lang_id::LangId;
use crate::unpack_or;
use crate::w7e::navcomp_group::{NavCompTick, NavCompTickSender};
use crate::widgets::editor_widget::label::label::{Label, LabelPos, LabelStyle};

// I use ID == String, because i32 might be small, and i64 is safe, so I send i64 as string and so I
// store it. LSP defines id integer as i32, while jsonrpc_core as u64.
pub struct CallInfo {
    pub method: &'static str,
    pub sender: Sender<jsonrpc_core::Value>,
}

pub type IdToCallInfo = HashMap<String, CallInfo>;

pub struct LSPFileDescriptor {
    // this is the ordering version used to distinguish between "which version of file this message
    // relates to". In general we don't care about messages coming for outdated versions.
    pub version: i32,

    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics
    // has a nice guarantee:
    // "Newly pushed diagnostics always replace previously pushed diagnostics. There is no merging that happens on the client side."
    // Nicely done Microsoft.
    pub diagnostics: Vec<Diagnostic>,
}

pub type FilesStore = Arc<RwLock<HashMap<Url, LSPFileDescriptor>>>;

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
    files: FilesStore,

    curr_id: u64,
    reader_handle: JoinHandle<Result<(), LspReadError>>,
    logger_handle: JoinHandle<Result<(), ()>>,

    error_sink: Sender<LspReadError>,
}

impl LspWrapper {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);

    /*
    This spawns a reader thread that awaits server's stdout/stderr and pipes messages.

     */
    // TODO make result
    pub fn new(
        lsp_path: PathBuf,
        workspace_root: PathBuf,
        language: LangId,
        tick_sender: NavCompTickSender,
        error_sink: Sender<LspReadError>,
    ) -> Option<LspWrapper> {
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
            Some(o) => BufReader::new(o),
        };

        let stderr = match child.stderr.take() {
            None => {
                error!("failed acquiring stderr");
                return None;
            }
            Some(e) => BufReader::new(e),
        };

        let ids = Arc::new(RwLock::new(IdToCallInfo::default()));
        let files = Arc::new(RwLock::new(HashMap::default()));

        let reader_identifier: String = format!(
            "{}-{}",
            lsp_path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| {
                error!("failed to unwrap filename");
                "noname".to_string()
            }),
            process::id()
        );

        let reader_identifier2 = reader_identifier.clone();

        let ids_clone = ids.clone();
        let files_clone = files.clone();
        let reader_handle: JoinHandle<Result<(), LspReadError>> =
            thread::spawn(move || Self::lsp_reader_thread_main(reader_identifier, ids_clone, files_clone, stdout, tick_sender));

        let logger_handle: JoinHandle<Result<(), ()>> = thread::spawn(|| Self::logger_thread(reader_identifier2, stderr));

        Some(LspWrapper {
            server_path: lsp_path,
            workspace_root_path: workspace_root,
            language,
            child,
            ids,
            files,
            curr_id: 1,
            reader_handle,
            logger_handle,
            error_sink,
        })
    }

    fn get_formatting_options(&self) -> lsp_types::FormattingOptions {
        lsp_types::FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            properties: Default::default(),
            trim_trailing_whitespace: Some(true),
            insert_final_newline: Some(true),
            trim_final_newlines: Some(true),
        }
    }

    fn get_position_params(url: Url, cursor: StupidCursor) -> lsp_types::TextDocumentPositionParams {
        lsp_types::TextDocumentPositionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: url },
            position: lsp_types::Position {
                line: cursor.line_0b,
                character: cursor.char_idx_0b,
            },
        }
    }

    fn send_message<R: lsp_types::request::Request>(&mut self, params: R::Params) -> Result<LSPPromise<R>, LspWriteError>
    where
        <R as lsp_types::request::Request>::Result: std::marker::Send,
    {
        let new_id = format!("{}", self.curr_id);
        self.curr_id += 1;

        let (sender, receiver) = crossbeam_channel::bounded::<jsonrpc_core::Value>(1);

        if self
            .ids
            .write()
            .map_err(|poison_error| LspWriteError::LockError(poison_error.to_string()))?
            .insert(new_id.clone(), CallInfo { method: R::METHOD, sender })
            .is_some()
        {
            // TODO this is a reuse of id, super unlikely
            warn!("id reuse, not handled properly");
        }

        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_request::<R, _>(stdin, new_id.clone(), params)?;
            Ok(LSPPromise::<R>::new(receiver, self.error_sink.clone()))
        } else {
            Err(LspWriteError::BrokenPipe)
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

    #[allow(deprecated)]
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

                    inline_value: None,
                    inlay_hint: None,
                    diagnostic: None,
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

                    type_hierarchy: None,
                    inline_value: None,
                    inlay_hint: None,
                    diagnostic: None,
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
            work_done_progress_params: Default::default(),
        })?;

        //before returning I will send syn-ack as protocol demands.
        if result.wait(Some(Self::DEFAULT_TIMEOUT)) == PromiseState::Ready {
            self.send_notification_no_params::<lsp_types::notification::Initialized>()?;
            Ok(result.take().unwrap())
        } else {
            // OK maybe I should put errors in promises?
            Err(LspIOError::Write(LspWriteError::IoError("failed init".to_string())))
        }
    }

    pub fn text_document_did_open(&mut self, url: Url, text: String) -> Result<(), LspWriteError> {
        {
            let mut lock = self.files.write()?;
            if let Some(fd) = lock.get(&url) {
                if fd.version > 0 {
                    warn!("expected document {:?} version to be 0, is {}", &url, fd.version);
                }
            } else {
                lock.insert(
                    url.clone(),
                    LSPFileDescriptor {
                        version: 0,
                        diagnostics: vec![],
                    },
                );
            }
        }

        self.send_notification::<lsp_types::notification::DidOpenTextDocument>(lsp_types::DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: url,
                language_id: self.language.to_lsp_lang_id_string().to_owned(),
                version: 1,
                text,
            },
        })
    }

    pub fn text_document_formatting(&mut self, url: Url) -> Result<LSPPromise<lsp_types::request::Formatting>, LspWriteError> {
        self.send_message::<lsp_types::request::Formatting>(lsp_types::DocumentFormattingParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: url },
            options: self.get_formatting_options(),
            work_done_progress_params: Default::default(),
        })
    }

    pub fn text_document_references(
        &mut self,
        url: Url,
        cursor: StupidCursor,
    ) -> Result<LSPPromise<lsp_types::request::References>, LspWriteError> {
        self.send_message::<lsp_types::request::References>(lsp_types::ReferenceParams {
            text_document_position: Self::get_position_params(url, cursor),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: lsp_types::ReferenceContext { include_declaration: true },
        })
    }

    pub fn text_document_goto_definition(
        &mut self,
        url: Url,
        cursor: StupidCursor,
    ) -> Result<LSPPromise<lsp_types::request::GotoDefinition>, LspWriteError> {
        self.send_message::<lsp_types::request::GotoDefinition>(lsp_types::GotoDefinitionParams {
            text_document_position_params: Self::get_position_params(url, cursor),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
    }

    /*
    This is a non-incremental variant of text_document_did_change
     */
    pub fn text_document_did_change(&mut self, url: Url, full_text: String) -> Result<(), LspWriteError> {
        let version = {
            let mut lock = self.files.write()?;
            if let Some(fd) = lock.get_mut(&url) {
                // debug!("updating document {} from {} to {}", &url, old_id, old_id+1);
                fd.version += 1;
                fd.version
            } else {
                error!("failed to find document version for {:?} - was document opened?", &url);
                // TODO add error for "no document version found"
                return Ok(());
            }
        };

        self.send_notification::<lsp_types::notification::DidChangeTextDocument>(lsp_types::DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier { uri: url, version },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: full_text,
            }],
        })
    }

    pub fn text_document_did_close(&mut self, url: Url) -> Result<(), LspWriteError> {
        self.send_notification::<lsp_types::notification::DidCloseTextDocument>(lsp_types::DidCloseTextDocumentParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: url },
        })
    }

    pub fn text_document_completion(
        &mut self,
        url: Url,
        cursor: StupidCursor,
        /*
        just typing or ctrl-space?
         */
        automatic: bool,
        /*
        '.' or '::' or other thing like that
         */
        trigger_character: Option<String>,
    ) -> Result<LSPPromise<lsp_types::request::Completion>, LspWriteError> {
        self.send_message::<lsp_types::request::Completion>(lsp_types::CompletionParams {
            text_document_position: Self::get_position_params(url, cursor),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: Some(lsp_types::CompletionContext {
                trigger_kind: if automatic {
                    lsp_types::CompletionTriggerKind::TRIGGER_CHARACTER
                } else {
                    lsp_types::CompletionTriggerKind::INVOKED
                },
                trigger_character,
            }),
        })
    }

    pub fn text_document_document_symbol(
        &mut self,
        url: Url,
    ) -> Result<LSPPromise<lsp_types::request::DocumentSymbolRequest>, LspWriteError> {
        self.send_message::<lsp_types::request::DocumentSymbolRequest>(lsp_types::DocumentSymbolParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: url },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
    }

    pub fn wait(&self) -> &JoinHandle<Result<(), LspReadError>> {
        &self.reader_handle
    }

    pub fn lsp_reader_thread_main(
        // used for debugging
        identifier: String,
        id_to_name: Arc<RwLock<IdToCallInfo>>,
        files: FilesStore,
        mut stdout: BufReader<ChildStdout>,
        tick_sender: Sender<NavCompTick>,
    ) -> Result<(), LspReadError> {
        let mut num: usize = 0;

        // so I can attach fkn clion
        // thread::sleep(time::Duration::from_secs(6));

        loop {
            num += 1;
            match read_lsp(&identifier, &mut num, &mut stdout, &id_to_name, &files) {
                Ok(_) => {
                    // TODO Pass LangId and whatever usize is?
                    match tick_sender.try_send(NavCompTick::LspTick(LangId::RUST, 0)) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("non-fatal: failed to send navcomp tick: {:?}", e);
                        }
                    }
                }
                Err(LspReadError::PromiseExpired { id }) => {
                    debug!("promise id {} expired", id);
                }
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
    This thread just dries the stderr
     */
    pub fn logger_thread(identifier: String, stderr_pipe: BufReader<ChildStderr>) -> Result<(), ()> {
        let mut stderr_lines = stderr_pipe.lines();
        let stderr_path = PathBuf::from(identifier).join("stderr.txt");

        loop {
            let line_res = match stderr_lines.next() {
                None => {
                    break;
                }
                Some(l) => l,
            };

            match line_res {
                Ok(line) => {
                    lsp_debug_save(stderr_path.clone(), format!("{}\n", line));
                }
                Err(e) => {
                    error!("stderr_pipe read error: {}", e);
                    return Err(());
                }
            }
        }

        debug!("closing logger thread");

        Ok(())
    }

    // Tells you "what is the current version of Labels for that file
    pub fn get_labels_file_version_id(&self, uri: &Url) -> Option<i32> {
        let lock = unpack_or!(self.files.read().ok(), None, "failed to lock LSP files store to read");
        let item = unpack_or!(lock.get(uri), None, "diagnostics of file {} not found", uri);

        Some(item.version)
    }

    // This should combine diagnostics (errors, warnings) AND types
    pub fn get_labels_for_file(&self, uri: &Url) -> Box<dyn Iterator<Item = Label>> {
        let lock = unpack_or!(
            self.files.read().ok(),
            Box::new(std::iter::empty()),
            "failed to lock LSP files store to read"
        );
        let item = unpack_or!(lock.get(uri), Box::new(std::iter::empty()), "diagnostics of file {} not found", uri);

        let labels: Vec<Label> = item
            .diagnostics
            .iter()
            .map(|d| {
                Label::new(
                    LabelPos::LineAfter { line_no_1b: d.line_no_1b },
                    match d.severity {
                        DiagnosticSeverity::Error => LabelStyle::Error,
                        DiagnosticSeverity::Warning => LabelStyle::Warning,
                        DiagnosticSeverity::Info => LabelStyle::TypeAnnotation,
                    },
                    Box::new(d.message.clone()),
                )
            })
            .collect();

        Box::new(labels.into_iter())
    }
}

impl Debug for LspWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LspWrapper({:?})", &self.server_path)
    }
}
