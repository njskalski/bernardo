use jsonrpc_core::MethodCall;
use lsp_types::{ApplyWorkspaceEditResponse, CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall, CodeAction, CodeActionResponse, CodeLens, ColorInformation, ColorPresentation, CompletionItem, CompletionResponse, DocumentHighlight, DocumentLink, DocumentSymbolResponse, FoldingRange, GotoDefinitionResponse, Hover, InitializeResult, LinkedEditingRanges, Location, lsp_request, MessageActionItem, Moniker, PrepareRenameResponse, SelectionRange, SemanticTokensFullDeltaResult, SemanticTokensRangeResult, SemanticTokensResult, ShowDocumentResult, SignatureHelp, SymbolInformation, TextEdit, WorkspaceEdit, WorkspaceFolder};
use lsp_types::request::{GotoDeclarationResponse, GotoImplementationResponse, GotoTypeDefinitionResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use crate::experiments::lsp_response::LspResponse;

// I was not able to simplify this for an hour, and refused to try more.
pub(crate) fn read_response(call: MethodCall) -> Result<Option<LspResponse>, jsonrpc_core::Error> {
    Ok(match call.method.as_str() {
        "initialize" => {
            let paras = call.params.parse::<InitializeResult>()?;
            Some(LspResponse::Initialize(paras))
        }
        "shutdown" => {
            Some(LspResponse::Shutdown(()))
        }
        "window/showMessageRequest" => {
            let paras = call.params.parse::<Option<MessageActionItem>>()?;
            Some(LspResponse::WindowShowMessageRequest(paras))
        }
        "client/registerCapability" => {
            Some(LspResponse::ClientRegisterCapability(()))
        }
        "client/unregisterCapability" => {
            Some(LspResponse::ClientUnregisterCapability(()))
        }
        "workspace/symbol" => {
            let paras = call.params.parse::<Option<Vec<SymbolInformation>>>()?;
            Some(LspResponse::WorkspaceSymbol(paras))
        }
        "workspace/executeCommand" => {
            let paras = call.params.parse::<Option<Value>>()?;
            Some(LspResponse::WorkspaceExecuteCommand(paras))
        }
        "textDocument/willSaveWaitUntil" => {
            let paras = call.params.parse::<Option<Vec<TextEdit>>>()?;
            Some(LspResponse::TextDocumentWillSaveWaitUntil(paras))
        }
        "textDocument/completion" => {
            let paras = call.params.parse::<Option<CompletionResponse>>()?;
            Some(LspResponse::TextDocumentCompletion(paras))
        }
        "completionItem/resolve" => {
            let paras = call.params.parse::<CompletionItem>()?;
            Some(LspResponse::CompletionItemResolve(paras))
        }
        "textDocument/hover" => {
            let paras = call.params.parse::<Option<Hover>>()?;
            Some(LspResponse::TextDocumentHover(paras))
        }
        "textDocument/signatureHelp" => {
            let paras = call.params.parse::<Option<SignatureHelp>>()?;
            Some(LspResponse::TextDocumentSignatureHelp(paras))
        }
        "textDocument/declaration" => {
            let paras = call.params.parse::<Option<GotoDeclarationResponse>>()?;
            Some(LspResponse::TextDocumentDeclaration(paras))
        }
        "textDocument/definition" => {
            let paras = call.params.parse::<Option<GotoDefinitionResponse>>()?;
            Some(LspResponse::TextDocumentDefinition(paras))
        }
        "textDocument/references" => {
            let paras = call.params.parse::<Option<Vec<Location>>>()?;
            Some(LspResponse::TextDocumentReferences(paras))
        }
        "textDocument/documentHighlight" => {
            let paras = call.params.parse::<Option<Vec<DocumentHighlight>>>()?;
            Some(LspResponse::TextDocumentDocumentHighlight(paras))
        }
        "textDocument/documentSymbol" => {
            let paras = call.params.parse::<Option<DocumentSymbolResponse>>()?;
            Some(LspResponse::TextDocumentDocumentSymbol(paras))
        }
        "textDocument/codeAction" => {
            let paras = call.params.parse::<Option<CodeActionResponse>>()?;
            Some(LspResponse::TextDocumentCodeAction(paras))
        }
        "textDocument/codeLens" => {
            let paras = call.params.parse::<Option<Vec<CodeLens>>>()?;
            Some(LspResponse::TextDocumentCodeLens(paras))
        }
        "codeLens/resolve" => {
            let paras = call.params.parse::<CodeLens>()?;
            Some(LspResponse::CodeLensResolve(paras))
        }
        "textDocument/documentLink" => {
            let paras = call.params.parse::<Option<Vec<DocumentLink>>>()?;
            Some(LspResponse::TextDocumentDocumentLink(paras))
        }
        "documentLink/resolve" => {
            let paras = call.params.parse::<DocumentLink>()?;
            Some(LspResponse::DocumentLinkResolve(paras))
        }
        "workspace/applyEdit" => {
            let paras = call.params.parse::<ApplyWorkspaceEditResponse>()?;
            Some(LspResponse::WorkspaceApplyEdit(paras))
        }
        "textDocument/rangeFormatting" => {
            let paras = call.params.parse::<Option<Vec<TextEdit>>>()?;
            Some(LspResponse::TextDocumentRangeFormatting(paras))
        }
        "textDocument/onTypeFormatting" => {
            let paras = call.params.parse::<Option<Vec<TextEdit>>>()?;
            Some(LspResponse::TextDocumentOnTypeFormatting(paras))
        }
        "textDocument/formatting" => {
            let paras = call.params.parse::<Option<Vec<TextEdit>>>()?;
            Some(LspResponse::TextDocumentFormatting(paras))
        }
        "textDocument/rename" => {
            let paras = call.params.parse::<Option<WorkspaceEdit>>()?;
            Some(LspResponse::TextDocumentRename(paras))
        }
        "textDocument/documentColor" => {
            let paras = call.params.parse::<Vec<ColorInformation>>()?;
            Some(LspResponse::TextDocumentDocumentColor(paras))
        }
        "textDocument/colorPresentation" => {
            let paras = call.params.parse::<Vec<ColorPresentation>>()?;
            Some(LspResponse::TextDocumentColorPresentation(paras))
        }
        "textDocument/foldingRange" => {
            let paras = call.params.parse::<Option<Vec<FoldingRange>>>()?;
            Some(LspResponse::TextDocumentFoldingRange(paras))
        }
        "textDocument/prepareRename" => {
            let paras = call.params.parse::<Option<PrepareRenameResponse>>()?;
            Some(LspResponse::TextDocumentPrepareRename(paras))
        }
        "textDocument/implementation" => {
            let paras = call.params.parse::<Option<GotoImplementationResponse>>()?;
            Some(LspResponse::TextDocumentImplementation(paras))
        }
        "textDocument/typeDefinition" => {
            let paras = call.params.parse::<Option<GotoTypeDefinitionResponse>>()?;
            Some(LspResponse::TextDocumentTypeDefinition(paras))
        }
        "textDocument/selectionRange" => {
            let paras = call.params.parse::<Option<Vec<SelectionRange>>>()?;
            Some(LspResponse::TextDocumentSelectionRange(paras))
        }
        "workspace/workspaceFolders" => {
            let paras = call.params.parse::<Option<Vec<WorkspaceFolder>>>()?;
            Some(LspResponse::WorkspaceWorkspaceFolders(paras))
        }
        "workspace/configuration" => {
            let paras = call.params.parse::<Vec<Value>>()?;
            Some(LspResponse::WorkspaceConfiguration(paras))
        }
        "window/workDoneProgress/create" => {
            Some(LspResponse::WindowWorkDoneProgressCreate(()))
        }
        "callHierarchy/incomingCalls" => {
            let paras = call.params.parse::<Option<Vec<CallHierarchyIncomingCall>>>()?;
            Some(LspResponse::CallHierarchyIncomingCall(paras))
        }
        "callHierarchy/outgoingCalls" => {
            let paras = call.params.parse::<Option<Vec<CallHierarchyOutgoingCall>>>()?;
            Some(LspResponse::CallHierarchyOutgoingCalls(paras))
        }
        "textDocument/moniker" => {
            let paras = call.params.parse::<Option<Vec<Moniker>>>()?;
            Some(LspResponse::TextDocumentMoniker(paras))
        }
        "textDocument/linkedEditingRange" => {
            let paras = call.params.parse::<Option<LinkedEditingRanges>>()?;
            Some(LspResponse::TextDocumentLinkedEditingRange(paras))
        }
        "textDocument/prepareCallHierarchy" => {
            let paras = call.params.parse::<Option<Vec<CallHierarchyItem>>>()?;
            Some(LspResponse::TextDocumentPrepareCallHierarchy(paras))
        }
        "textDocument/semanticTokens/full" => {
            let paras = call.params.parse::<Option<SemanticTokensResult>>()?;
            Some(LspResponse::TextDocumentSemanticTokensFull(paras))
        }
        "textDocument/semanticTokens/full/delta" => {
            let paras = call.params.parse::<Option<SemanticTokensFullDeltaResult>>()?;
            Some(LspResponse::TextDocumentSemanticTokensFullDelta(paras))
        }
        "textDocument/semanticTokens/range" => {
            let paras = call.params.parse::<Option<SemanticTokensRangeResult>>()?;
            Some(LspResponse::TextDocumentSemanticTokensRange(paras))
        }
        "workspace/willCreateFiles" => {
            let paras = call.params.parse::<Option<WorkspaceEdit>>()?;
            Some(LspResponse::WorkspaceWillCreateFiles(paras))
        }
        "workspace/willRenameFiles" => {
            let paras = call.params.parse::<Option<WorkspaceEdit>>()?;
            Some(LspResponse::WorkspaceWillRenameFiles(paras))
        }
        "workspace/willDeleteFiles" => {
            let paras = call.params.parse::<Option<WorkspaceEdit>>()?;
            Some(LspResponse::WorkspaceWillDeleteFiles(paras))
        }
        "workspace/semanticTokens/refresh" => {
            Some(LspResponse::WorkspaceSemanticTokensRefresh(()))
        }
        "workspace/codeLens/refresh" => {
            Some(LspResponse::WorkspaceCodeLensRefresh(()))
        }
        "codeAction/resolve" => {
            let paras = call.params.parse::<CodeAction>()?;
            Some(LspResponse::CodeActionResolve(paras))
        }
        "window/showDocument" => {
            let paras = call.params.parse::<ShowDocumentResult>()?;
            Some(LspResponse::WindowShowDocument(paras))
        }
        _ => None,
    })
}