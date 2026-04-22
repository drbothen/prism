// S-1.03 (ported to S-1.08 worktree): Capability Resolution Engine — STUB
//
// All function bodies are `unimplemented!()`.  The implementer must fill them
// in to make the test suite green.
//
// This file is the local stub for prism-core/src/capability.rs used by
// prism-security in the S-1.08 worktree. Mirrors the S-1.03 Red Gate stub
// exactly — S-1.08 tests depend on S-1.03 types but S-1.03 is not yet
// implemented. The implementer for S-1.08 brings in the S-1.03 implementation.
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
    pub fn new(_s: &str) -> Result<Self, PrismError> {
        unimplemented!("S-1.03/S-1.08: CapabilityPath::new — implement validation")
    }

    /// Returns the parent path (all segments except the last), or `None` if
    /// the path has only one segment.
    pub fn parent(&self) -> Option<CapabilityPath> {
        unimplemented!("S-1.03/S-1.08: CapabilityPath::parent — implement segment stripping")
    }

    /// Returns `true` if `self` is a prefix of `other` (or equal to `other`).
    pub fn is_prefix_of(&self, _other: &CapabilityPath) -> bool {
        unimplemented!("S-1.03/S-1.08: CapabilityPath::is_prefix_of — implement prefix check")
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
        unimplemented!("S-1.03/S-1.08: ClientCapabilities::new — implement empty construction")
    }

    /// Build from an iterator of `(CapabilityPath, CapabilityEffect)` pairs.
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (CapabilityPath, CapabilityEffect)>,
    {
        unimplemented!(
            "S-1.03/S-1.08: ClientCapabilities::from_iter — implement from-iterator construction"
        )
    }

    /// Add or replace a capability rule.
    pub fn grant(&mut self, _path: CapabilityPath, _effect: CapabilityEffect) {
        unimplemented!("S-1.03/S-1.08: ClientCapabilities::grant — implement rule insertion")
    }

    /// Decide whether `path` is permitted, returning the decision and a full
    /// audit explanation.
    pub fn is_allowed(&self, _path: &CapabilityPath) -> (bool, CapabilityExplanation) {
        unimplemented!("S-1.03/S-1.08: ClientCapabilities::is_allowed — implement resolution walk")
    }

    /// Return all capability rules in sorted (deterministic) order for display.
    pub fn capabilities_for_display(&self) -> Vec<(&CapabilityPath, &CapabilityEffect)> {
        unimplemented!(
            "S-1.03/S-1.08: ClientCapabilities::capabilities_for_display — implement sorted display"
        )
    }
}
