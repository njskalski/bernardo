use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use lsp_types::request::{Initialize, Request};
use serde::Serialize;
use serde_json::Error;
use crate::ConfigRef;
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

pub enum LspError {
    WrongValueType,
    SerializationError(serde_json::error::Error),
    BrokenPipe,
}

impl From<serde_json::error::Error> for LspError {
    fn from(e: Error) -> Self {
        LspError::SerializationError(e)
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
    ) -> Result<R::Result, LspError>
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
            self.child.stdin.as_mut().map(|i| {
                i.write(&buffer)
            })

            t.write_all(&buffer).await?;
            Ok(())
        } else {
            LspError::WrongValueType
        }
    }


    fn init(&self) {
        let x = self.send::<Initialize>(lsp_types::InitializeParams {
            process_id: None,
            root_path: None,
            root_uri: None,
            initialization_options: None,
            capabilities: Default::default(),
            trace: None,
            workspace_folders: None,
            client_info: None,
            locale: None,
        });

        let mut x = serde_json::Serializer::new(self.child.stdout.unwrap());

        x.serialize(serde_json::ser::Serializer)
    }

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

fn call<R>()
    where
        R: lsp_types::request::Request,
        R::Params: serde::Serialize,
        R::Result: serde::de::DeserializeOwned,
{}

