---
document_type: story
story_id: "DEFECT-RELOAD-OVERLAY-ADAPTER-FREEZE-001"
title: "reload_config non-recursive scan misses overlays; AdapterRegistry never rebuilt on reload"
wave: TBD
epic_id: engine-defects
priority: P2
status: draft
version: "0.1"
severity: MED
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pql-deficiencies.md
origin_finding: "F11 (D-1889 triage 2026-07-20)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts:
  - BC-2.16.005
  - BC-2.16.007
# BC status:
#   BC-2.16.005 (Reload-Config Tool): status: draft, lifecycle_status: active
#   BC-2.16.007 (Sensor Spec Hot Reload): status: active, lifecycle_status: active
#
# FINDING-A (MEDIUM, architect-routed): BC-2.16.005 carries status: draft while being
# cited as a governing contract for this defect. The architect or product-owner should
# confirm whether BC-2.16.005 is ready to anchor ACs before story decomposition begins.
#
# S-7.01: behavioral_contracts non-empty; status may advance to ready after ACs are authored.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: MED
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# DEFECT-RELOAD-OVERLAY-ADAPTER-FREEZE-001: reload_config non-recursive scan misses overlays; AdapterRegistry never rebuilt on reload

## Problem

Two related defects in the hot-reload path:

1. **Non-recursive scan:** `reload_config` (verified as a live method name in
   `crates/prism-mcp/src/tools/config.rs` and `crates/prism-mcp/tests/mcp_prism_describe.rs`)
   performs a non-recursive directory scan when discovering sensor spec files. Overlay
   spec files stored in subdirectories of the spec root are therefore silently missed on
   reload. This violates BC-2.16.005 §Postconditions which governs reload behavior
   (draft status — see FINDING-A note in frontmatter).

2. **AdapterRegistry not rebuilt:** After reload, the `AdapterRegistry` (verified in
   `crates/prism-core/src/error.rs` and `crates/prism-mcp/tests/bc_2_11_001_null_row_shape_test.rs`)
   is never rebuilt to reflect the newly-loaded specs. Sensors added or modified via
   overlay specs remain invisible to the query layer until process restart, making
   hot-reload effectively non-functional for overlay scenarios. This violates BC-2.16.007
   §Postconditions governing sensor spec hot-reload behavior.

The combination means that overlay-based multi-tenant sensor configurations cannot be
reloaded without restarting the process — a significant operational regression for
multi-tenant deployments.

## Origin — D-1889 Triage (F11)

**Triage date:** 2026-07-20  
**Source findings:** `findings/prism-pql-deficiencies.md`  
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
§Bucket-B table row F11

The triage capture notes that `reload_config` performs a non-recursive scan that misses
overlays and never rebuilds the `AdapterRegistry`. Coverage is PARTIAL: there is
existing behavioral contract coverage for parts of the reload path, but the overlay
discovery and registry-rebuild obligations are not fully met.

## Rule-3 Disclosure — Canonical Principle Rule 3 Violation

This defect was **flagged at D-1889 triage (2026-07-20) as a Canonical Principle Rule 3
violation**: an unanchored `boot.rs` deferral. The `reload_config` overlay-miss and
`AdapterRegistry` non-rebuild were deferred without (a) explicit human direction,
(b) a concrete future dependency making deferral necessary, or (c) attachment to a
specific future story or wave where the fix would be resolved. The deferral comment was
present in `boot.rs` but unnamed in any tracking artifact.

This stub is the **remediating registration** that satisfies condition (c): the defect
now has a trackable story ID and is no longer silently lost. The actual fix requirements
remain subject to product-owner BC amendment (BC-2.16.005 is draft; BC-2.16.007 is
active) and story-writer AC decomposition.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| BC-2.16.005 (Reload-Config Tool) | `status: draft` · `lifecycle_status: active` | §Postconditions — governs reload scan behavior and spec discovery; **draft status is a FINDING-A (MEDIUM, architect-routed): this BC must reach active status before ACs can be anchored** |
| BC-2.16.007 (Sensor Spec Hot Reload) | `status: active` · `lifecycle_status: active` | §Postconditions — governs `AdapterRegistry` rebuild after spec reload and sensor visibility in the query layer |

No governing ADR has been identified for this defect beyond existing hot-reload
architecture. The product-owner or architect should confirm whether BC-2.16.005's
`draft` status blocks dispatch.

## Routing

Route per triage: **product-owner → story-writer → implementer**

1. Product-owner confirms or advances BC-2.16.005 from `draft` to `active` (or authors
   a BC amendment covering overlay-recursive scan and `AdapterRegistry` rebuild
   obligations)
2. Story-writer decomposes ACs from the active BC(s)
3. Implementer closes both the non-recursive scan gap and the registry-rebuild gap
   under TDD

Wave assignment is TBD pending BC-2.16.005 status resolution.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, `tdd_mode` declaration, task decomposition, and story-point estimate are deferred
to the product-owner (BC amendment) and story-writer (AC decomposition). This stub
registers the defect as a trackable artifact and documents the Rule-3 violation
remediation. No implementation guidance is authored here.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (F11); Rule-3 violation remediation; FINDING-A for BC-2.16.005 draft status; no ACs or implementation guidance |
