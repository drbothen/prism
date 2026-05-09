// S-1.09: RiskTier — Three-Tier Risk Classification
//
// Story:  S-1.09 — prism-security: Confirmation Tokens (P1)
// BC:     BC-2.04.007 — Three-Tier Risk Classification for Operations
// BC:     BC-2.04.008 — Dry-Run Default for Reversible Write Operations
//
// Architecture compliance rules:
//   - Every MCP tool is assigned EXACTLY ONE risk tier at registration time.
//   - Risk tier CANNOT change at runtime.
//   - Risk tier determines the gating mechanism; there is no bypass.
//   - Destructive operations (delete sensor, wipe endpoint) are NOT exposed via
//     MCP at all — they are never registered, not classified as Irreversible.
//   - Ambiguous operations are conservatively classified as Irreversible
//     (BC-2.04.007 invariant).
//   - `RiskTier::Read` operations MUST never require a confirmation token and
//     MUST never trigger a dry-run gate.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────
// RiskTier enum (BC-2.04.007)
// ─────────────────────────────────────────────────────────────

/// Three-tier risk classification for MCP tool operations.
///
/// Assigned at tool registration time; immutable at runtime.
///
/// | Tier | Gate Mechanism | Example tools |
/// |------|---------------|---------------|
/// | `Read` | None — executes immediately | `list_alerts`, `get_device_info` |
/// | `Reversible` | Dry-run default (`dry_run: true`) | `acknowledge_alert`, `add_tag` |
/// | `Irreversible` | Confirmation token required | `contain_host`, `quarantine_file` |
// PRE-EXISTING-TD: two RiskTier enums coexist — this one (prism-security::RiskTier,
// Read | Reversible | Irreversible) and prism-core::RiskTier (Reversible | Irreversible).
// This crate's variant adds the Read tier for MCP gating. Consolidation deferred per
// pass-1 adversarial observation. See TD-VSDD-082 (to be filed in factory-artifacts post-merge).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskTier {
    /// Read-only operation. No gate. Executes immediately.
    Read,
    /// Write operation that can be reversed. Dry-run default (BC-2.04.008).
    Reversible,
    /// Write operation that cannot be reversed. Requires confirmation token
    /// (BC-2.04.009) before execution.
    Irreversible,
}

// ─────────────────────────────────────────────────────────────
// DryRunResponse (BC-2.04.008)
// ─────────────────────────────────────────────────────────────

/// Preview response returned when a `Reversible` tool is invoked in dry-run
/// mode (the default).
///
/// Contains what *would* change without actually executing the write.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DryRunResponse {
    /// The target entity affected by the write.
    pub target_entity: String,
    /// Human-readable description of the proposed change.
    pub proposed_change: String,
    /// Confirmation prompt for the agent (e.g., "Set dry_run: false to execute").
    pub confirmation_prompt: String,
    /// Always `true` for dry-run previews (BC-2.04.008 postcondition).
    pub dry_run: bool,
}

// ─────────────────────────────────────────────────────────────
// GateDecision
// ─────────────────────────────────────────────────────────────

/// The outcome of applying the risk tier gate to a tool invocation.
#[derive(Clone, Debug)]
pub enum GateDecision {
    /// `RiskTier::Read` — no gate, proceed immediately.
    Allow,
    /// `RiskTier::Reversible` + `dry_run: true` — return dry-run preview.
    DryRunPreview(DryRunResponse),
    /// `RiskTier::Irreversible` — a confirmation token must be generated.
    RequiresConfirmationToken,
}

impl RiskTier {
    /// Apply the risk tier gate for a tool invocation.
    ///
    /// # Parameters
    /// - `dry_run`: whether the caller explicitly set `dry_run`. If `None`,
    ///   the per-tier default applies (Reversible defaults to `true`).
    ///
    /// # Returns
    /// - `GateDecision::Allow` for `Read` tools (BC-2.04.007 postcondition).
    /// - `GateDecision::DryRunPreview` when `dry_run` is `true` (or omitted)
    ///   for `Reversible` tools (BC-2.04.008 postcondition).
    /// - `GateDecision::RequiresConfirmationToken` for `Irreversible` tools
    ///   (BC-2.04.009).
    ///
    /// # Invariant
    /// `RiskTier::Read` MUST NEVER produce `DryRunPreview` or
    /// `RequiresConfirmationToken`.
    pub fn apply_gate(&self, dry_run: Option<bool>) -> GateDecision {
        match self {
            // Read tier: always Allow regardless of dry_run (BC-2.04.007, EC-006).
            RiskTier::Read => GateDecision::Allow,

            // Reversible tier: dry-run default is true.
            // dry_run = None or Some(true) → DryRunPreview.
            // dry_run = Some(false) → Allow (explicit execute, EC-04-017).
            RiskTier::Reversible => {
                if dry_run == Some(false) {
                    GateDecision::Allow
                } else {
                    // Default (None) or explicit true → preview.
                    GateDecision::DryRunPreview(DryRunResponse {
                        target_entity: "target entity".to_string(),
                        proposed_change: "proposed change (dry-run preview)".to_string(),
                        confirmation_prompt: "Set dry_run: false to execute this operation."
                            .to_string(),
                        dry_run: true,
                    })
                }
            }

            // Irreversible tier: always requires a confirmation token (BC-2.04.009).
            // dry_run is not meaningful here — the gate cannot be bypassed.
            RiskTier::Irreversible => GateDecision::RequiresConfirmationToken,
        }
    }
}
