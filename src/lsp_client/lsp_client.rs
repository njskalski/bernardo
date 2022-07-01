use std::borrow::BorrowMut;
use std::collections::{HashMap, VecDeque};
use std::fmt::format;
use std::future::Future;
use std::{future, io};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::JoinHandle;
use crossbeam_channel::TryRecvError;
use jsonrpc_core::{Error, Id, MethodCall, Output};
use jsonrpc_core_client::RawClient;
use jsonrpc_core_client::transports::local::connect;
use log::{debug, error, warn};
use lsp_types::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use stream_httparse::streaming_parser::{ParseResult, RespParser};
use syntect::html::IncludeBackground::No;
use tokio::sync::oneshot::error::RecvError;
use crate::ConfigRef;
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_response::LspResponse;
use crate::lsp_client::lsp_value_to_typed_response;
use crate::lsp_client::lsp_write_error::LspWriteError;
use crate::tsw::lang_id::LangId;
use tokio::io::{AsyncBufRead, AsyncWriteExt};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;

const DEFAULT_RESPONSE_PREALLOCATION_SIZE: usize = 4192;
const FAKE_RESPONSE_PREFIX: &'static str = "HTTP/1.1 200 OK\r\n";
/*
Number of LanguageServerProtocol messages we don't know what to with do kept in memory.
 */
const DEFAULT_MAX_UNPROCESSED_MSGS: usize = 24;

// I use ID == String, because i32 might be small, and i64 is safe, so I send i64 as string and so I store it.
// LSP defines id integer as i32, while jsonrpc_core as u64.
struct CallInfo {
    method: &'static str,
    sender: tokio::sync::oneshot::Sender<serde_json::Value>,
}

type IdToCallInfo = HashMap<String, CallInfo>;


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
}

impl LspWrapper {
    pub fn todo_new(workspace_root: PathBuf) -> Option<LspWrapper> {
        // TODO unwrap
        // let path = PathBuf::from_str("/home/andrzej/.cargo/bin/rls").unwrap();
        let path = PathBuf::from_str("/usr/bin/clangd").unwrap();
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

        // let stderr = match child.stderr.take() {
        //     None => {
        //         error!("failed acquiring stderr");
        //         return None;
        //     }
        //     Some(e) => tokio::io::BufReader::new(e)
        // };
        let ids = Arc::new(RwLock::new(IdToCallInfo::default()));

        let id_to_name = ids.clone();

        let handle: tokio::task::JoinHandle<Result<(), LspReadError>> = tokio::spawn(async move {
            let mut resp_parser = RespParser::new_capacity(DEFAULT_RESPONSE_PREALLOCATION_SIZE);
            loop {
                match read_lsp(&mut stdout, &mut resp_parser, &id_to_name).await {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("terminating lsp_reader thread because {:?}", e);
                        return Err(e);
                    }
                }
            }
            Ok(())
        });

        Some(
            LspWrapper {
                server_path: path,
                workspace_root_path: workspace_root,
                language: LangId::RUST
                child,
                ids,
                curr_id: 1,
                reader_handle: handle,
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

    async fn send_notification<N: lsp_types::notification::Notification>(&mut self, params: N::Params) -> Result<(), LspIOError> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_notification::<N, _>(stdin, params).await?
        } else {
            Err(LspIOError::Write(LspWriteError::BrokenPipe.into()))
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
            // process_id: Some(pid),
            process_id: None,
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

    pub async fn send_did_open_text_document_notification(&mut self, url: Url) {
        self.send_notification < lsp_types::notification::DidOpenTextDocument > (
            lsp_types::DidOpenTextDocumentParams {
                text_document: lsp_types::TextDocumentItem {
                    uri: url,
                    language_id: self.language.to_lsp_lang_id_string().clone(),
                    version: 0,
                    text: "".to_string(),
                }
            }
        )
    }

    pub fn wait(&self) -> &tokio::task::JoinHandle<Result<(), LspReadError>> {
        &self.reader_handle
    }
}
