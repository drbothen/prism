---
document_type: story
story_id: S-DEMO-004
title: "prism-bin: Multi-Org × Multi-Sensor Isolation Smoke Test — BC-3.2.001 + ADR-029 demo validation"
wave: 5
epic_id: E-DEMO
priority: P0
status: draft
version: "1.1"
level: "L4"
producer: architect
timestamp: "2026-05-29T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-10, SS-11, SS-17, SS-22]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters): multi-org isolation requires per-org adapter registration;
#     AdapterRegistry must not cross-leak between orgs.
#   SS-10 (MCP Interface): demo drives tool_query with client_id scoping per org;
#     PrismServer routes queries per org identity.
#   SS-11 (Query Execution): fan_out() resolves (org_id, sensor_id) pairs; cross-org
#     leakage would appear as data contamination in RecordBatch output.
#   SS-17 (Multi-Tenancy): BC-3.2.001 org isolation is the primary contract under test.
#   SS-22 (Binary Entrypoint): boot step 9A produces per-org adapter entries; this test
#     verifies the boot output is correct for 3-org × mixed-sensor-combo configuration.
crates_touched: [prism-bin]
target_module: prism-bin
capabilities: [CAP-001, CAP-015, CAP-029, CAP-034]
behavioral_contracts:
  - BC-3.2.001  # Multi-tenant org isolation — no cross-org data leakage; AdapterRegistry
                # keyed by (OrgId, SensorId) guarantees per-org adapter isolation.
  - BC-2.06.014  # Instance Identity Resolution at Fanout — (org_id, sensor_id) → ResolvedSensorSpec;
                 # per-org overlay means each org's query resolves to its own DTU clone endpoint.
  - BC-2.11.005  # Ephemeral Materialization — fan_out() materializes per-org; no cross-call state.
  - BC-2.01.013  # DataSource Trait — spec-driven adapters are per-org; no shared adapter across orgs.
  - BC-2.10.001  # rmcp ServerHandler — `client_id` scoping parameter in tool_query routes to
                 # the correct org's adapters.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — this test extends the parity coverage to the
            # multi-org dimension; each org's adapter must resolve to its org-scoped DTU clone.
depends_on:
  - S-DEMO-001   # Per-org adapter registration (boot step 9A) must be complete.
  - S-DEMO-002   # Single-org single-sensor E2E smoke test must pass first (build on its foundation).
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Per-org overlay loading must be complete.
blocks: []
# Historical edge scrubbed (v1.1, 2026-06-10 story-writer micro-burst): blocks
# originally carried S-DEMO-003 ("runbook should not ship until multi-org isolation
# is verified"). S-DEMO-003 MERGED via PR #176 on 2026-06-08 while this story was
# still draft — a merged story cannot be blocked, so the edge is moot and would
# only mislead the wave scheduler. The shipped-runbook-before-isolation-verification
# risk the edge encoded is now tracked by this story's own ACs (the isolation
# assertions still must pass before the demo is presented). See §Dispatch Ordering
# for the annotated historical diagram.
points: 8
# Points justification:
#   - 3-org test config setup (prism.toml with 3 orgs + 3 customers/ overlay dirs): ~1 pt
#   - DTU demo server config to serve 3 independent sensor combos: ~1 pt
#   - Per-org MCP query harness (3 × tool_query with org-scoped client_id): ~1.5 pts
#   - Cross-org isolation assertion (Org A query for Org B's sensor → AdapterNotFound/error): ~2 pts
#   - DTU per-tenant data verification (Org A data ≠ Org B data): ~1 pt
#   - CI integration + SubprocessGuard reuse from S-DEMO-002: ~0.5 pts
#   - ADR-029 overlay correctness assertion (each org uses its own DTU base_url): ~1 pt
#   Total: 8 points (~1.5-2 days)
estimated_days: 2
risk: MEDIUM
# Risk justification: The multi-org isolation logic depends on boot step 9A correctly
# iterating per-org ResolvedSensorSpec entries and keying the AdapterRegistry by
# (OrgId, SensorId). If boot step 9A merges all orgs under the same SensorSpec (losing
# org scope), isolation assertions will fail with misleading "data looks correct" outputs.
# Test design must include a deliberate cross-org probe (Org A's client_id querying
# Org B's sensor) to catch this failure mode.
acceptance_criteria_count: 10
red_gate_tests: 4
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Use distinct DTU clone instances per sensor per org-group: Org A's CrowdStrike DTU runs
    at port P1; Org B's Claroty DTU runs at port P2. Assertions verify responses from P1 contain
    org-A-specific fixture data, not org-B data. DTU demo server config creates independent
    per-sensor clone instances."
  - "Cross-org isolation probe must use the correct org_id. Test must query Org A using Org B's
    sensor_id explicitly — this must return AdapterNotFound or an explicit isolation error, not
    Org A's data for a different sensor."
  - "Reuse SubprocessGuard and wait_for_file() helpers from S-DEMO-002 tests/helpers/mod.rs
    rather than duplicating subprocess management logic."
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - "crates/prism-bin/tests/helpers/mod.rs"
  - "crates/prism-dtu-demo-server/src/main.rs"
  - "crates/prism-dtu-demo-server/src/harness.rs"
  - ".factory/specs/behavioral-contracts/BC-3.2.001-multi-tenant-isolation.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/stories/S-DEMO-001-spec-driven-sensor-adapter-and-boot-step-9a.md"
  - ".factory/stories/S-DEMO-002-e2e-subprocess-smoke-test-all-sensors.md"
  - ".factory/stories/S-CONFIG-MULTI-TENANT-OVERRIDE-001-per-org-sensor-endpoint-overlay-loading.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-004 — Multi-Org × Multi-Sensor Isolation Smoke Test

**Story ID:** S-DEMO-004
**Status:** draft
**Version:** v1.0
**Wave:** 5
**Priority:** P0
**Points:** 8

---

## Origin

New story proposed by architect (2026-05-29) to close the multi-client demo scope gap not
covered by S-DEMO-002 v1.0.

S-DEMO-002 tests a single org with all 4 sensors. The user's demo target explicitly requires:
1. Multiple client orgs registered simultaneously with different sensor combos.
2. Verification that Org A's query for Org B's sensor returns an isolation error (BC-3.2.001).
3. Proof that DTU overlays route each org to its own clone instance (ADR-029 per-org overlay).

S-DEMO-002 does NOT cover these. This story extends the demo to the multi-org dimension.

**Reference demo configuration (user-stated 2026-05-29):**
- Org A: CrowdStrike + Armis
- Org B: Claroty + Cyberint
- Org C: all 4 sensors (CrowdStrike + Armis + Claroty + Cyberint)

---

## Narrative

As the Prism platform engineering team, I want an integration test that registers 3 orgs with
different sensor combinations, drives queries for each org via the MCP tool_query interface,
and asserts that: (a) each org's query returns data from its org-scoped DTU clone endpoint,
(b) querying Org A for a sensor not registered to Org A returns an isolation error (not data),
and (c) Org C's queries to all 4 sensors all succeed independently — so that BC-3.2.001 org
isolation is proven in code before the live demo.

---

## Story-Level Goal

After this story merges:
1. A 3-org integration test exists in `crates/prism-bin/tests/e2e_multi_org.rs`.
2. The test proves AdapterRegistry correctly scopes adapters to (OrgId, SensorId) pairs.
3. Cross-org isolation is verified: Org A's `client_id` + Org B's sensor_id → error (not data).
4. Per-org ADR-029 overlay routing is verified: each org's CrowdStrike adapter uses its org-specific DTU port.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-3.2.001 | Multi-Tenant Org Isolation — no cross-org data leakage |
| BC-2.06.014 | Instance Identity Resolution at Fanout — (org_id, sensor_id) → ResolvedSensorSpec |
| BC-2.11.005 | Ephemeral Materialization — fan_out() materializes per-org; no cross-call state |
| BC-2.01.013 | DataSource Trait — spec-driven adapters are per-org |
| BC-2.10.001 | rmcp ServerHandler — client_id scoping parameter routes to correct org's adapters |

---

## Multi-Org Demo Configuration

### Org registration (prism.toml)

```toml
[[orgs]]
org_id = "<uuid-v7-org-a>"
org_slug = "org-a"

[[orgs]]
org_id = "<uuid-v7-org-b>"
org_slug = "org-b"

[[orgs]]
org_id = "<uuid-v7-org-c>"
org_slug = "org-c"
```

### Sensor combos

| Org | CrowdStrike | Armis | Claroty | Cyberint |
|-----|-------------|-------|---------|----------|
| org-a | YES | YES | NO | NO |
| org-b | NO | NO | YES | YES |
| org-c | YES | YES | YES | YES |

### customers/ overlay structure

```
customers/
  org-a/
    crowdstrike.sensor.toml   # base_url = "http://127.0.0.1:<CS_PORT_ORG_A>"
    armis.sensor.toml         # base_url = "http://127.0.0.1:<ARMIS_PORT_ORG_A>"
  org-b/
    claroty.sensor.toml       # base_url = "http://127.0.0.1:<CLAROTY_PORT_ORG_B>"
    cyberint.sensor.toml      # base_url = "http://127.0.0.1:<CYBERINT_PORT_ORG_B>"
  org-c/
    crowdstrike.sensor.toml   # base_url = "http://127.0.0.1:<CS_PORT_ORG_C>"
    armis.sensor.toml         # base_url = "http://127.0.0.1:<ARMIS_PORT_ORG_C>"
    claroty.sensor.toml       # base_url = "http://127.0.0.1:<CLAROTY_PORT_ORG_C>"
    cyberint.sensor.toml      # base_url = "http://127.0.0.1:<CYBERINT_PORT_ORG_C>"
```

Each org gets its own DTU clone instances at distinct ephemeral ports. This proves
per-org overlay routing is correct — Org A's CrowdStrike queries hit CS_PORT_ORG_A,
not CS_PORT_ORG_C, even though both orgs have CrowdStrike registered.

### DTU multi-tenancy scope

**DTU clones do NOT need to support per-tenant data at the protocol level** for this test.
Each org gets its own independent clone instance (different port = different process-scoped
state). Org isolation is proven by the port binding, not by in-process tenant partitioning.
The DTU's per-session `X-Prism-Org-Id` header routing (BC-3.2.003, implemented for Cyberint
clone) is exercised by the Cyberint queries but is not the primary isolation mechanism tested
here — port-level isolation is sufficient for the demo scope.

---

## Acceptance Criteria

### AC-001: 3-org boot registrations correct
Given: prism.toml with 3 orgs and mixed sensor overlays as in §Multi-Org Demo Configuration.
When: prism-bin starts and boot step 9A completes.
Then: AdapterRegistry contains exactly the expected count:
- org-a: 2 adapters (CrowdStrike + Armis)
- org-b: 2 adapters (Claroty + Cyberint)
- org-c: 4 adapters (all 4 sensors)
Total: 8 adapters. Verified via `boot.step9a.adapter_registry_populated` event log assertion.
(traces to BC-2.22.001 boot sequencing postcondition)
Red Gate test: `test_BC_3_2_001_multi_org_boot_registers_correct_adapter_count`

### AC-002: Org A queries return data for registered sensors only
Given: org-a is registered with CrowdStrike + Armis (not Claroty or Cyberint).
When: `tool_query "FROM crowdstrike_detections LIMIT 5" client_id="org-a"` is sent.
Then: Returns non-empty data from org-a's CrowdStrike DTU clone (port CS_PORT_ORG_A).
(traces to BC-2.06.014 instance identity resolution)

### AC-003: Org B queries return data for registered sensors only
Given: org-b is registered with Claroty + Cyberint (not CrowdStrike or Armis).
When: `tool_query "FROM claroty_assets LIMIT 5" client_id="org-b"` is sent.
Then: Returns non-empty data from org-b's Claroty DTU clone.
(traces to BC-2.06.014; BC-2.11.005)

### AC-004: Org C queries succeed for all 4 sensors independently
Given: org-c is registered with all 4 sensors.
When: Each of the 4 `tool_query` calls is sent with `client_id="org-c"`.
Then: All 4 return non-empty data; no cross-sensor data contamination.
(traces to BC-2.01.013 spec-driven adapters are per-org)
Red Gate test: `test_BC_3_2_001_org_c_all_4_sensors_return_independent_data`

### AC-005: Cross-org isolation — Org A querying Org B's sensor returns isolation error
Given: Org A has CrowdStrike + Armis registered; Cyberint is NOT registered for Org A.
When: `tool_query "FROM cyberint_alerts LIMIT 5" client_id="org-a"` is sent.
Then: Response envelope contains an error code (AdapterNotFound or SensorNotAvailableForOrg);
NO data rows are returned; NO data from Org B leaks into Org A's response.
(traces to BC-3.2.001 invariant: no cross-org data leakage)
Red Gate test: `test_BC_3_2_001_cross_org_query_returns_isolation_error`

### AC-006: Per-org ADR-029 overlay routing verified (different orgs → different DTU ports)
Given: Org A and Org C both have CrowdStrike registered, but with different `base_url` overlays
pointing to CS_PORT_ORG_A and CS_PORT_ORG_C respectively.
When: `tool_query "FROM crowdstrike_detections LIMIT 5"` is sent for org-a and then for org-c.
Then: The two requests hit different DTU clone instances (verified via DTU access log or by
serving org-specific fixture data that differs between the two clone instances).
(traces to BC-2.06.014 precondition: org-specific ResolvedSensorSpec with overlay base_url)
Red Gate test: `test_BC_2_06_014_per_org_overlay_routes_to_distinct_dtu_instances`

### AC-007: Cyberint cookie_roundtrip auth works for org-b and org-c independently
Given: org-b and org-c both have Cyberint registered but at different DTU ports.
When: `tool_query "FROM cyberint_alerts LIMIT 5"` is sent for each org.
Then: Each query succeeds with its own session cookie from the respective org's DTU clone;
the session tokens do not cross between org-b and org-c.
(traces to BC-3.2.001 session isolation; BC-2.01.013 per-org adapter construction)

### AC-008: ResponseEnvelope metadata identifies correct org and sensor
Given: A successful multi-org query for any org/sensor combination.
When: The ResponseEnvelope is inspected.
Then: `_meta.data_source` contains the correct sensor name; the response is scoped to the
querying org's data — no org identifiers from other orgs appear in the response.
(traces to BC-2.09.008 response envelope trust annotations)

### AC-009: Concurrent queries for different orgs do not interfere
Given: org-a and org-c both query CrowdStrike concurrently (sent within 100ms of each other).
When: Both responses arrive.
Then: org-a's response contains only data from CS_PORT_ORG_A; org-c's response contains only
data from CS_PORT_ORG_C; no row-level mixing occurs.
(traces to BC-2.11.005 invariant: no cross-call state; ephemeral materialization)

### AC-010: Test is gated behind `#[ignore]` with explicit CI multi-org profile
Given: Standard nextest profile runs (no DTU server available).
When: `cargo nextest run -p prism-bin` is executed.
Then: Multi-org test is skipped (marked `#[ignore]`). CI runs with `--profile e2e-multi-org`
to execute it. Comment: `// E2E-MULTI-001: requires multi-org DTU setup; un-gated via 'e2e-multi-org' profile.`
(traces to BC-2.22.001 invariant: startup deterministic and testable)

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| AdapterRegistry keyed by (OrgId, SensorId) — not (SensorId) alone | BC-3.2.001 | Test AC-005 cross-org probe fails if AdapterRegistry lacks org scope |
| Per-org ResolvedSensorSpec from overlay must be used — not base spec | ADR-029 §D2 | Test AC-006 verifies DTU port from per-org overlay, not production URL |
| No shared mutable state between concurrent org queries | BC-2.11.005 ephemeral | Test AC-009 concurrent probe |
| Cyberint session cookie scoped to per-org CookieLoginAuthProvider instance | BC-3.2.001 | Test AC-007 org-specific session token |

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-bin/tests/e2e_multi_org.rs` | CREATE | Multi-org integration test with all 10 ACs |
| `crates/prism-bin/tests/helpers/mod.rs` | MODIFY | Add `write_multi_org_config()`, `MultiOrgDtuPorts` helper types |
| `crates/prism-bin/tests/fixtures/multi-org-prism.toml.template` | CREATE | Template with 3-org config |
| `.cargo/nextest.toml` | MODIFY | Add `[profile.e2e-multi-org]` that un-ignores E2E-MULTI-001-tagged tests |

---

## Tasks

1. **Read** S-DEMO-002 test helpers (`crates/prism-bin/tests/helpers/mod.rs`) — understand `SubprocessGuard`, `wait_for_file()`, `write_demo_config()` before extending.
2. **Read** `crates/prism-dtu-demo-server/src/harness.rs` — understand how to launch multiple clone instances per sensor for different orgs (distinct ports for org-a's CrowdStrike vs org-c's CrowdStrike).
3. **Write** `write_multi_org_config()` helper — generates the 3-org `prism.toml` + per-org `customers/org-x/*.sensor.toml` overlays using ports from DTU demo server.
4. **Write Red Gate tests** in `crates/prism-bin/tests/e2e_multi_org.rs` — AC-001, AC-004 shape tests fail RED before S-DEMO-001 merges.
5. **Implement** multi-org DTU launch helper — spawns DTU server with enough sensor instances to serve 3 orgs × mixed sensors (up to 4 per org × 3 orgs = up to 12 clone instances; demo server should batch these).
6. **Implement** cross-org isolation assertion (AC-005) — sends tool_query with org-a's client_id for a sensor not registered to org-a; asserts error response, not data.
7. **Implement** per-org routing assertion (AC-006) — two orgs with CrowdStrike at different ports; assert response data differs (DTU fixture data seeded differently per clone instance).
8. **Implement** Cyberint per-org session isolation (AC-007) — two orgs' Cyberint queries; assert CookieLoginAuthProvider constructs per-org sessions independently.
9. **Implement** concurrent query test (AC-009) — use `tokio::join!` to fire org-a and org-c CrowdStrike queries simultaneously; assert no cross-contamination.
10. **Add** `[profile.e2e-multi-org]` to `.cargo/nextest.toml`.
11. **Run** `cargo nextest run -p prism-bin --profile e2e-multi-org` after S-DEMO-001 + S-DEMO-002 merge; all assertions GREEN.
12. **Run** `just check` — final pre-push gate.

---

## Open Questions

1. **DTU demo server instance count:** The demo server `prism-dtu-demo-server` supports
   spawning multiple sensor clone instances. For 3 orgs with mixed sensors, we need up to
   8 distinct CrowdStrike/Armis/Claroty/Cyberint instances (2 CS + 2 Armis + 2 Claroty + 2 Cyberint).
   Confirm that the demo server config supports this (read `harness.rs` first). If not, each
   org's DTU instances may need to be spawned as separate demo server invocations.

2. **Org-specific fixture data:** To prove AC-006 (per-org routing), the two CrowdStrike DTU
   clone instances must return distinguishably different data. The DTU clone uses shared fixture
   files by default. Either (a) seed each clone with different runtime data via `POST /dtu/configure`
   before the test, or (b) rely on the port-binding proof (org-a's adapter uses CS_PORT_ORG_A;
   if it were routing to CS_PORT_ORG_C, the request would fail due to port mismatch, not just
   return different data). Option (b) is simpler and sufficient for the isolation proof.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Org registered in prism.toml but no `customers/org-x/` overlay directory exists | Boot step 9A uses base spec URL (production URL); demo fails with connection error to production URL. Test must not exercise this case — overlays must always exist for demo orgs. |
| EC-002 | Two orgs registered with same sensor but same DTU port (config mistake) | Both orgs' adapters resolve to the same DTU clone; data is shared. This is a config error, not a platform bug. Test must use distinct ports per org. |
| EC-003 | Cross-org query (AC-005) returns empty data instead of AdapterNotFound | Empty data looks like a "soft pass" but is a BC-3.2.001 violation — the isolation error MUST be explicit. AC-005 asserts on the error code, not just "no rows". |
| EC-004 | Cyberint login step fails for org-b but succeeds for org-c | CookieLoginAuthProvider instances are independent; one failing doesn't affect the other. org-c's Cyberint query proceeds normally. |

---

## Dispatch Ordering in Critical Path

> **HISTORICAL (annotated v1.1, 2026-06-10):** the diagram below reflects the
> 2026-05-29 authoring-time plan. S-DEMO-003 has since MERGED (PR #176,
> 2026-06-08) ahead of this story, so the `blocks: [S-DEMO-003]` edge was
> scrubbed from frontmatter (see frontmatter annotation). The isolation
> assertions remain a precondition for PRESENTING the live demo — that
> sequencing is carried by the demo-objective ordering (T5 Story B → T6 → T8),
> not by a story-graph edge to an already-merged story.

```
S-DEMO-001 (boot step 9A + all 3 auth providers)
    │
    └── S-DEMO-002 (single-org 4-sensor E2E smoke test)
            │
            ├── S-DEMO-004 (multi-org isolation test) ← THIS STORY
            │
            └── S-DEMO-003 (runbook + scripts — MERGED PR #176 2026-06-08; edge historical)
```

S-DEMO-004 can run in parallel with S-DEMO-003 preparation, but S-DEMO-003 should not ship
until S-DEMO-004's isolation assertions pass. *(Historical note: S-DEMO-003 shipped first
in practice — PR #176, 2026-06-08 — under the live-demo objective's re-sequencing; the
isolation-verification obligation transfers to this story's own AC gate before demo day.)*

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-06-10 | story-writer | Moot `blocks: [S-DEMO-003]` frontmatter edge scrubbed to `blocks: []` with historical annotation — S-DEMO-003 merged PR #176 (2026-06-08) ahead of this draft story; a merged story cannot be blocked and the stale edge would mislead the wave scheduler. §Dispatch Ordering diagram + narrative annotated historical (isolation-verification obligation transfers to this story's AC gate before demo presentation). Index row already carried the note since STORY-INDEX v2.342; file now matches (story_frontmatter_index_consistency). No AC/scope changes. |
| 1.0 | 2026-05-29 | architect | Initial draft — addresses multi-client demo scope gap not covered by S-DEMO-002 v1.0 |
