---
document_type: architect-decision-note
subject: TD-S-PLUGIN-PREREQ-A-004 close-as-superseded ratification
adjudicator: architect
adjudicated_at: 2026-06-10T00:00:00Z
source: 2026-06-10 full-codebase review package, recommendation ⑩ (human-approved as part of 14-recommendation package)
td_id: TD-S-PLUGIN-PREREQ-A-004
disposition: CLOSE-AS-SUPERSEDED
consumed_by: state-manager (TD register closure citation)
---

# Architect Ratification — TD-S-PLUGIN-PREREQ-A-004 Close-as-Superseded

TD-S-PLUGIN-PREREQ-A-004 closure criterion superseded by BC-2.22.001 AC-006; non-silent
requirement satisfied by boot.step9a structured events; approved by human 2026-06-10.
The TD's acceptance criterion (when boot step 8 is wired, emit a fatal `BootError` if
`AdapterRegistry::is_empty()`) was written before the step-9A design existed; the
shipped contract is intentionally different and strictly satisfies the TD's underlying
intent — that an empty registry must never propagate as *silent* empty results. Per
BC-2.22.001 and AC-006, an empty spec catalog at boot step 9A
(`crates/prism-bin/src/spec_driven_adapter.rs::step9a_populate_adapter_registry`) emits
the explicit structured event `boot.step9a.adapter_registry_populated` with
`sensor_count = 0` / `org_count = 0` plus a human-readable "spec catalog is empty — 0
adapters registered" message, and boot continues by design (an MSSP analyst session with
zero configured sensors is a valid, observable state, not a fatal one). Defense-in-depth
is retained at the query layer: the materialization pipeline's `is_empty()`
short-circuit (prism-query `materialization.rs`) prevents an empty registry from
masquerading as a successful fan-out. A fatal-on-empty assertion would now contradict
the active BC-2.22.001 contract; the spec wins (Source-of-Truth Precedence Rule 7).
State-manager: close TD-S-PLUGIN-PREREQ-A-004 in the tech-debt register citing this
note; closure class is SUPERSEDED-BY-SPEC, not FIXED.
