use std::io::Write;

use log::debug;
use tokio::io::AsyncWriteExt;

use crate::lsp_client::lsp_write_error::LspWriteError;

pub async fn internal_send_request<R: lsp_types::request::Request, W: tokio::io::AsyncWrite>(
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

        debug!("Sending request:\n---\n{}\n---\n", std::str::from_utf8(&buffer).unwrap());

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

pub async fn internal_send_notification<N: lsp_types::notification::Notification, W: tokio::io::AsyncWrite>(
    stdin: &mut W,
    params: N::Params,
) -> Result<(), LspWriteError>
    where
        N::Params: serde::Serialize,
        W: std::marker::Unpin
{
    if let serde_json::value::Value::Object(params) = serde_json::to_value(params)? {
        let req = jsonrpc_core::Notification {
            /*
            Protocol docs do not expect jsonrpc version here:
            https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#notificationMessage
             */
            jsonrpc: None,
            method: N::METHOD.to_string(),
            params: jsonrpc_core::Params::Map(params),
        };
        let request = serde_json::to_string(&req)?;
        let mut buffer: Vec<u8> = Vec::new();
        write!(
            &mut buffer,
            "Content-Length: {}\r\n\r\n{}",
            request.len(),
            request
        )?;

        debug!("Sending notification:\n---\n{}\n---\n", std::str::from_utf8(&buffer).unwrap());

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

pub async fn internal_send_notification_no_params<N: lsp_types::notification::Notification, W: tokio::io::AsyncWrite>(
    stdin: &mut W,
) -> Result<(), LspWriteError>
    where
        N::Params: serde::Serialize,
        W: std::marker::Unpin
{
    let req = jsonrpc_core::Notification {
        /*
        Protocol docs do not expect jsonrpc version here:
        https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#notificationMessage
         */
        jsonrpc: None,
        method: N::METHOD.to_string(),
        params: jsonrpc_core::Params::None,
    };
    let request = serde_json::to_string(&req)?;
    let mut buffer: Vec<u8> = Vec::new();
    write!(
        &mut buffer,
        "Content-Length: {}\r\n\r\n{}",
        request.len(),
        request
    )?;

    debug!("Sending notification (no params):\n---\n{}\n---\n", std::str::from_utf8(&buffer).unwrap());

    let len = stdin.write(&buffer).await?;
    if buffer.len() == len {
        stdin.flush().await?;
        Ok(())
    } else {
        Err(LspWriteError::InterruptedWrite)
    }
}