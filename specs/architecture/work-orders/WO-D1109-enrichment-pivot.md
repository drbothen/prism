---
document_type: architecture-work-order
work_order_id: WO-D1109
title: "Enrichment-Pivot Feature Design — Design-Faithful Infusion Path"
status: approved
decision_id: D-1109
date: 2026-06-12
author: architect
consumers: [product-owner, story-writer]
decision: "Design-faithful plugin-backed infusion path (Option A)"
traces_to: [CAP-031, CAP-027, BC-2.06.019, BC-2.06.020]
blocking_story: S-DEMO-DTU-LIVE-SCENARIO-001-B
---

# WO-D1109: Enrichment-Pivot Feature Design

## Context and Decision Record

**Origin:** BPRL-P4-01 (MED) from the PR #185 adversary cascade on S-DEMO-DTU-LIVE-SCENARIO-001-B.
BC-2.06.019 PC-4's IOC-on-alert masking clause was found production-inert: no DTU generator stamps
IOC fields on alert records using real schema field names. The cyberint alerts-route filter matches
only a synthetic `_ioc_value` field that tests inject — a field that does not exist in the real
Cyberint API response shape.

**User decision (D-1109):** "We want the Design-faithful path. Infusion is flagship feature that
needs to be correct." This work-order records the architect design conclusions for that path and
provides the PO and story-writer with precise scope for each story.

This work moves OUT of PR #185 into the enrichment-pivot story chain. The BPRL-P4-01 finding
is deferred to the IOC-stamping story defined below (Story 2 in the chain).

---

## Question 1: Infusion Source Type for DTU-Backed Enrichment

### The Decision

**Recommendation: Option (a) — plugin-backed `.prx` infusion source calling the DTU HTTP API.**

This is the only design-faithful mechanism. Rationale follows.

### Evaluated Options

**Option (a): Plugin-backed `.prx` infusion calling DTU HTTP API**

CAP-031 already specifies plugin-backed (`type = "plugin"`) infusion sources that may make external
HTTP calls. The ThreatIntel and NVD DTU clones are HTTP services exposing routes that match the
infusion lookup pattern exactly (single-value lookup → structured response). The `.prx` WASM plugin
architecture (CAP-032, AD-019) is the correct mechanism for any infusion source that requires HTTP:
it is sandboxed, hot-reloadable, and credential-safe per AD-017.

Implementation path:
1. A `threatintel.infusion.toml` spec declares `type = "plugin"`, references a `threatintel-lookup.prx`
   WASM plugin, and declares fields: `threat_is_known_malicious` (Boolean), `threat_score` (Integer),
   `threat_source` (String).
2. The `.prx` plugin receives the IOC value (IP, domain, or hash) and calls the DTU clone's lookup
   HTTP endpoint, returning the parsed response.
3. In the demo scenario, the DTU clone's `fixture_registry` is pre-populated with scenario IOCs at
   construction time (BC-2.06.020 PC-1), so lookups return `FixtureKey::Malicious` → `threat_score >= 75`.
4. The `| enrich threat_intel on ioc_value` pipe stage (or `threat_score(ioc_value)` UDF invocation)
   resolves correctly against the live DTU endpoint.

Same pattern for NVD: `nvd.infusion.toml` with `type = "plugin"`, `nvd-lookup.prx`, fields:
`cvss_base_score` (Float), `cvss_severity` (String), `cvss_vector` (String).

This path represents the production architecture faithfully. The analyst demo shows real infusion UDFs
resolving against real HTTP services — not a mock, not a shortcut.

**Option (b): New first-class `http` infusion source type**

Rejected. CAP-031 already covers HTTP-backed infusions through the plugin path. Adding a first-class
`http` source type would require:
- CAP-031 spec amendment
- New `InfusionSource` variant in `prism-spec-engine`
- New credential model for inline HTTP auth (duplicating what the plugin ABI already handles)
- Additional Wave 5 scope

The plugin path achieves the same result with no spec amendment and respects the existing two-tier
architecture (TOML spec for declarative config; `.prx` for HTTP-capable sources). Option (b) would
be appropriate only if we needed to support HTTP lookups without any Rust/WASM code — but the demo
does not require that constraint.

**Option (c): JSON-lookup infusion from exported DTU seed data**

Rejected. This misrepresents the flagship. Exporting the DTU seed data to JSON and configuring
`type = "json_lookup"` would produce correct demo output but would not exercise the actual infusion
pipeline that analysts will use in production. The whole point of building ThreatIntel/NVD DTU clones
was to demonstrate the pivot workflow against live HTTP enrichment services. Substituting a static
JSON file defeats that purpose. Under the production-grade default (CLAUDE.md §1), a demo mechanism
that misrepresents the production path is a defect, not a shortcut.

### CAP-031 Amendment Assessment

No CAP-031 amendment is required. The plugin-backed `type = "plugin"` source type is already
specified. The stories below define `.infusion.toml` specs and `.prx` plugin implementations within
the existing framework.

The only architect note: BC-2.19.003 prohibits plugin-backed infusion UDFs in detection rule filter
expressions (to prevent blocking async HTTP in the DataFusion filter path). This prohibition applies
to `threat_score(ioc_value)` in detection rule filters but NOT in regular PrismQL queries. The
demo scenario uses the `| enrich threat_intel on ioc_value` pipe stage in a regular query — not a
detection rule. No conflict.

---

## Question 2: Story Decomposition and Sequencing

### Proposed Story Chain

Three new stories, named for the enrichment-pivot epic:

#### Story 1: S-DEMO-ENRICHMENT-PIVOT-001 — Infusion Engine Prerequisites (S-1.14-REDO unlock)

**Owner:** story-writer produces; implementer delivers.

**Scope:** Before plugin-backed infusion specs can be written or tested, the infusion engine must
be partially operational. S-1.14-REDO (Wave 5) implements the full engine including the plugin
bridge. However, the demo requires only the subset needed to register and execute a plugin-backed
infusion UDF:
- `InfusionLoader::parse` and `load_all` for `type = "plugin"` specs
- `PluginInfusionSource::enrich_single` / `enrich_batch` on the existing `InfusionSource` trait
  (implement the `unimplemented!()` stubs in `plugin_bridge.rs` via the trait — NOT a new free
  function `plugin_bridge::enrich_via_plugin`; D-1109 Ruling 2)
- DataFusion UDF registration wiring in `prism-query` for plugin-type infusion descriptors
- The `InfusionRegistry::is_api_backed()` check for BC-2.19.003

**Note for story-writer:** This story should be scoped as a FORWARD-SUBSET of S-1.14-REDO, not a
replacement. When S-1.14-REDO lands in Wave 5, it extends this subset with the full engine
(MMDB, CSV, JSON-lookup, three-tier cache, VP-048/049). The story spec should make the graduation
relationship explicit with `graduates:` frontmatter pointing at S-1.14. The Wave 5 S-1.14-REDO
story must be updated to reflect this — it now REDOs only what this story leaves unimplemented.

**Depends on:** S-1.14 partial-merge state (scaffolding is real; plugin bridge has `unimplemented!()` stubs).
**Blocks:** Story 2 (infusion specs), Story 3 (IOC stamping + pivot query).

#### Story 2: S-DEMO-ENRICHMENT-PIVOT-002 — ThreatIntel/NVD Infusion Specs (`.infusion.toml` + `.prx` plugins)

**Owner:** story-writer produces; implementer delivers.

**Scope:**
1. `specs/infusions/threatintel.infusion.toml` grounded against `prism-dtu-threatintel` route surface
   (lookup endpoint, request/response shape, credential reference per ADR-032).
2. `specs/infusions/nvd.infusion.toml` grounded against `prism-dtu-nvd` route surface (`/nvd/cves/{id}`
   response shape: `cve_id`, `metrics.cvss_metric_v31[0].cvss_data.base_score`, `.base_severity`).
3. `crates/prism-threatintel-infusion/src/lib.rs` — WASM guest cdylib (wasm32-wasip1) wrapped into
   a `.prx` Component via the Justfile `wasm-tools component new --adapt` pipeline (wasm-tools 1.248.0,
   same pattern as `build-plugin-crowdstrike-oauth2`). New crate placed at
   `crates/plugins/prism-threatintel-infusion/` — standalone workspace (NOT a workspace member;
   excluded from root `Cargo.toml` like `ocsf-complex-transforms`) to avoid host-target gate
   cross-compilation failures. Build via dedicated Justfile recipe analogous to
   `build-plugin-crowdstrike-oauth2`.
4. `crates/plugins/prism-nvd-infusion/src/lib.rs` — same pattern for NVD.
5. Integration test: with demo server running, `| enrich threat_intel(ioc.value)` applied to
   Cyberint alerts returns `threat_is_known_malicious = true` for scenario IOCs;
   `| enrich nvd(cve_id)` applied to Armis/CrowdStrike device records returns
   `cvss_base_score >= 7.0` for scenario CVEs.

**DTU grounding requirement (ADR-028/ADR-031):** The infusion TOML specs MUST be grounded against
the actual DTU clone route surface, not against assumed production API URLs. Use the DTU clone routes
as the source of truth for endpoint paths and response schemas. SAP-2 applies: adversary will
read `prism-dtu-threatintel/src/routes/` and `prism-dtu-nvd/src/routes/` to verify column parity.

**Depends on:** Story 1 (infusion engine plugin-bridge operational).
**Blocks:** Story 3.

#### Story 3: S-DEMO-ENRICHMENT-PIVOT-003 — IOC Stamping (Cyberint + CrowdStrike) and Demo Pivot Query

**Owner:** story-writer produces; implementer delivers.

**Scope — IOC stamping:**
1. Cyberint alerts DTU: add real-schema IOC fields to the `Alert` struct and fixture generator.
   The Cyberint `Alert` struct currently has no IOC fields (`types.rs` confirms: only `alert_id`,
   `title`, `severity`, `status`, `created_at`, `source`, `alert_type`, `affected_assets`). The
   real Cyberint API populates:
   - `ioc: { type: String, value: String }` (single IOC, inline)
   - `iocs: Vec<Ioc>` where `Ioc { ioc_type: String, value: String }` — already in `ThreatItem`
     but NOT on `Alert`; must be added per real API shape
   - `alert_data.*` typed observables: `alert_data.ip`, `alert_data.domain`, `alert_data.url`
   The fixture generator for scenario-enabled Cyberint clones must stamp scenario IOCs (from
   `ScenarioEntityCatalog.ioc_ips`, `ioc_domains`, `ioc_hashes`) onto alert records.
   The existing `_ioc_value` / `_ioc_type` synthetic-field filter in `alerts.rs` must be REPLACED
   with a filter that reads the real schema fields. This is the root cause of BPRL-P4-01.

2. CrowdStrike detections DTU: the `behaviors[]` array in CrowdStrike detection records contains
   native IOC fields: `ioc_type` (enum: `hash`, `domain`, `filename`, `registry`, `cmdline` — note:
   NO `ipv4`/`ipv6` per real API; IPs only appear on streaming NetworkAccesses shape, not detections),
   `ioc_value`, `ioc_source`, `ioc_description`. The `behaviors[]` array is on the Detection object,
   not on the host/device object. The fixture generator must populate `behaviors[0].ioc_type = "hash"`
   and `behaviors[0].ioc_value = <catalog.ioc_hashes[0]>` for scenario-enabled detections.
   BC-2.06.019 PC-4 / the per-sensor IOC-surface matrix (see Question 3 below) governs which
   records carry which IOC types.

3. Armis and Claroty: NO IOC stamping. This is permanent, on fidelity grounds (see Question 3).

**Scope — TOML spec alignment:**
4. The Cyberint sensor TOML spec (`crowdstrike.toml` equivalent for cyberint, if present) must
   declare the new `ioc`, `iocs[]`, and `alert_data.*` columns per the real-schema fields added above.
   SAP-2 applies.
5. The CrowdStrike detections TOML spec must declare `behaviors[].ioc_type`, `behaviors[].ioc_value`,
   `behaviors[].ioc_source`, `behaviors[].ioc_description` columns.

**Scope — demo pivot query:**
6. The canonical analyst pivot query for the capstone demo (T13) must be authored and validated
   against the enriched data:
   ```prismql
   FROM cyberint_alerts
   | where severity = "high"
   | enrich threat_intel(ioc.value)
   | where threat_is_known_malicious = true
   | sort threat_score desc
   | head 10
   ```
   And the CVE path:
   ```prismql
   FROM armis_devices
   | where has device_cves
   | enrich nvd(device_cves_first)
   | where cvss_base_score >= 7.0
   | sort cvss_base_score desc
   | head 10
   ```
   **Syntax note (D-1109 Ruling 1):** The implemented parser uses `enrich infusion(field_path)` —
   function-call form, not `ON` keyword form. `device_cves[0]` bracket-index is also not in
   `FieldPath`; the DTU fixture generator must project the first CVE ID into a scalar column
   `device_cves_first` (String) so the enrich field path is a plain dotted segment.
   See S-DEMO-ENRICHMENT-PIVOT-002/003 story specs for the TOML column declaration requirement.

   Both queries must execute against the demo server and return non-empty results at scenario
   stage >= 3 (Exfil, when ioc_ips + ioc_domains + ioc_hashes are visible per BC-2.06.019 PC-2).

**Depends on:** Story 2.
**Blocks:** T11 (demo prep) and T13 (capstone) in the demo objective sequence.

### Dependency Edges and Sequencing vs Demo Objective

Current demo sequence: T5 (PR #185, converging) → T6 (S-DEMO-MULTI-TENANT-DTU-001) →
T8 → capability-discovery block (S-5.02/03/04 + S-3.13, D-1107) → T11 → T13 capstone.

Proposed insertion:

```
T5 (PR #185) 
  → T6 (multi-tenant)
    → T8
      → capability-discovery block (D-1107)
        → S-DEMO-ENRICHMENT-PIVOT-001 (engine prereqs)
          → S-DEMO-ENRICHMENT-PIVOT-002 (infusion specs + plugins)
            → S-DEMO-ENRICHMENT-PIVOT-003 (IOC stamping + pivot query)
              → T11 → T13 capstone
```

Rationale for inserting after capability-discovery:
- Stories 1 and 2 require the infusion engine plugin bridge, which depends on S-1.14 partial-merge
  state. S-1.14 depends on S-WAVE5-PREP-01 (shipped, PR #138). No capability-discovery dependency
  exists, so Stories 1–2 could theoretically run in parallel with capability-discovery. However:
- Story 3 (IOC stamping) modifies Cyberint DTU routes and TOML sensor specs. If capability-discovery
  (S-5.02/03/04 + S-3.13) modifies the same sensor specs, there is a merge conflict risk.
- Sequential placement after the capability-discovery block is safer and does not materially delay
  T13 capstone, since the demo requires both capability discovery AND enrichment pivot to be
  operational.

If the capability-discovery block is delayed or descoped, the story-writer may elect to run
S-DEMO-ENRICHMENT-PIVOT-001/002 in parallel with T8 and converge before T11. The architect
does not prescribe that timing — this is story-writer judgment based on actual wave schedule.

---

## Question 3: PC-4 Amendment Shape (for PO)

BC-2.06.019 PC-4 currently reads as the IOC-on-alert masking clause. The PO should amend PC-4
to encode the per-sensor IOC-surface matrix below, replacing the current `_ioc_value`-based prose.

### Per-Sensor IOC-Surface Matrix

| Sensor | IOC-Surface Available? | Field Path(s) | IOC Types | Story That Implements | Fidelity Basis |
|--------|------------------------|---------------|-----------|----------------------|----------------|
| Cyberint alerts | YES | `ioc.value` (single), `iocs[].value` (list), `alert_data.ip`, `alert_data.domain`, `alert_data.url` | ip, domain, url, hash | S-DEMO-ENRICHMENT-PIVOT-003 | Real Cyberint API populates `ioc`/`iocs[]` and typed `alert_data.*` observables |
| CrowdStrike detections | YES | `behaviors[].ioc_value` | hash, domain, filename, registry, cmdline (NOT ipv4/ipv6 — IPs only on streaming NetworkAccesses shape, not detection records) | S-DEMO-ENRICHMENT-PIVOT-003 | Real FalconPy SDK / Detect API `behaviors[]` array carries `ioc_type`/`ioc_value`/`ioc_source`/`ioc_description` |
| CrowdStrike devices | NO | — | — | — | Device/host records do not carry IOC fields; IOCs live on detection/alert records |
| Armis alerts | NO (permanent) | — | — | — | Armis alert payloads are reference-only (deviceIds, activityUUIDs, endpoints). No structured IOC fields in real API. Stamping would fabricate fields that don't exist. SAP-2 / ADR-031 fidelity violation. |
| Claroty xDome alerts | NO (permanent) | — | — | — | Claroty alerts carry IP addresses only as free text in `alert_name`; no structured IOC schema. Stamping structured IOC fields would violate DTU=True-DTU principle (ADR-031). |

### PC-4 Amended Text (for PO to encode in BC-2.06.019)

Replace the current PC-4 clause with language equivalent to:

> **Postcondition 4 (amended) — Per-Sensor IOC-Surface Masking via Per-Sensor IOC-Surface Matrix**
>
> The `StageMask.ioc_*` fields govern IOC visibility for sensors whose real API surfaces native IOC
> data. Which sensors carry IOC fields is governed by the Per-Sensor IOC-Surface Matrix defined in
> WO-D1109 (and reproduced in the story spec for S-DEMO-ENRICHMENT-PIVOT-003). The matrix
> is the authoritative source; BC-2.06.019 PC-4 references it by name.
>
> Sensors with IOC surface (Cyberint, CrowdStrike detections): alert/detection records referencing
> catalog IOC values in their native schema fields (see matrix for field paths) are filtered per
> the stage mask. When `ioc_hashes = false`, detection records referencing `catalog.ioc_hashes`
> values in `behaviors[].ioc_value` are withheld. When `ioc_ips = false` / `ioc_domains = false`,
> Cyberint alerts with matching `ioc.value` / `iocs[].value` / `alert_data.ip` / `alert_data.domain`
> are withheld.
>
> Sensors without IOC surface (Armis, Claroty): these sensors have no structured IOC fields in their
> real API responses. IOC masking does not apply. These sensors are permanently excluded from the
> per-sensor IOC-surface matrix. Fabricating IOC fields on Armis/Claroty records would violate the
> DTU=True-DTU fidelity principle (ADR-031). The existing `_ioc_value` synthetic-field filter
> in the Cyberint alerts route is removed in S-DEMO-ENRICHMENT-PIVOT-003 and replaced with
> real-schema field matching.

---

## Question 4: Route x Entity Coverage Matrix Codification

### Problem

The process gap BPRL-P4-01 revealed: BC-2.06.019 PC-4 specified IOC masking behavior without
enumerating which routes on which DTU clones are affected by the mask. The `_ioc_value` filter
only appeared in `prism-dtu-cyberint/src/routes/alerts.rs`, but the CrowdStrike detections route
was never mentioned. A future stage-mask story could miss routes for the same reason.

### Recommended Mechanism

**Inline in the BC that introduces or modifies stage masking (not a separate standing requirement).**

The architect does not recommend a new BC section type for this. The overhead of maintaining a
separate index is not justified for the current scale (4 sensor DTU clones, 2 with IOC surface).
Instead, every BC or story spec that introduces or modifies a `StageMask` field MUST include a
"Route Coverage Table" section in the format:

```
## Route Coverage Table (for this StageMask field)

| StageMask Field | DTU Clone | Route File | Route Path | Filter Logic |
|----------------|-----------|-----------|-----------|-------------|
| ioc_hashes | prism-dtu-cyberint | routes/alerts.rs | GET /api/v1/alerts | Filter records where iocs[].value IN catalog.ioc_hashes when mask=false |
| ioc_hashes | prism-dtu-crowdstrike | routes/detections.rs | GET /detects/queries/detects/v1 | Filter records where behaviors[].ioc_value IN catalog.ioc_hashes when mask=false |
```

This table is required in:
- BC-2.06.019 PC-4 (amended — PO responsibility)
- Story specs for S-DEMO-ENRICHMENT-PIVOT-003 and any future story that modifies StageMask fields

The story-writer should add a "Route Coverage Table required" standing note to the IOC-stamping
story template.

**Why not a separate standing story template requirement?** The STORY-INDEX and BC-INDEX already
require traceability. Adding a third index for route coverage is over-engineering for the current
scope. The inline table in each BC is sufficient and is the correct locus for this information —
the BC is read at implementation time, which is exactly when the implementer needs to know which
routes to update.

---

## Question 5: Cyberint `_ioc_value` Filter Disposition

### Recommendation: Remove in S-DEMO-ENRICHMENT-PIVOT-003, Replace with Real-Schema Fields

The `_ioc_value` / `_ioc_type` synthetic-field filter in
`crates/prism-dtu-cyberint/src/routes/alerts.rs` is the direct cause of BPRL-P4-01. It was
introduced because the `Alert` struct lacked real IOC schema fields. Once S-DEMO-ENRICHMENT-PIVOT-003
adds those fields, the synthetic filter has no purpose and must be removed in the same story.

The replacement logic is:
```rust
// OLD (synthetic field, non-existent in real API):
if let Some(ioc_value) = rec.get("_ioc_value").and_then(|v| v.as_str()) { ... }

// NEW (real schema fields):
// Filter based on rec.ioc.value, rec.iocs[].value, rec.alert_data.ip/domain
// when the corresponding mask bit is false and the value matches a catalog IOC.
```

This is a SAME-STORY removal — not a follow-up story. The synthetic filter and the real-schema
filter cannot coexist in the same route handler; the implementer removes the synthetic filter
and adds the real-schema filter in one atomic change within S-DEMO-ENRICHMENT-PIVOT-003.

**Do not keep the synthetic-field filter as an interim.** The production-grade default (CLAUDE.md §1)
prohibits shipping a known-incorrect filter with a plan to replace it later. Story 3 ships the
correct filter; the synthetic one is deleted in that same story.

---

## ARCH-INDEX Amendment Assessment

No ARCH-INDEX version bump or new ADR is required for this work-order. The design conclusions
are:

1. Plugin-backed infusion (Option A) fits within the existing CAP-031 specification with no
   amendment to that capability definition.
2. The per-sensor IOC-surface matrix is BC content (PO domain), not architecture content.
3. The route coverage table recommendation is a story/BC template convention (story-writer domain).

An ADR would be warranted only if we were making a novel architectural decision. The decision here
is to use the existing plugin-backed infusion path as designed — which is not novel; it follows the
established two-tier sensor/infusion architecture (AD-020, CAP-031, CAP-032). No ADR.

---

## Summary for PO and Story-Writer

### Recommended Infusion Mechanism

Plugin-backed `.prx` infusion (CAP-031 `type = "plugin"`) calling the ThreatIntel/NVD DTU HTTP
API. This is the only design-faithful path. No CAP-031 spec amendment needed.

### Proposed Story List

| Story ID | Title | Key Deliverable | Depends On | Blocks |
|----------|-------|-----------------|------------|--------|
| S-DEMO-ENRICHMENT-PIVOT-001 | Infusion Engine Plugin-Bridge Prereqs | `PluginInfusionSource::enrich_single/enrich_batch` implemented (trait path, not free fn); InfusionLoader for `type = "plugin"` specs; DataFusion UDF registration for plugin infusions | S-1.14 partial-merge state | Stories 002, 003 |
| S-DEMO-ENRICHMENT-PIVOT-002 | ThreatIntel/NVD Infusion Specs and Plugins | `threatintel.infusion.toml`, `nvd.infusion.toml` + `.prx` plugins (cdylib → wasm-tools pipeline, crates/plugins/ excluded from workspace); integration test against demo server | Story 001 | Story 003 |
| S-DEMO-ENRICHMENT-PIVOT-003 | IOC Stamping + Demo Pivot Query | Cyberint Alert struct adds real IOC fields; CrowdStrike detections `behaviors[].ioc_*` stamped; `_ioc_value` synthetic filter removed + replaced; canonical pivot queries validated | Story 002 | T11, T13 capstone |

### Demo Objective Sequencing

Insert after capability-discovery block (D-1107):
`T5 → T6 → T8 → capability-discovery → PIVOT-001 → PIVOT-002 → PIVOT-003 → T11 → T13`

### BC-2.06.019 PC-4 Amendment (for PO)

Replace current PC-4 with a per-sensor IOC-surface matrix clause referencing the table in
Question 3 above. The matrix is the authoritative enumeration of which sensors carry IOC fields,
which field paths, which IOC types, and which stories implement the stamping. Armis and Claroty
are permanently excluded on ADR-031 fidelity grounds.

### Cyberint `_ioc_value` Filter

Remove in S-DEMO-ENRICHMENT-PIVOT-003 atomically with adding the real-schema filter. No interim
state. The synthetic field does not exist in the real Cyberint API.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-12 | architect | Initial work-order. D-1109 decision record. Answers to all five design questions. Three-story chain defined. |
| 1.1 | 2026-06-12 | architect | D-1109 micro-rulings (follow-up adjudication). Ruling 1: enrich syntax is function-call form `enrich infusion(field_path)` — grammar doc `ON` form was brownfield-era extraction, superseded by S-1.14 implementation (ast.rs AD-020/S-1.14 citation). `device_cves[0]` bracket-index not supported by FieldPath; re-shaped to scalar `device_cves_first` column projected by DTU fixture generator. Grammar doc prismql-grammar.md §6 pipe-stage table + EBNF §stage to be amended by story-writer before PIVOT-001 story spec is written (not by architect — grammar doc is BA/story-writer domain). Ruling 2: plugin bridge implements via existing `PluginInfusionSource::enrich_single/enrich_batch` trait methods — no new free function `plugin_bridge::enrich_via_plugin`. Ruling 3: new WASM guest crates under `crates/plugins/` namespace, out-of-workspace (standalone `[workspace]` table in each crate's Cargo.toml, excluded from root Cargo.toml — same pattern as `ocsf-complex-transforms`). Ruling 4: canonical toolchain is cdylib → wasm32-wasip1 → `wasm-tools component new --adapt` (Justfile, wasm-tools 1.248.0); `cargo-component` phrase removed from Story 2 scope. WO updated to reflect all four rulings. |
