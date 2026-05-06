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

    /// Construct a `ParseError` for an invalid ISO-8601 timestamp string.
    ///
    /// Used by `TimestampLiteral::new` when `chrono` cannot parse the input as
    /// RFC-3339 (a strict subset of ISO-8601). The `cause` string is appended
    /// verbatim so that analysts can see the underlying parse failure reason.
    pub fn invalid_timestamp(input: &str, cause: impl std::fmt::Display) -> Self {
        ParseError::new(
            0,
            format!(
                "E-QUERY-001: invalid ISO-8601 timestamp '{}': {}",
                input, cause
            ),
        )
    }

    /// Serialize this error to a JSON string for MCP tool responses.
    ///
    /// Returns a compact JSON representation; ariadne terminal formatting
    /// is intentionally NOT used here (MCP consumers need machine-readable
    /// output, not ANSI-colored output).
    pub fn to_json(&self) -> String {
        // Use serde_json for compact, machine-readable JSON.
        // unwrap_or_else: ParseError is always serializable (all fields are
        // String/usize/Option<String> which serde_json handles infallibly),
        // but we provide a fallback for safety.
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!(
                r#"{{"offset":{},"message":"serialization error"}}"#,
                self.offset
            )
        })
    }

    /// Format all errors in `errors` as a human-readable ariadne report,
    /// returned as a plain-text string (no ANSI escape codes).
    ///
    /// The `source` argument is the original query string, used by ariadne
    /// to produce source-annotated snippets.
    pub fn format_report(errors: &[ParseError], source: &str) -> String {
        use ariadne::{Config, Label, Report, ReportKind, Source};
        use std::fmt::Write as _;

        if errors.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        for err in errors {
            // Clamp offset to the source length so ariadne never panics on
            // invalid span positions (defensive guard).
            let src_len = source.len().max(1);
            let offset = err.offset.min(src_len - 1);

            let mut buf: Vec<u8> = Vec::new();
            let span = offset..(offset + 1).min(src_len);

            let report_result =
                Report::<(&str, std::ops::Range<usize>)>::build(ReportKind::Error, "query", offset)
                    .with_config(
                        Config::default().with_color(false), // no ANSI codes — MCP-safe plain text
                    )
                    .with_message(err.message.clone())
                    .with_label(Label::new(("query", span)).with_message(err.message.clone()))
                    .finish()
                    .write(("query", Source::from(source)), &mut buf);

            match report_result {
                Ok(()) => {
                    if let Ok(s) = std::str::from_utf8(&buf) {
                        let _ = write!(output, "{s}");
                    }
                }
                Err(_) => {
                    // Fallback: plain text representation.
                    let _ = writeln!(
                        output,
                        "parse error at offset {}: {}",
                        err.offset, err.message
                    );
                }
            }

            if let Some(ref label) = err.recovery_label {
                let _ = writeln!(output, "  recovery point: {label}");
            }
        }

        output
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at offset {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Truncate a user-supplied string for inclusion in error messages.
///
/// # Security (B-9, BC-2.11.006)
/// Error messages MUST NOT echo arbitrary user input verbatim — a 10KB CIDR
/// string would produce a 10KB+ error message. This helper truncates at
/// `max_bytes` bytes, appending `"…"` when truncation occurs.
///
/// Default `max_bytes` is 200.
///
/// # Example
/// ```
/// use prism_query::error::truncate_for_display;
/// assert_eq!(truncate_for_display("hello", 200), "hello");
/// let long = "x".repeat(300);
/// let truncated = truncate_for_display(&long, 200);
/// assert!(truncated.len() <= 204); // 200 + "…" (3 bytes UTF-8)
/// ```
pub fn truncate_for_display(s: &str, max_bytes: usize) -> std::borrow::Cow<'_, str> {
    if s.len() <= max_bytes {
        std::borrow::Cow::Borrowed(s)
    } else {
        // Truncate at a valid UTF-8 boundary.
        let mut end = max_bytes;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        std::borrow::Cow::Owned(format!("{}…", &s[..end]))
    }
}
