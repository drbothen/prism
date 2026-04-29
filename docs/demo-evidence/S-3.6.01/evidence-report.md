# Demo Evidence Report — S-3.6.01

**Story:** S-3.6.01 — HS-006 multi-tenant state recovery holdout refresh — re-anchor to Wave 3 BCs
**Phase:** 3.A
**Wave:** 3
**Closes TD:** TD-HOLDOUT-W2-002
**Date:** 2026-04-29
**Recorder:** Demo Recorder

---

## BC Anchors

| BC ID | Title | Role in HS-006 |
|-------|-------|----------------|
| BC-3.2.001 | Per-Org Sensor Data Isolation | Asserts `(OrgId, device_id)` composite key isolation in RocksDB; no cross-org device IDs |
| BC-3.2.003 | Per-Org Session Token Isolation | Asserts token store keyed by `(OrgId, token_string)`; tokens not accepted cross-org |
| BC-3.5.001 | Harness Logical Isolation | Asserts `devices(OrgA) ∩ devices(OrgB) = ∅` cross-tenant invariant throughout all scenarios |
| BC-3.6.001 | Per-Org Failure Injection | Asserts `inject_failure` modifies only target org's `FailureLayerShared`; sibling orgs unaffected |
| BC-3.6.002 | Harness Crash Detection | Asserts `CloneCrashed` detection within 1 second of Tokio task exit; non-crashed orgs continue |

---

## Coverage Map

### AC-001 — HS-006 frontmatter `behavioral_contracts` lists current BC IDs

| Recording | Path | Description |
|-----------|------|-------------|
| AC-001-hs-006-anchor-tests-green.gif | `docs/demo-evidence/S-3.6.01/AC-001-hs-006-anchor-tests-green.gif` | Animated recording of 5/5 anchor tests passing |
| AC-001-hs-006-anchor-tests-green.webm | `docs/demo-evidence/S-3.6.01/AC-001-hs-006-anchor-tests-green.webm` | Archival webm of same run |
| AC-001-hs-006-anchor-tests-green.tape | `docs/demo-evidence/S-3.6.01/AC-001-hs-006-anchor-tests-green.tape` | VHS tape source |

**Command demonstrated:** `cargo test --test hs_006_anchor_test 2>&1`

**Result:** `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

**Tests verified:**
- `test_hs_006_anchored_to_wave_3_bcs` — `behavioral_contracts` contains exactly the 5 required Wave 3 BC IDs (BC-3.2.001, BC-3.2.003, BC-3.5.001, BC-3.6.001, BC-3.6.002), no duplicates, no extras
- `test_hs_006_phase_is_3a` — `phase: "3.A"` and `closes_td: [TD-HOLDOUT-W2-002]` present in frontmatter
- `test_hs_006_no_stub_markers` — no TODO/STUB/(stub: markers remain in any sub-scenario body
- `test_hs_006_seven_sub_scenarios_present` — all 7 sub-scenario headings (HS-006-01 through HS-006-07) present with complete Expected Outcome sections
- `test_hs_006_no_legacy_wave_bc_references` — no Wave 1/2 BC references; every BC Anchors line cites at least one BC-3.x.xxx entry

---

### AC-002 — Frontmatter snapshot showing Wave 3 anchors

| Recording | Path | Description |
|-----------|------|-------------|
| AC-002-frontmatter-anchors.gif | `docs/demo-evidence/S-3.6.01/AC-002-frontmatter-anchors.gif` | Animated recording of HS-006 frontmatter |
| AC-002-frontmatter-anchors.webm | `docs/demo-evidence/S-3.6.01/AC-002-frontmatter-anchors.webm` | Archival webm of same |
| AC-002-frontmatter-anchors.tape | `docs/demo-evidence/S-3.6.01/AC-002-frontmatter-anchors.tape` | VHS tape source |

**Command demonstrated:** `head -35 tests/holdout-scenarios/HS-006-state-recovery.md`

**Frontmatter fields visible in recording:**
- `behavioral_contracts: [BC-3.2.001, BC-3.2.003, BC-3.5.001, BC-3.6.001, BC-3.6.002]`
- `phase: "3.A"`
- `closes_td: [TD-HOLDOUT-W2-002]`

---

## Sub-Scenario Summary

| Sub-Scenario | Title | BC Anchors | Wave 3 Change |
|---|---|---|---|
| HS-006-01 | Multi-Tenant Harness Restart with RocksDB State Persistence | BC-3.2.001, BC-3.5.001 | Phase 1b FileStore cursor resume replaced with RocksDB column family keyed by `(OrgId, device_id)`; 3 orgs resume from persisted state |
| HS-006-02 | Clone Task Panic Mid-Operation; CloneCrashed Detection Within 1 Second | BC-3.6.002, BC-3.5.001 | Atomic file write crash replaced with Tokio task panic; `HarnessError::CloneCrashed` expected; cross-tenant integrity asserted |
| HS-006-03 | Query Fingerprint Mismatch Forces Per-Org State Reset | BC-3.2.001, BC-3.5.001 | Per-org fingerprint mismatch clears only affected org's RocksDB state; sibling org state unaffected |
| HS-006-04 | Per-Org RocksDB Offset Is Monotonically Non-Decreasing | BC-3.2.001, BC-3.2.003 | `set_offset` regression returns `HarnessError::OffsetRegression`; forward progress invariant enforced per org |
| HS-006-05 | Per-Org Session Token Survives Harness Restart; Cross-Org Tokens Remain Isolated | BC-3.2.003, BC-3.5.001 | Token store keyed by `(OrgId, token_string)`; acme token rejected in globex context after restart |
| HS-006-06 | Simultaneous Multi-Org Clone Crash; Independent Recovery; Cross-Tenant Integrity Preserved | BC-3.6.002, BC-3.5.001, BC-3.2.001 | 4-org harness with 8 clones; two simultaneous panics detected independently within 1s; non-crashed orgs unaffected |
| HS-006-07 | Per-Org Failure Injection Triggers Crash Detection; Sibling Org Unaffected | BC-3.6.001, BC-3.6.002, BC-3.5.001 | `inject_failure("acme-corp", Cyberint, InternalError)` triggers `CloneCrashed` for acme-corp only; globex returns HTTP 200 |

---

## Toolchain

| Tool | Version | Role |
|------|---------|------|
| VHS | 0.10.0 | Terminal session recording (.gif + .webm) |
| cargo | stable | Test runner |
| FiraCode Nerd Font Mono | installed | VHS terminal font |
