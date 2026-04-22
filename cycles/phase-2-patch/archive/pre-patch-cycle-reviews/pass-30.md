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
pass: 30
previous_review: pass-29.md
cycle: phase-2-patch
novelty: MEDIUM
findings: 4
critical: 0
high: 0
medium: 3
low: 1
previous_pass: 29 (5 findings: 2 HIGH, 2 MED, 1 LOW — all 5 closed Burst 30 + 9 scripted-sweep drifts + 2 marker strips closed)
convergence_counter: 0 of 3
---

# Pass 30 — Scripted sweep VERIFIED (0 drifts in 2-col); novel drift in 3-col schema + Policy 8 AC gaps

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix from `.factory/current-cycle` — `P3PATCH` for phase-2-patch
- `<PASS>`: Two-digit pass number (`P30`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

This cycle uses short-form IDs (`P3P30-A-M-001` etc.) consistent with all prior passes in this cycle.

## Scope

Fresh-context review of full Prism Phase 2 spec+story package at commit `f94c17c`. Focus: Burst 30 pass-29 closure verification, **independent verification of scripted-sweep zero-drift claim**, Policy 2/6/8/9 comprehensive sweeps, fresh BC spot-check, test-vectors.md v2.1 structural integrity.

## Part A — Fix Verification (Burst 30 Closure)

All 5 pass-29 closures verified. Scripted sweep zero-drift claim **independently confirmed** via Grep-based walk of ~38 canonical 2-col BC rows against BC-INDEX v4.10 — 0 drifts found.

| Finding | Claim | Verified |
|---------|-------|----------|
| H-001 (S-1.10 line 41 BC-2.09.004) | Canonical "Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)" | CLOSED |
| H-002 (S-1.10 line 40 BC-2.09.003) | Canonical "with NFKC Normalization" | CLOSED |
| M-001 (S-1.12 lines 38/41/42) | 3 MCP tool names backticked | CLOSED |
| M-002 (S-1.08 line 41 BC-2.04.004) | Double-hyphen `--`, not em-dash | CLOSED |
| L-001 | 11 3-col schema stories deferred | AS-PLANNED |
| Scripted sweep claim | 0 title drifts across canonical 2-col stories | VERIFIED (0 drifts in independent walk) |
| Marker strip | 0 `[SCOPE EXPANSION]`/`[PHASE 3 PATCH]` in story bodies | VERIFIED (only changelog mentions remain) |

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

None.

### MEDIUM

#### P3P30-A-M-001 — S-1.05 description cell contradicts BC-2.02.008 H1 ("Three-tier" vs "Four-Tier")

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Policy violated:** 7 (bc_h1_is_title_source_of_truth), 4 (semantic_anchoring_integrity)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.05-ocsf-field-mapping.md` line 51
- **Novelty:** NEW — 3-col schema description cell drift; class skipped by scripted sweep (only scans canonical 2-col rows)
- **Description:** S-1.05 3-col BC table description cell uses "Three-tier" and a completely wrong tier taxonomy. BC-2.02.008 was explicitly renamed from Three-Tier to Four-Tier in BC-INDEX v4.6 with a note that the old title was factually wrong. S-1.05 never received the update.
- **Evidence:**
  - S-1.05 line 51: `| BC-2.02.008 | postcondition 1 | Three-tier field alias resolution: OCSF canonical → vendor alias → fallback |`
  - BC-INDEX v4.10 line 51 canonical title: `Four-Tier Field Alias Resolution`
  - BC-2.02.008 H1 (file line 23): `# BC-2.02.008: Four-Tier Field Alias Resolution`
  - BC postconditions (lines 30-34): enumerate FOUR tiers — (1) Prism metadata, (2) Proto descriptor fields, (3) raw_extensions JSON, (4) None
  - BC-INDEX v4.6 changelog explicitly reconciled: "BC-2.02.008: 'Three-Tier' → 'Four-Tier' (BC body confirmed 4 tiers)"
- **Why it fails:** Implementer reading S-1.05 from body writes 3-tier resolver with wrong tier semantics — regresses vs BC-2.02.008 postconditions. 3-col schema doesn't require verbatim title match but description cell MUST NOT contradict BC body.
- **Proposed fix:** Line 51 → `| BC-2.02.008 | postcondition 1 | Four-tier field alias resolution: Prism metadata → Proto descriptor fields → raw_extensions JSON → None |`.

---

#### P3P30-A-M-002 — S-1.10 frontmatter has 3 BCs with no AC trace (Policy 8)

- **Severity:** MEDIUM
- **Category:** ac-coverage
- **Policy violated:** 8 (bc_array_changes_propagate_to_body_and_acs)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.10-prompt-injection-defense.md`
- **Novelty:** NEW — first systematic bidirectional Policy 8 audit this cycle
- **Description:** S-1.10 declares 8 BCs in frontmatter but 3 have no AC exercising their postconditions. All 3 are P0/P1 security contracts.
- **Evidence:**
  - Frontmatter line 20 behavioral_contracts: 8 BCs (BC-2.09.001 through .008)
  - AC traces (lines 61-65): AC-1→BC-2.09.003/.004; AC-2→BC-2.09.005/.008; AC-3→BC-2.09.002; AC-4→BC-2.09.004; AC-5→VP-024
  - **No AC traces to:** BC-2.09.001 (Structural Separation of Untrusted Data), BC-2.09.006 (Tool Description Security Warnings), BC-2.09.007 (OutputSchema for Type-Safe LLM Reasoning)
  - All 3 present in body BC table (lines 38/43/44) — frontmatter/body consistent, but no AC exercises their postconditions
- **Why it fails:** Policy 8 Step 2 requires "at least one AC per BC in frontmatter". Three P0/P1 security contracts have no test vehicle in acceptance criteria.
- **Proposed fix:** Add 3 new ACs or extend existing ACs:
  - AC-6 for BC-2.09.001 (structural separation of `data` vs `_meta`)
  - AC-7 for BC-2.09.006 (provenance warning in tool description registration)
  - AC-8 for BC-2.09.007 (OutputSchema structured return types)

---

#### P3P30-A-M-003 — S-1.08 frontmatter has 1 BC with no AC trace (Policy 8)

- **Severity:** MEDIUM
- **Category:** ac-coverage
- **Policy violated:** 8 (bc_array_changes_propagate_to_body_and_acs)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.08-feature-flags.md`
- **Novelty:** NEW
- **Description:** S-1.08 declares 8 BCs in frontmatter; BC-2.04.003 (Hierarchical Capability Resolution — BTreeMap, Most-Specific-Path Wins, Deny Support) has no AC exercising it despite being referenced in Task 3.
- **Evidence:**
  - Frontmatter line 20: 8 BCs (BC-2.04.001/.002/.003/.004/.005/.006/.013/.015)
  - AC traces (lines 61-67): AC-1→BC-2.04.001/.004; AC-2→BC-2.04.002; AC-3→BC-2.04.005; AC-4→BC-2.04.006; AC-5→BC-2.04.013; AC-6→BC-2.04.015; AC-7→VP-020
  - **No AC trace to:** BC-2.04.003 (Hierarchical Capability Resolution — BTreeMap, Most-Specific-Path Wins, Deny Support)
  - BC-2.04.003 in body BC table (line 40) and Task 3 (line 51 references it) but no AC exercises BTreeMap / most-specific / deny-override
- **Why it fails:** BC-2.04.003 is P0 (BC-INDEX line 70); three distinct behavioral commitments deserve AC coverage.
- **Proposed fix:** Add AC verifying most-specific-path-wins or deny-override (e.g., "Given `defaults.capabilities = {read: Allow}` and `clients.acme.capabilities = {crowdstrike.read: Deny}`, When check_permission(acme, crowdstrike.read) is called, Then it returns Deny (most-specific wins)") tracing BC-2.04.003.

---

### LOW

#### P3P30-A-L-001 — S-1.10 Task 4 uses stale "parallel fields" wording contradicting BC-2.09.004

- **Severity:** LOW
- **Category:** spec-fidelity
- **Policy violated:** 4 (semantic_anchoring_integrity)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.10-prompt-injection-defense.md` line 52
- **Novelty:** NEW (secondary manifestation of BC-2.09.004 rename — body table correct, Task prose stale)
- **Description:** Task 4 prose says "parallel fields" and "`_safety_flags` array" simultaneously — internally contradictory and contradicts BC-2.09.004 which explicitly prohibits per-field parallel flags.
- **Evidence:**
  - Line 52: `4. Implement \`SafetyFlag\` parallel fields: \`_safety_flags\` array with \`{field, pattern, category}\` — original data never stripped (BC-2.09.004)`
  - Internally contradictory: "parallel fields" and "`_safety_flags` array" describe incompatible models
  - BC-2.09.004 H1: "Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field)"
  - BC-2.09.004 postconditions: "NO per-field parallel `{field_name}_safety_flag` fields. Safety information is centralized in `_meta.safety_flags` only."
- **Why it fails:** Implementer may implement both array and per-field flags "to cover both interpretations" (introducing the anti-pattern explicitly prohibited by BC).
- **Proposed fix:** Rewrite Task 4 as: `Implement centralized safety flag recording: append each detection to \`_meta.safety_flags\` array with \`{field, index, pattern, category}\` objects — original data never stripped; NO per-field \`{field}_safety_flag\` parallel fields (BC-2.09.004)`.

---

## Observations

1. **Scripted sweep independently verified — genuine validation.** Walked ~38 stories with canonical 2-col BC tables. Every title matches BC-INDEX v4.10 verbatim including backticks, em-dashes, and all Burst 19/21/28 enrichments.

2. **BC-INDEX arithmetic clean:** 195 + 6 + 2 = 203 ✓; Summary sums to 166 P0 + 29 P1 = 195 active ✓.

3. **VP-INDEX Policy 9 clean:** 39 VPs (20 Kani + 11 Proptest + 6 Fuzz + 2 Integration); per-module coverage-matrix sums match; all 39 in verification-architecture.md ✓.

4. **Policy 2 DI orphan scan clean:** All 28 active DIs cited by at least one BC L2 Invariants field.

5. **Policy 6 ARCH-INDEX sync clean:** Sampled 15+ stories; all subsystems: fields use valid registry IDs.

6. **test-vectors.md v2.1 structural integrity:** All frontmatter fields, required body sections, 10 TVs, VP-034 only in changelog (historical).

7. **S-1.14/S-1.15 AC traces use INV-* invariant names** (INV-INFUSE-001..005 map 1:1 to BC-2.19.001..005; INV-PLUGIN-001..006 to BC-2.17.001..006). Coverage traceable indirectly. Strict Policy 8 interpretation would flag 11 ACs; marginal — not raising.

8. **3-col schema description cells are a drift-prone axis.** S-1.05 M-001 demonstrates the cost of deferring 3-col→2-col migration: scripted sweep can't cover these. Future burst should decide whether to un-defer migration OR extend the sweep to parse 3-col descriptions.

## Novelty Assessment

**Novelty: MEDIUM.** All 4 findings genuinely new:
- M-001: 3-col description drift (pre-existing since BC-INDEX v4.6 Burst 19 — a class of drift the scripted sweep methodology explicitly skips)
- M-002/M-003: First systematic per-story Policy 8 bidirectional AC-trace audit this cycle
- L-001: Secondary manifestation of BC-2.09.004 rename in Task prose

Trajectory: 26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→**4**. No HIGH for first time this cycle. CRIT=0 streak holds (16+ consecutive passes).

## Convergence Recommendation

**BLOCK at 0/3.** 4 surgical findings prevent clean-pass. But novelty of findings is narrow (3-col description axis + Policy 8 AC traces in 2 stories) — easily remediable.

**Burst 31 scope (all surgical):**
1. M-001: S-1.05 line 51 description "Three-tier..." → "Four-tier: Prism metadata → Proto descriptor fields → raw_extensions JSON → None"
2. M-002: S-1.10 add AC-6/7/8 for BC-2.09.001/.006/.007
3. M-003: S-1.08 add AC tracing BC-2.04.003 (most-specific-path-wins or deny-override)
4. L-001: S-1.10 Task 4 rewrite to centralized-array wording
5. (Optional) discuss un-deferring 3-col schema migration

Expected post-Burst-31: 0 findings → counter advances to 1/3.

## Summary

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 3 | P3P30-A-M-001, P3P30-A-M-002, P3P30-A-M-003 |
| LOW | 1 | P3P30-A-L-001 |
| **Total** | **4** | |

**Convergence counter:** 0/3 (BLOCKED — findings open)
**Novelty:** MEDIUM
**Trajectory:** ...9→5→5→**4** (first pass with no HIGH this cycle)

## Relevant Files

- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-INDEX.md` (v4.10)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.02.008-field-alias-resolution.md` (canonical "Four-Tier")
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.09.004-safety-flag-parallel-fields.md` (canonical centralized-array model)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.04.003-*.md` (hierarchical resolution P0)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.09.001-*.md`, `.006-*.md`, `.007-*.md`
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` (v2.1)
- `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` (v1.22)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.05-ocsf-field-mapping.md` (M-001 line 51)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.08-feature-flags.md` (M-003 lines 20, 61-67)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.10-prompt-injection-defense.md` (M-002 lines 20, 59-65; L-001 line 52)
