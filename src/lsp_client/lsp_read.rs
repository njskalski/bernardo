use std::path::{Path, PathBuf};
use std::sync::Arc;

use json::{JsonValue, stringify_pretty};
use jsonrpc_core::{Call, Id, MethodCall, Output};
use log::{debug, error};
use regex::internal::Input;
use serde_json::{json, Value};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

use crate::lsp_client::debug_helpers::{format_or_noop, lsp_debug_save};
use crate::lsp_client::lsp_client::IdToCallInfo;
use crate::lsp_client::lsp_notification::{LspNotificationParsingError, LspServerNotification, parse_notification};
use crate::lsp_client::lsp_read_error::LspReadError;

const FAKE_RESPONSE_PREFIX: &'static str = "HTTP/1.1 200 OK\r\n";

// TODO one can reduce allocation here
fn id_to_str(id: Id) -> String {
    match id {
        Id::Null => "".to_string(),
        Id::Num(u) => format!("{}", u),
        /// String id
        Id::Str(s) => s,
    }
}

pub async fn read_lsp<R: tokio::io::AsyncRead + std::marker::Unpin>(
    identifier: &str,
    num: &mut usize,
    input: &mut R,
    id_to_method: &Arc<RwLock<IdToCallInfo>>,
    notification_sink: &UnboundedSender<LspServerNotification>,
) -> Result<(), LspReadError> {
    let mut headers: Vec<u8> = Vec::new();

    loop {
        let mut buf: [u8; 1] = [0];
        input.read(&mut buf).await?;
        headers.push(buf[0]);

        if headers.len() > 3 {
            // The crosses below are an exorcism against heretic line terminators.
            if headers.ends_with(/* ✞ */"\r\n\r\n".as_bytes() /* ✞ */) {
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
        input.read(&mut buf).await?;
        body.push(buf[0]);
    }

    // debug!("got it:\n[{}]", std::str::from_utf8(&body).unwrap());
    let s = std::str::from_utf8(&body).unwrap();

    #[cfg(debug_assertions)]
    {
        let dir = Path::new(identifier);

        let file = dir.join(format!("incoming-{}.json", num));
        let pretty_string = format_or_noop(s.to_string());

        tokio::spawn(lsp_debug_save(file, pretty_string));
    }

    if let Ok(call) = jsonrpc_core::serde_from_str::<jsonrpc_core::Call>(&s) {
        match call {
            Call::MethodCall(call) => {
                debug!("deserialized call->method_call");
                let value: Value = call.params.into();
                internal_send(&id_to_method,
                              call.id.clone(),
                              value,
                              Some(&call.method),
                ).await
            }
            Call::Notification(notification) => {
                debug!("deserialized call->notification");
                match parse_notification(notification) {
                    Ok(no) => {
                        notification_sink.send(no).map_err(|_| LspReadError::BrokenChannel)?;
                        Ok(())
                    }
                    Err(e) => {
                        Err(LspReadError::DeError(e.to_string()))
                    }
                }
            }
            Call::Invalid { id } => {
                debug!("deserialized invalid id: {:?}", id);

                // the fact that failed to

                if let Ok(resp) = jsonrpc_core::serde_from_str::<jsonrpc_core::Response>(&s) {
                    debug!("deserialized response");
                    if let jsonrpc_core::Response::Single(single) = resp {
                        match single {
                            Output::Failure(fail) => {
                                debug!("failed parsing response, because {:?}", fail);
                                Err(LspReadError::JsonRpcError(fail.error.to_string()))
                            },
                            Output::Success(succ) => {
                                debug!("call info id {:?}", &succ.id);
                                internal_send(&id_to_method,
                                              succ.id,
                                              succ.result,
                                              None,
                                ).await
                            }
                        }
                    } else {
                        Err(LspReadError::NotSingleResponse)
                    }
                } else if let Ok(notification) = jsonrpc_core::serde_from_str::<jsonrpc_core::Notification>(&s) {
                    debug!("deserialized notification");
                    match parse_notification(notification) {
                        Ok(no) => {
                            notification_sink.send(no).map_err(|_| LspReadError::BrokenChannel)?;
                            Ok(())
                        }
                        Err(e) => {
                            Err(LspReadError::DeError(e.to_string()))
                        }
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

async fn internal_send(
    id_to_method: &Arc<RwLock<IdToCallInfo>>,
    id: Id,
    value: Value,
    method: Option<&String>,
) -> Result<(), LspReadError> {
    let id = id_to_str(id);
    debug!("call info id {}", &id);
    if let Some(call_info) = id_to_method.write().await.remove(&id) {
        match call_info.sender.send(value) {
            Ok(_) => {
                debug!("sent {} to {}", call_info.method, &id);
                Ok(())
            },
            Err(_) => {
                debug!("failed to send {} to {}, because of broken channel", call_info.method, &id);
                Err(LspReadError::BrokenChannel)
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

static CONTENT_LENGTH_HEADER: &'static str = "Content-Length:";

pub fn get_len_from_headers(headers: &String) -> Option<usize> {
    for line in headers.lines() {
        if line.trim().starts_with(&CONTENT_LENGTH_HEADER) {
            let bytes_num_str = &line[CONTENT_LENGTH_HEADER.len() + 1..];
            let bytes_num = bytes_num_str.parse::<usize>().ok();
            return bytes_num;
        }
    }

    None
}


#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Call>(&s).is_ok(), true);
        assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::MethodCall>(&s).is_ok(), true);
        // assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Notification>(&s).is_ok(), true);
        assert_eq!(jsonrpc_core::serde_from_str::<jsonrpc_core::Request>(&s).is_ok(), true);
    }
}