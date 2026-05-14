---
document_type: fix-burst-closure-report
target_artifact: S-PLUGIN-PREREQ-D
fix_burst_N: 30
target_pass: 32
findings_closed: "3 in-scope (1 CRIT F-LP32-CRIT-001 phantom variant Path A removal; 2 MED F-LP32-MED-001 stale v1.5 pin + F-LP32-MED-002 changelog schema regression)"
findings_intent_adjudicated: 0
findings_deferred: "1 OBS (F-LP32-OBS-002 codification candidate #17 BC-amendment error-variant existence verification — cycle-close session-reviewer adjudication)"
findings_addressed_in_scope: "1 OBS (F-LP32-OBS-001 §BC Amendments past-tense reframe addressed inline)"
producer: "state-manager (orchestrator-coordinated; story-writer + product-owner parallel + state-manager closure)"
specialist_routing: "Multi-agent burst: story-writer (3 story-edits) + product-owner (BC-2.17.002 v1.7 amendment + BC-INDEX v4.73) parallel + state-manager closes single commit"
path_a_adjudication: "Path A selected per CLAUDE.md Canonical Principle Rule 2 (feature order is only acceptable speed lever). Path B (introduce new PluginError variant) would have required signature change on host_http_request, new E-PLUGIN-NNN code, error.rs variant addition, error-taxonomy.md row, §Error Taxonomy Additions row, AC-7 update, 6 test site updates — larger blast radius for security-equivalent outcome. Path A uses existing E-PLUGIN-005 SandboxViolation semantics aligning with AC-7's prescribed HTTP 403 + existing code at host_functions.rs:65 + existing taxonomy. Zero new scope."
story_v_before: "1.29"
story_v_after: "1.30"
bc_2_17_002_v_before: "1.6"
bc_2_17_002_v_after: "1.7"
bc_index_v_before: "4.72"
bc_index_v_after: "4.73"
factory_shas: ["b9c4edea", "9f56f2f4", "<closure SHA TBD>"]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → 5 → 1 → 1 → 3 → 4 → CLOSED"
next_action: "Adversary pass-33 dispatch — target streak 0/3 → 1/3 if CLEAN. Apply codifications #11-#16 candidates + sub-extension + #17 candidate. Trajectory pass-25..pass-32: 4→1→4→5→1→1→3→4."
codification_candidate_17: "BC-amendment error-variant existence verification — when BC body references a PluginError/PrismError/SpecEngineError variant, the variant must exist in the canonical error enum OR be introduced via story §Error Taxonomy Additions with corresponding error-taxonomy.md row. Pass-32 surfaced this via fix-burst-29's introduction of phantom PluginError::AllowlistRejected. Path A closure used existing variant semantics."
---

# Fix-Burst-30 Closure Report — S-PLUGIN-PREREQ-D

**Burst type:** Multi-agent parallel (story-writer + product-owner) + state-manager closure
**Pattern:** PREREQ-D fix-burst-30 path A; 33rd consecutive single-commit (TD-VSDD-053)
**Findings closed:** 3 in-scope (1 CRIT + 2 MED). 1 OBS addressed inline. 1 OBS routed cycle-close.

---

## Summary

Pass-32 BLOCKED on 1 CRIT + 2 MED + 2 OBS. Fix-burst-30 closed all 3 in-scope findings via multi-agent parallel dispatch (story-writer + product-owner) with state-manager single-commit closure. The critical finding was caused by fix-burst-29's introduction of a phantom `PluginError::AllowlistRejected` variant that does not exist in the canonical error enum. Path A adjudication used existing `E-PLUGIN-005 SandboxViolation` semantics rather than introducing a new variant, keeping blast radius at zero.

---

## Story-Writer Fixes (v1.29 → v1.30)

### F-LP32-MED-001 — Stale v1.5 pin at AC-9 closure note line 419

**Before:** AC-9 closure note at story line 419 cited `BC-2.17.002 v1.5` as the pinned version.

**After:** Corrected to `BC-2.17.002 v1.7` with fix-burst-30 attribution, reflecting the two intermediate amendments: v1.6 (fix-burst-29 EC-17-007 security-semantic rewrite) and v1.7 (fix-burst-30 EC-17-007 phantom-variant removal, this burst).

**Root cause:** Version pin was last updated at fix-burst-23 (v1.5 was correct then). fix-burst-29 bumped to v1.6 but the sibling-site sweep at AC-9 closure note was not included in the 5/5 scope declared by the story-writer. fix-burst-30 catches the residual.

**Sibling-sweep:** 1 active-body site (line 419). Changelog/audit-trail rows preserved per TD-VSDD-060.

---

### F-LP32-MED-002 — §Changelog rows 1.27/1.28/1.29 missing Burst column

**Before:** §Changelog rows for story versions v1.27, v1.28, and v1.29 were rendered in a 4-cell schema (Version | Date | Author | Summary), missing the mandatory Burst column that all prior changelog rows carry.

**After:** Burst column restored to all three affected rows:
- v1.27 row: `Burst: fix-burst-27 stage-1`
- v1.28 row: `Burst: fix-burst-28 stage-1`
- v1.29 row: `Burst: fix-burst-29 stage-1`

This restores 5-cell schema parity with the rest of the §Changelog table and eliminates the rendering corruption that adversary pass-32 observed.

**Root cause:** fix-burst-27 initially authored v1.27 changelog entry without Burst column; the gap propagated to v1.28 and v1.29 since both entries were modeled on v1.27. The adversary detected the schema regression at pass-32 (first full-schema audit of these three rows).

---

### F-LP32-OBS-001 — §BC Amendments In-Scope retrospectively reframed

**Before:** §BC Amendments section was titled "BC Amendments In-Scope" and framed amendments in forward-looking directive voice ("BC-2.17.002 will be amended…").

**After:** Section retitled "BC Amendments Landed" and content reframed past-tense, documenting:
- v1.6 amendment (fix-burst-29): EC-17-007 security-semantic rewrite from pre-AC-7 allow-all to post-AC-7 default-deny
- v1.7 amendment (fix-burst-30): EC-17-007 phantom AllowlistRejected variant removed; replaced with existing E-PLUGIN-005 SandboxViolation semantics

**Classification:** OBS finding. Addressed inline per orchestrator scope authorization. No separate deferral.

---

### Frontmatter and §Changelog

- Frontmatter: `version: 1.29 → 1.30`; `timestamp: 2026-05-14T10:00:00Z → 2026-05-14T13:00:00Z`
- v1.30 §Changelog row added (5 cells; Burst: fix-burst-30 stage-1)
- Sibling-site sweep: 5/5 CLEAN (story-writer report)

---

## Product-Owner Fixes — BC-2.17.002 v1.6 → v1.7

### F-LP32-CRIT-001 — Phantom PluginError::AllowlistRejected variant (Path A)

**Before (BC-2.17.002 v1.6 EC-17-007):**
```
EC-17-007: PluginError::AllowlistRejected
  Condition: HTTP request target not in allowed_urls
  Result: host_http_request returns Err(PluginError::AllowlistRejected)
```

This variant was introduced by fix-burst-29 when amending EC-17-007 from "allow-all" to "default-deny" semantics. However, `PluginError::AllowlistRejected` does not exist in `crates/prism-core/src/error.rs`. The canonical `PluginError` enum has exactly 8 variants: `Trapped / Timeout / MemoryExceeded / NotLoaded / InvalidInterface / SandboxViolation / CompilationFailed / EmptyPluginId`. The phantom variant was not present in error-taxonomy.md, story §Error Taxonomy Additions, or AC-7.

**Path A adjudication rationale:**

Path A (use existing `E-PLUGIN-005 SandboxViolation` semantics) was selected over Path B (introduce new `PluginError::AllowlistRejected` variant + new E-PLUGIN-NNN + error.rs change + error-taxonomy.md row + §Error Taxonomy Additions row + AC-7 update + 6 test site updates). Per CLAUDE.md Canonical Principle Rule 2, feature order is the only acceptable speed lever. The security outcome is equivalent:
- AC-7 prescribes HTTP 403 response (both paths achieve this)
- `E-PLUGIN-005 SandboxViolation` semantics match: plugin attempted action that violates sandbox policy
- Existing code at `crates/prism-spec-engine/src/plugin/host_functions.rs:65` already implements this path
- Zero new scope required

**After (BC-2.17.002 v1.7 EC-17-007):**
```
EC-17-007: E-PLUGIN-005 (SandboxViolation — URL not in allowlist)
  Condition: HTTP request target not in allowed_urls (empty vec![] or explicit list)
  Result: host_http_request returns HttpResponse { status: 403, body: b"", headers: {} } synchronously (per AC-7)
  Audit: tracing::warn!(event_type = "plugin_http_request_blocked", plugin_id = %id, url = %target, "HTTP request blocked — URL not in allowlist")
```

**BC-2.17.002 §Changelog v1.7 row added:**
```
| v1.7 | 2026-05-14 | fix-burst-30 | EC-17-007: phantom PluginError::AllowlistRejected removed; replaced with existing E-PLUGIN-005 SandboxViolation semantics (HttpResponse { status: 403 } synchronously per AC-7 + existing host_functions.rs:65 code + existing taxonomy). Path A closure. |
```

**Sibling-site sweep (product-owner):** 5/5 CLEAN. No active-body sites in BC-2.17.002 v1.7 retain the phantom variant name.

---

## BC-INDEX v4.72 → v4.73

BC-INDEX row for BC-2.17.002 updated to reflect v1.7 per POL-11.

**Sweep coverage:** BC-INDEX row version cell updated. No count changes. bc_index_version: "4.72" → "4.73" (recorded in STATE.md + SESSION-HANDOFF.md frontmatter).

---

## Sibling-Site Sweep Results

### Story-writer sweep (5/5 CLEAN)
1. AC-9 closure note line 419 — FIXED (v1.5 → v1.7)
2. §Changelog rows 1.27/1.28/1.29 Burst column — FIXED (4-cell → 5-cell)
3. §BC Amendments section heading + content — FIXED (past-tense reframe)
4. Active body scan for forward-looking "will be amended" — CLEAN
5. Active body scan for `AllowlistRejected` references — CLEAN (only in changelog/audit-trail rows, which are historical and preserved)

### Product-owner sweep (5/5 CLEAN)
1. BC-2.17.002 body scan for `AllowlistRejected` — CLEAN after v1.7 amendment (historical v1.6 changelog row preserved per audit-trail)
2. BC-2.17.002 EC-17-007 HTTP 403 alignment with AC-7 — VERIFIED CONSISTENT
3. BC-2.17.002 audit-log mechanism — VERIFIED (`tracing::warn!(event_type = "plugin_http_request_blocked", ...)` aligns with BC-2.16.002 §Canonical Structured Event Catalog row plugin_http_request_blocked)
4. error-taxonomy.md E-PLUGIN-005 canonical semantics match — VERIFIED (SandboxViolation semantics consistent)
5. BC-INDEX row — UPDATED v4.72 → v4.73

---

## Codification Candidate #17 (Detailed Entry)

**ID:** codification-candidate-17
**Name:** BC-amendment error-variant existence verification
**Category:** Process-gap (4th consecutive fix-burst-closure-introduced drift class)
**Trigger:** Pass-32 surfaced `PluginError::AllowlistRejected` phantom variant in BC-2.17.002 v1.6 EC-17-007, introduced by fix-burst-29 without verification against the canonical error enum in `error.rs`.
**Pattern:** When a BC amendment introduces or references a `PluginError`, `PrismError`, or `SpecEngineError` variant (or any named enum variant from the error taxonomy), the amending agent MUST verify: (a) the variant exists in the canonical enum at `crates/prism-core/src/error.rs` or `crates/prism-spec-engine/src/error.rs`, OR (b) the variant is being simultaneously introduced via the story's §Error Taxonomy Additions with a corresponding `error-taxonomy.md` row. Referencing a variant that satisfies neither condition is a CRIT finding.
**Recurrence count:** 1 (first instance; pattern extracted immediately per 1-recurrence-for-safety rule on CRIT findings)
**Resolution:** Session-reviewer adjudicates at cycle-close. Candidate for POL-22 Phase C extension (BC-amendment amendment-integrity verification).
**F-LP32-OBS-002 routing:** DEFERRED to cycle-close session-reviewer for codification adjudication.

---

## Convergence Trajectory (Full)

```
16 → 8 → 6 → 4 → 0(false-CLEAN) → 4(RESET) → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1
→ 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0(1/3) → 4(RESET idempotency) → 1
→ 4 → 5 → 1 → 1 → 3 → 4 → CLOSED(fix-burst-30)
```

Pass-25..pass-32 trajectory: `4 → 1 → 4 → 5 → 1 → 1 → 3 → 4` — two consecutive breaks of the decreasing trend. Both breaks caused by fix-burst-closure-introduced drift (fix-burst-23 spawn_blocking anchor at pass-25; fix-burst-29 phantom variant at pass-32). Pass-33 dispatch next; streak 0/3 HOLD.

---

## Operational Notes

- **Single-commit protocol:** This burst follows TD-VSDD-053 single-commit-per-burst. All story-writer + product-owner changes staged; state-manager closure in ONE atomic commit. 33rd consecutive single-commit.
- **No BC promotions:** POL-14 applies at PREREQ-D PR merge, not at fix-burst closure. BC-2.17.002 remains `lifecycle_status: draft`.
- **No push:** factory-artifacts branch is local-only per standing directive (56+ commit divergence correct state; NO PUSH without explicit human authorization).
- **Content-SHA TBD:** Per TD-VSDD-053 anti-pattern #2, this document does not cite the closure commit SHA inline. Run `git -C .factory log -1` after commit to retrieve the SHA.
