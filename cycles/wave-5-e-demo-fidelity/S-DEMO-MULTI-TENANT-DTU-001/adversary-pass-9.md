---
document_type: adversary-pass-report
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 9
protocol: BC-5.39.001 3-CLEAN-strict
verdict_clean_strict: "YES"
verdict_clean_pr_merge: "YES"
streak_before: 0
streak_after: 1
findings_total: 0
findings_by_severity:
  CRIT: 0
  HIGH: 0
  MED: 0
  LOW: 0
  OBS: 0
all_findings_status: CLOSED
classification: CLEAN
date: 2026-06-13
d_anchor: D-1153
spec_versions_at_pass:
  story: v1.10
  bc: v1.7
  bc_index: "6.50"
  story_index: "v2.378"
---

# Adversary Pass 9 — S-DEMO-MULTI-TENANT-DTU-001

## Verdict

- **CLEAN (strict):** YES — zero findings of any severity. Streak ADVANCES 0/3 → 1/3.
- **CLEAN (PR-merge):** YES — zero CRIT/HIGH/MED findings.

## Part A — F-P8-HIGH-001 Fix Verification (Bidirectional)

The Pass-8 HIGH finding was: F-P8-HIGH-001 — prism-dtu-armis/src/ code scope added by the F-P1-HIGH-001 fix-burst was never recorded in story sibling surfaces or BC-2.06.017 crates array. Fix applied at D-1152 (story v1.9→v1.10; BC v1.6→v1.7).

**Verification of each fix surface:**

| Surface | Expected After Fix | Verified |
|---------|-------------------|----------|
| story `crates_touched` frontmatter | includes `prism-dtu-armis` | CONFIRMED |
| story Architecture Mapping table | row for prism-dtu-armis (state.rs + clone.rs + routes/dtu.rs) | CONFIRMED |
| story File Structure Reference | rows for prism-dtu-armis/src/state.rs, clone.rs, routes/dtu.rs | CONFIRMED |
| story AC-006 note | clarifies "demo-server/harness src/ never names clone types"; prism-dtu-armis/src/ explicitly in-scope | CONFIRMED |
| story perimeter assertion | disambiguates: ArmisClone crate instruments own control-plane (/dtu/*); not INV-PERIMETER-001 breach | CONFIRMED |
| BC-2.06.017 `crates:` frontmatter | includes `prism-dtu-armis` | CONFIRMED |
| BC-2.06.017 Postcondition 4 note | server-side counter in prism-dtu-armis/src/; AC-006 isolation proof mechanism; /dtu/request-count is control-plane route | CONFIRMED |

Fix is bidirectional — both story and BC correctly document the crate scope expansion. No residual surface omits prism-dtu-armis.

## Crate-Scope Completeness Re-Check

Fresh-context re-derivation: which production crates does this story's delivery actually touch?

- `crates/prism-demo-server/` — demo-server crate; MultiInstanceServers lifecycle handle; `start_instances` API owner. DOCUMENTED.
- `crates/prism-demo-harness/` — harness crate; MultiInstanceHarness; overlay config wiring. DOCUMENTED.
- `crates/prism-dtu-armis/` — ArmisClone; server-side request counter (AtomicU64 in state.rs + count_request_middleware in clone.rs + GET /dtu/request-count in routes/dtu.rs) added for AC-006 isolation proof. DOCUMENTED (added D-1152).
- Non-exhaustive perimeter gate crate (`tests/external/perimeter-violation/`) — compile-fail gate EXPECTED count update (52→60 at merge). DOCUMENTED (ci.yml + struct_violations.rs enumeration through v61).

No other production crate was modified. The F-P8-HIGH-001 fix completed the crate-scope manifest. No further undocumented scope exists.

## Full Axis Check

| Axis | Result | Detail |
|------|--------|--------|
| SAP-1 — tracing emission catalog completeness | CLEAN | No new `event_type =` emissions since Pass-1. rg 'event_type\s*=' confirms all story-scope emissions pre-date BC-2.16.002 catalog rows |
| SAP-2 — DTU↔TOML schema parity | CLEAN | /dtu/request-count is control-plane; not in sensor TOML [[tables]]; no schema parity gap |
| Gate EXPECTED=60 exact | CLEAN | ci.yml EXPECTED=60; 7 E0639 arms (MultiInstanceServers + 6 other) + 1 E0004 arm; struct_violations.rs enumeration through v61/MultiInstanceServers |
| INV-PERIMETER-001 | INTACT | /dtu/request-count is /dtu/* control-plane, not /api/* application plane; ArmisClone is its own crate with no prism-spec-engine/prism-query/prism-sensors import |
| Isolation proof load-bearing (not paper-fix) | VERIFIED | AtomicU64 server-side counter in ArmisClone state.rs; test asserts delta_b==1 (any cross-tenant request → delta >1 → test fails); server-side, not client-side |
| `unwrap()`/`expect()` in production code paths | CLEAN | Code unchanged since Pass-1 commit 9b4f4154; pre-verified in prior passes |
| BC-2.06.017 v1.7 internal consistency | CLEAN | Postcondition 1 = Ok(MultiInstanceServers); Postcondition 2 = (String,String) keys + 3 REQUIRED overlay fields (extends/instance_id/base_url) per INV-SCALAR-003; EC-017-002 = Err(BindFailure); TV-017-009 present; crates: includes prism-dtu-armis; Postcondition 4 note present |
| Story v1.10 internal consistency | CLEAN | crates_touched = [prism-demo-server, prism-demo-harness, prism-dtu-armis]; Architecture Mapping complete; File Structure Reference complete; AC-006 proof note disambiguated; iter() (not iter_mut()) throughout AC-004/AC-007/Task-5/§Locked API sketch; BC version citations version-agnostic (TD-VSDD-091 fix); H1 heading version-agnostic |
| BC-INDEX / STORY-INDEX title+subsystem+version sync | CLEAN | BC-INDEX v6.50 reflects v1.7; STORY-INDEX v2.378 reflects v1.10 |
| Semantic anchoring (socket_map return type) | CLEAN | `socket_map()` returns `&HashMap<(String,String),SocketAddr>` per U-004/D-1075; String keys confirmed; NOT OrgSlug/SensorId newtypes |
| ArmisClone counter type consistency | CLEAN | AtomicU64 in state.rs (isolation proof); AtomicUsize NOT conflated with AtomicU64; story and BC both reference AtomicU64 for the /dtu/request-count counter explicitly |
| MultiInstanceHarness docs (Drop-only) | CLEAN | No ghost `shutdown()` reference after F-P6-MED-001 fix (D-1151 implementer fix a27b0f54) |
| Watcher-task comments | CLEAN | Describe `shutdown_tx` broadcast + `with_graceful_shutdown` after F-P2-MED-001 fix |
| Test file scaffold comments | CLEAN | All updated to accurate present-tense after F-P4-OBS-1 sweep |
| struct_violations.rs enumeration | CLEAN | Extends through v61/MultiInstanceServers after F-P4-OBS-2 fix |
| ci.yml failure-branch diagnostic message | CLEAN | "60 types (including MultiInstanceServers)" after F-P4-MED-002 fix |
| Overlay format (3 REQUIRED fields) | CLEAN | BC Postcondition 2 + story AC-005/§File-Structure/Task-6 enumerate extends/instance_id/base_url; matches OverlayLoader/INV-SCALAR-003; code correct since Pass-1 |
| AC↔test coverage completeness | CLEAN | 20 tests total: 18 multi_instance/harness + 2 bind-failure; all 7 ACs have corresponding tests |
| No `println!` in production code | CLEAN | Code unchanged since Pass-1 |
| `reqwest::Client` timeout | CLEAN | Not applicable to demo-server/harness/dtu-armis scope (no reqwest calls added) |
| Novelty sweep | ZERO | No new finding candidates identified in any axis |

## Convergence State After Pass 9

- **Streak: 1/3** (advanced from 0/3)
- **CLEAN (strict):** YES
- **CLEAN (PR-merge):** YES
- Code HEAD: unchanged at 9b4f4154 (code stable since Pass-1)
- Story: v1.10 (D-1152 — unchanged this pass)
- BC-2.06.017: v1.7 (D-1152 — unchanged this pass)
- BC-INDEX: v6.50 (unchanged)
- STORY-INDEX: v2.378 (unchanged)
- Next: Pass 10 — need passes 10 + 11 CLEAN(strict) to complete 3/3 streak
