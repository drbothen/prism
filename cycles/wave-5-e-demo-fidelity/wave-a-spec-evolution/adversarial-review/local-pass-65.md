---
document_type: adversarial-review
review_id: wave-a-spec-pass-65
pass_number: 65
frozen_head: "factory-artifacts@af2684bfa"
perimeter: wave-a-spec-evolution
mode: spec-review with code grounding
streak_pre_pass: "0/3"
streak_post_pass: "0/3"
verdict: BLOCKED
findings_count: 15
severity_breakdown:
  critical: 2
  high: 5
  medium: 6
  low: 1
  observation: 1
  process_gap: 0
novelty: HIGH
related_state_decision: D-2043
date: 2026-07-27
closed_in_fb63: [CRIT-001, CRIT-002, MED-003, MED-004, MED-006]
open_after_fb63: [HIGH-001, HIGH-002, HIGH-003, HIGH-004, HIGH-005, MED-001, MED-002, MED-005, LOW-001, OBS-001]
version: "1.0"
changelog:
  - version: "1.0"
    date: 2026-07-27
    author: state-manager
    note: "Initial persistence of LOCAL adversary pass 65 report (FB63)."
---

# Adversarial Review — Wave-A Spec-Evolution Perimeter, LOCAL Pass 65

**Frozen `factory-artifacts` HEAD:** `af2684bfa` · **Mode:** spec-review with code grounding · **Context:** fresh, no prior passes read

---

## Version Reconciliation

All 31 perimeter artifacts verified on disk; every version confirmed matching the dispatch list. Key verifications:

| Artifact | On disk | Verdict |
|---|---|---|
| ADR-056 | v0.1, `status: accepted` | resolved |
| all 12 BCs | as dispatched | confirmed |
| VP-153/159/160/161, VP-INDEX | as dispatched | confirmed |
| verification-architecture / coverage-matrix / ARCH-INDEX | v1.47 / v1.49 / v2.283 | confirmed |
| error-taxonomy / invariants | v2.70 / v1.11 | confirmed |
| 9 stories + STORY-INDEX | as dispatched (`total_stories: 265`) | confirmed |

Recomputed arithmetic — VP-INDEX totals (32+88+6+6+29 = 161), verification-architecture TIER1 = 32 Kani IDs, S-WAVE-A-ENGINE-001 counts (27 ACs / 39 RGTs), BC-2.10.007 §Complete field specification 9 rows, RFC 9110 §5.6.2 tchar cardinality 77 — ALL CLEAN.

L1/L7 spot-check: BC-2.16.009, BC-2.16.002, error-taxonomy, ARCH-INDEX, VP-161 — all frontmatter version equals top changelog row; all descending. CLEAN.

---

## Critical Findings

### F-WASE-P65-CRIT-001 — Armis story anchors all sensor-behavior obligations to the Cyberint Assets contract; BC-2.01.008 is unreferenced [CLOSED in FB63]

**Artifacts:** `S-WAVE-A-ARMIS-REMEDIATION-001` frontmatter `behavioral_contracts:`, frontmatter `# BC status:` comment, §Acceptance Criteria AC traces (4 sites), §Product-Owner Tasks PO-002, §Behavioral Contracts table, §Token Budget Estimate

**Defect.** The story declared `behavioral_contracts: [BC-2.01.006, BC-2.06.003]` and asserted "BC-2.01.006 covers Armis behavior." BC-2.01.006 §Description (H1: "Cyberint Assets Cookie-Based Authentication") contains zero occurrences of "Armis." The correct contract is BC-2.01.008 §Description (H1: "Armis Token Exchange Auth with AQL Query Forwarding and Timestamp Fallback"), which directly covers `auth_type = "token_exchange"`, `header_scheme = "raw"`, and `[[credential_refs]] name = "secret_key"` — the exact scope of this story.

Eight mis-anchored sites, including two phantom anchors (POLICY 21): §Product-Owner Tasks PO-002 cited `BC-2.01.006 §Armis-specific postconditions` — a section that does not exist in BC-2.01.006; §Token Budget Estimate carried a `BC-2.01.006 Armis section` row — a phantom section. STORY-INDEX §BC Traceability Matrix row for BC-2.01.008 listed only merged stories, meaning the freshly-amended Armis contract had no live implementing story; POLICY 14 auto-promotion at merge would have promoted the wrong BC.

Partial-fix evidence: story §Version History v1.2 row records "BC-2.01.006 pin from `current` to v1.8" — a prior pass corrected the version pin on the wrong BC ID without questioning the identifier. POL-23 sweeps pin strings; POL-8 passes when all three consistent sites cite the same wrong BC; POL-7 is inapplicable (no Title column). No running gate checks BC subject-matter match.

**Routing:** story-writer (BC array + body + AC traces + PO-002 + token budget); state-manager (STORY-INDEX §Full Story List row + §BC Traceability Matrix rows for BC-2.01.006 and BC-2.01.008). **CLOSED in FB63.**

---

### F-WASE-P65-CRIT-002 — Cyberint Alerts-surface ACs anchored to the Assets contract; BC-2.01.018 is a total story-graph orphan [CLOSED in FB63]

**Artifacts:** `S-WAVE-A-CYBERINT-SPEC-001` frontmatter `behavioral_contracts:`, §Acceptance Criteria AC-003/AC-004/AC-007/AC-008 traces, §Behavioral Contracts table, §Product-owner dependency; BC-2.01.018 §Story Anchor

**Defect.** BC-2.01.006 v1.8 §Description explicitly states it is "restricted to the Cyberint Assets surface only" and that the Alerts surface is "covered by BC-2.01.018." Yet S-WAVE-A-CYBERINT-SPEC-001 v1.4 routed the Alerts obligations (AC-003: POST `/alert/api/v1/alerts` `$.alerts` response path; AC-004: page/size pagination) to BC-2.01.006. BC-2.01.018 appeared zero times across all story files and STORY-INDEX; its §Story Anchor still read "(pending — Wave-A story decomposition, Task #8)" despite Wave-A decomposition having landed. The story also contained a stale §Product-owner dependency demanding "BC-2.01.006 must be amended or split … BC-2.01.006a for alerts" — contradicting POLICY 1's append-only numbering scheme — when the split had already landed as BC-2.01.018.

**Routing:** story-writer (re-anchor Alerts ACs to BC-2.01.018; delete discharged PO gate); product-owner (BC-2.01.018 §Story Anchor); state-manager (STORY-INDEX matrix row for BC-2.01.018). **CLOSED in FB63.**

---

## Important Findings

### F-WASE-P65-HIGH-001 — VP-161 CTL-escape byte set excludes 0x09 (TAB), contradicting BC-2.16.009, error-taxonomy, and the implementing story [OPEN — architect]

**Artifacts:** `vp-161-rule9-error-message-echo-cap-and-ctl-escaping.md` §Property Statement (Property 2), §Proof Method table, §Proof Harness Skeleton SYMBOL RESOLUTION block + Harness 2 predicate; VP-INDEX.md VP-161 row; verification-architecture.md VP-161 row

**Defect.** VP-161 declares the escape domain as `0x00–0x08, 0x0A–0x1F, 0x7F` — explicitly excluding 0x09 (TAB) — and its Harness 2 predicate encodes the corresponding byte check. All three normative sources specify `0x00–0x1F` (inclusive) plus `0x7F`: BC-2.16.009 §Validation Rule 9 §CTL-character escaping, error-taxonomy E-SPEC-027 `{value}` description, and S-WAVE-A-ENGINE-001 AC-025 and §Tasks T-B02 Step 2. The Kani proof is therefore strictly weaker than the contract it claims to prove. EC-009-049 and RG-030 make `header_scheme = "cookie:\t"` a primary Rule-9 test vector, so 0x09 is reachable on the exact path VP-161 governs. The wrong byte set propagated to three documents in FB62 (POLICY 25 multi-cite sweep failure).

**Routing:** architect (VP-161 §Property Statement + §Proof Method + Harness 2 predicate + same-burst POLICY 9 propagation to VP-INDEX and verification-architecture). **OPEN.**

---

### F-WASE-P65-HIGH-002 — ADR-056 §D9 omits two in-crate exhaustive `match PaginationConfig` sites; `#[non_exhaustive]` has no in-crate effect [OPEN — architect, then story-writer]

**Artifacts:** `ADR-056-page-number-pagination-variant.md` §D9 + §Consequences; `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-09, §File Structure Requirements, §Architecture Mapping

**Defect.** ADR-056 §D9 asserts external match arms on `PaginationConfig` are protected by `#[non_exhaustive]`, "no external callsite migration is needed." That is correct for external crates but incomplete: `#[non_exhaustive]` has no effect inside the defining crate (`prism-spec-engine::spec_parser`). Two same-crate exhaustive matches have no wildcard arm:

1. `prism-spec-engine::types` `sensor_table_descriptor_from_table_spec`: `.map(|pag| match pag { CursorToken => PaginationType::Cursor, OffsetLimit => PaginationType::Offset, None => PaginationType::None })`
2. `prism-spec-engine::validation` `validate_sensor_spec` pagination block: three arms — `CursorToken`, `OffsetLimit { page_size }`, `None`

Adding `PageNumber` yields a compile error at both sites. Neither appears in ADR-056 §Consequences' obligation list, nor in the story's §File Structure Requirements or §Architecture Mapping. The `types` site is not mechanical: `PaginationType` is a wire-visible `#[non_exhaustive]` type (`SensorTableDescriptor.pagination_type`), and ADR-056 makes no decision on whether `PageNumber` maps to a new `PaginationType::Page` variant or folds into `Offset`. An implementer adding `_ => PaginationType::None` would silently mislabel paginated tables as unpaginated on the MCP surface.

**Routing:** architect (ADR-056 §D9 + §Consequences + `PaginationType` mapping decision); story-writer (T-09, §File Structure, §Architecture Mapping). **OPEN.**

---

### F-WASE-P65-HIGH-003 — `page_size = 0` semantics contradictory across ADR-056 §D3, `validation.rs::validate_sensor_spec`, and the story [OPEN — architect, then product-owner + story-writer]

**Artifacts:** `ADR-056-page-number-pagination-variant.md` §D3 "Activation gate" + §D4; BC-2.16.002 §Postconditions "PageNumber Pagination Dispatch" activation gate bullet; `S-WAVE-A-CYBERINT-SPEC-001` §Tasks T-09 test list

**Defect.** ADR-056 §D3 states the activation gate is "identical to `OffsetLimit`": `page_size = 0` skips injection. But `prism-spec-engine::validation` `validate_sensor_spec` shows the `OffsetLimit` precedent is two-layered: at spec-load time, `OffsetLimit { page_size: 0 }` produces a `SpecErrorCode::ESpec001` validation error — a TOML author cannot declare `page_size = 0`. The `0` sentinel is engine-internal only. ADR-056 ratifies the runtime half and is silent on the spec-load half. §D4's termination arm with `ps = 0` would never break, running to `MAX_PAGES_PER_STEP` = 1,000 requests. Three inconsistent readings are all textually supported: reject at spec-load (OffsetLimit parity), silent unpaginated, or 1,000-page runaway. The silent-unpaginated reading is a CWE-390-class silent truncation on a security-data path.

**Routing:** architect (ADR-056 amendment); product-owner (BC-2.16.002 postcondition sweep); story-writer (T-09 test expectation). **OPEN.**

---

### F-WASE-P65-HIGH-004 — BC-2.01.018 §Postconditions still mandate `(Timestamp, AlertID)` cursor pagination, contradicting ADR-056 and BC-2.16.002 v2.11 [OPEN — product-owner]

**Artifacts:** `BC-2.01.018-cyberint-alerts-cookie-auth.md` §Description, §Postconditions, §Canonical Test Vectors TV-BC-2.01.018-001/002, EC-018-001

**Defect.** BC-2.01.018 §Postconditions mandates `(Timestamp, AlertID)` 2-tuple cursor pagination; TV-BC-2.01.018-001 requires a cursor set; EC-018-001 specifies cursor fallback behavior. ADR-056 (accepted 2026-07-26) and BC-2.16.002 v2.11 replace cursor pagination on the Cyberint Alerts surface with `PageNumber`; S-WAVE-A-CYBERINT-SPEC-001 AC-004 explicitly deletes `ac_6_cursor_pagination.rs`; ADR-056 §D7 documents that `CursorToken` "would silently malfunction" against Cyberint's schema. The FB53a/FB53b/FB62 bursts swept BC-2.16.002 and the story but never BC-2.01.018 — because no story referenced it (F-WASE-P65-CRIT-002). The orphaning and sweep miss are causally linked: an unreferenced BC is invisible to sibling-sweep greps.

**Routing:** product-owner (BC-2.01.018 §Description, §Postconditions, TV-001, TV-002, EC-018-001 re-grounded on ADR-056 PageNumber). **OPEN.**

---

### F-WASE-P65-HIGH-005 — RG-007/RG-008 assert POST-body keys `page_number`/`page_size` instead of the canonical wire keys `page`/`size` [OPEN — story-writer]

**Artifact:** `S-WAVE-A-CYBERINT-SPEC-001` §Red Gate tests RG-007, RG-008

**Defect.** Canonical wire keys are `page` and `size` — established by ADR-056 §D3 ("inject `"page"` and `"size"`"), ADR-056 §D8 verbatim PO row, BC-2.16.002 §Postconditions PageNumber row, and the story's own AC-003, AC-004, §Tasks T-09 §3, and sibling RG-015. RG-007 asserts the first request body contains `page_number = 1` and `page_size = 100`; RG-008 asserts `page_number = 2`. `page_number`/`page_size` are the TOML declaration names and the Cyberint Assets OpenAPI field name — not the Alerts POST-body wire keys. A test-writer following RG-007/RG-008 literally would author assertions on JSON keys that will never be emitted.

**Routing:** story-writer. **OPEN.**

---

## Medium Findings

### F-WASE-P65-MED-001 — VP-159 normative body carries a bare-L line cite to ADR-054 that has decayed [OPEN — spec-steward]

**Artifact:** `vp-159-declarative-http-auth-lazy-acquisition-and-refresh-on-expiry.md` §Property Statement note block and §Feasibility/TTL discussion (two sites)

**Defect.** Two live-body (non-changelog) sites carry a bare-`L` positional line-number cite embedded in normative prose referring to `ADR-054 §D4 step 4` — a volatile form forbidden by TD-VSDD-091 arm 5. Beyond being forbidden, the cite is already stale: at ADR-054 v0.56, the cited position falls inside step 3, not step 4. The cite points a formal-verifier at the wrong step of the ratified algorithm. Correct anchor form: `ADR-054 §D4 step 4 → "relative_seconds": u64 seconds, default 1799, minus ttl_buffer_secs`.

**Routing:** spec-steward (records-tier; TD-VSDD-096 micro-burst eligible if bundled only with MED-002). **OPEN.**

---

### F-WASE-P65-MED-002 — ADR-052 body carries ~10 live-body `file.rs:NNN` cites; at least one is demonstrably decayed [OPEN — architect]

**Artifact:** `ADR-052-prismql-native-temporal-typing-utf8-to-arrow-timestamp.md` §Context evidence prose, §Rationale ADR-033 push-down paragraph, §Implementation manifest table rows 1/2/3/9

**Defect.** The §Implementation manifest and §Rationale carry multiple live-body cites in the prohibited `file.rs:NNN` form — including the reported forms `spec_driven_adapter.rs` with a line number, `pipe_sql_emitter.rs` with two separate line numbers, `high002_plan_pinning_tests.rs` with a line number, `column.rs` with a line number, and `pushdown.rs` with a line number cited twice. Verified decay: the ADR-033 T1 push-down extractor cite for `pushdown.rs` §range-op section is stale — at that position in `prism-query::pushdown`, the `is_range_op` guard interior currently contains a `return;` statement, not the range-op extractor. None of TD-VSDD-091's two surviving exceptions (Red Gate test tables; AC source-of-truth tables) covers an ADR implementation manifest.

**Routing:** architect (ADR-052 §Implementation manifest re-anchored to symbols). **OPEN.**

---

### F-WASE-P65-MED-003 — S-WAVE-A-CYBERINT-SPEC-001 pinned BC-2.01.006 as `v1.x`, a non-resolvable wildcard [CLOSED in FB63]

**Artifact:** `S-WAVE-A-CYBERINT-SPEC-001` §Behavioral Contracts table BC-2.01.006 row; frontmatter comment

**Defect.** BC-2.01.006 is on disk at v1.8. The pin `v1.x` is a wildcard placeholder indistinguishable from "current" by grep and mechanically unevaluable by POL-23. The sibling `S-WAVE-A-ARMIS-REMEDIATION-001` pins the same BC concretely at v1.8 (per its v1.2 FB60 version history), creating an asymmetry within the same burst family.

**Routing:** story-writer. **CLOSED in FB63** (resolved to v1.8).

---

### F-WASE-P65-MED-004 — S-WAVE-A-ENGINE-001 §Behavioral Contracts Title column diverges from BC H1 at 3 of 4 rows [CLOSED in FB63]

**Artifact:** `S-WAVE-A-ENGINE-001` §Behavioral Contracts table

**Defect.** Three BC H1 titles were shortened in the story's Title column:

| Row | Story title cell | BC H1 (source of truth) |
|---|---|---|
| BC-2.16.009 | Spec File Validation | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation |
| BC-2.01.017 | StaticCookieAuthProvider — No Login Roundtrip | StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection |
| BC-2.01.016 | SensorAuth Open Trait Contract | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) |

Third recurrence of the `DRIFT-LAUNCHER-SIBLING-TITLE-001` pattern (POLICY 7 verbatim-title requirement). BC-2.16.014 was already exact.

**Routing:** story-writer. **CLOSED in FB63.**

---

### F-WASE-P65-MED-005 — RG-030 demands the E-SPEC-027(a) message "VERBATIM" without stating the escaped `{value}` form, unlike sibling RG-029 [OPEN — story-writer + product-owner]

**Artifact:** `S-WAVE-A-ENGINE-001` §Red Gate tests RG-030; `BC-2.16.009` §Error Conditions EC-009-049

**Defect.** RG-030 drives `header_scheme = "cookie:\t"` and instructs the test to assert the E-SPEC-027(a) message "VERBATIM." Per AC-025, §Tasks T-B02 Step 2, and BC-2.16.009 §CTL-character escaping, TAB (0x09) is inside the escape class, so the emitted `{value}` must be the eleven-character string `cookie:\x09`, not `cookie:` followed by a raw TAB. RG-030 never says this, while sibling RG-029 spells the escaped LF expectation out in full. A test-writer building the expected string from the raw TOML input would author a wrong assertion. EC-009-049 has the same silence.

**Routing:** story-writer (RG-030); product-owner (EC-009-049 expected-message column). **OPEN.**

---

### F-WASE-P65-MED-006 — BC-2.01.018 has no row in the STORY-INDEX §BC Traceability Matrix [CLOSED in FB63]

**Artifact:** `STORY-INDEX.md` §BC Traceability Matrix

**Defect.** `BC-2.01.018` returned zero matches anywhere in STORY-INDEX. BC-2.16.014 (introduced the same week) has a matrix row; new BCs normally get registered. The matrix was not updated when the ADR-053 D3 dual-surface split created BC-2.01.018 on 2026-07-22.

**Routing:** state-manager (matrix row + STORY-INDEX version bump per POLICY 11). **CLOSED in FB63.**

---

## Observations

### F-WASE-P65-LOW-001 — VP-161 `timestamp` frontmatter not advanced for the v1.1 edit *(pending intent verification)* [OPEN — architect or human]

VP-161 frontmatter carries `timestamp: 2026-07-26T00:00:00Z` and `modified: []`, but its top changelog row is for the v1.1 edit dated 2026-07-27. Sibling VP-160 also uses `modified: []`, so an empty-array convention for VPs appears deliberate and this cannot be adjudicated without author intent verification. The `timestamp` staleness relative to the v1.1 edit is the narrower half. **OPEN — architect or human.**

---

### F-WASE-P65-OBS-001 [process-gap] — VP-INDEX rows for VP-157 and VP-158 exist with no corresponding VP files; no gate detects row↔file existence [OPEN — story-writer or architect]

VP-INDEX registers VP-157 (added D-1099, 2026-06-11) and VP-158 (added 2026-06-12) with full metadata, but no VP files exist for them in the verification-properties directory — the VP file sequence skips from VP-156 to VP-153/VP-159/VP-160/VP-161. Draft VPs VP-154 and VP-155 do have files, so this is not a drafts-have-no-file convention. VP-INDEX §ADR-037 Retirement section documents index-only VPs explicitly as an exception; VP-157/VP-158 carry no equivalent marker. POLICY 9 makes VP-INDEX the catalog source of truth but nothing verifies row↔file existence — these have survived ~6 weeks and 60+ adversarial passes. Natural AC extension to `S-MAINT-ADR-ANCHOR-GATE-001`. **OPEN.**

---

## Dismissed

| Candidate | Why dismissed |
|---|---|
| ADR-053 `anchor_stories` omits S-WAVE-A-CYBERINT-PATCH-001 | Derivation rule is "verified from §Authority citations in each story." PATCH-001 has no §Authority naming ADR-053; cites BC-2.16.009 and mentions ADR-053 only to exclude itself. Correct as written. |
| S-WAVE-A-ENGINE-001 AC-017 POL-24 comment pinning "error-taxonomy v2.57 … VP-153 v0.21" | Verified: v2.57 is precisely where `token_exchange` was appended to E-SPEC-012's Valid-values clause. Provenance cites, not currency pins — POL-23 inapplicable. |
| VP-153 changelog rows citing "proof-harness comment" positions | Pre-existing unchanged rows, dated 2026-05-16/17 — grandfathered under TD-VSDD-092 L9 ratchet. |
| BC-2.16.002 v2.10 changelog `materialization.rs` line list; BC-2.01.006 / BC-2.02.004 "ADR-023" line rows | Dated 2026-07-10 and 2026-05-11 respectively — pre-2026-07-24, grandfathered under L9 ratchet. |
| ADR-056 hardcoding wire names `page`/`size` as a POLICY 36 violation | Not sensor-conditional; structurally identical to `OffsetLimit` arm's hardcoded `offset`/`limit`. ADR-056 §D1 loose prose, not contradiction of §D3. |
| S-WAVE-A-ENGINE-001 stated 27 ACs / 39 RGTs | Recount confirmed: exactly 27 and 39; density-check chain sums correctly; full AC→RGT coverage. |
| VP-INDEX / verification-architecture / verification-coverage-matrix VP-161 propagation and per-tool arithmetic | Independently recomputed; every total and per-module count is internally consistent. |
| VP-161 Harness 1 target `truncate_at_char_boundary` symbol grounding | Verified: `pub(crate) fn truncate_at_char_boundary` present in `prism_spec_engine::validation`. POLICY 31 satisfied; `[PLANNED]` marker on provisional Harness 2 symbol present. |
| 77-character tchar set and ≤128-codepoint cookie-name bound | Recomputed 15+26+26+10 = 77. |
| SAP-1 spot-check on 7 `event_type` values | All 7 present in BC-2.16.002. No new `event_type` site introduced by any perimeter story. |

---

## Severity Breakdown

| Severity | Count | IDs | Status after FB63 |
|---|---|---|---|
| CRITICAL | 2 | CRIT-001, CRIT-002 | Both CLOSED |
| HIGH | 5 | HIGH-001…HIGH-005 | All OPEN |
| MEDIUM | 6 | MED-001…MED-006 | MED-003, MED-004, MED-006 CLOSED; MED-001, MED-002, MED-005 OPEN |
| LOW | 1 | LOW-001 | OPEN |
| OBSERVATION / process-gap | 1 | OBS-001 | OPEN |
| **Total** | **15** | | 5 CLOSED / 10 OPEN |

---

## Novelty Assessment

**Novelty: HIGH.** Both CRITICAL findings are wrong-BC-ID mis-anchors — a defect class structurally invisible to every running sweep axis (POL-23 sweeps pin strings; POL-8 sweeps frontmatter↔body↔AC consistency, which passes when all three cite the same wrong BC; POL-7 is inapplicable on stories with no Title column; POLICY 22 Phase A/B had not been applied to the sensor-BC layer). The evidence is inside the artifacts: `S-WAVE-A-ARMIS-REMEDIATION-001` §Version History records a prior pass correcting the version pin on BC-2.01.006 — the correct-version-of-wrong-BC pattern.

HIGH-002/HIGH-003 required reading Rust to catch: ADR-056 §D9 makes a correct-sounding but scope-wrong claim about `#[non_exhaustive]` that stops further inquiry. HIGH-001 is same-burst self-consistent error (VP-161 authored and propagated to three documents in FB62 — hardest for an index-currency check to see). HIGH-004 demonstrates the cascade: an orphaned BC (CRIT-002) becomes invisible to sibling-sweep greps, so ADR-056's pagination change never reached it.

---

## Coverage Declaration

| Probe / Artifact | Status | Notes |
|---|---|---|
| Version/frontmatter reconciliation, all 31 perimeter artifacts | FULLY-CHECKED | |
| TD-VSDD-092 L1 (frontmatter = top changelog row) | PARTIALLY-CHECKED | verified on BC-2.16.009, BC-2.16.002, error-taxonomy, ARCH-INDEX, VP-161; other 9 BCs, 8 ADRs, VP-153/159/160, VP-INDEX, verification-* not checked |
| TD-VSDD-092 L7 (descending changelogs) | PARTIALLY-CHECKED | verified on error-taxonomy, BC-2.16.009, ARCH-INDEX, VP-161 |
| TD-VSDD-091 volatile-cite scan | FULLY-CHECKED across perimeter BCs, ADRs, VPs, and all 7 Wave-A stories | 2 live-body violations found (MED-001, MED-002); changelog hits triaged for L9 ratchet grandfathering |
| POL-23 pin currency | FULLY-CHECKED for all BC pins in all 7 Wave-A story §Behavioral Contracts tables | 1 violation (MED-003) |
| POLICY 4/5/22 semantic anchoring (BC↔story subject match) | FULLY-CHECKED for 7 Wave-A stories | 2 CRITICAL found |
| POLICY 7 (BC H1 ↔ story/index titles) | FULLY-CHECKED for perimeter BCs (12 H1s read) | 1 violation (MED-004) |
| POLICY 8 (frontmatter ↔ body ↔ AC bidirectional) | FULLY-CHECKED for S-WAVE-A-ENGINE-001, ARMIS, CYBERINT-SPEC; PARTIALLY for remaining 4 stories | |
| POLICY 9 (VP-INDEX → architecture propagation) | FULLY-CHECKED for VP-160/VP-161; arithmetic recomputed end-to-end | |
| POLICY 11/13/26/32 (index bumps, status agreement, changelog schema) | PARTIALLY-CHECKED | STORY-INDEX rows read for all 9 stories; statuses agree; full 265-row scan NOT-REACHED |
| POLICY 15 (ADR `anchor_stories` / runtime wiring) | FULLY-CHECKED for all 9 perimeter ADRs | |
| POLICY 21 (phantom §anchors) | PARTIALLY-CHECKED | ADR-056 §D3/§D8 verified real; 2 phantoms found in ARMIS story |
| POLICY 31 (VP harness symbol grounding) | FULLY-CHECKED for VP-161; NOT-REACHED for VP-153/159/160 | |
| AC/RGT/VP/field-count/charset arithmetic | FULLY-CHECKED | all correct |
| SAP-1 (tracing event_type catalog completeness) | PARTIALLY-CHECKED | full workspace grep; 7 values spot-checked; no new emission introduced. Full 88-value cross-walk NOT performed |
| **SAP-2 (DTU ↔ TOML schema parity)** | **NOT-REACHED** | S-WAVE-A-CYBERINT-SPEC-001 and S-WAVE-A-ARMIS-REMEDIATION-001 both author new sensor TOMLs; DTU types and route handlers NOT read column-by-column. Largest gap in this pass; SAP-2 standalone probe dispatched before fix cascade per human direction. |
| SAP-3 (spec-arm end-to-end reachability) | PARTIALLY-CHECKED | per-arm test-existence audit against real test files NOT performed (all stories draft) |
| SID-1/SID-2 | PARTIALLY-CHECKED | SID-2 verified on RG-028/RG-029; gap found at RG-030 (MED-005). SID-1 not applicable |
| POLICY 2 (DI-NNN → BC orphan detection) | **NOT-REACHED** | version confirmed only; DI enumeration and BC-citation sweep not performed |

---

```
CLEAN (strict): no
CLEAN (PR-merge): no
```

**BC-5.39.001 consequence:** 3-CLEAN streak RESET to 0/3. Fix-burst dispatch required on STRICT criterion.

**TD-VSDD-096 NOT applicable** — finding set includes contract-semantics, algorithm, and API-contract defects; full cascade ceremony applies.

**Pass 66** must open with SAP-2 (`prism-dtu-cyberint` route handlers read column-by-column against TOML column schemas) and POLICY 2 (DI orphan sweep) — both NOT-REACHED here. SAP-2 standalone probe was dispatched before the fix cascade and surfaced 2 P1 CRITICALs (see `sap2-probe-dtu-toml-parity.md`).
