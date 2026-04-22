// S-1.08: Feature Flag Audit Logging — STUB (Red Gate)
//
// All function bodies are `unimplemented!()`.  The implementer must fill them
// in to make the test suite green.
//
// Story:  S-1.08 — prism-security: Feature Flags (P0 Core)
// BC:     BC-2.04.013 (capability check audit logging for write operations)
//
// Architecture compliance rules:
//   - Every write capability check MUST emit a `CapabilityCheckEvent` via
//     `tracing::info!` (BC-2.04.013 invariant DI-004).
//   - Audit events MUST be emitted for BOTH Allow and Deny outcomes.
//   - Read operations do NOT emit capability check events.
//   - If the tracing subscriber fails, the capability check still proceeds
//     (best-effort audit per BC-2.04.013 error cases).
//   - EC-003: audit emission failure must NOT affect the gate result.

// ─────────────────────────────────────────────────────────────
// CapabilityCheckEvent
// ─────────────────────────────────────────────────────────────

/// Structured audit event emitted for every write capability check.
///
/// Emitted via `tracing::info!` with structured fields for SOC 2 / ISO 27001.
#[derive(Clone, Debug)]
pub struct CapabilityCheckEvent {
    /// Always `"capability_check"` (BC-2.04.013).
    pub event_type: &'static str,
    /// The tenant/client whose capabilities were evaluated.
    pub client_id: String,
    /// The capability path that was checked (e.g., `"sensor.crowdstrike.containment"`).
    pub capability: String,
    /// `"allowed"` or `"denied"`.
    pub result: &'static str,
    /// The MCP tool name that triggered the check.
    pub tool_name: String,
    /// Denial reason when `result == "denied"`.
    /// One of:
    /// - `"Feature not compiled"`
    /// - `"Not enabled in client config"`
    /// - `"No matching capability path"`
    pub denied_reason: Option<String>,
    /// UTC timestamp of the check.
    pub timestamp: String,
}

// ─────────────────────────────────────────────────────────────
// FlagAuditEmitter
// ─────────────────────────────────────────────────────────────

/// Emits `CapabilityCheckEvent`s via `tracing::info!` (BC-2.04.013).
///
/// This type is the effectful boundary for audit logging. All flag evaluation
/// logic is pure; only this emitter has side effects.
pub struct FlagAuditEmitter;

impl FlagAuditEmitter {
    /// Construct a new emitter.
    pub fn new() -> Self {
        unimplemented!("S-1.08: FlagAuditEmitter::new — implement construction")
    }

    /// Emit a write capability check audit event (BC-2.04.013 postconditions).
    ///
    /// MUST be called for every write capability check, regardless of outcome.
    /// Read operations MUST NOT call this method.
    ///
    /// If the tracing subscriber fails, this method MUST NOT panic or return
    /// an error — it is best-effort (BC-2.04.013 error cases).
    pub fn emit_write_check(&self, event: &CapabilityCheckEvent) {
        unimplemented!("S-1.08: FlagAuditEmitter::emit_write_check — implement tracing::info! emission")
    }

    /// Construct an `Allowed` event for a write capability check.
    pub fn allowed_event(
        client_id: impl Into<String>,
        capability: impl Into<String>,
        tool_name: impl Into<String>,
    ) -> CapabilityCheckEvent {
        unimplemented!("S-1.08: FlagAuditEmitter::allowed_event — implement allowed event construction")
    }

    /// Construct a `Denied` event for a write capability check.
    pub fn denied_event(
        client_id: impl Into<String>,
        capability: impl Into<String>,
        tool_name: impl Into<String>,
        denied_reason: impl Into<String>,
    ) -> CapabilityCheckEvent {
        unimplemented!("S-1.08: FlagAuditEmitter::denied_event — implement denied event construction")
    }
}

impl Default for FlagAuditEmitter {
    fn default() -> Self {
        Self::new()
    }
}
