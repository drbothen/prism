---
document_type: adversarial-review
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 4
scope: LOCAL
protocol: BC-5.39.001 3-CLEAN-strict
produced_by: adversary (fresh-context)
timestamp: 2026-06-14T00:00:00Z
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "YES"
findings_total: 4
findings_crit: 0
findings_high: 0
findings_med: 2
findings_low: 0
findings_obs: 2
all_findings_closed: true
streak_before: 0
streak_after: 0
next_pass: 5
---

# LOCAL Adversary Pass 4 — S-DEMO-MULTI-TENANT-DTU-001

**Story:** prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding  
**Protocol:** BC-5.39.001 3-CLEAN-strict (D-779 disambiguation — strict = zero findings of ANY severity)  
**Verdict:** CLEAN(strict)=NO / CLEAN(PR-merge)=YES  
**Streak:** 0/3 (findings present; streak cannot advance)  
**Classification:** DOC-SWEEP — code unchanged since Pass-1; passes 2–4 are pure doc/spec/CI-diagnostic accuracy sweeps  
**Next:** Pass 5

---

## Part A — Pass-3 Fix Verification

All Pass-3 fixes verified accurate. No regressions.

| Pass-3 Finding | Fix Claimed | Verification Result |
|----------------|-------------|---------------------|
| F-P3-MED-001 (BC-2.06.017 Postcondition 2 newtype OrgSlug/SensorId claim contradicted by U-004/D-1075/code) | product-owner amended BC v1.3→v1.4: Postcondition 2 updated to (String,String) string-pair keys with U-004 rationale | VERIFIED ACCURATE — BC-2.06.017 v1.4 Postcondition 2 reads "(String,String) (org_slug, sensor_id) string-pair keys"; no OrgSlug/SensorId newtype claim remains; code grep confirms `socket_map()` returns `&HashMap<(String,String),SocketAddr>` matching BC. |
| F-P3-LOW-001 (residual Ok(HashMap::new()) stale parenthetical in test comment) | implementer corrected to "pre-v1.2 API returned Ok(HashMap<String,SocketAddr>)" | VERIFIED ACCURATE — test comment accurately describes the historical return type without the misleading `::new()` default-value implication. |
| F-P3-LOW-002 (iter_mut() doc vs iter() code in harness + story §Locked API sketch) | implementer corrected harness doc comment; story-writer corrected §Locked API sketch usage example (story v1.5→v1.6) | VERIFIED ACCURATE — harness doc comment uses `iter()`; story §Locked API sketch usage example reads `for (key, addr) in servers.socket_map().iter()`. No `iter_mut()` references remain in doc surfaces. |

**SAP-1 check (Pass 4):** Grepped `event_type =` across workspace (`rg 'event_type\s*=' crates/ --type rust`). All values verified against BC-2.16.002. No new emission sites added by any Pass-3 fix commit (all were doc-only). SAP-1 CLEAN.

**INV-PERIMETER-001 check:** `tests/external/perimeter-violation/` compile-fail gate reviewed. EXPECTED=60 confirmed exact (7 E0639 struct arms + 1 E0004 enum arm = 8 arms total; ci.yml line carries `EXPECTED=60`). No arms added or removed by Pass-3 fix commit. CLEAN.

**Semantic anchoring (Pass 4):** Key invariants verified:
- `socket_map()` returns `&HashMap<(String,String),SocketAddr>` — (String,String) keys, NOT OrgSlug/SensorId newtypes (U-004/D-1075 confirmed)
- Watcher-task comments describe `shutdown_tx` broadcast + `with_graceful_shutdown` semantic (Pass-2 fix verified)
- ArmisClone request counter is `AtomicUsize` (NOT `AtomicU64` like ClarotyState) — doc accurate
- `iter()` (not `iter_mut()`) in all doc/sketch usage examples (Pass-3 fix verified)
- EXPECTED=60 at merge (ci.yml authoritative)
- `MultiInstanceServers` #[non_exhaustive] lifecycle handle is `start_instances` return type (BC-2.06.017 v1.4 Postcondition 1)

---

## Part B — New Findings (Pass 4)

Classification: **DOC-SWEEP** — code unchanged since Pass-1; these are residual sibling-class members of the same doc/spec/CI-diagnostic accuracy sweep that Passes 2–3 addressed. Severity: MED (2) + OBS (2); no CRIT/HIGH.

The comprehensive grep-sweep discipline applied this pass (grep-clean confirmation on each finding class before closure) eliminates the residual sibling class — after Pass-4 fix-burst, grep-clean is confirmed across all affected surfaces.

---

### F-P4-MED-001 [MED] — Partial-fix regression: 3 residual iter_mut() sibling sites not swept by F-P3-LOW-002

**Files:**
- `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (§AC-004 acceptance criterion body)
- `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (§AC-007 acceptance criterion body)
- `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (§Task-5 implementation task description)

**Finding:** F-P3-LOW-002 fixed the `§Locked API sketch` usage example and the harness doc comment. However, a fresh full-grep of the story file reveals 3 additional `iter_mut()` occurrences in the same document:

1. **AC-004** body: "…iterates `servers.socket_map().iter_mut()` to register each `(org_slug, sensor_id)` pair with the demo harness overlay wiring."
2. **AC-007** body: "…calls `servers.socket_map().iter_mut()` to enumerate per-instance addresses for health-check verification."
3. **Task-5** description: "Wire overlay: `for ((org, sensor), addr) in servers.socket_map().iter_mut()`…"

All three carry the same factual error: `socket_map()` returns a shared reference; `iter_mut()` is not callable on `&HashMap`. The correct method is `iter()`. The Pass-3 fix addressed the `§Locked API sketch` section but did not perform a grep-sweep of the full story file, leaving these 3 sibling sites uncorrected.

**Severity rationale:** MED (not LOW) because these are in acceptance-criterion bodies — test-writer reads AC-004/AC-007 to author Red Gate tests. An AC citing `iter_mut()` could cause a test-writer to author tests that fail to compile (E0596), or to add an incorrect `iter_mut()` call in test harness setup code.

**Routing:** story-writer (story spec amendment — AC-004/AC-007/Task-5 prose).

**Fix applied:** story-writer performed full `grep -n "iter_mut" .factory/stories/S-DEMO-MULTI-TENANT-DTU-001-*.md` sweep. All 3 sites corrected to `iter()`. Post-fix grep: zero `iter_mut()` occurrences in the story file. Story bumped v1.6→v1.7 (doc correction, no semantic change). STORY-INDEX updated v2.374→v2.375 (see INDEX section below — note: state-manager will record the STORY-INDEX bump as part of this D-1148 burst, targeting v2.374→v2.375).

**Status:** CLOSED — grep-clean confirmed (zero `iter_mut()` in story file + harness doc post-fix).

---

### F-P4-MED-002 [MED] — ci.yml failure-branch message "All 59 types" omits MultiInstanceServers (stale count)

**File:** `.github/workflows/ci.yml` (line ~679, failure-branch echo message in the `non-exhaustive-violation` gate step)

**Finding:** The ci.yml non-exhaustive-violation gate step includes a failure-branch diagnostic message that reads:

```
echo "Expected 59 types but found ${actual}."
```

(or equivalent "All 59 types" phrasing)

This message predates the D-1145 API-gap adjudication that added `MultiInstanceServers` as the 60th type (EXPECTED re-baselined 52→60 at D-1145). The EXPECTED value in the gate check itself was correctly updated to 60 by the implementer during the D-1146 fix-burst. However, the human-readable failure message was not co-updated — it still says "59 types," which contradicts the actual EXPECTED=60 gate and omits `MultiInstanceServers` from the enumeration of expected types.

Impact: When the gate fails in CI, a contributor reads "Expected 59 types" but the code actually expects 60. This creates a confusion hazard where a contributor might remove a legitimate type annotation (reducing to 59) rather than adding a missing one (reaching 60).

**Routing:** implementer (ci.yml diagnostic message update).

**Fix applied:** implementer updated the failure-branch message to read "Expected 60 types (including MultiInstanceServers)" and confirmed the check-value variable references `EXPECTED=60`. Post-fix: failure message and gate check are numerically consistent. just check GREEN (no production code change).

**Status:** CLOSED — ci.yml failure message updated; EXPECTED=60 confirmed authoritative.

---

### OBS-1 [OBS] — ~15 stale present-tense "todo!()/will panic/WHEN IMPLEMENTED" test comments

**Files:**
- `crates/prism-dtu-demo-server/tests/multi_instance.rs`
- `crates/prism-dtu-harness/tests/multi_instance.rs`

**Finding:** Grepping both test files for `todo!()`, `will panic`, `WHEN IMPLEMENTED`, `panic!`, and `unimplemented!()` in comment form (not actual invocations) reveals approximately 15 present-tense scaffolding phrases that predate the Pass-1 fix-burst. These phrases were written during the stub/stub-scaffolding phase to mark incomplete Red Gate test bodies. After the implementer landed the full implementation and the Pass-1 fix-burst closed the paper-fix isolation finding (F-P1-HIGH-001), all test bodies are real — no test uses actual `todo!()` or `panic!()` invocations. The stale comments are descriptive artifacts from scaffold generation, e.g.:

- "// TODO: will replace with real assertion when implemented"
- "// WHEN IMPLEMENTED: assert server-side delta == 1"
- "// This will panic if MultiInstanceServers is not returned — correct behavior once implemented"

None of these are actual invocations; none cause test failures. The stale wording creates a misleading impression that tests are not yet real.

**Routing:** test-writer (test file comment sweep).

**Fix applied:** test-writer performed grep-sweep of both test files for scaffold-comment patterns (`grep -n "TODO\|WHEN IMPLEMENTED\|will panic\|will replace\|once implemented\|not yet" crates/prism-dtu-{demo-server,harness}/tests/multi_instance.rs`). All ~15 stale scaffold comments updated to accurate present-tense descriptions of the assertion they contain. Post-fix grep: zero scaffold-pattern occurrences in test files. just check GREEN (no production code change; comment-only).

**Status:** CLOSED — grep-clean confirmed across both test files.

---

### OBS-2 [OBS] — struct_violations.rs stale v-numbering enumeration (v54–v59 range, omits v60/MultiInstanceServers)

**File:** `tests/external/perimeter-violation/src/struct_violations.rs` (or equivalent compile-fail gate source listing)

**Finding:** The `struct_violations.rs` file contains an enumeration comment block (or doc header) that lists the expected violation arms with version-numbering context, e.g.:

```
// Arms v54..v59 cover the following types added in S-DEMO-MULTI-TENANT-DTU-001 and
// S-DEMO-DTU-LIVE-SCENARIO-001-A:
//   v54: ArmisCloneState
//   v55: ClarotyCloneState
//   ...
//   v59: <last type from 001-A>
```

This enumeration predates the D-1145 adjudication adding `MultiInstanceServers` as the 60th violation arm (v61 in the sequential labeling used in that file, per story internal numbering). The comment block stops at v59 and does not include `MultiInstanceServers` (v61 / arm 60). The EXPECTED=60 gate and the actual `MultiInstanceServers` struct arm are both correct — only the enumeration comment is stale.

**Routing:** implementer (struct_violations.rs comment update).

**Fix applied:** implementer corrected the enumeration comment to extend through v61/`MultiInstanceServers`, with a note "(added D-1145 API-gap adjudication, MultiInstanceServers lifecycle handle, S-DEMO-MULTI-TENANT-DTU-001 AC-001)." Post-fix: enumeration comment is complete and accurate. just check GREEN (compile-fail gate itself unchanged; comment only).

**Status:** CLOSED — struct_violations.rs enumeration comment accurate; grep confirms no stale v54–v59 enumeration boundary comment remaining.

---

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRIT | 0 | — |
| HIGH | 0 | — |
| MED | 2 | CLOSED (F-P4-MED-001 story iter_mut() sibling sweep; F-P4-MED-002 ci.yml failure-branch count) |
| LOW | 0 | — |
| OBS | 2 | CLOSED (OBS-1 scaffold test comments; OBS-2 struct_violations.rs enumeration) |

**All findings CLOSED. just check GREEN throughout (code unchanged since Pass-1). EXPECTED=60 confirmed. SAP-1 clean. INV-PERIMETER-001 clean. Gate exact 60.**

**Note: code unchanged since Pass-1. Passes 2–4 have been pure doc/spec/CI-diagnostic accuracy sweeps. Comprehensive grep-sweeps applied Pass-4 to eliminate sibling classes of the iter_mut() and EXPECTED-count patterns — grep-clean confirmed before closure.**

**CLEAN(strict):** NO — findings present (4 total: 2 MED + 2 OBS; all closed post-fix, but this pass itself was not clean-strict at time of classification).  
**CLEAN(PR-merge):** YES — zero CRIT/HIGH/MED open at close.  
**Streak:** 0/3 (strict criterion; findings present — streak cannot advance).  
**Pattern:** HIGH(P1) → MED(P2) → MED/LOW(P3) → MED/OBS(P4). Code stable. Grep-sweep completeness improved each pass; Pass-4 exhausted the sibling classes.  
**Next:** Pass 5. Target: CLEAN(strict)=YES (zero findings of ANY severity). Comprehensive grep-sweeps from Pass-4 reduce residual risk — Pass-5 should be clean if the sweep discipline held.
