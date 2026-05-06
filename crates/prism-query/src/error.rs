//! `ParseError` — PrismQL parse error type and ariadne-based formatting.
//!
//! Errors from Chumsky are collected and wrapped into `ParseError` for
//! consumption by callers. JSON serialization is provided so that MCP tool
//! responses receive structured errors rather than ANSI-colored terminal
//! output.
//!
//! Story: S-3.01
//! S-3.06 | BC-2.11.004 — write-specific error codes (E-QUERY-010, E-QUERY-022, E-QUERY-023, E-QUERY-024)

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

// ─────────────────────────────────────────────────────────────────────────────
// S-3.06 write-specific error code constructors (BC-2.11.004)
// ─────────────────────────────────────────────────────────────────────────────

/// Error code string for internal-table write-protection violation.
///
/// Emitted at parse time when SQL DML targets a `prism_*` table.
/// Also emitted at execution time (same code, different layer) — S-3.06 covers
/// the parse-time emission; execution-time emission is S-3.07's scope.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub const E_QUERY_010: &str = "E-QUERY-010";

/// Error code string for an unbounded write (no WHERE / LIMIT guard).
///
/// Emitted at parse time when `DELETE FROM` or `UPDATE` lacks a WHERE clause,
/// or when `INSERT INTO … SELECT` lacks a LIMIT or WHERE on the source SELECT.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub const E_QUERY_022: &str = "E-QUERY-022";

/// Error code string for an unknown write verb in terminal pipe position.
///
/// Emitted when an identifier in terminal pipe position is neither a known
/// pipe stage keyword nor a registered write verb. The error message includes
/// a suggestion list from `WriteVerbRegistry::verbs_for_sensor`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub const E_QUERY_023: &str = "E-QUERY-023";

/// Error code string for a write stage appearing in non-terminal pipe position.
///
/// Emitted when a write verb appears mid-pipeline (followed by another `|` stage).
/// Write stages must be the final stage in a pipeline.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub const E_QUERY_024: &str = "E-QUERY-024";

impl ParseError {
    /// Construct an `E-QUERY-010` error for an internal-table write attempt.
    ///
    /// Message: "Internal Prism table 'prism_TABLE' is write-protected; use
    /// the dedicated MCP tool for this operation"
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn internal_table_write_protected(offset: usize, table_name: &str) -> Self {
        ParseError::new(
            offset,
            format!(
                "{E_QUERY_010}: Internal Prism table '{table_name}' is write-protected; \
                 use the dedicated MCP tool for this operation"
            ),
        )
    }

    /// Construct an `E-QUERY-022` error for an unbounded write.
    ///
    /// Message includes a suggestion to add a WHERE clause or LIMIT.
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn unbounded_write(offset: usize, operation: &str) -> Self {
        ParseError::new(
            offset,
            format!(
                "{E_QUERY_022}: unbounded {operation} rejected — add a WHERE clause \
                 (or LIMIT for INSERT...SELECT) to scope the operation, \
                 or use explicit opt-in if provided by the sensor spec"
            ),
        )
    }

    /// Construct an `E-QUERY-023` error for an unknown verb in terminal pipe position.
    ///
    /// `available_verbs` is the suggestion list from `WriteVerbRegistry::verbs_for_sensor`
    /// for the source sensor.
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn unknown_write_verb(offset: usize, verb: &str, available_verbs: &[&str]) -> Self {
        let suggestion = if available_verbs.is_empty() {
            "no write verbs are registered for this sensor".to_string()
        } else {
            format!("available verbs: {}", available_verbs.join(", "))
        };
        ParseError::new(
            offset,
            format!("{E_QUERY_023}: unknown write verb '{verb}' — {suggestion}"),
        )
    }

    /// Construct an `E-QUERY-024` error for a write stage in non-terminal position.
    ///
    /// `verb` is the write verb that appeared mid-pipeline; `position` is the
    /// zero-indexed pipe stage position where it was detected.
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension
    pub fn write_stage_not_terminal(offset: usize, verb: &str, position: usize) -> Self {
        ParseError::new(
            offset,
            format!(
                "{E_QUERY_024}: write stage must be in terminal pipe position — \
                 '{verb}' at position {position} is followed by additional stages"
            ),
        )
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
