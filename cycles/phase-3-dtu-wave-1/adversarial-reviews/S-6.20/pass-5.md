---
document_type: adversarial-review
level: ops
version: "1.4"
status: complete
producer: adversary
timestamp: 2026-04-23T00:00:00Z
phase: 3
inputs:
  - .factory/stories/S-6.20-dtu-demo-server.md
  - .factory/specs/architecture/decisions/ADR-002-l2-dtu-clone-template.md
  - .factory/policies.yaml
  - Cargo.toml
  - crates/prism-dtu-crowdstrike/src/lib.rs
  - crates/prism-dtu-claroty/src/lib.rs
  - crates/prism-dtu-cyberint/src/lib.rs
  - crates/prism-dtu-armis/src/lib.rs
  - crates/prism-dtu-threatintel/src/lib.rs
  - crates/prism-dtu-nvd/src/lib.rs
  - crates/prism-dtu-common/src/lib.rs
  - crates/prism-dtu-crowdstrike/src/clone.rs
  - crates/prism-dtu-claroty/src/clone.rs
  - crates/prism-dtu-cyberint/src/clone.rs
  - crates/prism-dtu-armis/src/clone.rs
  - crates/prism-dtu-threatintel/src/clone.rs
  - crates/prism-dtu-nvd/src/clone.rs
  - crates/prism-dtu-crowdstrike/Cargo.toml
  - crates/prism-dtu-nvd/Cargo.toml
  - crates/prism-dtu-threatintel/Cargo.toml
  - crates/prism-dtu-crowdstrike/tests/ac_4_rate_limit.rs
  - crates/prism-dtu-crowdstrike/tests/integration_vp033.rs
  - crates/prism-dtu-crowdstrike/tests/integration_vp036.rs
  - crates/prism-dtu-crowdstrike/tests/edge_cases.rs
  - crates/prism-dtu-crowdstrike/tests/ac_6_determinism.rs
  - .factory/cycles/phase-3-dtu-wave-1/adversarial-reviews/S-6.20/pass-4.md
input-hash: "5778cd9"
traces_to: S-6.20-dtu-demo-server.md
pass: 5
previous_review: .factory/cycles/phase-3-dtu-wave-1/adversarial-reviews/S-6.20/pass-4.md
cycle: phase-3-dtu-wave-1
target: "S-6.20 v1.4 @ 11e6fed7"
verdict: BLOCKED
findings_total: 7
counts:
  critical: 0
  high: 2
  medium: 3
  low: 2
  observation: 0
regressions_from_prior_passes: 0
novel_findings: 7
next_action: "story-writer v1.5 remediation; then adversary Pass 6"
---

# Adversarial Review: S-6.20 DTU Demo Server (Pass 5)

**Verdict: BLOCKED** — v1.4 resolved Pass 4 CRITICALs but 6 genuine novel findings remain (2H + 3M + 1L).

## Finding ID Convention

Finding IDs use the format: `ADV-WV1-P05-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `WV1`: Cycle prefix for phase-3-dtu-wave-1
- `P05`: Pass 5
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

## Part A — Fix Verification (pass >= 2 only)

v1.4 remediation landed at 11e6fed7. Pass 4 had 2 CRITICAL and 12 HIGH/MEDIUM/LOW findings.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-WV1-P04-CRIT-001 (Task 14 "one-line update" impossible — 4 clones lack server_handle) | CRITICAL | RESOLVED | v1.4 introduced per-crate delta table with explicit struct-field additions for cyberint, armis, threatintel, nvd. Scope correctly expanded. |
| ADV-WV1-P04-CRIT-002 (references absent crates prism-dtu-ocsf / prism-dtu-osquery) | CRITICAL | RESOLVED | v1.4 scopes Task 14 to the 6 present workspace crates (crowdstrike, claroty, cyberint, armis, threatintel, nvd). |
| ADV-WV1-P04-HIGH-001 (stop_all mechanism — no JoinHandle in 4 clones) | HIGH | RESOLVED | Subsumed by CRIT-001 remediation. Per-crate delta now includes server_handle field additions. |
| ADV-WV1-P04-HIGH-002 (bind hardcode) | HIGH | RESOLVED | StubConfig bind/port fields added per Task 2 migration table in v1.4. |
| ADV-WV1-P04-MED-001 through HIGH/MED remaining | MEDIUM | RESOLVED | v1.4 addressed apply_config loopback policy and EC-008/AC-5 contradiction per story changelog. |

## Part B — New Findings (or all findings for pass 1)

### HIGH

#### ADV-WV1-P05-HIGH-001: File Structure Requirements row undercounts Task 14 test-file delta (internal contradiction)

- **Severity:** HIGH
- **Category:** contradictions
- **Location:** Task 2 migration table (lines 169-173) vs File Structure Requirements row (line 907)
- **Description:** Task 2 migration table enumerates 5 crowdstrike test files needing `bind: None` updates: `ac_4_rate_limit.rs`, `integration_vp033.rs`, `integration_vp036.rs`, `edge_cases.rs`, `ac_6_determinism.rs`. The File Structure Requirements row at line 907 states "Add `bind: None` to StubConfig struct-literal sites (**8 files**, ~12 LOC)." The numbers are contradictory.
- **Evidence:** 5 rows enumerated in migration table (lines 169-173); literal "8 files" at line 907; all 5 named files contain literal `StubConfig { ... }` without spread at cited lines (confirmed via direct file read). Three additional files are neither named nor located.
- **Proposed Fix:** Reconcile to one number — either enumerate the 3 additional sites with file/line in the migration table, or correct line 907 to "5 files". A compilation failure on struct-literal breaking change makes this high-risk if sites are missed.

#### ADV-WV1-P05-HIGH-002: AC-10 asserts `/dtu/health` on 6 ports but 2 current clone routers do not expose it (silent AC failure)

- **Severity:** HIGH
- **Category:** verification-gaps
- **Location:** AC-10 (lines 692-694); `crates/prism-dtu-nvd/src/clone.rs:63-70`; `crates/prism-dtu-threatintel/src/clone.rs:46-53`
- **Description:** AC-10 states "when `curl http://127.0.0.1:<port>/dtu/health` is issued for each of the 6 clone ports (17080-17085), then all 6 return HTTP 200 with `{"status":"ok"}`." The spec delegates `/dtu/health` deployment to TD-WV0-05 (Task 3 pre-check). However, the pre-check validates file existence and symbol presence via grep, but does NOT assert that the route is mounted in `build_router`.
- **Evidence:**
  - `nvd/src/clone.rs:63-70` — `build_router()` mounts `/dtu/configure`, `/dtu/reset`, `/dtu/request-count/:cve_id` — no `/dtu/health`.
  - `threatintel/src/clone.rs:46-53` — `build_router()` mounts `/dtu/configure` only — no `/dtu/reset`, no `/dtu/health`.
  - Task 3 pre-check validates `test -f crates/prism-dtu-threatintel/src/routes/dtu.rs` and `grep -l 'get_health' crates/prism-dtu-nvd/src/` — not that routes are mounted in `build_router`.
- **Proposed Fix:** Strengthen Task 3 pre-check to assert the route string appears in a `.route()` call within `clone.rs` (e.g., `grep '\.route.*dtu/health' crates/prism-dtu-nvd/src/clone.rs`). OR cross-reference TD-WV0-05 as a hard blocker for AC-10 with explicit mount requirement.

### MEDIUM

#### ADV-WV1-P05-MED-001: R-DEMO-002 references `POST /dtu/reset` on all clone ports but threatintel clone router does not expose it (AC-5 cleanup path broken)

- **Severity:** MEDIUM
- **Category:** interface-gaps
- **Location:** R-DEMO-002 (line 963); EC-004 (line 945); AC-5 (line 659)
- **Description:** R-DEMO-002 states "provide `POST /dtu/reset` on each clone port to restore fixture state." AC-5 calls `clone.reset().await.ok()` via the trait (not HTTP). The threatintel clone does not mount `/dtu/reset` over HTTP, so a demo script posting to `http://127.0.0.1:17084/dtu/reset` receives HTTP 404.
- **Evidence:** `threatintel/src/clone.rs:46-53` — only `/dtu/configure` is mounted. Trait-level `reset()` works; HTTP endpoint does not exist for threatintel.
- **Proposed Fix:** Add `/dtu/reset` to Task 3 pre-check, OR state explicitly that R-DEMO-002 requires TD-WV0-05 to complete `/dtu/reset` and `/dtu/health` mounts on threatintel before the demo script is valid.

#### ADV-WV1-P05-MED-002: Trait extension contradicts Task 14's ~+37 LOC delta for crowdstrike/claroty (existing `start()` bodies must change)

- **Severity:** MEDIUM
- **Category:** ambiguous-language
- **Location:** Task 14 Crate 1 delta (lines 467-475); `crowdstrike/src/clone.rs:62-97`; ADR-002 Amendment (line 759)
- **Description:** Task 14 Crate 1 (crowdstrike) says "Replace `start()` body: use `with_graceful_shutdown` wired to `shutdown_rx` param | ~+8" and "Modify `start()` default: delegate to `start_on(...)` | ~+3". The ADR-002 Amendment (line 759-761) provides a default `start()` body IN THE TRAIT. If clones keep a custom `start()`, they shadow the trait default. The existing crowdstrike `start()` is ~35 LOC — converting to a 1-line shim is approximately -34 LOC delta, not +3.
- **Evidence:** Line 759-761 shows trait-level `start()` default. Task 14 says "+3" for modifying the custom impl. These are contradictory descriptions of the same operation.
- **Proposed Fix:** Clarify: either (a) crowdstrike/claroty DELETE their `start()` impls and rely on trait default (net negative LOC), OR (b) keep custom `start()` as a one-liner `self.start_on(...).await.map(|_| ())`. Remove the "+3" delta entry if option (a) is chosen.

#### ADV-WV1-P05-MED-003: `DemoConfig.clones.<name>.continue_on_error` field specified in M5 but absent from TOML schema and struct definition

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** Task 5 M5 note (lines 344-348); Task 2 `DemoConfig` schema (lines 187-206)
- **Description:** M5 states `continue_on_error: bool` (default `false`) is a field on each ClonePair. The TOML schema enumerates `enabled`, `bind`, `port`, `fixture_set`, `initial_failure_mode`, `seed`, `tls` — `continue_on_error` is absent. No acceptance criterion covers `continue_on_error` behavior.
- **Evidence:** Line 345 specifies the field; lines 192-200 TOML schema does not list it. An implementer copying the schema verbatim produces a struct with no such field and M5 behavior is silently dropped.
- **Proposed Fix:** Either add `continue_on_error = false` to the TOML schema, struct definition, and an AC asserting the behavior, or remove M5's `continue_on_error` feature entirely.

### LOW

#### ADV-WV1-P05-LOW-001: AC-11 describes outcome but not observable mechanism (listener-leak-free is not directly testable as written)

- **Severity:** LOW
- **Category:** verification-gaps
- **Location:** AC-11 (lines 696-699)
- **Description:** AC-11 states "the 3 already-started clones have had `stop()` called on them (no listener leak)." No observation mechanism is provided. No helper exists in `prism-dtu-common::test_utils` for this. TCP port release is race-prone.
- **Evidence:** AC-11 text provides no assertion call, no helper reference, no polling strategy. A test author must invent non-deterministic logic (e.g., polling `TcpStream::connect` with backoff).
- **Proposed Fix:** Specify the observation: e.g., harness exposes `StartReport { already_started_and_stopped: Vec<String> }` asserted against `["crowdstrike", "claroty", "cyberint"]`. Add helper `assert_port_released(addr)` as a secondary signal with bounded retry.

#### ADV-WV1-P05-LOW-002: `lib.rs` doc-fix string "currently 6; target: 13" — math verified correct, finding RESOLVED on re-check

- **Severity:** LOW
- **Category:** contradictions
- **Location:** Line 810-814 fix description; `prism-dtu-common/src/lib.rs:6`
- **Description:** Adversary initially flagged "6 + 7 = 13" as potentially miscounted. Re-verification: S-6.11-S-6.13 = 3 stories; S-6.16-S-6.19 = 4 stories; 3+4 = 7 pending; 6 merged + 7 pending = 13. Math is correct.
- **Evidence:** Arithmetic reconciles. No defect.
- **Status:** FALSE POSITIVE — downgraded to RESOLVED on re-verification. Doc-fix string is arithmetically correct. Retained for audit trail.
- **Proposed Fix:** None required.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 3 |
| LOW | 1 (genuine) + 1 (resolved false positive) |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision (v1.5 remediation needed)

Three targeted edits would likely converge Pass 6:
1. Reconcile HIGH-1 file count (Task 2 table vs line 907)
2. Strengthen Task 3 pre-check (HIGH-2, MED-001)
3. Clarify Task 14 LOC delta semantics (MED-002)

Plus add `continue_on_error` to TOML schema (MED-003) and specify AC-11 observation mechanism (LOW-001).

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 5 |
| **New findings** | 6 (HIGH-1, HIGH-2, MED-1, MED-2, MED-3, LOW-1) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 6 / (6 + 0) = 1.00 |
| **Median severity** | MEDIUM |
| **Trajectory** | 14→7 (pass 4→5; pass 4 had 14 findings pre-CRIT resolution) |
| **Verdict** | FINDINGS_REMAIN |
