use std::sync::Arc;

use jsonrpc_core::{Id, Output};
use log::{debug, error};
use serde_json::json;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

use crate::lsp_client::lsp_client::IdToCallInfo;
use crate::lsp_client::lsp_notification::{LspServerNotification, parse_notification};
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
    input: &mut R,
    response_parser: &mut stream_httparse::streaming_parser::RespParser,
    id_to_method: &Arc<RwLock<IdToCallInfo>>,
    notification_sink: &UnboundedSender<LspServerNotification>,
) -> Result<(), LspReadError> {
    {
        let (done, leftover_bytes) = response_parser.block_parse(FAKE_RESPONSE_PREFIX.as_bytes());
        if done || leftover_bytes > 0 {
            error!("unexpected early exit of http response parser: (done: {}, leftover_bytes: {})", done, leftover_bytes);
        }
    }

    loop {
        let mut buf: [u8; 1] = [0; 1];
        let bytes_read = input.read(&mut buf).await?;
        if bytes_read == 0 {
            error!("bytes_read == 0, eof?");
            return Err(LspReadError::BrokenChannel);
        }
        let (done, leftover_bytes) = response_parser.block_parse(&buf);
        if leftover_bytes > 0 {
            error!("expected to stop parsing before retrieving unused bytes, got {}", leftover_bytes);
        }

        if done {
            break;
        }
    }

    let response = response_parser.finish()?;
    debug!("Receiving {:?} and body of {} bytes", response.status_code(), response.body().len());
    debug!("{}", std::str::from_utf8(response.body()).unwrap());

    // TODO add notification type.
    if let Ok(resp) = serde_json::from_slice::<jsonrpc_core::Response>(response.body()) {
        if let jsonrpc_core::Response::Single(single) = resp {
            match single {
                Output::Failure(fail) => {
                    debug!("failed parsing response, because {:?}", fail);
                    Err(LspReadError::LspFailure(fail.error))
                },
                Output::Success(succ) => {
                    let id = id_to_str(succ.id);
                    debug!("call info id {}", &id);
                    if let Some(call_info) = id_to_method.write().await.remove(&id) {
                        match call_info.sender.send(succ.result) {
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
                        Err(LspReadError::UnknownMethod)
                    }
                }
            }
        } else {
            Err(LspReadError::NotSingleResponse)
        }
    } else if let Ok(notification) = serde_json::from_slice::<jsonrpc_core::Notification>(response.body()) {
        match parse_notification(notification) {
            Ok(no) => {
                notification_sink.send(no).map_err(|_| LspReadError::BrokenChannel)?;
                Ok(())
            }
            Err(e) => {
                Err(LspReadError::BrokenChannel)
            }
        }
    } else {
        Err(LspReadError::UnexpectedContents)
    }
}