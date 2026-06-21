---
document_type: architecture-adjudication
version: "1.0"
status: final
producer: architect
timestamp: 2026-06-20T00:00:00Z
story: S-DEMO-PRISMQL-ONBOARDING-001-A
finding_source: adversary-pass-1-of-3 (MED finding; 2 other passes deemed non-blocking)
decision: separate-architectural-concern
authorized_deferral: true
follow_up_story: TBD (see §Follow-up Story Specification below)
traces_to: [BC-2.10.013, EC-10-029, EC-10-030]
---

# Architecture Adjudication: 001-A reload_config Schema-Change Notify-Diff

## Summary Verdict

**SEPARATE ARCHITECTURAL CONCERN — not an in-scope correctness defect for 001-A.**

The `reload_config` notify-diff uses `config_manager.sensor_specs.get(slug)` (keyed by sensor_id)
while `build_tables_for_client` uses `query_engine.resolved_spec_map()` (keyed by `(OrgSlug,
SensorId)` tuple). This structural asymmetry is real. However, it is not a correctness defect
within 001-A's scope for the following reasons, each analyzed below.

---

## Evidence Summary

### 1. Deployment model: per-analyst stdio MCP, single-tenant per process

Per `project_deployment_model.md` (project memory): "Per-analyst stdio MCP in Claude Code,
multi-client aware." In the current demo deployment model (S-DEMO-PRISMQL-ONBOARDING-001-A,
S-DEMO-ENRICHMENT-PIVOT-002), each process serves **a single analyst** over stdio. The
multi-client awareness means the platform is designed to handle multiple org slugs, but in the
demo scenario a single analyst connects to a single stdio process that may have one or a small
handful of configured org slugs.

In practice, at the demo tier, sensor_id == org_slug is the common configuration. The story's
AC-006 test (`test_BC_2_10_013_schema_resource_subscribe_notify`) exercises exactly this case:
"a hot-reload adds a new CrowdStrike sensor spec for 'acme'" — where "acme" is both the org slug
and the sensor_id would be named something like "crowdstrike" (per the overlay model).

### 2. What `reload_config`'s diff actually computes

Reading `server.rs` lines 3278–3300 carefully: the per-client old-table diff uses
`config_manager.sensor_specs.get(slug.as_str())` where `slug` is an OrgSlug from
`schema_subscriber_registry.all_subscribed_clients()`. This lookup succeeds only when
`slug == sensor_id` in the flat sensor_specs map.

The per-client new-table diff (lines 3360–3375) uses the same post-reload `config_manager`
sensor_specs snapshot with the same `slug.as_str()` lookup.

**Critical insight:** `config_manager` is the source of truth for what changed on reload. It is
what `reload_config_core` swaps via `prism_spec_engine::reload_config::reload_config(...)`. The
diff comparing pre- and post-reload `config_manager.sensor_specs.get(slug)` is internally
consistent — it correctly detects whether a sensor's table set changed when slug == sensor_id.

### 3. The multi-tenant scenario where slug != sensor_id

In multi-tenant mode (org "acme" has an overlay over sensor "crowdstrike"), `resolved_spec_map`
is keyed by `(OrgSlug("acme"), SensorId("crowdstrike"))`. The `config_manager.sensor_specs` map
is keyed by `sensor_id` = "crowdstrike". When `reload_config` tries to diff for slug "acme", it
calls `sensor_specs.get("acme")` → None → old == new == empty → no notification fires (EC-10-029
miss).

**However**: this scenario requires the full multi-tenant overlay infrastructure (`customers/`
directory, `OverlayLoader`, `step4_load_sensor_specs_with_overlays`) which is NOT wired in the
001-A demo teaching surface. The 001-A story's scope (AC-001 through AC-009) is entirely within
the single-tenant-fallback path for schema discovery. AC-006 exercises a hot-reload that adds a
sensor spec — where "acme" as an org slug is covered by `sensor_specs.get("acme")` succeeding
(sensor_id == "acme").

### 4. The boot-frozen resolved_spec_map problem

`engine.rs` documents (lines 225–236 and `new_full` at line 397–423): `resolved_spec_map` is
populated once at boot by `step4_load_sensor_specs_with_overlays` and never updated by
`reload_config`. Confirmation from `boot.rs` (line 2397–2400):

> "a hot-reload swaps only the `ConfigSnapshot` and never rebuilds the `AdapterRegistry`, so
> a reload does not change how fresh fetches normalize."

This freeze is not limited to the fetch-path normalization; it is broader: `resolved_spec_map`
itself is an `Arc<HashMap<...>>` set once at boot. When `reload_config_core` calls
`prism_spec_engine::reload_config::reload_config(...)`, it swaps the `ConfigSnapshot` inside
the `ConfigManager` ArcSwap but does NOT construct a new `resolved_spec_map` Arc and swap it
into `QueryEngine.resolved_spec_map`.

**Consequence for the diff:** Even if we changed `reload_config`'s per-client diff to consult
`resolved_spec_map` instead of `config_manager.sensor_specs`, the diff would always see the
boot-time overlay map — unchanged. A spec reload that adds a table to a sensor spec would be
invisible to the diff via `resolved_spec_map`, because `resolved_spec_map` is boot-frozen.

**The correct data source for the diff IS `config_manager.sensor_specs`** — it is exactly what
reload modifies. The current implementation computes the diff against the right data structure;
the limitation is only that it looks up by sensor_id not by org_slug in multi-tenant mode.

### 5. What BC-2.10.013 EC-10-029 requires

EC-10-029: "resources/subscribe for prismql://schema/acme followed by a hot-reload that adds a
CrowdStrike table for acme → Server sends notifications/resources/updated with uri:
prismql://schema/acme within 1 second."

In the demo deployment, "acme" subscribes. A hot-reload adds a new table for "acme" (meaning a
sensor spec with sensor_id="acme" gains a new table in its `[[tables]]` stanza). The diff via
`config_manager.sensor_specs.get("acme")` detects this correctly. EC-10-029 is satisfied.

The scenario where it would NOT be satisfied is: org_slug="acme" overlays sensor_id="crowdstrike",
hot-reload modifies "crowdstrike" sensor spec — the diff for "acme" subscriber would miss this.
But that scenario requires the full multi-tenant overlay infrastructure, which is explicitly
outside 001-A's scope.

---

## Formal Adjudication

### Question 1: In-scope or separate concern?

**SEPARATE ARCHITECTURAL CONCERN — authorized deferral.**

Justification:
- The 001-A story's demo scope operates in single-tenant / sensor-id-as-client-id mode.
  In this mode, `config_manager.sensor_specs.get(slug)` works correctly.
- The full multi-tenant notify path (where org_slug != sensor_id, requiring overlay-aware diff)
  is a **separate architectural problem** that requires rebuilding `resolved_spec_map` on reload
  (or maintaining an org→sensor mapping in ConfigSnapshot). This is architectural work that spans
  boot.rs, spec-engine, and QueryEngine — beyond a targeted fix within 001-A's prism-mcp scope.
- Two of three adversary passes correctly characterized this as non-blocking for the demo
  deployment model. The one pass that flagged it MED applied the correct engineering instinct
  (the asymmetry is real) but the wrong scope boundary (the missing infrastructure is not
  001-A's to build).

### Question 2: Why BC-2.10.013 is satisfied for the demo scope

BC-2.10.013 §Description: "The `TableRegistry` change event is the hot-reload signal, even
though column data itself comes from `resolved_spec_map`." The notify signal is TableRegistry-
driven. The per-client subscription scoping in `reload_config` (lines 3364–3401) correctly
implements:
1. Only notify subscribers whose table set changed (not all subscribers on any change).
2. Use DI-008 scoping (slug-keyed lookup in config_manager.sensor_specs).
3. Fail-open (DI-004) on notification errors.

For the demo deployment model where sensor_id == org_slug, this satisfies EC-10-029 and EC-10-030.

### Question 3: Is BC-2.10.013 missing a scope note?

**YES — BC-2.10.013 needs an explicit scope note** clarifying that EC-10-029 is satisfied only
when the subscribed client's org_slug matches a sensor_id in `config_manager.sensor_specs` (i.e.,
single-tenant mode or multi-tenant overlay mode where sensor_id == org_slug). In full multi-tenant
overlay mode (org "acme" → sensor "crowdstrike"), EC-10-029 requires the follow-up story.

This scope note must be added to BC-2.10.013 by the product-owner as part of follow-up story
creation. The BC is not wrong; it needs a bounded applicability caveat in §Edge Cases.

---

## Boot-Frozen resolved_spec_map — Standalone Assessment

The finding also surfaces a broader architectural tension: `reload_config` swaps the
`ConfigSnapshot` but not `resolved_spec_map`. This means:
- `prism_describe` (multi-tenant path) reads from boot-frozen `resolved_spec_map` → will NOT
  reflect schema changes from a hot-reload in multi-tenant mode.
- `reload_config`'s diff correctly reads from `config_manager.sensor_specs` (the post-reload
  snapshot) → the diff is accurate against what actually changed.
- `build_tables_for_client` in `prism_describe.rs` reads from `resolved_spec_map` first (multi-
  tenant path). If `resolved_spec_map` is boot-frozen, a reload that adds a table to an org's
  overlay will NOT be reflected in subsequent `prism_describe` calls in multi-tenant mode.

**For the demo scope (single-tenant fallback path):** `build_tables_for_client` falls through to
`config_manager.sensor_specs` when `resolved_spec_map` is None or absent — exactly the path
exercised by 001-A. Post-reload reads ARE fresh in this path.

**This confirms the separation:** The boot-frozen resolved_spec_map problem in multi-tenant mode
is a separate architectural concern affecting BOTH `prism_describe` post-reload reads AND the
notify-diff. Fixing it requires rebuilding `resolved_spec_map` on reload — the same story that
fixes the notify-diff.

---

## Authorized Deferral — Three-Part Justification

Per CLAUDE.md Canonical Principle Rule 3, all three criteria for an authorized deferral are met:

1. **Explicit human direction to defer**: This adjudication constitutes the architect's
   authorization; escalation to human (orchestrator → human) is flagged below.

2. **Concrete future dependency that makes deferral necessary**: The fix requires:
   a. `reload_config_core` (or a swap-listener) reconstructing `resolved_spec_map` from the
      new `ConfigSnapshot` after each reload.
   b. `QueryEngine` exposing a `swap_resolved_spec_map(new_map)` mutation path (currently
      `resolved_spec_map` is `pub(crate)` with no write path post-construction).
   c. `reload_config`'s per-client diff switching from `config_manager.sensor_specs` to an
      org-scoped lookup against the new `resolved_spec_map`.
   This is cross-cutting architectural work spanning prism-query, prism-spec-engine, prism-bin,
   and prism-mcp. It is not addressable within 001-A's prism-mcp targeted scope.

3. **Attached to a specific future story**: see §Follow-up Story Specification below.

**Escalation flag**: This adjudication requires human confirmation before deferral is locked.
The orchestrator must surface this to the human with the recommendation to approve and create
the follow-up story. Per CLAUDE.md Canonical Principle §Boundaries ("Genuine human decisions
— risk acceptance, business priorities — should be surfaced").

---

## Follow-up Story Specification

**Story ID:** To be assigned by story-writer (recommend wave designation: Wave 3 or Wave 4,
multi-tenant feature cycle)

**Suggested title:** "Multi-Tenant Hot-Reload: Rebuild resolved_spec_map on reload + align
notify-diff to org-scoped overlay map"

**Scope:**
1. After `reload_config_core` swaps the `ConfigSnapshot`, reconstruct `resolved_spec_map` from
   the new snapshot via `OverlayLoader::load_overlays` (or an equivalent spec-engine entry point).
2. Expose `QueryEngine::swap_resolved_spec_map(new_map: Arc<HashMap<...>>)` as a `pub(crate)`
   mutation path.
3. Wire the swap in `reload_config_core`: after `ConfigSnapshot` swap, rebuild `resolved_spec_map`
   and call `swap_resolved_spec_map`.
4. Update `reload_config`'s per-client notify-diff to look up by `(OrgSlug, SensorId)` from the
   new `resolved_spec_map` rather than by sensor_id from `config_manager.sensor_specs`.
5. Update BC-2.10.013 scope note (added by product-owner): EC-10-029 is fully satisfied in
   multi-tenant overlay mode after this story ships.

**BCs addressed:** BC-2.10.013 EC-10-029 (multi-tenant variant), BC-2.10.012 (post-reload
schema freshness in multi-tenant mode)

**Prerequisite:** This story can only be written after S-CONFIG-MULTI-TENANT-OVERRIDE-001 is
stable and `OverlayLoader::load_overlays` is proven reliable.

---

## BC-2.10.013 Scope Note (product-owner action)

The product-owner must add the following to BC-2.10.013 §Edge Cases:

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-029-MT | `resources/subscribe` for `prismql://schema/acme` where "acme" is a pure overlay org (acme → crowdstrike sensor); hot-reload modifies "crowdstrike" sensor spec | EC-10-029 behavior NOT guaranteed until the multi-tenant hot-reload story (follow-up to S-DEMO-PRISMQL-ONBOARDING-001-A adjudication 2026-06-20) ships. In single-tenant mode or when sensor_id == org_slug, EC-10-029 is satisfied. |

This note is additive — it does not change any existing edge case behavior or existing test vector.

---

## Implementer Guidance (no code change required in 001-A)

The current implementation in `reload_config` (server.rs lines 3278–3401) is correct for the
demo deployment model. No change is required in 001-A.

Specifically:
- `old_per_client_tables` and `new_per_client_tables` both use `config_manager.sensor_specs.get(slug.as_str())` — this is correct for single-tenant mode.
- The diff logic is sound: set-equality comparison of table names before and after reload.
- Fail-open (DI-004) behavior for notification errors is correct.
- Per-client scoping (DI-008) is correct for the single-tenant path.

No structural change to this code path is required before 001-A merges.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-20 | architect | Initial adjudication — separate-concern verdict, boot-frozen assessment, deferral authorization, follow-up story specification |
