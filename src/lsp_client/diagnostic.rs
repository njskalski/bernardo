pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

impl From<lsp_types::DiagnosticSeverity> for DiagnosticSeverity {
    fn from(value: lsp_types::DiagnosticSeverity) -> Self {
        match value {
            lsp_types::DiagnosticSeverity::ERROR => DiagnosticSeverity::Error,
            lsp_types::DiagnosticSeverity::WARNING => DiagnosticSeverity::Warning,
            lsp_types::DiagnosticSeverity::INFORMATION => DiagnosticSeverity::Info,
            _ => DiagnosticSeverity::Info,
        }
    }
}

pub struct Diagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub line_no_1b: usize,
}

impl From<lsp_types::Diagnostic> for Diagnostic {
    fn from(diag: lsp_types::Diagnostic) -> Self {
        Diagnostic {
            message: diag.message,
            severity: diag.severity.map(DiagnosticSeverity::from).unwrap_or(DiagnosticSeverity::Info),
            line_no_1b: (diag.range.start.line + 1) as usize,
        }
    }
}
