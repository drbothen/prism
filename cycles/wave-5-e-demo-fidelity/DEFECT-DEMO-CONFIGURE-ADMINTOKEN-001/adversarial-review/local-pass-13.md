---
document_type: adversarial-review-pass
story: DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001
pass: 13
scope: LOCAL
reviewer: general-purpose-as-adversary
frozen_head: 4feac52b
story_version: v0.13
bc_versions:
  BC-2.06.017: v1.12
date: 2026-07-17
clean_strict: false
clean_pr_merge: true
findings_summary: "0 CRIT / 0 HIGH / 0 MED / 4 LOW / 1 OBS — first zero-MED+ pass of the cascade"
streak_after: "0/3 (new HEAD 558d5881 from fb-12 pushes; DRIFT-ORCH-PRLEVEL-PUSH-001 streak reset)"
next_pass: LOCAL pass-14 on frozen 558d5881
---

# Adversarial Review — DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 LOCAL Pass 13

**Reviewer:** general-purpose-as-adversary (fresh context; no prior pass reports read)
**Frozen HEAD:** `4feac52b`
**Story version:** v0.13
**BC versions:** BC-2.06.017 v1.12; BC-3.6.001 (current)
**Date:** 2026-07-17
**Cascade tally:** 13 passes / 11 fix-bursts

## Verdict

```
CLEAN (strict):    NO   — 4 LOW + 1 OBS present
CLEAN (PR-merge):  YES  — 0 CRIT / 0 HIGH / 0 MED
```

**First zero-MED+ pass of the entire DEFECT-DEMO-CONFIGURE-ADMINTOKEN-001 cascade.**

Streak: 0/3 (reset by fb-12 push at 558d5881 per DRIFT-ORCH-PRLEVEL-PUSH-001).

---

## Positive Verifications

- **Sweep counts reproduce at HEAD:** write_multi_admin_token_sidecar_to_path 447 grep hits, write_token_sidecar_to_path 131 hits, token_map() 6 hits, TOKEN_MULTI_FILE 8 hits — all match reported values.
- **Phantom-anchor closure verified:** zero live citations of `§Sidecar-availability` anchor anywhere in workspace (`rg '§Sidecar-availability' .factory/ crates/` returned empty).
- **GAP-3 citations semantically accurate:** "URL sidecar polling gap" prose in story correctly names the gap and references the GAP-3 label from BC-2.06.017 §Notes.
- **POL-22 Phase A/C verbatim compliance:** ADR quotes in story AC-001, AC-002, AC-003 all verbatim against `architecture/adr/ADR-022-dependency-injection-framework.md` §A and §C (6 quoted passages verified word-for-word).
- **SAP-1 zero new emissions:** `rg 'event_type\s*=' crates/prism-dtu-demo-server/src/ --type rust` returns 0 hits in the fb-11 diff (no new tracing emissions; no BC-2.16.002 catalog update needed).
- **AD-017 clean:** no credential values in story text, no token literals in diff.
- **nextest green:** 59 passing + 3 known pre-existing fails on `4feac52b` (T-01 through T-09 GREEN; T-10 known DTU-EXT-001 integration skip).
- **Test G 10/10:** write_multi_admin_token_sidecar_to_path fixture-gen verified — generates files for all 10 fixture orgs, each at correct path pattern, each with correct 0600 permissions.

---

## Findings

### F-ADMTOK-P13-LOW-001 [LOW] [process-gap] — fb-11 determinism sweep incomplete: 4 unsorted HashMap-key diagnostic lists

**Severity:** LOW (process-gap tag: determinism property missing from sweep scope)

**Location:** `crates/prism-dtu-demo-server/src/harness.rs` — two NEW-code sites in `write_multi_admin_token_sidecar_to_path`; plus sibling mirror sites in `write_multi_url_sidecar_to_path`.

**Description:**

The fb-11 fix-burst closed F-ADMTOK-P12-LOW-001 (sorted iteration order) by adding `.sorted()` to the `admin_token_map().iter()` walk in the primary `write_multi_admin_token_sidecar_to_path` call path. However, the fix-burst sweep was incomplete:

1. `write_multi_admin_token_sidecar_to_path` — **two NEW-code diagnostic-list sites** (error accumulation paths that collect per-org failure strings into a `Vec`) iterate over `HashMap` keys in unspecified order. In test contexts with multiple orgs, failure message ordering is non-deterministic across runs.
2. `write_multi_url_sidecar_to_path` — **two SIBLING sites** (same pattern; were not in-scope for fb-11 because they use `url_map()` not `admin_token_map()`) have the identical un-sorted iteration pattern.

The SWEEP-MIRROR convention (AC-004 ¶1, story v0.13) requires that any fix applied to `write_multi_admin_token_sidecar_to_path` is mirrored to `write_multi_url_sidecar_to_path`. The sort fix was applied to the primary iteration but NOT propagated to the diagnostic-list accumulation path in either function.

**Impact:** Test flakiness under multi-org fixture loads; error messages in `E-DEMO-007` structured content arrive in non-deterministic order when multiple sidecars fail.

**Required fix:** Add `.sorted()` (or `.collect::<BTreeSet<_>>()`) to all four diagnostic-list accumulation sites. Implement a load-bearing test assertion that verifies sorted error-message order (Test F variant or new Test K).

---

### F-ADMTOK-P13-LOW-002 [LOW] — Story structural tables misdescribed as-built file structure: harness.rs surfaces invisible to spec

**Severity:** LOW

**Location:** Story §File Structure table + §Architecture Mapping table + §Purity Classification table — all missing `harness.rs` write_token_sidecar_to_path + token_map().

**Description:**

Story v0.13 §File Structure lists the implementation files touched by the defect fix. The table is missing:

- `crates/prism-dtu-demo-server/src/harness.rs` — contains `write_token_sidecar_to_path` (single-instance path) and `token_map()` (accessor used by the multi-instance path)

This means `harness.rs` contribution is invisible to any reader using the story as the implementation map. Additionally:

- §Architecture Mapping does not list `harness.rs` as a touched subsystem component
- §Purity Classification does not classify `harness.rs` mutation (it is effectful I/O — file write — so it belongs in the Effectful tier)

Separately: AC-002 acceptance bullet still claims "write_url_sidecar writes TOKEN_FILE to sidecar path". This is factually wrong — `write_url_sidecar_to_path` writes the URL sidecar, not TOKEN_FILE. TOKEN_FILE is written by `write_token_sidecar_to_path` (in `harness.rs`). The AC-002 bullet conflates the two.

**Required fix:** Add `harness.rs` row to §File Structure + §Architecture Mapping + §Purity Classification. Correct AC-002 bullet to accurately describe which function writes which file.

---

### F-ADMTOK-P13-LOW-003 [LOW] — AC-004 ¶1 "naming each enumerated site" never reconciled with SWEEP-MIRROR condensed-mirror convention

**Severity:** LOW

**Location:** Story v0.13 AC-004 §Acceptance Criteria, paragraph 1.

**Description:**

AC-004 ¶1 states: "Any sweep fix applied to `write_multi_admin_token_sidecar_to_path` MUST be mirrored to `write_multi_url_sidecar_to_path`, naming each enumerated site."

The phrase "naming each enumerated site" was introduced in story v0.10 but was never reconciled with the v0.10 SWEEP-MIRROR condensed-mirror convention adopted in the same version. The condensed-mirror convention (SWEEP-MIRROR) records the site count and disposition in a table, not a named-site list. The current AC-004 ¶1 text creates an apparent contradiction: SWEEP-MIRROR produces a table with site counts; "naming each enumerated site" implies a full per-site named list.

Implementers reading AC-004 ¶1 in isolation will apply the named-site list requirement (which is more burdensome than SWEEP-MIRROR). The correct text should reference the SWEEP-MIRROR format explicitly and not use the phrase "naming each enumerated site."

**Required fix:** Rewrite AC-004 ¶1 to codify the SWEEP-MIRROR convention format (disposition table with site counts + sample anchor). Remove the ambiguous "naming each enumerated site" phrasing.

---

### F-ADMTOK-P13-LOW-004 [LOW] [process-gap] — BC-INDEX v8.34 changelog narrative copied pass-12 PROPOSED fix instead of fix-as-landed

**Severity:** LOW (process-gap tag)

**Location:** BC-INDEX.md changelog v8.34 entry (~line 415 pre-correction).

**Description:**

The v8.34 BC-INDEX changelog entry stated: "Postcondition 1 phantom `§Sidecar-availability` anchor citation stripped and replaced with correct `§Postconditions/Postcondition-7` citation form."

This description is **factually wrong**. `Postcondition 7` is the duplicate-key semantics postcondition — it is structurally unrelated to the sidecar-availability pattern described in Postcondition 1. The actual BC-2.06.017 v1.12 fix replaced the phantom anchor with `(tmp+rename, same atomic-write pattern as the URL sidecar; cf. GAP-3 sidecar-poll note, S-DEMO-LAUNCHER-CONSOLIDATION-001 Changelog v2.1)`.

The v8.34 narrative appears to have been copied from a pass-12 PROPOSED fix description that cited Postcondition-7 as a destination anchor. The fix-as-landed did not use a Postcondition-7 anchor.

**Required fix:** Correct v8.34 changelog narrative in-place; add v8.35 row documenting the correction (POL-11, POL-26). State-manager closure.

---

### F-ADMTOK-P13-OBS-001 [OBS] — Test J doc comment wrong mechanism claim "BTreeMap-backed hash seed"

**Severity:** OBS (observational)

**Location:** `crates/prism-dtu-demo-server/tests/` Test J doc comment.

**Description:**

Test J's doc comment states the determinism property is ensured by "BTreeMap-backed hash seed." This is mechanistically wrong. The determinism guarantee at `4feac52b` comes from calling `.sorted()` on the `HashMap` iterator before writing sidecars, not from any BTreeMap backing of the hash seed. The HashMap seed is not BTreeMap-backed; the sort is applied at the iterator level.

The wrong mechanism claim in a test doc comment misleads future maintainers about where to look if determinism breaks.

**Required fix:** Correct Test J doc comment to state the actual mechanism: `.sorted()` iterator on HashMap keys guarantees lexicographic write order.

---

## Resolution

| Finding | Severity | Closed by | Mechanism |
|---------|----------|-----------|-----------|
| F-ADMTOK-P13-LOW-001 | LOW [process-gap] | implementer @558d5881 | 4 `.sorted()` additions to diagnostic-list accumulation paths + exhaustive SWEEP-MIRROR table with disposition column + new Test F sorted-order load-bearing assertion |
| F-ADMTOK-P13-LOW-002 | LOW | product-owner story v0.14 | harness.rs added to §File Structure + §Architecture Mapping + §Purity Classification; AC-002 bullet corrected |
| F-ADMTOK-P13-LOW-003 | LOW | product-owner story v0.14 | AC-004 ¶1 rewritten to codify SWEEP-MIRROR convention; "naming each enumerated site" removed |
| F-ADMTOK-P13-LOW-004 | LOW [process-gap] | state-manager D-1799 | BC-INDEX v8.34 narrative corrected in-place; v8.35 changelog row added |
| F-ADMTOK-P13-OBS-001 | OBS | implementer @558d5881 | Test J doc comment corrected to ".sorted() iterator on HashMap keys" mechanism |

All findings closed. Streak reset to 0/3 by fb-12 push (558d5881). Next = LOCAL pass-14 on frozen 558d5881.
