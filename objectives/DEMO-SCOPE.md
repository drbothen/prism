---
document_type: demo-scope
level: ops
producer: state-manager
version: "1.8"
timestamp: 2026-08-21T12:00:00Z
project: prism
---

# DEMO SCOPE — Multi-Client SOC-Analyst Live Demo (Authoritative)

> **THIS IS THE SINGLE SOURCE OF TRUTH** for "everything we are including in the demo."
> Referenced by SESSION-HANDOFF.md §ACTIVE OBJECTIVE and `.factory/objectives/multi-client-soc-demo-tasks.md`.
> A zero-context restart MUST read this file to understand what the demo includes, what is already built, and what the honest gaps are.

> **READ ORDER NOTE (cold-resume agents):** STATUS values in this document track build progress (MERGED / SCOPED-NOT-BUILT). For the AUTHORITATIVE current pipeline position, next action, and develop HEAD, the source-of-truth is **STATE.md frontmatter** + **SESSION-HANDOFF.md §RESUME SNAPSHOT**. This document is the demo SCOPE and NARRATIVE reference — not the live pipeline position.

---

## v1 FIRST RELEASE — Claroty xDome (Authoritative, 2026-08-21)

> **GOVERNING DECISION (D-2264, human-directed 2026-08-21).** This section supersedes the broader multi-sensor SOC-analyst demo framing below for the purposes of v1. The multi-sensor demo remains the long-term product vision; for v1, scope is intentionally narrowed to Claroty xDome.

### v1 Target

The **v1 FIRST RELEASE** is defined as: a fully-working, stable **Claroty xDome** sensor, end-to-end.

### Validation Approach

Validation is performed against the **REAL Claroty xDome tenant (live API)** — true live acceptance. AD-017 opaque-credential path applies: credentials never transit AI context.

### v1 Claroty-xDome Analyst-Path Scope

The complete Claroty-xDome SOC-analyst path:

1. **Client + sensor onboarding** — onboarding a client and their xDome sensor
2. **OCSF field-mapping correctness** — correct normalization of xDome device and audit_log data to OCSF shapes (delivered: COERCION Stage 1 via S-ADR058-OCSF-COERCION-001 / ROUTING Stage 2 via S-ADR058-OCSF-ROUTING-001)
3. **All query shapes working** — devices, audit_logs, all supported PrismQL query patterns
4. **Push-down working correctly** — filter push-down, time-box push-down, predicate push-down to the xDome API
5. **SOC-analyst Q&A loop** — what questions an analyst can ask and prism returning the data needed to answer
6. **Stability under real use** — validated against the real xDome tenant (live API)

### Release Gate

Immediately after **S-ADR058-OCSF-ROUTING-001** merges (completing Claroty OCSF correctness: COERCION Stage 1 + ROUTING Stage 2), the pipeline shifts to comprehensive live end-to-end xDome validation as the **v1 release gate**.

### Explicitly DE-SCOPED to POST-v1

The following stories are NOT v1 blockers and are de-scoped to post-v1:

- **S-OCSF-FIDELITY-CROWDSTRIKE-001** — CrowdStrike OCSF fidelity stub
- **S-OCSF-FIDELITY-CYBERINT-001** — Cyberint OCSF fidelity stub
- **S-OCSF-FIDELITY-ARMIS-001** — Armis OCSF fidelity stub
- **S-ADR058-DTU-PARITY-MIGRATION-001** — DTU parity test migration (already parked post-v1)

These stories remain in the backlog for post-v1 work. They are not v1 blockers.

---

## Status Legend

- **MERGED** — code on `develop`; spec/contracts active
- **IN CONVERGENCE** — code complete locally; PR-LEVEL adversarial cascade in progress; not yet merged
- **SCOPED-NOT-BUILT** — designed and planned; no code yet

---

## The Frame

Prism runs as a **per-analyst MCP server** inside Claude Code. The analyst works multiple client orgs, queries their security sensors in PrismQL; prism fans out to sensor APIs **in parallel** (DTU clones standing in for real vendors for the demo), normalizes to OCSF, returns a unified result.

**Execution model:** Prism is a single process with a multi-threaded tokio runtime. A query fans out to multiple sensors/clients IN PARALLEL (bounded by MAX_FANOUT_CONCURRENCY=10, nested under HTTP_SEMAPHORE_PERMITS=200). Concurrent queries are NOT serialized by any lock (`PrismServer::query` takes `&self`; ArcSwap lock-free config reads). The only sequential aspect is stdin message framing in the stdio transport — a transport/client characteristic, NOT engine serialization.

**READ-ONLY.** Write-back / actions (TDE — Threat Detection Engineer workflow) is DEFERRED. Reason: requires the absent `prism-operations` crate + wiring the dead write path (E-SENSOR-070 / TODO W3-FIX-S307-001).

---

## MERGED — Built and Merged on `develop`

### 6-Sensor DTU Fleet (behavioral clones of real vendor APIs)

Four **operational sensors**:
- **CrowdStrike** — devices, hosts, alerts, detections
- **Cyberint** — alerts
- **Claroty** — devices, audit_log
- **Armis** — devices/hosts, alerts/detections

Two **enrichment sources**:
- **ThreatIntel** — IOC lookup (threat intelligence)
- **NVD** — CVE lookup (vulnerability database)

These are behavioral HTTP clones of real vendor APIs. The demo queries them instead of live vendor endpoints.

### Per-Client Distinct Seeded Data (Story A / BC-2.06.018)

Merged as **PR #181** (`develop@c287b00d`).

Each client org's sensors return **different** devices/alerts. Org isolation holds: `devices(OrgA) ∩ devices(OrgB) = ∅`. This is what makes it a believable **multi-client** demo — not just routing, but genuinely different data per client.

BC-2.06.018 v1.6 is **active** (promoted at merge per POL-14).

### Review-Cycle Hardening

Three fix-PRs that hardened the fleet for demo quality:
- **PR #182** (`develop@939f36ce`) — DTU fleet lane: DTU schema parity, Postcondition-5 propagation, Armis isolation, CrowdStrike parity
- **PR #183** (`develop@f88b10e3`) — query-core lane: E-QUERY taxonomy splits, watchdog grace period, cache/hot-reload fixes
- **PR #184** (`develop@c200d5a2`) — mcp-boot lane: fail-closed write audit, reload_config WriteTool, capability fields, security polish

All 43 CI checks pass on `develop@939f36ce`.

---

## MERGED — T5 (PR #185, S-DEMO-DTU-LIVE-SCENARIO-001-B)

> **MERGED develop@7fd35b77 (2026-06-13; D-1139). BC-2.06.019 v1.7 + BC-2.06.020 v1.6 ACTIVE (POL-14). T5 DONE.**

### Deterministic 5-Stage Attack Progression Over Wall-Clock Time

**Stages:** Baseline → Recon → Lateral Movement → Exfil → Containment

- Same seed + same clock-offset = same timeline (reproducible for demo replay)
- Stage transitions happen at real wall-clock time during the demo — the analyst can query prism WHILE the attack progresses and see the picture grow

BCs: **BC-2.06.019 v1.7** (scenario progression) — **ACTIVE** (POL-14 at merge D-1139).

### Stage-Gated Visibility (StageMask)

As the attack unfolds, querying sensors **mid-scenario shows a GROWING picture**:
- Stage 0 (Baseline): clean baseline, normal devices/alerts only
- Stage 1 (Recon): compromised device `dev-deadbeef-…` appears in sensor data
- Stage 2 (Lateral Movement): lateral spread devices appear; IOC hashes surface
- Stage 3 (Exfil): IOCs expand to IPs/domains; exfil alerts fire
- Stage 4 (Containment): containment activates; isolation flags set on devices

**Earlier stages legitimately show less** — this is the demo narrative: "watch the investigation expand as the attack progresses."

### Cross-DTU Coherence

The **same compromised device** (`dev-deadbeef-…`) appears consistently across Armis + CrowdStrike + Claroty at the correct stage. Correlating across sensors tells ONE coherent story per client.

The analyst can pivot: "this device in Armis — what does CrowdStrike say about it? what does Claroty say?"

### Enrichment Correlation (D-1117 Enhancement)

Every scenario IOC resolves in **ThreatIntel** as Malicious (threat score ≥ 75). Every scenario CVE resolves in **NVD** at HIGH CVSS 8.1.

Per user direction (D-1117):
- Every synthetic CVE is **collision-safe**: `CVE-9999-{:04}` format — never matches a real advisory
- Cyberint alert CVEs correlate end-to-end: a Cyberint alert's CVE genuinely resolves in NVD via the real `lookup_and_count` pivot (proven by integration test VP-020-K)
- Enrichment data is scenario-seeded per-client — not random fixture noise

BCs: **BC-2.06.020 v1.6** (enrichment correlation) — **ACTIVE** (POL-14 at merge D-1139).

### Acceptance Criteria and Demo Evidence

19 acceptance criteria, all with recorded VHS demo evidence (19/19 complete).

---

## MERGED — T6 (PR #187, S-DEMO-MULTI-TENANT-DTU-001)

> **MERGED develop@664566e9 (2026-06-14; D-1158). BC-2.06.017 v1.10 ACTIVE (POL-14). T6 DONE.**

### Per-Instance Multi-Address Binding — Multi-Tenant DTU Overlay

Per-client DTU instance binding, graceful-shutdown lifecycle handle (`MultiInstanceServers`), and end-to-end FanOutTarget DTU-routing isolation proof. Delivered:

- `MultiInstanceConfig` / `InstanceEntry` / `start_instances` → `MultiInstanceServers` lifecycle handle with `socket_map()` accessor + `shutdown()` + `Drop` graceful drain (prism-dtu-demo-server)
- `MultiInstanceHarness` / `HarnessEntry` / overlay_wiring + `BindError` / `HarnessError` variants (prism-dtu-harness)
- `ArmisClone` server-side request counter (`GET /dtu/request-count`) for AC-006 isolation proof (prism-dtu-armis)
- End-to-end `FanOutTarget` DTU-routing isolation test (`prism-sensors/tests/multi_tenant_dtu_routing_integration.rs`)
- Non-exhaustive gate 52→60 (`ci.yml EXPECTED=60`)

BC-2.06.017 v1.10 is **ACTIVE** (POL-14 at merge D-1158). Story v1.14. LOCAL 11-pass 3-CLEAN strict + PR-LEVEL 10-pass 3-CLEAN strict CONVERGED (BC-5.39.001). CI 43/43 green.

---

## MERGED — PrismQL Case-Insensitive Operators (PR #217, S-PRISMQL-CASE-INSENSITIVE-001)

> **MERGED develop@f935edb6 (2026-07-08; D-1607). BC-2.11.024 v1.4 + BC-2.02.013 v1.10 ACTIVE (POL-14).**

### Case-Insensitive PrismQL Operator Surface (IEQ / IIN / INE)

PrismQL ships three opt-in case-insensitive operators that complete the `I`-prefix family (`ICONTAINS`/`ISTARTSWITH`/`IENDSWITH`). ADR-047 ACCEPTED; design basis in `architecture/decisions/ADR-047`.

| Operator | Syntax | Example |
|----------|--------|---------|
| `IEQ` | `field IEQ 'value'` | `severity IEQ 'high'` matches stored `'High'` |
| `IIN` | `field IIN ('v1', 'v2')` | `severity IIN ('high', 'critical')` matches `'High'` or `'Critical'` |
| `INE` | `field INE 'value'` | `severity INE 'informational'` excludes rows where `lower(severity) = 'informational'` |

Available in **filter mode** and **pipe-mode `| where` stages**. Raw SQL mode intentionally rejects `IEQ`/`IIN`/`INE` at parse time (`E-QUERY-001`) with a structured message pointing to the correct filter/pipe syntax — this is a pedagogical error-UX beat the demo can demonstrate once. The default `=`, `!=`, `IN` remain case-sensitive and unchanged (backward-compatible additive change).

### Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization

All OCSF enum-label string fields (`severity`, `status`, `activity_name`, `disposition`) are normalized to canonical OCSF Title-case at the spec-driven adapter boundary (`build_column_array` in `spec_driven_adapter.rs`) before DataFusion receives the data. Vendor-specific casings — CrowdStrike already emits Title-case; Armis emitted `'UNHANDLED'` as-received (left as-received with warn when unrecognized); Claroty emitted as-received — are handled at ingest, not at query time.

**Cross-sensor `GROUP BY` fix:** `GROUP BY severity` now produces at most 7 distinct buckets (`Unknown`/`Informational`/`Low`/`Medium`/`High`/`Critical`/`Other`) regardless of which sensor contributed the row — no more `'HIGH'` vs `'High'` fragmentation across sensors.

**Typed guidance (E-QUERY-002):** Applying `IEQ` to an integer `_id` companion column (e.g., `severity_id IEQ 'high'` where `severity_id` is an integer column) returns `E-QUERY-002 (QueryTypeMismatch)` with a suggestion to use the string sibling column (`severity`). This steers analysts who target the integer companion toward the correct string field.

BCs: **BC-2.11.024 v1.4** (IEQ/IIN/INE operators) + **BC-2.02.013 v1.10** (adapter-boundary normalization) — both **ACTIVE** (POL-14 at merge D-1607). 27 ACs, 81 Red Gate tests, 5317/5317 workspace tests GREEN.

---

## SCOPED-NOT-BUILT — Honest Gaps

> Critical to not forget. These items are designed and scoped but have NO code yet.

### THE `enrich` QUERY PATH IS NOT WIRED YET — REQUIRED BEFORE DEMO (D-1164 USER DECISION)

> **D-1164 USER SCOPE DECISION (2026-06-14):** FULL Option-A infusion framework is REQUIRED before the demo is recorded. See §Binding Demo Invariant corollary above.

ThreatIntel + NVD are seeded with correlated data and the DTU clones **return it**, but the analyst **CANNOT yet pivot** `| enrich nvd(cve_id)` **through prism** in a PrismQL query. The infusion engine (`S-1.14`) is partial-merge / 100% `unimplemented!()` (TD-PLUGIN-P0-002 P0 open).

**D-1164 resolution:** The user has chosen Full Option A — build the entire infusion framework before demo recording. Enrichment must run through the REAL prism code path with DTU clones as the only substituted element.

**The FULL Option-A infusion chain (REQUIRED, demo-critical-path) — 4 stories (D-1168 architect verdict: S-1.15 DROPPED from demo lane):**

Designed in WO-D1109 at `.factory/specs/architecture/work-orders/WO-D1109-enrichment-pivot.md`. Four stories in linear dependency order (S-1.15 REMOVED from demo enrichment lane — see §S-1.15 DROP below):
- **S-1.14-REDO** (~8pt; draft/blocked) — Full infusion engine: InfusionLoader + 3-tier cache + all source types (MMDB/CSV/JSON + plugin). FOUNDATIONAL. (`S-DEMO-ENRICHMENT-PIVOT-001` is its `forward_subset_implemented_by`.)
- **S-DEMO-ENRICHMENT-PIVOT-001** (~5pt; ready v1.3) — plugin-type `InfusionLoader::parse` + `PluginInfusionSource` + DataFusion `ScalarUDF` registration in prism-query.
- **S-DEMO-ENRICHMENT-PIVOT-002** (~8pt; draft v1.1) — `threatintel.infusion.toml` + `nvd.infusion.toml` grounded vs DTU route surfaces + two WASM `.prx` plugin crates (`prism-threatintel-infusion`, `prism-nvd-infusion`) calling DTU HTTP endpoints.
- **S-DEMO-ENRICHMENT-PIVOT-003** (~8pt; draft v1.8) — real IOC/CVE field stamping in Cyberint/CrowdStrike DTU fixtures + validation of canonical pivot queries `| enrich threat_intel(ioc_value)` / `| enrich nvd(device_cves_first)` against demo server at scenario stage >= 3.

WASM toolchain risk ACCEPTED with documented contingency per D-1164: if WASM blocks, `PluginInfusionSource::enrich_single` may fall back to a direct `reqwest` HTTP call to the DTU endpoint, TD-anchored to S-1.14-REDO/S-1.15 for replacement. This is a human-directed deferral per Canonical Principle Rule 3.

**§S-1.15 DROP from demo enrichment lane (D-1168 architect verdict):** S-1.15's remaining work is `fire_alert`/`fire_case`/`fire_report` action-plugin dispatch (TD-PLUGIN-P0-008). This is write-back/TDE (DEFERRED), NOT enrichment. `enrich_single` (the enrichment path) is already operational on develop. The enrichment lane needs NO S-1.15 work. S-1.15 is tracked alongside S-4.08 as deferred-TDE, NOT demo-blocking. S-1.15 as a story STILL EXISTS in STORY-INDEX (total_stories unchanged); it is only removed from the demo enrichment lane set.

This is **THE FLAGSHIP `enrich` FEATURE**. Slots AFTER the capability-discovery block, BEFORE T11 launcher and T13 capstone. Closes TD-PLUGIN-P0-002 (P0) upon merge.

### Capability-Discovery Block — REQUIRED (D-1107 scoped, D-1162 promoted to REQUIRED)

> **D-1162 USER SCOPE DECISION (2026-06-14):** These stories are NOT optional. User explicitly stated "are not optional." All four are now REQUIRED core demo deliverables.

Four stories:
- **S-5.02** — Tool routing / errors / client scoping (depends_on S-5.01; PREREQ-VERIFICATION needed — S-5.01 formal story row shows not-started but S-5.01-FOLLOWUP-MCP-BOOT merged PR #163 2026-05-29 is the graduation vehicle)
- **S-5.03** — Resources and prompts (hard dep of S-5.04; depends_on S-5.02; transitive pull-in per D-1162)
- **S-5.04** — Sensor health subsystem (depends_on S-5.03 + S-DEMO-001)
- **S-3.13** — Dynamic table availability (parallel after PO authors dedicated BCs; depends_on S-3.02 SATISFIED + S-1.12 PREREQ-VERIFICATION needed — partial-merge; S-1.12-FOLLOWUP BLOCKED)

All four require `dclaude:remove-uncertainty` before TDD delivery (D-1110 standing rule). PO must author dedicated BCs for S-5.02 and S-3.13 before status=ready. Delivery ordering: S-5.01-verify → S-5.02 → S-5.03 → S-5.04; S-1.12-verify → S-3.13 (parallel chain).

### T11 — Launcher Consolidation

**Story:** `S-DEMO-LAUNCHER-CONSOLIDATION-001` (ready v2.1; depends_on S-DEMO-003 SATISFIED)

Option-2 Rust executed (D-1167/D-1168): `StartMulti` CLI subcommand wiring `start_instances`/`MultiInstanceConfig`; `MultiOrgDemoConfig`/`OrgConfig` structs; nested `{org_slug:{sensor:url}}` sidecar; 13 ACs; 8 pts; tdd_mode tdd; 5 Red Gate tests; `fixture-gen` feature required (HARD-ERROR if absent — GAP-1 closure); Cyberint `new_with_seed`+`configure({access_token})` composite (GAP-2 closure); DEMO_RUN_DIR note (GAP-3); new `[[test]] required-features=["dtu","fixture-gen"]` guard. remove-uncertainty NEXT.

### T13 — Capstone

**Story:** Multi-client SOC-analyst narrative story (not yet named or authored). Owner: product-owner + story-writer. After data layer + tooling exist.

---

## What the Demo CAN SHOW Today (Post-T6-Merge; develop@664566e9)

An analyst in Claude Code, querying **multiple client orgs** through prism, watching an attack **unfold in real time** across CrowdStrike / Armis / Claroty / Cyberint — devices appearing, IOCs surfacing, containment triggering — with **cross-sensor correlation** telling one coherent story per client. Per-instance multi-address binding (T6) enables genuine multi-tenant DTU overlay with routing isolation proofs.

The enrichment pivot (alert → IOC/CVE → ThreatIntel/NVD answer) is demonstrable at the DTU level and becomes a TRUE in-prism analyst pivot once the PIVOT-001/002/003 infusion chain lands.

---

## Build Sequence (Where Each Piece Sits)

```
T1–T4 DONE
  → T4-A MERGED (PR #181 develop@c287b00d) — per-client distinct seeded data; BC-2.06.018 v1.6 active
  → T5 MERGED (PR #185 develop@7fd35b77 2026-06-13) — unfolding-attack scenario; BC-2.06.019 v1.7 + BC-2.06.020 v1.6 active
  → T6 MERGED (PR #187 develop@664566e9 2026-06-14) — multi-tenant DTU overlay; BC-2.06.017 v1.10 active
  → T10 MERGED (PR #188 develop@7241f5ef 2026-06-15) — multi-org isolation smoke test; BC-2.06.017/018 verified
  → T11 MERGED (PR #190 develop@c3ecf6c8 2026-06-16) — launcher consolidation; start-multi Rust subcommand
  → T12 MERGED (PR #189 develop@1b2e9a31 2026-06-16) — PIVOT-001 plugin-type UDF; BC-2.19.001+BC-2.19.003 active
  → T15a S-5.02 MERGED (PR #191 develop@bec894a2 2026-06-17) — MCP tool routing + client scoping; BC-2.10.011 active
  → T15d S-3.13 MERGED (PR #192 develop@60249ccc 2026-06-16) — dynamic table availability; BC-2.16.007 active

  *** CURRENT POINTER: L-POST — ALL prior lanes CLOSED; develop@60249ccc ***
  *** SCOPING RESOLVED (D-1205 2026-06-16): PIVOT-002+PIVOT-003+S-1.14-REDO DEMO-BLOCKING ***

  PARALLEL BLOCK (all start now; T13 waits for all):
  → S-1.14-REDO [DEMO-BLOCKING D-1205; deps S-WAVE5-PREP-01+S-3.02-FOLLOWUP-RUNTIME SATISFIED]
      Full infusion engine: non-plugin InfusionSource types + three-tier cache + VP-048 Kani + VP-049 proptest
  → PIVOT-002 [DEMO-BLOCKING D-1205; deps PIVOT-001 MERGED]
      MANDATORY pre-start security gates: DRIFT-PIVOT-UDFNAME-VALIDATION-001 +
        DRIFT-PIVOT-PLUGINCONFIG-PUB-FIELD-001 + DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001
      Delivers: prism-threatintel-infusion + prism-nvd-infusion WASM .prx plugin crates
  → T15b S-5.03 [DEMO-BLOCKING per D-1162; deps S-5.02 SATISFIED]
      remove-uncertainty before TDD; hard prereq of S-5.04
  PIVOT-003 [DEMO-BLOCKING D-1205; deps PIVOT-002 MERGED]
      Real IOC/CVE field stamping in Cyberint/CrowdStrike DTU fixtures
      Canonical | enrich pivot-query validation at scenario stage >= 3
      Closes BC-2.06.019 §Interim State _ioc_value violation
      Closes TD-PLUGIN-P0-002 (P0)
  → T15c S-5.04 [DEMO-BLOCKING per D-1162; deps S-5.03 MERGED]
      Sensor health subsystem; remove-uncertainty before TDD
  → T13 capstone [LAST before recording — AFTER all enrichment + capability-discovery + S-1.14-REDO merged]
  → T14 demo recording
```

---

## Binding Demo Invariant — DTU-EVERYTHING (D-1163, user reaffirmation 2026-06-14)

> **This invariant is authoritative and binding on ALL remaining demo stories.**

**DTU-EVERYTHING: For the live demo, ALL data sources run on prism DTU behavioral clones — every sensor (CrowdStrike/Armis/Claroty/Cyberint) AND every enrichment source (ThreatIntel/NVD). NO real third-party API connections in the demo.**

All remaining demo stories (S-5.02 / S-5.03 / S-5.04 / S-3.13 / launcher consolidation / narrative capstone) MUST scope against DTU clones, not live services. Story specs, acceptance criteria, and Red Gate tests must ground against DTU clone routes, not production vendor endpoints.

**Corollary — infusion/WASM enrichment (D-1164 SUPERSEDES D-1163 corollary):** Per user decision D-1164 (2026-06-14), the FULL infusion framework (Option A) is REQUIRED before the demo is recorded. Real enrichment must flow through the REAL prism code path the same structural way sensors do: `| enrich` PrismQL pipe → DataFusion UDF → InfusionRegistry → PluginInfusionSource → WASM plugin → DTU HTTP endpoint. DTU clones are the ONLY substituted element — this is fully consistent with the DTU-EVERYTHING invariant (the DTUs ARE the endpoints; the prism enrichment code is real). Story B's demo-server-side enrichment correlation (BC-2.06.020) remains correct and on develop, but is acknowledged as NOT sensor-parity-real: it pre-seeds ThreatIntel/NVD DTU registries from `ScenarioEntityCatalog` without executing any prism enrichment code path. D-1164 supersedes and completes Story B's work via the FULL Option-A infusion framework. TD-PLUGIN-P0-002 (P0 open — infusion 100% `unimplemented!()`) is scheduled for closure by this work (S-1.14-REDO + S-1.15 + S-DEMO-ENRICHMENT-PIVOT-001/002/003). WASM toolchain risk ACCEPTED with contingency: if WASM blocks, `PluginInfusionSource::enrich_single` may fall back to a direct `reqwest` HTTP call to the DTU endpoint, TD-anchored to S-1.14-REDO/S-1.15 for replacement. **The PIVOT-001/002/003 enrichment chain is REQUIRED BEFORE T13 capstone/T14 recording.** Four Option-A stories (D-1168: S-1.15 DROPPED from demo enrichment lane — deferred-TDE with S-4.08; TD-PLUGIN-P0-008): S-1.14-REDO (full infusion engine) + S-DEMO-ENRICHMENT-PIVOT-001 (plugin-type UDF, ~5pt) + S-DEMO-ENRICHMENT-PIVOT-002 (2 WASM `.prx` plugins + infusion.toml, ~8pt) + S-DEMO-ENRICHMENT-PIVOT-003 (IOC stamping + pivot-query validation, ~8pt).

**Cross-ref:** Task ledger `.factory/objectives/multi-client-soc-demo-tasks.md` §PREREQ-CONFIRMED block (D-1163).

---

## Cross-References

| Document | Role |
|----------|------|
| `.factory/objectives/multi-client-soc-demo-tasks.md` | Granular task tracker (task status, next action, story sequence, CURRENT POINTER) |
| `.factory/SESSION-HANDOFF.md §ACTIVE OBJECTIVE` | Resume-oriented narrative + build sequence mirror |
| `.factory/STATE.md` | Live pipeline state (current phase, active decision rows) |
| `.factory/stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md` | Story B spec (Story B v2.16, BCs 019+020) |
| `.factory/specs/architecture/work-orders/WO-D1109-enrichment-pivot.md` | Enrichment pivot design (PIVOT-001/002/003 architecture) |
| `.factory/specs/behavioral-contracts/BC-2.06.017.md` | Multi-tenant DTU overlay contract (v1.11 active) |
| `.factory/specs/behavioral-contracts/BC-2.06.019.md` | Scenario progression contract (v1.7 active) |
| `.factory/specs/behavioral-contracts/BC-2.06.020.md` | Enrichment correlation contract (v1.6 active) |
