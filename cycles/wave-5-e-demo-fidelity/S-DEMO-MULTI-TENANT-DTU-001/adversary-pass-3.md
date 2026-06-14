---
document_type: adversarial-review
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 3
scope: LOCAL
protocol: BC-5.39.001 3-CLEAN-strict
produced_by: adversary (fresh-context)
timestamp: 2026-06-13T23:59:30Z
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "YES"
findings_total: 3
findings_crit: 0
findings_high: 0
findings_med: 1
findings_low: 2
all_findings_closed: true
streak_before: 0
streak_after: 0
next_pass: 4
---

# LOCAL Adversary Pass 3 — S-DEMO-MULTI-TENANT-DTU-001

**Story:** prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding  
**Protocol:** BC-5.39.001 3-CLEAN-strict (D-779 disambiguation — strict = zero findings of ANY severity)  
**Verdict:** CLEAN(strict)=NO / CLEAN(PR-merge)=YES  
**Streak:** 0/3 (findings present; streak cannot advance)  
**Next:** Pass 4

---

## Part A — Pass-2 Fix Verification

All Pass-2 fixes verified accurate. No regressions.

| Pass-2 Finding | Fix Claimed | Verification Result |
|----------------|-------------|---------------------|
| F-P2-MED-001 (success-path watcher comments wrong, 4 sites) | Implementer corrected all 4 sites to describe shutdown_tx broadcast + with_graceful_shutdown semantic | VERIFIED ACCURATE — comments now describe oneshot/broadcast shutdown signal + graceful drain; no "watcher loop" / "monitoring" language remains; 4 sites confirmed updated in both demo-server and harness multi_instance.rs files. |
| F-P2-MED-002 (ArmisClone doc comment false "mirrors ClarotyState" claim) | Implementer amended to accurately describe AtomicUsize type + origin + isolation test purpose | VERIFIED ACCURATE — new comment correctly notes AtomicUsize type (not AtomicU64), clarifies it does not mirror ClarotyState, and cites the story/pass origin. |
| F-P2-LOW-001 (stale Arc<AtomicUsize> handler-closure comment) | Corrected to "access counter field via Arc::clone for handler closure capture" | VERIFIED ACCURATE — comment now accurately describes field-reference access pattern. |
| OBS-2 (test comment Ok(HashMap::new()) stale parenthetical) | No action taken in Pass-2; carried to Pass-3 | REVIEWED — OBS-2 test comment examined fresh-context. See F-P3-LOW-001 below. |

**SAP-1 check (Pass 3):** Grepped `event_type =` across workspace. All values verified against BC-2.16.002. No new emission sites added by Pass-2 doc-only fix commit. SAP-1 CLEAN.

**INV-PERIMETER-001 check:** `tests/external/perimeter-violation/` compile-fail gate reviewed. EXPECTED=60 confirmed exact. No new E0639/E0004 arms added or removed by Pass-2 fix commit. CLEAN.

**Gate exact:** EXPECTED=60 confirmed. just check GREEN post-Pass-2 fix-burst.

---

## Part B — New Findings (Pass 3)

Classification: **DOC-SWEEP** — no code bugs. Severity pattern: HIGH→MED→MED/LOW. Code is stable; remaining findings are doc/spec-prose correctness issues.

### F-P3-MED-001 [MED] — BC-2.06.017 Postcondition 2 newtype claim contradicted by U-004 / D-1075 / code

**File:** `.factory/specs/behavioral-contracts/BC-2.06.017-per-dtu-instance-multi-address-binding.md`  
**Site:** Postcondition 2 (return-type element description).

**Finding:** BC-2.06.017 v1.3 Postcondition 2 states the `socket_map()` accessor returns "a map from `(OrgSlug, SensorId)` newtype keys to `SocketAddr`." This is contradicted by three authoritative sources:

1. **U-004** (uncertainty resolved during D-1076 T3 finalization): `OrgSlug` and `SensorId` are defined in `prism-core`; the story's `§Locked API sketch` records that the harness key is `(String, String)` — the story intentionally uses plain String to avoid adding a prism-core dependency on prism-dtu-harness (a DTU-layer crate that should not depend on core semantic types). This was an explicit scoping decision.

2. **D-1075** (architect decision on multi-address binding API): the locked API surface in the story uses `HashMap<(String, String), SocketAddr>` as the internal type for `socket_map()` — no newtype wrapping. The story `§Locked API sketch` (v1.5) shows `pub fn socket_map(&self) -> &HashMap<(String, String), SocketAddr>`.

3. **Code** (implementer landing): `socket_map()` returns `&HashMap<(String, String), SocketAddr>`, confirmed by `grep -n "socket_map" crates/prism-dtu-harness/src/multi_instance.rs`.

The BC prose in Postcondition 2 was authored before U-004 resolved the newtype question and was not updated when the story API was locked to `(String, String)`. The mismatch is a spec accuracy defect: downstream readers of BC-2.06.017 (including future test-writers and integration story authors) would expect OrgSlug/SensorId newtypes that do not exist in the implementation.

**Routing:** product-owner (BC amendment — Postcondition 2 prose update).

**Fix applied:** product-owner amended BC-2.06.017 v1.3→v1.4: Postcondition 2 updated to "a map from `(String, String)` (org_slug, sensor_id) string-pair keys to `SocketAddr`; plain String keys are used at the DTU-harness layer to avoid a cross-crate prism-core dependency on a DTU-layer crate (U-004 resolution, D-1075)."

**Status:** CLOSED — BC-INDEX updated v1.3→v1.4. Verified BC-2.06.017 inline row updated. STORY-INDEX story row description updated to reference v1.6 (reflects BC v1.4 + this pass). just check GREEN (no code change — doc/spec only).

---

### F-P3-LOW-001 [LOW] — Residual test comment Ok(HashMap::new()) stale parenthetical (OBS-2 escalated)

**File:** `crates/prism-dtu-harness/src/multi_instance.rs` (test module)  
**Site:** 1 comment in test setup helper.

**Finding:** OBS-2 carried from Pass-2: comment reads "// old API returned Ok(HashMap::new()) — now returns Ok(MultiInstanceServers)". On fresh-context review: the parenthetical `Ok(HashMap::new())` is misleading because the pre-v1.2 return was `Ok(HashMap<String,SocketAddr>)` not `Ok(HashMap::new())` (default value). A reader might incorrectly believe the v1.1 API returned an empty map by default. Escalated to LOW because it introduces a factually wrong historical claim, not just cosmetic noise.

**Status:** CLOSED — implementer removed the parenthetical; comment now reads "// pre-v1.2 API returned Ok(HashMap<String,SocketAddr>); now returns Ok(MultiInstanceServers)." Verified accurate.

---

### F-P3-LOW-002 [LOW] — Doc shows iter_mut() but code consumes socket_map by-value in test harness setup

**Files:**
- `crates/prism-dtu-harness/src/multi_instance.rs` (doc comment in `DemoHarness::start_multi` or equivalent internal helper)
- `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (§Locked API sketch section)

**Finding:** A doc comment inside the harness implementation references `iter_mut()` over `socket_map()` as the canonical access pattern. The `socket_map()` method returns `&HashMap<(String,String),SocketAddr>` (shared reference) — `iter_mut()` is not callable on a shared reference; the caller would need `iter()` for read-only iteration. The story §Locked API sketch also shows `iter_mut()` in a usage example.

The code itself uses `iter()` (correct). The mismatch is purely in the doc comment + story prose example, but it misleads future callers who read the BC or story before reading the code: they may attempt `iter_mut()` and encounter an E0596 "cannot borrow as mutable" error.

**Routing:** implementer (harness doc comment) + story-writer (story §Locked API sketch usage example).

**Fix applied:**
- Implementer: corrected doc comment from `iter_mut()` to `iter()` with explanation "socket_map() returns a shared reference; use iter() for read-only enumeration."
- Story-writer: corrected §Locked API sketch usage example `for (key, addr) in servers.socket_map().iter_mut()` → `for (key, addr) in servers.socket_map().iter()`. Story bumped v1.5→v1.6 (doc/sketch correction, no semantic change).

**Status:** CLOSED — both sites corrected. Story-writer bumped story to v1.6. STORY-INDEX updated. just check GREEN (no code change — doc-only on the fix; code already correct).

---

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRIT | 0 | — |
| HIGH | 0 | — |
| MED | 1 | CLOSED (F-P3-MED-001 — BC Postcondition 2 newtype claim → product-owner BC v1.3→v1.4) |
| LOW | 2 | CLOSED (F-P3-LOW-001 OBS-2 escalated; F-P3-LOW-002 iter_mut doc) |
| OBS | 0 | None |

**All findings CLOSED. just check GREEN post-fix. EXPECTED=60 unchanged. SAP-1 clean. INV-PERIMETER-001 clean. Gate exact 60.**

**CLEAN(strict):** NO — findings present (3 total: 1 MED + 2 LOW; all closed post-fix, but this pass itself was not clean-strict at time of classification).  
**CLEAN(PR-merge):** YES — zero CRIT/HIGH/MED open at close.  
**Streak:** 0/3 (strict criterion; findings were present).  
**Pattern:** HIGH→MED→MED/LOW. Severity decaying. Code stable. Remaining findings are doc/spec-prose sweeps.  
**Next:** Pass 4. Target: CLEAN(strict)=YES. If clean, streak advances to 1/3.
