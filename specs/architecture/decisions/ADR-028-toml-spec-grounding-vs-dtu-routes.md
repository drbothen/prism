---
document_type: adr
adr_id: "ADR-028"
title: "TOML Spec URLs and auth_type Ground Against DTU Clone Routes (Real-API Canonical), Not Production Rust Adapter URLs"
status: Proposed
date: "2026-05-20"
modified: "2026-05-20"
version: "1.6"
producer: architect
subsystems_affected: [SS-01, SS-07, SS-16, SS-17]
supersedes: ["ADR-026 §D3 (partial — auth_type_name() return values for Cyberint/Claroty/Armis non-CrowdStrike sensors)"]
superseded_by: null
amends: null
anchor_stories: [PLUGIN-MIGRATION-001-D, PLUGIN-MIGRATION-001-A, PLUGIN-MIGRATION-001-B, PLUGIN-MIGRATION-001-C, PLUGIN-MIGRATION-001-E]
related_adrs: [ADR-003, ADR-023, ADR-027]
related_bcs: [BC-2.16.013, BC-2.16.001, BC-2.16.009]
locked_decisions: ["D-737 Decision 1", "D-737 Decision 4"]
wiring_deferred_to: null
---

# ADR-028: TOML Spec URLs and auth_type Ground Against DTU Clone Routes (Real-API Canonical), Not Production Rust Adapter URLs

## Status

Proposed 2026-05-20, v1.0 (initial proposal version; current frontmatter v1.6 per §Changelog). Locks D-737 Decisions 1 and 4 as a durable architectural principle. Will be promoted to ACCEPTED after PLUGIN-MIGRATION-001-D LOCAL adversarial cascade reaches 3-CLEAN convergence per ADR-021 promotion lifecycle.

---

## Context

### The Plugin-Migration Saga

ADR-023 (Plugin-Only Sensor Architecture) mandates that all sensor definitions be expressed as declarative TOML specs driven through the plugin runtime, with hardcoded Rust adapters removed. PLUGIN-MIGRATION-001 is the saga that executes this transition across 8 stories (001-A through 001-H).

PLUGIN-MIGRATION-001-D ("Author 4 Production TOML Sensor Specs — Reverse-Engineered + DTU-Parity Tests") is the first deliverable Wave-1 story. It authors the canonical TOML sensor spec files for CrowdStrike, Claroty, Cyberint, and Armis, and proves them correct via DTU parity tests.

### Temporary Status of the Legacy Rust Adapters

The four legacy auth modules in `crates/prism-sensors/src/auth/{crowdstrike,claroty,cyberint,armis}.rs` exist as temporary scaffolding. Their only remaining purpose is to serve as a reference implementation for parity verification during PLUGIN-MIGRATION-001-D. PLUGIN-MIGRATION-001-A deletes them entirely. After 001-A merges, sensors run exclusively from TOML specs through the plugin runtime.

### The Latent Adapter Bug Exposed by Pass-4

During the LOCAL spec-level adversarial cascade for PLUGIN-MIGRATION-001-D, pass-3 closures (F-LP3-CRIT-002, -003, -HIGH-001, -HIGH-002) re-grounded BC-declared URL paths against the production Rust adapter code. Pass-4 (2026-05-20) surfaced a systemic regression: the Rust adapters themselves have simplified URL paths that do not match the real third-party APIs. All four sensors exhibited mismatches when cross-checked against the DTU clone route registrations:

| Sensor | Adapter URL (simplified) | DTU clone route (real-API) |
|---|---|---|
| CrowdStrike detections query | `/queries/detections` | `/detects/queries/detects/v1` |
| CrowdStrike detections summary | `/entities/detections/GET` | `/detects/entities/summaries/GET/v1` |
| CrowdStrike devices query | `/queries/devices` | `/devices/queries/devices/v1` |
| CrowdStrike devices detail | `/entities/devices/GET` | `/devices/entities/devices/v2` |
| Cyberint alerts | `/api/alerts` | `/api/v1/alerts` |

In addition, the legacy Cyberint adapter declares `auth_type_name() -> "bearer_static"` but its actual HTTP flow uses cookie-based session auth (`reqwest` cookie store; per `crates/prism-sensors/src/auth/cyberint.rs::CyberintAdapter::new()` (cookie-store `reqwest::Client::builder().cookie_store(true).build()` construction; consumed by `CyberintAdapter::get_page()` in the per-page fetch loop)). The legacy Claroty adapter has the inverse problem: `auth_type_name() -> "cookie_roundtrip"` but uses `Authorization: Bearer` headers. These are latent label/behavior bugs in code that PLUGIN-MIGRATION-001-A deletes.

### DTU Clones as Real-API Models

Per ADR-003 (DTU Reset Lookup and Fidelity Auth), DTU clones in `crates/prism-dtu-{sensor}/` model the real third-party API with the fidelity required to serve as the integration test environment. DTU routes are derived from real API documentation, not from the legacy adapter code. The DTU clone route registrations are the executable reference for what the real API accepts.

### CLAUDE.md Source-of-Truth Precedence

CLAUDE.md §Source-of-Truth Precedence rule 7 states: "For code-vs-spec conflicts: the SPEC wins. Code is brought into alignment via fix-burst or follow-up story, not the other way around." This ADR extends that principle to the specific case where the code reference used to author a spec is itself wrong: the spec must be grounded against the correct reference (DTU clone routes), and latent bugs in the wrong reference (legacy adapter URLs) are irrelevant to the spec's correctness.

---

## Decision

### D1 — URL Grounding Rule

TOML sensor spec URL paths (`base_url`, per-table `path` fields) MUST be derived from DTU clone route registrations (`crates/prism-dtu-{sensor}/src/routes/*.rs` and `clone.rs`), which themselves model the real third-party API. The authoritative sources are:

- **CrowdStrike:** `crates/prism-dtu-crowdstrike/src/routes/mod.rs` route table
- **Claroty:** `crates/prism-dtu-claroty/src/clone.rs` `build_router()` method
- **Cyberint:** `crates/prism-dtu-cyberint/src/clone.rs` `build_router()` method
- **Armis:** `crates/prism-dtu-armis/src/clone.rs` `build_router()` method

Production Rust adapter URL paths in `crates/prism-sensors/src/auth/{sensor}.rs` are NOT a grounding reference for TOML spec URL declarations. They are deleted by PLUGIN-MIGRATION-001-A.

Any spec artifact (BC, story, holdout scenario, test spec) that must cite an endpoint path for these sensors MUST cite the DTU clone route, not the adapter code.

### D2 — auth_type Grounding Rule

**Supersedes ADR-026 §D3 (partial — `auth_type_name()` return values for non-CrowdStrike sensors; effective at PLUGIN-MIGRATION-001-A merge):** ADR-026 §D3 mandated the legacy `auth_type_name()` returns (`cyberint="bearer_static"`, `claroty="cookie_roundtrip"`, `armis="api_key"`) that this ADR identifies as latent label bugs misaligned with the underlying DTU enforcement behavior. ADR-026 §D3 is amended by this ADR effective with PLUGIN-MIGRATION-001-A merge. CrowdStrike's `"oauth2_client_credentials"` value is correct under both ADRs and stays unchanged. The Red Gate test `crates/prism-sensors/src/auth/mod.rs::test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing` MUST be amended in PLUGIN-MIGRATION-001-A scope to assert the corrected DTU-grounded values per §D6.

TOML sensor spec `auth_type` values MUST be derived from DTU clone authentication enforcement behavior, which reflects the real third-party API's auth contract. The authoritative sources are the DTU clone middleware and handler implementations:

- **Cyberint:** `crates/prism-dtu-cyberint/src/routes/alerts.rs::extract_session_token()` enforces cookie-based session auth (extracts `cyberint_session` cookie) → spec declares `auth_type = "cookie_roundtrip"`
- **Claroty:** `crates/prism-dtu-claroty/src/clone.rs` enforces `Authorization: Bearer` header → spec declares `auth_type = "bearer_static"`
- **CrowdStrike:** OAuth2 client credentials flow per DTU enforcement → spec declares `auth_type = "oauth2_client_credentials"`
- **Armis:** Bearer token header per DTU enforcement (HTTP 403 on missing/invalid `Authorization: Bearer {non-empty}` per `crates/prism-dtu-armis/src/lib.rs` module-level `//!` doc-comment (Armis Centrix BearerStatic API contract)) → spec declares `auth_type = "bearer_static"`. Legacy `ArmisAuth::auth_type_name()` returns `"api_key"` — this is the latent label bug §D2 was authored to immunize against.

Legacy `auth_type_name()` return strings in production Rust adapters are NOT a grounding reference. The observed code-vs-label inconsistencies (Cyberint adapter uses cookies but `auth_type_name()` returns `"bearer_static"`; Claroty adapter uses bearer but `auth_type_name()` returns `"cookie_roundtrip"`) are latent bugs in code deleted by PLUGIN-MIGRATION-001-A. No tech-debt entry is required for these bugs; they become moot at deletion.

### D3 — Parity Reference OCSF Grounding Rule

Parity tests that verify TOML plugin output against a known-good OCSF shape MUST load their reference OCSF output from committed fixture JSON files at:

```
crates/prism-dtu-{sensor}/fixtures/parity/reference-ocsf/<table>.json
```

Fixtures are recorded once by running the legacy adapter against the DTU clone (capturing real-API-shaped responses, not adapter-simplified responses). After PLUGIN-MIGRATION-001-A deletes the legacy adapters, the committed fixture JSON remains as the permanent parity reference.

Parity tests MUST NOT require a `prism-sensors` dev-dep on `prism-spec-engine` at test runtime. The fixture mechanism eliminates this dependency. Story §Forbidden Dependencies (`prism-sensors` blocked in `prism-spec-engine`) remains intact.

### D4 — Adapter-as-Reference Is Forbidden

No future spec artifact (BC, story, holdout scenario, test spec) may cite `crates/prism-sensors/src/auth/{sensor}.rs` symbols as ground-truth for URL paths, auth flows, or canonical OCSF shapes. The correct citation is:

- URL paths: DTU clone route table (per D1)
- Auth behavior: DTU clone enforcement middleware (per D2)
- OCSF shapes: committed fixture JSON (per D3)

This prohibition applies to all future PLUGIN-MIGRATION stories (001-A through 001-H) and any subsequent Wave-2 cleanup stories.

### D5 — DTU Clones Must Precede Spec

Where a real third-party endpoint must be added to a TOML sensor spec (e.g., a new table mapping), the DTU clone MUST first be extended to register that route. Spec follows DTU; a spec entry for a URL path that has no corresponding DTU route registration is an architectural violation.

**Known gaps identified by pass-4 (flagged to orchestrator for follow-up; NOT a blocker for PLUGIN-MIGRATION-001-D cascade convergence):**

| Gap | Description | Recommended resolution |
|---|---|---|
| CrowdStrike `incidents` | No DTU route registered for incidents; real Falcon Detects/Incidents API exists | Extend `prism-dtu-crowdstrike` with incidents routes in a follow-up story; OR remove incidents table from 001-D BC scope |
| Claroty `assets` table | DTU has `/api/v1/devices`, not `/api/v1/assets`; xDome API has both devices and assets endpoints | Extend `prism-dtu-claroty` with `/api/v1/assets` OR reconcile that Claroty "assets" table maps to `/api/v1/devices` (table name vs endpoint may differ) |
| Armis `/api/v1/search` | DTU has `/api/v1/devices` + `/api/v1/alerts`; Armis Search API does exist in real API | Extend `prism-dtu-armis` with `/api/v1/search` OR split Armis tables to per-entity endpoints matching current DTU routes |

Pass-5 adversarial review will surface these gaps as findings if not resolved before then. The orchestrator must open follow-up stories for DTU clone extension. Architect recommendation: extend DTU clones (not strip BC scope), because the real APIs DO expose these endpoints and fidelity is the project commitment per ADR-003.

### D6 — PLUGIN-MIGRATION-001-A Scope Expansion (Auth Module Migration)

**Per user Path A adjudication (D-747):** PLUGIN-MIGRATION-001-A scope EXPANDS to include:

1. **Rewrite of `{Cyberint,Claroty,Armis}Auth::auth_type_name()` returns** to match DTU-grounded values per §D2:
   - `CyberintAuth::auth_type_name()` → `"cookie_roundtrip"` (corrected from `"bearer_static"`)
   - `ClarotyAuth::auth_type_name()` → `"bearer_static"` (corrected from `"cookie_roundtrip"`)
   - `ArmisAuth::auth_type_name()` → `"bearer_static"` (corrected from `"api_key"`)
   - `CrowdStrikeAuth::auth_type_name()` → `"oauth2_client_credentials"` (unchanged; correct under both ADRs)
2. **Amendment of Red Gate test** `crates/prism-sensors/src/auth/mod.rs::test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing` to assert the new DTU-grounded values for Cyberint, Claroty, and Armis. The CrowdStrike assertion is unchanged.
3. **Bidirectional supersession linkage:** ADR-026 §D3 received `superseded_by:` linkage to ADR-028 **(applied simultaneously with this §D6 authoring in FB-IMPL-P13-ARCH; reflected in ADR-026 v1.30 frontmatter)**.
4. **Scope authority:** This expansion is in-scope for PLUGIN-MIGRATION-001-A under user Path A adjudication (D-747). The implementer and test-writer for PLUGIN-MIGRATION-001-A must treat §D6 as a binding constraint alongside the story's existing AC list.

Until PLUGIN-MIGRATION-001-A merges, code in `crates/prism-sensors/src/auth/{cyberint,claroty,armis}.rs` continues to return the ADR-026 §D3 values; the Red Gate test asserts those legacy values. This is the LIVE contract through the migration window.

---

## Consequences

### Positive

- **Parity tests give the correct verdict.** When the plugin-driven path produces the right DTU-aligned URL and the legacy adapter does not, parity tests will FAIL for the legacy path and PASS for the plugin path. This is the signal PLUGIN-MIGRATION-001 requires: it proves the plugin-driven implementation is correct and the legacy adapter is buggy, directly justifying the deletion in 001-A.

- **Durable traceability for all future PLUGIN-MIGRATION stories.** Stories 001-A through 001-H can cite this ADR for the grounding rule rather than re-adjudicating it per story.

- **Eliminates per-cascade architectural overhead.** Without this ADR, each adversarial pass in each story must re-debate which reference grounds the spec contract. That overhead cascaded into 3 fix-bursts and a pass-4 BLOCKED-soft checkpoint for 001-D alone.

- **Spec-wins precedent applied consistently.** CLAUDE.md §Source-of-Truth Precedence #7 is applied at the reference selection level: the correct reference is DTU (executable, CI-verifiable), not adapter code (latent bugs, deleted).

### Negative

- **Three DTU route gaps must be reconciled before full BC scope can be verified.** CrowdStrike incidents, Claroty `/api/v1/assets`, and Armis `/api/v1/search` have no DTU registration. Until DTU clones are extended (or BC table scope is adjusted), the corresponding spec entries cannot be verified by parity tests. These are follow-up stories, not blockers for 001-D.

- **One-time fixture recording required.** Committed fixture JSON files (`~12` fixtures: 4 sensors × ~3 tables) must be recorded once by running the legacy adapter against the DTU clone before 001-A deletes the adapter. This is bounded and manual but must be sequenced correctly: fixture recording is a prerequisite task within PLUGIN-MIGRATION-001-D.

---

## Alternatives Considered

### Alt 1: Ground TOML spec URLs against the production Rust adapter code

**Rejected.** The production Rust adapters have latent URL simplification bugs. Grounding the spec against the adapter means the parity tests would verify buggy-plugin-code against buggy-legacy-code and produce a false PASS. The whole point of PLUGIN-MIGRATION-001-D parity tests is to validate that the plugin-driven path correctly handles real third-party API shapes. Grounding against the DTU is the only way to achieve that.

### Alt 2: Ground against real third-party API documentation directly

**Rejected.** Real third-party API documentation (Falcon, xDome, Cyberint, Armis) is not executable in CI. The DTU clones exist precisely to provide an executable, offline, reproducible verification surface for real-API shapes. Grounding against docs and grounding against DTU routes should be equivalent; where they diverge, a DTU extension story is the correct resolution (D5 above).

### Alt 3: Defer the URL grounding decision to PLUGIN-MIGRATION-001-A

**Rejected.** PLUGIN-MIGRATION-001-D ships the TOML sensor specs AND the parity tests that validate them. If the specs contain wrong URLs when 001-D ships, the parity tests cannot validate the plugin runtime — they would test the plugin's ability to hit nonexistent routes. Deferral would mean 001-D ships with unverifiable acceptance criteria, which is a production-grade violation under CLAUDE.md §Canonical Principle.

---

## Related Decisions

- **Supports ADR-023** (Plugin-Only Sensor Architecture — TOML Specs as Declarative Baseline): This ADR specifies the authoritative grounding source for the TOML specs that ADR-023 mandates.
- **Supports ADR-003** (DTU Reset Lookup and Fidelity Auth): This ADR operationalizes the ADR-003 fidelity commitment by requiring that TOML specs be grounded against DTU route registrations rather than legacy code.
- **Supports ADR-027** (CustomAdapter Rust Trait Same-Burst Removal): The prohibition on citing adapter code as a reference (D4) is consistent with ADR-027's deletion of the adapter surface.
- **References CLAUDE.md §Source-of-Truth Precedence #7:** Spec wins on code-vs-spec conflict; this ADR extends the principle to spec-authoring reference selection.
- **References D-737 in `.factory/STATE.md`:** User-adjudicated Decisions 1 and 4 in STATE.md v7.424 frontmatter `architectural_decisions_locked`; this ADR is the durable codification of those decisions.
- **Anchored by BC-2.16.013** (Bundled Sensor Spec Authoring and DTU-Parity Verification): The BC postconditions cite URL paths and auth_type values that must comply with this ADR.
- **Referenced by PLUGIN-MIGRATION-001-D story:** Story §Implementation Notes and §Task Breakdown will cross-reference ADR-028 as the grounding authority for sensor spec URL and auth_type declarations.

---

## Changelog

| Version | Date | Author | Summary |
|---|---|---|---|
| 1.6 | 2026-05-20 | architect | FB-IMPL-P14-ARCH: F-LP14-MED-002 closure — §Status self-cite "current frontmatter v1.4" advanced to "current frontmatter v1.6" (stale after v1.5 bump in P13; same defect class as F-LP10-LOW-001). F-LP14-MED-003 closure — §D6 Action 3 parenthetical rewritten: future-tense "applied in the PLUGIN-MIGRATION-001-A merge burst" replaced with realized past-tense "applied simultaneously with this §D6 authoring in FB-IMPL-P13-ARCH; reflected in ADR-026 v1.30 frontmatter". POL-29 self-verification greps: CLEAN. |
| 1.5 | 2026-05-20 | architect | Pass-13 FB-IMPL-P13-ARCH per user Path A adjudication (D-747): `supersedes:` frontmatter field added (ADR-026 §D3 partial — non-CrowdStrike `auth_type_name()` returns). §D2 supersession prefix paragraph added: explicitly supersedes ADR-026 §D3 for Cyberint/Claroty/Armis `auth_type_name()` return values effective at PLUGIN-MIGRATION-001-A merge; CrowdStrike `"oauth2_client_credentials"` unchanged. New §D6 documents PLUGIN-MIGRATION-001-A scope expansion: rewrite three `Auth::auth_type_name()` returns to DTU-grounded values + amend Red Gate `test_BC_2_01_016_003` + bidirectional supersession linkage. F-LP13-HIGH-001 closure. |
| 1.4 | 2026-05-20 | architect | Pass-7 FB-IMPL-P7 — §Context cyberint symbol-path mis-anchor corrected: `cyberint.rs::CyberintAuth::get_page` (HALLUCINATION — wrong type namespace; method belongs to `CyberintAdapter`, not `CyberintAuth`) → `cyberint.rs::CyberintAdapter::new()` (cookie-store `reqwest::Client::builder().cookie_store(true).build()` construction) + `::get_page()` consumption. Semantic claim corrected: cookie-store is BUILT in `CyberintAdapter::new()` (not "established in per-page fetch loop"). F-LP7-HIGH-001 closure. Root cause: FB-IMPL-P6 propagated wrong type namespace from pass-6 review without grep-verifying symbol against codebase. Going-forward discipline: ALL symbol-path anchor replacements MUST be grep-verified against `crates/` before commit (TD-VSDD-059 paper-fix variant; TD-VSDD-091 anti-volatile-pin). Pass-10 FB-IMPL-P10 — §Status historical-anchor disambiguation: "Proposed 2026-05-20, v1.0" → "Proposed 2026-05-20, v1.0 (initial proposal version; current frontmatter v1.4 per §Changelog)" to prevent reader confusion about current revision. F-LP10-LOW-001 closure (POL-29 body-frontmatter coherence axis). |
| 1.3 | 2026-05-20 | architect | Pass-6 FB-IMPL-P6 — POL-25 expanded sibling-anti-pattern sweep. §D2 Armis row: `crates/prism-dtu-armis/src/lib.rs:16-17` → `crates/prism-dtu-armis/src/lib.rs` module-level `//!` doc-comment (Armis Centrix BearerStatic contract). §Context cyberint cite: `crates/prism-sensors/src/auth/cyberint.rs:155` → `::CyberintAuth::get_page` symbol path. TD-VSDD-091 anti-volatile-pin. F-LP6-LOW-001 closure (architect scope; PO + SW closing sibling sites in parallel bursts). |
| 1.2 | 2026-05-20 | architect | Pass-5 fix-burst — §D2 Cyberint row symbol-anchored: `prism-dtu-cyberint/src/routes/alerts.rs:43-46` → `::extract_session_token()` per TD-VSDD-091. F-LP5-LOW-001 closure (POL-25 sibling sweep — BC-2.16.013/HS-015 already fixed by PO this burst). |
| 1.1 | 2026-05-20 | architect | Pass-5 fix-burst — §D2 Armis row corrected from `"api_key"` (legacy adapter `auth_type_name()` return) to `"bearer_static"` (DTU `Authorization: Bearer` enforcement per `prism-dtu-armis/src/lib.rs:16-17`). The original v1.0 §D2 Armis row was itself the latent label bug §D2 was authored to immunize against — fresh-context adversary surfaced the contradiction. F-LP5-HIGH-001. |
| 1.0 | 2026-05-20 | architect | Initial version — locks D-737 Decisions 1 and 4; enumerates 5 decision rules; documents 3 DTU gap follow-ups |
