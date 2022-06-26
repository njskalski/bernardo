use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::format;
use std::io;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use jsonrpc_core::{Error, Id, MethodCall, Output};
use jsonrpc_core_client::RawClient;
use jsonrpc_core_client::transports::local::connect;
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

// I use ID == String, because i32 might be small, and i64 is safe, so I send i64 as string and so I store it.
// LSP defines id integer as i32, while jsonrpc_core as u64.
type IdToMethod = HashMap<String, &'static str>;

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
    ids: IdToMethod,

    curr_id: u64,
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
                ids: IdToMethod::default(),
                curr_id: 1,
            }
        )
    }

    pub fn send_message<R: lsp_types::request::Request>(&mut self, params: R::Params) -> Result<(), LspWriteError> {
        let new_id = format!("{}", self.curr_id);
        self.curr_id += 1;

        if self.ids.insert(new_id.clone(), R::METHOD).is_some() {
            // TODO this is a reuse of id, super unlikely
        }

        if let Some(stdin) = self.child.stdin.as_mut() {
            internal_send_request::<R, _>(stdin, new_id, params)
        } else {
            Err(LspWriteError::BrokenPipe)
        }
    }
}

fn internal_send_request<R: lsp_types::request::Request, W: Write>(
    stdin: &mut W,
    id: String,
    params: R::Params,
) -> Result<(), LspWriteError>
    where
        R::Params: serde::Serialize
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

        let len = stdin.write(&buffer)?;
        if buffer.len() == len {
            stdin.flush()?;
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
    pub params: LspResponse,
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

pub fn read_lsp<R: BufRead>(input: &mut R, id_to_method: &IdToMethod) -> Result<LspResponseTuple, LspReadError> {
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

                let resp = serde_json::from_slice::<jsonrpc_core::Response>(&body)?;

                if let jsonrpc_core::Response::Single(single) = resp {
                    match single {
                        Output::Failure(fail) => Err(LspReadError::LspFailure(fail.error)),
                        Output::Success(succ) => {
                            let id = id_to_str(succ.id);
                            if let Some(method) = id_to_method.get(&id) {
                                if let Some(response) = lsp_matcher::read_response(*method, succ.result)? {
                                    Ok(LspResponseTuple {
                                        id,
                                        method: *method,
                                        params: response,
                                    })
                                } else {
                                    Err(LspReadError::UnknownMethod)
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