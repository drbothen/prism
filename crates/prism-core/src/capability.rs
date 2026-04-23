//! Capability Resolution Engine for the Prism platform.
//!
//! Provides hierarchical permission resolution with deny-by-default semantics,
//! longest-prefix-match (most-specific-wins), and explicit-deny precedence.
//!
//! # Types
//! - [`CapabilityPath`] — validated dot-separated capability identifier
//! - [`CapabilityEffect`] — `Allow` or `Deny`
//! - [`CapabilityExplanation`] — audit record returned with every decision
//! - [`ClientCapabilities`] — the full rule set for a single client
//!
//! # Resolution semantics
//! 1. **Deny-by-default** — empty map denies everything (VP-002).
//! 2. **Most-specific wins** — walk from the exact path upward through parents;
//!    the first (longest) matching entry wins (VP-003).
//! 3. **Last-write wins for same path** — `BTreeMap` stores one entry per key;
//!    calling `grant()` twice for the same path overwrites the first entry.
//!    It is therefore impossible to store both `Allow` and `Deny` for the same
//!    path simultaneously (VP-004).

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::PrismError;

// ─────────────────────────────────────────────────────────────
// CapabilityPath
// ─────────────────────────────────────────────────────────────

/// A dot-separated hierarchical capability identifier.
///
/// Examples: `"crowdstrike.hosts.write"`, `"audit.read"`,
/// `"detections.acknowledge"`.
///
/// # Invariants (enforced by [`CapabilityPath::new`])
/// - Non-empty string.
/// - Each dot-separated segment matches `[a-zA-Z0-9_]+` (non-empty, valid chars).
/// - At most 8 segments.
/// - Total string length ≤ 256 characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CapabilityPath(Arc<str>);

impl CapabilityPath {
    /// Construct and validate a `CapabilityPath` from a string slice.
    ///
    /// # Errors
    /// Returns [`PrismError::InvalidCapabilityPath`] if:
    /// - `s` is empty
    /// - Any segment is empty (consecutive dots, leading/trailing dot)
    /// - Any segment contains characters outside `[a-zA-Z0-9_]`
    /// - More than 8 segments
    /// - Total length exceeds 256 characters
    pub fn new(s: &str) -> Result<Self, PrismError> {
        // Length check first — fast rejection.
        if s.is_empty() {
            return Err(PrismError::InvalidCapabilityPath {
                reason: "capability path must not be empty".to_owned(),
            });
        }
        if s.len() > 256 {
            return Err(PrismError::InvalidCapabilityPath {
                reason: format!(
                    "capability path length {} exceeds maximum of 256 characters",
                    s.len()
                ),
            });
        }

        let segments: Vec<&str> = s.split('.').collect();

        if segments.len() > 8 {
            return Err(PrismError::InvalidCapabilityPath {
                reason: format!(
                    "capability path has {} segments; maximum is 8",
                    segments.len()
                ),
            });
        }

        for segment in &segments {
            if segment.is_empty() {
                return Err(PrismError::InvalidCapabilityPath {
                    reason:
                        "capability path segment must not be empty (check for consecutive dots)"
                            .to_owned(),
                });
            }
            for ch in segment.chars() {
                if !ch.is_ascii_alphanumeric() && ch != '_' {
                    return Err(PrismError::InvalidCapabilityPath {
                        reason: format!(
                            "capability path segment '{segment}' contains invalid character '{ch}'; \
                             only [a-zA-Z0-9_] is allowed"
                        ),
                    });
                }
            }
        }

        Ok(Self(Arc::from(s)))
    }

    /// Returns the parent path (all segments except the last), or `None` if
    /// the path has only one segment.
    ///
    /// # Examples
    /// - `"a.b.c".parent()` → `Some("a.b")`
    /// - `"a".parent()` → `None`
    pub fn parent(&self) -> Option<CapabilityPath> {
        // Find the last dot and slice everything before it.
        self.0.rfind('.').map(|pos| {
            // SAFETY: the inner string was validated at construction; the substring
            // up to the last dot is also a valid capability path.
            Self(Arc::from(&self.0[..pos]))
        })
    }

    /// Returns `true` if `self` is a prefix of `other` (segment-boundary-aware)
    /// or is equal to `other`.
    ///
    /// `"a.b"` is a prefix of `"a.b.c"` but NOT of `"a.bc"`.
    pub fn is_prefix_of(&self, other: &CapabilityPath) -> bool {
        if self.0.as_ref() == other.0.as_ref() {
            return true;
        }
        // `other` must start with `self` followed by a dot separator.
        other.0.starts_with(self.0.as_ref()) && other.0.as_bytes().get(self.0.len()) == Some(&b'.')
    }

    /// Returns the inner string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CapabilityPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ─────────────────────────────────────────────────────────────
// CapabilityEffect
// ─────────────────────────────────────────────────────────────

/// The effect associated with a capability rule: permit or deny.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityEffect {
    Allow,
    Deny,
}

// ─────────────────────────────────────────────────────────────
// CapabilityExplanation
// ─────────────────────────────────────────────────────────────

/// Audit record returned alongside every [`ClientCapabilities::is_allowed`] decision.
///
/// Callers MUST NOT re-invoke `is_allowed()` merely to obtain audit details —
/// this struct carries them in a single call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityExplanation {
    /// The final allow/deny decision.
    pub allowed: bool,

    /// The capability path whose entry determined the outcome, or `None` when
    /// the result comes from deny-by-default (no path matched at all).
    pub matched_path: Option<CapabilityPath>,

    /// The effect of the matched entry (or `Deny` for the default).
    pub effect: CapabilityEffect,

    /// Human-readable reason token.  One of:
    /// - `"deny-by-default"` — no path matched; default deny applies.
    /// - `"explicit-allow"` — an exact path entry with `Allow` matched.
    /// - `"explicit-deny"` — an exact path entry with `Deny` matched.
    /// - `"parent-allow"` — a parent/ancestor path with `Allow` matched.
    /// - `"parent-deny"` — a parent/ancestor path with `Deny` matched.
    pub reason: &'static str,
}

// ─────────────────────────────────────────────────────────────
// ClientCapabilities
// ─────────────────────────────────────────────────────────────

/// The complete set of capability rules for a single client.
///
/// Backed by a `BTreeMap` for deterministic iteration order (required for
/// [`ClientCapabilities::capabilities_for_display`] and Kani reproducibility).
///
/// # Construction
/// Build during configuration loading with [`ClientCapabilities::new`] +
/// [`ClientCapabilities::grant`], or use [`ClientCapabilities::from_iter`].
/// Once request handling begins, treat the value as immutable — do not call
/// `grant()` from a request handler.
///
/// # Resolution semantics
/// 1. **Deny-by-default** — empty map denies everything (VP-002).
/// 2. **Most-specific wins** — walk from the exact path upward; the first
///    (longest) matching prefix wins (VP-003).
/// 3. **Last-write wins for same path** — `BTreeMap` stores one entry per
///    key; calling `grant()` twice for the same path overwrites the first
///    entry.  The API therefore makes it impossible to store both `Allow` and
///    `Deny` for the same path simultaneously (VP-004).
#[derive(Clone, Debug, Default)]
pub struct ClientCapabilities {
    rules: BTreeMap<CapabilityPath, CapabilityEffect>,
}

impl ClientCapabilities {
    /// Construct an empty `ClientCapabilities` (deny-all by default).
    pub fn new() -> Self {
        Self {
            rules: BTreeMap::new(),
        }
    }

    /// Add or replace a capability rule.
    ///
    /// Calling `grant()` twice for the same path overwrites the first call
    /// (last-write wins — single BTreeMap entry per key).
    pub fn grant(&mut self, path: CapabilityPath, effect: CapabilityEffect) {
        self.rules.insert(path, effect);
    }

    /// Decide whether `path` is permitted, returning the decision and a full
    /// audit explanation.
    ///
    /// # Resolution algorithm
    /// Walk from the exact path upward through parent segments.  The first
    /// entry found in the rule map is the most-specific match and wins.
    /// If no entry matches at any ancestor level, deny-by-default applies.
    ///
    /// This is O(depth) per lookup — at most 8 BTreeMap lookups for an
    /// 8-segment path.
    pub fn is_allowed(&self, path: &CapabilityPath) -> (bool, CapabilityExplanation) {
        // Walk from exact path up through ancestors.
        let mut current: Option<CapabilityPath> = Some(path.clone());
        let mut is_exact = true;

        while let Some(candidate) = current {
            if let Some(&effect) = self.rules.get(&candidate) {
                let (allowed, reason) = match (effect, is_exact) {
                    (CapabilityEffect::Allow, true) => (true, "explicit-allow"),
                    (CapabilityEffect::Deny, true) => (false, "explicit-deny"),
                    (CapabilityEffect::Allow, false) => (true, "parent-allow"),
                    (CapabilityEffect::Deny, false) => (false, "parent-deny"),
                };
                let explanation = CapabilityExplanation {
                    allowed,
                    matched_path: Some(candidate),
                    effect,
                    reason,
                };
                return (allowed, explanation);
            }
            current = candidate.parent();
            is_exact = false;
        }

        // No rule matched at any level — deny by default.
        (
            false,
            CapabilityExplanation {
                allowed: false,
                matched_path: None,
                effect: CapabilityEffect::Deny,
                reason: "deny-by-default",
            },
        )
    }

    /// Return all capability rules in sorted (deterministic BTreeMap) order.
    ///
    /// Used by the MCP `list_capabilities` tool.
    pub fn capabilities_for_display(&self) -> Vec<(&CapabilityPath, &CapabilityEffect)> {
        self.rules.iter().collect()
    }
}

impl FromIterator<(CapabilityPath, CapabilityEffect)> for ClientCapabilities {
    /// Build a `ClientCapabilities` from an iterator of `(path, effect)` pairs.
    ///
    /// Duplicate paths: the last entry wins (same as sequential `grant()` calls).
    fn from_iter<I: IntoIterator<Item = (CapabilityPath, CapabilityEffect)>>(iter: I) -> Self {
        let mut caps = Self::new();
        for (path, effect) in iter {
            caps.grant(path, effect);
        }
        caps
    }
}
