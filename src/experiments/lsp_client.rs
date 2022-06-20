use std::borrow::BorrowMut;
use std::io;
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use jsonrpc_core::{Error, MethodCall};
use log::error;
use lsp_types::lsp_request;
use lsp_types::request::{Initialize, Request};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::ConfigRef;
use crate::experiments::lsp_client::LspWriteError::BrokenPipe;
use crate::experiments::lsp_client::LspReadError::UnexpectedContents;
use crate::tsw::lang_id::LangId;

struct LspFinder {
    config: ConfigRef,
}

impl LspFinder {
    pub fn new(config: ConfigRef) -> LspFinder {
        LspFinder {
            config
        }
    }

    pub fn get_lsp(lang_id: LangId) -> Option<LspWrapper> {
        if lang_id == LangId::RUST {
            LspWrapper::todo_new()
        } else {
            None
        }
    }
}

struct LspWrapper {
    path: PathBuf,
    language: LangId,

    child: Child,
}

pub enum LspWriteError {
    WrongValueType,
    SerializationError(serde_json::error::Error),
    IoError(io::Error),
    BrokenPipe,
    InterruptedWrite,
}

impl From<serde_json::error::Error> for LspWriteError {
    fn from(e: serde_json::error::Error) -> Self {
        LspWriteError::SerializationError(e)
    }
}

impl From<io::Error> for LspWriteError {
    fn from(ioe: io::Error) -> Self {
        LspWriteError::IoError(ioe)
    }
}

impl LspWrapper {
    pub fn todo_new() -> Option<LspWrapper> {
        // TODO unwrap
        let path = PathBuf::from_str("/home/andrzej/.cargo/bin").unwrap();
        let child = Command::new(path.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .ok()?;

        Some(
            LspWrapper {
                path,
                language: LangId::RUST,
                child,
            }
        )
    }


    fn send_request<R: lsp_types::request::Request>(
        &mut self,
        id: u64,
        params: R::Params,
    ) -> Result<(), LspWriteError>
        where
            R::Params: serde::Serialize,
    {
        if let serde_json::value::Value::Object(params) = serde_json::to_value(params)? {
            let req = jsonrpc_core::Call::MethodCall(jsonrpc_core::MethodCall {
                jsonrpc: Some(jsonrpc_core::Version::V2),
                method: R::METHOD.to_string(),
                params: jsonrpc_core::Params::Map(params),
                id: jsonrpc_core::Id::Num(id),
            });
            let request = serde_json::to_string(&req)?;
            let mut buffer: Vec<u8> = Vec::new();
            write!(
                &mut buffer,
                "Content-Length: {}\r\n\r\n{}",
                request.len(),
                request
            )?;
            if let Some(stdin) = &mut self.child.stdin {
                let len = stdin.write(&buffer)?;
                if buffer.len() == len {
                    stdin.flush()?;
                    Ok(())
                } else {
                    Err(LspWriteError::InterruptedWrite)
                }
            } else {
                Err(LspWriteError::BrokenPipe)
            }
        } else {
            Err(LspWriteError::WrongValueType)
        }
    }


    // fn init(&self) {
    //     let x = self.send::<Initialize>(lsp_types::InitializeParams {
    //         process_id: None,
    //         root_path: None,
    //         root_uri: None,
    //         initialization_options: None,
    //         capabilities: Default::default(),
    //         trace: None,
    //         workspace_folders: None,
    //         client_info: None,
    //         locale: None,
    //     });
    //
    //     let mut x = serde_json::Serializer::new(self.child.stdout.unwrap());
    //
    //     // x.serialize(serde_json::ser::Serializer)
    // }

    // fn send<R : Request>(&self, params : R::Params) {
    //     let x = json::stringify(R)
    // }

    // fn send_request_unchecked<R>(&self, params: R::Params) -> jsonrpc::Result<R::Result>
    //     where
    //         R: lsp_types::request::Request,
    // {
    //     let id = self.next_request_id();
    //     let request = Request::from_request::<R>(id, params);
    //
    //     let response = match self.clone().call(request).await {
    //         Ok(Some(response)) => response,
    //         Ok(None) | Err(_) => return Err(Error::internal_error()),
    //     };
    //
    //     let (_, result) = response.into_parts();
    //     result.and_then(|v| {
    //         serde_json::from_value(v).map_err(|e| Error {
    //             code: ErrorCode::ParseError,
    //             message: e.to_string(),
    //             data: None,
    //         })
    //     })
    // }
}

enum LspReadError {
    NoLine,
    IoError(io::Error),
    DeError(serde_json::error::Error),
    ParamCastFailed,
    UnexpectedContents,
}

#[derive(Debug)]
enum LspResponse {
    Unknown(String),
    Initialized(lsp_types::InitializeResult),
}

impl From<io::Error> for LspReadError {
    fn from(ioe: io::Error) -> Self {
        LspReadError::IoError(ioe)
    }
}

impl From<serde_json::error::Error> for LspReadError {
    fn from(dee: serde_json::error::Error) -> Self {
        LspReadError::DeError(dee)
    }
}

impl From<jsonrpc_core::Error> for LspReadError {
    fn from(_: Error) -> Self {
        LspReadError::ParamCastFailed
    }
}

fn read_lsp<R: BufRead>(input: &mut R) -> Result<LspResponse, LspReadError> {
    if let Some(line_res) = input.lines().next() {
        let line = line_res?;

        if !line.starts_with("Content-Length") {
            return Err(LspReadError::UnexpectedContents);
        }

        if let Some(idx) = line.find(":") {
            if let Ok(content_length) = &line[idx + 1..].parse::<usize>() {
                // skipping "\r\n" between Content-Length and body
                if input.lines().next().is_none() {
                    return Err(LspReadError::NoLine);
                }

                // reading content_length
                let mut body: Vec<u8> = Vec::with_capacity(*content_length);
                input.read_exact(body.borrow_mut())?;

                let call = serde_json::from_slice::<jsonrpc_core::MethodCall>(&body)?;


                Ok(match call.method.as_str() {
                    "initialize" => {
                        let params = call.params.parse::<lsp_types::InitializeResult>()?;
                        LspResponse::Initialized(params)
                    }
                    other => {
                        LspResponse::Unknown(call.method)
                    }
                })
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
