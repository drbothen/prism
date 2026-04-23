---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-04-23T00:00:00
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
pass: 3
previous_review: cycles/phase-3-dtu-wave-1/adversarial-reviews/wave-1-integration-gate/pass-2.md
review_scope: wave-1-integration-gate
cycle: phase-3-dtu-wave-1
reviewer: adversary
develop_head: e187acec
stories_merged: 20
prs_merged: 31
stories_sampled: S-1.07, S-1.09, S-1.12, S-1.15, S-6.20
verdict: BLOCKED
policy_coverage:
  - POLICY 1 (append_only_numbering): not_verified — indexes on factory-artifacts branch
  - POLICY 2 (lift_invariants_to_bcs): not_verified
  - POLICY 3 (state_manager_runs_last): drift_found — STATE.md + tech-debt-register.md count mismatch (M-001)
  - POLICY 4 (semantic_anchoring): violation_found — S-1.07 EC-001 references E-CRED-003 with wrong semantic (H-001)
  - POLICY 5 (creators_justify_anchors): not_verified
  - POLICY 6 (architecture_subsystem_source_of_truth): drift_found — AD-001 crate count stale (L-001)
  - POLICY 7 (bc_h1_source_of_truth): not_verified
  - POLICY 8 (bc_array_propagates): not_verified
  - POLICY 9 (vp_index_source_of_truth): not_verified
  - POLICY 10 (demo_evidence_story_scoped): not_verified
counts:
  critical: 0
  high: 1
  medium: 1
  low: 1
  observation: 1
  total: 4
novelty: HIGH
trajectory: "pass-1:11 → pass-2:10 → pass-3:4 (different surface — new findings, not carry-forwards)"
---

# Adversarial Review — Wave 1 Integration Gate (Pass 3)

## Finding ID Convention

Finding IDs for this pass use the project-scoped format: `P3WV1C-A-<SEV>-<SEQ>`

- `P3WV1C`: Phase 3, Wave 1, Pass C (third pass) — distinguishes from P3WV1 (Pass 1) and P3WV1B (Pass 2)
- `A`: adversary pass
- `<SEV>`: severity abbreviation (`H` = HIGH, `M` = MEDIUM, `L` = LOW, `OBS` = OBSERVATION)
- `<SEQ>`: three-digit sequence within severity

This is Pass 3 of the wave-level integration gate review. Pass 1 findings were remediated via
PR #30 (f290f450). Pass 2 findings were remediated via PR #31 (e187acec) and factory-artifacts
4eba02a2. All findings in this pass are new — none are carry-forwards from Pass 1 or Pass 2.

## Summary

Pass 3 of the wave-level adversarial review. develop HEAD: e187acec (PR #31 merged — 9 Pass 2
findings closed). Four findings surfaced on a fresh surface: a semantic mis-anchor in S-1.07
EC-001 row referencing E-CRED-003 (a decryption-failure error code) in a context that BC-2.03.005
defines as a `ConfirmationToken` response object; summary-line drift in tech-debt-register.md
where P2 count is one less than the actual body count after TD-WV1-03 was appended without
updating the Summary; stale AD-001 crate count in ARCH-INDEX.md (the "12 in Phase 1; +4 DTU
crates" annotation misrepresents the actual 8+8=16 layout); and a date discrepancy between
STATE.md (`wave_0a_complete: 2026-04-21`) and wave-state.yaml (`gate_date: 2026-04-22`) for
the wave_0a gate — already tracked as TD-CV-04.

Pass 2 closed the env-var race (H-001), demo README annotation (M-001), TOCTOU (M-003), and
CI negative-path (M-004) via PR #31; and closed AC-1 rescoping (H-002), wave-state.yaml HEAD
(H-003), TD-CV-02/TD-CV-03 resolution (M-002), vp_index_version prefix (L-001), and AD-001
crate count (L-002) via factory-artifacts 4eba02a2. Pass 3 finds no new CRITICAL or
security-surface findings at the remediated HEAD. Novelty is HIGH — none of these findings
were present or surfaced in Pass 1 or Pass 2 context.

**Verdict: BLOCKED** — 1 HIGH finding (semantic mis-anchor) requires remediation before PASSED
verdict. Pass 4 required; the 3-pass clean window must restart from zero.

## Finding Summary Table

| ID | Severity | Category | Location | Title |
|----|----------|----------|----------|-------|
| P3WV1C-A-H-001 | HIGH | semantic-anchoring | S-1.07 AC-1 deferral note (~line 72) + EC-001 row (~line 94) | E-CRED-003 mis-anchor: decryption-failure error code used where ConfirmationToken response object is specified |
| P3WV1C-A-M-001 | MEDIUM | state-drift | tech-debt-register.md:17-18; STATE.md:35 | TD register Summary P2 count (10) lags body count (11); active count (18) one short; STATE.md entry count stale |
| P3WV1C-A-L-001 | LOW | state-drift | ARCH-INDEX.md:65 (AD-001) | AD-001 annotation "12 in Phase 1; +4 DTU crates" misrepresents actual 8+8=16 workspace layout |
| P3WV1C-A-OBS-001 | OBSERVATION | state-drift | STATE.md:27 vs wave-state.yaml:14 | wave_0a_complete date off-by-one (TD-CV-04) — STATE.md 2026-04-21 vs wave-state.yaml 2026-04-22 |

---

## Part A — Fix Verification (pass >= 2 only)

All Pass 2 findings remediated before this pass began:

- P3WV1B-A-H-001 (env-var race) → closed via PR #31 (e187acec) — `#[serial]` added
- P3WV1B-A-H-002 (AC-1 MCP deferral rescope) → closed at factory-artifacts 4eba02a2 — story v1.7
- P3WV1B-A-H-003 (wave-state.yaml stale HEAD/gate) → closed at factory-artifacts 4eba02a2
- P3WV1B-A-M-001 (README AC-4 row contradicts TD-WV1-04) → closed via PR #31 (e187acec)
- P3WV1B-A-M-002 (TD-CV-02 + TD-CV-03 stale active) → closed at factory-artifacts 4eba02a2
- P3WV1B-A-M-003 (TOCTOU in add_sensor_spec.rs) → closed via PR #31 (e187acec)
- P3WV1B-A-M-004 (CI no-default-features gap) → closed via PR #31 (e187acec)
- P3WV1B-A-L-001 (vp_index_version `v` prefix) → closed at factory-artifacts 4eba02a2
- P3WV1B-A-L-002 (ARCH-INDEX AD-001 "12 crates") → closed at factory-artifacts 4eba02a2
- P3WV1B-A-OBS-001 + OBS-002 → deferred (informational; no action required per policy)

No Pass 2 findings are carry-forwards into Pass 3. All findings in this pass are new.

---

## Part B — New Findings (Pass 3 — all findings are new)

### HIGH

#### P3WV1C-A-H-001: E-CRED-003 mis-anchor — decryption-failure error code used where BC-2.03.005 specifies ConfirmationToken response object

- **Severity:** HIGH
- **Category:** semantic-anchoring (Policy 4 violation)
- **Location:**
  - `.factory/stories/S-1.07-credential-crud.md` line ~72 (AC-1 deferral note)
  - `.factory/stories/S-1.07-credential-crud.md` line ~94 (EC-001 row)
- **Description:** Pass 2 (P3WV1B-A-H-002) rescoped AC-1 from MCP-layer to library-layer
  and added a deferral note. The rescoped AC-1 deferral note (line ~72) reads:

  > "MCP-layer tool surface that wraps this with E-CRED-003 error code is deferred to Wave 2
  > per TD-S-1.07-01"

  The EC-001 row (line ~94) reads:

  > "Returns E-CRED-003 requiring confirmation (BC-2.03.005)"

  Both references anchor to `E-CRED-003`. Consulting `error-taxonomy.md` line 65:
  `E-CRED-003` is defined as **"Credential decryption failed"** — a decryption-failure
  error code, not a confirmation-required status. BC-2.03.005 postcondition (line 42 of
  the BC file) specifies that the confirmation-required response is a **`ConfirmationToken`
  response object**, not a structured error code.

  The semantic mis-anchor was introduced when Pass 2 remediation amended AC-1 but
  carried forward the pre-existing `E-CRED-003` reference from the original story text
  without verifying the error taxonomy. The MCP-layer confirmation flow does not emit
  `E-CRED-003`; it returns a `ConfirmationToken` response object. `E-CRED-003` is the
  code emitted when the credential backend fails to decrypt a stored value — a different
  failure mode entirely.

  This is a Policy 4 (semantic_anchoring) violation: the anchor `E-CRED-003` is mis-anchored
  to a behavior (confirmation-required) that the taxonomy assigns to a different code path.
  Any implementer reading AC-1 or EC-001 will wire the wrong error code into the MCP tool
  surface when Wave 2 lands.

- **Evidence:**
  - `S-1.07-credential-crud.md:72` deferral note: "wraps this with E-CRED-003 error code"
  - `S-1.07-credential-crud.md:94` EC-001 row: "Returns E-CRED-003 requiring confirmation"
  - `error-taxonomy.md:65`: E-CRED-003 = "Credential decryption failed"
  - `BC-2.03.005:42` postcondition: confirmation-required response = `ConfirmationToken` object

- **Proposed Fix:** Remove all `E-CRED-003` references from the confirmation-required
  context in S-1.07:

  AC-1 deferral note (line ~72) — change:
  > "MCP-layer tool surface that wraps this with E-CRED-003 error code is deferred..."

  To:
  > "MCP-layer tool surface that wraps this with the `ConfirmationToken` response object
  > (per BC-2.03.005 postcondition for the update case — NOT a structured error) is deferred
  > to Wave 2 per TD-S-1.07-01"

  EC-001 row (line ~94) — change:
  > "Returns E-CRED-003 requiring confirmation (BC-2.03.005)"

  To:
  > "Returns `Err(CredentialError::ConfirmationRequired)` at library scope (or `ConfirmationToken`
  > response object at MCP tool scope per BC-2.03.005 postcondition; confirmation-required is
  > a status signal, not a structured error code)"

  Bump story version to 1.8 with changelog entry documenting the fix.

---

### MEDIUM

#### P3WV1C-A-M-001: TD register Summary P2 count (10) lags body count (11); active count stale; STATE.md entry stale

- **Severity:** MEDIUM
- **Category:** state-drift (Policy 3 — state_manager_runs_last)
- **Location:**
  - `.factory/tech-debt-register.md` lines 17-18 (Summary table)
  - `.factory/STATE.md` line 35 (`tech_debt_register_entries: 18`)
- **Description:** The tech-debt-register.md Summary table (lines 17-18) reads:

  | P2 (backlog) | 10 | 5 |
  _Active items: 18._

  Counting the actual body rows by priority:

  **P1 items (active):** TD-WV0-01, TD-WV0-02, TD-WV0-03, TD-WV0-04, TD-WV1-01,
  TD-WV1-02, TD-S-1.07-01, TD-WV1-04 = **8 active P1 items** ✓ (matches)

  **P2 items (active):** TD-WV0-06, TD-WV0-07, TD-WV0-08, TD-WV0-09, TD-WV0-10,
  TD-WV0-11, TD-WV0-12, TD-CV-04, TD-WV1-03, TD-S620-004, TD-S620-005 = **11 active P2
  items** (Summary claims 10)

  The discrepancy originates from TD-WV1-03 (S-1.09 `consume()` refactor, PR review
  suggestion, introduced during the Pass 2 remediation cycle). TD-WV1-03 was appended to
  the body without bumping the Summary P2 count from 10 to 11 or the active total from
  18 to 19.

  STATE.md frontmatter `tech_debt_register_entries: 18` also reflects the pre-TD-WV1-03
  count and must be updated to 19.

- **Evidence:**
  - `tech-debt-register.md:18` Summary: `P2 (backlog) | 10`
  - `tech-debt-register.md:20` narrative: "Active items: 18"
  - Body row TD-WV1-03: present and active (P2, no RESOLVED marker)
  - Counting P2 active rows: 11 (7 WV0 + TD-CV-04 + TD-WV1-03 + TD-S620-004 + TD-S620-005)
  - `STATE.md:35`: `tech_debt_register_entries: 18`

- **Proposed Fix:** Update tech-debt-register.md:
  - Summary table P2 count: 10 → 11
  - Active items narrative: 18 → 19

  Update STATE.md:
  - `tech_debt_register_entries: 18` → `tech_debt_register_entries: 19`

  (Note: if P3WV1C-A-OBS-001 is remediated in the same burst by resolving TD-CV-04, the
  P2 count returns to 10 and active to 18 after that resolution. Apply both fixes
  sequentially: first fix M-001 to 11/19, then resolve OBS-001 to drop back to 10/18.)

---

### LOW

#### P3WV1C-A-L-001: ARCH-INDEX.md AD-001 crate count annotation misrepresents actual 8+8=16 workspace layout

- **Severity:** LOW
- **Category:** state-drift (Policy 6 — architecture_subsystem_source_of_truth)
- **Location:** `.factory/specs/architecture/ARCH-INDEX.md` line 65 (AD-001 row)
- **Description:** The AD-001 Architecture Decision row reads:

  > "Modular monolith via Cargo workspace with 16 crates (12 in Phase 1; +4 DTU crates
  > added in Phase 3 Wave 0–1)"

  The parenthetical annotation "(12 in Phase 1; +4 DTU crates)" is inconsistent with the
  actual workspace composition. PR #30 (f290f450) registered 6 previously-missing production
  crates into the workspace, bringing the full count to 16 total: 8 non-DTU crates (prism-core,
  prism-credentials, prism-mcp, prism-ocsf, prism-security, prism-spec-engine, prism-storage,
  ocsf-proto-gen) and 8 DTU test-only crates. "12 in Phase 1" is the original Phase 1 plan
  target, not the current state; the DTU count of "+4" applies only to the DTU crates added
  in Wave 0, not the full 8-crate DTU suite present after Wave 1.

  Additionally, `module-decomposition.md:20` plans 12 production crates total for Phase 1;
  only 7 currently exist (5 planned: prism-bin, prism-query, prism-sensors, prism-operations,
  prism-audit are Wave 2+ targets). The AD-001 annotation should reflect the actual current
  layout, not a projection.

- **Evidence:**
  - `ARCH-INDEX.md:65`: "(12 in Phase 1; +4 DTU crates)"
  - Post-PR-#30 workspace: 16 members (8 non-DTU + 8 DTU per TD-S620-001 resolution note)
  - `module-decomposition.md:20`: 12 production crates planned; 7 exist currently

- **Proposed Fix:** Update AD-001 to:

  > "Modular monolith via Cargo workspace; current workspace has 16 member crates (8 non-DTU
  > production/build-helper crates; 8 DTU test-only crates added in Phase 3 Waves 0–1).
  > Remaining Phase-1 production crates (prism-bin, prism-query, prism-sensors, prism-operations,
  > prism-audit) are targeted for Wave 2+."

---

### OBSERVATION

#### P3WV1C-A-OBS-001: wave_0a_complete date off-by-one — STATE.md 2026-04-21 vs wave-state.yaml 2026-04-22 (TD-CV-04)

- **Severity:** OBSERVATION
- **Category:** state-drift (existing tracked item TD-CV-04)
- **Location:**
  - `.factory/STATE.md` line 27 (`wave_0a_complete: 2026-04-21`)
  - `.factory/wave-state.yaml` line 14 (`gate_date: 2026-04-22`)
- **Description:** STATE.md records `wave_0a_complete: 2026-04-21`. The wave-state.yaml
  wave_0a block records `gate_date: 2026-04-22`. The two authoritative state files disagree
  by one day on when wave_0a's gate closed. This discrepancy is already tracked as TD-CV-04
  ("wave_0a_complete date off-by-one in STATE.md") with a due date of "next state-manager
  burst." The current remediation burst is the appropriate time to reconcile and close
  TD-CV-04.

  wave-state.yaml is the gate-date-of-record (it records the specific `gate_date` field
  for the wave gate). STATE.md's `wave_0a_complete` field should match.

- **Evidence:**
  - `STATE.md:27`: `wave_0a_complete: 2026-04-21`
  - `wave-state.yaml:14`: `gate_date: 2026-04-22`
  - `tech-debt-register.md` TD-CV-04 row: active, "next state-manager burst"

- **Proposed Fix:** Update STATE.md `wave_0a_complete: 2026-04-21` → `wave_0a_complete: 2026-04-22`.
  Mark TD-CV-04 RESOLVED in tech-debt-register.md Resolution History. After resolution, P2
  count drops from 11 to 10, active total from 19 to 18 (net-neutral relative to pre-M-001
  state; correct sequence: fix M-001 to 11/19, then resolve OBS-001 TD-CV-04 to 10/18).

---

## Summary

| Severity | Count |
|----------|-------|
| HIGH | 1 |
| MEDIUM | 1 |
| LOW | 1 |
| OBSERVATION | 1 |
| **Total** | **4** |

**Overall Assessment:** BLOCKED
**Convergence:** FINDINGS_REMAIN — remediate H-001 before Pass 4 cert; 3-pass clean window resets
**Novelty:** HIGH — 4 new findings on different surface than Pass 2 (Pass 3 focused on semantic anchoring, count drift, and annotation accuracy; not carry-forwards)

## Trajectory

| Pass | Findings | Delta | Notes |
|------|----------|-------|-------|
| Pass 1 | 11 | baseline | 1C+3H+3M+2L+2OBS — workspace CI, TLS, entropy, state drift |
| Pass 2 | 10 | −1 | 3H+4M+1L+2OBS — new surface; all Pass 1 findings closed |
| Pass 3 | 4 | −6 | 1H+1M+1L+1OBS — new surface; all Pass 2 findings closed |

**Trajectory shorthand:** `11 → 10 → 4`

## Remediation Priority

| Priority | Finding | Owner | When |
|----------|---------|-------|------|
| P1 (before Pass 4 cert) | P3WV1C-A-H-001 | state-manager (story amendment) | this burst |
| P1 (count correction) | P3WV1C-A-M-001 | state-manager | this burst |
| P1 (annotation accuracy) | P3WV1C-A-L-001 | state-manager | this burst |
| P1 (TD-CV-04 close) | P3WV1C-A-OBS-001 | state-manager | this burst |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 4 |
| **Carry-forwards from Pass 2** | 0 |
| **Novelty score** | 1.0 (4 / (4 + 0)) |
| **Median severity** | MEDIUM |
| **Trajectory** | 11 (pass 1) → 10 (pass 2, different surface) → 4 (pass 3, different surface) |
| **Verdict** | FINDINGS_REMAIN — BLOCKED; Pass 4 required |
