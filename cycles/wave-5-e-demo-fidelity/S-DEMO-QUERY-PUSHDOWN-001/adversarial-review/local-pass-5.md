# LOCAL Adversary Pass 5 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-spec-engine: Thread QueryParams push-down into PipelineExecutor via FetchContext
**Pass:** LOCAL adversary pass 5
**Feature HEAD at pass (frozen):** `ed27b5ff`
**Feature HEAD after pass:** `ed27b5ff` (NO fix applied — lane PAUSED for human scope decision)
**Date:** 2026-06-05
**Authority:** BC-5.39.001 D-779 | SAP-2 | CLAUDE.md Canonical Principle

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak after: 0/3 (RESET from 1/3)**

3 CRITICAL + 1 HIGH + 1 MED findings. Streak RESETS 1/3 → 0/3.

**ROOT CAUSE (pass-5 angle):** Passes 1–4 validated green tests against FABRICATED fixtures that do not match production sensor TOMLs/DTU routes. The test helpers (`make_crowdstrike_like_spec`, `make_cyberint_like_spec`, `make_armis_like_spec`) hand-construct TOML shapes inconsistent with the real `*.sensor.toml` files and DTU response types. This is the fixture-level paper-test class (SAP-2 extended: "fabricated fixture parity"). Only CrowdStrike push-down has been independently verified to have real effect (DTU `DetectionListParams` accepts `filter` + `limit` + `offset`).

**NO FIX APPLIED.** Feature HEAD unchanged at `ed27b5ff`. Lane PAUSED at D-1004 for human scope decision (see §Open Decision below).

---

## Findings

### F-PUSHDOWN5-CRIT-001 — CRITICAL — Cyberint time-window push-down inert (paper-test)

**Severity:** CRITICAL
**Confidence:** HIGH
**Category:** SAP-2 DTU↔TOML schema parity | TD-VSDD-059 paper-fix detection

**Description:**
`apply_push_down_to_json_body()` only runs inside `if let Some(body_template)`. The real `cyberint.sensor.toml` `fetch_alerts` step uses `GET` with no `body_template` — it uses cursor pagination, not a POST body. The AC-003 test passes only because the test fixture constructs a spec with a POST body template, which is fabricated and does not match production.

The time-window Cyberint translation path (from/to date injection into the POST body) is unreachable for the real sensor spec. AC-003 validates a code path that production data will never trigger.

**Production sensor shape:** `cyberint.sensor.toml` `fetch_alerts`: `GET` + cursor pagination (no `body_template`).
**Test fixture shape:** Fabricated `make_cyberint_like_spec` constructs a step with `body_template` — a shape that does not exist in production.

**Classification:** TD-VSDD-059 paper-test. The optimization claim for Cyberint is unverified against production spec.

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

### F-PUSHDOWN5-CRIT-002 — CRITICAL — Armis push-down param mismatch vs BC-2.11.007 and DTU

**Severity:** CRITICAL
**Confidence:** HIGH
**Category:** SAP-2 DTU↔TOML schema parity | BC contract violation

**Description:**
The push-down implementation emits `maxResults` and `timeFrame` params for the Armis path. The real `prism-dtu-armis` `SearchQueryParams` only accepts `aql`, `page`, `size`, `offset`, `limit`. The `maxResults` and `timeFrame` fields do not exist in the DTU struct — they are silently dropped.

Furthermore, BC-2.11.007 §Mechanism B documents Armis filtering as verbatim-AQL passthrough, NOT a `timeFrame` param injection. The push-down implementation contradicts the governing behavioral contract.

**DTU route examined:** `crates/prism-dtu-armis/src/routes/search.rs` — `SearchQueryParams` struct.
**BC-2.11.007 §Mechanism B:** Armis filtering is AQL passthrough (`query_filters["aql"]` → `aql` query param). No `timeFrame` or `maxResults` translation is specified.

**Effect:** The Armis time-window push-down is inert (silently dropped params) AND contradicts the governing BC. This is a SAP-2 §3 P1 CRITICAL class finding: translated param has no DTU equivalent → silently wrong data.

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

### F-PUSHDOWN5-CRIT-003 — CRITICAL — Claroty body `limit` push-down inert (paper-test)

**Severity:** CRITICAL
**Confidence:** MEDIUM
**Category:** SAP-2 DTU↔TOML schema parity | TD-VSDD-059 paper-fix detection

**Description:**
`claroty.sensor.toml` uses `body_template: '{}'` with URL-based `offset/limit` pagination. The implementation injects `limit` into the POST body. This body injection does not match the DTU's actual query dispatch mechanism. The S-DEMO-CLAROTY-PAGINATION-001 story is OPEN and body pagination was explicitly deferred there.

The AC-004 test constructs a fabricated Claroty spec fixture with a body template containing a `limit` slot — this shape does not match the production `claroty.sensor.toml` which has `body_template: '{}'` (empty object, no substitution slots).

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

### F-PUSHDOWN5-HIGH-001 — HIGH — BC-2.01.013 v1.12 false "implemented" claims

**Severity:** HIGH
**Confidence:** HIGH
**Category:** Intra-story contradiction | BC accuracy

**Description:**
BC-2.01.013 v1.12 §Postconditions per-sensor table (authored for THIS story at F-PUSHDOWN-008) claims Cyberint `page_size` translation and Claroty `offset` translation as "implemented". These behaviors are NOT implemented (see F-PUSHDOWN5-CRIT-001 and F-PUSHDOWN5-CRIT-003 above). The story §3 (lines ~151–153) repeats the false claim. The story's own ACs (AC-003 and AC-004) omit these behaviors — creating an intra-story contradiction between the BC body and the AC table.

**Route:** product-owner (BC-2.01.013 per-sensor table correction + story §3 narrative correction).

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004; awaits human scope decision then product-owner routing)

---

### F-PUSHDOWN5-MED-001 — MED — AC-005 result-equivalence proven only on single-step fabricated spec

**Severity:** MED
**Confidence:** MEDIUM
**Category:** Test coverage gap | AC completeness

**Description:**
AC-005 `test_BC_2_11_007_push_down_result_equivalence_invariant` proves result equivalence using a single-step Wiremock fixture. The real production path is a two-step pipeline (spec-engine hands off to PipelineExecutor; PipelineExecutor constructs FetchContext and calls the adapter). AC-005 does not exercise the full production two-step path.

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

## Summary Table

| Finding | Severity | Conf | Status |
|---------|----------|------|--------|
| F-PUSHDOWN5-CRIT-001 | CRITICAL | HIGH | OPEN — paused |
| F-PUSHDOWN5-CRIT-002 | CRITICAL | HIGH | OPEN — paused |
| F-PUSHDOWN5-CRIT-003 | CRITICAL | MED | OPEN — paused |
| F-PUSHDOWN5-HIGH-001 | HIGH | HIGH | OPEN — paused (product-owner route) |
| F-PUSHDOWN5-MED-001 | MED | MED | OPEN — paused |

---

## Fabricated-Fixture vs Production-Spec Comparison

This table exposes the core root cause: passes 1–4 validated push-down against fabricated fixture shapes. The real production sensor TOMLs and DTU routes have different shapes, making 3 of 4 sensor push-down translations inert.

| Sensor | Test Fixture Shape | Production TOML Shape | DTU Accepted Params | Push-down Effect |
|--------|-------------------|----------------------|--------------------|--------------------|
| CrowdStrike | Fabricated, but spec shape matches DTU route | `GET` query params | `filter`, `limit`, `offset` in `DetectionListParams` | REAL — `limit` + `filter` injection works |
| Cyberint | `make_cyberint_like_spec` constructs POST body template | `GET` + cursor pagination; no `body_template` | No POST body | INERT — body injection unreachable for real spec |
| Armis | `make_armis_like_spec` (shape details unclear) | AQL via `?aql=` query param | `aql`, `page`, `size`, `offset`, `limit` in `SearchQueryParams` | INERT — `maxResults`/`timeFrame` not in DTU |
| Claroty | `make_claroty_like_spec` constructs body with `limit` slot | `body_template: '{}'`; URL offset/limit pagination | — (pagination deferred to S-DEMO-CLAROTY-PAGINATION-001) | INERT — body injection bypasses DTU dispatch |

---

## Convergence Trajectory (through pass 5)

| Pass | Feature HEAD | Findings | Delta | CLEAN(strict) | CLEAN(PR-merge) | Streak |
|------|-------------|----------|-------|--------------|-----------------|--------|
| 1 | 19184786→a75fada4 | 8 (2H+2M+2L+2OBS) | — | no | no | 0/3 |
| 2 | a75fada4→688f82b5 | 3 (1M+1L+1OBS) | -5 | no | no | 0/3 |
| 3 | 688f82b5→ed27b5ff | 1 (1 LOW) | -2 | no | yes | 0/3 |
| 4 | ed27b5ff | 0 (fixture-only lens) | -1 | yes (limited lens) | yes | 1/3 |
| 5 | ed27b5ff (frozen) | 5 (3C+1H+1M) | +5 REGRESSION | no | no | **0/3 RESET** |

**Regression explanation:** Pass 4 used a limited lens (unit-test correctness within the test suite as written). Pass 5 applied SAP-2 extended to fabricated-fixture parity, discovering the fixture-vs-production spec gap that passed 1–4 did not test. This is a fixture-level paper-test class, not new implementation defects.

---

## Open Decision (D-1004)

See local-pass-6.md §Root-Cause Synthesis and §Open Decision for the full scope-decision framing with Option A / Option B. Lane PAUSED pending human direction.

---

## [process-gap] Codification Candidate

LOCAL adversary passes should cross-reference test fixtures against production `*.sensor.toml` files and DTU route structs EARLY (passes 1–3 missed this 3 times). SAP-2 spirit ("verify TOML column ↔ DTU types.rs field parity") should extend to "fabricated fixture parity" even when no TOML changed in the diff.

Proposed rule: when a story's test suite uses `make_<sensor>_like_spec` or any in-test spec constructor that does NOT load from `crates/prism-sensors/specs/*.sensor.toml`, the adversary MUST in the FIRST pass verify that the constructed fixture shape matches the production TOML for that sensor (method, pagination mode, body_template presence, DTU accepted params).

**Label:** [process-gap] SAP-2-FABRICATED-FIXTURE-EXTENSION — codification candidate for session-reviewer at cycle-close.
