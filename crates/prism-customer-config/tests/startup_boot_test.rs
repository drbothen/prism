//! Integration tests for the OrgRegistry boot orchestrator (S-3.3.02).
//!
//! Tests cover AC-001 through AC-006 from the story spec:
//!   - AC-001: Two valid distinct files → both registered, no error
//!   - AC-002: Duplicate org_id → E-CFG-011, exit 1, zero registry entries
//!   - AC-003: Duplicate org_slug → E-CFG-012, exit 1, zero registry entries
//!   - AC-004: N valid files → exactly N entries, bijectivity holds
//!   - AC-005: Zero .toml files → empty registry, no error
//!   - AC-006: Multiple file errors → all written before exit (multi-error pass)
//!
//! BC anchors: BC-3.1.003, BC-3.1.004, BC-3.3.004
//!
//! STATUS: Stub scaffold — no test functions yet.
//!         Test Writer fills in test_BC_* functions in the next phase.
//!         cargo check must pass; cargo test reports 0 tests for this binary.
