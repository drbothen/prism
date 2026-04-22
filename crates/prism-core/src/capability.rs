// S-1.03: Capability Resolution Engine — STUB (Red Gate)
//
// All function bodies are `unimplemented!()`.  The implementer must fill them
// in to make the test suite green.
//
// Story: S-1.03 — prism-core: Capability Resolution Engine
// VPs:   VP-002 (deny-by-default), VP-003 (most-specific-wins),
//        VP-004 (deny-overrides-allow / exact-match explanation)
//
// Architecture compliance rules (from story spec):
//   - `ClientCapabilities` MUST be immutable after construction for thread safety.
//   - Path resolution MUST be O(depth) per lookup, not O(n) over all capabilities.
//   - `CapabilityExplanation` MUST be returned with every `is_allowed()` call.
//   - The capability engine MUST NOT have any I/O dependencies (pure computation).

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
/// Invariants (enforced by `new()`):
/// - Non-empty.
/// - Each segment matches `[a-zA-Z0-9_]+`.
/// - At most 8 segments.
/// - Total string length ≤ 256 characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityPath(Arc<str>);

impl CapabilityPath {
    /// Construct and validate a `CapabilityPath` from a string slice.
    ///
    /// Returns `Err(PrismError::InvalidCapabilityPath)` if validation fails.
    pub fn new(_s: &str) -> Result<Self, PrismError> {
        unimplemented!("S-1.03: CapabilityPath::new — implement validation")
    }

    /// Returns the parent path (all segments except the last), or `None` if
    /// the path has only one segment.
    ///
    /// # Examples
    /// - `"a.b.c".parent()` → `Some("a.b")`
    /// - `"a".parent()` → `None`
    pub fn parent(&self) -> Option<CapabilityPath> {
        unimplemented!("S-1.03: CapabilityPath::parent — implement segment stripping")
    }

    /// Returns `true` if `self` is a prefix of `other` (or equal to `other`).
    ///
    /// `"a.b"` is a prefix of `"a.b.c"` but not of `"a.bc"`.
    pub fn is_prefix_of(&self, _other: &CapabilityPath) -> bool {
        unimplemented!("S-1.03: CapabilityPath::is_prefix_of — implement prefix check")
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
///
/// Used by audit logging and MCP error responses.  Callers must not re-invoke
/// `is_allowed()` merely to obtain audit details — this struct carries them.
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
/// `capabilities_for_display()` and Kani reproducibility).
///
/// # Construction
/// Build during configuration loading with `new()` + `grant()`.
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
        unimplemented!("S-1.03: ClientCapabilities::new — implement empty construction")
    }

    /// Build from an iterator of `(CapabilityPath, CapabilityEffect)` pairs.
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (CapabilityPath, CapabilityEffect)>,
    {
        unimplemented!(
            "S-1.03: ClientCapabilities::from_iter — implement from-iterator construction"
        )
    }

    /// Add or replace a capability rule.
    ///
    /// Calling `grant()` twice for the same path overwrites the first call
    /// (last-write wins — single BTreeMap entry per key).
    pub fn grant(&mut self, _path: CapabilityPath, _effect: CapabilityEffect) {
        unimplemented!("S-1.03: ClientCapabilities::grant — implement rule insertion")
    }

    /// Decide whether `path` is permitted, returning the decision and a full
    /// audit explanation.
    ///
    /// Resolution order: exact path → parent → grandparent → … → deny-by-default.
    /// The first matching entry (most specific) determines the outcome (VP-003).
    pub fn is_allowed(&self, _path: &CapabilityPath) -> (bool, CapabilityExplanation) {
        unimplemented!("S-1.03: ClientCapabilities::is_allowed — implement resolution walk")
    }

    /// Return all capability rules in sorted (deterministic) order for display.
    ///
    /// Used by the MCP `list_capabilities` tool.
    pub fn capabilities_for_display(&self) -> Vec<(&CapabilityPath, &CapabilityEffect)> {
        unimplemented!(
            "S-1.03: ClientCapabilities::capabilities_for_display — implement sorted display"
        )
    }
}
