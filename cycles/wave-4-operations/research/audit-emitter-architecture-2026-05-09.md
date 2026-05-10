---
document_type: architectural-research
topic: AuditEmitter refactor vs BootAuditEmitter retention
related_story: S-WAVE5-PREP-01
related_bc: BC-2.05.012
date: 2026-05-09
recommendation: KEEP_BOOTAUDITEMITTER_AMEND_BC
inputs:
  - .factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md
  - .factory/specs/behavioral-contracts/BC-2.05.001-audit-entry-per-tool-invocation.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
input-hash: "4bee8c2"
---

# Architectural Research: AuditEmitter Refactor vs BootAuditEmitter Retention

**Topic:** Should `BootAuditEmitter` be refactored into a single `AuditEmitter` type, or should the two-type design be retained?
**Related Story:** S-WAVE5-PREP-01
**Related BC:** BC-2.05.012
**Date:** 2026-05-09
**Recommendation:** KEEP_BOOTAUDITEMITTER_AMEND_BC

---

## 1. Internal State Summary

The current implementation introduces two audit emitter types:

- **`BootAuditEmitter`** (`crates/prism-audit/src/lib.rs`): A scoped emitter for boot-lifecycle events (boot start, boot complete sentinel). It is constructed in `boot()`, used to emit boot-phase events, and dropped when `boot()` returns. It does not survive into the MCP server runtime.

- **`AuditEmitter`** (`crates/prism-audit/src/lib.rs`): The runtime emitter for per-tool-invocation audit entries per BC-2.05.001. It is constructed in `boot()` and propagated into `McpServerContext` for use throughout the process lifetime.

The adversarial pass-3 finding F-PASS3-MED-1 identified that BC-2.05.012 §Postconditions item 4 says the `AuditEmitter` instance "MUST be propagated to the MCP server context for use throughout the process lifetime" and "MUST NOT be dropped at the end of the boot phase." This language was written assuming a single emitter type. The implementation uses two types, and `BootAuditEmitter` IS dropped at end of boot. Hence the postcondition drift.

The question: fix the implementation (merge into one type, propagate forward) or fix the BC (acknowledge the two-type design)?

---

## 2. Pattern Survey

### 2.1 Boot-phase-scoped vs runtime-scoped resources in systems with audit trails

**Linux audit daemon (auditd):** Uses separate boot-time logging (kernel ring buffer via printk) and runtime audit logging (auditd socket). The two channels are distinct by design: boot-time events cannot safely use the runtime audit mechanism because the runtime mechanism is not yet initialized. This is the exact pattern in `prism-bin`.

**OpenTelemetry SDK (Rust):** The `TracerProvider` is initialized once at startup and propagated forward. However, a "bootstrap span" pattern exists where pre-initialization events are recorded separately and flushed into the main telemetry pipeline after initialization completes. This mirrors `BootAuditEmitter` semantics.

**PostgreSQL:** Uses `ereport()` from the earliest startup phases. The log destination starts as stderr and transitions to the configured log sink after catalog initialization. Two-phase logging is the norm, not an exception.

**Pattern consensus:** Separating boot-phase audit from runtime audit is architecturally normal. Collapsing them into a single type requires either (a) making the runtime emitter available before its dependencies are initialized, or (b) buffering boot events for deferred flush — both of which add complexity with no behavioral benefit for the Prism use case.

### 2.2 The specific Prism initialization order constraint

From ADR-022 §Boot Sequence (11-step sequence):

1. Load config
2. Init RocksDB storage
3. **Init BootAuditEmitter** (depends on step 2 — RocksDB must be open)
4. Emit boot-start event
5. Init OrgRegistry
6. Init CredentialStore
7. Iterate credential_refs
8. Init AuditEmitter (runtime, for MCP context)
9. Emit boot-complete sentinel
10. Construct McpServerContext (with AuditEmitter from step 8)
11. Start MCP server

`BootAuditEmitter` is created at step 3, before `AuditEmitter` at step 8. If they were merged into one type, the single emitter would need to be constructed at step 3 (when it's needed for boot-start) but also be ready for MCP propagation at step 10. The type's design would need to handle the "pre-complete-initialization" state without compromising the runtime audit contract.

**Verdict from initialization order analysis:** The two-type design is a direct consequence of the initialization order. Merging is possible but would add state machine complexity (e.g., a `BootPhase` vs `RuntimePhase` discriminant on the type) with no functional benefit.

---

## 3. Trade-Off Matrix

| Criterion | Merge into one AuditEmitter | Keep BootAuditEmitter + AuditEmitter |
|-----------|----------------------------|--------------------------------------|
| BC-2.05.012 compliance (current wording) | PASS (single type propagated) | FAIL (BootAuditEmitter dropped) |
| BC-2.05.012 compliance (amended wording) | PASS | PASS |
| BC-2.05.001 runtime audit compliance | PASS | PASS |
| Implementation complexity delta | +2 (state discriminant or builder pattern) | 0 (current design) |
| Initialization order safety | Requires careful design (boot events before runtime ready) | Inherently safe (separate types = separate lifetimes) |
| Type clarity (single responsibility) | FAIL (one type does boot-phase + runtime) | PASS (each type has one role) |
| Test isolation | FAIL (harder to unit test boot path without runtime deps) | PASS (BootAuditEmitter testable without McpServerContext) |
| Future extensibility | Moderate (adding boot-specific fields pollutes runtime type) | Good (types can evolve independently) |
| ADR-022 alignment | Neutral (ADR-022 does not mandate single vs dual type) | Strong (ADR-022 §Boot Sequence step numbering implies staged init) |
| Behavioral risk | LOW (functional change, well-understood) | ZERO (no code change required) |

**Scorecard:** Keep two types is superior on 6 of 8 criteria. Merging is superior only on BC-2.05.012 compliance under CURRENT (pre-amendment) wording.

---

## 4. Recommendation

**KEEP `BootAuditEmitter`. Amend BC-2.05.012.**

The two-type design is architecturally correct. `BootAuditEmitter` is a boot-phase-scoped resource with a single responsibility: emit boot-lifecycle audit events (start + complete sentinel) during the period when the system is not yet ready for runtime audit traffic. `AuditEmitter` is the runtime resource propagated into `McpServerContext`.

Merging the types would:
1. Require a state discriminant or builder pattern adding complexity with no behavioral gain
2. Violate the single-responsibility principle (one type doing two distinct jobs)
3. Compromise testability (harder to unit test boot audit without MCP dependencies)
4. Introduce initialization order risk (runtime emitter used before runtime dependencies ready)

The BC-2.05.012 postcondition drift is a spec authoring artifact: the BC was written before the two-type design was finalized. The fix is a BC amendment, not an implementation change.

---

## 5. BC Amendment Proposal

**File:** `.factory/specs/behavioral-contracts/BC-2.05.012-audit-subsystem-init.md`

### Description section (lines 31-32)

**Current wording (approximate):**
> The audit subsystem initialization establishes a durable `AuditEmitter` instance that MUST be propagated to the MCP server context for use throughout the process lifetime.

**Proposed amendment:**
> The audit subsystem initialization establishes two complementary resources: (1) a `BootAuditEmitter` that is scoped to the boot phase and MUST emit the boot-start event and boot-complete sentinel before being dropped, and (2) a runtime `AuditEmitter` that MUST be propagated to the MCP server context for use throughout the process lifetime. The `BootAuditEmitter` is intentionally scoped; it is NOT expected to survive into the MCP server runtime.

### Postconditions section (line 58)

**Current wording (approximate):**
> 4. The `AuditEmitter` instance MUST be propagated to the MCP server context for use throughout the process lifetime. It MUST NOT be dropped at the end of the boot phase.

**Proposed amendment:**
> 4. The runtime `AuditEmitter` instance MUST be propagated to the MCP server context for use throughout the process lifetime. It MUST NOT be dropped at the end of the boot phase.
> 5. The `BootAuditEmitter` instance is boot-phase-scoped and MUST be dropped after `emit_boot_complete()` returns. It MUST NOT be propagated into the MCP server context.

### OQ-2 resolution (lines 64-65)

If BC-2.05.012 contains an open question about single vs dual emitter type (OQ-2), it should be resolved with:

> **OQ-2 RESOLVED (2026-05-09):** Two-type design (`BootAuditEmitter` + `AuditEmitter`) is the canonical architecture. `BootAuditEmitter` is boot-phase-scoped. `AuditEmitter` is runtime-propagated. See `cycles/wave-4-operations/research/audit-emitter-architecture-2026-05-09.md` for full analysis.

---

## 6. Implementation Cost Estimate

**If recommendation is accepted (BC amendment path):**
- Product owner: BC-2.05.012 amendment — estimated 30 minutes
- State manager: record D-311 + commit — 15 minutes
- No implementation code changes required

**If recommendation is rejected (implementation merge path):**
- Implementer: merge `BootAuditEmitter` into `AuditEmitter` with boot-phase state tracking — estimated 2-4 hours
- Test writer: update integration tests — estimated 1 hour
- Adversary: pass-4 must verify merged type handles initialization order correctly — risk of regression

**Recommendation path is 6-10x cheaper and lower risk.**

---

## Sources

1. BC-2.05.012 §Postconditions — current postcondition 4 wording triggering F-PASS3-MED-1
2. BC-2.05.001 §Description — runtime audit-entry-per-tool-invocation requirement
3. ADR-022 §Boot Sequence — 11-step initialization order with step-by-step resource construction
4. `crates/prism-audit/src/lib.rs` — `BootAuditEmitter` and `AuditEmitter` implementations
5. `crates/prism-bin/src/boot.rs:786-800` — BootAuditEmitter usage scope
6. `crates/prism-bin/src/mcp_context.rs:18-42` — McpServerContext field list (AuditEmitter present, BootAuditEmitter absent — intentional)
7. Linux audit subsystem documentation — boot-time vs runtime audit separation pattern
8. OpenTelemetry Rust SDK — bootstrap span pattern
9. PostgreSQL source (src/backend/utils/error/elog.c) — two-phase log destination pattern
10. ADR-022 §Wiring Contracts — propagation requirements for runtime resources
11. `cycles/wave-4-operations/adversarial-reviews/s-wave5-prep-01-local-pass-3.md` — F-PASS3-MED-1 finding that triggered this research
12. `cycles/wave-4-operations/adversarial-reviews/s-wave5-prep-01-local-pass-2.md` — pass-2 closure verification context

---

## Research Methods

- Static analysis of `prism-audit` crate types and their usage in `prism-bin`
- Initialization order reconstruction from ADR-022 §Boot Sequence
- Pattern survey across three production systems (Linux auditd, OTel Rust, PostgreSQL) for boot-phase vs runtime audit separation precedent
- Trade-off matrix evaluation across 8 architectural criteria
- BC-2.05.012 postcondition analysis against implementation at `boot.rs:786-800`
- Cost estimation for both resolution paths (BC amendment vs implementation merge)
