//! `ParseError` — PrismQL parse error type and ariadne-based formatting.
//!
//! Errors from Chumsky are collected and wrapped into `ParseError` for
//! consumption by callers. JSON serialization is provided so that MCP tool
//! responses receive structured errors rather than ANSI-colored terminal
//! output.
//!
//! Story: S-3.01

use serde::{Deserialize, Serialize};

/// A single PrismQL parse error returned alongside a partial AST.
///
/// Multiple errors may be returned in a single parse attempt when Chumsky's
/// error-recovery strategies are active (S-3.01 §error_recovery).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseError {
    /// Byte offset in the input string where the error was detected.
    pub offset: usize,
    /// Human-readable error message.
    pub message: String,
    /// Optional label identifying the recovery point (e.g. `"after 'WHERE'"`).
    pub recovery_label: Option<String>,
}

impl ParseError {
    /// Construct a new `ParseError` with a message.
    pub fn new(offset: usize, message: impl Into<String>) -> Self {
        ParseError {
            offset,
            message: message.into(),
            recovery_label: None,
        }
    }

    /// Attach a recovery label to this error.
    pub fn with_recovery_label(mut self, label: impl Into<String>) -> Self {
        self.recovery_label = Some(label.into());
        self
    }

    /// Serialize this error to a JSON string for MCP tool responses.
    ///
    /// Returns a compact JSON representation; ariadne terminal formatting
    /// is intentionally NOT used here (MCP consumers need machine-readable
    /// output, not ANSI-colored output).
    pub fn to_json(&self) -> String {
        todo!("S-3.01: implement JSON serialization of ParseError via serde_json")
    }

    /// Format all errors in `errors` as a human-readable ariadne report,
    /// returned as a plain-text string (no ANSI escape codes).
    ///
    /// The `source` argument is the original query string, used by ariadne
    /// to produce source-annotated snippets.
    pub fn format_report(errors: &[ParseError], source: &str) -> String {
        todo!(
            "S-3.01: implement ariadne Report construction from ParseError slice; \
             source len={} errors={}",
            source.len(),
            errors.len()
        )
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at offset {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}
