---
document_type: research-artifact
research_type: general
topic: "Armis Centrix ASQ/AQL entity-discriminator syntax (devices vs alerts)"
slug: armis-aql-discriminator-syntax
date: "2026-06-01"
producer: research-agent
status: complete
confidence: HIGH
triggered_by:
  - "BC-2.16.013 §Canonical Test Vectors vs implementation discriminator conflict"
  - "S-DEMO-ARMIS-AQL-001 (in-progress) AQL search endpoint fidelity"
  - "ADR-031 §D8-a (DTU=true-DTU fidelity principle)"
discriminator_in_conflict:
  bc_2_16_013: "in:devices / in:alerts"
  implementation: "in:type=Device / in:type=Alert"
recommended_convention: "in:devices / in:alerts"
recommendation_target: "implementation must change to match BC (BC is already correct)"
architect_adjudication_required: false
related_artifacts:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.008-armis-bearer-aql.md"
  - ".factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md"
  - ".factory/semport/poller-coaster/poller-coaster-broad-sweep.md"
  - ".factory/stories/S-DEMO-ARMIS-AQL-001-armis-aql-search-endpoint-fidelity.md"
  - "crates/prism-sensors/specs/armis.sensor.toml"
  - "crates/prism-dtu-armis/src/routes/devices.rs"
  - "crates/prism-spec-engine/tests/parity/armis.rs"
---

# Research: Armis Centrix ASQ/AQL Entity-Discriminator Syntax (Devices vs Alerts)

**Date:** 2026-06-01
**Type:** General (technology/implementation fidelity)
**Confidence:** HIGH (3 independent external sources + the project's own production-poller ingestion all converge)
**Bottom line:** The real Armis syntax is **`in:devices` / `in:alerts`** (the `in:<entity>` form). The implementation's `in:type=Device` / `in:type=Alert` form is **not used by the real Armis API, the real 1898 & Co production poller, or any documented third-party Armis integration**. BC-2.16.013 is correct; the implementation must change.

---

## 1. The Question

Prism's Armis DTU clone forwards an AQL/ASQ query string to filter results by entity type. Two conventions are in conflict across project artifacts:

| Artifact | Discriminator used |
|----------|-------------------|
| BC-2.16.013 §Canonical Test Vectors (and §Postconditions §1 Armis bullets) | `in:devices` / `in:alerts` |
| Implementation: `S-DEMO-ARMIS-AQL-001` story body, DTU harness tests (`tests/harness_tests.rs`, `tests/ac_1_aql_capture_and_device_list.rs`), `routes/dtu.rs` doc-comment example | `in:type=Device` / `in:type=Alert` (also `in:type=switch`, `in:type=plc` in fixtures) |

This matters because ADR-031 (DTU = true-DTU fidelity) requires the DTU clone to accept the SAME query shape the real Armis API accepts, so AQL push-down fidelity is genuine and not a synthetic call pattern.

---

## 2. Canonical Answer (HIGH confidence)

### 2.1 Entity discrimination is the `in:<entity>` prefix

In the Armis Centrix / Asset Intelligence Platform v1 search API, the entity type is selected **inside the AQL/ASQ query string** by a leading `in:<entity>` clause with a **bare, lowercase, plural** entity name. No `=` sign, no `type` field:

- Devices → query string begins with **`in:devices`**
- Alerts → query string begins with **`in:alerts`**

Additional field predicates follow, space-separated, e.g. `in:devices name:(system)`, `in:alerts alertId:(57)`, `in:alerts status:Open`, `in:devices timeFrame:"Last 3 Hours"`.

The full set of Armis entities follows the same pattern: `in:activity`, `in:auditLog`, `in:riskFactors`, `in:connections`, `in:devices`, `in:vulnerabilities`, `in:users`, `in:applications`. (Note camelCase for multi-word entities like `auditLog`/`riskFactors`, lowercase plural for the rest.)

### 2.2 Canonical endpoint + parameter

- **Endpoint:** `GET /api/v1/search/` (v1 tenant API, host `https://<tenant>.armis.com`)
  - Both `GET /api/v1/search` (no trailing slash) and `GET /api/v1/search/` are observed across sources; the real poller-coaster SDK uses `GetSearch`, and the community example uses the trailing-slash form. The discriminator question is independent of the trailing-slash detail.
- **Query parameter name:** **`aql`** (e.g. `?aql=in:alerts&limit=10`)
- Optional params: `from` / `length` (or `limit`) for pagination, `fields` for field selection, `includeTotal`, `includeSample`, `orderBy`.

### 2.3 Forms that are NOT real Armis syntax

None of the surveyed sources — including Armis' own developer/community docs, three independent third-party connectors, AND the project's own ingested production poller — use any of:

- `in:type=Device` / `in:type=Alert` ❌ (the implementation's current form)
- `in:type=switch` / `in:type=plc` ❌ (the DTU fixture strings)
- `in:aqlSearch` / `aqlSearch=` ❌ (these appear only as connector-internal UI labels/variable names, never as Armis API constructs)

`in:type=…` does not appear anywhere in the public record. The only place `type` appears in real Armis AQL is as a **predicate field** on a record (e.g. a device's `type` column), never as the entity selector after `in:`.

---

## 3. Evidence — Project's Own Ground Truth (DECISIVE)

The strongest evidence is internal: the project ingested the **real 1898 & Co Armis Centrix production poller** ("poller-coaster"), which uses the actual `github.com/1898andCo/armis-sdk-go/v2` SDK against the live Armis Centrix Search API. This is the canonical reference ADR-031 §D8-a itself cites as "the way the production poller does it."

Source: `.factory/semport/poller-coaster/poller-coaster-broad-sweep.md`

The poller's single API operation is `GetSearch(ctx, aql, includeSample, includeTotal)` (§3). All 7 data sources go through `GetSearch` with these **verbatim default AQL strings** (§4.1–4.7):

| Data source | Real production AQL (poller-coaster §4) |
|-------------|-----------------------------------------|
| Alerts | **`in:alerts status:Open`** |
| Activities | `in:activity` |
| Audit Logs | `in:auditLog` |
| Risk Factors | `in:riskFactors` |
| Connections | `in:connections` |
| **Devices** | **`in:devices`** |
| Vulnerabilities | `in:vulnerabilities` |

This is definitive: the real production poller uses `in:devices` and `in:alerts`. It does **not** use `in:type=Device`/`in:type=Alert` anywhere.

Corroborating internal evidence — the project's own spec-engine parity test already encodes the correct form:
- `crates/prism-spec-engine/tests/parity/armis.rs` (AC-010 AQL-forwarding sub-case) sets the filter to **`"in:devices timeFrame:\"Last 3 Hours\""`** — i.e., the codebase already has the correct `in:devices` convention in its pipeline-level test, while the DTU-level harness tests use the incorrect `in:type=…` form. The conflict is internal-to-the-codebase as well as spec-vs-implementation.

---

## 4. Evidence — Independent External Corroboration

Three independent third-party sources confirm the `in:devices`/`in:alerts` form on `/api/v1/search?aql=`:

1. **Swimlane Turbine "Armis Centrix" connector** — documents explicit AQL examples `in:devices name:(system)` and `in:alerts alertId:(57)`, and describes the HTTP interface as `GET /api/v1/search/` with a required string parameter `aql`. (https://docs.swimlane.com/connectors/armis-centrix — accessed 2026-06-01 via WebFetch; the page escapes the colon, but the canonical form is `in:devices` / `in:alerts`.)

2. **Query.AI "Armis Centrix" connector** — documents the search routes literally as `/search+in:devices`, `/search+in:users`, `/search:in:vulnerabilities`, confirming the `in:<entity>` selector on the `search` endpoint. States the connector "generates ASQ/AQL on the user's behalf." (https://docs.query.ai/docs/armis-centrix — accessed 2026-06-01 via Tavily.)

3. **Axonius Armis adapter** — documents the default AQL it sends as `in:devices timeFrame:"{days_ago} days"`. This is the SAME shape as prism's existing parity test, independently confirming `in:devices` + a `timeFrame:` predicate. (https://docs.axonius.com/docs/armis — accessed 2026-06-01 via Tavily.)

Additional supporting context:
- **Cortex XSOAR / Demisto Armis integration** — commands `armis-search-devices-by-aql` and `armis-search-alerts-by-aql-string`, each taking an `aql_string` argument. Confirms AQL is the query language for both devices and alerts; the connector prepends the `in:<entity>` clause internally. (https://github.com/demisto/content/blob/master/Packs/Armis/Integrations/Armis/README.md — accessed 2026-06-01 via Tavily.)
- **Armis developer community** — concrete v1 call `https://integration-partner.armis.com/api/v1/search/?aql=in:alerts&limit=10`, confirming both the endpoint path, the `aql` parameter name, and the `in:alerts` selector. (https://dev.armis.com/discuss — cited via Perplexity research.)
- **Armis dev portal glossary** — defines ASQ (Armis Standard Query) as the Centrix query language; "AQL" in third-party docs is the same language under an older/external name. (https://dev.armis.com/docs/glossary — cited via Perplexity research.)

**Note on API generations:** Newer Armis APIs (Intelligence Center `https://ic.armis.com/api/v1/device/_search`, and v3 `POST https://api.armis.com/v3/assets/_search`) do NOT use the `in:` AQL string — they encode the entity in the path and filter via structured params/JSON. Armis states v1/v2 are NOT deprecated. Since prism's poller-coaster reference and DTU model the **v1 AQL search** path, the `in:devices`/`in:alerts` form is the correct target. (https://dev.armis.com/docs/migration-from-api-v1, https://docs.ic.armis.com/docs/documentation_filtering — cited via Perplexity research.)

---

## 5. Convergence Table

| Source | Devices selector | Alerts selector | Endpoint | Param |
|--------|------------------|-----------------|----------|-------|
| poller-coaster (real 1898 production poller) | `in:devices` | `in:alerts status:Open` | `GetSearch`/`/api/v1/search` | aql |
| prism existing parity test (`parity/armis.rs`) | `in:devices timeFrame:…` | (n/a in test) | `/api/v1/search`(target) | aql |
| Swimlane Armis Centrix connector | `in:devices name:(system)` | `in:alerts alertId:(57)` | `GET /api/v1/search/` | aql |
| Query.AI Armis Centrix connector | `/search+in:devices` | (in:alerts implied) | `/search` | aql |
| Axonius Armis adapter | `in:devices timeFrame:"…"` | (n/a) | search | aql |
| Armis community example | (in:devices implied) | `in:alerts` | `/api/v1/search/?aql=` | aql |
| **prism implementation (CURRENT — WRONG)** | **`in:type=Device`** | **`in:type=Alert`** | `/api/v1/search` | aql |

Six sources agree on `in:devices`/`in:alerts`. The prism implementation is the lone outlier.

---

## 6. Recommendation

### 6.1 Standardize on `in:devices` / `in:alerts`

**BC-2.16.013 is already correct.** Its §Canonical Test Vectors and §Postconditions §1 Armis bullets use `in:devices` / `in:alerts`, matching the real Armis API and the real production poller. **Do NOT change the BC.** (Per CLAUDE.md §Source-of-Truth Precedence #7, the SPEC wins on a code-vs-spec conflict, and here the spec also happens to match external ground truth — the strongest possible position.)

**The implementation must change** to bring `in:type=Device`/`in:type=Alert` into alignment with `in:devices`/`in:alerts`. This is a code fix, routed to `vsdd-factory:implementer` via the orchestrator, in scope of the in-progress `S-DEMO-ARMIS-AQL-001` story (which is the natural home — it is currently introducing the `/api/v1/search` route and is not yet merged). Fixing it now avoids shipping a non-fidelity discriminator that would violate ADR-031 §D8-a on the very story that exists to establish AQL fidelity.

### 6.2 Concrete code/spec changes required

Code (`crates/prism-dtu-armis/`) — change the discriminator pattern-match and all fixture/test strings from `in:type=…` to `in:<entity>`:

| File | Current | Target |
|------|---------|--------|
| `S-DEMO-ARMIS-AQL-001` story body (AC-002/AC-003, Tasks 8, handler routing) | `in:type=Device` / `in:type=Alert` | `in:devices` / `in:alerts` |
| `src/routes/search.rs` (to be created) handler routing logic | match on `in:type=Device` / contains `Alert` | match on `in:devices` / `in:alerts` (substring `in:alerts` / `in:devices`) |
| `tests/harness_tests.rs`, `tests/ac_1_aql_capture_and_device_list.rs`, `tests/reset_state_invariants.rs` | `in:type=switch`, `in:type=plc` | `in:devices` (+ optional real predicate, e.g. `in:devices type:(switch)` if a device-type filter is the test intent) |
| `src/routes/dtu.rs` doc-comment example (`"in:type=switch"`) | `in:type=switch` | `in:devices` |
| `docs/demo-evidence/S-6.10/*` AQL-capture demos (if re-recorded) | `in:type=switch` | `in:devices` |

> Subtlety worth flagging to the implementer: `in:type=switch` was conflating two distinct AQL concepts. `in:<entity>` selects the COLLECTION (devices); a device's *category/type* (switch, plc) is a **predicate field** on a device record, e.g. `in:devices type:(switch)` or `in:devices category:(switch)`. The current fixture string `in:type=switch` is neither valid Armis selector syntax nor a valid predicate. If the test intent is "filter to switches," the real form is `in:devices type:(switch)`. The implementer should confirm intent and pick the correct real form rather than a 1:1 token swap.

TOML (`crates/prism-sensors/specs/armis.sensor.toml`) — when `S-DEMO-ARMIS-AQL-001` sets `path_template = "/api/v1/search"` and forwards `${query.filter.aql}`, the default/constructed AQL for the devices table must be `in:devices` and for the alerts table `in:alerts` (matching BC-2.16.013 test vectors and poller-coaster defaults). No `in:type=…` should appear in the spec.

Because the DTU treats AQL as opaque (R-DTU-002 — stored verbatim, never parsed), the pattern-match in the DTU handler is a fidelity convenience only; using the real `in:devices`/`in:alerts` substrings keeps the DTU's routing aligned with what prism (and the real Armis API) actually send.

### 6.3 Reconciliation already anticipated

BC-2.16.013 v1.21 changelog (2026-06-01) explicitly recorded this exact divergence and deferred it: *"AQL discriminator strings (`in:devices`/`in:alerts`) preserved as-specified in BC; story-vs-BC discriminator convention divergence (`in:type=Device`/`in:type=Alert` in implementation) reported for separate reconciliation — not fixed here."* This research IS that separate reconciliation. The disposition is: **BC stands, implementation conforms.**

---

## 7. Confidence & Limitations

**Confidence: HIGH.** The determination rests on the project's own ingested production poller (the canonical reference ADR-031 cites) PLUS three independent external connectors PLUS an Armis community example — six sources, fully convergent, zero dissent for `in:devices`/`in:alerts`. The outlier `in:type=…` form has **zero** supporting sources.

**Limitations (do not affect the discriminator answer):**
- Armis does not publish a complete public ASQ grammar (operators, boolean precedence, escaping). Not needed here — the DTU treats AQL as opaque per R-DTU-002.
- Trailing-slash on `/api/v1/search` vs `/api/v1/search/` varies by source; this is a separate fidelity detail (cf. the Claroty trailing-slash story S-DEMO-CLAROTY-TRAILING-SLASH-001) and is orthogonal to the discriminator. The poller-coaster SDK abstracts the path via `GetSearch`; recommend matching whatever the `armis-sdk-go/v2` SDK emits if exact-path fidelity is later required.
- Entity-name casing for the `alerts`/`devices` selectors is confirmed lowercase plural; case-sensitivity of the Armis parser is not publicly documented, but every observed example uses lowercase, so lowercase is the safe and correct choice.

**Architect adjudication required: NO.** The answer is mechanically determinable from the canonical production reference; no cross-component architectural tradeoff exists. Routing is: orchestrator → `vsdd-factory:implementer` (code fix in scope of S-DEMO-ARMIS-AQL-001). BC requires no change.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research | 1 | Comprehensive reconstruction of Armis ASQ/AQL device-vs-alert syntax, endpoint, and param from public docs/connectors/community |
| Tavily tavily_search | 1 | Independent cross-validation of `in:devices`/`in:alerts` + `/api/v1/search?aql=` against a second search index |
| WebFetch | 1 | Direct retrieval of Swimlane Armis Centrix connector page for exact AQL example + endpoint/param |
| Context7 | 0 | Not applicable — no library API surface to verify; this is an external HTTP API/query-language question |
| Read (repo) | 5 | armis.sensor.toml, BC-2.16.013, BC-2.01.008, S-DEMO-ARMIS-AQL-001 story, parity/armis.rs, routes/devices.rs, poller-coaster ingestion |
| Grep (repo) | 4 | Located all `in:type` / `in:devices` / `in:alerts` occurrences across crates + .factory; ADR-031 §D8-a anchors |
| Glob (repo) | 6 | Located BC/ADR/story/route files and ingestion artifacts |
| Training data | 0 areas | Not relied upon for any claim — all findings sourced from web tools or repo artifacts |

**Total MCP tool calls:** 3 (1 Perplexity research, 1 Tavily search, 1 WebFetch) + repo reads/greps.
**Training data reliance:** LOW — every load-bearing claim is grounded in either the project's ingested production poller (`poller-coaster-broad-sweep.md`) or a cited live URL accessed 2026-06-01. No version numbers or API shapes were taken from model memory.
