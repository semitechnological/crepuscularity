use crepuscularity_core::{diagnose_crepus_source, CrepusDiagnostic};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn crepus_diagnostics_to_lsp(source: &str) -> Vec<Diagnostic> {
    diagnose_crepus_source(source)
        .into_iter()
        .map(crepus_diagnostic_to_lsp)
        .collect()
}

pub fn crepus_diagnostic_to_lsp(d: CrepusDiagnostic) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line: d.start_line,
                character: d.start_character,
            },
            end: Position {
                line: d.end_line,
                character: d.end_character,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: d.message,
        ..Diagnostic::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsx_error_becomes_diagnostic_range() {
        let source = "<div ";
        let ds = crepus_diagnostics_to_lsp(source);
        assert!(!ds.is_empty());
        assert!(!ds[0].message.is_empty());
    }
}
