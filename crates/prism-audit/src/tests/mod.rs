//! Test module declarations for prism-audit (S-2.04).
//!
//! Test naming follows `test_BC_2_05_NNN_xxx()` convention for full traceability.

// Shared test helpers (FailingBackend, make_request, count_audit_entries, etc.)
pub mod helpers;

// BC-2.05.001 — Every MCP tool invocation produces exactly one audit entry
pub mod bc_2_05_001;

// BC-2.05.002 — Audit entries use structured JSON format with complete fields
pub mod bc_2_05_002;

// BC-2.05.003 — Credential values are never present in audit entries
pub mod bc_2_05_003;

// BC-2.05.004 — Write operations log capability check and execution outcome
pub mod bc_2_05_004;

// BC-2.05.006 — Audit entries are append-only and immutable
pub mod bc_2_05_006;

// BC-2.05.008 — Audit entries satisfy SOC 2 Type II and ISO 27001 requirements
pub mod bc_2_05_008;

// S-2.05 specialized event tests:
// BC-2.05.005 — Credential access events are audit-logged with context
// BC-2.05.007 — Audit entries are compatible with the Vector pipeline
// BC-2.05.009 — Feature flag evaluations for write operations are audit-logged
// BC-2.05.010 — Confirmation token lifecycle events are audit-logged
pub mod specialized_event_tests;

// WGC-W2-001 — Emitters must call append_audit_entry (persistence fix)
pub mod wgc_w2_001_emitter_persistence;

// S-3.1.07 — BC-3.1.002: AuditEntry org_id + org_slug fields (Red Gate)
pub mod bc_3_1_001_org_fields;

// S-3.1.07 — BC-3.1.002 / TD-ADR005-002: SHA-256 aql_hash computation (Red Gate)
pub mod bc_3_1_002_aql_hash;

// F-PASS3-OBS-1 (S-WAVE5-PREP-01 fix-pass-3): owning-crate unit tests for BootAuditEmitter
pub mod boot_emitter;
