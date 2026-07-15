---
document_type: adversarial-review
scope: LOCAL
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
passes: [1]
feature_head_at_review: f1ae322f
date: 2026-07-15
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 4
  crit: 0
  high: 0
  med: 1
  low: 2
  obs: 1
  process_gap: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 1 — S-MAINT-CI-DISK-EXHAUSTION-001

---

## Pass 1 (frozen f1ae322f; fresh-context adversary; CI disk-exhaustion hardening; streak candidate — 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 4 total (0 CRIT / 0 HIGH / 1 MED / 2 LOW / 1 OBS / 0 PROCESS-GAP)

**STREAK: 0/3** — MED finding is merge-blocking; all findings are spec-snippet-rooted.

**Code HEAD at review:** f1ae322f (Red Gate @43f532ab → implementation; AC-001..AC-004 wired; LOCAL-ONLY on maintenance/ci-disk-hardening)

**CLEAN(strict):** NO — 1 MED + 2 LOW + 1 OBS findings present

**CLEAN(PR-merge):** NO — 1 MED merge-blocking finding present

---

## Finding Register

### F-CIDISK-P1-MED-001 [MED] AC-001 grep inert — `df -h` alternation matched sibling steps

**Severity:** MED

**Classification:** implementation defect — assertion does not exercise the contract it claims

**Description:**
The AC-001 verify-workflow-structure grep used `df -h` as its search term. Because `df -h` appears verbatim in sibling step bodies (the AC-002 post-reclaim verification and the AC-004 failure annotation), the grep matched on those lines rather than the "Report initial disk space" preflight step. The assertion counted matches from wrong steps, making it inert for its intended purpose.

**Fix required:** Replace the grep with a step-name anchor: `grep -cE '^\s+- name: Report initial disk space\s*$' .github/workflows/ci.yml`. The `^\s+- name:` prefix is indent-agnostic, the `\s*$` suffix prevents partial matches, and the assertion line itself starts with `count=$(grep...` — not `- name:` — making it self-match-proof.

---

### F-CIDISK-P1-LOW-001 [LOW] Misleading gsub in ≥25 GB gate

**Severity:** LOW

**Classification:** implementation correctness — silent field-parsing error on non-standard locale

**Description:**
The ≥25 GB gate extracted the available-space field using a `gsub` substitution on the `df -h` human-readable output. Human-readable output uses locale-dependent separators (e.g., a period vs comma for thousands) and abbreviated suffixes (G, M, T) that are not uniformly parseable. A `gsub` pattern that strips non-numeric characters produces incorrect values when the field contains embedded locale punctuation.

**Fix required:** Use `df -P /` (POSIX format, 1K-block columns) and extract field 4 (`$4`) via awk, then convert from 1K-blocks to GiB: `AVAIL_GB=$(df -P / | awk 'NR==2 { print int($4 / 1024 / 1024) }')`. POSIX output is locale-invariant.

---

### F-CIDISK-P1-LOW-002 [LOW] USED_PCT empty on df parse failure → secondary failure

**Severity:** LOW

**Classification:** defensive coding gap — empty variable causes arithmetic error

**Description:**
The gate used a `USED_PCT` variable derived from `df` output without guarding against empty-string expansion. If `df` fails (e.g., slow filesystem enumeration or signal), `USED_PCT` is empty; the subsequent numeric comparison (`-ge`) raises `bash: [: -ge: unary operator expected`, masking the real disk-state check behind a bash error.

**Fix required:** Add a default-value guard: `AVAIL_GB=${AVAIL_GB:-0}`. With the `df -P` form (F-CIDISK-P1-LOW-001 fix), the variable to guard is `AVAIL_GB`, defaulting to 0 to force the ≥25 GB gate to fail-fast with the diagnostic message rather than a bash error.

---

### F-CIDISK-P1-OBS-001 [OBS] Linux-only scope not stated in step conditions

**Severity:** OBS

**Classification:** spec-documentation gap — impl scope ambiguity

**Description:**
The AC-001, AC-002, and AC-004 steps are intended for Linux runners only (ubuntu-latest; macOS and Windows legs are exempt per the AC spec). The ci.yml implementation did not add `if: runner.os == 'Linux'` guards or equivalent runner-matrix scoping to these steps. A future author adding a macOS or Windows job leg would unknowingly inherit these steps.

**Fix required:** The Linux-only scope should be documented via an inline comment in ci.yml adjacent to the step definitions, noting that these steps are intentionally absent from macOS/Windows matrix legs. Alternatively, if the matrix is restructured, add explicit `if: runner.os == 'Linux'` guards.

---

## Fix-Burst 2 Closure Audit

All 4 findings above were closed in fix-burst-2:

**story-writer:** S-MAINT-CI-DISK-EXHAUSTION-001 v0.3→v0.4 — AC-001 spec updated to name-anchored grep (`^\s+- name:` form, count ≥ 2); AC-002 spec updated to `df -P` form + `AVAIL_GB` variable with `${AVAIL_GB:-0}` guard; OBS-001 Linux-only scope added to AC spec prose.

**implementer @99f20d13:** ci.yml updated — AC-001 verify-workflow-structure assertion replaced with `^\s+- name: Report initial disk space\s*$` name-anchor grep; AC-002 post-reclaim gate replaced with `df -P /` POSIX form; `AVAIL_GB=${AVAIL_GB:-0}` guard added; inline Linux-only scope comment added to affected steps.

**Result after FB-2:** HEAD @99f20d13 on maintenance/ci-disk-hardening (LOCAL-ONLY). Streak 0/3.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** N/A — this story modifies `.github/workflows/ci.yml` only; no `event_type =` assignments in scope.

**SAP-2:** N/A — no sensor TOML spec modifications.

**SID-1:** N/A — Red Gate assertions are verify-workflow-structure bash assertions, not `#[ignore]`'d Rust tests.

---

## Convergence Assessment

**Pass 1 on frozen f1ae322f:** 0/3 (1 MED + 2 LOW + 1 OBS; all spec-snippet-rooted; fix-burst-2 closed all 4)

**Cascade tally at FB-2 close:** 1 pass / 1 fix-burst.

**New HEAD after FB-2:** @99f20d13 (LOCAL-ONLY).

**NEXT:** LOCAL pass 2 on frozen @99f20d13 (streak 0/3).
