---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T00:00:00Z
phase: 3
inputs: []
input-hash: "8ee0dee"
traces_to: ""
pass: 1
previous_review: null
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
pass_n: 1
target_sha: 4ab8d33c
base_sha: c6dd6602
red_gate_sha: 84f4d35d
verdict: BLOCKED-hard
streak: 0/3
finding_counts:
  CRITICAL: 2
  HIGH: 5
  MED: 4
  LOW: 3
  OBS: 2
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 1)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: `W4OPS` (wave-4-operations cycle)
- `<PASS>`: Two-digit pass number (`P01`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples for this pass: `ADV-W4OPS-P01-CRIT-001`, `ADV-W4OPS-P01-HIGH-002`

## ADV-S-PLUGIN-PREREQ-A-LP1 — Pass 1 Verdict

**Target:** feature/S-PLUGIN-PREREQ-A@4ab8d33c
**Diff base:** develop@c6dd6602
**Verdict:** BLOCKED-hard (CRITICAL + HIGH findings present)
**Streak:** 0/3 (RESET)

## Part A — Fix Verification (pass >= 2 only)

_Pass 1 — no prior pass to verify._

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

#### ADV-W4OPS-P01-CRIT-001: Silent acceptance of unknown source tables regresses ADV-W3MT-P58-LOW-002 fix
- **Severity:** CRITICAL
- **Category:** spec-fidelity / regression
- **Location:** crates/prism-query/src/materialization.rs:787-798 (sensor_type_from_table_name) and caller at materialization.rs:623-635
- **Description:** Pre-migration `sensor_type_from_table_name` returned `None` for unknown sensor prefixes; caller emitted E-QUERY-006. Post-migration any non-empty prefix becomes a `SensorId`, registry returns empty, query returns `Ok(vec![])`. Resolution of ADV-W3MT-P58-LOW-002 has been structurally undone.
- **Evidence:** The caller at materialization.rs:623-635 previously checked `is_none()` to gate E-QUERY-006 emission. After SensorType→SensorId migration the None path no longer exists; all prefixes succeed as SensorId construction.
- **Proposed Fix:** Either consult `AdapterRegistry` membership before constructing `SensorId`, OR after `get_all_for_sensor_type` returns empty, return E-QUERY-006. Add regression test.

#### ADV-W4OPS-P01-CRIT-002: AC-6 perimeter compile-fail test for SensorType absent
- **Severity:** CRITICAL
- **Category:** verification-gaps / spec-fidelity
- **Location:** tests/external/perimeter-violation/src/main.rs (zero SensorType refs); story AC-6 L234-239
- **Description:** AC-6 requires the compile-fail perimeter test to assert `SensorType` reintroduction fails. Implementer marked PARTIAL with TD-S-PLUGIN-PREREQ-A-001 citing structural enforcement. Adversary rejects: structural deletion is a one-time event, not a recurring CI guard. VP-PLUGIN-001 cannot be mechanically verified at CI time without the assertion.
- **Evidence:** `grep -r "SensorType" tests/external/perimeter-violation/src/main.rs` returns zero matches. AC-6 acceptance criterion is unmet.
- **Proposed Fix:** Add `SensorType` assertion to `perimeter-violation/src/main.rs`; verify CI regex catches E0432 error. Policy: POL-15 (runtime_wiring_required_for_accepted_adrs) + Standing Rule 3.

### HIGH

#### ADV-W4OPS-P01-HIGH-001: 9 stale SensorType doc-comment references in production source files (5 files)
- **Severity:** HIGH
- **Category:** spec-fidelity / partial-fix-residual
- **Location:** engine.rs:179, materialization.rs:14 (module doc), prism-sensors/src/lib.rs:10,95,96,97,98,153, registry.rs:121
- **Description:** AC-2 requires ZERO matches in non-test production code. Implementer report's "zero non-comment live code references" is too narrow; doc comments ARE production source code (ship in `cargo doc`).
- **Evidence:** `grep -rn "SensorType" crates/prism-query/src/ crates/prism-sensors/src/ crates/prism-core/src/ --include="*.rs"` returns 9 hits in doc-comment positions across 5 files.
- **Proposed Fix:** Search-and-replace `SensorType`→`SensorId` in doc-comments across 5 files. Re-run grep to confirm zero hits. Policy: S-7.01 (c) + AC-2 verbatim.

#### ADV-W4OPS-P01-HIGH-002: Stale Red Gate doc-comments claim "panic at todo!()" / "MUST FAIL"
- **Severity:** HIGH
- **Category:** spec-fidelity / partial-fix-residual
- **Location:** prism-core/src/sensor_id.rs:140,148,170,191; prism-sensors/src/tests/bc_2_01_013_sensorid.rs:7-9,27,32,74; prism-query/tests/sensorid_dispatch_redgate.rs:7-10,27,46,53
- **Description:** Three test files retain Red Gate framing in module doc-comments. Tests are GREEN but docs say they should be RED. Future reader confused.
- **Evidence:** Module doc-comments in the three test files contain phrases such as "MUST FAIL" and "panic at todo!()" — Red Gate framing that no longer matches the GREEN post-migration state.
- **Proposed Fix:** Update doc-comments to reflect GREEN state across all sites. Policy: S-7.01 (a) body-content propagation.

#### ADV-W4OPS-P01-HIGH-003: register() API contract drift vs AC-4 text
- **Severity:** HIGH
- **Category:** spec-fidelity / interface-gaps
- **Location:** prism-sensors/src/registry.rs:63-66
- **Description:** `register(org_id, adapter)` derives `SensorId` from `adapter.sensor_type()` internally; AC-4 says register accepts `SensorId` or `&str` explicitly. Implementation is arguably better (adapter owns identity) but deviates from spec text. Adversary marked "pending intent verification".
- **Evidence:** AC-4 acceptance criterion text specifies a `SensorId` or `&str` parameter; implementation signature takes only `org_id` + `adapter`.
- **Proposed Fix:** Orchestrator decision: ADOPT the implementation; update story AC-4 wording to specify "register(org_id, adapter) derives sensor_id from adapter.sensor_type() internally — adapter owns identity invariant". Policy: POL-4 (semantic_anchoring_integrity).

#### ADV-W4OPS-P01-HIGH-004: Duplicate registry methods: get_all_for_sensor_type/get_all_for_sensor + get/get_by_id
- **Severity:** HIGH
- **Category:** code-quality / partial-fix-residual
- **Location:** registry.rs:90-99 + 105-111 + 117-119
- **Description:** Three duplicate method pairs on 130-line file. `get_all_for_sensor` (new) has zero callers; `get_by_id` is identical to `get`. API surface bloat.
- **Evidence:** `get_all_for_sensor` introduced by migration; `grep -n "get_all_for_sensor\b" crates/prism-sensors/src/` returns zero callers outside registry.rs itself. `get_by_id` body is identical to `get`.
- **Proposed Fix:** Delete the variant with zero callers; delete `get_by_id` (use `get` everywhere). Policy: POL-4 + production-grade-closure.

#### ADV-W4OPS-P01-HIGH-005: SensorId::Deserialize accepts ANY string — injection surface
- **Severity:** HIGH
- **Category:** security-surface
- **Location:** prism-core/src/sensor_id.rs:131-136
- **Description:** No validation on `Deserialize` impl. `AnalystId`, `OrgSlug`, `CredentialName` all validate at newtype boundary (DI-014). `SensorId` silently breaks the pattern. Wave 1 plugins deserialize `SensorId` from external TOML — attacker-controlled `.prx` gets unvalidated `SensorId`.
- **Evidence:** `Deserialize` impl at L131-136 delegates directly to `Arc<str>` deserialization with no length or charset check.
- **Proposed Fix:** Add `validate_sensor_id_string()` (1-64 length, `[a-z0-9_-]` charset, no leading/trailing hyphen/underscore). Apply to `Deserialize` impl + `From<&str>`/`From<String>`. Policy: DI-014 sibling pattern (Credential Name Sanitization).

### MEDIUM

#### ADV-W4OPS-P01-MED-001: Dead PrismError::UnknownSensorId variant
- **Severity:** MEDIUM
- **Category:** code-quality
- **Location:** prism-core/src/error.rs:64-66
- **Description:** Variant declared but never constructed. E-SENSOR-010 emitted via `SensorError::AdapterNotFound` (cleaner path).
- **Evidence:** `grep -rn "UnknownSensorId" crates/` returns only the declaration; zero construction sites.
- **Proposed Fix:** Delete `UnknownSensorId` from error.rs (story Task 4 gave "remove if unused" option). Policy: POL-1 trivially satisfied; spec authorial intent gives delete option.

#### ADV-W4OPS-P01-MED-002: Case-sensitivity asymmetry between explain and materialization dispatch
- **Severity:** MEDIUM
- **Category:** interface-gaps
- **Location:** explain.rs:662 (lowercases) vs materialization.rs:790 (no lowercase)
- **Description:** `explain` `SensorId` is lowercase; materialization `SensorId` is case-preserving. Mixed-case query (`Crowdstrike_hosts`) makes explain say `crowdstrike`, fan-out lookup miss.
- **Evidence:** explain.rs:662 calls `.to_lowercase()` before constructing SensorId; materialization.rs:790 constructs directly from table prefix without case normalization.
- **Proposed Fix:** Lowercase in materialization.rs:790 (consistent with explain). Apply also in `validate_sensor_id_string` charset (lowercase-only). Policy: POL-4 semantic anchoring.

#### ADV-W4OPS-P01-MED-003: WriteToolInvalidationMap.sensor_name: &'static str — closed-set residue
- **Severity:** MEDIUM
- **Category:** purity-boundary-violations / partial-fix-residual
- **Location:** invalidation.rs:38-92
- **Description:** Static array with `&'static str` entries. Plugin sensors cannot register write tools. Closed-set masquerading as open dispatch.
- **Evidence:** `WriteToolInvalidationMap` is a const/static array with hardcoded sensor name strings; no runtime registration path exists.
- **Proposed Fix:** Convert to `LazyLock<Vec<WriteToolInvalidationMap>>` with `SensorId` field; OR keep static defaults + add runtime registry. Add plugin-write-tool registration test. Policy: POL-15 (runtime_wiring_required_for_accepted_adrs).

#### ADV-W4OPS-P01-MED-004: Sentinel-nil OrgId in write_dispatch.rs (pre-existing tech debt; story crystallizes)
- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** write_dispatch.rs:283-290
- **Description:** TODO W3-FIX-S307-002 reference exists. Write dispatch structurally broken in production (always emits E-SENSOR-010 due to sentinel-nil OrgId).
- **Evidence:** write_dispatch.rs:283-290 constructs a sentinel OrgId rather than resolving from request context; the TODO comment acknowledges this.
- **Proposed Fix:** Surface TD with explicit successor story ID (TD-S-PLUGIN-PREREQ-A-002 P1 — wire OrgRegistry into WriteDispatcher; depends on W3-FIX-S307-002 graduation). Keep TODO comment. Policy: POL-15.

### LOW

#### ADV-W4OPS-P01-LOW-001: AC-8 atomic-commit wording vs feature-branch reality
- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** story AC-8 wording vs branch history
- **Description:** AC-8 says "no intermediate broken state in commit history" but Red Gate commit alone doesn't build. Squash-merge IS atomic but branch isn't.
- **Evidence:** Branch history at 84f4d35d (Red Gate SHA) does not compile without the subsequent implementation commits.
- **Proposed Fix:** ADOPT current branch model; reword AC-8 to "the squash-merged commit is atomic and builds; intermediate Red Gate commit on the feature branch is permitted to reference yet-to-be-defined symbols". Policy: S-7.01 spec ⇆ body coherence.

#### ADV-W4OPS-P01-LOW-002: Duplicate # Construction doc-block in sensor_id.rs
- **Severity:** LOW
- **Category:** code-quality
- **Location:** sensor_id.rs:28-43
- **Description:** `# Construction` section appears twice in the module doc-block.
- **Evidence:** Lines 28-43 contain a duplicate heading block identical to an earlier occurrence in the same doc-comment.
- **Proposed Fix:** Delete the duplicate doc-block.

#### ADV-W4OPS-P01-LOW-003: Latency heuristic match in explain.rs:1049 — string-match O(arms) at runtime
- **Severity:** LOW
- **Category:** code-quality (deferrable)
- **Location:** explain.rs:1049
- **Description:** Match arms over sensor name strings scales linearly. Acceptable at N=4; becomes a smell at N=10+.
- **Evidence:** explain.rs:1049 uses a `match` over string literals for latency heuristics.
- **Proposed Fix:** Migrate to `HashMap<SensorId, u64>` when N grows. Deferred.

## Observations

### OBS-001 — STORY-INDEX status consistency
- **Severity:** OBS
- **Description:** story file (status: ready) consistent with STORY-INDEX L389 ([ready]). POL-13 satisfied.

### OBS-002 — VP-PLUGIN-007 enforcement gap (PREREQ-E scope)
- **Severity:** OBS
- **Description:** No automated CI check that `CustomAdapter` has zero production callers. Deferrable to PREREQ-E.

## Process-Gap Callouts (4)

1. [process-gap] Test-writer protocol lacks Red-Gate-prose → Green-Gate-prose transition step
2. [process-gap] Implementer protocol lacks comprehensive doc-comment SensorType sweep step (grep should be `'SensorType'` not `'SensorType::'`)
3. [process-gap] No perimeter compile-fail scaffold template for type-deletion ACs
4. [process-gap] Sibling-site review missed in fix-burst protocol (F-LP1-MED-003, HIGH-004, MED-002 all share pattern)

## KUDOs

1. `Arc<str>` payload choice (correct for cheap Clone across thread boundaries)
2. `Borrow<str>` + `AsRef<str>` without redundant getter
3. `PartialOrd` canonical-form (correct lexicographic ordering)
4. `Debug` impl preserves type identity (not transparent)
5. Red Gate test naming convention (clear intent labeling)
6. `SensorError::AdapterNotFound` typed field (not stringly-typed)
7. Four sensor adapter impls clean (no SensorType residue in implementation)

## Convergence Position

- Red Gate: 5/5 GREEN (verified by source reading)
- ACs: 6 PASS, 3 FAIL (AC-2, AC-6, AC-5 partial via invalidation.rs), 2 PARTIAL (AC-8, AC-4 drift)
- AC-6 partial-close: REJECTED by adversary — must graduate to PASS pre-merge

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 5 |
| MEDIUM | 4 |
| LOW | 3 |
| OBS | 2 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate
**Readiness:** requires revision (fix-burst-1 required before pass-2)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 14 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.00 (14 / 14) |
| **Median severity** | HIGH |
| **Trajectory** | 14 (pass 1 baseline) |
| **Verdict** | FINDINGS_REMAIN |
