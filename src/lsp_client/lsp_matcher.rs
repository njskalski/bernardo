use jsonrpc_core::{MethodCall, Response, Success};
use lsp_types::{ApplyWorkspaceEditResponse, CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall, CodeAction, CodeActionResponse, CodeLens, ColorInformation, ColorPresentation, CompletionItem, CompletionResponse, DocumentHighlight, DocumentLink, DocumentSymbolResponse, FoldingRange, GotoDefinitionResponse, Hover, InitializeResult, LinkedEditingRanges, Location, lsp_request, MessageActionItem, Moniker, PrepareRenameResponse, SelectionRange, SemanticTokensFullDeltaResult, SemanticTokensRangeResult, SemanticTokensResult, ShowDocumentResult, SignatureHelp, SymbolInformation, TextEdit, WorkspaceEdit, WorkspaceFolder};
use lsp_types::request::{GotoDeclarationResponse, GotoImplementationResponse, GotoTypeDefinitionResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use crate::lsp_client::lsp_response::LspResponse;

// I was not able to simplify this for an hour, and refused to try more.
pub fn read_response(method: &str, value: Value) -> Result<Option<LspResponse>, serde_json::error::Error> {
    Ok(match method {
        "initialize" => {
            let paras = serde_json::from_value::<InitializeResult>(value)?;
            Some(LspResponse::Initialize(paras))
        }
        "shutdown" => {
            Some(LspResponse::Shutdown(()))
        }
        "window/showMessageRequest" => {
            let paras = serde_json::from_value::<Option<MessageActionItem>>(value)?;
            Some(LspResponse::WindowShowMessageRequest(paras))
        }
        "client/registerCapability" => {
            Some(LspResponse::ClientRegisterCapability(()))
        }
        "client/unregisterCapability" => {
            Some(LspResponse::ClientUnregisterCapability(()))
        }
        "workspace/symbol" => {
            let paras = serde_json::from_value::<Option<Vec<SymbolInformation>>>(value)?;
            Some(LspResponse::WorkspaceSymbol(paras))
        }
        "workspace/executeCommand" => {
            let paras = serde_json::from_value::<Option<Value>>(value)?;
            Some(LspResponse::WorkspaceExecuteCommand(paras))
        }
        "textDocument/willSaveWaitUntil" => {
            let paras = serde_json::from_value::<Option<Vec<TextEdit>>>(value)?;
            Some(LspResponse::TextDocumentWillSaveWaitUntil(paras))
        }
        "textDocument/completion" => {
            let paras = serde_json::from_value::<Option<CompletionResponse>>(value)?;
            Some(LspResponse::TextDocumentCompletion(paras))
        }
        "completionItem/resolve" => {
            let paras = serde_json::from_value::<CompletionItem>(value)?;
            Some(LspResponse::CompletionItemResolve(paras))
        }
        "textDocument/hover" => {
            let paras = serde_json::from_value::<Option<Hover>>(value)?;
            Some(LspResponse::TextDocumentHover(paras))
        }
        "textDocument/signatureHelp" => {
            let paras = serde_json::from_value::<Option<SignatureHelp>>(value)?;
            Some(LspResponse::TextDocumentSignatureHelp(paras))
        }
        "textDocument/declaration" => {
            let paras = serde_json::from_value::<Option<GotoDeclarationResponse>>(value)?;
            Some(LspResponse::TextDocumentDeclaration(paras))
        }
        "textDocument/definition" => {
            let paras = serde_json::from_value::<Option<GotoDefinitionResponse>>(value)?;
            Some(LspResponse::TextDocumentDefinition(paras))
        }
        "textDocument/references" => {
            let paras = serde_json::from_value::<Option<Vec<Location>>>(value)?;
            Some(LspResponse::TextDocumentReferences(paras))
        }
        "textDocument/documentHighlight" => {
            let paras = serde_json::from_value::<Option<Vec<DocumentHighlight>>>(value)?;
            Some(LspResponse::TextDocumentDocumentHighlight(paras))
        }
        "textDocument/documentSymbol" => {
            let paras = serde_json::from_value::<Option<DocumentSymbolResponse>>(value)?;
            Some(LspResponse::TextDocumentDocumentSymbol(paras))
        }
        "textDocument/codeAction" => {
            let paras = serde_json::from_value::<Option<CodeActionResponse>>(value)?;
            Some(LspResponse::TextDocumentCodeAction(paras))
        }
        "textDocument/codeLens" => {
            let paras = serde_json::from_value::<Option<Vec<CodeLens>>>(value)?;
            Some(LspResponse::TextDocumentCodeLens(paras))
        }
        "codeLens/resolve" => {
            let paras = serde_json::from_value::<CodeLens>(value)?;
            Some(LspResponse::CodeLensResolve(paras))
        }
        "textDocument/documentLink" => {
            let paras = serde_json::from_value::<Option<Vec<DocumentLink>>>(value)?;
            Some(LspResponse::TextDocumentDocumentLink(paras))
        }
        "documentLink/resolve" => {
            let paras = serde_json::from_value::<DocumentLink>(value)?;
            Some(LspResponse::DocumentLinkResolve(paras))
        }
        "workspace/applyEdit" => {
            let paras = serde_json::from_value::<ApplyWorkspaceEditResponse>(value)?;
            Some(LspResponse::WorkspaceApplyEdit(paras))
        }
        "textDocument/rangeFormatting" => {
            let paras = serde_json::from_value::<Option<Vec<TextEdit>>>(value)?;
            Some(LspResponse::TextDocumentRangeFormatting(paras))
        }
        "textDocument/onTypeFormatting" => {
            let paras = serde_json::from_value::<Option<Vec<TextEdit>>>(value)?;
            Some(LspResponse::TextDocumentOnTypeFormatting(paras))
        }
        "textDocument/formatting" => {
            let paras = serde_json::from_value::<Option<Vec<TextEdit>>>(value)?;
            Some(LspResponse::TextDocumentFormatting(paras))
        }
        "textDocument/rename" => {
            let paras = serde_json::from_value::<Option<WorkspaceEdit>>(value)?;
            Some(LspResponse::TextDocumentRename(paras))
        }
        "textDocument/documentColor" => {
            let paras = serde_json::from_value::<Vec<ColorInformation>>(value)?;
            Some(LspResponse::TextDocumentDocumentColor(paras))
        }
        "textDocument/colorPresentation" => {
            let paras = serde_json::from_value::<Vec<ColorPresentation>>(value)?;
            Some(LspResponse::TextDocumentColorPresentation(paras))
        }
        "textDocument/foldingRange" => {
            let paras = serde_json::from_value::<Option<Vec<FoldingRange>>>(value)?;
            Some(LspResponse::TextDocumentFoldingRange(paras))
        }
        "textDocument/prepareRename" => {
            let paras = serde_json::from_value::<Option<PrepareRenameResponse>>(value)?;
            Some(LspResponse::TextDocumentPrepareRename(paras))
        }
        "textDocument/implementation" => {
            let paras = serde_json::from_value::<Option<GotoImplementationResponse>>(value)?;
            Some(LspResponse::TextDocumentImplementation(paras))
        }
        "textDocument/typeDefinition" => {
            let paras = serde_json::from_value::<Option<GotoTypeDefinitionResponse>>(value)?;
            Some(LspResponse::TextDocumentTypeDefinition(paras))
        }
        "textDocument/selectionRange" => {
            let paras = serde_json::from_value::<Option<Vec<SelectionRange>>>(value)?;
            Some(LspResponse::TextDocumentSelectionRange(paras))
        }
        "workspace/workspaceFolders" => {
            let paras = serde_json::from_value::<Option<Vec<WorkspaceFolder>>>(value)?;
            Some(LspResponse::WorkspaceWorkspaceFolders(paras))
        }
        "workspace/configuration" => {
            let paras = serde_json::from_value::<Vec<Value>>(value)?;
            Some(LspResponse::WorkspaceConfiguration(paras))
        }
        "window/workDoneProgress/create" => {
            Some(LspResponse::WindowWorkDoneProgressCreate(()))
        }
        "callHierarchy/incomingCalls" => {
            let paras = serde_json::from_value::<Option<Vec<CallHierarchyIncomingCall>>>(value)?;
            Some(LspResponse::CallHierarchyIncomingCall(paras))
        }
        "callHierarchy/outgoingCalls" => {
            let paras = serde_json::from_value::<Option<Vec<CallHierarchyOutgoingCall>>>(value)?;
            Some(LspResponse::CallHierarchyOutgoingCalls(paras))
        }
        "textDocument/moniker" => {
            let paras = serde_json::from_value::<Option<Vec<Moniker>>>(value)?;
            Some(LspResponse::TextDocumentMoniker(paras))
        }
        "textDocument/linkedEditingRange" => {
            let paras = serde_json::from_value::<Option<LinkedEditingRanges>>(value)?;
            Some(LspResponse::TextDocumentLinkedEditingRange(paras))
        }
        "textDocument/prepareCallHierarchy" => {
            let paras = serde_json::from_value::<Option<Vec<CallHierarchyItem>>>(value)?;
            Some(LspResponse::TextDocumentPrepareCallHierarchy(paras))
        }
        "textDocument/semanticTokens/full" => {
            let paras = serde_json::from_value::<Option<SemanticTokensResult>>(value)?;
            Some(LspResponse::TextDocumentSemanticTokensFull(paras))
        }
        "textDocument/semanticTokens/full/delta" => {
            let paras = serde_json::from_value::<Option<SemanticTokensFullDeltaResult>>(value)?;
            Some(LspResponse::TextDocumentSemanticTokensFullDelta(paras))
        }
        "textDocument/semanticTokens/range" => {
            let paras = serde_json::from_value::<Option<SemanticTokensRangeResult>>(value)?;
            Some(LspResponse::TextDocumentSemanticTokensRange(paras))
        }
        "workspace/willCreateFiles" => {
            let paras = serde_json::from_value::<Option<WorkspaceEdit>>(value)?;
            Some(LspResponse::WorkspaceWillCreateFiles(paras))
        }
        "workspace/willRenameFiles" => {
            let paras = serde_json::from_value::<Option<WorkspaceEdit>>(value)?;
            Some(LspResponse::WorkspaceWillRenameFiles(paras))
        }
        "workspace/willDeleteFiles" => {
            let paras = serde_json::from_value::<Option<WorkspaceEdit>>(value)?;
            Some(LspResponse::WorkspaceWillDeleteFiles(paras))
        }
        "workspace/semanticTokens/refresh" => {
            Some(LspResponse::WorkspaceSemanticTokensRefresh(()))
        }
        "workspace/codeLens/refresh" => {
            Some(LspResponse::WorkspaceCodeLensRefresh(()))
        }
        "codeAction/resolve" => {
            let paras = serde_json::from_value::<CodeAction>(value)?;
            Some(LspResponse::CodeActionResolve(paras))
        }
        "window/showDocument" => {
            let paras = serde_json::from_value::<ShowDocumentResult>(value)?;
            Some(LspResponse::WindowShowDocument(paras))
        }
        _ => None,
    })
}