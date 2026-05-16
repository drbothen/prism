---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 1
scope: spec
verdict: BLOCKED
total_findings: 14
severity_breakdown:
  critical: 1
  high: 4
  medium: 5
  low: 2
  observation: 2
in_scope_findings: 12
observations_queued: 2
produced_by: adversary
reviewed_at: 2026-05-15
fix_burst: fix-burst-1
fix_burst_closed_at: D-575
streak_after_fix: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 1

**Verdict: BLOCKED — 14 findings (1C + 4H + 5M + 2L + 2OBS)**

Fix-burst-1 closed 12 in-scope findings (1C + 4H + 5M + 2L) via PO+architect parallel dispatch
with state-manager catching F-LP1-HIGH-004 (POL-20 introduced-field format on 6 files → 7 files
with VP-156). 2 OBS (process-gap) queued for cycle-close session-review. Streak reset to 0/3.
Adversary pass-2 NEXT.

---

## Finding Summary Table

| ID | Severity | Status | Closed By | Description |
|----|----------|--------|-----------|-------------|
| F-LP1-CRIT-001 | CRITICAL | CLOSED | architect | VP-154 schema mismatch: old 3-field OCSF schema (id/occurred_at/raw) did not match BC-2.16.011 canonical 9-field OCSF 2004 Detection Finding schema. VP-154 §Acceptance Criteria completely rewritten to 9-field canonical (type_uid/class_uid/category_uid/severity_id/severity/time/message/finding_info.uid/raw_data). |
| F-LP1-HIGH-001 | HIGH | CLOSED | product-owner | ADR-026 D1 trait surface mismatch: BC-2.01.016 §Preconditions listed 3 methods (sensor_id + auth_type + build_request_auth) from a different trait surface; canonical ADR-026 D1 decision is 2-method trait (as_any() + auth_type_name()). BC-2.01.016 §Preconditions corrected to 2-method surface. |
| F-LP1-HIGH-002 | HIGH | CLOSED | architect | ADR-026 §E runtime_deliverable named entity: phantom runtime_deliverable section cited in story spec did not correspond to any ADR-026 heading. ADR-026 §E section corrected to remove phantom entity reference. |
| F-LP1-HIGH-003 | HIGH | CLOSED | product-owner + architect | 18 §C5 phantom-heading citations across 3 BCs + story spec: `ADR-023 §C5` cited as a heading anchor; ADR-023 has no `## C5` heading — C5 is a bold-labeled bullet inside `## Architectural Constraints`. All 18 sites corrected to `ADR-023 §Architectural Constraints (C5 bullet, Rule N)` form per POL-21. (16 planned + 2 TD-VSDD-060 sibling-catches.) |
| F-LP1-HIGH-004 | HIGH | CLOSED | state-manager | POL-20 introduced-field format violation: 6 artifacts used story-ID/slug format (S-PLUGIN-PREREQ-E or plugin-prereq-e) rather than ISO date YYYY-MM-DD. POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. All 7 files (3 BCs + 4 VPs incl. VP-156) fixed to `"2026-05-15"`. |
| F-LP1-MED-001 | MEDIUM | CLOSED | product-owner | E-SPEC-008 retirement annotation: path-a closure — error-taxonomy.md E-SPEC-008 annotated as retired per replacement by E-SPEC-012/013/014. error-taxonomy v1.25→v1.26. |
| F-LP1-MED-002 | MEDIUM | CLOSED | architect | ADR-026 D7 register_write_tool duplicate-registration semantics: was "implementer chooses"; corrected to error-on-duplicate via SpecEngineError::DuplicateWriteToolRegistration variant. ADR-026 v1.1→v1.2. BC-2.16.012 EC-016-012-004 updated accordingly. |
| F-LP1-MED-003 | MEDIUM | CLOSED | architect | BC-2.16.012 §Verification Properties coverage gap: "(none in this story)" replaced by VP-156 (new VP authored to cover register_write_tool happens-before + uniqueness proptest). VP-156 authored; VP-INDEX v1.38→v1.39 (Proptest 87→88, P1 33→34, total 155→156). |
| F-LP1-MED-004 | MEDIUM | CLOSED | product-owner | 11 TD-A-003 alias citations across spec package: informal alias TD-A-003 replaced with canonical TD-S-PLUGIN-PREREQ-A-003 per POL-23 named-entity-existence-verification. All 11 sites updated. |
| F-LP1-MED-005 | MEDIUM | CLOSED | product-owner | Red Gate test 2 phrasing: ambiguous phrasing in story spec Red Gate table row 2 clarified to match BC-2.16.011 behavioral semantics. Story v1.1→v1.2. |
| F-LP1-LOW-001 | LOW | CLOSED | product-owner | D1 trait doc comment: `as_any()` doc comment in BC-2.01.016 did not describe the trait method's behavioral purpose. Doc comment updated to explain Any-downcast usage pattern. |
| F-LP1-LOW-002 | LOW | CLOSED | architect | ADR-026 D6/D7 ordering: D6 and D7 were listed in reverse priority order relative to their architectural impact. Reordered D6 (SensorAuth::as_any downcast) before D7 (register_write_tool error-on-duplicate) per decision dependency hierarchy. |
| F-LP1-OBS-001 | OBSERVATION | QUEUED-CYCLE-CLOSE | — | Process gap: ADR-026 §E runtime_deliverables named-entity-check pattern (POL-22 Phase C extension). Adversary found phantom runtime_deliverable reference that POL-22 Phase A lexical check would have caught but Phase C named-entity-existence verification was not applied during authoring. Codification candidate for cycle-close session-review. |
| F-LP1-OBS-002 | OBSERVATION | QUEUED-CYCLE-CLOSE | — | Process gap: POL-25 VP↔BC bidirectional sweep amendment. VP-154 ↔ BC-2.16.011 bidirectional traceability gap was not caught by the D-574 consistency-validator because POL-25 multi-cite propagation sweep was not applied at VP authoring time. Codification candidate for cycle-close session-review — potential POL-25 extension to VP-authoring workflow. |

---

## Key Decisions from Fix-Burst-1

| Decision | Chosen Option | Rationale |
|----------|---------------|-----------|
| D1 trait surface | 2-method: `as_any() + auth_type_name()` | ADR-026 D1 canonical; over 1-method as-built or 3-method BC-suggested |
| D7 register_write_tool uniqueness | error-on-duplicate via SpecEngineError::DuplicateWriteToolRegistration | ADR-026 D7 v1.2; deterministic error preferred over silent overwrite or last-write-wins |
| E-SPEC-008 retirement | path-a: annotation in taxonomy v1.26 | Surgical annotation; no BC rewrite required |
| VP-156 authoring approach | proptest (not Kani) | String-keyed uniqueness is proptest territory; Kani bounded-model-checking not appropriate |
| §C5 phantom-heading repair | `§Architectural Constraints (C5 bullet, Rule N)` canonical form | POL-21; ADR-023 has no ## C5 heading |

---

## Artifact Versions After Fix-Burst-1

| Artifact | Before | After |
|----------|--------|-------|
| ADR-026 | v1.1 | v1.2 |
| BC-2.01.016 | v1.1 | v1.2 |
| BC-2.16.011 | v1.1 | v1.2 |
| BC-2.16.012 | v1.0 | v1.2 (architect v1.1 + PO v1.1 combined = v1.2 effective) |
| VP-154 | v0.3 | v0.4 |
| VP-156 | (new) | v0.1 |
| VP-INDEX | v1.38 | v1.39 |
| error-taxonomy | v1.25 | v1.26 |
| STATE + HANDOFF | v7.279 | v7.280 |

---

## Streak Status

Pass 1 BLOCKED → Fix-burst-1 CLOSED → Streak 0/3.
Adversary pass-2 dispatch NEXT (fresh-context, BC-5.39.001 3-CLEAN protocol).

_Full adversary output is in the orchestrator dispatch log for this session. This file is the
structured audit-trail record of pass-1 findings and fix-burst-1 closures per the Single-Commit
Burst Protocol (TD-VSDD-053 / D-575)._
