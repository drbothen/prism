---
document_type: triage-capture
title: LIVE-DEMO/DTU Findings Triage — 2026-07-20
source_readings:
  - findings/prism-pql-deficiencies.md
  - findings/prism-pushdown-audit.md
  - findings/dtu-fidelity-gaps.md
  - findings/dtu-scenario-enhancements.md
  - findings/per-sensor specs + auth fidelity reports (cyberint, crowdstrike, armis, claroty)
decision: D-1889
date: 2026-07-20
authored_by: state-manager (inventory + routing only; no spec/BC content authored)
status: OPEN — open decisions require human sign-off before dispatch
---

# LIVE-DEMO/DTU Findings Triage Capture (D-1889)

This is a triage-CAPTURE: inventory + routing + open-decisions only.
No spec, BC body, or architectural content is authored here.
Full VSDD pipeline directed by human (D-1888 priority pivot).

---

## Bucket B — Engine Defects

Source: findings/prism-pql-deficiencies.md + findings/prism-pushdown-audit.md

| Finding ID | Sev | Summary | BC anchor | Route | Coverage |
|---|---|---|---|---|---|
| DEFECT-PQL-SUBQUERY-FANOUT-001 (F1) | CRIT | cross-sensor WHERE IN(SELECT) silently 0 rows; fix = recursive source extraction at materialization.rs:727 | BC-2.11.005 | PO→test-writer+implementer (SAP-3 e2e) | NEW |
| DEFECT-JOIN-DUPKEY-COLLAPSE-001 (F2) | MED | duplicate JSON keys collapse at MCP serialization boundary | BC-2.11.001 | PO→implementer (wire-shape/SID-2) | NEW |
| DEFECT-REFERENCE-JOIN-BNF-001 (F3) | MED | prismql://reference omits JOIN BNF; spec BC-2.10.014 v1.2 already requires it (impl drift) | BC-2.10.014 | implementer | PARTIAL |
| DEFECT-PQL-ON-CONTAINS-001 (F4) | LOW | CONTAINS valid in WHERE not ON | BC-2.11.003 | PO decision→implementer | NEW |
| DEFECT-OCSF-STATUS-VOCAB-001 (F5) | MED | over-claiming enum-normalization contract; needs enum_value_map | BC-2.02.013+ADR-047 | architect+PO→SW→impl | PARTIAL |
| DEFECT-HEALTH-PER-TABLE-BLINDSPOT-001 (F6) | HIGH | health probe per-sensor not per-table; missed 80min of E-SENSOR-030 | BC-2.08.001/005/007 (+new) | PO→SW→impl | NEW |
| DEFECT-PIPE-WHERE-PUSHDOWN-001 (F7=G1) | HIGH | Ast::Pipe/Filter get zero pushdown | BC-2.11.007/020 | PO→impl | NEW |
| DEFECT-SPEC-ARRAY-COLUMN-TYPE-001 (F8=GAP-2c) | HIGH | no array ColumnType; ip_list/mac_list dropped; real-client impact; unregistered promised story (Rule-3 violation) | new BC + prism-core | architect+PO→SW→impl | NEW |
| DEFECT-SENSOR-ERROR-FLATTEN-001 (F9) | HIGH | all SpecEngineError flattened to Internal; 401 misreported as unreachable | BC-2.08.002 | implementer+PO | NEW |
| DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (F10) | CRIT | live xDome HTTPS fails (WAF profile: h1-only, no UA); primary fix = add http2 reqwest feature + user-agent (ADR-050 compliant, source-recommended); secondary option native-tls CONFLICTS w/ADR-050 → architect decides between options. Source F10 also requires: capture error source-chains + surface per-target errors. (consistency-validator cross-check 2026-07-21: original note distorted primary fix path — prism-pql-deficiencies.md §Finding 10 source dive shows http2 not compiled in + no User-Agent in any production crate; native-tls was listed as alternative, not primary) | new BC/ADR | architect FIRST→impl | NEW |
| DEFECT-RELOAD-OVERLAY-ADAPTER-FREEZE-001 (F11) | MED | reload_config non-recursive scan misses overlays + never rebuilds AdapterRegistry; unanchored boot.rs deferral (Rule-3) | BC-2.16.005/007 | PO→SW→impl | PARTIAL |
| DEFECT-PAGINATION-ROW-BUDGET-001 (F12=G3) | HIGH | no row budget; limit 5→148 upstream reqs | BC-2.07.001/002+2.11.006 | architect+PO→SW→impl | NEW |
| DEFECT-QUERY-TIMEOUT-ORPHAN-SWEEP-001 (G4) | CRIT/HIGH | query timeout drops parent future; detached tokio::spawn fan-out keeps hitting live tenant API; retry re-sweeps offset 0 | BC-2.11.006 | architect+impl | NEW (split from F12) |
| DEFECT-PUSHDOWN-TIMEWINDOW-CLAROTY-CYBERINT-001 (G2) | HIGH | time-window structurally impossible for claroty+cyberint | BC-2.01.013 (+FetchStep grammar) | architect+PO→SW | PARTIAL (claroty half=unauthored stub S-DEMO-CLAROTY-TIME-001) |
| DEFECT-CACHE-KEY-FRAGMENTATION-001 (G5) | MED | dropped filters still key cache → duplicate fetches | BC-2.07.005 (draft) | PO→impl | ADJACENT |
| DEFECT-PUSHDOWN-OPERATOR-CLASS-001 (G6) | MED | IN/!=/OR/range never extracted; classify_predicates uncalled | ADR-022 §C | architect→SW | WEAK |
| DEFECT-PUSHDOWN-EQUALITY-SLOTS-001 (G7) | MED | equality extracted then dropped (no spec slots) | BC-2.11.007 | architect+PO→SW | NEW |
| DEFECT-CS-DEVICES-INCIDENTS-TW-001 (G8) | MED | CS devices/incidents no FQL time-window slot | BC-2.01.013 | impl+PO | NEW |

**Engine CRIT count: 3 (F1, F10, G4)**

---

## Per-Sensor Spec/Auth Fidelity

Proposed story IDs with routing and human-gate flags.

| Proposed ID | Sev | Summary | Key BCs | Human-gate? |
|---|---|---|---|---|
| S-ARMIS-AUTH-FIDELITY-001 | CRIT | bearer_static→token-exchange (POST /api/v1/access_token/), drop Bearer prefix; no token_exchange auth_type exists; DTU has no access_token route | BC-2.01.008/016, BC-2.06.003, ADR-028/D-747 | YES (overturns LOCKED D-747) |
| S-ARMIS-FIELD-FIDELITY-001 | HIGH | DTU↔real field-name drift (title/time/deviceIds vs name/created_at/device_id) | BC-2.02.006 | no |
| S-ARMIS-COLLECTIONS-001 | HIGH/MED | 2 of ~12 AQL collections mapped (vulnerabilities/activity T1) | BC-2.02.006 | no |
| S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001 | CRIT | Detects EOL 2025-09-30 → /alerts/queries\|entities/alerts/v2; composite_id key/fan-out; fixes probe_table poison | BC-2.16.013, BC-2.02.003, BC-2.01.005 | architect (ADR-028 grounding) |
| S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001 | CRIT | Incidents API removed ~2026-03; retire table; DTU-EXT-001 re-adjudicate | BC-2.16.013, BC-2.02.003 | product decision |
| S-CROWDSTRIKE-SPOTLIGHT-VULNS-001 | HIGH | add Spotlight vulnerabilities table (ExPRT/CVSS/KEV) + DTU routes | BC-2.16.013 | no |
| DEFECT-CYBERINT-SPEC-FIDELITY-001 | CRIT | phantom incidents table (delete) + alerts retable POST /alert/api/v1/alerts page/size $.alerts + DTU re-clone; created_at→created_date; add ref_id | BC-2.02.004, BC-2.16.013 | no (auth dim gated separately) |
| ARCH-CYBERINT-AUTH-READJUDICATION-001 | HIGH | split Alerts(no securityScheme,likely X-Api-Key)/Assets(cookie) base_url+auth | BC-2.01.006/017, BC-2.06.003 | YES (overturns LOCKED #4) |
| S-DEMO-CLAROTY-LIVE-DRIFT-BACKPORT-001 | HIGH | backport .prism-live fields[] bodies + /audit_log/get (no trailing slash) into canonical spec | BC-2.02.005 | no |
| S-DEMO-CLAROTY-DEVICE-ENRICH-001 | MED/HIGH | arrays via source_path + scalars + alert_class (DTU struct expansion, SAP-2) | BC-2.02.005 | no |
| S-DEMO-CLAROTY-RELATIONS-001 | HIGH | device_alert_relations table + new DTU route/type/fixtures | BC-2.02.005 | no |
| S-DEMO-CLAROTY-VULN-LAYER-001 | MED | vulnerabilities table (DTU already ahead) | BC-2.02.005 | no |

**Sensor CRIT count: 5 (S-ARMIS-AUTH-FIDELITY-001, S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001, S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001, DEFECT-CYBERINT-SPEC-FIDELITY-001 + 1 arch-human-gate ARCH-CYBERINT-AUTH-READJUDICATION-001)**

**Total CRIT (engine + sensor): 8**

> **Cyberint Alerts auth CONFIRMED (research 2026-07-20, 6 sources):** static Cookie:access_token (NOT X-Api-Key); X-Api-Key hypothesis REJECTED. D3-c precondition RESOLVED — ARCH-CYBERINT-AUTH-READJUDICATION-001 auth-header uncertainty removed; ADR-053 §D3 Cyberint Alerts header_scheme=cookie:access_token ratified in v0.2.

---

## DTU Fidelity + Scenario Enhancements

Source: findings/dtu-fidelity-gaps.md + findings/dtu-scenario-enhancements.md

**Fidelity gaps:**

- **GAP-1** incidents routes — CONFLICT: retire-vs-build — OPEN DECISION (see Open Decisions #4)
- **GAP-2a** tracked as S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001; subsumes stub S-DTU-CROWDSTRIKE-COLUMN-COVERAGE-001
- **GAP-2b** degenerate IP/MAC + no MAC field (HIGH, blocks cross-sensor identity merge)
- **GAP-2c = F8** (DEFECT-SPEC-ARRAY-COLUMN-TYPE-001; listed in engine defects above)
- **GAP-3** shared device.uid contradictory profiles (HIGH, ADR-036 amendment required)
- **GAP-4** EDR detections on OT devices (MED)
- **GAP-5** cross-sensor alert-uid collisions (MED)
- **GAP-6** CS fleet homogeneity (LOW)

**Scenario enhancements:**

- E1: drip-feed steady-state
- E2: multi-cycle attack chain
- E3: FP/BTP disposition diversity
- E4: IOC richness (CORRECTNESS)
- E5: degradation-recovery
- E6: alert-lifecycle transitions

---

## Wrong-Direction Stories Flagged for Retirement/Re-scope

**PENDING human/PO adjudication — do NOT retire yet.**

| Story | Action | Reason |
|---|---|---|
| S-DEMO-CYBERINT-INCIDENTS-SEEDING-001 | retire | would build phantom endpoint |
| S-DTU-CROWDSTRIKE-INCIDENTS-ROUTE-001 | re-scope/retire | PRODUCT-DECISION-PENDING |
| DTU-EXT-005 (cyberint page_size) | retire | wrong pagination model |
| DTU-EXT-001 (crowdstrike incidents route) | re-adjudicate | conflicts with Incidents API removal |
| S-DTU-CROWDSTRIKE-COLUMN-COVERAGE-001 | merge into S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001 | subsumption |

---

## Open Decisions Requiring Human Sign-Off

All 5 are OPEN — no dispatch until sign-off received.

| # | Decision Required |
|---|---|
| 1 | Grounding-chain supersession ADR: flip ADR-028 from spec←DTU to spec←OpenAPI (root cause of all-sensor drift) |
| 2 | LOCKED decision #4 (cyberint=cookie_roundtrip, D-747) overturn — required for ARCH-CYBERINT-AUTH-READJUDICATION-001 |
| 3 | Armis auth (bearer_static, D-747) overturn — required for S-ARMIS-AUTH-FIDELITY-001 |
| 4 | Incidents disposition (retire dead endpoints vs build DTU routes) — cross-report CONFLICT between dtu-fidelity reader (build) and endpoint readers (retire) |
| 5 | Cycle structure / wave sequencing |

**Fold-in note:** DRIFT-D849-002 (StaticCookieAuthProvider no-network VP) to be folded into Cyberint auth architect dispatch. DRIFT-CI-STDBOOL and DRIFT-LOCAL-DEVELOP-FF-001 are unrelated to this triage set.

---

## Wave Assignments (D-1889 human-adjudicated 4-wave structure; annotated 2026-07-21 per D-1900 resume cross-check)

| Wave | Theme | Items |
|---|---|---|
| **Wave A — grounding+auth** | Grounding flip ADR (ADR-054 native-auth-acquisition + ADR-053 grounding+auth-shape); native declarative HTTP auth acquisition + crowdstrike-oauth2.prx retirement (D-1895); S-ARMIS-AUTH-FIDELITY-001 (Armis token-exchange); S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001 + S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001 (Alerts-v2 migration + incidents retire+derive from Alerts aggregate_id); DEFECT-CYBERINT-SPEC-FIDELITY-001 + ARCH-CYBERINT-AUTH-READJUDICATION-001 (Cyberint dual-surface split, alerts vs assets auth); xDome live-drift backport (S-DEMO-CLAROTY-LIVE-DRIFT-BACKPORT-001). | ADR-054 + ADR-053 cascade; 6 sensor/auth stories |
| **Wave B — endpoint fidelity** | CrowdStrike Spotlight vulnerabilities (S-CROWDSTRIKE-SPOTLIGHT-VULNS-001); xDome device enrich (S-DEMO-CLAROTY-DEVICE-ENRICH-001) + device_alert_relations (S-DEMO-CLAROTY-RELATIONS-001) + vuln layer (S-DEMO-CLAROTY-VULN-LAYER-001; includes claroty_device_vuln_relations vs claroty_vulnerabilities disambiguation + claroty_ot_activity_events Tier-1 table — see addendum item 9); Armis collections (S-ARMIS-COLLECTIONS-001). | 6 stories |
| **Wave C — Bucket-B engine** | F1 subquery-fanout CRIT (DEFECT-PQL-SUBQUERY-FANOUT-001); F7 pipe-where pushdown (DEFECT-PIPE-WHERE-PUSHDOWN-001); F9 error-flatten (DEFECT-SENSOR-ERROR-FLATTEN-001); F10 xDome TLS CRIT (DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — ADR-050-compliant path only; architect decides http2 vs native-tls in ADR/story scope); F12 row-budget (DEFECT-PAGINATION-ROW-BUDGET-001); G4 timeout-orphan-sweep CRIT (DEFECT-QUERY-TIMEOUT-ORPHAN-SWEEP-001). | 6 engine stories |
| **Wave D — extended coverage + scenario/DTU-fidelity** | S-ARMIS-FIELD-FIDELITY-001; remaining DTU scenario enhancements (E1–E6); DTU attribution open question (see addendum item 10); extended sensor coverage gaps from addendum items 2–8 (as prioritized at backlog opening). | TBD at backlog opening |

---

## Summary Counts

| Category | Count |
|---|---|
| Engine defects (Bucket B) | 18 |
| Sensor spec/auth fidelity | 12 |
| DTU scenario enhancements | 6 |
| Wrong-direction stories flagged | 5 |
| Open human-gate decisions | 5 |
| **Total findings registered** | **36** |
| CRIT total | 8 (F1, F10, G4 engine + 5 sensor) |
| Systemic root cause | ADR-028 spec←DTU circular grounding |

---

## Cross-Check Addendum 2026-07-21 (consistency-validator coverage gaps vs source findings)

Additional items identified by fresh-context consistency-validator cross-check (D-1900 resume, pre-architect-dispatch). Not in original triage table. Each item carries source citation, severity, and disposition. These are ADVISORY registrations — not blocking current Wave-A dispatch.

| # | Sev | Item | Source | Disposition |
|---|---|---|---|---|
| 1 | LOW | F3-minor: stale credential CLI usage strings — `credential_cli.rs:69` and `cli.rs:107` omit `--org-slug` in help text; print appears above clap usage block. | prism-pql-deficiencies.md §Finding 3 minor item | Fold into DEFECT-REFERENCE-JOIN-BNF-001 scope, or standalone LOW story at Wave-B/C backlog opening |
| 2 | MED | CrowdStrike G4: no threat intel domain (indicators, actors, reports) — unmodeled API surface. | prism-crowdstrike-endpoint-plan.md §Tier 2 G4 | Wave B+/unassigned; register at backlog opening |
| 3 | MED | CrowdStrike G5: no IOC management pivot. | Same file G5 | Wave B+/unassigned |
| 4 | MED | CrowdStrike G6: no Identity Protection (needs GraphQL adapter). | Same file G6 | Wave B+/unassigned |
| 5 | LOW | CrowdStrike G8: no hidden-hosts / host-groups. | Same file G8 | Wave B+/unassigned |
| 6 | LOW-MED (WAVE-A PREREQUISITE) | CrowdStrike G9: not region-aware; `Alerts:READ` scope required for Alerts-v2 migration — if tenant credential only has `Detects:READ`, S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001 blocks at implementation with a scope error. | Same file G9 | Prerequisite note for ADR-053 §D / S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001 (Wave A); mention in story acceptance criteria |
| 7 | MED (WAVE-A SCOPE NOTE) | Cyberint C7: AlertSeverity returns lowercase `low/medium/high/very_high` (OCSF Title-case convention violation; distinct from F5 status-vocab issue); AlertStatus has no 1:1 OCSF mapping. | prism-cyberint-endpoint-plan.md §Gaps C7 | Add to DEFECT-CYBERINT-SPEC-FIDELITY-001 scope (Wave A) or combine with F5 follow-up story |
| 8 | MED | Cyberint C8: entire ASM/assets/technologies/analytics/credentials read domain unmodeled (net-new value surface). | Same file C8 | Wave B+/unassigned |
| 9 | (WAVE-B SCOPING) | xDome: `claroty_ot_activity_events` (POST `/api/v1/ot_activity_events/`) is a distinct Tier-1 table not mentioned in current triage; S-DEMO-CLAROTY-VULN-LAYER-001 may conflate `claroty_vulnerabilities` (CVE catalog) with `claroty_device_vuln_relations` (214-field device↔CVE join) — these are separate tables. | prism-xdome-endpoint-plan.md §Tier 1 | Disambiguate at Wave-B story scoping before S-DEMO-CLAROTY-VULN-LAYER-001 is written; add claroty_ot_activity_events as Wave-B story |
| 10 | OPEN QUESTION | DTU attribution: short-lived node processes (PIDs 76231/76309, 17:27–17:35 UTC) deleted 59 org-c ticket files + part of mock-JIRA log then reconstructed them. DTU owners must confirm whether this is expected tooling behavior or anomalous. | dtu-fidelity-gaps.md §Open attribution question | Route to dtu-validator at Wave-D / DTU-fidelity work; track as open question pending DTU owner confirmation |
