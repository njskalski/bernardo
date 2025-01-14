use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crossbeam_channel::{SendError, Sender};
use jsonrpc_core::{Call, Id, Output};
use log::{debug, error, warn};
use serde_json::Value;

use crate::lsp_client::debug_helpers::{format_or_noop, lsp_debug_save};
use crate::lsp_client::diagnostic;
use crate::lsp_client::diagnostic::Diagnostic;
use crate::lsp_client::lsp_client::{FilesStore, IdToCallInfo, LSPFileDescriptor};
use crate::lsp_client::lsp_notification::{parse_notification, LspServerNotification};
use crate::lsp_client::lsp_read_error::LspReadError;
use crate::unpack_unit_e;

// TODO one can reduce allocation here
fn id_to_str(id: Id) -> String {
    match id {
        Id::Null => "".to_string(),
        Id::Num(u) => format!("{}", u),
        // String id
        Id::Str(s) => s,
    }
}

pub fn read_lsp<R: Read>(
    identifier: &str,
    num: &mut usize,
    input: &mut R,
    id_to_method: &Arc<RwLock<IdToCallInfo>>,
    files: &FilesStore,
) -> Result<(), LspReadError> {
    let mut headers: Vec<u8> = Vec::new();

    loop {
        let mut buf: [u8; 1] = [0];
        input.read_exact(&mut buf)?;
        headers.push(buf[0]);

        if headers.len() > 3 {
            // The crosses below are an exorcism against heretic line terminators.
            if headers.ends_with(/* ✞ */ "\r\n\r\n".as_bytes() /* ✞ */) {
                break;
            }
        }
    }

    let headers_string = String::from_utf8(headers)?;
    let body_len = match get_len_from_headers(&headers_string) {
        None => return Err(LspReadError::NoContentLength),
        Some(i) => i,
    };
    debug!("Receiving body of {} bytes", body_len);

    let mut body: Vec<u8> = Vec::with_capacity(body_len);
    while body.len() < body_len {
        let mut buf: [u8; 1] = [0];
        input.read_exact(&mut buf)?;
        body.push(buf[0]);
    }

    // debug!("got it:\n[{}]", std::str::from_utf8(&body).unwrap());
    let s = std::str::from_utf8(&body).unwrap();

    #[cfg(debug_assertions)]
    {
        let dir = Path::new(identifier);

        let file = dir.join(format!("incoming-{}.json", num));
        let pretty_string = format_or_noop(s.to_string());

        lsp_debug_save(file, pretty_string);
    }

    if let Ok(call) = jsonrpc_core::serde_from_str::<jsonrpc_core::Call>(s) {
        match call {
            Call::MethodCall(call) => {
                debug!("deserialized call->method_call");
                let value: Value = call.params.into();
                internal_send_to_promise(id_to_method, call.id.clone(), value, Some(&call.method))
            }
            Call::Notification(notification) => {
                debug!("deserialized call->notification {:?}", &notification.method);
                match parse_notification(notification) {
                    Ok(no) => {
                        internal_process_incoming_notification(no, files);
                        Ok(())
                    }
                    Err(e) => Err(LspReadError::DeError(e.to_string())),
                }
            }
            Call::Invalid { id } => {
                debug!("deserialized invalid id: {:?}", id);

                if let Ok(resp) = jsonrpc_core::serde_from_str::<jsonrpc_core::Response>(s) {
                    debug!("deserialized response");
                    if let jsonrpc_core::Response::Single(single) = resp {
                        match single {
                            Output::Failure(fail) => {
                                debug!("failed parsing response, because {:?}", fail);
                                Err(LspReadError::JsonRpcError(fail.error.to_string()))
                            }
                            Output::Success(succ) => {
                                debug!("succ call info id {:?}", &succ.id);
                                internal_send_to_promise(id_to_method, succ.id, succ.result, None)
                            }
                        }
                    } else {
                        Err(LspReadError::NotSingleResponse)
                    }
                } else if let Ok(notification) = jsonrpc_core::serde_from_str::<jsonrpc_core::Notification>(s) {
                    debug!("deserialized notification");
                    match parse_notification(notification) {
                        Ok(no) => {
                            internal_process_incoming_notification(no, files);
                            Ok(())
                        }
                        Err(e) => Err(LspReadError::DeError(e.to_string())),
                    }
                } else {
                    error!("failed to parse [{}] into either Notification or Response", &s);
                    Err(LspReadError::UnexpectedContents)
                }
            }
        }
    } else {
        debug!("failed deserializing as call, even a failed one.");
        Err(LspReadError::UnexpectedContents)
    }
}

fn internal_send_to_promise(
    id_to_method: &Arc<RwLock<IdToCallInfo>>,
    id: Id,
    value: Value,
    method: Option<&String>,
) -> Result<(), LspReadError> {
    let id = id_to_str(id);
    debug!("call info id {}", &id);
    if let Some(call_info) = id_to_method.write()?.remove(&id) {
        match call_info.sender.send(value) {
            Ok(_) => {
                debug!("sent {} to {}", call_info.method, &id);
                Ok(())
            }
            Err(_) => {
                debug!("failed to send {} to {}, promise expired", call_info.method, &id);
                Err(LspReadError::PromiseExpired { id })
            }
        }
    } else {
        debug!("not waiting for call with id {:?}", &id);
        Err(LspReadError::UnmatchedId {
            id: id.to_string(),
            method: method.map(|m| m.to_string()).unwrap_or("<unset>".to_string()),
        })
    }
}

fn internal_process_incoming_notification(no: LspServerNotification, files: &FilesStore) {
    match no {
        LspServerNotification::TextDocumentPublishDiagnostics(diagnostics) => {
            let mut file_store = unpack_unit_e!(files.write().ok(), "failed to lock LSP FileStore");

            if !file_store.contains_key(&diagnostics.uri) {
                file_store.insert(
                    diagnostics.uri.clone(),
                    LSPFileDescriptor {
                        version: 0,
                        diagnostics: vec![],
                    },
                );
            }

            let file_desc = file_store.get_mut(&diagnostics.uri).unwrap();

            if let Some(version) = diagnostics.version {
                if version < file_desc.version {
                    warn!("discarding outdated diagnostics for file {}", diagnostics.uri);
                }
                file_desc.version = version;
            }

            let diagnostics: Vec<Diagnostic> = diagnostics.diagnostics.into_iter().map(diagnostic::Diagnostic::from).collect();
            file_desc.diagnostics = diagnostics;
        }
        other => {
            debug!("ignoring LSP diagnostics {:?}, because no handling of it is implemented", other);
        }
    }
}

static CONTENT_LENGTH_HEADER: &str = "Content-Length:";

pub fn get_len_from_headers(headers: &str) -> Option<usize> {
    for line in headers.lines() {
        if line.trim().starts_with(CONTENT_LENGTH_HEADER) {
            let bytes_num_str = &line[CONTENT_LENGTH_HEADER.len() + 1..];
            let bytes_num = bytes_num_str.parse::<usize>().ok();
            return bytes_num;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn json_deserialize() {
        let s = r#"{"jsonrpc":"2.0","id":0,"method":"client/registerCapability","params":{"registrations":[{"id":"textDocument/didSave","method":"textDocument/didSave","registerOptions":{"includeText":false,"documentSelector":[{"pattern":"**/*.rs"},{"pattern":"**/Cargo.toml"},{"pattern":"**/Cargo.lock"}]}}]}}"#;

        // This entire test was to figure out what does the output of LSP deserializes to.
        // Guess what, it deserializes to REQUEST. W T F.

        //Failure, Output, Response, Success
        // assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Failure>(&s).is_ok(), true);
        // assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Output>(&s).is_ok(), true);
        // assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Response>(&s).is_ok(), true);
        // assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Success>(&s).is_ok(), true);

        //Call, MethodCall, Notification, Request
        assert!(jsonrpc_core::serde_from_str::<jsonrpc_core::Call>(s).is_ok());
        assert!(jsonrpc_core::serde_from_str::<jsonrpc_core::MethodCall>(s).is_ok());
        // assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Notification>(&s).is_ok(), true);
        assert!(jsonrpc_core::serde_from_str::<jsonrpc_core::Request>(s).is_ok());
    }
}
