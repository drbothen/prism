// S-1.03 (ported to S-1.08 worktree): Capability Resolution Engine
//
// Story: S-1.08 — prism-security: Feature Flags (P0 Core)
// Depends-on: S-1.01, S-1.03

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::PrismError;

// ─────────────────────────────────────────────────────────────
// CapabilityPath
// ─────────────────────────────────────────────────────────────

/// A dot-separated hierarchical capability identifier.
///
/// Examples: `"sensor.crowdstrike.containment"`, `"sensor.crowdstrike.read"`.
///
/// Invariants (enforced by `new()`):
/// - Non-empty.
/// - Each segment matches `[a-zA-Z0-9_]+`.
/// - At most 8 segments.
/// - Total string length ≤ 256 characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CapabilityPath(Arc<str>);

impl CapabilityPath {
    /// Construct and validate a `CapabilityPath` from a string slice.
    pub fn new(s: &str) -> Result<Self, PrismError> {
        if s.is_empty() {
            return Err(PrismError::InvalidCapabilityPath {
                reason: "capability path must not be empty".to_string(),
            });
        }
        if s.len() > 256 {
            return Err(PrismError::InvalidCapabilityPath {
                reason: format!("capability path too long: {} > 256 chars", s.len()),
            });
        }
        let segments: Vec<&str> = s.split('.').collect();
        if segments.len() > 8 {
            return Err(PrismError::InvalidCapabilityPath {
                reason: format!("too many segments: {} > 8", segments.len()),
            });
        }
        for seg in &segments {
            if seg.is_empty() {
                return Err(PrismError::InvalidCapabilityPath {
                    reason: "capability path segment must not be empty (consecutive dots)"
                        .to_string(),
                });
            }
            if !seg.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(PrismError::InvalidCapabilityPath {
                    reason: format!("invalid segment '{}': must match [a-zA-Z0-9_]+", seg),
                });
            }
        }
        Ok(CapabilityPath(Arc::from(s)))
    }

    /// Returns the parent path (all segments except the last), or `None` if
    /// the path has only one segment.
    pub fn parent(&self) -> Option<CapabilityPath> {
        let s = self.as_str();
        let pos = s.rfind('.')?;
        let parent_str = &s[..pos];
        // Parent is always valid if self was valid
        Some(CapabilityPath(Arc::from(parent_str)))
    }

    /// Returns `true` if `self` is a prefix of `other` (or equal to `other`).
    pub fn is_prefix_of(&self, other: &CapabilityPath) -> bool {
        let self_str = self.as_str();
        let other_str = other.as_str();
        if self_str == other_str {
            return true;
        }
        // self must be a proper prefix segment: other starts with self + "."
        other_str.starts_with(self_str) && other_str.as_bytes().get(self_str.len()) == Some(&b'.')
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

/// Audit record returned alongside every `is_allowed()` decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityExplanation {
    /// The final allow/deny decision.
    pub allowed: bool,

    /// The capability path whose entry determined the outcome.
    pub matched_path: Option<CapabilityPath>,

    /// The effect of the matched entry.
    pub effect: CapabilityEffect,

    /// Human-readable reason token. One of:
    /// - `"deny-by-default"`
    /// - `"explicit-allow"`
    /// - `"explicit-deny"`
    /// - `"parent-allow"`
    /// - `"parent-deny"`
    pub reason: &'static str,
}

// ─────────────────────────────────────────────────────────────
// ClientCapabilities
// ─────────────────────────────────────────────────────────────

/// The complete set of capability rules for a single client.
///
/// Backed by a `BTreeMap` for deterministic iteration order (required for
/// `E-FLAG-001` resolution trace and Kani reproducibility — BC-2.04.003).
#[derive(Clone, Debug, Default)]
pub struct ClientCapabilities {
    rules: BTreeMap<CapabilityPath, CapabilityEffect>,
}

impl ClientCapabilities {
    /// Construct an empty `ClientCapabilities` (deny-all by default).
    pub fn new() -> Self {
        ClientCapabilities {
            rules: BTreeMap::new(),
        }
    }

    /// Add or replace a capability rule.
    pub fn grant(&mut self, path: CapabilityPath, effect: CapabilityEffect) {
        self.rules.insert(path, effect);
    }

    /// Decide whether `path` is permitted, returning the decision and a full
    /// audit explanation.
    ///
    /// Resolution algorithm (BC-2.04.003 most-specific-path-wins):
    /// 1. Check exact match.
    /// 2. Walk up the path hierarchy from most-specific to least-specific.
    /// 3. First match wins.
    /// 4. If no match: deny-by-default.
    pub fn is_allowed(&self, path: &CapabilityPath) -> (bool, CapabilityExplanation) {
        // Walk from exact match up to least-specific ancestor.
        let mut current = Some(path.clone());
        while let Some(check_path) = current {
            if let Some(effect) = self.rules.get(&check_path) {
                let is_exact = check_path.as_str() == path.as_str();
                let reason = match (effect, is_exact) {
                    (CapabilityEffect::Allow, true) => "explicit-allow",
                    (CapabilityEffect::Deny, true) => "explicit-deny",
                    (CapabilityEffect::Allow, false) => "parent-allow",
                    (CapabilityEffect::Deny, false) => "parent-deny",
                };
                let allowed = *effect == CapabilityEffect::Allow;
                return (
                    allowed,
                    CapabilityExplanation {
                        allowed,
                        matched_path: Some(check_path),
                        effect: *effect,
                        reason,
                    },
                );
            }
            current = check_path.parent();
        }

        // Deny-by-default: no matching rule found.
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

    /// Return all capability rules in sorted (deterministic) order for display.
    pub fn capabilities_for_display(&self) -> Vec<(&CapabilityPath, &CapabilityEffect)> {
        self.rules.iter().collect()
    }
}

impl FromIterator<(CapabilityPath, CapabilityEffect)> for ClientCapabilities {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (CapabilityPath, CapabilityEffect)>,
    {
        ClientCapabilities {
            rules: iter.into_iter().collect(),
        }
    }
}
