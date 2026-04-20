---
document_type: adversarial-review
level: ops
version: "1.0"
status: findings-open
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
pass: 48
previous_review: pass-47.md
novelty: HIGH — new drift class: MCP resource URI naming divergence between api-surface.md (canonical) vs BCs + stories (4 URIs drifted across BC-2.10.008, S-5.03, S-3.13)
findings_total: 5
findings_crit: 0
findings_high: 4
findings_med: 1
findings_low: 0
findings_observational: 0
convergence_counter: 0
date: 2026-04-19
---

# Adversarial Review: Prism (Pass 48)

## Finding ID Convention

Finding IDs use the format: `P3P<PASS>-A-<SEV>-<SEQ>`

- `P3P`: Cycle prefix (Phase-3-Patch cycle)
- `<PASS>`: Pass number (e.g., `48`)
- `A`: Adversary agent identifier
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Example: `P3P48-A-HIGH-001`

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| P3P47-A-HIGH-001 | HIGH | RESOLVED | S-5.05 lines 245-249 rewrite — fabricated `load_config`/`validate_config`/`show_config` cluster replaced with canonical `reload_config`; S-5.05 v1.3; changelog row present |

### 16-Dimension Pre-Sweep

| Dim | Axis | Status | Notes |
|-----|------|--------|-------|
| A-01 | BC-INDEX completeness (195 BCs, 6 dual-anchor, 13 removed) | CLEAN | BC-INDEX v4.10; 195 active BCs verified |
| A-02 | STORY-INDEX completeness (75 stories, 195 BCs, 39 VPs) | CLEAN | STORY-INDEX v1.28; counts consistent |
| A-03 | VP-INDEX completeness (39=20+11+6+2; 32 P0 + 7 P1) | CLEAN | VP breakdown matches index |
| A-04 | api-surface canonical tool list (28 read + 24 write = 52 tools) | CLEAN | Tool names internally consistent |
| A-05 | Burst 48 closure verification — S-5.05 Architecture Mapping rewrite + v1.3 + changelog | CLEAN | S-5.05 v1.3 verified; P3P47-A-HIGH-001 closed |
| A-06 | Stale tool name sweep (known-stale set) | CLEAN | No stale tool names in live corpus |
| A-07 | ARCH-INDEX subsystem coverage SS-01..SS-20 | CLEAN | All 20 subsystems present and consistent |
| A-08 | AI-opaque credentials semantics alignment | CLEAN | Reference-based model consistent |
| A-09 | Policy 8 bidirectional samples (BC → story + story → BC) | CLEAN | Sampled 10 pairs; all consistent |
| A-10 | Changelog discipline (version bumps + audit trail) | CLEAN | All Burst 48 changes carry changelog rows |
| A-11 | Error code reconciliation | CLEAN | No orphan or fabricated error codes |
| A-12 | Test-vector ↔ BC/VP traceability | CLEAN | test-vectors.md v2.3 consistent |
| A-13 | MCP resource URI consistency — api-surface.md canonical vs BCs/stories | **FINDING** | URI drift class: 4 URIs diverge from api-surface.md in BC-2.10.008, S-5.03, S-3.13 |
| A-14 | api-surface.md resource table completeness | **FINDING** | Per-client sensor subresource referenced in BCs+stories but absent from api-surface.md |
| A-15 | Schema resource URI parameter naming | **FINDING** | S-5.03 uses `{sensor}/{source}` vs canonical `{sensor_id}/{table_name}` |
| A-16 | Long-tail multi-document URI drift (new drift class check) | CONFIRMED | URI naming divergence spans BC + 2 stories — broader than single-story pattern of passes 45-47 |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

#### P3P48-A-HIGH-001: S-5.03 Uses Non-Canonical `prism://clients`

- **Severity:** HIGH
- **Category:** spec-fidelity / URI naming drift
- **Location:** S-5.03 (7 sites)
- **Description:** S-5.03 references `prism://clients` at 7 locations. The canonical URI per api-surface.md Configuration State Resources table is `prism://config/clients`. The `prism://config/` prefix is the established namespace for all configuration resources.
- **Evidence:** api-surface.md Configuration State Resources table lists `prism://config/clients` as the canonical URI. S-5.03 consistently omits the `config/` segment, referencing a non-existent `prism://clients` resource.
- **Proposed Fix:** Replace all 7 instances of `prism://clients` → `prism://config/clients` in S-5.03. Bump S-5.03 to v1.4.

---

#### P3P48-A-HIGH-002: Per-Client Sensor Subresource — 3-Way Contradiction

- **Severity:** HIGH
- **Category:** contradictions / interface-gaps
- **Location:** api-surface.md, S-5.03, BC-2.10.008
- **Description:** Three documents give three different answers for the per-client sensor listing resource URI: (1) api-surface.md — resource absent entirely; (2) S-5.03 — `prism://sensors/{client_id}`; (3) BC-2.10.008 — `prism://clients/{client_id}/sensors`. No canonical definition exists.
- **Evidence:** api-surface.md Configuration State Resources table contains no row for a per-client sensor subresource. S-5.03 uses `prism://sensors/{client_id}`. BC-2.10.008 postconditions and test cases use `prism://clients/{client_id}/sensors`. All three are irreconcilable without an architect decision.
- **Proposed Fix:** (a) Add `prism://config/clients/{client_id}/sensors` to api-surface.md as the canonical per-client sensor subresource; (b) reconcile S-5.03 and BC-2.10.008 to use that canonical form. Bump api-surface.md to v1.4, BC-2.10.008 to v1.2, S-5.03 to v1.4.

---

#### P3P48-A-HIGH-003: BC-2.10.008 Architecture Anchor Integrity Violated

- **Severity:** HIGH
- **Category:** spec-fidelity / architecture anchor integrity
- **Location:** BC-2.10.008 (postconditions, edge cases, test cases, Architecture Anchor)
- **Description:** BC-2.10.008 claims in its Architecture Anchor that it traces to api-surface.md as canonical, yet its postconditions, edge cases, and test cases use `prism://clients` and `prism://clients/{client_id}/sensors` — neither of which appears in api-surface.md. The Architecture Anchor integrity guarantee is violated.
- **Evidence:** BC-2.10.008 Architecture Anchor: cites api-surface.md. BC-2.10.008 postconditions: `prism://clients` (non-canonical). BC-2.10.008 test cases: `prism://clients/{client_id}/sensors` (non-canonical). api-surface.md: neither URI exists in the resource table.
- **Proposed Fix:** Update all URI references in BC-2.10.008 to the canonical forms established by the api-surface.md fix in HIGH-002. Bump BC-2.10.008 to v1.2.

---

#### P3P48-A-HIGH-004: S-3.13 References Non-Existent `prism://sensors` Bare URI

- **Severity:** HIGH
- **Category:** spec-fidelity / URI naming drift
- **Location:** S-3.13 (6 sites)
- **Description:** S-3.13 contains 6 references to `prism://sensors` as a bare resource URI. This URI does not exist in api-surface.md. The sensor listing capability is served by `prism://config/clients/{client_id}/sensors` (per-client) or by enumeration via `prism://config/clients`.
- **Evidence:** api-surface.md contains no `prism://sensors` resource entry. S-3.13 references it 6 times as if it were a first-class MCP resource. The closest canonical resource covering sensor enumeration is `prism://config/clients/{client_id}/sensors`.
- **Proposed Fix:** Determine architect intent for the bare sensor listing URI. Apply decision to all 6 S-3.13 sites. Bump S-3.13 to v1.2 (or higher per architect guidance).

---

### MEDIUM

#### P3P48-A-MED-001: S-5.03 Schema URI Uses Non-Canonical Parameter Names

- **Severity:** MED
- **Category:** spec-fidelity / parameter naming
- **Location:** S-5.03 (schema resource references)
- **Description:** S-5.03 uses `prism://schema/{sensor}/{source}` parameter placeholders. The canonical parameter names per api-surface.md are `{sensor_id}` and `{table_name}`. The use of `{sensor}` and `{source}` creates inconsistency with the canonical URI template.
- **Evidence:** api-surface.md schema resource: `prism://schema/{sensor_id}/{table_name}`. S-5.03: `prism://schema/{sensor}/{source}`. Parameter names `sensor` and `source` are aliases not present in the canonical template.
- **Proposed Fix:** Replace `{sensor}` → `{sensor_id}` and `{source}` → `{table_name}` in all S-5.03 schema URI references. Included in S-5.03 v1.4 bump for HIGH-001 fix.

---

### LOW

None.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 4 |
| MEDIUM | 1 |
| LOW | 0 |

**Overall Assessment:** pass-with-findings
**Convergence:** Findings remain — iterate. Counter stays 0/3.
**Readiness:** Burst 49 URI reconciliation required across api-surface.md, BC-2.10.008, S-5.03, S-3.13 before pass-49 dispatch.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 48 |
| **New findings** | 5 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (5 new / 5 total) |
| **Median severity** | HIGH (4.0/5.0) |
| **Trajectory** | 29→24→21→7→4→3→2→CLEAN→26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→4→6→2→3→3→12→4→3→3→8→4→3→0(CLEAN)→5(RESET)→5→1→1→1→**5** |
| **Verdict** | FINDINGS_REMAIN — counter 0/3; Burst 49 closes all 5 findings; pass-49 targets CLEAN |

<!--
  Novelty: HIGH. URI naming divergence is a new drift class not identified in passes 1-47.
  Prior sweeps caught tool-name drift and paragraph-level fabrication; this class is broader —
  it spans the BC layer, story layer, and api-surface.md simultaneously.

  The 3-way contradiction in HIGH-002 (api-surface absent + story wrong + BC wrong) is the
  highest-risk finding: it means no authoritative contract existed for this resource in any
  prior pass. Burst 49 must add the resource to api-surface.md before reconciling downstream.

  Pattern shift: passes 45-47 were single-document, single-axis. Pass 48 is multi-document,
  multi-axis. Adversary should widen URI consistency sweep scope in pass-49.
-->
