---
document_type: adversarial-review
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
cycle: phase-2-patch
pass: 35
previous_review: pass-34.md
status: findings-open
novelty: HIGH — fresh cross-referencing of api-surface/capabilities/error-taxonomy trios + Policy 8 sweep across SS-17/18/19 stories surfaced 2 Burst 35 regressions + 4 pre-existing systematic gaps
findings_total: 12
findings_crit: 2
findings_high: 6
findings_med: 3
findings_low: 0
findings_observational: 1
previous_pass: 34
convergence_counter: 0
date: 2026-04-19
---

# Adversarial Review: Prism (Pass 35)

## Finding ID Convention

Finding IDs use the format: `P3P35-A-{SEV}-NNN`

- `P3P35`: Pass 35 of the Phase 2 patch cycle
- `A`: Adversarial review prefix
- `{SEV}`: Severity abbreviation (`C` = CRITICAL, `H` = HIGH, `M` = MEDIUM, `L` = LOW, `O` = OBSERVATIONAL)
- `NNN`: Three-digit sequence within the pass

## Part A — Methodology and Corpus

**Corpus scanned (11 dimensions enumerated):**
1. `api-surface.md` — MCP Tool Registry, subsystem tags, Mermaid subgraph counts
2. `capabilities.md` — CAP-031 (Infusion Enrichment), CAP-032 (WASM Plugin Runtime), CAP-033 (Action Delivery Engine) tool enumerations
3. `error-taxonomy.md` — new v1.1 rows for E-PLUGIN/E-INFUSE/E-ACTION families
4. `ARCH-INDEX` — Subsystem Registry (authoritative SS-ID → name mappings)
5. `BC-INDEX` — BC frontmatter subsystem/CAP columns
6. `BC-2.17.*` files — WASM Plugin Runtime contracts
7. `BC-2.18.*` files — Action Delivery Engine contracts
8. `BC-2.19.*` files — Infusion Enrichment Framework contracts
9. `S-1.14` — Infusion enrichment story (AC traces)
10. `S-1.15` — WASM plugin story (AC traces)
11. `S-4.08` — Action delivery story (AC traces); `S-5.06` — Phase 3-patch tools story (behavioral_contracts frontmatter)

**Pass scope:** Burst 35 closure verification (H-001 CAP-022, M-001 error-taxonomy +18 rows, M-002 api-surface +8 rows). Policy 6 re-sweep (subsystem mis-anchoring). Policy 7 semantic integrity sweep (error code definitions). Policy 8 sweep across SS-17/18/19 story AC traces. Policy re-sweep on `policies.yaml`.

**Burst 35 closure verification:**

| Item | Previous Severity | Status | Notes |
|------|-------------------|--------|-------|
| H-001 CAP-022 tool list corrected (6 tools per BC-2.14.003) | HIGH | RESOLVED | capabilities.md CAP-022 now lists create_case, update_case, acknowledge_alert, list_cases, get_case, case_metrics |
| M-001 error-taxonomy +18 rows (E-ACTION-002..010, E-PLUGIN-004..008, E-INFUSE-002..005) | MEDIUM | RESOLVED | error-taxonomy.md v1.1 confirmed; 18 rows added with BC citations |
| M-002 api-surface +8 S-5.06 Tool Registry rows | MEDIUM | RESOLVED | api-surface.md v1.1 confirmed; 4 always-visible + 4 capability-gated rows added |

Burst 35 closures verified 3/3. New findings below are regressions introduced by Burst 35 edits and pre-existing gaps exposed by this pass's expanded sweep axes.

## Part B — New Findings

### CRITICAL

#### P3P35-A-C-001 — Subsystem mis-anchoring of 4 new S-5.06 tools in api-surface.md (inverted SS-17/SS-19). Burst 35 regression.

**Policy violated:** Policy 6 (architecture ↔ implementing spec subsystem consistency)
**Severity:** CRITICAL
**Confidence:** HIGH
**Novelty:** NEW — Burst 35 regression introduced during M-002 api-surface +8 rows fix

**Evidence:**
- `api-surface.md:133-138` attributes infusion tools to SS-17 and plugin tools to SS-18.
- ARCH-INDEX Subsystem Registry: SS-17 = WASM Plugin Runtime, SS-18 = Action Delivery Engine, SS-19 = Infusion Enrichment Framework.
- Correct mapping: `list_infusions`/`infusion_status` → SS-19; `list_plugins`/`plugin_status` → SS-17.
- BC frontmatter confirms: BC-2.17.* → SS-17, BC-2.19.* → SS-19.
- Changelog entry v1.1 on line 338 also has wrong SS IDs.

**Why it fails:** Tool routing logic and test-harness authors reading api-surface.md will wire infusion tools to the WASM plugin subsystem and plugin tools to infusion — a complete inversion that would cause runtime dispatch failures.

**Proposed Fix:** Rewrite rows 133-138 with correct subsystems (SS-19 for infusion tools, SS-17 for plugin tools); fix v1.1 changelog entry (line 338) to record correct SS IDs.

---

#### P3P35-A-C-002 — Error code double-definition: E-PLUGIN-001, E-PLUGIN-002, E-INFUSE-002, E-ACTION-004 have semantically incompatible meanings across error-taxonomy.md vs. BC bodies vs. stories.

**Policy violated:** Policy 7 (semantic integrity — no code may carry two distinct meanings)
**Severity:** CRITICAL
**Confidence:** HIGH
**Novelty:** NEW — introduced when error-taxonomy.md v1.1 added rows without reconciling pre-existing seed codes

**Evidence:**

- **E-PLUGIN-001:** taxonomy = "Plugin execution failed" (generic); BC-2.17.001 = "Plugin trap caught at Rust boundary"; BC-2.17.005/006 also use; capabilities.md CAP-032 uses for panic. (May be consistent under generic umbrella — flagged for explicit review and confirmation.)
- **E-PLUGIN-002:** taxonomy = "WIT interface incompatible"; BC-2.17.005 = "Plugin not loaded" (PluginError::NotLoaded); S-5.06:304,329 + S-1.15:161 = "Plugin not loaded"; capabilities.md CAP-032 = memory-limit OOM. **Three different meanings.**
- **E-INFUSE-002:** taxonomy = "Duplicate UDF name"; S-5.06:121 = "Infusion '{id}' not found". **Two different meanings.**
- **E-ACTION-004:** taxonomy = "action_state CF write failure during retry"; S-5.06:495 = "Cannot create action spec: config directory is not writable". **Two different meanings.**

**Why it fails:** Test-writers and implementers consulting the taxonomy will implement the wrong error semantics for E-PLUGIN-002, E-INFUSE-002, and E-ACTION-004. Assertion failures at runtime will be non-obvious because the codes appear defined.

**Proposed Fix options per code:**
- (a) Assign fresh codes for the BC/story semantics and register them; update all references.
- (b) Rewrite taxonomy v1.1 seed rows to match BC canonical semantics and update all consuming documents.

---

### HIGH

#### P3P35-A-H-001 — Dangling error refs E-PLUGIN-009 and E-PLUGIN-010 in BC-2.17.006. Orphans.

**Policy violated:** Policy 7 (error codes must be registered in error-taxonomy.md)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** NEW

**Evidence:**
- BC-2.17.006 body cites E-PLUGIN-009 (EC-17-025) and E-PLUGIN-010 (EC-17-027).
- error-taxonomy.md PLUGIN section defines E-PLUGIN-001..008 only.
- No rows exist for E-PLUGIN-009 or E-PLUGIN-010.

**Proposed Fix:** Add E-PLUGIN-009 (Plugin binary exceeds maximum size of 50MB, ref BC-2.17.006 EC-17-025) and E-PLUGIN-010 (`plugin_id` cannot be empty, ref EC-17-027) rows to error-taxonomy.md.

---

#### P3P35-A-H-002 — capabilities.md does not enumerate 8 of the 12 tools in api-surface.md (L2↔L3 asymmetry). Burst 35 touched capabilities.md only for CAP-022; did not propagate Burst 35 api-surface.md additions.

**Policy violated:** Policy 7 (L2 capabilities ↔ L3 api-surface tool enumeration must be consistent)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** NEW

**Evidence — tools missing from capabilities.md:**
- **CAP-031 (Infusion Enrichment):** `infusion_status`, `reload_infusion` absent from MCP tools prose
- **CAP-032 (WASM Plugin Runtime):** `list_plugins`, `plugin_status`, `reload_plugin` absent
- **CAP-033 (Action Delivery Engine):** `create_action`, `delete_action`, `test_action` absent

**Proposed Fix:** Add the 8 tool names to the MCP tools enumeration inside each capability's prose; bump capabilities.md → v1.2 with Burst 36 changelog entry.

---

#### P3P35-A-H-003 — list_actions and action_status mis-anchored to SS-12 (Scheduler) in api-surface.md.

**Policy violated:** Policy 6 (subsystem anchor integrity)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** Pre-existing drift — adjacent to Burst 35 surface; exposed by this pass's api-surface sweep

**Evidence:**
- `api-surface.md` tags `list_actions` and `action_status` to SS-12.
- ARCH-INDEX: SS-12 = Scheduler; SS-18 = Action Delivery Engine.
- `list_actions` and `action_status` are action-delivery tools, not scheduler tools.

**Proposed Fix:** Retag `list_actions` and `action_status` to SS-18 in api-surface.md.

---

#### P3P35-A-H-004 — Policy 8 violation in S-1.14: frontmatter declares BC-2.19.001..005 but all ACs trace only by INV-INFUSE-00N, zero by BC ID.

**Policy violated:** Policy 8 (bidirectional AC traceability — AC lines must cite BC IDs)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** Pre-existing — surfaced by first Policy 8 sweep of SS-19 story

**Evidence:** S-1.14 lines 177, 182, 187, 192, 197, 202 — each AC trace references `INV-INFUSE-00N` only; no BC-2.19.* IDs present despite frontmatter declaring BC-2.19.001..005.

**Proposed Fix:** Annotate each AC trace line with its BC ID (e.g., `(traces to BC-2.19.001 / INV-INFUSE-001)`).

---

#### P3P35-A-H-005 — Policy 8 violation in S-1.15: frontmatter declares BC-2.17.001..006 but ACs trace only by INV-PLUGIN-00N.

**Policy violated:** Policy 8 (bidirectional AC traceability)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** Pre-existing — surfaced by first Policy 8 sweep of SS-17 story

**Evidence:** S-1.15 lines 194, 199, 204, 209, 214, 219, 224, 239 — each AC trace references `INV-PLUGIN-00N` only; no BC-2.17.* IDs present.

**Proposed Fix:** Annotate each AC trace line with its BC ID (same remediation pattern as H-004).

---

#### P3P35-A-H-006 — Policy 8 violation in S-4.08: frontmatter declares BC-2.18.001..009 but only line 271 traces to a BC ID. Systematic pattern across Burst 1/2.5 stories for SS-17/18/19.

**Policy violated:** Policy 8 (bidirectional AC traceability)
**Severity:** HIGH
**Confidence:** HIGH
**Novelty:** Pre-existing — systematic pattern; same class as pass-31 sweep findings

**Evidence:** S-4.08 — 10 AC traces use `INV-ACTION-00N` only; only line 271 carries a BC ID. This pattern is systematic across the three Phase 3-patch subsystem stories (S-1.14, S-1.15, S-4.08).

**Proposed Fix:** Systematic sweep across S-1.14, S-1.15, S-4.08 annotating all AC traces with BC IDs.

---

### MEDIUM

#### P3P35-A-M-001 — capabilities.md CAP-032 prose still references stale E-PLUGIN-002 (memory) and E-PLUGIN-003 (CPU) codes; error-taxonomy.md v1.1 uses E-PLUGIN-006 (memory) and E-PLUGIN-007 (CPU).

**Policy violated:** Policy 7 (error code cross-reference integrity)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** Pre-existing — stale codes survived Burst 35 CAP-022 edit

**Evidence:**
- `capabilities.md` CAP-032 prose: references E-PLUGIN-002 (memory limit) and E-PLUGIN-003 (CPU limit).
- `error-taxonomy.md` v1.1 canonical: E-PLUGIN-006 (memory, ref BC-2.17.003) and E-PLUGIN-007 (CPU, ref BC-2.17.004).
- E-PLUGIN-002 and E-PLUGIN-003 carry different canonical meanings in v1.1 (see C-002).

**Proposed Fix:** CAP-032 prose: E-PLUGIN-002 → E-PLUGIN-006 (memory); E-PLUGIN-003 → E-PLUGIN-007 (CPU).

---

#### P3P35-A-M-002 — api-surface.md Mermaid header counts stale. Subgraph labels say "(24 Read Tools)" and "(20 Write Tools)"; actual counts are 28 always-visible and 22 capability-gated.

**Policy violated:** Policy 7 (internal document arithmetic integrity)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** NEW — Burst 35 +8 rows did not update the Mermaid subgraph header labels

**Evidence:**
- `api-surface.md:24` subgraph label: "(24 Read Tools)" — actual always-visible count: 28.
- `api-surface.md:51` subgraph label: "(20 Write Tools)" — actual capability-gated count: 22.

**Proposed Fix:** Update the two subgraph labels to reflect correct tool counts.

---

#### P3P35-A-M-003 — S-5.06 frontmatter declares `behavioral_contracts: []` but story materially implements tools exposed via BC-2.18.003, BC-2.17.005, BC-2.19.004, BC-2.05.001.

**Policy violated:** Policy 8 (frontmatter BC array must reflect all implemented BCs)
**Severity:** MEDIUM
**Confidence:** HIGH
**Novelty:** Pre-existing — empty array hides Burst 35 coverage impact

**Evidence:**
- S-5.06 `behavioral_contracts: []` in frontmatter.
- Story body implements: `fire_action`, `create_action`, `delete_action`, `reload_plugin`, `reload_infusion`, `test_action` — directly exposing behaviors spec'd in BC-2.18.003, BC-2.17.005, BC-2.19.004, BC-2.05.001.
- Empty array means cross-cycle BC coverage tracking omits S-5.06's contributions.

**Proposed Fix options:**
- (a) Populate frontmatter with cross-subsystem consumed BCs (BC-2.18.003, BC-2.17.005, BC-2.19.004, BC-2.05.001).
- (b) Add SS-10 BCs for MCP tool surface (e.g., BC-2.10.012 `fire_action` MCP Tool) and anchor S-5.06 to them.

---

### OBSERVATIONAL

#### P3P35-A-O-001 — BC-2.17.005 EC-17-021 references E-PLUGIN-001 for "WIT mismatch" — same ambiguity as C-002. Rolled into C-002 resolution.

**Severity:** OBSERVATIONAL
**Novelty:** Pre-existing — subsumed by C-002 fix

**Note:** When C-002 is resolved (E-PLUGIN-001 semantics clarified or re-assigned), verify BC-2.17.005 EC-17-021 is updated consistently.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 6 |
| MEDIUM | 3 |
| LOW | 0 |
| OBSERVATIONAL | 1 |
| **TOTAL** | **12** |

**Overall Assessment:** NOT CLEAN — BLOCK
**Convergence counter:** 0/3 (unchanged — Burst 35 introduced 2 CRIT regressions; counter cannot advance)
**Readiness:** Requires Burst 36 revision before next pass

### Verdict

Burst 35 closed its 3 target findings but introduced 2 CRIT regressions (C-001 SS-ID inversion in api-surface.md rows 133-138 and changelog; C-002 error code double-definition across four codes). Six HIGH findings are a mix of 2 Burst 35 propagation gaps (H-002 capabilities.md tool enumeration, H-003 list_actions/action_status SS tag) and 3 pre-existing Policy 8 AC-trace gaps (H-004/H-005/H-006) surfaced by this pass's first SS-17/18/19 Policy 8 sweep. Three MEDIUM findings are stale code refs and arithmetic (M-001/M-002) plus an empty behavioral_contracts array (M-003).

Convergence counter holds at **0/3**. Burst 36 must resolve C-001 and C-002 before counter can advance.

**Burst 36 recommended scope:**
1. **C-001** (architect): api-surface.md rows 133-138 — correct SS-ID tags (SS-19 for infusion, SS-17 for plugins); fix v1.1 changelog entry
2. **C-002** (PO): Resolve E-PLUGIN-002/E-INFUSE-002/E-ACTION-004 double-definitions; assign canonical meanings in error-taxonomy.md and propagate to all BC bodies and stories
3. **H-001** (PO): error-taxonomy.md — add E-PLUGIN-009 and E-PLUGIN-010 rows
4. **H-002** (architect): capabilities.md CAP-031/032/033 — add 8 missing tool names; bump to v1.2
5. **H-003** (architect): api-surface.md — retag list_actions/action_status to SS-18
6. **H-004/H-005/H-006** (story-writer): S-1.14, S-1.15, S-4.08 — annotate AC traces with BC IDs (systematic sweep)
7. **M-001** (architect): capabilities.md CAP-032 prose — E-PLUGIN-002→E-PLUGIN-006, E-PLUGIN-003→E-PLUGIN-007
8. **M-002** (architect): api-surface.md Mermaid labels — update subgraph counts (28 read, 22 write)
9. **M-003** (story-writer): S-5.06 frontmatter — populate behavioral_contracts array

**Trajectory:** 29→24→21→7→4→3→2→CLEAN→(reset)→26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→4→6→2→3→3→**12** (uptick — Burst 35 regressions + Policy 8 sweep expansion)
