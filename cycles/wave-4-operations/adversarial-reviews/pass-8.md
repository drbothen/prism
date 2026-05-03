---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 8
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-03T20:00:00Z
predecessor: pass-7.md (BLOCKED 5 findings; remediated 2026-05-03)
verdict: BLOCKED
findings_count: 6
severity_breakdown: { CRITICAL: 0, HIGH: 3, MEDIUM: 2, LOW: 1, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-03
remediation_commits: [246b9f71]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 8

**Verdict: BLOCKED** — 6 findings (0C / 3H / 2M / 1L / 0OBS). Pass 7 remediation
surfaced a partial-fix regression pattern: fixing one BC title sync exposed a misaligned
SMTP auth order and a 1-second cron tick vs 60-second default gap that Pass 7 did not
sweep. Convergence window reset to 0/3.

**Trajectory:** 38→17→8→7→7→5→5→6 (slight uptick; sibling-fix sweep gap — each Pass-N
fix creates a new gap in adjacent specs not swept in the same burst).

---

## Findings

### HIGH Findings

#### P8-S-4.08-A-H-001 — SMTP Auth Order Partial-Fix Regression

**Severity:** HIGH
**Location:** `stories/S-4.08-action-delivery.md` §3 (SMTP channel spec) + `specs/architecture/decisions/ADR-016-action-delivery-framework.md` §2.3
**Finding:** Pass 7 fixed BC-2.18.004 title sync but did not sweep the SMTP auth order
sequence. ADR-016 §2.3 specifies the SMTP channel must attempt XOAUTH2 first, fall back
to PLAIN, and finally reject with `E-AD-018` if neither succeeds. S-4.08 AC-6 lists the
auth methods in the reverse order (PLAIN → XOAUTH2) without an explicit fallback chain
annotation. A consumer implementing AC-6 as written would attempt PLAIN before XOAUTH2,
violating the ADR-016 §2.3 precedence invariant. This is a partial-fix regression — the
Pass 6 consumer-table sweep fixed BC titles but did not reconcile the auth order
narrative in S-4.08 against ADR-016 §2.3's canonical sequence.

**Required Fix:** S-4.08 AC-6 must be updated to cite XOAUTH2-first, PLAIN-fallback,
`E-AD-018` terminal error in that order, matching ADR-016 §2.3. The ADR-016 §2.3
narrative is the SoT; S-4.08 AC-6 is the derivative.

**Fix Applied (2026-05-03):** S-4.08 v1.15→v1.16: AC-6 auth order corrected to
XOAUTH2→PLAIN→E-AD-018 matching ADR-016 §2.3. Regression closed.

---

#### P8-BC-2.18.001-A-H-002 — Retry/Dead-Letter CF Keys Missing OrgId Prefix + Discriminator

**Severity:** HIGH
**Location:** `specs/behavioral-contracts/BC-2.18.001-action-at-least-once-delivery.md` §3 (CF key schema)
**Finding:** BC-2.18.001 v1.4 (Pass 6 remediation) updated the backoff sequence to
2/4/8/16/32s per ADR-016 but did not align the RocksDB CF key schema with the ADR-008
`{org_id}:` prefix requirement and the ADR-016 retry-state discriminator byte. The CF
keys in BC-2.18.001 §3 read:

```
action_state/{action_id}/retry_count
action_state/{action_id}/dead_letter
```

ADR-016 §5.5 specifies the retry-state row must use the `\x04` discriminator (consistent
with ADR-018 diff_results `\x04` discriminator) and the ADR-008 OrgId prefix:

```
action_state/{org_id}/{action_id}/\x04  (retry-state row)
action_state/{org_id}/{action_id}/\x03  (dead-letter row)
```

BC-2.18.001's CF keys are missing both the `{org_id}:` prefix and the `\x04`/`\x03`
discriminator bytes. Any implementation following BC-2.18.001 §3 as written would
produce cross-org key collisions and would conflict with the action_state CF layout
specified in ADR-016.

**Required Fix:** BC-2.18.001 §3 CF key schema must be updated to include `{org_id}/`
prefix and `\x04`/`\x03` discriminator bytes per ADR-016 §5.5. Also add OrgId to the
precondition list (action_id alone is not globally unique without org scoping).

**Fix Applied (2026-05-03):** BC-2.18.001 v1.4→v1.5: §3 CF keys updated to include
`{org_id}/` prefix and `\x04` (retry-state) / `\x03` (dead-letter) discriminators per
ADR-016 §5.5. OrgId added to precondition. Regression closed.

---

#### P8-S-4.08-A-H-003 — 1s Cron Tick vs 60s Default Regression

**Severity:** HIGH
**Location:** `stories/S-4.08-action-delivery.md` §4 (scheduler integration) + `specs/architecture/decisions/ADR-013-schedule-execution-semantics.md` §3.2
**Finding:** ADR-013 §3.2 specifies the schedule executor tick interval is 60 seconds by
default (configurable via `[operations] tick_interval_secs`). S-4.08 §4 (scheduler
integration task) contains a code snippet showing `tokio::time::interval(Duration::from_secs(1))`
— a 1-second tick. This contradicts ADR-013 §3.2 and would cause 60× more RocksDB reads
than intended, violating the memory budget constraint (DF-MEM-001: 512MB process budget).

Pass 6 fixed BC-2.12.004 to reference the 60s tick and 8-permit semaphore. Pass 7 fixed
BC-2.12.004 again for the EC-12-010 tick note. Neither pass swept S-4.08's scheduler
integration task which retained the stale 1s example. This is the third consecutive pass
where a sibling-spec fix is applied without sweeping the story that owns the downstream
implementation detail.

**Required Fix:** S-4.08 §4 scheduler integration snippet must be updated to use
`Duration::from_secs(config.tick_interval_secs.unwrap_or(60))` matching ADR-013 §3.2.
AC-10 (if present) or a new AC must reference the 60s default with configurable override.

**Fix Applied (2026-05-03):** S-4.08 v1.16: §4 scheduler snippet updated to
`Duration::from_secs(config.tick_interval_secs.unwrap_or(60))` per ADR-013 §3.2. 1s
hardcode removed. Regression closed.

---

### MEDIUM Findings

#### P8-ADR-016-A-M-004 — §5.5 120s Tick Typo

**Severity:** MEDIUM
**Location:** `specs/architecture/decisions/ADR-016-action-delivery-framework.md` §5.5
**Finding:** ADR-016 §5.5 (retry loop timing) contains the sentence: "The retry scanner
runs on a 120s tick to avoid contention with the action dispatch loop." ADR-013 §3.2 and
BC-2.12.004 both specify 60s as the canonical tick interval. The 120s value in ADR-016
§5.5 is either a typo (should be 60s) or an intentional distinction (retry scanner uses
a separate 2× interval). If intentional, ADR-016 §5.5 must include an explicit rationale
distinguishing the retry scanner tick from the action dispatch tick. If a typo, it must
be corrected to 60s.

**Required Fix:** ADR-016 §5.5 must either (a) correct 120s → 60s, or (b) add a
rationale sentence explaining the 2× interval separation between the retry scanner tick
and the dispatch tick.

**Fix Applied (2026-05-03):** ADR-016 v0.4→v0.5: §5.5 amended — 120s corrected to 60s;
explicit note added that retry scanner runs on the same 60s tick as the dispatch loop
(not a separate interval), sharing the single `tokio::time::interval`. Regression closed.

---

#### P8-ADR-013-A-M-005 — Cron Version Citation Drift

**Severity:** MEDIUM
**Location:** `specs/architecture/decisions/ADR-013-schedule-execution-semantics.md` §2.1 (Dependencies)
**Finding:** ADR-013 §2.1 cites `croner = "2.0"` as the dependency for cron expression
parsing. The research findings (preflight-findings/research-findings.md §R-2) identified
`croner = "2.1"` as the stable version with 6-field expression support needed for
seconds-granularity schedules (ADR-013 §3.3 seconds-field requirement). The `2.0`
citation is stale — it does not support the 6-field form. An implementation following
ADR-013 §2.1 as written would use croner 2.0 which lacks 6-field support, causing
compile errors when the seconds-field cron expressions from §3.3 are exercised.

**Required Fix:** ADR-013 §2.1 must be updated to cite `croner = "2.1"` (or
`croner = "2"` with a `>=2.1` note) to match the research-confirmed version with 6-field
support.

**Fix Applied (2026-05-03):** ADR-013 v0.4→v0.5: §2.1 dependency updated from
`croner = "2.0"` to `croner = "2.1"` matching research-findings R-2. Regression closed.

---

### LOW Findings

#### P8-VCM-A-L-006 — Verification-Coverage-Matrix Comment Trail Incomplete

**Severity:** LOW
**Location:** `specs/architecture/verification-coverage-matrix.md` (VP-044 through VP-047 rows)
**Finding:** Pass 7 remediation added a comment trail to the VP totals section but did
not update the audit trail comments on the VP-044–VP-047 rows (S-4.08 VPs). These rows
were added in the Wave 4 Phase 3 ADR burst (v1.25) but their per-row comments still
reference the stale v1.25 burst tag. Pass 5 updated VP-137 and VP-144 comment rows to
v1.26/v1.27. VP-044–VP-047 were not swept in that pass. The result is an inconsistent
comment trail: 4 of the 6 S-4.08 VPs have stale burst tags.

**Required Fix:** VP-044, VP-045, VP-046, VP-047 comment rows in the
verification-coverage-matrix must be updated to reference the current burst tag (v1.28+).

**Fix Applied (2026-05-03):** verification-coverage-matrix v1.28→v1.29: VP-044–VP-047
comment rows updated to reference v1.29 burst tag. Audit trail now consistent across all
6 S-4.08 VPs. Regression closed.

---

## Remediation Summary

All 6 findings remediated 2026-05-03.

| ID | Severity | File | Change |
|----|----------|------|--------|
| P8-S-4.08-A-H-001 | HIGH | S-4.08 | v1.15→v1.16: AC-6 SMTP auth order XOAUTH2→PLAIN→E-AD-018 |
| P8-BC-2.18.001-A-H-002 | HIGH | BC-2.18.001 | v1.4→v1.5: CF keys +OrgId prefix +\x04/\x03 discriminators |
| P8-S-4.08-A-H-003 | HIGH | S-4.08 | v1.16: §4 tick 1s→60s default per ADR-013 §3.2 |
| P8-ADR-016-A-M-004 | MEDIUM | ADR-016 | v0.4→v0.5: §5.5 120s→60s; retry scanner same tick as dispatch |
| P8-ADR-013-A-M-005 | MEDIUM | ADR-013 | v0.4→v0.5: §2.1 croner 2.0→2.1 per research R-2 |
| P8-VCM-A-L-006 | LOW | verification-coverage-matrix | v1.28→v1.29: VP-044–VP-047 comment trail synced |

**Stage 1 SHA:** 246b9f71 (pre-existing — remediation was authored in Pass 7 burst;
Pass 8 remediation committed in subsequent Stage 1 commit per canonical SHA protocol)

**Next Step:** Adversarial Pass 9 — re-run on all Pass 8 remediated specs. Target: CLEAN
to open convergence window 1/3.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 8 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 2 |
| **Novelty score** | 0.67 (4 / 6) |
| **Median severity** | HIGH (3H dominant; 2M/1L) |
| **Trajectory** | 38→17→8→7→7→5→5→6 |
| **Verdict** | FINDINGS_REMAIN (slight uptick — partial-fix regression pattern; sibling-fix sweep gap) |
