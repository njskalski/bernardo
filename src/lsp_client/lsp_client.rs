use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;

use log::{debug, error, warn};
use lsp_types::Url;
use serde_json::Value;
use stream_httparse::streaming_parser::RespParser;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::io::BufReader;
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_notification::LspServerNotification;
use crate::lsp_client::lsp_read::read_lsp;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_write::{internal_send_notification, internal_send_request};
use crate::lsp_client::lsp_write_error::LspWriteError;
use crate::tsw::lang_id::LangId;

const DEFAULT_RESPONSE_PREALLOCATION_SIZE: usize = 4192;

/*
Number of LanguageServerProtocol messages we don't know what to with do kept in memory.
 */
const DEFAULT_MAX_UNPROCESSED_MSGS: usize = 24;

// I use ID == String, because i32 might be small, and i64 is safe, so I send i64 as string and so I store it.
// LSP defines id integer as i32, while jsonrpc_core as u64.
pub struct CallInfo {
    pub method: &'static str,
    pub sender: tokio::sync::oneshot::Sender<serde_json::Value>,
}

pub type IdToCallInfo = HashMap<String, CallInfo>;

/*
Represents a single LSP server connection
 */
pub struct LspWrapper {
    server_path: PathBuf,
    workspace_root_path: PathBuf,
    language: LangId,
    child: tokio::process::Child,
    ids: Arc<RwLock<IdToCallInfo>>,

    curr_id: u64,
    reader_handle: tokio::task::JoinHandle<Result<(), LspReadError>>,
    logger_handle: tokio::task::JoinHandle<Result<(), ()>>,
}

impl LspWrapper {
    /*
    This spawns a reader thread that awaits server's stdout/stderr and pipes messages.

     */
    pub fn todo_new(workspace_root: PathBuf) -> Option<LspWrapper> {
        debug!("starting LspWrapper for directory {:?}", &workspace_root);
        // TODO unwrap
        let path = PathBuf::from_str("/home/andrzej/.local/bin/rust-analyzer").unwrap();
        // let path = PathBuf::from_str("/usr/bin/clangd").unwrap();
        let mut child = tokio::process::Command::new(path.as_os_str())
            // .args(&["--cli"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .ok()?;

        let mut stdout = match child.stdout.take() {
            None => {
                error!("failed acquiring stdout");
                return None;
            }
            Some(o) => tokio::io::BufReader::new(o)
        };

        let stderr = match child.stderr.take() {
            None => {
                error!("failed acquiring stderr");
                return None;
            }
            Some(e) => tokio::io::BufReader::new(e)
        };

        let (notification_sender, notification_receiver) = tokio::sync::mpsc::unbounded_channel::<LspServerNotification>();
        let ids = Arc::new(RwLock::new(IdToCallInfo::default()));

        let reader_handle: tokio::task::JoinHandle<Result<(), LspReadError>> = tokio::spawn(Self::reader_thread(
            ids.clone(),
            notification_sender,
            stdout,
        ));

        let logger_handle: tokio::task::JoinHandle<Result<(), ()>> = tokio::spawn(Self::logger_thread(
            stderr,
            notification_receiver,
        ));

        Some(
            LspWrapper {
                server_path: path,
                workspace_root_path: workspace_root,
                language: LangId::RUST,
                child,
                ids,
                curr_id: 1,
                reader_handle,
                logger_handle,
            }
        )
    }

    async fn send_message<R: lsp_types::request::Request>(&mut self, params: R::Params) -> Result<R::Result, LspIOError> {
        let new_id = format!("{}", self.curr_id);
        self.curr_id += 1;

        let (sender, receiver) = tokio::sync::oneshot::channel::<Value>();

        if self.ids.write().await.insert(new_id.clone(), CallInfo {
            method: R::METHOD,
            sender,
        }).is_some() {
            // TODO this is a reuse of id, super unlikely
            warn!("id reuse, not handled properly");
        }

        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_request::<R, _>(stdin, new_id.clone(), params).await?;

            match receiver.await {
                Err(_) => {
                    error!("failed retrieving message for id {}", &new_id);
                    Err(LspIOError::Read(LspReadError::BrokenChannel))
                }
                Ok(resp) => {
                    serde_json::from_value::<R::Result>(resp).map_err(|e| {
                        LspIOError::Read(LspReadError::DeError(e))
                    })
                }
            }
        } else {
            Err(LspIOError::Write(LspWriteError::BrokenPipe.into()))
        }
    }

    async fn send_notification<N: lsp_types::notification::Notification>(&mut self, params: N::Params) -> Result<(), LspWriteError> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_notification::<N, _>(stdin, params).await?;
            Ok(())
        } else {
            Err(LspWriteError::BrokenPipe.into())
        }
    }

    pub async fn initialize(&mut self) -> Result<lsp_types::InitializeResult, LspIOError> {
        let pid = std::process::id();

        let abs_path = self.workspace_root_path.to_str().unwrap(); // TODO should be absolute //TODO unwrap

        let root_url = Url::parse(&format!("file:///{}", abs_path)).unwrap(); //TODO unwrap
        let root_uri = Some(root_url.clone());

        let trace = if cfg!(debug_assertions) {
            lsp_types::TraceValue::Verbose
        } else {
            lsp_types::TraceValue::Messages
        };

        let workspace = lsp_types::WorkspaceFolder {
            uri: root_url,
            name: "".to_string(),
        };

        // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        self.send_message::<lsp_types::request::Initialize>(lsp_types::InitializeParams {
            process_id: Some(pid),
            // process_id: None,
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities: lsp_types::ClientCapabilities {
                workspace: None,
                text_document: Some(lsp_types::TextDocumentClientCapabilities {
                    synchronization: Some(lsp_types::TextDocumentSyncClientCapabilities {
                        dynamic_registration: Some(true),
                        will_save: Some(true),
                        will_save_wait_until: None, // TODO?
                        did_save: Some(true),
                    }),
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
            trace: Some(trace),
            workspace_folders: Some(vec![workspace]),
            client_info: None,
            // I specifically refuse to support any locale other than US English. Not sorry.
            locale: None,
        }).await
    }

    pub async fn send_did_open_text_document_notification(&mut self, url: Url) -> Result<(), LspWriteError> {
        self.send_notification::<lsp_types::notification::DidOpenTextDocument>(
            lsp_types::DidOpenTextDocumentParams {
                text_document: lsp_types::TextDocumentItem {
                    uri: url,
                    language_id: self.language.to_lsp_lang_id_string().to_owned(),
                    version: 0,
                    text: "".to_string(),
                }
            }
        ).await
    }

    pub fn wait(&self) -> &tokio::task::JoinHandle<Result<(), LspReadError>> {
        &self.reader_handle
    }

    pub async fn reader_thread(
        id_to_name: Arc<RwLock<IdToCallInfo>>,
        notification_sender: UnboundedSender<LspServerNotification>,
        mut stdout: BufReader<ChildStdout>,
    ) -> Result<(), LspReadError> {
        let mut resp_parser = RespParser::new_capacity(DEFAULT_RESPONSE_PREALLOCATION_SIZE);
        loop {
            match read_lsp(&mut stdout, &mut resp_parser, &id_to_name, &notification_sender).await {
                Ok(_) => {}
                Err(e) => {
                    debug!("terminating lsp_reader thread because {:?}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /*
    This thread just dries the pipes of whatever we have no good receiver for: so stderr and
    LspNotifications that I do not handle in any specific way yet.
     */
    pub async fn logger_thread(
        mut stderr_pipe: BufReader<ChildStderr>,
        mut notification_receiver: UnboundedReceiver<LspServerNotification>,
    ) -> Result<(), ()> {
        //TODO dry the other channel too before quitting

        let mut stderr_lines = stderr_pipe.lines();

        let mut more_lines = true;
        let mut notification_channel_open = true;
        loop {
            tokio::select! {
                line_res = stderr_lines.next_line(), if more_lines => {
                    match line_res {
                        Ok(line_op) => {
                            if let Some(line) = line_op {
                                error!("Lsp: {}", line);
                            } else {
                                debug!("no more lines in LSP stderr_pipe.");
                                more_lines = false;
                            }
                        }
                        Err(e) => {
                            error!("stderr_pipe read error: {}", e);
                        }
                    }
                },
                notification_op = notification_receiver.recv(), if notification_channel_open => {
                    match notification_op {
                        Some(notification) => {
                            debug!("received LSP notification:\n---\n{:?}\n---\n", notification);
                        }
                        None => {
                            debug!("notification channel closed.");
                            notification_channel_open = false;
                        }
                    }
                },
                else => { break; },
            }
        }

        debug!("closing logger thread");

        Ok(())
    }
}
