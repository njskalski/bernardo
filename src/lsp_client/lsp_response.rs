use lsp_types::{ApplyWorkspaceEditResponse, CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall, CodeAction, CodeActionResponse, CodeLens, ColorInformation, ColorPresentation, CompletionItem, CompletionResponse, DocumentHighlight, DocumentLink, DocumentSymbolResponse, FoldingRange, GotoDefinitionResponse, Hover, InitializeResult, LinkedEditingRanges, Location, MessageActionItem, Moniker, PrepareRenameResponse, SelectionRange, SemanticTokensFullDeltaResult, SemanticTokensRangeResult, SemanticTokensResult, ShowDocumentResult, SignatureHelp, SymbolInformation, TextEdit, WorkspaceEdit, WorkspaceFolder};
use lsp_types::request::{GotoDeclarationResponse, GotoImplementationResponse, GotoTypeDefinitionResponse};
use serde_json::Value;

#[derive(Debug)]
pub enum LspResponse {
    Initialize(InitializeResult),
    Shutdown(()),
    WindowShowMessageRequest(Option<MessageActionItem>),
    ClientRegisterCapability(()),
    ClientUnregisterCapability(()),
    WorkspaceSymbol(Option<Vec<SymbolInformation>>),
    WorkspaceExecuteCommand(Option<Value>),
    TextDocumentWillSaveWaitUntil(Option<Vec<TextEdit>>),
    TextDocumentCompletion(Option<CompletionResponse>),
    CompletionItemResolve(CompletionItem),
    TextDocumentHover(Option<Hover>),
    TextDocumentSignatureHelp(Option<SignatureHelp>),
    TextDocumentDeclaration(Option<GotoDeclarationResponse>),
    TextDocumentDefinition(Option<GotoDefinitionResponse>),
    TextDocumentReferences(Option<Vec<Location>>),
    TextDocumentDocumentHighlight(Option<Vec<DocumentHighlight>>),
    TextDocumentDocumentSymbol(Option<DocumentSymbolResponse>),
    TextDocumentCodeAction(Option<CodeActionResponse>),
    TextDocumentCodeLens(Option<Vec<CodeLens>>),
    CodeLensResolve(CodeLens),
    TextDocumentDocumentLink(Option<Vec<DocumentLink>>),
    DocumentLinkResolve(DocumentLink),
    WorkspaceApplyEdit(ApplyWorkspaceEditResponse),
    TextDocumentRangeFormatting(Option<Vec<TextEdit>>),
    TextDocumentOnTypeFormatting(Option<Vec<TextEdit>>),
    TextDocumentFormatting(Option<Vec<TextEdit>>),
    TextDocumentRename(Option<WorkspaceEdit>),
    TextDocumentDocumentColor(Vec<ColorInformation>),
    TextDocumentColorPresentation(Vec<ColorPresentation>),
    TextDocumentFoldingRange(Option<Vec<FoldingRange>>),
    TextDocumentPrepareRename(Option<PrepareRenameResponse>),
    TextDocumentImplementation(Option<GotoImplementationResponse>),
    TextDocumentTypeDefinition(Option<GotoTypeDefinitionResponse>),
    TextDocumentSelectionRange(Option<Vec<SelectionRange>>),
    WorkspaceWorkspaceFolders(Option<Vec<WorkspaceFolder>>),
    WorkspaceConfiguration(Vec<Value>),
    WindowWorkDoneProgressCreate(()),
    CallHierarchyIncomingCall(Option<Vec<CallHierarchyIncomingCall>>),
    CallHierarchyOutgoingCalls(Option<Vec<CallHierarchyOutgoingCall>>),
    TextDocumentMoniker(Option<Vec<Moniker>>),
    TextDocumentLinkedEditingRange(Option<LinkedEditingRanges>),
    TextDocumentPrepareCallHierarchy(Option<Vec<CallHierarchyItem>>),
    TextDocumentSemanticTokensFull(Option<SemanticTokensResult>),
    TextDocumentSemanticTokensFullDelta(Option<SemanticTokensFullDeltaResult>),
    TextDocumentSemanticTokensRange(Option<SemanticTokensRangeResult>),
    WorkspaceWillCreateFiles(Option<WorkspaceEdit>),
    WorkspaceWillRenameFiles(Option<WorkspaceEdit>),
    WorkspaceWillDeleteFiles(Option<WorkspaceEdit>),
    WorkspaceSemanticTokensRefresh(()),
    WorkspaceCodeLensRefresh(()),
    CodeActionResolve(CodeAction),
    WindowShowDocument(ShowDocumentResult),
}