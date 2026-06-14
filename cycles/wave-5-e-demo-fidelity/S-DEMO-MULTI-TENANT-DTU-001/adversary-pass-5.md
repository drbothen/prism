---
document_type: adversarial-review
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 5
scope: LOCAL
protocol: BC-5.39.001 3-CLEAN-strict
produced_by: adversary (fresh-context)
timestamp: 2026-06-14T00:00:00Z
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "YES"
findings_total: 3
findings_crit: 0
findings_high: 0
findings_med: 3
findings_low: 0
findings_obs: 0
all_findings_closed: true
streak_before: 0
streak_after: 0
next_pass: 6
---

# LOCAL Adversary Pass 5 — S-DEMO-MULTI-TENANT-DTU-001

**Story:** prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding  
**Protocol:** BC-5.39.001 3-CLEAN-strict (D-779 disambiguation — strict = zero findings of ANY severity)  
**Verdict:** CLEAN(strict)=NO / CLEAN(PR-merge)=YES  
**Streak:** 0/3 (findings present; streak cannot advance)  
**Classification:** SPEC-ACCURACY — code unchanged since Pass-1; passes 2–5 are pure doc/spec accuracy sweeps  
**Next:** consistency-audit then Pass 6

---

## Part A — Pass-4 Fix Verification

All Pass-4 fixes verified accurate. No regressions.

| Pass-4 Finding | Fix Claimed | Verification Result |
|----------------|-------------|---------------------|
| F-P4-MED-001 (3 residual iter_mut() sibling sites in AC-004/AC-007/Task-5 not swept by F-P3-LOW-002) | story-writer full `grep -n "iter_mut"` sweep of story file; all 3 corrected to `iter()`; grep-clean confirmed; story v1.6→v1.7 | VERIFIED ACCURATE — grep of story file confirms zero `iter_mut()` occurrences in any AC or Task body; AC-004/AC-007/Task-5 all read `iter()`. |
| F-P4-MED-002 (ci.yml failure-branch message "All 59 types" predating D-1145 MultiInstanceServers addition) | implementer updated failure message to "60 types (including MultiInstanceServers)"; EXPECTED=60 confirmed authoritative | VERIFIED ACCURATE — ci.yml failure-branch message now reads "60 types (including MultiInstanceServers)"; gate check variable references EXPECTED=60. Numerically consistent. |
| OBS-1 (~15 stale scaffold test comments in both multi_instance test files) | test-writer grep-swept both files; all stale scaffold comments updated to accurate present-tense descriptions; grep-clean confirmed | VERIFIED ACCURATE — grep of both test files for scaffold patterns (TODO, WHEN IMPLEMENTED, will panic, will replace, once implemented) returns zero hits. |
| OBS-2 (struct_violations.rs enumeration comment block stopped at v59, omitted v61/MultiInstanceServers) | implementer corrected enumeration to extend through v61 with D-1145 citation | VERIFIED ACCURATE — struct_violations.rs enumeration comment now includes MultiInstanceServers (v61) with D-1145 API-gap adjudication citation. |

**SAP-1 check (Pass 5):** Grepped `event_type =` across workspace (`rg 'event_type\s*=' crates/ --type rust`). All values verified against BC-2.16.002. No new emission sites added by any Pass-4 fix commit (all were doc-only). SAP-1 CLEAN.

**INV-PERIMETER-001 check:** `tests/external/perimeter-violation/` compile-fail gate reviewed. EXPECTED=60 confirmed exact (7 E0639 struct arms + 1 E0004 enum arm = 8 arms total). No arms added or removed by Pass-4 fix commits. CLEAN.

**Semantic anchoring (Pass 5):** Key invariants verified:
- `socket_map()` returns `&HashMap<(String,String),SocketAddr>` — (String,String) keys per U-004/D-1075 (NOT OrgSlug/SensorId newtypes)
- `iter()` (not `iter_mut()`) in ALL doc/sketch usage examples — AC-004, AC-007, Task-5, §Locked API sketch all corrected
- Watcher-task comments describe `shutdown_tx` broadcast + `with_graceful_shutdown` semantic
- ArmisClone request counter is `AtomicUsize` (NOT `AtomicU64` like ClarotyState)
- ci.yml failure-branch message reads "60 types (including MultiInstanceServers)"
- struct_violations.rs enumeration extends through v61 with D-1145 citation
- EXPECTED=60 at merge (ci.yml authoritative)
- Test files grep-clean of all scaffold comment patterns

---

## Part B — New Findings (Pass 5)

Classification: **SPEC-ACCURACY** — code unchanged since Pass-1; these are BC-version-citation drift and overlay-format spec gap findings. All 3 are CLOSED via targeted spec/story amendments. Severity: MED (3); no CRIT/HIGH/LOW/OBS.

---

### F-P5-MED-001 [MED] — Stale "BC-2.06.017 v1.1" citations at 8 story sites (BC now v1.5)

**Files:**
- `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (8 occurrences of "BC-2.06.017 v1.1" or "v1.2" or "v1.3" or "v1.4" in body, ACs, and amendment-history log)

**Finding:** The story file contains multiple inline citations to earlier BC-2.06.017 version numbers (v1.1 through v1.4) in its body sections and amendment-history log. After Passes 1–4 drove BC-2.06.017 from v1.1 → v1.5 (via D-1076→D-1145→D-1146→D-1147→Pass-5-F-P5-MED-003), references to "BC-2.06.017 v1.1" in the story's §Behavioral Contracts BC table row pin and its amendment-history entries present as stale version citations that contradict the authoritative BC file (which is now v1.5).

**Severity rationale:** MED (not LOW) because version-pin citations in the story §Behavioral Contracts table row are read by the implementer and pr-manager to confirm BC currency. A stale "v1.1" pin in that row suggests the story was written against an older BC when it was not — and the BC row constitutes a scope-boundary anchor.

**Routing:** story-writer (story spec amendment — version-pin sweep in §Behavioral Contracts row + amendment-history entries).

**Fix applied:** story-writer performed full `grep -n "v1\.[1-4]" .factory/stories/S-DEMO-MULTI-TENANT-DTU-001-*.md` sweep. All 8 stale version citations corrected: §Behavioral Contracts BC table row pin updated to `BC-2.06.017 v1.5`; amendment-history log entries retain their historical version citations (e.g., "BC-2.06.017 v1.1 (D-1076)") as immutable audit trail — only the §Behavioral Contracts active-pin row is updated to v1.5. Post-fix: BC table pin row reads v1.5; historical amendment-log entries retain their original version numbers. Story bumped v1.7→v1.8 (D-1149 spec-accuracy sweep, no semantic change). STORY-INDEX row updated (v2.375→v2.376 — state-manager records in this burst).

**Status:** CLOSED — grep-clean confirmed (§Behavioral Contracts table row pin reads v1.5; amendment log preserved).

---

### F-P5-MED-002 [MED] — Story H1 "v1.4" vs frontmatter version v1.7 desync

**File:** `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (H1 heading line)

**Finding:** The story file's H1 heading contains a version stamp in the title text, e.g.:

```
# S-DEMO-MULTI-TENANT-DTU-001 — Per-DTU-Instance Multi-Address Binding (v1.4)
```

The frontmatter `version:` field correctly reads `1.7` (post-Pass-4 amendment), but the H1 version annotation was not updated during the v1.4→v1.5→v1.6→v1.7 bumps. This desync means the H1 announces a stale version to any reader who sees the heading before reading the frontmatter.

**Severity rationale:** MED (not LOW) because the H1 heading is the first human-readable content any agent or reviewer reads. A story H1 announcing "v1.4" when the authoritative version is v1.7 creates a trust signal about whether the document is current.

**Routing:** story-writer (story spec amendment — H1 version annotation).

**Fix applied:** story-writer made H1 version-agnostic: the H1 no longer carries an inline version stamp. The version is authoritative in the frontmatter `version:` field; the H1 reads:

```
# S-DEMO-MULTI-TENANT-DTU-001 — Per-DTU-Instance Multi-Address Binding
```

This pattern is consistent with other stories in the corpus (most do not carry inline version annotations in H1). Covered within the story v1.7→v1.8 bump from F-P5-MED-001 fix (same commit; no additional version bump needed).

**Status:** CLOSED — H1 heading now version-agnostic; frontmatter `version: "1.8"` is the sole authoritative version source.

---

### F-P5-MED-003 [MED] [SUBSTANTIVE SPEC-IMPLEMENTATION GAP] — BC/story described overlay as base_url-only but code writes 3 REQUIRED fields

**Files:**
- `.factory/specs/behavioral-contracts/BC-2.06.017-dtu-per-instance-multi-address-binding.md` (Postcondition 2 and TV-017-009)
- `.factory/stories/S-DEMO-MULTI-TENANT-DTU-001-dtu-per-instance-multi-address-binding.md` (§AC-005 + §File-Structure + §Task-6)

**Finding:** BC-2.06.017 Postcondition 2 (v1.4) described overlay wiring as:

> "For each `(org_slug, sensor_id)` key: the overlay file at `<overlays_dir>/<org_slug>/<sensor_id>.yaml` is written with `base_url` overriding the sensor TOML's default URL."

Similarly, the story's AC-005 acceptance criterion and §Task-6 implementation task described overlay wiring as writing only the `base_url` field.

However, a fresh read of `OverlayLoader` (which the story and BC describe in §Architecture Mapping) and `INV-SCALAR-003` (the overlay scalar invariant) reveals that:

1. `OverlayLoader` treats three fields as REQUIRED for a valid overlay: `extends` (sensor TOML filename stem), `instance_id` (unique instance identifier), and `base_url` (URL override).
2. An overlay file containing only `base_url` would be rejected by `OverlayLoader` with a parse/validation error at load time.
3. The actual code produced by the Pass-1 implementer (which passed all Red Gate tests) writes all three fields — this is correct behavior. The spec was underspecified relative to the implementation.

**This is a real spec-implementation gap.** The code is correct; the spec description was incomplete. The BC and story must be amended to accurately describe the 3-field overlay format.

**Severity rationale:** MED (not HIGH) because: the code is correct and tests pass; no runtime regression exists; the gap is a spec-accuracy issue (description of overlay format was underspecified). However, it must be classified MED because a test-writer or implementer reading the BC/story verbatim could produce an incorrect overlay file (base_url-only) that fails at load time.

**Routing:** product-owner (BC-2.06.017 amendment: Postcondition 2 + TV-017-009); story-writer (AC-005, §File-Structure overlay snippet, §Task-6).

**Fix applied:**
- **product-owner:** BC-2.06.017 v1.4→v1.5. Postcondition 2 amended: overlay file writes 3 REQUIRED fields — `extends` (sensor TOML filename stem, e.g. `armis`), `instance_id` (org_slug + sensor_id composite, e.g. `org-a/armis`), and `base_url` (the per-instance DTU address). TV-017-009 added (or amended if present): test vector describing valid overlay format with all 3 fields. INV-SCALAR-003 cross-reference added to Postcondition 2.
- **story-writer:** S-DEMO-MULTI-TENANT-DTU-001 v1.7→v1.8. AC-005 acceptance criterion body updated to specify all 3 overlay fields. §File-Structure overlay YAML snippet updated to show all 3 fields. §Task-6 implementation task description updated to specify all 3 fields when writing overlay files. BC-2.06.017 version pin in §Behavioral Contracts row updated from v1.4 to v1.5.

**Status:** CLOSED — BC-2.06.017 v1.5 + story v1.8 both accurately describe the 3-field overlay format. INV-SCALAR-003 cross-referenced. just check unaffected (no code touched this pass — code was already correct).

---

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRIT | 0 | — |
| HIGH | 0 | — |
| MED | 3 | CLOSED (F-P5-MED-001 BC-version-citation sweep v1.5; F-P5-MED-002 H1 version-agnostic; F-P5-MED-003 overlay 3-field spec gap BC v1.4→v1.5 + story v1.7→v1.8) |
| LOW | 0 | — |
| OBS | 0 | — |

**All findings CLOSED. just check GREEN (code UNCHANGED since Pass-1 — all passes 2–5 are pure doc/spec/CI accuracy sweeps). EXPECTED=60 confirmed. SAP-1 clean. INV-PERIMETER-001 clean. Gate exact 60.**

**Note: code unchanged since Pass-1. Passes 2–5 have been pure doc/spec/CI-diagnostic accuracy sweeps. F-P5-MED-003 is the most substantive finding of this cascade — a real spec-implementation gap (BC underspecified overlay format; code was correct; spec now matches). Comprehensive consistency audit planned before Pass-6 to eliminate any remaining cross-document citation drift.**

**CLEAN(strict):** NO — 3 MED findings present (all closed post-fix; this pass was not clean-strict at time of classification).  
**CLEAN(PR-merge):** YES — zero CRIT/HIGH/MED open at close.  
**Streak:** 0/3 (strict criterion; findings present — streak cannot advance).  
**Pattern:** HIGH(P1) → MED(P2) → MED/LOW(P3) → MED/OBS(P4) → MED(P5). Code stable. Passes 4–5 reached comprehensive sweep coverage across all doc/spec surfaces.  
**Next:** consistency-audit then Pass 6. Target: CLEAN(strict)=YES (zero findings of ANY severity). F-P5-MED-003 overlay gap closure + BC-version-citation sweep from this pass reduce residual risk for Pass-6.
