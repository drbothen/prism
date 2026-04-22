//! SafetyFlag — structured detection record for `_meta.safety_flags` (BC-2.09.004).
//!
//! Flags are centralized in `_meta.safety_flags`; original data is NEVER modified
//! (flag-don't-strip principle per BC-2.09.004 invariant).

use serde::{Deserialize, Serialize};

/// Category of suspicious pattern matched by the injection scanner.
///
/// BC-2.09.003: categories distinguish the type of threat detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternCategory {
    /// Prompt injection: ignore/forget/disregard + previous/prior + instructions/context.
    PromptInjection,
    /// Role impersonation: `SYSTEM:`, `ASSISTANT:`, `Human:`, `Claude:`.
    RoleImpersonation,
    /// XML context-escape: `<system>`, `<instructions>`, `<tool_result>`.
    XmlContextEscape,
    /// Code fence sequences that may break context framing (triple backticks).
    CodeFenceEscape,
    /// Base64-encoded payload that decodes to injection content.
    Base64Encoded,
    /// Field value exceeded scan length limit; only partially scanned.
    TruncatedScan,
}

/// One entry in `_meta.safety_flags`.
///
/// BC-2.09.004: centralized, additive — original data never modified.
/// Shape: `{"field": "hostname", "index": 0, "pattern": "...", "category": "prompt_injection"}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyFlag {
    /// Name of the sensor record field that triggered the detection.
    pub field: String,
    /// Index of the item in the results array (0-based).
    pub index: usize,
    /// Human-readable description of the matched pattern.
    pub pattern: String,
    /// Category of the matched pattern.
    pub category: PatternCategory,
}

impl SafetyFlag {
    /// Construct a new `SafetyFlag`.
    pub fn new(
        field: impl Into<String>,
        index: usize,
        pattern: impl Into<String>,
        category: PatternCategory,
    ) -> Self {
        SafetyFlag {
            field: field.into(),
            index,
            pattern: pattern.into(),
            category,
        }
    }
}
