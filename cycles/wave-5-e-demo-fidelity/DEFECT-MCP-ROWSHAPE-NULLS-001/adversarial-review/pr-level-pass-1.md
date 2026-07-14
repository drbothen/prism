---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [1]
feature_head_at_review: d0140f6e
date: 2026-07-13
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 6
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 4
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 1 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 1 (frozen d0140f6e; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate; PR-LEVEL cascade begin; streak candidate 1/3 — NOT ADVANCING — 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 6 total (0 CRIT / 0 HIGH / 1 MED / 1 LOW / 4 OBS / 0 PROCESS-GAP)

**STREAK:** 0/3 — NOT CLEAN(strict); 1 MED finding present; streak does not advance. All 6 findings CLOSED fix-burst 14 (branch commits d0140f6e→91c8dc7f, 5 commits, PUSHED; PR #222 head confirmed 91c8dc7f; streak resets on push per DRIFT-ORCH-PRLEVEL-PUSH-001). NEW FROZEN HEAD 91c8dc7f.

**Code HEAD at review:** d0140f6e (frozen; fix-burst 13 staleness-gate redesign source-hash sidecar + HUMAN-APPROVED SEC-001 wasm-tools SHA-256 pin; pushed to origin; PR #222 OPEN base develop; 5461/5461 GREEN; non-exhaustive 91/91; CI fully green 2026-07-13)

**CLEAN(strict):** NO — 1 MED + 1 LOW + 4 OBS findings; strict criterion requires zero findings of any severity

**CLEAN(PR-merge):** YES — zero CRIT + HIGH + MED findings pending after fix-burst 14 closures

**SAP-1 result:** CLEAN — no new event_type emissions without BC-2.16.002 catalog rows

---

## Findings

### F-MCPRS-PRL1-MED-001 — CI supply-chain: staleness-gate false-pass vector (sidecar regen without rebuild)

**Severity:** MED
**Classification:** ci-supply-chain / staleness-gate integrity

**Finding:** The staleness-gate CI job (`.github/workflows/threatintel-staleness-gate.yml`) compared the source-hash sidecar (`.prx.src-tree-hash`) against a freshly-recomputed hash of the manifest inputs. However, an adversary could pass the gate by regenerating ONLY the `.prx.src-tree-hash` sidecar (e.g., re-running `scripts/compute_src_tree_hash.sh`) without rebuilding the `.prx` binary, leaving a stale `.prx` in the repository while the gate reports PASS. The equality-rule comparison (`diff <sidecar> <recomputed>`) was satisfiable via sidecar-regen alone — no proof that the `.prx` artifact was actually rebuilt from the updated manifest. An ancestry rule (`git merge-base --is-ancestor SIDECAR PRX`) would close this vector: the `.prx` commit must be a descendant of or equal to the sidecar commit for the gate to pass.

**Closure:** Fix-burst 14 implemented an architect-directed 3-iteration cascade:
- @9ed3bc1a: equality-rule commit-atomicity gate + `fetch-depth:0` + timeout 15→20m
- @37b5ea08: ancestry rule `git merge-base --is-ancestor SIDECAR PRX` replacing equality; README ancestry semantics documented
- @91c8dc7f: byte-identical-rebuild discovery (deterministic same-platform builds mean equality is always satisfiable without proving rebuild) → architect option (iii): plugin API version re-cut 1.0.0→1.0.1 (`lib.rs` `fn version()` literal + source manifest); 5 artifacts committed atomically; all 3 gate checks verified PASS locally (ancestry, freshness e887141e, manifest parity)

**Status:** CLOSED @91c8dc7f

---

### F-MCPRS-PRL1-LOW-001 — Test naming: H8b redundancy-sweep test name conflated catch-all vs internal category

**Severity:** LOW
**Classification:** test-naming / clarity

**Finding:** The test `test_BC_2_11_007_H8b_redundancy_sweep` in `crates/prism-mcp/tests/` covered the H8b (message/suggestion dedup) redundancy-sweep acceptance criterion but its name was generic "redundancy_sweep" without specifying the catch-all variant scope. The doc-comment described the test as a "catch-all for category 'internal'" when the actual purpose was specifically exercising H8b dedup across all internal-category variants. This caused a naming ambiguity: a future test-writer might add a separate "catch_all_variants" test duplicating the same coverage because the existing name did not signal its completeness.

**Closure:** @b6a2d517: test renamed `test_BC_2_11_007_H8b_redundancy_sweep_catch_all_variants`; doc-comment corrected to accurately describe the catch-all scope; prism-mcp 461/461 GREEN.

**Status:** CLOSED @b6a2d517

---

### F-MCPRS-PRL1-OBS-001 — Config placeholder: manifest prod-URL placeholder lacked story anchor

**Severity:** OBS
**Classification:** config-placeholder / Canonical Principle Rule 3

**Finding:** The `threatintel-lookup.manifest.toml` `allowed_urls` field listed dev-only endpoints (`localhost`, `127.0.0.1`) with a comment noting "production endpoint TBD." Per Canonical Principle Rule 3, deferring production configuration requires explicit human direction + concrete future dependency + story anchor. The placeholder comment contained no story anchor — a future implementer would have no tracking artifact for when/how to replace the dev-only list with the real production ThreatIntel API endpoint.

**Closure:** New draft story `S-MCP-THREATINTEL-PROD-ENDPOINT-001` authored (P2; wave unscheduled gated on business team identifying the production endpoint URL; traces BC-2.17.007 + BC-2.17.002; file `stories/S-MCP-THREATINTEL-PROD-ENDPOINT-001-threatintel-plugin-prod-endpoint.md`). Satisfies Canonical Principle Rule 3: explicit human direction (orchestrator per human session 2026-07-13) + concrete external dependency (production ThreatIntel endpoint not yet identified) + story anchor (S-MCP-THREATINTEL-PROD-ENDPOINT-001).

**Status:** CLOSED — story registered in this burst

---

### F-MCPRS-PRL1-OBS-002 — Pub field direct-assignment: resolved_spec_map and org_registry (23+3 callsites) vs builder pattern

**Severity:** OBS
**Classification:** encapsulation / API surface

**Finding:** `ResolvedQueryContext` had `pub resolved_spec_map` and `pub org_registry` fields directly assigned at 23 and 3 call sites respectively. Direct field assignment allows callers to construct partially-initialized structs in ways that bypass any future validation invariants the type might acquire. A builder pattern (`with_resolved_spec_map` / `with_org_registry`) would make the construction site explicit and allow enforcement of invariants at the single construction boundary.

**Closure:** @d12b50f2: `with_resolved_spec_map` and `with_org_registry` builder methods added; both fields changed `pub` → `pub(crate)`; 23 + 3 callsites migrated; zero direct-write assignments remain; workspace 5495/5495 GREEN via `just check`.

**Status:** CLOSED @d12b50f2

---

### F-MCPRS-PRL1-OBS-003 — Test citation: "ADR-051 §D2 null-input short-circuit" for unspecced behavior

**Severity:** OBS
**Classification:** spec-citation accuracy / TD-VSDD-091

**Finding:** Several tests in `crates/prism-mcp/tests/` cited "ADR-051 §D2 null-input short-circuit" as the behavioral contract authority for the guard that returns early when the source JSON is `None` or empty. ADR-051 §D2 specifies the `source_column` mandatory requirement, not a null-input short-circuit. The actual null-input guard is a pre-call short-circuit (applied before the full infusion UDF path) documented in the code but not in ADR-051 §D2. A future test-maintainer reading ADR-051 §D2 would not find the clause being exercised, and POL-22 code-truth correction was required.

**Closure:** @d12b50f2 (code side): 5 + 2 test citation sites corrected "null-input short-circuit" → "null-input guard"; zero stale citations remain. ADR-051 v1.4→v1.5 (spec side): null-input guard (pre-call short-circuit) sub-clause added to §D2; §D4 table row corrected: source IS called for JSON-list-to-typed path (POL-22 code-truth correction).

**Status:** CLOSED @d12b50f2 (code) + ADR-051 v1.5 (spec, this burst)

---

### F-MCPRS-PRL1-OBS-004 — Staleness-gate CI job timeout 15m: no cold-cache headroom

**Severity:** OBS
**Classification:** ci-reliability / timeout

**Finding:** The `.github/workflows/threatintel-staleness-gate.yml` job had a `timeout-minutes: 15` value. The job involves a cargo build of the plugin crate from scratch on GitHub Actions runners. Cold-cache builds of Rust crates with WASM toolchain setup can easily exceed 15 minutes, particularly on GitHub-hosted runners without warmed Rust caches. A timeout of 15m was insufficient for reliability on first-push or cache-eviction runs. Recommended minimum for this class of build: 20–25 minutes.

**Closure:** @9ed3bc1a (as part of MED-001 fix-burst cascade): timeout bumped 15→20m. Subsumed into the MED-001 fix-burst at architect's direction.

**Status:** CLOSED @9ed3bc1a (subsumed into F-MCPRS-PRL1-MED-001 fix)

---

## Summary

| Finding | Severity | Category | Closed At |
|---------|----------|----------|-----------|
| F-MCPRS-PRL1-MED-001 | MED | ci-supply-chain staleness-gate false-pass | @91c8dc7f (3-commit architect cascade) |
| F-MCPRS-PRL1-LOW-001 | LOW | test-naming H8b catch-all | @b6a2d517 |
| F-MCPRS-PRL1-OBS-001 | OBS | config-placeholder story anchor | S-MCP-THREATINTEL-PROD-ENDPOINT-001 registered |
| F-MCPRS-PRL1-OBS-002 | OBS | pub field direct-assignment 26 sites | @d12b50f2 builder pattern |
| F-MCPRS-PRL1-OBS-003 | OBS | test ADR citation accuracy | @d12b50f2 + ADR-051 v1.5 |
| F-MCPRS-PRL1-OBS-004 | OBS | CI timeout 15m | @9ed3bc1a (subsumed MED-001) |

**New frozen HEAD after fix-burst 14:** 91c8dc7f (pushed; PR #222 OPEN; CI running)

**PR-LEVEL streak:** 0/3 (DRIFT-ORCH-PRLEVEL-PUSH-001 reset on push of 91c8dc7f)

**NEXT:** PR-LEVEL pass 2 on frozen 91c8dc7f
