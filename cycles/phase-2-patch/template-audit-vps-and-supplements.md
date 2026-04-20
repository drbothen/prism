---
document_type: audit-report
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
purpose: template-compliance-audit
scope: [verification-properties/, prd-supplements/]
traces_to: STATE.md
---

# Template Compliance Audit — VPs and PRD Supplements

> **Scope:** 39 VP files (`verification-properties/vp-NNN-*.md`) + 4 PRD supplement files
> **Reference:** Canonical VP shape per L4-verification-property-template; supplement canonical shapes
> **Audit date:** 2026-04-20
> **Constraint:** Read-only. No files were modified.

---

## VP Count Reconciliation vs VP-INDEX v1.5

VP-INDEX v1.5 declares **39 VPs** (VP-001–VP-039, continuous).

Filesystem glob finds **39 files**. Count matches.

| Check | Result |
|-------|--------|
| VP-INDEX total | 39 (32 P0 + 7 P1) |
| Filesystem count | 39 |
| Missing from filesystem | None |
| Extra on filesystem (not in index) | None |
| Count mismatch | **None — counts agree** |

---

## Section 1: VP Gap Inventory

### Canonical VP Shape (Template Reference)

**Required frontmatter fields:**
`document_type`, `level`, `version`, `producer`, `timestamp`, `inputs`, `input-hash`,
`traces_to`, `priority`, `status`, `verification_method`

**Required body sections:**
`## Property Statement`, `## Source Contract`, `## Proof Method`,
`## Proof Harness Skeleton`, `## Feasibility Assessment`, `## Lifecycle`

**Required if version > "1.0":**
`## Changelog`

---

### Gap Table

> **Key:** Missing frontmatter = canonical template field absent from YAML front matter.
> Missing section = required body heading absent. Notes flag additional deviations.

| File | VP ID | Missing Frontmatter Fields | Missing Body Sections | Version | Priority | Changelog | Notes |
|------|-------|---------------------------|----------------------|---------|----------|-----------|-------|
| vp-001-tenant-id-validation.md | VP-001 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method` instead of `verification_method`. `input-hash` populated. `traces_to: prd.md` (phase-1b pattern). Lifecycle section is table, no separate ## Lifecycle heading missing. `withdrawal_reason` ordering differs from 1c template (not missing). |
| vp-002-capability-deny-by-default.md | VP-002 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `input-hash` populated. `traces_to: prd.md`. |
| vp-003-capability-most-specific-wins.md | VP-003 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null` (should be `[]`). `introduced: cycle-1` (inconsistent with v1.0.0 in phase-1b VPs). Harness is TODO stub only. |
| vp-004-capability-deny-overrides-allow.md | VP-004 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub only. |
| vp-005-case-state-machine.md | VP-005 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `input-hash` populated. `traces_to: prd.md`. |
| vp-006-case-state-no-self-transitions.md | VP-006 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-007-confirmation-token-expiry-boundary.md | VP-007 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-008-confirmation-token-single-use.md | VP-008 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-009-confirmation-token-content-hash.md | VP-009 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-010-token-cap-100.md | VP-010 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-011-credential-name-path-traversal.md | VP-011 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-012-alias-depth-limit.md | VP-012 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-013-alias-cycle-detection.md | VP-013 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-014-query-oversized-rejection.md | VP-014 | `priority`, `verification_method` | — | 1.1 | P0 | Present | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. Changelog present for v1.1 — COMPLIANT. |
| vp-015-query-nesting-depth.md | VP-015 | `priority`, `verification_method` | — | 1.1 | P0 | Present | Uses `proof_method`. `modified: null`. Harness is TODO stub. Changelog present for v1.1 — COMPLIANT. |
| vp-016-ocsf-output-valid-protobuf.md | VP-016 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. `introduced: cycle-1`. Harness is TODO stub. |
| vp-017-ocsf-unmapped-fields-preserved.md | VP-017 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-018-detection-rule-validation.md | VP-018 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-019-diff-deterministic.md | VP-019 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-020-feature-flag-two-tier.md | VP-020 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-021-prismql-parser-no-panic.md | VP-021 | `priority`, `verification_method` | — | 1.1 | P0 | Present | Uses `proof_method`. `traces_to: prd.md` (phase-1b). `input-hash` populated. Changelog present — COMPLIANT. Harness is complete (not stub). |
| vp-022-ocsf-normalizer-no-panic.md | VP-022 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub (comment only). |
| vp-023-sensor-spec-parser-no-panic.md | VP-023 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub (comment only). |
| vp-024-injection-scanner-detects-known.md | VP-024 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-025-cache-key-deterministic.md | VP-025 | `priority`, `verification_method` | — | 1.0 | P1 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-026-splay-deterministic.md | VP-026 | `priority`, `verification_method` | — | 1.0 | P1 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-027-alert-dedup-key.md | VP-027 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-028-template-interpolation-no-panic.md | VP-028 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub (comment only). |
| vp-029-cursor-cap-200.md | VP-029 | `priority`, `verification_method` | — | 1.0 | P1 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-030-schedule-rule-caps.md | VP-030 | `priority`, `verification_method` | `## Changelog` | 1.1 | P1 | **MISSING** | Uses `proof_method`. `modified: 2026-04-19` (set). Changelog entry is embedded in `## Lifecycle` table body rather than in a separate `## Changelog` section — non-compliant for v1.1. |
| vp-031-required-column-enforcement.md | VP-031 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-032-hot-reload-atomicity.md | VP-032 | `priority`, `verification_method` | — | 1.0 | P1 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-033-audit-buffer-write-before-delivery.md | VP-033 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. Module correctly shows `prism-dtu-crowdstrike` per VP-INDEX reassignment. |
| vp-034-encryption-round-trip.md | VP-034 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-035-key-derivation-deterministic.md | VP-035 | `priority`, `verification_method` | — | 1.0 | P1 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. |
| vp-036-session-context-drop.md | VP-036 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Module correctly shows `prism-dtu-crowdstrike` per VP-INDEX reassignment. Harness is TODO stub. |
| vp-037-alias-expansion-no-panic.md | VP-037 | `priority`, `verification_method` | — | 1.0 | P1 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub (comment only). |
| vp-038-injection-scanner-no-panic.md | VP-038 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub (comment only). |
| vp-039-audit-forward-watermark-monotonic.md | VP-039 | `priority`, `verification_method` | — | 1.0 | P0 | N/A (v1.0) | Uses `proof_method`. `modified: null`. Harness is TODO stub. Module `prism-audit` (correct per VP-INDEX v1.5). |

---

### VP Summary

| Metric | Count |
|--------|-------|
| Total VPs audited | 39 |
| VPs with zero gaps | 0 |
| VPs with gaps | 39 |
| Compliance % (zero-gap) | **0%** |

> **Note on compliance scoring:** All 39 VPs share exactly two systematic frontmatter deviations from the canonical template. No VPs have missing body sections (except VP-030 which is missing `## Changelog` as a v1.1 document). The 0% zero-gap compliance reflects two universal field-naming issues, not substantive structural failures.

### VP Frequency Table — Missing Frontmatter Fields

| Missing Field | Count | % of VPs |
|---------------|-------|----------|
| `priority` | 39 | 100% |
| `verification_method` (filed as `proof_method`) | 39 | 100% |

> Both gaps are **systemic** — every VP uses `proof_method` in place of the template's `verification_method`, and none carry `priority` in frontmatter (priority lives in VP-INDEX only).

### VP Frequency Table — Missing Body Sections

| Missing Section | Count | % of VPs |
|-----------------|-------|----------|
| `## Changelog` (required for v > 1.0, absent) | 1 (VP-030) | 3% |

### Secondary Deviations (Non-Template-Gap, Informational)

| Deviation | Affected VPs | Count |
|-----------|-------------|-------|
| `modified: null` instead of `[]` | VP-003–039 except VP-001,002,005 | 35 |
| `introduced: cycle-1` instead of semantic version | All phase-1c VPs | 35 |
| `traces_to: prd.md` vs `architecture/verification-architecture.md` | VP-001,002,005,021 (phase-1b) | 4 |
| Harness is TODO stub (no concrete code) | VP-003–VP-039 except VP-021 | 34 |
| `input-hash: null` | VP-003–VP-039 except where populated | 34 |

> These deviations are **not template gaps** per the task's canonical shape definition; they are generational inconsistencies between phase-1b and phase-1c VP authoring patterns. Flagged for awareness.

### Worst Offenders (Most Gaps)

1. **VP-030** (vp-030-schedule-rule-caps.md) — 3 gaps: `priority` missing, `verification_method` naming wrong, `## Changelog` section missing despite v1.1.
2. All other 38 VPs — 2 gaps each: `priority` missing, `verification_method` naming wrong.

---

## Section 2: Supplement Gap Inventory

### Canonical Supplement Shape (Template Reference)

**Required frontmatter fields:** `document_type`, `level`, `version`, `status`, `producer`, `timestamp`, `inputs`, `input-hash`, `traces_to`

**Structural requirements:**
- error-taxonomy: error codes table with Code/Severity/Category/Message/Retryable/Description columns + Changelog
- interface-definitions: interface signatures with inputSchema/outputSchema + Changelog
- nfr-catalog: NFR table with Category/Requirement/Measurement/Traces_to rows + Changelog
- test-vectors: test-vector inventory with Input/Expected Output/Category/Notes tables + Change Log

### Gap Table

| File | Supplement Name | Missing Frontmatter Fields | Missing Sections | Version | Changelog | Notes |
|------|-----------------|---------------------------|-----------------|---------|-----------|-------|
| error-taxonomy.md | Error Taxonomy | `inputs`, `input-hash`, `traces_to` | — | 1.3 | Present (## Changelog at end) | Has `origin: greenfield` (non-standard field, not a gap). No `inputs`, `input-hash`, or `traces_to` in frontmatter — these are present in test-vectors.md and are part of the canonical supplement schema. Changelog is complete with 4 version entries. Structure fully compliant otherwise (all error namespace tables present). |
| interface-definitions.md | Interface Definitions | `inputs`, `input-hash`, `traces_to` | — | 2.2 | Present (## 5. Changelog at end) | No `inputs`, `input-hash`, or `traces_to` in frontmatter. Has `origin: greenfield` (non-standard). Changelog present with 3 version entries. All tool schemas complete. Section numbering consistent (1.1–1.49 + sections 2–5). |
| nfr-catalog.md | NFR Catalog | `inputs`, `input-hash`, `traces_to` | `## Changelog` | 1.0 | **MISSING** | No `inputs`, `input-hash`, or `traces_to` in frontmatter. Has `origin: greenfield`. No changelog section present at all. Version at 1.0 — changelog may not be required at v1.0, but the supplement has no traceability of changes. This is the most structurally sparse supplement. |
| test-vectors.md | Test Vectors | `input-hash` (present but `null`) | — | 2.3 | Present (## Change Log at end) | Has `inputs` and `traces_to` (only supplement to have them). `input-hash: null` — field present but unpopulated. Changelog complete with 5 entries (v1.0–v2.3). Traceability matrix present. Most compliant supplement. |

---

### Supplement Summary

| Metric | Count |
|--------|-------|
| Total supplements audited | 4 |
| Supplements with zero gaps | 0 |
| Supplements with gaps | 4 |
| Compliance % (zero-gap) | **0%** |

> If `input-hash: null` in test-vectors is treated as a filled-but-incomplete field rather than a missing-field gap, then test-vectors is borderline. All 4 still have at least one gap by strict reading.

### Supplement Frequency Table — Missing Frontmatter Fields

| Missing Field | Count | % of Supplements |
|---------------|-------|-----------------|
| `inputs` | 3 (error-taxonomy, interface-definitions, nfr-catalog) | 75% |
| `input-hash` | 4 (all — nfr/interface/error missing entirely; test-vectors has `null`) | 100% |
| `traces_to` | 3 (error-taxonomy, interface-definitions, nfr-catalog) | 75% |

### Supplement Frequency Table — Missing Sections

| Missing Section | Count | % of Supplements |
|-----------------|-------|-----------------|
| `## Changelog` | 1 (nfr-catalog) | 25% |

### Worst Offenders (Most Gaps)

1. **nfr-catalog.md** — 4 gaps: `inputs` missing, `input-hash` missing, `traces_to` missing, `## Changelog` missing.
2. **error-taxonomy.md** — 3 gaps: `inputs` missing, `input-hash` missing, `traces_to` missing.
3. **interface-definitions.md** — 3 gaps: `inputs` missing, `input-hash` missing, `traces_to` missing.
4. **test-vectors.md** — 1 gap: `input-hash: null` (unpopulated).

---

## Cross-Cutting Observations

### Systemic Issues

1. **`priority` absent from all VP frontmatter (39/39 VPs):** Priority is tracked exclusively in VP-INDEX.md. If the canonical L4 template requires it in-file, every VP needs a `priority:` field added. This is a single-burst mechanical fix across all 39 files.

2. **`proof_method` vs `verification_method` (39/39 VPs):** Every VP uses `proof_method` as the frontmatter key. The canonical template specifies `verification_method`. If the template is authoritative, all 39 files need a key rename. If `proof_method` was intentionally adopted as Prism's term, the template reference should be updated to match.

3. **`inputs`/`input-hash`/`traces_to` absent from 3/4 supplements:** Three supplements were authored without the drift-detection triad. Adding these fields enables `compute-input-hash` to track specification drift. The test-vectors.md authoring pattern (with all three fields) is the reference.

4. **`nfr-catalog.md` has no changelog:** The NFR catalog is the only supplement with no version history. Given it is at v1.0, this may be acceptable if no changes have been made — but it should be verified that the current NFR table is genuinely v1.0-unchanged.

### Non-Blocking Observations

5. **34/39 VPs have TODO-stub harnesses:** Harness skeletons are placeholders (`// [TODO: harness skeleton — author during Phase 5 formal-verify]`). This is the designed intent for Phase 2 artifacts — harnesses are authored in Phase 5. Not a template compliance gap, but noted for Phase 3 TDD build awareness.

6. **35/39 VPs have `modified: null`:** Phase-1c VPs set `modified: null` where phase-1b VPs use `modified: []`. Neither is wrong per se, but `[]` is more semantically precise for "no modifications yet." Mechanical fix in a single pass.

7. **VP-030's Lifecycle table doubles as changelog:** The modification history for VP-030 v1.1 is embedded in the `## Lifecycle` table rather than a dedicated `## Changelog` section. VP-014, VP-015, and VP-021 all have proper `## Changelog` sections. VP-030 should be made consistent.

---

## Remediation Priority Ranking

| Priority | Issue | Effort | Impact |
|----------|-------|--------|--------|
| P1 | Add `priority` frontmatter field to all 39 VPs | Mechanical, 39 file edits | Closes 100% of frontmatter gap |
| P1 | Resolve `proof_method` vs `verification_method` naming (rename or update template) | Decision + 39 edits OR 1 template edit | Closes 100% of naming gap |
| P1 | Add `inputs`, `input-hash`, `traces_to` to error-taxonomy, interface-definitions, nfr-catalog | 3 file edits | Enables drift detection on 75% of supplements |
| P2 | Add `## Changelog` section to VP-030 | 1 file edit | Closes sole body-section gap |
| P2 | Add `## Changelog` section to nfr-catalog.md | 1 file edit | Version history traceability |
| P3 | Normalize `modified: null` → `modified: []` across 35 phase-1c VPs | Mechanical, 35 edits | Consistency only |
| P3 | Populate `input-hash` across all 35 phase-1c VPs (currently `null`) | Requires compute-input-hash | Drift detection |
