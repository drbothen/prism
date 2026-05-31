---
document_type: adversarial-review
cycle: wave-0-plugin-prereqs
story_id: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: PR-2
type: PR-LEVEL
date: 2026-05-30
feature_head: "76e9684e"
pr_number: 164
base_branch: develop
base_head: "e898c3c9"
diff_artifact_supplied: true
clean_strict: false
clean_pr_merge: false
findings_count: 2
findings_by_severity:
  MED: 1
  OBS: 1
streak_after_pass: 0
target_streak: 3
status: "FB-PR2 CLOSED — Pass 3 next"
---

# PR-LEVEL Adversary Pass 2 — S-DTU-CYBERINT-AUTH-FIDELITY-001 PR #164

## Header

- **Pass:** PR-LEVEL Pass 2
- **Date:** 2026-05-30
- **Feature HEAD at review:** 76e9684e (FB-PR1 fix: stale comment pipeline.rs)
- **PR:** #164 (feature/S-DTU-CYBERINT-AUTH-FIDELITY-001 → develop)
- **Base develop HEAD:** e898c3c9 (S-5.01-FOLLOWUP-MCP-BOOT merge, 2026-05-29T16:44:42Z)
- **Diff artifact:** SUPPLIED (resolved OBS-PR1-001 read-only gap)
- **CLEAN(strict):** NO (1 MED + 1 OBS finding)
- **CLEAN(PR-merge):** NO (1 MED finding present)
- **Streak after pass:** 0/3 (reset by MED finding)

## Findings

### F-PR2-MED-001 [MED] — BC-2.01.017 E-AUTH-004/No-Retry Contract Unimplemented at Pipeline Layer

**Severity:** MED
**Status:** CLOSED by FB-PR2 (implementer commit dd244736 + story-writer commit dc72c7a3 + BC 216f8983)

**Description:** BC-2.01.017 EC-017-002 specifies that `CookieRoundtrip` sensors must return `E-AUTH-004` on 401 with NO retry. However, `pipeline.rs` lines 748-822 applied OAuth2-style refresh-retry logic to `CookieRoundtrip` 401 responses: it called `acquire_token()` again (equivalent to retry), emitted false `auth_refresh_*` events, and returned `AuthRefreshFailed` — contradicting BC-2.01.017.

**Root cause:** Spec-vs-spec conflict. The story's EC-006 described a retry path for CookieRoundtrip that contradicted BC-2.01.017's authoritative no-retry contract. The story was authored before BC-2.01.017 was finalized.

**PO adjudication:** BC-2.01.017 is AUTHORITATIVE per CLAUDE.md Source-of-Truth Precedence rule #1 (BC supersedes story when conflict is about contract semantics). BC unchanged. Story and code corrected.

**Resolution (FB-PR2):**
- Implementer commit dd244736: `pipeline.rs` `CookieRoundtrip` 401 path rewired — instead of refresh-retry, now immediately aborts with `SpecEngineError::CookieAuthFailed` (E-AUTH-004) and emits `cookie_auth_401` audit event per BC-2.16.002 catalog row added at 216f8983. No retry. No `auth_refresh_*` events on this path.
- Story-writer commit dc72c7a3: Story spec v1.5→v1.6 — EC-005 corrected (E-AUTH-004 for 401-no-retry, not E-AUTH-005), EC-006 corrected (no retry for CookieRoundtrip, contradicting prior text), AC-010¶1 corrected (CookieAuthFailed not AuthRefreshFailed), Task-20 sibling corrected to match. Also: E-AUTH-004→E-AUTH-005 correction for credential-not-found path (EC-005 sibling fix).
- BC-2.16.002 commit 216f8983: `cookie_auth_401` WARN event row added to Canonical Structured Event Catalog (v1.60, catalog count 67→68). TV-006 test added.

---

### OBS-PR2 [OBS] [process-gap] — Worktree Path Resolution Hazard

**Severity:** OBS (process-gap)
**Status:** Registered for Cycle-Closing-Checklist; non-blocking to merge

**Description:** During PR-LEVEL review, read-only tool calls (Read, Grep) resolved against the main develop checkout rather than the per-story worktree. This creates a systematic risk: if the feature branch has diverged from develop on a file being reviewed, the adversary reads the wrong version. In this pass, mitigation was applied by confirming known-new symbols (e.g., `cookie_auth_401`, `CookieAuthFailed`) existed in the reviewed code before accepting the pass.

**Recommended mitigation (process-gap):** PR-review dispatch must mandate absolute worktree-prefixed paths for all Read/Grep tool calls. A known-new symbol resolution check should be performed as the first adversary action to verify correct worktree is being read.

**Registration:** Cycle-Closing-Checklist. Requires follow-up story or justified deferral before cascade CLOSE. Non-blocking to merge.

---

## Probe Results

| Probe | Result | Notes |
|-------|--------|-------|
| SAP-1 (tracing event catalog) | PASS (after FB-PR2) | `cookie_auth_401` catalog row added at 216f8983; all event_type sites now cataloged |
| SAP-2 (DTU↔TOML schema parity) | PASS | No TOML/DTU schema changes in FB-PR2 |
| SID-1 (no-ignored-test rationalization) | PASS | TV-006 test non-#[ignore]'d unit test |
| POL-10 (no-pragma-once convergence) | PASS | No shortcut language |
| POL-12 (no todo!/unimplemented!) | PASS | No todo!() in changed paths |

## Summary

**CLEAN(strict):** NO — 1 MED + 1 OBS finding
**CLEAN(PR-merge):** NO — MED finding (F-PR2-MED-001) blocks PR merge gate
**Streak:** 0/3 (reset from 0/3 by MED finding; streak was already at 0 after Pass 1)
**Root cause adjudication:** PO adjudicated spec-vs-spec conflict (BC-2.01.017 wins per Source-of-Truth Precedence #1). BC unchanged. Story v1.5→v1.6 (dc72c7a3). Code fixed (dd244736). BC-2.16.002 v1.59→v1.60 catalog row added (216f8983).
**Process-gap registered:** OBS-PR2 worktree-path-resolution hazard — Cycle-Closing-Checklist.
**Next:** FB-PR2 all three artifacts committed. Push feature branch to remote with FB-PR1+FB-PR2 commits. CI must re-run. PR-LEVEL Pass 3 dispatched (fresh context against HEAD dd244736).
