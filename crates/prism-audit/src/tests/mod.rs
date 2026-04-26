//! Test module declarations for prism-audit (S-2.04).
//!
//! Test bodies are added by the Test Writer (next dispatch).
//! Every test will follow the `test_BC_2_05_NNN_xxx()` naming convention.

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
