use jsonrpc_core::Error;
use lsp_types::{CancelParams, CreateFilesParams, DeleteFilesParams, DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializedParams, LogMessageParams, LogTraceParams, ProgressParams, PublishDiagnosticsParams, RenameFilesParams, SetTraceParams, ShowMessageParams, WillSaveTextDocumentParams, WorkDoneProgressCancelParams};
use lsp_types::notification as n;
use lsp_types::notification::Notification;

#[derive(Debug)]
pub enum LspNotificationParsingError {
    UnknownMethod,
    ParamsParseFailed(jsonrpc_core::error::Error),
}

impl From<jsonrpc_core::error::Error> for LspNotificationParsingError {
    fn from(e: Error) -> Self {
        LspNotificationParsingError::ParamsParseFailed(e)
    }
}

impl ToString for LspNotificationParsingError {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

pub fn parse_notification(jn: jsonrpc_core::Notification) -> Result<LspServerNotification, LspNotificationParsingError> {
    match jn.method.as_str() {
        n::Cancel::METHOD => {
            let params = jn.params.parse::<CancelParams>()?;
            Ok(LspServerNotification::CancelRequest(params))
        }
        n::SetTrace::METHOD => {
            let params = jn.params.parse::<SetTraceParams>()?;
            Ok(LspServerNotification::SetTrace(params))
        }
        n::LogTrace::METHOD => {
            let params = jn.params.parse::<LogTraceParams>()?;
            Ok(LspServerNotification::LogTrace(params))
        }
        n::Initialized::METHOD => {
            let params = jn.params.parse::<InitializedParams>()?;
            Ok(LspServerNotification::Initialized(params))
        }
        n::Exit::METHOD => {
            Ok(LspServerNotification::Exit(()))
        }
        n::ShowMessage::METHOD => {
            let params = jn.params.parse::<ShowMessageParams>()?;
            Ok(LspServerNotification::WindowShowMessage(params))
        }
        n::LogMessage::METHOD => {
            let params = jn.params.parse::<LogMessageParams>()?;
            Ok(LspServerNotification::WindowLogMessage(params))
        }
        n::WorkDoneProgressCancel::METHOD => {
            let params = jn.params.parse::<WorkDoneProgressCancelParams>()?;
            Ok(LspServerNotification::WindowWorkDoneProgressCancel(params))
        }
        n::TelemetryEvent::METHOD => {
            let params = jn.params.parse::<serde_json::Value>()?;
            Ok(LspServerNotification::TelemetryEvent(params))
        }
        n::DidOpenTextDocument::METHOD => {
            let params = jn.params.parse::<DidOpenTextDocumentParams>()?;
            Ok(LspServerNotification::TextDocumentDidOpen(params))
        }
        n::DidChangeTextDocument::METHOD => {
            let params = jn.params.parse::<DidChangeTextDocumentParams>()?;
            Ok(LspServerNotification::TextDocumentDidChange(params))
        }
        n::WillSaveTextDocument::METHOD => {
            let params = jn.params.parse::<WillSaveTextDocumentParams>()?;
            Ok(LspServerNotification::TextDocumentWillSave(params))
        }
        n::DidSaveTextDocument::METHOD => {
            let params = jn.params.parse::<DidSaveTextDocumentParams>()?;
            Ok(LspServerNotification::TextDocumentDidSave(params))
        }
        n::DidCloseTextDocument::METHOD => {
            let params = jn.params.parse::<DidCloseTextDocumentParams>()?;
            Ok(LspServerNotification::TextDocumentDidClose(params))
        }
        n::PublishDiagnostics::METHOD => {
            let params = jn.params.parse::<PublishDiagnosticsParams>()?;
            Ok(LspServerNotification::TextDocumentPublishDiagnostics(params))
        }
        n::DidChangeConfiguration::METHOD => {
            let params = jn.params.parse::<DidChangeConfigurationParams>()?;
            Ok(LspServerNotification::WorkspaceDidChangeConfiguration(params))
        }
        n::DidChangeWatchedFiles::METHOD => {
            let params = jn.params.parse::<DidChangeWatchedFilesParams>()?;
            Ok(LspServerNotification::WorkspaceDidChangeWatchedFiles(params))
        }
        n::DidChangeWorkspaceFolders::METHOD => {
            let params = jn.params.parse::<DidChangeWorkspaceFoldersParams>()?;
            Ok(LspServerNotification::WorkspaceDidChangeWorkspaceFolders(params))
        }
        n::Progress::METHOD => {
            let params = jn.params.parse::<ProgressParams>()?;
            Ok(LspServerNotification::Progress(params))
        }
        n::DidCreateFiles::METHOD => {
            let params = jn.params.parse::<CreateFilesParams>()?;
            Ok(LspServerNotification::WorkspaceDidCreateFiles(params))
        }
        n::DidRenameFiles::METHOD => {
            let params = jn.params.parse::<RenameFilesParams>()?;
            Ok(LspServerNotification::WorkspaceDidRenameFiles(params))
        }
        n::DidDeleteFiles::METHOD => {
            let params = jn.params.parse::<DeleteFilesParams>()?;
            Ok(LspServerNotification::WorkspaceDidDeleteFiles(params))
        }
        _ => Err(LspNotificationParsingError::UnknownMethod)
    }
}

#[derive(Debug)]
pub enum LspServerNotification {
    CancelRequest(CancelParams),
    SetTrace(SetTraceParams),
    LogTrace(LogTraceParams),
    Initialized(InitializedParams),
    Exit(()),
    WindowShowMessage(ShowMessageParams),
    WindowLogMessage(LogMessageParams),
    WindowWorkDoneProgressCancel(WorkDoneProgressCancelParams),
    TelemetryEvent(serde_json::Value),
    TextDocumentDidOpen(DidOpenTextDocumentParams),
    TextDocumentDidChange(DidChangeTextDocumentParams),
    TextDocumentWillSave(WillSaveTextDocumentParams),
    TextDocumentDidSave(DidSaveTextDocumentParams),
    TextDocumentDidClose(DidCloseTextDocumentParams),
    TextDocumentPublishDiagnostics(PublishDiagnosticsParams),
    WorkspaceDidChangeConfiguration(DidChangeConfigurationParams),
    WorkspaceDidChangeWatchedFiles(DidChangeWatchedFilesParams),
    WorkspaceDidChangeWorkspaceFolders(DidChangeWorkspaceFoldersParams),
    Progress(ProgressParams),
    WorkspaceDidCreateFiles(CreateFilesParams),
    WorkspaceDidRenameFiles(RenameFilesParams),
    WorkspaceDidDeleteFiles(DeleteFilesParams),
}

