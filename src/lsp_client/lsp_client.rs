use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::format;
use std::future::Future;
use std::io;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use crossbeam_channel::TryRecvError;
use jsonrpc_core::{Error, Id, MethodCall, Output};
use jsonrpc_core_client::RawClient;
use jsonrpc_core_client::transports::local::connect;
use log::{debug, error, warn};
use lsp_types::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use syntect::html::IncludeBackground::No;
use tokio::sync::oneshot::error::RecvError;
use crate::ConfigRef;
use crate::lsp_client::lsp_io_error::LspIOError;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_response::LspResponse;
use crate::lsp_client::lsp_value_to_typed_response;
use crate::lsp_client::lsp_write_error::LspWriteError;
use crate::tsw::lang_id::LangId;
use tokio::io::AsyncWriteExt;
use tokio::process::{ChildStderr, ChildStdout};
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;

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

}

impl LspWrapper {
    pub fn todo_new(workspace_root: PathBuf) -> Option<LspWrapper> {
        // TODO unwrap
        let path = PathBuf::from_str("/home/andrzej/.cargo/bin/rls").unwrap();
        let mut child = tokio::process::Command::new(path.as_os_str())
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

        tokio::spawn(async move {
            loop {
                match read_lsp(&mut stdout, &id_to_name).await {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("terminating lsp_reader thread because {:?}", e);
                        break;
                    }
                }
            }
        });

        Some(
            LspWrapper {
                server_path: path,
                workspace_root_path: workspace_root,
                language: LangId::RUST,
                child,
                ids,
                curr_id: 1,
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

    pub async fn initialize(&mut self) -> Result<lsp_types::InitializeResult, LspIOError> {
        let pid = std::process::id();

        let abs_path = self.workspace_root_path.to_str().unwrap(); // TODO should be absolute //TODO unwrap

        let root_uri = Some(Url::parse(&format!("file:///{}", abs_path)).unwrap()); //TODO unwrap

        let trace = if cfg!(debug_assertions) {
            lsp_types::TraceValue::Verbose
        } else {
            lsp_types::TraceValue::Messages
        };

        self.send_message::<lsp_types::request::Initialize>(lsp_types::InitializeParams {
            process_id: Some(pid),
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities: lsp_types::ClientCapabilities {
                workspace: None,
                text_document: None,
                window: None,
                general: None,
                experimental: None,
            },
            trace: Some(trace),
            workspace_folders: None,
            client_info: None,
            // I specifically refuse to support any locale other than US English. Not sorry.
            locale: None,
        }).await
    }
}

async fn internal_send_request<R: lsp_types::request::Request, W: tokio::io::AsyncWrite>(
    stdin: &mut W,
    id: String,
    params: R::Params,
) -> Result<(), LspWriteError>
    where
        R::Params: serde::Serialize,
        W: std::marker::Unpin
{
    if let serde_json::value::Value::Object(params) = serde_json::to_value(params)? {
        let req = jsonrpc_core::Call::MethodCall(jsonrpc_core::MethodCall {
            jsonrpc: Some(jsonrpc_core::Version::V2),
            method: R::METHOD.to_string(),
            params: jsonrpc_core::Params::Map(params),
            id: jsonrpc_core::Id::Str(id),
        });
        let request = serde_json::to_string(&req)?;
        let mut buffer: Vec<u8> = Vec::new();
        write!(
            &mut buffer,
            "Content-Length: {}\r\n\r\n{}",
            request.len(),
            request
        )?;

        let len = stdin.write(&buffer).await?;
        if buffer.len() == len {
            stdin.flush().await?;
            Ok(())
        } else {
            Err(LspWriteError::InterruptedWrite)
        }
    } else {
        Err(LspWriteError::WrongValueType)
    }
}


#[derive(Debug)]
pub struct LspResponseTuple {
    pub id: String,
    pub method: &'static str,
    pub params: jsonrpc_core::Value,
}

// TODO one can reduce allocation here
fn id_to_str(id: Id) -> String {
    match id {
        Id::Null => "".to_string(),
        Id::Num(u) => format!("{}", u),
        /// String id
        Id::Str(s) => s,
    }
}

async fn read_lsp<R: tokio::io::AsyncBufRead + std::marker::Unpin>(input: &mut R, id_to_method: &Arc<RwLock<IdToCallInfo>>) -> Result<(), LspReadError> {
    if let Some(line) = input.lines().next_line().await? {
        if !line.starts_with("Content-Length") {
            return Err(LspReadError::UnexpectedContents);
        }

        if let Some(idx) = line.find(":") {
            if let Ok(content_length) = &line[idx + 1..].trim().parse::<usize>() {
                // skipping "\r\n" between Content-Length and body

                debug!("cl2: {}", content_length);
                if let Some(empty_line) = input.lines().next_line().await? {
                    if !empty_line.is_empty() {
                        debug!("expected empty, got [{}]", &empty_line);
                        return Err(LspReadError::UnexpectedContents);
                    }
                } else {
                    return Err(LspReadError::NoLine);
                }

                // reading content_length
                let mut body: Vec<u8> = Vec::with_capacity(*content_length);
                input.read_exact(body.borrow_mut()).await?;

                let resp = serde_json::from_slice::<jsonrpc_core::Response>(&body)?;

                if let jsonrpc_core::Response::Single(single) = resp {
                    match single {
                        Output::Failure(fail) => Err(LspReadError::LspFailure(fail.error)),
                        Output::Success(succ) => {
                            let id = id_to_str(succ.id);
                            if let Some(call_info) = id_to_method.write().await.remove(&id) {
                                match call_info.sender.send(succ.result) {
                                    Ok(_) => Ok(()),
                                    Err(_) => Err(LspReadError::BrokenChannel)
                                }
                            } else {
                                Err(LspReadError::UnknownIdentifier)
                            }
                        }
                    }
                } else {
                    Err(LspReadError::UnexpectedContents)
                }
            } else {
                Err(LspReadError::UnexpectedContents)
            }
        } else {
            error!("unexpected line contents: [{}]", line);
            Err(LspReadError::UnexpectedContents)
        }
    } else {
        Err(LspReadError::NoLine)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_keycode_to_string() {}
}