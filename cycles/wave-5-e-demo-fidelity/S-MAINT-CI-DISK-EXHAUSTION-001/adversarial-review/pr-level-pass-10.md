---
pass: 10
story: S-MAINT-CI-DISK-EXHAUSTION-001
lane: PR-LEVEL
frozen_head: 0939973f
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
date: 2026-07-16
---

# S-MAINT-CI-DISK-EXHAUSTION-001 PR-LEVEL Pass 10

**Frozen HEAD:** 0939973f
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**Streak:** 0/3 (reset per DRIFT-ORCH-PRLEVEL-PUSH-001 — fix-burst-13 code @0939973f pushed new HEAD)

## Findings Summary

### CRITICAL (1)

- **F-MAINT-P10-CRIT-001** — Mirrorlist-blind fallback: when the GitHub Actions runner has `/etc/apt/apt-mirrors.txt` present, apt treats `ubuntu.sources` as using the `mirror+file:` protocol. The `ubuntu.sources` file has no `mirror+file:/etc/apt/apt-mirrors.txt` stanza; apt silently falls back to the first available mirror from the list OR fails with a protocol error. Both CI runs on frozen HEAD 0939973f failed on this path. The fallback logic in AC-006 (v0.18) assumes the fallback host (`archive.ubuntu.com`) is always reachable via `http://`, but the mirrorlist mechanism pre-empts the fallback entirely — apt never tries the http:// fallback when the mirror+file protocol is active. Empirical runner probe confirmed: ubuntu.sources on the GitHub Actions runner uses `mirror+file:/etc/apt/apt-mirrors.txt` as the primary mirror protocol. Correct intervention: overwrite `/etc/apt/apt-mirrors.txt` with a known-good snapshot (single entry: `http://archive.ubuntu.com/ubuntu`) BEFORE any apt-get invocation, so the mirror protocol resolves predictably.

### HIGH (1)

- **F-MAINT-P10-HIGH-001** — azure-cli.sources missed: the Microsoft Azure CLI apt source file (`/etc/apt/sources.list.d/azure-cli.list` or `azure-cli.sources`) was not addressed by the AC-006 third-party source isolation logic in v0.18. The `sed` host-anchored rewrite targets `archive.ubuntu.com` entries only; Microsoft's CDN (`packages.microsoft.com`) entries remain untouched. When the mirrorlist mechanism fails (F-MAINT-P10-CRIT-001), the azure-cli source file can produce spurious 403 errors during `apt-get update` that are counted as failures under the AC-005 three-green-run criterion. AC-006 must explicitly enumerate third-party source files to be quarantined or isolated before the apt-get update + install sequence.

### MEDIUM (2)

- **F-MAINT-P10-MED-001** — AC-005 evidence void: both CI runs on frozen HEAD 0939973f failed (F-MAINT-P10-CRIT-001). AC-005 requires 3 consecutive green `pull_request` CI runs before merge gate clears. The count resets to 0/3 on any failure. Neither run on 0939973f was green; AC-005 is at 0/3.

- **F-MAINT-P10-MED-002** — Echo arithmetic stale: the summary echo at the end of the disk-space-reclaimer step claims specific counts for reachability checks and configuration checks. After the v0.18 AC-006 redesign (third-party source removal + host-anchored sed), the echo arithmetic was not re-verified against the new operation count. The echo may undercount or overcount the total checks, producing misleading CI diagnostics.

### LOW (2)

- **LOW-001** — PR description stale: PR #224 description was refreshed against bd65e93a (pass-9 era). Fix-burst-13 pushed 0939973f with AC-006 redesign; the PR description still cites bd65e93a commit SHAs, the old fallback strategy description, and the pass-9 fix narrative. Must be refreshed against frozen HEAD 0939973f before merge gate re-evaluation.

- **LOW-002** — Story v0.18 ACR/FP carve-outs need updated validation: AC-006 v0.18 includes `## Acceptance Criteria Revision (ACR)` and `## Forbidden Patterns` carve-out annotations that reference the pass-8 MED-001 finding (ACR/FP self-contradiction). After the v0.18 structural redesign (CRIT-level fallback failure), these carve-outs may no longer fully address the now-revised AC-006 scope. The carve-outs should be audited against v0.19 when story-writer produces it.

### OBSERVATION (2)

- **OBS-001** — apt-mirrors.txt overwrite requires validation that the single-entry snapshot is valid for ubuntu 22.04 LTS on GitHub Actions runners in all relevant AZs; the empirical probe confirmed the path and protocol, but did not verify the CDN hostname availability under all GitHub Actions runner geographic distributions.

- **OBS-002** — Diagnostic echo messages use `::error::` annotations for threshold failures; if `grep -c` exits non-zero (zero matches = forbidden pattern present), the `::error::` annotation fires before the step's exit code propagates. Pre-existing OBS from pass-8 (LOW-002 carryover); not fully addressed in v0.18.

### PROCESS-GAP (1)

- **PG-001** — Pass-10 CI evidence gap: both CI runs failed; no green run exists on frozen HEAD 0939973f. The AC-005 three-green-run evidence clock requires CI runs on EACH frozen HEAD. After fix-burst-13 pushed 0939973f, CI was expected to start running on the new HEAD; however, both runs hit the mirrorlist-blind CRIT before any apt packages were installed. No disk-space-reclaimer evidence (AC-001/002/003/004/006/007 step output) is available from the pass-10 frozen HEAD. Story v0.19 amendment must redesign AC-006 to address the mirrorlist-blind path before AC-005 can restart.

## Disposition

All findings require AC-006 redesign at the spec layer. Empirical runner probe (D-1795) confirmed the correct intervention: overwrite `/etc/apt/apt-mirrors.txt` with a known-good snapshot before any apt-get invocation. Story v0.19 amendment dispatched (story-writer) to encode the apt-mirrors.txt overwrite strategy. Fix-burst-14 (implementer) will apply v0.19 to ci.yml + e2e.yml after spec layer confirmation. AC-005 evidence clock restarts on the fix-burst-14 frozen HEAD. Streak 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001.
