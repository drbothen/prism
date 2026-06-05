---
document_type: research
research_type: general
topic: "Armis Centrix ASQ/AQL time-window / temporal filtering syntax (device last-seen + alert created-time + bounded ranges)"
slug: armis-aql-time-window-syntax
date: "2026-06-05"
producer: research-agent
status: complete
confidence: HIGH
triggered_by:
  - "S-DEMO-QUERY-PUSHDOWN-001 v2 (Armis time-window push-down via AQL-clause augmentation)"
  - "Architect flag: existing timeFrame:\"Last 3 Hours\" and lastSeen:>\"T\" forms UNCONFIRMED / possibly-guessed"
forms_in_codebase_under_review:
  parity_test: 'in:devices timeFrame:"Last 3 Hours"  (crates/prism-spec-engine/tests/parity/armis.rs:156,383)'
  story_ac_armis_001: 'in:devices lastSeen:>"2026-01-01"  (S-DEMO-QUERY-PUSHDOWN-001 AC-ARMIS-001)'
recommended_syntax:
  device_relative: 'in:devices timeFrame:"3 Hours"'
  device_absolute_lower: 'in:devices after:2026-01-01T00:00:00'
  device_bounded_range: 'in:devices after:2026-01-01T00:00:00 before:2026-01-02T00:00:00'
  alert_relative: 'in:alerts timeFrame:"1 Hours"'
  alert_bounded_range: 'in:alerts after:2026-01-01T00:00:00 before:2026-01-02T00:00:00'
  alert_time_field_for_sort: 'time (camelCase confirmed; used as orderBy=time)'
  field_naming: camelCase
architect_adjudication_required: false
related_artifacts:
  - ".factory/research/armis-aql-discriminator-syntax-2026-06.md"
  - ".factory/stories/S-DEMO-QUERY-PUSHDOWN-001-query-param-push-down-into-pipeline-executor.md"
  - ".factory/specs/architecture/decisions/ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md"
  - "crates/prism-spec-engine/tests/parity/armis.rs"
  - "crates/prism-dtu-armis/src/routes/search.rs"
  - "crates/prism-sensors/specs/armis.sensor.toml"
---

# Research: Armis Centrix ASQ/AQL Time-Window / Temporal Filtering Syntax

**Date:** 2026-06-05
**Type:** General (technology / wire-syntax fidelity)
**Confidence:** HIGH for the recommended forms; specific residual-uncertainty points called out in §7.

---

## TL;DR — RECOMMENDED-SYNTAX (build against this)

The real Armis Centrix v1 search API (`GET /api/v1/search?aql=...`) expresses temporal filtering **inside the AQL string** with two distinct, fully-attested mechanisms — and does **NOT** use Lucene-style field-comparison operators (`lastSeen:>"T"`).

| Need | Canonical literal AQL clause | Confidence |
|------|------------------------------|------------|
| Device — relative window | `in:devices timeFrame:"3 Hours"` | HIGH |
| Device — absolute lower bound | `in:devices after:2026-01-01T00:00:00` | HIGH |
| Device — bounded range (start AND end) | `in:devices after:2026-01-01T00:00:00 before:2026-01-02T00:00:00` | HIGH |
| Alert — relative window | `in:alerts timeFrame:"1 Hours"` | HIGH |
| Alert — bounded range | `in:alerts after:2026-01-01T00:00:00 before:2026-01-02T00:00:00` | HIGH |
| Alert time/sort field name | `time` (camelCase) — used as `orderBy=time`; alert payload field is `time` | HIGH (as field name) / MED (as a filter predicate) |
| Field naming convention | **camelCase** (`lastSeen`, `firstSeen`, `macAddress`, `alertId`, `timeFrame`) | HIGH |

**Key correction to the codebase (the architect's flag was correct):**

- `timeFrame:"..."` is a **REAL** Armis AQL predicate (4 independent sources). The value is a quoted **`"<N> <unit>"`** duration (e.g. `"1 Hours"`, `"3 Hours"`, `"7 days"`, `"10 Seconds"`). The exact string `"Last 3 Hours"` used in `parity/armis.rs` is **NOT attested** — every real example is the bare-duration form (`"3 Hours"`), not the `"Last N …"` phrasing. Recommend changing the test/demo string to `"3 Hours"`.
- `lastSeen:>"2026-01-01"` (a `field:>"value"` comparison operator embedded in the AQL string, in `S-DEMO-QUERY-PUSHDOWN-001` AC-ARMIS-001) is **NOT a confirmed Armis AQL form**. No surveyed source shows `>`/`<`/`>=` comparison operators inside the AQL query string. The real absolute-time mechanism is the **`after:` / `before:` keywords with bare ISO timestamps**. `lastSeen` appears in the wild only as an `orderBy`/sort field (`orderBy=lastSeen`, `lastSeen:desc`), never as a `>`-comparison filter. **The `lastSeen:>"T"` form should be replaced with `after:T` / `before:T`.**

---

## 1. Scope & what was flagged

`S-DEMO-QUERY-PUSHDOWN-001` v2 augments the Armis AQL string with a time-window clause derived from the PrismQL WHERE predicate. Two candidate forms exist in the current tree and were flagged by the architect as possibly-guessed:

| Form | Location | Verdict |
|------|----------|---------|
| `in:devices timeFrame:"Last 3 Hours"` | `crates/prism-spec-engine/tests/parity/armis.rs:156` and `:383` | Predicate REAL; value-string `"Last 3 Hours"` UNATTESTED (use `"3 Hours"`) |
| `in:devices lastSeen:>"2026-01-01"` | `S-DEMO-QUERY-PUSHDOWN-001` AC-ARMIS-001 (story body) | Comparison-operator form UNCONFIRMED — replace with `after:`/`before:` |

Note: per the existing story `AC-ARMIS-001`/`AC-ARMIS-002`, prism's design treats the AQL string as **user-supplied / opaque passthrough** — prism does NOT inject `timeFrame`/`maxResults` itself, and time predicates outside the AQL string are DataFusion post-filters. This research therefore answers "what is the correct literal AQL syntax when a time-window clause IS placed in the AQL string" (for demos, test vectors, and the DTU's opaque-string handling), which is exactly what gates v2.

---

## 2. Device last-seen / device time filtering (Q1)

### 2.1 Relative window — `timeFrame:"<N> <unit>"` (HIGH)

**Literal form:** `in:devices timeFrame:"3 Hours"`

Worked example (verbatim from Google Cloud Chronicle Armis-devices parser):
```
curl ... "${ARMIS_INSTANCE}/api/v1/search/" \
  --data-urlencode "aql=in:devices timeFrame:\"1 Hours\"" \
  --data-urlencode "length=1" \
  --data-urlencode "fields=id,name,type,category,model,manufacturer,ipAddress,macAddress,operatingSystem,riskLevel,site,firstSeen,lastSeen"
```

Attested value formats (all quoted `"<number> <unit>"`):
- `"1 Hours"` — Google Chronicle (devices + alerts parsers)
- `"{days_ago} days"` (default) — Axonius Armis adapter
- `"10 Seconds"`, `"1 Hours"` — PyPI `armis` Python SDK examples
- Units observed: `Seconds`, `Hours`, `days` (and from the Cortex XSOAR `time_frame` param's accepted formats: minutes/hours/days/weeks/months/years).

> The Google Chronicle doc additionally states the Armis API supports a **maximum timeFrame of 100 days** — a real operational ceiling worth noting for the DTU/demo.

### 2.2 Absolute lower bound — `after:<ISO timestamp>` (HIGH)

**Literal form:** `in:devices after:2026-01-01T00:00:00`

Verbatim from Google Cloud Chronicle Armis-devices parser (Python):
```python
# Build AQL query with time filter
start_str = start_time.strftime('%Y-%m-%dT%H:%M:%S')
aql_query = f'in:devices after:{start_str}'
```

Key observations:
- The timestamp is **bare (unquoted)** — `after:2026-01-01T00:00:00`, NOT `after:"2026-01-01T00:00:00"`.
- The format used is `%Y-%m-%dT%H:%M:%S` (ISO-8601 **without** a trailing `Z`/offset). A date-only form `after:2022-03-10` is also attested (BlinkOps).
- This is the same source that uses `orderBy=lastSeen` for devices — confirming `lastSeen` is the sort/observation field, but it is NOT used as a `>` filter; the filter is `after:`.

### 2.3 `lastSeen:>"T"` is NOT a confirmed filter form

No surveyed source uses a `lastSeen:>`/`lastSeen:>=` comparison operator inside the AQL string. `lastSeen` appears in the corpus as: (a) a response field (`firstSeen`/`lastSeen` device attributes), and (b) an `orderBy` value / `lastSeen:desc` sort directive. The temporal *filter* mechanism is `after:`/`before:`/`timeFrame:`. → **Replace `lastSeen:>"2026-01-01"` with `after:2026-01-01T00:00:00`.**

---

## 3. Bounded absolute range — start AND end (Q2)

**Literal form (HIGH):** `in:devices after:2026-01-01T00:00:00 before:2026-01-02T00:00:00`

Decisive source — BlinkOps Armis Centrix Search action documents the bounded-range example **verbatim**:
```
in:devices after:2022-03-10 before:2023-05-15
```

Construction rules (all confirmed by the BlinkOps example + Chronicle `after:` usage):
- Two keyword clauses **`after:`** (lower bound) and **`before:`** (upper bound), **space-separated**.
- **No `AND` keyword**, **no parentheses**, **no comparison operators**, **dates bare/unquoted**.
- The same form applies to `in:alerts`.

This directly refutes all the bracket/operator candidates that were guessed: it is NOT `lastSeen:(>"T1" AND <"T2")`, NOT `lastSeen:[T1 TO T2]`, NOT `timeFrame:` for a bounded absolute range. It IS `after:T1 before:T2`.

---

## 4. Alert creation-time filtering (Q3)

### 4.1 Time-window filtering on alerts uses the SAME `after:` / `timeFrame:` mechanism (HIGH)

Verbatim from Google Cloud Chronicle Armis-**alerts** parser:
```python
start_str = start_time.strftime('%Y-%m-%dT%H:%M:%S')
aql_query = f'in:alerts after:{start_str}'
```
and the verification curl:
```
--data-urlencode "aql=in:alerts timeFrame:\"1 Hours\""
--data-urlencode "fields=alertId,type,title,description,severity,status,time,activityUUIDs,deviceIds,connectionIds"
```

So:
- Relative: `in:alerts timeFrame:"1 Hours"`
- Absolute lower bound: `in:alerts after:2026-01-01T00:00:00`
- Bounded: `in:alerts after:T1 before:T2`

### 4.2 The alert time field is `time` (NOT `timestamp`/`createdAt`/`firstDetected`) (HIGH for field name)

The alert record's temporal field is **`time`** (camelCase), confirmed three ways:
- Chronicle alerts parser uses **`orderBy: 'time'`** and lists `time` in the alert `fields=` set.
- Cortex XSOAR alert payload: `"time": "2021-02-16T06:23:02.101479+00:00"`.
- Elastic Armis integration maps `armis.alert.time` (date).

**Caveat (MED):** these confirm `time` as the alert's *field/sort* name. None of the surveyed sources show a `time:`-as-AQL-filter-predicate example. For alert time-window *filtering*, the attested mechanism is the entity-agnostic `after:`/`before:`/`timeFrame:` keywords (§4.1), not a `time:>"..."` predicate. Recommend filtering alerts via `after:`/`before:`/`timeFrame:`, and using `time` only where a field reference is needed (sort, projection, post-filter).

---

## 5. Entity-type discriminator + clause composition (Q4)

- The `in:devices` / `in:alerts` prefix convention prism's DTU already uses is **CONFIRMED CORRECT** (re-validated here; consistent with the prior 2026-06-01 discriminator research, HIGH confidence). BlinkOps enumerates the full valid set: `in:alerts`, `in:applications`, `in:businessApplications`, `in:connections`, `in:devices`, `in:operatingSystems`, `in:riskFactors`, `in:services`, `in:traffic`, `in:users`, `in:vulnerabilities`, `in:activity`.
- **Clause composition is whitespace-separated, NOT `AND`-joined.** The entity prefix leads, then space-separated predicates/keywords:
  - `in:devices timeFrame:"3 Hours"`
  - `in:devices after:2026-01-01T00:00:00 before:2026-01-02T00:00:00`
  - `in:alerts status:Open after:2026-01-01T00:00:00` (combining a field predicate + time keywords, all space-separated — consistent with the production poller default `in:alerts status:Open`).
- No `AND` keyword appears in any attested time-window example.

---

## 6. Field naming convention (Q5)

**camelCase. HIGH confidence.** Every attested field/keyword is camelCase, with zero snake_case counter-examples in any Armis-native source:
- `lastSeen`, `firstSeen`, `macAddress`, `ipAddress`, `operatingSystem`, `riskLevel`, `alertId`, `activityUUIDs`, `deviceIds`, `timeFrame`, `orderBy`, `time`.
- Sources that show snake_case (Elastic `armis.alert.time`/`last_seen`, Cortex XSOAR's `time_frame` *parameter*, Axonius `days_ago`) are **third-party-internal mappings/parameter names**, explicitly NOT the Armis wire field names. The Cortex XSOAR vs AQL distinction is explicit: the integration's `time_frame` parameter is translated to the AQL `timeFrame` predicate.

---

## 7. Confidence & residual uncertainty

**Overall: HIGH — confident enough to implement the recommended forms.** The two load-bearing forms (`after:`/`before:` for absolute/bounded, `timeFrame:"N unit"` for relative) are each corroborated by multiple independent sources, including two official Google Cloud Chronicle parser docs that contain *working production code* (not prose), the BlinkOps action doc with a verbatim bounded-range example, the Axonius adapter default, and the PyPI `armis` SDK examples.

Residual-uncertainty points (and the defensive build guidance for each):

| # | Point | Confidence | Defensive guidance |
|---|-------|------------|--------------------|
| R1 | `timeFrame` value phrasing: `"3 Hours"` (bare duration) is attested; `"Last 3 Hours"` (the string in `parity/armis.rs`) is NOT. | HIGH that predicate is real; MED that `"Last 3 Hours"` is invalid | Change test/demo strings to the bare-duration form `"3 Hours"`. If a `"Last N …"` form must be supported for UX, treat it as unverified and validate against a live tenant before shipping. |
| R2 | Exact ISO format for `after:`/`before:` — Chronicle uses `%Y-%m-%dT%H:%M:%S` (no `Z`/offset); BlinkOps uses date-only `YYYY-MM-DD`. Whether a trailing `Z`/`+00:00` is accepted is unconfirmed. | MED | Emit the timezone-naive `YYYY-MM-DDTHH:MM:SS` form (matches Chronicle production code) or date-only `YYYY-MM-DD` (matches BlinkOps). Do NOT append `Z` unless verified. Since prism normalizes to UTC internally, emit UTC wall-clock without the `Z` suffix. |
| R3 | Whether `after:`/`before:` values may be quoted. All attested examples are **bare/unquoted**. | MED-HIGH (bare) | Emit bare (unquoted) timestamps. |
| R4 | Alert `time:` as an AQL *filter predicate* (vs. field/sort). Confirmed as a field name (`orderBy=time`), not shown as a `time:>` filter. | MED | Filter alerts via the entity-agnostic `after:`/`before:`/`timeFrame:` keywords (attested) rather than a `time:`-comparison predicate (unattested). |
| R5 | Armis does not publish a complete public ASQ grammar (operator precedence, escaping, full keyword set). Query.AI explicitly notes not every key is filterable. | n/a | Keep the DTU's R-DTU-002 opaque-AQL treatment; do not build a parser that assumes a closed grammar. For prism's own constructed clauses, restrict to the attested `after:`/`before:`/`timeFrame:` keywords. |
| R6 | `in:` entity casing/case-sensitivity not formally documented. | MED-HIGH | Use lowercase plural (`in:devices`/`in:alerts`) — universal across all sources. |

**Defensive DTU recommendation:** the Armis DTU clone (which treats AQL as opaque, R-DTU-002) should — for fidelity — accept **all** the real temporal forms it might receive without choking: `timeFrame:"…"`, `after:…`, `before:…` (and tolerate both `YYYY-MM-DD` and `YYYY-MM-DDTHH:MM:SS`). Because the DTU stores AQL verbatim and routes only on the `in:<entity>` discriminator (already implemented in `routes/search.rs`), it does not need to parse the time clause — but the DTU's test vectors and any prism-*constructed* AQL should use the recommended canonical forms above, NOT `lastSeen:>"T"` and NOT `timeFrame:"Last 3 Hours"`.

**Architect / Armis-docs decision required: NO for the core forms.** The recommended `after:`/`before:` + `timeFrame:"N unit"` syntax is grounded enough to implement. The only items that warrant a quick live-tenant smoke (if a real Armis instance is reachable) are R1 (`"Last 3 Hours"` vs `"3 Hours"`) and R2 (`Z`-suffix acceptance) — both have safe defaults above, so they do not block implementation; they only refine it.

---

## 8. Convergence table (time-window forms)

| Source | Type | Relative form | Absolute / bounded form | Alert time field |
|--------|------|---------------|--------------------------|------------------|
| Google Cloud Chronicle — Armis devices parser | Official-Google (working code) | `in:devices timeFrame:"1 Hours"` | `in:devices after:{%Y-%m-%dT%H:%M:%S}` | — (orderBy=lastSeen) |
| Google Cloud Chronicle — Armis alerts parser | Official-Google (working code) | `in:alerts timeFrame:"1 Hours"` | `in:alerts after:{%Y-%m-%dT%H:%M:%S}` | `time` (orderBy=time; fields=…,time,…) |
| BlinkOps Armis Centrix Search action | Third-party connector | — | `in:devices after:2022-03-10 before:2023-05-15` | — |
| Axonius Armis adapter | Third-party connector | `in:devices timeFrame:"{days_ago} days"` | — | — |
| PyPI `armis` SDK | Community SDK | `in:devices timeFrame:"10 Seconds"`, `in:activity timeFrame:"1 Hours"` | — | — |
| Cortex XSOAR Armis | Third-party connector | `time_frame` param → translated to `timeFrame` | accepts `2019-10-10T12:22:00` / `2019-10-10` | alert payload `"time"` |
| Elastic Armis integration | Third-party schema | — | — | `armis.alert.time` (date) |
| Query.AI Armis Centrix | Third-party connector | (notes not all keys filterable) | — | — |
| **prism CURRENT (parity test) — under review** | internal | **`timeFrame:"Last 3 Hours"`** (value unattested) | — | — |
| **prism CURRENT (story AC-ARMIS-001) — under review** | internal | — | **`lastSeen:>"2026-01-01"`** (form unattested) | — |

Six external sources converge on `after:`/`before:` + `timeFrame:"N unit"`; zero sources support the `lastSeen:>"T"` comparison-operator form or the `"Last 3 Hours"` value phrasing.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 1 | Comprehensive reconstruction of Armis AQL temporal syntax (device last-seen, bounded range, alert time, naming) across official + third-party docs |
| Perplexity perplexity_search | 1 | Targeted lookup of `after:`/`before:` bounded-range syntax (surfaced BlinkOps + PyPI armis) |
| Tavily tavily_search | 2 | Independent cross-validation of timeFrame/lastSeen and `in:alerts` time-filter forms (surfaced Google Chronicle devices+alerts parsers, XSOAR event collector, Elastic) |
| Tavily tavily_extract | 1 | Full extraction of Google Chronicle Armis-alerts parser (verbatim `after:` Python + `timeFrame` curl + `orderBy=time`) |
| WebFetch | 2 | Direct retrieval of Chronicle devices parser (initial nav-only result noted) + BlinkOps Search action (verbatim bounded-range example + valid `in:` types) |
| Context7 | 0 | N/A — external HTTP API / query-language question, no library API surface to verify |
| Read (repo) | 4 | prior 2026-06-01 discriminator research, routes/search.rs, parity/armis.rs, story spec |
| Grep / Glob (repo) | 6 | Located timeFrame/lastSeen/after/before occurrences across crates + .factory; story + parity-test anchors |
| Training data | 0 areas | Not relied upon for any load-bearing claim — every syntax form is sourced to a cited live URL accessed 2026-06-05 |

**Total MCP tool calls:** 7 (1 Perplexity research + 1 Perplexity search + 2 Tavily search + 1 Tavily extract + 2 WebFetch) plus repo reads/greps.
**Training data reliance:** LOW — all temporal-syntax forms grounded in cited live sources (two Google Cloud Chronicle parser docs with working code, BlinkOps action doc, Axonius, PyPI armis, Cortex XSOAR, Elastic). No syntax was taken from model memory.
