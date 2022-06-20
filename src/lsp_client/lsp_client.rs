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
use crate::lsp_client::lsp_client::LspWriteError::BrokenPipe;
use crate::lsp_client::lsp_client::LspReadError::UnexpectedContents;
use crate::lsp_client::lsp_matcher;
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::lsp_client::lsp_response::LspResponse;
use crate::lsp_client::lsp_write_error::LspWriteError;
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
}


#[derive(Debug)]
pub struct LspResponseTuple {
    pub id: jsonrpc_core::Id,
    pub params: LspResponse,
}

pub fn read_lsp<R: BufRead>(input: &mut R) -> Result<LspResponseTuple, LspReadError> {
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
                let id = call.id.clone();

                match lsp_matcher::read_response(call) {
                    Ok(None) => Err(LspReadError::UnknownMethod),
                    Ok(Some(params)) => {
                        Ok(LspResponseTuple {
                            id,
                            params,
                        })
                    }
                    Err(_) => Err(LspReadError::ParamCastFailed)
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
