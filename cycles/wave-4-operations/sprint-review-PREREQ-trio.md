---
document_type: sprint-review-brief
level: ops
version: "1.0"
producer: state-manager
timestamp: 2026-05-12T23:45:00Z
inputs:
  - .factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md
  - .factory/stories/S-PLUGIN-PREREQ-B-real-pipeline-executor.md
  - .factory/stories/S-PLUGIN-PREREQ-C-toml-grammar-extensions-plus-pub-api-hardening.md
  - .factory/STATE.md
  - .factory/tech-debt-register.md
  - .factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md
input-hash: "[live-state]"
traces_to: ""
cycle: wave-4-operations
sprint_scope: "PREREQ keystone trio (S-PLUGIN-PREREQ-A + -B + -C)"
stories_merged: [S-PLUGIN-PREREQ-A, S-PLUGIN-PREREQ-B, S-PLUGIN-PREREQ-C]
develop_head_at_brief: ea958a4d
sprint_window: "2026-05-11 through 2026-05-12"
---

# PREREQ Keystone Trio — Sprint Review Analysis

## 1. Sprint Summary

- **sprint_name:** PREREQ keystone trio (post-PREREQ-F → post-PREREQ-C arc) within cycle `wave-4-operations`
- **stories_completed:** 3 (S-PLUGIN-PREREQ-A, -B, -C — all merged to develop)
- **total_points:** 34 (13 + 13 + 8)
- **epics_touched:** 1 (PLUGIN-MIGRATION-001, Wave 0 prerequisite layer)
- **completion_rate:** 100% of planned PREREQ keystone trio (3 of 3); Wave 0 prerequisites remaining: 2 stories (PREREQ-D, PREREQ-E)
- **key_metrics:**
  - Merged commits: `90d7c80f` (A), `ae7e26c8` (B), `ea958a4d` (C)
  - Aggregate diff: 183 files changed, ~18,633 insertions, ~2,690 deletions across trio
  - Workspace test count after trio: **3,598 tests pass** (just check clean per STATE.md frontmatter)
  - LOCAL adversary passes: A=12, B=16, C=5 → 33 total LOCAL passes
  - LOCAL fix-bursts: A=7, B=13, C=4 → 24 total fix-bursts
  - PR-LEVEL passes: A=4, B=1, C=1
  - CI checks at C merge: **36/36 PASS** (D-442)
  - BC catalog: BC-2.16.002 evolved v1.4 → v1.10; Structured Event Catalog grew to 16 rows
  - Non-exhaustive types audited: **30** (AC-5 catalog grew 8 → 14 → 30 across cascade)

## 2. Epic Breakdown

### Epic PLUGIN-MIGRATION-001: Plugin-Only Sensor Architecture (Wave 0 prerequisites — 3 of 5 stories)

| Story | Points | Crates Touched | Deliverable |
|-------|--------|----------------|-------------|
| S-PLUGIN-PREREQ-A | 13 | prism-core, prism-sensors, prism-query, prism-spec-engine, prism-dtu-* | Deleted closed `enum SensorType { CrowdStrike, Cyberint, Claroty, Armis }`; replaced with open `SensorId(Arc<str>)` newtype (crates/prism-core/src/sensor_id.rs, 554 lines); 7 dispatch sites converted; perimeter compile-fail E0432 CI gate (VP-PLUGIN-001); 11/11 ACs; 6/6 Red Gate tests at BC-prefixed names; PR-LEVEL caught `pub type SensorId = String` shadow alias in prism-query::cache_key and migrated 30+ sibling sites |
| S-PLUGIN-PREREQ-B | 13 | prism-spec-engine | Replaced `Ok(Vec::new())` "architectural fraud" stub (ADR-023 §C2) with real `PipelineExecutor::execute`: HTTP client DI, multi-step variable interpolation, cursor+offset pagination (MAX_PAGES_PER_STEP=1000), fan-out batching, 401-retry via new `AuthProvider` trait (object-safe, with `NullAuthProvider`/`MockAuthProvider`), eager-token semantics, RFC 6901 JSON Pointer, 10K DI-019 truncation guard; 9/9 ACs; **64 Red Gate tests**; BC-2.16.002 v1.4→v1.9 with **14-row Structured Event Catalog** for all `tracing::*!(event_type=…)` sites; AuthToken redacted-Debug (AD-017) |
| S-PLUGIN-PREREQ-C | 8 | prism-spec-engine, prism-core | (1) TOML grammar: `page_size` on cursor pagination (CrowdStrike `first: N`); JSONPath `[N]` index + `[*]` wildcard in `extract_at_path`; `$${…}` Interpolator literal-escape; (2) **30 `#[non_exhaustive]` types audited** (sibling-sweep from 8 → 14 → 30) with `Default` impls + constructors + CI compile-fail gate `non-exhaustive-violation-compile-fail` (E0639) modeled on PREREQ-A perimeter-violation pattern; (3) prism-core hardening: `SensorIdValidationError` re-exported at crate root; `OrgSlug::new_unchecked` (+ siblings) feature-gated behind `cfg(test-helpers)`; proptests for `fan_out_batches` totality + `extract_at_path` totality + `Interpolator` round-trip; 7/7 ACs |

**Key achievements:** The trio is the load-bearing keystone for ADR-023 v1.18 plugin-only sensor architecture. PREREQ-A made sensor identity a runtime string. PREREQ-B made the spec engine actually execute. PREREQ-C made the pub-API surface safe for plugin schema evolution. Together they unblock all 4 lettered Wave 1 stories (PLUGIN-MIGRATION-001-A/B/C/D).

## 3. Business Linkage

| Deliverable | What It Enables | MSSP Analyst Business Outcome |
|-------------|-----------------|-------------------------------|
| `SensorId(Arc<str>)` open newtype (PREREQ-A) | Sensor identity is data, not code. New sensor integrations no longer require a core-crate enum variant + recompile + re-ship. Plugin TOML specs register new sensors at boot without touching prism-core. | MSSP managing 30+ client environments with heterogeneous sensor estates can add vendor sensors per-client without waiting for a prism-core release cycle. Analyst tooling stays current as client sensor zoo grows. |
| Real `PipelineExecutor` + `AuthProvider` trait (PREREQ-B) | The spec engine actually executes multi-step fetch pipelines against live sensor APIs. OAuth2 refresh-on-401, cursor/offset pagination, fan-out batching, 10K record truncation guard — all wired and contractually tested. | Analyst `LIST sensors` and query execution calls now return real sensor data rather than empty stubs. Multi-step pipelines (authenticate → paginate → fan-out) work end-to-end, which is the actual CrowdStrike and Cyberint query pattern. |
| BC-2.16.002 Structured Event Catalog — 16 rows (PREREQ-B → v1.10) | Every `tracing::*!(event_type=…)` emission in the spec engine is contractually enumerated with file:line anchor, log level, and test coverage citation. Dashboards built on these events have stable field contracts. | Ops teams can build Grafana dashboards on `prism_pipeline_*` event types with confidence they won't silently disappear across versions. Ingestion debugging is operationally tractable: the catalog is the runbook for "what do these log events mean?" |
| 30 `#[non_exhaustive]` types + CI compile-fail gate (PREREQ-C) | External plugin code that pattern-matches on TOML-deserialized types gets a compile error (not a silent `_` match) when a new field is added to a spec struct. Plugin authors are forced to handle new fields explicitly. | Plugin schema evolution (e.g., adding `retry_policy` to `FetchStep`) does not silently break customer-authored plugins that exhaustively match on the struct. MSSP plugin lifecycle is safer as the sensor spec vocabulary grows across client versions. |

## 4. Convergence Efficiency

| Story | LOCAL passes | Fix-bursts | PR-LEVEL passes | Pass-1 findings | Trajectory |
|-------|-------------|-----------|-----------------|-----------------|------------|
| PREREQ-A | 12 | 7 | 4 | 14 | 14→12→6→4→2→6→4→0→4→0→0→0 |
| PREREQ-B | 16 | 13 | 1 | 20 | 20→10→4→7→10→9→8→4→4→2→3→3→2→1→0→0 |
| PREREQ-C | 5 | 4 | 1 | 18 | 18→8→5→5→1 |

**Acceleration: 12 → 16 → 5 LOCAL passes. PREREQ-C converged 2.4–3.2× faster than PREREQ-A/B.**

Hypothesis for acceleration:

1. Methodology TDs landed mid-cascade (TD-VSDD-053/059/060/091, Standing Rules)
2. PG-LP11-001 SOP codified at PREREQ-B D-419 (BC↔tracing-emission discipline)
3. Scope discipline (D-428) — PREREQ-C scoped to 7 carry-forward TDs as ACs, preventing scope creep
4. Pattern reuse — PREREQ-C compile-fail CI gate lifted directly from PREREQ-A perimeter pattern

## 5. Tech Debt Scoreboard

**Closed by trio (7 TDs, verified at tech-debt-register.md v2.15):**

| TD | Priority | Description | Closed by |
|----|----------|-------------|-----------|
| TD-S-PLUGIN-PREREQ-B-001 | P2 | `page_size` on cursor pagination | PREREQ-C PR #144 |
| TD-S-PLUGIN-PREREQ-B-003 | P3 | JSONPath bracket/wildcard in `extract_at_path` | PREREQ-C PR #144 |
| TD-S-PLUGIN-PREREQ-B-006 | P2 | Proptest coverage for pure pipeline functions | PREREQ-C PR #144 |
| TD-S-PLUGIN-PREREQ-B-008 | P3 | Interpolator `$${…}` literal-escape | PREREQ-C PR #144 |
| TD-S-PLUGIN-PREREQ-B-016 | P2 | `#[non_exhaustive]` crate-wide audit | PREREQ-C PR #144 |
| TD-S-PLUGIN-PREREQ-A-006 | P3 | Cross-newtype `OrgSlug::new_unchecked` audit | PREREQ-C PR #144 |
| TD-S-PLUGIN-PREREQ-A-008 | P3 | `SensorIdValidationError` re-export at crate root | PREREQ-C PR #144 |

**Active TD count: 91** (was 98 at PREREQ-A start; 7 closed by trio).

**Most impactful open TDs emerging from arc:**

| TD | Priority | Description |
|----|----------|-------------|
| TD-S-PLUGIN-PREREQ-A-002 | P1 | WriteDispatcher sentinel-nil OrgId — production write dispatch structurally broken |
| TD-S-PLUGIN-PREREQ-A-003 | P1 | WriteToolInvalidationMap read-only `LazyLock` — runtime plugin write-tool registration impossible |
| TD-S-PLUGIN-PREREQ-A-004 | P1 | AdapterRegistry empty-after-boot deferred assertion — silent empty results possible |
| TD-S-PLUGIN-PREREQ-B-002 | P3 | `AuthToken` missing `zeroize`-on-drop — token bytes linger in heap |
| TD-S-PLUGIN-PREREQ-B-004 | P3 | `MAX_REQUESTS_PER_PIPELINE` unimplemented — cumulative request cap missing |
| TD-S-PLUGIN-PREREQ-B-005 | P2 | Production `reqwest::Client` missing timeout — slow-loris hang in production wiring |
| TD-S-PLUGIN-PREREQ-B-007 | P3 | `HttpRequestFailed.status_code:0` overloaded across 11 distinct error origins |
| TD-VSDD-091 | P3 | Volatile line-citations — Step 2 maintenance sweep scope |
| TD-VSDD-093 | P3 | lefthook pre-commit hook for BC↔tracing-emission discipline (PG-LP11-001 Layer 4) |

## 6. Process Gap Insights

**Codified during arc (9 process improvements):**

1. **TD-VSDD-053** — Single-commit-per-burst protocol (factory-dispatcher `MULTI_COMMIT_CHAIN_NOT_ALLOWED` guard; prevents "Stage 1 / Stage 2 backfill" loop)
2. **TD-VSDD-059** — Paper-fix detection (D-374, P0): every adversary pass re-derives closures from source-of-truth; claims must include file:line grep evidence
3. **TD-VSDD-060** — Sibling-site sweep on value changes: any rename/value-change triggers codebase-wide grep before closure is declared
4. **TD-VSDD-091** — Volatile line-citation anti-pattern: spec docs MUST NOT cite file:line positions as stable identifiers (line numbers shift with every edit)
5. **PG-LP11-001** — BC↔tracing-emission discipline (D-419+D-422; 4-layer enforcement): implementer self-check, story AC coverage check, adversary positive-assertion check, lefthook hook pending)
6. **POL-10** — Demo evidence story-scoped (PREREQ-C 8-file 835-line index per D-441)
7. **POL-11** — Positive-coverage logging in CI gates: compile-fail tests MUST emit a successful compilation line proving the gate fires
8. **POL-14** — Auto-promotion of BCs at merge: draft → active triggered by squash-merge event, not by separate state-manager burst
9. **BC-5.39.001** — 3-CLEAN convergence protocol locked in by all three stories: streak of 3 consecutive CLEAN adversary passes required before PR delivery

## 7. Next-Wave Readiness

**PLUGIN-MIGRATION Wave 1 dependency check (from STORY-INDEX.md:394-401):**

| Story | Depends On | Status |
|-------|-----------|--------|
| PLUGIN-MIGRATION-001-A | S-PLUGIN-PREREQ-A/B/C/E + 001-D + 001-E | A/B/C MERGED; E + 001-D + 001-E pending |
| PLUGIN-MIGRATION-001-B | S-PLUGIN-PREREQ-A/C + 001-A | A/C MERGED; 001-A pending |
| PLUGIN-MIGRATION-001-C | S-PLUGIN-PREREQ-C/D + 001-A | C MERGED; D + 001-A pending |
| PLUGIN-MIGRATION-001-D | S-PLUGIN-PREREQ-B/C + 001-A | B/C MERGED; **001-A listed as dep — apparent cycle** |

**CRITICAL INCONSISTENCY flagged for Step 2 (per D-444 cycle resolution note):**

STORY-INDEX:397 has `PLUGIN-MIGRATION-001-D` depends_on `[S-PLUGIN-PREREQ-B, S-PLUGIN-PREREQ-C, PLUGIN-MIGRATION-001-A]` — but STORY-INDEX:394 has `PLUGIN-MIGRATION-001-A` depends_on `[..., PLUGIN-MIGRATION-001-D, ...]`. This is an apparent cycle: 001-A → 001-D and 001-D → 001-A.

Per D-444 resolution: `001-D` should depend on `PREREQ-A/B/C/D` (NOT 001-A); `001-A` then depends on `001-D`. This is a data error in STORY-INDEX:397 introduced at D-334. **Flag for Step 2 maintenance burst before any Wave 1 dispatch.**

True Wave 1 order (post-fix): PREREQ-D → PREREQ-E → 001-D → 001-E → 001-A → [001-B || 001-C].

**Other inconsistency flagged:**

- PREREQ-C adversarial pass reports live under `.factory/code-delivery/S-PLUGIN-PREREQ-C/adversarial-review/` while PREREQ-A/B use `.factory/cycles/wave-4-operations/adversarial-reviews/`. Path-convention drift, cosmetic — flag for maintenance sweep.

## 8. Demo Points (MSSP Analyst Framing)

1. **"Sensor identity is now data, not code."** — `SensorId(Arc<str>)` replaces closed `enum SensorType { CrowdStrike, Cyberint, Claroty, Armis }`. Compile-fail CI gate (VP-PLUGIN-001) structurally prevents regression to named-sensor dispatch. PR #142 (`90d7c80f`).

2. **"The spec engine actually executes now."** — Real `PipelineExecutor::execute` (6,650 line-add at PR #143, `ae7e26c8`) replaces the `Ok(Vec::new())` architectural-fraud stub that ADR-023 §C2 identified as the primary blocker for plugin-based sensor integration.

3. **"Operational observability is now contractual."** — BC-2.16.002 v1.10 enumerates 16 `tracing::*!(event_type=…)` emissions in a Structured Event Catalog. Dashboard authors and ops teams have stable field contracts to build against.

4. **"Plugin schema can evolve without breaking customers."** — 30 `#[non_exhaustive]` types across `prism-spec-engine` + CI compile-fail gate (E0639) ensures that adding a field to `FetchStep`, `PaginationConfig`, or `SensorSpec` forces plugin authors to handle the new field explicitly.

5. **"Convergence is getting faster."** — 12 → 16 → 5 LOCAL adversary passes across the trio. 9 process gaps codified during the arc as standing methodology (TD-VSDD-053, PG-LP11-001, POL-10/11/14, BC-5.39.001). The discipline improvements are self-reinforcing.

## Files Referenced

**Story specs:**
- `.factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md` (v1.6, status: merged)
- `.factory/stories/S-PLUGIN-PREREQ-B-real-pipeline-executor.md` (v1.18, status: merged)
- `.factory/stories/S-PLUGIN-PREREQ-C-toml-grammar-extensions-plus-pub-api-hardening.md` (v1.4, status: merged)

**Pipeline state:**
- `.factory/STATE.md` (v7.178 at brief generation)
- `.factory/SESSION-HANDOFF.md` (v7.178 at brief generation)

**Tech debt:**
- `.factory/tech-debt-register.md` (v2.15 — 91 active TDs; 7 closed by trio)

**Behavioral contracts:**
- `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md` (v1.10, active)
- `.factory/specs/behavioral-contracts/BC-2.01.013-sensor-identity-open-newtype.md` (v1.6, active)

**Merge commits:**
- `90d7c80f` — S-PLUGIN-PREREQ-A, PR #142, 2026-05-11
- `ae7e26c8` — S-PLUGIN-PREREQ-B, PR #143, 2026-05-12
- `ea958a4d` — S-PLUGIN-PREREQ-C, PR #144, 2026-05-12

**Key implementation files:**
- `crates/prism-core/src/sensor_id.rs` (554 lines — SensorId newtype, PREREQ-A)
- `crates/prism-spec-engine/src/pipeline.rs` (PipelineExecutor, PREREQ-B)
- `crates/prism-spec-engine/src/auth_provider.rs` (AuthProvider trait + impls, PREREQ-B)
- `tests/external/perimeter-violation/src/main.rs` (compile-fail CI gate, PREREQ-A)
- `tests/external/non-exhaustive-violation-compile-fail/` (compile-fail CI gate, PREREQ-C)

**Adversarial review reports:**
- `.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-A-pass-*.md` (passes 1-12)
- `.factory/cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-B-pass-*.md` (passes 1-16)
- `.factory/code-delivery/S-PLUGIN-PREREQ-C/adversarial-review/` (passes 1-5; path-convention drift from A/B — cosmetic)

**STORY-INDEX:**
- `.factory/stories/STORY-INDEX.md` (v2.65 — 150 stories total; STORY-INDEX:394-401 for Wave 1 dependency graph)

**Forward task map:**
- `.factory/cycles/wave-4-operations/forward-task-map.md` (Tier 1-8 roadmap sealed D-444)
