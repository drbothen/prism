---
document_type: adversarial-review
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 6
scope: LOCAL
protocol: BC-5.39.001 3-CLEAN-strict
produced_by: adversary (fresh-context)
timestamp: 2026-06-14T00:30:00Z
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "YES"
findings_total: 1
findings_crit: 0
findings_high: 0
findings_med: 1
findings_low: 0
findings_obs: 0
all_findings_closed: true
streak_before: 0
streak_after: 0
next_pass: 7
---

# LOCAL Adversary Pass 6 — S-DEMO-MULTI-TENANT-DTU-001

**Story:** prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding
**Protocol:** BC-5.39.001 3-CLEAN-strict (D-779 disambiguation — strict = zero findings of ANY severity)
**Verdict:** CLEAN(strict)=NO / CLEAN(PR-merge)=YES
**Streak:** 0/3 (1 MED finding present; streak cannot advance)
**Classification:** DOC-ONLY — one doc-accuracy gap found and closed; code unchanged since Pass-1
**Next:** Pass 7

---

## Part A — Consistency-Audit Verification

All 10 pre-Pass-6 consistency-audit findings (D-1150) were verified CLOSED.

| Finding | Summary | Verification |
|---------|---------|--------------|
| B-001 | overlay-wiring docs — BC Postcondition 2 underspecified (base_url-only vs 3 REQUIRED fields) | VERIFIED CLOSED — BC v1.6 Postcondition 2 enumerates all 3 fields (extends, instance_id, base_url) + INV-SCALAR-003 xref; closed D-1149 F-P5-MED-003 |
| B-002 | story AC-005/§File-Structure/Task-6 overlay YAML snippet underspecified | VERIFIED CLOSED — story v1.9 AC-005/§File-Structure/Task-6 corrected to 3-field format; closed D-1149 F-P5-MED-003 |
| M-001 | BC-version-citation drift at 8 story sites | VERIFIED CLOSED — citations now version-agnostic (frontmatter `version:` is sole authority); closed D-1149 F-P5-MED-001 |
| M-002 | H1 heading inline version stamp desync | VERIFIED CLOSED — H1 version-agnostic; closed D-1149 F-P5-MED-002 |
| M-003 | BC-INDEX inline note draft_contracts: 2 stale (BC-2.06.017 missing) | VERIFIED CLOSED — corrected to "draft_contracts: 3 covers BC-2.06.011 + BC-2.06.017 + BC-2.21.001"; closed D-1150 burst |
| M-004 | CLAUDE.md EXPECTED count 52 vs worktree 60 | VERIFIED AUTO-RESOLVES AT MERGE — squash-merge will propagate worktree ci.yml EXPECTED=60 to develop; no action required pre-merge |
| M-005 | 3-field overlay code path correctness | VERIFIED CLOSED — code correct since Pass-1 commit 9b4f4154; product-owner spec now matches |
| M-006 | ArmisClone AtomicUsize doc false-claim | VERIFIED CLOSED — corrected D-1147 F-P2-MED-002 |
| M-007 | socket_map() (String,String) keys doc | VERIFIED CLOSED — corrected D-1147 F-P3-MED-001 |
| N-001 | BC version citation log coverage | VERIFIED CLOSED — amendment-log preserved; version-agnostic citations in place |

**Part A verdict: ALL 10 CONSISTENCY-AUDIT FINDINGS VERIFIED CLOSED.**

---

## Part B — Pass 6 Story-Scoped Sweep

**Scope:** BC-2.06.017 v1.6 · S-DEMO-MULTI-TENANT-DTU-001 v1.9 · implementation code in worktree · test files · ci.yml gate · struct_violations.rs enumeration

### Axes checked

| Axis | Result |
|------|--------|
| SAP-1 (tracing emission catalog) | CLEAN — no `event_type =` emissions in story src |
| INV-PERIMETER-001 (EXPECTED=60 gate) | CLEAN — EXPECTED=60 confirmed in ci.yml; struct_violations.rs enumeration extends through v61/MultiInstanceServers |
| unwrap/expect/println in story src | CLEAN — none in production code paths |
| BC/INDEX title + subsystem + version sync | CLEAN — BC-2.06.017 v1.6 matches BC-INDEX row |
| Semantic anchoring (CAP-036) | CLEAN — MultiInstanceHarness + MultiInstanceServers anchor to CAP-036 correctly |
| AC ↔ test coverage | CLEAN — 20 tests cover all ACs (18 multi_instance/harness + 2 bind-failure) |
| iter() discipline (not iter_mut()) | CLEAN — all usage examples, AC-004/AC-007/Task-5/§Locked API sketch use iter() |
| socket_map() return type | CLEAN — HashMap<(String,String),SocketAddr> per U-004/D-1075 throughout |
| Overlay 3-field format | CLEAN — AC-005/§File-Structure/Task-6 and BC Postcondition 2 both enumerate extends/instance_id/base_url |
| Watcher-task comment accuracy | CLEAN — describes shutdown_tx broadcast + with_graceful_shutdown |
| ArmisClone counter type | CLEAN — AtomicUsize (not AtomicU64; not mirroring ClarotyState) |
| Scaffold comment residue | CLEAN — Pass-4 OBS-1 grep-swept both test files; clean confirmed |
| ci.yml failure-branch message | CLEAN — reads "60 types (including MultiInstanceServers)" |

### Findings

---

#### F-P6-MED-001 [MEDIUM — CLOSED] MultiInstanceHarness doc referenced non-existent `shutdown()` method

**Severity:** MEDIUM
**Classification:** DOC-ONLY (sibling copy-over from demo-server MultiInstanceServers)
**Root cause:** MultiInstanceHarness documentation referenced a `shutdown()` method that does not exist on `MultiInstanceHarness`. The harness lifecycle is Drop-only per BC-2.06.017 Postcondition 2 (MultiInstanceServers owns `shutdown()` and Drop; harness wraps via `MultiInstanceServers` and inherits Drop-based cleanup but does NOT expose its own `shutdown()` method). The stale reference was a copy-paste artefact from the demo-server `MultiInstanceServers` block into the harness documentation.

**Contract anchor:** BC-2.06.017 Postcondition 2 + AC-004 (harness lifecycle is Drop-mediated; no harness-level `shutdown()` is specified or required)

**Fix:** Implementer doc-only commit `a27b0f54`. Replaced all `harness.shutdown()` references in `MultiInstanceHarness` documentation with Drop-only phrasing consistent with BC Postcondition 2 / AC-004. Full grep sweep confirmed no residual `shutdown()` call on `MultiInstanceHarness` type after fix. `just check` GREEN (code unchanged; doc-only).

**Verification path:** `grep -n "shutdown" crates/prism-dtu-harness/src/multi_instance.rs` — zero hits on harness struct methods; `just check` exit 0.

**Status:** CLOSED — doc-only fix commit a27b0f54; grep-clean; `just check` GREEN.

---

### Pass 6 Summary

| Category | Count |
|----------|-------|
| CRIT | 0 |
| HIGH | 0 |
| MED | 1 (F-P6-MED-001 — CLOSED) |
| LOW | 0 |
| OBS | 0 |
| **Total** | **1** |

**CLEAN(strict):** NO (1 MED finding)
**CLEAN(PR-merge):** YES (no CRIT/HIGH/MED open findings after fix)
**Streak:** 0/3 (finding present; streak does not advance)
**All findings closed:** YES
**Next:** Pass 7 (verify F-P6-MED-001 fix; full sweep for convergence)
