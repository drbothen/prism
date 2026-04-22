//! InjectionScanner — suspicious pattern detection with NFKC normalization (BC-2.09.003).
//!
//! # Design
//!
//! - Regex patterns compiled ONCE via `OnceLock<Vec<(Regex, PatternCategory, &str)>>`.
//! - NFKC normalization applied before every pattern match (EC-09-011 homoglyph defense).
//! - Base64 heuristic: strings >20 chars matching `[A-Za-z0-9+/=]{20,}` are decoded
//!   and re-scanned for injection markers (EC-09-003).
//! - Original field values are NEVER modified (flag-don't-strip principle, BC-2.09.004).
//! - `_meta.safety_flags` is additive; all detections appended (EC-09-008).
//! - Scan limit: 10KB per field; beyond limit a `TruncatedScan` flag is added (EC-09-006).
//!
//! # Flag-Don't-Strip Principle
//!
//! The scanner MUST flag detected patterns and return the original value unchanged.
//! Stripping data could mask attacker activity from the operator and cause silent
//! data loss. The LLM consumer is expected to handle flagged data with appropriate
//! skepticism. This principle MUST NOT be reversed.

use std::sync::OnceLock;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use prism_core::{PatternCategory, SafetyFlag};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Maximum number of bytes scanned per string field (BC-2.09.003 EC-09-006).
pub const SCAN_LIMIT_BYTES: usize = 10_240;

/// Compiled pattern entry: regex, category, human-readable description.
struct PatternEntry {
    regex: Regex,
    category: PatternCategory,
    description: &'static str,
}

/// Base64 detection regex — matches strings that look like base64 payloads.
static BASE64_REGEX: OnceLock<Regex> = OnceLock::new();

fn base64_regex() -> &'static Regex {
    BASE64_REGEX
        .get_or_init(|| Regex::new(r"[A-Za-z0-9+/=]{20,}").expect("base64 regex must compile"))
}

/// Compiled injection patterns — initialized once on first use.
static PATTERNS: OnceLock<Vec<PatternEntry>> = OnceLock::new();

fn patterns() -> &'static Vec<PatternEntry> {
    PATTERNS.get_or_init(|| {
        // All patterns are case-insensitive via (?i) flag.
        // Order: more specific patterns first to ensure correct category assignment.
        let defs: &[(&str, PatternCategory, &str)] = &[
            // PromptInjection — ignore/forget/disregard previous/prior instructions/context
            (
                r"(?i)ignore\s+(all\s+)?previous\s+instructions",
                PatternCategory::PromptInjection,
                "ignore previous instructions",
            ),
            (
                r"(?i)ignore\s+(all\s+)?prior\s+instructions",
                PatternCategory::PromptInjection,
                "ignore prior instructions",
            ),
            (
                r"(?i)forget\s+(all\s+)?previous\s+context",
                PatternCategory::PromptInjection,
                "forget previous context",
            ),
            (
                r"(?i)disregard\s+(above|all|previous|prior)\s+instructions",
                PatternCategory::PromptInjection,
                "disregard above instructions",
            ),
            // RoleImpersonation — SYSTEM:, ASSISTANT:, Human:, Claude:
            (
                r"(?i)SYSTEM\s*:",
                PatternCategory::RoleImpersonation,
                "SYSTEM: role prefix",
            ),
            (
                r"(?i)ASSISTANT\s*:",
                PatternCategory::RoleImpersonation,
                "ASSISTANT: role prefix",
            ),
            (
                r"(?i)Human\s*:",
                PatternCategory::RoleImpersonation,
                "Human: role prefix",
            ),
            (
                r"(?i)Claude\s*:",
                PatternCategory::RoleImpersonation,
                "Claude: role prefix",
            ),
            // XmlContextEscape — <system>, <instructions>, <tool_result>
            (
                r"(?i)<\s*system\s*>",
                PatternCategory::XmlContextEscape,
                "<system> XML tag",
            ),
            (
                r"(?i)<\s*instructions\s*>",
                PatternCategory::XmlContextEscape,
                "<instructions> XML tag",
            ),
            (
                r"(?i)<\s*tool_result\s*>",
                PatternCategory::XmlContextEscape,
                "<tool_result> XML tag",
            ),
            // CodeFenceEscape — triple backticks
            (
                r"```",
                PatternCategory::CodeFenceEscape,
                "triple backtick code fence",
            ),
        ];

        defs.iter()
            .map(|(pattern, category, description)| PatternEntry {
                regex: Regex::new(pattern)
                    .unwrap_or_else(|e| panic!("pattern '{pattern}' must compile: {e}")),
                category: category.clone(),
                description,
            })
            .collect()
    })
}

/// Input to `InjectionScanner::scan`.
///
/// Wraps a field name, item index (for the results array), and the raw string
/// value to scan.
pub struct ScanInput<'a> {
    /// Name of the field in the sensor record (e.g., `"hostname"`).
    pub field: &'a str,
    /// Zero-based index of the item in the results array.
    pub index: usize,
    /// Raw string value from the sensor record.
    pub value: &'a str,
}

/// Result of scanning one field value.
///
/// BC-2.09.004: flags are centralized in `_meta.safety_flags`; original value
/// is preserved unchanged.
pub struct ScanResult {
    /// All safety flags detected for this field value.
    /// Empty when no patterns matched.
    pub flags: Vec<SafetyFlag>,
    /// The original, unmodified field value (flag-don't-strip principle).
    pub original_value: String,
}

/// Injection scanner implementing BC-2.09.003 and BC-2.09.004.
///
/// The regex patterns are compiled once via `OnceLock` at first use.
/// The scanner is stateless after initialization — `scan()` takes `&self`.
pub struct InjectionScanner;

/// Singleton scanner instance.
static SCANNER: OnceLock<InjectionScanner> = OnceLock::new();

impl InjectionScanner {
    /// Returns the singleton scanner instance.
    ///
    /// Regex patterns are compiled once on first call via `OnceLock`.
    /// Subsequent calls return the same instance with zero overhead.
    pub fn global() -> &'static InjectionScanner {
        SCANNER.get_or_init(|| {
            // Initialize both pattern sets eagerly at first call.
            let _ = patterns();
            let _ = base64_regex();
            InjectionScanner
        })
    }

    /// Scan a single field value for injection patterns.
    ///
    /// ## Procedure (BC-2.09.003)
    /// 1. Apply NFKC normalization to `input.value`.
    /// 2. If value exceeds `SCAN_LIMIT_BYTES`, truncate for scanning only;
    ///    original is preserved; a `TruncatedScan` flag is emitted.
    /// 3. Match normalized string against all compiled regex patterns.
    /// 4. Apply base64 heuristic on strings matching `[A-Za-z0-9+/=]{20,}`.
    /// 5. Append one `SafetyFlag` per pattern match to result.
    /// 6. Return `ScanResult` with flags and original (unmodified) value.
    ///
    /// This method NEVER panics (VP-038).
    pub fn scan<'a>(&self, input: ScanInput<'a>) -> ScanResult {
        let original_value = input.value.to_owned();
        let mut flags = Vec::new();

        // Step 1: NFKC normalization
        let normalized: String = input.value.nfkc().collect();

        // Step 2: Scan limit — truncate for scanning only
        let truncated = normalized.len() > SCAN_LIMIT_BYTES;
        let scan_target = if truncated {
            // Truncate at a char boundary to avoid panicking on UTF-8
            truncate_to_char_boundary(&normalized, SCAN_LIMIT_BYTES)
        } else {
            normalized.as_str()
        };

        if truncated {
            flags.push(SafetyFlag::new(
                input.field,
                input.index,
                "field exceeded scan limit (10KB); partially scanned",
                PatternCategory::TruncatedScan,
            ));
        }

        // Step 3: Match against all compiled patterns
        for entry in patterns() {
            if entry.regex.is_match(scan_target) {
                flags.push(SafetyFlag::new(
                    input.field,
                    input.index,
                    entry.description,
                    entry.category.clone(),
                ));
            }
        }

        // Step 4: Base64 heuristic
        // Find substrings matching base64 pattern, decode, re-scan
        for mat in base64_regex().find_iter(scan_target) {
            let candidate = mat.as_str();
            if let Ok(decoded_bytes) = B64.decode(candidate) {
                if let Ok(decoded_str) = std::str::from_utf8(&decoded_bytes) {
                    // Re-scan decoded content against injection patterns
                    let b64_normalized: String = decoded_str.nfkc().collect();
                    for entry in patterns() {
                        if entry.regex.is_match(&b64_normalized) {
                            flags.push(SafetyFlag::new(
                                input.field,
                                input.index,
                                entry.description,
                                PatternCategory::Base64Encoded,
                            ));
                            // One flag per base64 candidate that decodes to injection
                            break;
                        }
                    }
                }
            }
        }

        ScanResult {
            flags,
            original_value,
        }
    }

    /// Scan a byte slice — used by the fuzz target (VP-038).
    ///
    /// Invalid UTF-8 is handled gracefully (lossy decode). Never panics.
    pub fn scan_bytes(&self, field: &str, index: usize, data: &[u8]) -> ScanResult {
        let s = String::from_utf8_lossy(data);
        self.scan(ScanInput {
            field,
            index,
            value: &s,
        })
    }

    /// Scan all string fields in a sensor record.
    ///
    /// Returns a flat list of all `SafetyFlag`s found across all fields.
    /// Fields are provided as `(field_name, item_index, value)` triples.
    pub fn scan_record(&self, fields: &[(&str, usize, &str)]) -> Vec<SafetyFlag> {
        let mut all_flags = Vec::new();
        for &(field, index, value) in fields {
            let result = self.scan(ScanInput {
                field,
                index,
                value,
            });
            all_flags.extend(result.flags);
        }
        all_flags
    }
}

/// Truncate `s` to at most `limit` bytes at a valid UTF-8 char boundary.
fn truncate_to_char_boundary(s: &str, limit: usize) -> &str {
    if s.len() <= limit {
        return s;
    }
    let mut boundary = limit;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &s[..boundary]
}
