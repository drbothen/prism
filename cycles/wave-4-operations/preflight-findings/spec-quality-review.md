---
document_type: preflight-findings
phase: 4.A
producer: spec-reviewer
timestamp: 2026-05-02T18:30:00Z
inputs:
  - .factory/stories/S-4.01-schedule-crud.md
  - .factory/stories/S-4.02-diff-results-packs.md
  - .factory/stories/S-4.03-detection-rules.md
  - .factory/stories/S-4.04-detection-evaluation.md
  - .factory/stories/S-4.05-alert-generation.md
  - .factory/stories/S-4.06-case-management.md
  - .factory/stories/S-4.07-case-metrics.md
  - .factory/stories/S-4.08-action-delivery.md
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/architect-adr-identification.md
  - .factory/stories/S-3.3.04-harness-network-isolation.md
  - .factory/stories/STORY-INDEX.md
verdict: APPROVED_WITH_CONDITIONS
total_findings: 47
severity_breakdown: { HIGH: 6, MEDIUM: 21, LOW: 12, KUDO: 8 }
---

# Wave 4 Spec Quality Review

## Summary

- **Stories reviewed:** 8 (S-4.01 through S-4.08)
- **Verdict:** APPROVED_WITH_CONDITIONS — implementable as-written but with quality drift in
  AC measurability, risk-concentration patterns, and weak Previous Story Intelligence in
  the entry-cohort stories (S-4.01, S-4.03). Six HIGH findings should be fixed before
  dispatch; the remainder can be remediated during Phase 4.B story-update bursts.
- **Top quality concerns:**
  - S-4.06 has a documented frontmatter↔body↔manifest sizing inconsistency (manifest=9pts,
    frontmatter=5pts, scope=11 tasks + 4 VPs). One source of truth must be chosen.
  - S-4.08 is a 5-pt story with 16 tasks, 4 VPs, 7 depends_on edges, 4 destination types,
    and 18 files to create. Severely under-pointed; high churn risk on first delivery.
  - Cross-cutting pattern: ACs that say "within N seconds" or "within Xms" are routinely
    asserted as binary tests but lack specified measurement methodology (what clock, hot
    vs cold, CI vs local). S-3.3.04 set the baseline ("on a CI runner") — Wave 4 stories
    drift away from that precision.
  - S-4.06 introduces a 5-state, 12-transition state machine with reopen semantics in a
    single story. ADR-017 (per architect findings) is the right home for the table; the
    story should reference the ADR rather than re-encode the table inline.
  - S-4.05 is sized at 2 points but pulls in: TemplateInterpolator + 4-level resolution +
    InjectionScanner integration + UUID v7 ID generation + RocksDB persistence + rate
    limiting under Mutex + broadcast channel + fuzz target. 2pt is implausible.
  - S-4.01 + S-4.08 share a 16-permit semaphore with no story owning the construction
    contract; this is a join-point latent defect (architect raised separately as ADR-013
    open question, but the stories themselves don't ack the shared-resource lifecycle).
- **Strengths:**
  - S-4.06 task 9a/9b (pure decide_dedup_action + effectful wrapper split) is exemplary —
    propagate this pattern to S-4.04 dedup and S-4.05 rate_limit.
  - S-4.04 EC table is comprehensive (8 entries spanning concurrency, panics, eviction,
    cross-mode collisions). Strongest of the 8 stories.
  - S-4.03 IOC pattern store design (BC-2.13.014, ArcSwap hot reload, RegexSet aggregation)
    is well-specified and proves the value of expanding stub BCs into concrete deliverables.
  - S-4.02's "exactly-once epoch" semantics with merge-operator atomicity is precise and
    falsifiable.
  - All 8 stories have Token Budget Estimate tables — consistent and credible (one
    exception flagged below).

---

## Per-Story Quality Findings

### S-4.01 Schedule CRUD and Execution Loop

**Quality verdict:** APPROVED_WITH_CONDITIONS
**Sizing assessment:** 5 points justified — at the upper edge but defensible
(state types + CRUD + executor + persistence + 2 Kani proofs = 5 task buckets).

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-401-001 | HIGH | A measurability | AC-4 says "execution starts at T+5m+30s" with no tolerance. Wall-clock scheduled execution under 60s tick granularity cannot be asserted as exact T+5m+30s — the tick could fire at T+5m+0s through T+5m+59s plus splay. AC will fail intermittently. | Restate as: "execution fires within the [next_run_at + splay_offset, next_run_at + splay_offset + tick_interval] window, and the recorded `last_run_at` falls within this window." |
| QUAL-401-002 | MEDIUM | K risk concentration | S-4.01 owns the 16-permit `Arc<Semaphore>` shared with S-4.08, but the story does not specify (a) where the Semaphore is constructed, (b) lifetime/Arc ownership, (c) which crate exports it, (d) startup ordering when S-4.08 needs it before its own subsystem comes up. Deferred to ADR-013 by architect, but the STORY itself does not flag this dependency. | Add explicit task: "Export `pub fn schedule_semaphore() -> Arc<Semaphore>` (or equivalent) so S-4.08's `ActionEngine::new()` can inject the same instance. Document Arc ownership: lib.rs constructs it, hands to Scheduler and ActionEngine via DI." |
| QUAL-401-003 | MEDIUM | E edge case sufficiency | EC table covers cap, in-flight skip, restart recovery, wildcards — but missing: (a) clock skew across restart (next_run_at < now() by hours due to suspended laptop) — does the executor fire once or skip until current time? (b) RocksDB write-fail mid-execution. | Add EC-12-009: "Resume after long suspension where many schedules' `next_run_at` are in the past — fire each at most once at startup, do NOT play catch-up burst." Add EC-12-010: RocksDB write failure on `save_schedule_state`. |
| QUAL-401-004 | MEDIUM | A measurability | AC-5 ("the execution is skipped (not queued) and a skip event is logged") is testable only by observing log output. Brittle. | Add structured-event assertion: "the skip MUST emit a `schedule_skipped` event with fields `{schedule_id, reason: 'in_flight' \| 'semaphore_exhausted'}`." Tests assert on the event, not log lines. |
| QUAL-401-005 | LOW | J previous story intelligence | "Previous Story Intelligence" says "N/A — first story in `prism-operations`" but Wave 3 introduced multiple cross-cutting patterns directly relevant: (a) S-3.1.x OrgId/OrgSlug newtype patterns; (b) S-3.3.x harness DI/Arc-sharing pattern (relevant for the semaphore); (c) S-2.05 audit emitter pattern (reference for skip-event emission); (d) S-3.3.01 customer config TOML schema (for `PRISM_MAX_SCHEDULES`). | Replace "N/A" with a 4-row table citing those four wave-3 stories for pattern guidance. |
| QUAL-401-006 | LOW | C BC anchor semantic fit | `anchor_capabilities: [CAP-017]` is BC-INDEX-derived, but BC-2.12.010 (state persistence) is arguably CAP-018 territory (storage). Fine to keep CAP-017 since BC-INDEX is authoritative, but anchor_capabilities could include CAP-018 for completeness without breaking the BC-INDEX rule. | Optional — append `CAP-018` if BC-INDEX permits dual capability anchoring for storage-touching stories. |

**Strengths:**
- Architecture Compliance Rules section is concrete and falsifiable (try_acquire MUST,
  splay MUST be pure, etc.).
- 8 EC entries with explicit expected behavior — solid baseline.
- VP-026 / VP-030 are right-sized for Kani.

---

### S-4.02 Differential Results and Packs

**Quality verdict:** APPROVED
**Sizing assessment:** 3 points justified.

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-402-001 | MEDIUM | E edge case sufficiency | EC-005 "evicted epoch range" handled, but missing: (a) what happens when prev hash set is corrupted (deserialization fails)? (b) what happens when the 200MB cap is reached mid-write (RocksDB block cache eviction during `set_block_cache`)? Critical for diff-engine correctness. | Add EC-009: corrupted prev snapshot → treat as first-run (diff = added=all, removed=[]); WARN log emitted. Add EC-010: 200MB cap reached → oldest schedule's diff state evicted first; downstream get_diff_results returns E-SCHED-003 for evicted ranges. |
| QUAL-402-002 | MEDIUM | A measurability | AC-2 ("output is identical to AC-1") is structural prose. The proptest VP-019 covers the property, but the AC itself doesn't anchor on a deterministic comparison. | Specify: "DiffResult.added and DiffResult.removed contain byte-identical Vec<RowHash> elements in identical order to the AC-1 output." |
| QUAL-402-003 | MEDIUM | M TV coverage | BC-2.12.005 likely has canonical test vectors (TV-1..N) for hash determinism, but Tasks 1-2 don't reference them. | Confirm BC-2.12.005's TV table; if present, Task 1 should cite "implements TV-1..N" inline. |
| QUAL-402-004 | LOW | F narrative crispness | Narrative opens "As a Prism operations engine, I want..." — this is the system speaking, not a user. Wave 3 baseline (S-3.3.04) uses "As a Prism integration-test author" — concrete role. | Optional reframe: "As a Prism analyst running recurring queries, I want differential results so I see only what changed between runs." |
| QUAL-402-005 | LOW | J previous story intelligence | Cites S-4.01 well, but does NOT note that S-4.02's output (added rows) is the input to S-4.04 (detection eval) — this is a cross-wave coupling worth mentioning so implementers don't accidentally change the DiffResult shape and break downstream. | Add row: "S-4.04 consumes `DiffResult.added` — schema is a contract; do not mutate without coordinated update." |

**Strengths:**
- Token budget is conservative and accurate.
- Architecture Compliance Rules call out merge-operator atomicity vs read-then-write
  TOCTOU — high-quality detail.
- "Pack delete: no confirmation required (config not data)" + contrast with S-4.01
  schedule-delete confirmation is explicit and well-reasoned.

---

### S-4.03 Detection Rule Loading and Compilation

**Quality verdict:** APPROVED_WITH_CONDITIONS
**Sizing assessment:** 5 points UNDER-SIZED. Story has 9 tasks (one of which — Task 8a IOC
expansion — is itself a 3-pt sub-story) + 8 BCs + 9 ACs + 9 EC entries + 13 files to
create. Real footprint is 7-8 pts.

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-403-001 | HIGH | D sizing | Task 8a (IOC file loading and pattern store) is a complete subsystem: file format definition, regex compilation, RegexSet aggregation, ArcSwap hot reload, S-1.12 file watcher integration, 5 error codes (E-IOC-001..004 + size limits), and a UDF. This alone is 3 points. The base story (TOML loader + 3 UDFs + scope resolver + compiler + 1000-rule cap + VP-018 proptest) is 5 points. Combined = 8 pts. | Either (a) split into S-4.03 base (5pt) + S-4.03b IOC store (3pt), OR (b) re-point S-4.03 to 8. Recommend (a) — S-4.03b unblocks S-4.04 detection eval testing without IOC store dependency. |
| QUAL-403-002 | MEDIUM | A measurability | AC-9 (IOC loading) is a multi-clause AC bundling 6 distinct assertions (compile each file, INFO log, E-IOC-001/002/003/004 rejection, prior state retention, no crash). Compound ACs are hard to attribute when one clause fails. | Decompose AC-9 into AC-9a (successful load+log), AC-9b (E-IOC-001 invalid regex), AC-9c (E-IOC-002 oversize), AC-9d (E-IOC-003 over-pattern-count), AC-9e (E-IOC-004 file count cap), AC-9f (no-crash invariant). |
| QUAL-403-003 | MEDIUM | E edge case sufficiency | EC table covers IOC failures and rule cap, but missing: (a) `.detect` file with valid TOML but unknown `condition.type` (not in {single, correlation, sequence}); (b) `create_rule` race — two clients send concurrent create requests pushing count from 999 → 1001; (c) empty `analyst_id` on Analyst-scoped rule. | Add EC-010 unknown condition type → RuleValidationError. Add EC-011 concurrent-create race at cap → atomic check ensures only one succeeds. Add EC-012 missing analyst_id on Analyst scope → validation rejection. |
| QUAL-403-004 | MEDIUM | C BC anchor semantic fit | `anchor_bcs` lists 8 BCs but BC-2.13.014 (IOC) is only weakly anchored to capabilities CAP-020 (rule loading) and CAP-027 (compilation). IOC pattern matching may warrant a third capability anchor. | Confirm with BC-INDEX whether BC-2.13.014 has a distinct CAP. If yes, add to `anchor_capabilities`. |
| QUAL-403-005 | MEDIUM | L AC count vs scope | 9 ACs across 9 tasks ratio is reasonable, but AC-1 through AC-8 are all ~1-line crisp; AC-9 is a 9-line monolith. Imbalanced — AC-9 alone is the size of AC-1..6 combined. | See QUAL-403-002 — decomposition fixes both findings. |
| QUAL-403-006 | LOW | J previous story intelligence | Says "first story in detection subsystem" — true, but pattern guidance from S-3.x is missing. S-3.3.01 (customer-config TOML), S-3.7.01 (config-driven generator) both establish TOML+validation patterns directly applicable to `.detect` files. | Add 2 rows referencing those wave-3 stories for TOML loader patterns. |

**Strengths:**
- Three-scope rule resolution (Global + Client + Analyst) is correctly specified as
  union, not override — explicit prevention of common detection-engine bug.
- IOC pattern store design (RegexSet + ArcSwap) — exemplary and reusable. KUDO.
- Task 9 atomic-cap check + dedicated E-RULE-011 error code is concrete.

---

### S-4.04 Detection Evaluation (Single/Correlation/Sequence)

**Quality verdict:** APPROVED
**Sizing assessment:** 5 points justified — right at the boundary. State persistence,
3 evaluation modes, dedup, and a proptest is genuinely 5 pts of work.

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-404-001 | MEDIUM | A measurability | AC-3 ("an alert fires and the counter resets to 0") — counter reset is a side-effect on a private struct; testability requires either exposing an inspector method or asserting on subsequent behavior. | Add AC-3a: "after the firing alert, the 6th matching record arriving in the same window does NOT fire (counter at 1, not at 6)" — observable assertion. |
| QUAL-404-002 | MEDIUM | E edge case sufficiency | EC-006 covers SessionContext panic, but missing: (a) sequence rule's partial-match tracker exceeding state size (e.g., 1M unique entity_ids without completion) — bounded? (b) clock-skew on `event_time` (record's event_time is in future relative to now) — included in window or excluded? | Add EC-009: per-rule sequence tracker cap (suggest 50k entities, similar to correlation 10k group keys); over-cap → silently drop with WARN. Add EC-010: future-dated event_time → included in window; rationale: replay tolerance. |
| QUAL-404-003 | MEDIUM | K risk concentration | S-4.04 dedup module shares `\x02` keys in `detection_state` CF with S-4.05 rate_limit (`\x01`) and S-4.04 group state (`\x00`). Three writers to one CF, three implicit lock domains. The story says "rate limit counter writes MUST be under Mutex" but doesn't specify lock granularity (per-CF, per-rule, per-key). | Add Architecture Compliance Rule: "The Mutex protecting `detection_state` writes is per-rule (`Mutex<()>` keyed by RuleId), NOT a single global Mutex. Per-CF lock would serialize all detection evaluation across all rules." |
| QUAL-404-004 | MEDIUM | C BC anchor semantic fit | BC-2.13.013 (alert dedup) covers Single/Correlation/Sequence dedup keys. But `dedup_window` defaulting to "interval of parent schedule, or 1 hour" is a runtime resolution that the story specifies but BC-2.13.013 may not. | Confirm BC-2.13.013 explicitly captures the fallback rule. If not, this should be a remediation note for the BC owner during Phase 4.A. |
| QUAL-404-005 | LOW | M TV coverage | Tasks reference VP-027 proptest but do not enumerate canonical test vectors for the three dedup-key modes. BC-2.13.013 likely has TVs. | If BC-2.13.013 has TV-1..N, cite them in Task 6 dedup-mode rows. |

**Strengths:**
- "Removed records are NOT evaluated by the detection engine" — explicit, prevents a
  whole category of misuses. KUDO.
- VP-027 proptest covering cross-mode key collisions is the right verification level.
- Architecture Compliance Rule "event_time anchoring not Instant::now()" is precise.
- 8-row EC table — second-most-comprehensive after S-3.3.04.

---

### S-4.05 Alert Generation

**Quality verdict:** REQUEST_CHANGES
**Sizing assessment:** 2 points SEVERELY UNDER-SIZED. 5 modules to author + 7 ACs +
8 ECs + a fuzz target + Mutex-guarded RocksDB write + InjectionScanner integration +
broadcast channel + UUID v7 generator + 64KB snapshot truncation logic. Comparable
S-3.x work (e.g. S-3.1.04 prism-credentials migration = 3pt for less surface area).
Realistic = 4-5 points.

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-405-001 | HIGH | D sizing | 2pt is a major mismatch. The cycle-manifest table even lists S-4.05 as 1pt and 2 BCs (frontmatter says 1 BC — BC-2.13.005). Manifest disagrees with frontmatter. Both disagree with actual scope. | Re-point to 4. Update cycle-manifest accordingly. (Note: manifest already says 1pt; this is a documented inconsistency.) |
| QUAL-405-002 | HIGH | A measurability | AC-1 hardcodes the rendered string `"Alert: High Severity Detection on server-01"` but the rule.name in the test fixture is not specified to be `"High Severity Detection"`. AC will only pass if the test author happens to pick that exact rule name. | Either parameterize: "the alert title is `"Alert: <rule.name> on <hostname>"` for the test fixture's `rule.name` and `hostname` values" — or specify the fixture explicitly. |
| QUAL-405-003 | MEDIUM | A measurability | AC-7 says "VP-028 fuzz target runs 30 minutes without any panics." 30min is a CI-budget claim. Production fuzz runs are 24h+. What does "30 min on CI" actually prove? | Reframe: "VP-028 fuzz target runs at least 30 minutes on CI without panics; documented in TD register if longer-soak fuzz is deferred." |
| QUAL-405-004 | MEDIUM | E edge case sufficiency | EC-008 ("event_time null → triggered_at fallback") is good, but missing: (a) UUID v7 collision (statistically improbable but observable in 64-bit rand suffix); (b) broadcast channel subscriber that panics — does the broadcast survive? (c) snapshot truncation when ALL fields are large (no low-priority fields to truncate first). | Add EC-009 broadcast subscriber panic → other subscribers continue receiving. Add EC-010 all-large-fields snapshot → truncate all fields proportionally with `snapshot.truncated = true` + `truncation_reason = "all_fields_large"`. |
| QUAL-405-005 | MEDIUM | C BC anchor semantic fit | Single BC anchor (BC-2.13.005) for a story that touches: alert template rendering, snapshot truncation, rate limiting, broadcast, persistence, injection scanning. BC-2.13.005 may bundle all these, but the story would benefit from sub-clause anchoring (postcondition X for AC-Y). | Audit BC-2.13.005 for postcondition/invariant numbering; align ACs to specific clauses (e.g., "AC-4 traces to BC-2.13.005 postcondition 7 / EC-rate-limit"). |
| QUAL-405-006 | MEDIUM | I TDD mode | Frontmatter does not have `tdd_mode: strict`. P0 stories should default to strict per Wave 3 baseline (S-3.3.04 has it). | Add `tdd_mode: strict` to frontmatter. Apply to all 8 Wave 4 stories — none currently have this field. |
| QUAL-405-007 | LOW | F narrative crispness | "I want to generate rich alerts from detection rule matches with interpolated templates, inline event snapshots, and per-rule rate limiting" — narrative bundles 4 concerns. Wave 3 baseline narratives are tighter (one capability per sentence). | Tighten: "As a Prism case manager, I want detection alerts to arrive complete and self-contained — rendered title, inline event snapshot, severity — so I never need to re-query sensor data to triage." |

**Strengths:**
- "Flag don't strip" pattern (BC-2.09.004 reference + AC-5 + Architecture Compliance) —
  explicit and correct. KUDO.
- VP-028 fuzz target + libfuzzer-sys is the right tool for "never panics" claim.
- EC-008 (event_time null fallback with `ttd_approx = true` flag) — good downstream
  metrics-correctness signal.

---

### S-4.06 Case Management

**Quality verdict:** REQUEST_CHANGES (sizing) / APPROVED_WITH_CONDITIONS (quality)
**Sizing assessment:** Frontmatter says 5pt; cycle-manifest says 9pt; actual scope
(11 tasks, 4 VPs, 9 BCs, 12 ACs, 12 ECs, 13 files) is closer to 8-9 pts. The
frontmatter↔manifest disagreement is itself a finding.

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-406-001 | HIGH | D sizing | Frontmatter `points: 5` contradicts cycle-manifest `Pts: 9`. Both must be reconciled before dispatch. Real scope is 9pt (4 VPs alone is high formal-verification load). | Update frontmatter to `points: 9` to match manifest, OR split into S-4.06a (CRUD + state machine, 5pt) + S-4.06b (auto-case-creation + dedup + VPs, 4pt). The split is natural — auto-case-creation depends on the broadcast channel from S-4.05 and is conceptually distinct from manual case CRUD. |
| QUAL-406-002 | HIGH | C BC anchor semantic fit | The story re-encodes the 5-state, 12-transition table inline (Task 1 lines 113-117) AND lists transitions in EC-005, EC-006, AC-4. This duplicates ADR-017 (per architect findings) and risks drift. | Move the canonical transition table to ADR-017; story references "per ADR-017 §X.Y transition table." Remove inline transition listing in Task 1; replace with "use `prism-core::CaseStatus::can_transition_to()` per ADR-017." |
| QUAL-406-003 | MEDIUM | A measurability | AC-12 ("dedup window active — no second case created") and AC-12b (pure function VP-060) are well-traced, but AC-11 says title is `"AUTO: {rule_name} — {client_id}"` truncated to 200 chars. Truncation rule is one-sided (truncate from end? from middle? preserve client_id?). | Specify truncation rule: "if `len(title) > 200`, truncate from the end after `{rule_name}` substring; ALWAYS preserve `AUTO: ` prefix and `— {client_id}` suffix." |
| QUAL-406-004 | MEDIUM | E edge case sufficiency | 12 ECs are excellent, but missing: (a) `update_case` with transition AND disposition AND annotation simultaneously — apply order? (b) two analysts concurrently update same case — last-write-wins or optimistic-lock? (c) timeline entry for analyst whose AnalystId was revoked between case creation and timeline append. | Add EC-013: combined transition+disposition+annotation update — apply order is disposition → transition → annotation (VP-052 enforces); single RocksDB write. Add EC-014: concurrent updates → last-write-wins on the cases CF (RocksDB single put), but VP-052 ordering preserved per-write. Add EC-015: revoked analyst → still recorded in timeline (audit trail must not lie); revocation is forward-only. |
| QUAL-406-005 | MEDIUM | M TV coverage | 4 VPs (VP-052, VP-053, VP-054, VP-060) each have proptest/Kani files. But the BC TV references aren't enumerated in tasks. BC-2.14.002 likely has TVs for the 12-transition table. | Confirm BC-2.14.002 TVs exist; cite in Task 1. |
| QUAL-406-006 | MEDIUM | I TDD mode | Missing `tdd_mode: strict` (same as S-4.05 finding). With 4 VPs and a state machine, strict TDD is essential. | Add `tdd_mode: strict`. |
| QUAL-406-007 | LOW | J previous story intelligence | Says "S-1.02 defines CaseStatus and can_transition_to()" but per architect open question, S-1.02 may be a stub. Story doesn't acknowledge the risk that the function isn't fully implemented in the merged codebase. | Add explicit task 0: "verify `prism-core::CaseStatus::can_transition_to()` encodes all 12 transitions; if S-1.02 left it stubbed, this story extends it (in scope)." |
| QUAL-406-008 | LOW | F narrative crispness | "I want a full case management subsystem with create, transition, annotate, list, and retrieve operations" enumerates 5 capabilities. Wave 3 baseline pattern: one capability per narrative. | Trim to: "I want to organize related alerts into investigation cases with a structured state machine, so my team can track investigation progress and disposition without losing the audit trail." |

**Strengths:**
- Task 9a/9b pure-decision + effectful-wrapper split is **the model pattern** for
  Wave 4. KUDO. Propagate to S-4.04 dedup and S-4.05 rate_limit.
- 4 VPs (VP-052..060) with method assignment (proptest vs Kani) — correct verification
  granularity per property.
- EC-006 reopen semantics ("`closed_at` cleared; `resolved_at` preserved") — exact and
  testable. KUDO.
- TimelineEntry's structured `from_status: Option<CaseStatus>` field (line 122-128 of the
  story) so MTTR computation in S-4.07 doesn't need regex parsing — design foresight. KUDO.

---

### S-4.07 Case Metrics and Acknowledge Alert

**Quality verdict:** APPROVED
**Sizing assessment:** 3 points justified.

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-407-001 | MEDIUM | E edge case sufficiency | EC-006 mentions t-digest/GK summary for >10K cases but no AC enforces this — implementer could load all into memory and pass tests for sub-10K case sets, breaking when production loads cross that threshold silently. | Add AC-10: "Given 50,000 cases in the cases CF, when `case_metrics` is called, the response is produced within 5s and process RSS does not exceed +200MB during the call." (Streaming-percentile becomes mandatory to pass.) |
| QUAL-407-002 | MEDIUM | B AC↔BC traceability | AC-2 cites BC-2.14.008 ("evicted alert record fallback") but the story says alerts CF "must already be completed and reviewed (prerequisite gate)." Eviction policy on alerts CF is not defined in this story. | Either (a) cross-reference S-4.05's alerts-CF retention policy in Previous Story Intelligence, or (b) drop AC-2 and document as a future story when alerts retention is specified. |
| QUAL-407-003 | MEDIUM | I TDD mode | Missing `tdd_mode: strict`. | Add `tdd_mode: strict`. |
| QUAL-407-004 | LOW | L AC count vs scope | 9 ACs for 3 BCs and 6 tasks is well-balanced; sound. — | (No action.) |
| QUAL-407-005 | LOW | F narrative crispness | Narrative is good but "MSSP analysts and clients" mixes two audiences. Pick one. | "As a Prism MSSP analyst, I want auto-computed MTTD/MTTR/MTTI metrics, so I can demonstrate detection and response performance to my clients without manual aggregation." |

**Strengths:**
- BC-2.14.012 prerequisite-gate language ("BC must already be completed and reviewed") —
  defensive and correct.
- Capability-flag gating on `acknowledge_alert` (CAPABILITY_ACKNOWLEDGE_ALERT) — explicit
  about audit-sensitivity.
- AC-2 fallback semantics (`triggered_at` + `mttd_approx = true`) propagate the
  S-4.05 design for downstream metrics correctness.

---

### S-4.08 Action Delivery Framework

**Quality verdict:** REQUEST_CHANGES
**Sizing assessment:** 5 points (frontmatter) vs 9 points (manifest) — manifest correct,
frontmatter UNDER-SIZED. Real scope is 9-13 points and may warrant a split.

| ID | Severity | Dimension | Finding | Suggestion |
|----|----------|-----------|---------|------------|
| QUAL-408-001 | HIGH | D sizing | Frontmatter `points: 5` vs cycle-manifest `Pts: 9`. Story has 16 tasks, 4 destination types, 4 VPs (1 Kani + 3 proptests), cron loop, hot reload, retry+dead-letter, template rendering, rate limiting (3 controls), 7 depends_on edges, 18 files to create + 1 fixture. By Wave 3 sizing baselines (S-3.3.03 = 13pt for the harness with logical mode), this is comfortably 9-13pt. | Re-point to 9 (matching manifest). Strongly consider splitting into S-4.08a (action-engine core: spec loader, trigger eval, webhook destination, retry, rate limit — 5pt) + S-4.08b (multi-destination: email/syslog/plugin + cron + report-render — 5pt). 13 file-creates is a context-window risk for a single subagent dispatch. |
| QUAL-408-002 | HIGH | K risk concentration | S-4.08 has 7 `depends_on` edges (S-4.05, S-4.06, S-4.01, S-1.15, S-6.11, S-6.12, S-6.13). If ANY upstream slips, S-4.08 cannot dispatch. Topology has S-4.08 as the terminal join node — single point of failure for entire wave delivery. | Decouple where possible. The S-6.11/12/13 (Slack/PagerDuty/Jira DTU) deps are TEST-FIXTURE deps, not BUILD deps. Mark them as `test_depends_on` not `depends_on`. Refactor frontmatter to distinguish: `depends_on` = compile-time, `test_depends_on` = integration-test-only. |
| QUAL-408-003 | MEDIUM | A measurability | AC-1 says webhook POST happens "within 2 seconds" — clock origin? From alert broadcast emit, from `ActionEngine::deliver()` entry, from rule fire? CI vs local? | "Within 2 seconds of the alert appearing on the broadcast channel, measured from `Instant::now()` at `broadcast::Receiver::recv()` returning, until the first byte sent on the TCP stream — measured on a CI runner in the standard test harness." |
| QUAL-408-004 | MEDIUM | A measurability | AC-3 hardcodes the exponential-backoff schedule "2s, 4s, 8s, 30s, 60s." Test would need to wait 104+ seconds to validate — too slow for CI. | Add: "VP-044 Kani proof verifies the state machine bounds; integration test uses fault-injection to exercise the retry path with a 10x speedup factor (200ms, 400ms, 800ms, 3s, 6s) controlled by `cfg(test)` constant." |
| QUAL-408-005 | MEDIUM | E edge case sufficiency | 11 ECs cover most failure paths. Missing: (a) `.action.toml` syntactically invalid TOML — fail loudly without taking down whole reload? (b) plugin destination's WASM panic mid-delivery — does retry kick in? (c) credential reference resolution failure (env var unset) — different error class than inline-credential rejection. | Add EC-012 invalid TOML → file rejected; OTHER action specs continue loading; ERROR log emitted with file path. Add EC-013 plugin trap mid-delivery → classified as `DeliveryError::Transient`; retry per VP-044. Add EC-014 unresolved credential reference → `E-ACTION-003` (distinct from E-ACTION-001 inline rejection). |
| QUAL-408-006 | MEDIUM | C BC anchor semantic fit | 10 BC anchors (BC-2.18.001..009 + BC-2.09.004) is appropriate, but BC-2.09.004 is "Safety Flags" — a Wave 2.09 contract dragged into a Wave 4 action delivery story. The cross-wave reference is correct (per pass-64-fix changelog), but it muddies the SS-18 anchor. | Add a "Cross-Wave Contract Dependency" subsection to Behavioral Contracts table noting BC-2.09.004 is a Wave 2 reference, not a Wave 4 SS-18 BC. |
| QUAL-408-007 | MEDIUM | G token budget realism | Token budget says ~24,200 tokens "within 30% context window for a 128k-context agent" (= ~38k budget). 24,200 / 38k = 64% — NOT within 30%. Math doesn't add up. | Recompute: 30% of 128k = 38.4k; 24.2k / 128k = 18.9% (within 30%) but 24.2k / 38.4k = 63%. The "30% budget" reference is unclear — clarify whether it's 30% of total context or 30% of 38k. |
| QUAL-408-008 | MEDIUM | I TDD mode | Missing `tdd_mode: strict`. With 4 VPs + a retry state machine, strict TDD is non-negotiable. | Add `tdd_mode: strict`. |
| QUAL-408-009 | LOW | L AC count vs scope | 15 ACs for 16 tasks and 4 VPs — proportional and good. But AC-14/15/16 (VP proofs pass) are formulaic and could be one consolidated AC: "All Wave 4 VPs (VP-044, 045, 046, 047) pass under their declared methods." | Consolidate VP-passing ACs into a single multi-clause AC. |

**Strengths:**
- BC-2.18.007 inline-credential-rejection (E-ACTION-001) is a security-positive
  default — KUDO. Mirrors S-3.3.01 customer-config TOML pattern.
- VP-047 UUID-v7 validation for `${case.alert_ids_quoted}` is injection-defense at
  template-render time — correct layer.
- "flag don't strip" (AC-8 / BC-2.09.004 trace) — preserves data, surfaces signal.
- Manual triggers' fire-and-forget contract (AC-11) is explicit about non-retry,
  non-dead-letter — important for AI caller semantics.
- 4 VPs cover state machine, semaphore, credentials, UUID — orthogonal axes. Good
  coverage design.

---

## Cross-Cutting Quality Patterns

| Pattern | Affected Stories | Recommendation |
|---------|------------------|----------------|
| Missing `tdd_mode: strict` in frontmatter | S-4.01, S-4.02, S-4.03, S-4.04, S-4.05, S-4.06, S-4.07, S-4.08 (ALL 8) | All P0 stories should default to strict. Add to all 8 in a single Phase 4.B story-update burst. Wave 3 baseline (S-3.3.04) has it — Wave 4 regressed. |
| Cycle-manifest ↔ frontmatter point disagreement | S-4.05 (mfst=1, fm=2), S-4.06 (mfst=9, fm=5), S-4.08 (mfst=9, fm=5) | Reconcile to single source of truth. Recommend frontmatter is canonical; update cycle-manifest to match (or vice versa once true points are decided). 3 mismatches in 8 stories is high. |
| AC measurement methodology unspecified ("within Ns") | S-4.01 AC-4, S-4.05 AC-7, S-4.08 AC-1 / AC-3, S-4.07 EC-006 | Adopt S-3.3.04 baseline language: "on a CI runner" + named clock origin + named clock terminus. Include in all timing-sensitive ACs. |
| Pure/effectful split pattern not propagated | S-4.04 dedup (could split decide_dedup vs effectful write), S-4.05 rate_limit (could split decide_rate_limit_action vs Mutex+RocksDB write) | Apply S-4.06 task 9a/9b model. Pure decision functions are proptest-cheap; effectful wrappers are integration-test territory. Promotes verification economy. |
| Compound ACs (multiple assertions in one) | S-4.03 AC-9 (6 clauses), S-4.06 AC-12 (3 clauses), S-4.08 AC-3 (4 clauses) | Decompose to AC-Na, AC-Nb, AC-Nc form for attributability when one clause fails. |
| `Previous Story Intelligence` weak in entry-cohort stories | S-4.01 ("N/A"), S-4.03 ("first story in subsystem"), S-4.05 (only S-4.04 cited), S-4.07 (only S-4.06 cited) | Cite Wave 3 cross-cutting patterns (S-3.1.x OrgId newtype, S-3.3.x Arc-DI, S-3.3.01 TOML, S-2.05 audit, S-1.08 feature flags). Even if no direct dep, the patterns inform implementation. |
| Risk concentration on shared resources not flagged in story | S-4.01 + S-4.08 (16-permit semaphore lifecycle), S-4.04 + S-4.05 (detection_state CF Mutex granularity) | Each story sharing a resource MUST cite the construction site, ownership model, and lock granularity in Architecture Compliance Rules. Architect noted these as ADR-013 / ADR-016 open questions; the stories themselves should also acknowledge. |
| Test-fixture deps mixed with build deps | S-4.08 (`depends_on: [..., S-6.11, S-6.12, S-6.13]`) | Add `test_depends_on:` frontmatter field for test-only dependencies. Keeps `depends_on:` as build-blocking only — improves wave dispatch parallelism. |
| TenantId / OrgSlug residual references | S-4.01 narrative uses "ClientId" (modern), but Tasks say `clients: Vec<ClientId>` while ADR-006 + Wave 3 D-041 standardized on `OrgId`. Architect already flagged this as open question. | Per Wave 3 D-157 (one-wave alias), TenantId→OrgSlug aliases should retire in Wave 4. Story bodies need ClientId→OrgId substitution audit. (Out of spec-quality scope but flagged for spec-drift sibling.) |

---

## Sizing Recommendations

| Story | Frontmatter Pts | Manifest Pts | Recommended Pts | Rationale |
|-------|-----------------|--------------|-----------------|-----------|
| S-4.01 | 5 | 5 | **5** | Aligned. Upper edge but defensible (4 modules + 2 Kani proofs). |
| S-4.02 | 3 | 5 | **3** | Frontmatter is correct; manifest overstates. Diff engine + epoch + pack CRUD + 1 proptest is solidly 3pt. |
| S-4.03 | 5 | 8 | **8** (or split 5+3) | Manifest correct. 9 tasks including IOC subsystem. Recommend split: S-4.03a base (5pt) + S-4.03b IOC store (3pt). |
| S-4.04 | 5 | 5 | **5** | Aligned. Three eval modes + dedup + state CF + proptest — right-sized. |
| S-4.05 | 2 | 1 | **4** | Both wrong. 5 modules + fuzz target + Mutex+RocksDB + InjectionScanner + UUID v7 + 64KB truncation. Compare S-3.x: this is comfortably 4pt. |
| S-4.06 | 5 | 9 | **9** (or split 5+4) | Manifest correct. 11 tasks + 4 VPs + 9 BCs + 12 ECs + 13 files. Recommend split: S-4.06a CRUD+state-machine (5pt) + S-4.06b auto-case-creation+VPs (4pt). |
| S-4.07 | 3 | 3 | **3** | Aligned. |
| S-4.08 | 5 | 9 | **9** (or split 5+5) | Manifest correct. 16 tasks + 4 destinations + 4 VPs + cron + retry + 18 files. Recommend split: S-4.08a engine-core+webhook (5pt) + S-4.08b multi-destination+report+cron (5pt). |

**Total recommended Wave 4 points:** 45 (without splits) / 47 (with the 3 splits =
13 stories instead of 8). Original manifest sum was 45.

---

## Strengths Worth Preserving (KUDO findings)

| ID | Story | Pattern | Why It Matters |
|----|-------|---------|----------------|
| KUDO-W4-001 | S-4.06 | Pure `decide_dedup_action()` + effectful `CaseDedupRegistry::check_and_create()` split | Verification economy: proptest the pure fn cheaply; integration-test the effectful wrapper sparingly. THE model pattern for Wave 4 decision logic. Propagate. |
| KUDO-W4-002 | S-4.04 | "Removed records are NOT evaluated by detection engine" — explicit invariant in Architecture Compliance Rules | Prevents an entire category of detection-engine misuse where deletion events are mistakenly treated as detection events. Crisp negative-space specification. |
| KUDO-W4-003 | S-4.03 | IOC pattern store with `RegexSet` aggregation + `ArcSwap<Arc<PatternStore>>` for hot reload | O(n_patterns) multi-pattern matching with single scan; in-flight UDF calls hold prior Arc safely. Reusable concurrency primitive. |
| KUDO-W4-004 | S-4.06 | Structured `from_status: Option<CaseStatus>` on TimelineEntry instead of regex-parsing the content string | Design foresight — S-4.07's MTTR computation reads structured fields, not regex. Saves entire bug class. |
| KUDO-W4-005 | S-4.08 | BC-2.18.007 inline-credential-rejection (E-ACTION-001) at TOML load time | Security-positive default. Mirrors S-3.3.01 customer-config pattern. Catches operator mistakes before runtime credential leakage. |
| KUDO-W4-006 | S-4.08 | "Flag don't strip" pattern (BC-2.09.004) — InjectionScanner sets `_safety_flags` but original value still interpolated | Preserves audit-trail integrity while warning consumers. Avoids the common "scrub-then-render" anti-pattern that destroys forensic evidence. |
| KUDO-W4-007 | S-4.05 | UUID v7 + lexicographic prefix scan = "no separate index needed for time-ordered listing" — explicitly documented in Dev Notes | Implementation guidance that prevents future maintainer from adding a redundant time-index CF. Storage-efficiency win. |
| KUDO-W4-008 | S-4.02 | `delete_pack` no-confirmation + contrast with `delete_schedule` confirmation, with explicit reasoning ("packs are config, not data") | Decision-rationale documentation. Future maintainer reading both stories sees WHY they differ, not just THAT they differ. |

---

## Bridge to Sibling Findings (Architect + Consistency-Validator)

This spec-quality review surfaces a class of findings the architect's ADR-identification
pass and the consistency-validator's drift audit will NOT catch:

1. **AC measurement methodology gaps (HIGH).** Architect identifies decisions that need
   ADRs; spec-quality identifies that the existing ACs are not falsifiable as written.
   E.g., S-4.01 AC-4's "execution starts at T+5m+30s" is unimplementable as a binary
   test. The drift audit cannot detect this — both story and ADR could be drift-clean
   while the AC is still untestable.

2. **Sizing inconsistencies (HIGH).** Frontmatter↔manifest disagreements on S-4.05,
   S-4.06, S-4.08. The drift audit catches *broken references*; the manifest pointing
   at a different number than the frontmatter is internally consistent at the
   document-reference level — only sizing-judgment review surfaces this.

3. **Pure/effectful pattern propagation (MEDIUM).** S-4.06's task 9a/9b is exemplary,
   but no policy or ADR mandates this pattern across stories. S-4.04 dedup and S-4.05
   rate_limit could adopt it but currently don't. Architect won't author an ADR for a
   coding pattern; spec-quality is the right surface to flag this.

4. **Compound AC anti-pattern (MEDIUM).** S-4.03 AC-9 bundles 6 assertions; S-4.06
   AC-12 bundles 3; S-4.08 AC-3 bundles 4. Drift audit sees these as "AC present and
   traced"; only quality review sees them as testability/attributability hazards.

5. **Test-fixture deps presented as build deps (MEDIUM).** S-4.08's depends_on includes
   S-6.11/12/13 (DTU clones) which are test-fixture deps, not build-time deps. The
   topology graph treats them as blocking — over-serializes the wave. Drift audit
   doesn't reason about dep semantics; quality review does.

6. **`Previous Story Intelligence` quality (LOW).** Half the W4 stories say "N/A" or
   "first story in subsystem." Drift audit only checks fields exist; quality review
   checks they carry useful intelligence.

**Convergent findings (where this review and architect overlap):**

- Both this review (QUAL-401-002, QUAL-404-003, QUAL-408-002) and architect's open
  questions flag the 16-permit semaphore lifecycle and `detection_state` CF lock
  granularity as hazards. Architect proposes ADR-013 + ADR-016 to specify the
  decisions; spec-quality additionally requests the STORIES carry forward those
  decisions in Architecture Compliance Rules. Both are needed.

- Both reviews surface the S-1.02 `CaseStatus::can_transition_to()` stub-vs-implemented
  question (architect open question + QUAL-406-007). Both agree this needs verification
  before S-4.06 dispatch.

**Divergent findings (this review only):**

- Sizing reconciliation (NOT an architectural concern; pure quality).
- TDD mode missing (NOT a drift concern; pure quality).
- AC compound-clause decomposition (NOT a drift concern).
- KUDO patterns (architect doesn't have a KUDO output; quality review does).

**Recommended sequencing:**

1. Architect's 5 ADRs (ADR-013, 015, 016, 017, 018) author and converge first per D-202.
2. Consistency-validator's drift audit completes.
3. Phase 4.B story-update burst applies BOTH this quality review's HIGH findings AND
   the drift audit's HIGH findings in a single coordinated pass — minimizes story
   churn.
4. Re-run spec-quality + drift audit (this review's pass-2) before story dispatch to
   verify HIGH findings are RESOLVED.
