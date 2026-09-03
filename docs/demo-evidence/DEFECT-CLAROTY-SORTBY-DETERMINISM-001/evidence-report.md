---
story_id: DEFECT-CLAROTY-SORTBY-DETERMINISM-001
title: "Demo Evidence — Claroty xDome sort_by determinism (7 tables)"
generated: "2026-09-03"
live_data_free: true
recording_tool: VHS 0.11.0
font: "FiraCode Nerd Font Mono"
---

# Demo Evidence: DEFECT-CLAROTY-SORTBY-DETERMINISM-001

## Live-Data-Free Declaration (D-2410)

ALL recordings in this directory are **live-data-free** and **synthetic/mock only**.

- **No live xDome connections.** No requests to the monroe tenant or any real xDome instance.
- **No tenant data captured.** No device UIDs, CVE IDs, IP addresses, audit log content, or any real row values appear in any recording.
- **Evidence source A:** `claroty.sensor.toml` — the static TOML spec file, which contains the `body_template` strings including the `sort_by` arrays. This is configuration data, not tenant data.
- **Evidence source B:** `show_sort_by.py` — Python helper that reads the TOML and extracts the `sort_by` arrays. Output is derived solely from the spec file.
- **Evidence source C:** `demo_error_path.py` — Synthetic demonstration of the pre-fix defect condition using hardcoded example body templates (no real data).
- **Evidence source D:** `cargo nextest run -E 'test(obs)'` — 3 wiremock-backed OBS wire tests that use a localhost mock server (no real network). Test bodies contain only the synthetic sort_by arrays, not tenant data.

---

## AC Coverage Map (7/7)

| AC | Table | BC / EC anchor | sort_by emitted | Recording | Type |
|----|-------|----------------|-----------------|-----------|------|
| AC-001 | `claroty_vulnerabilities` | BC-2.16.015 §Post §1 / EC-016-015-009 | `[adjusted_vulnerability_score desc, name asc]` | AC-001-vulnerabilities-sort_by | VHS (success + error) |
| AC-002 | `claroty_audit_logs` | BC-2.16.013 §Post §1 / EC-016-013-011 | `[timestamp asc]` — timestamp-only | AC-002-audit_logs-sort_by | VHS (success + error) |
| AC-003 | `claroty_server_interfaces` | BC-2.16.019 §Post §1 / EC-016-019-007 | `[server_name asc, interface_name asc]` | AC-003-server_interfaces-sort_by | VHS (success + error) |
| AC-004 | `claroty_organization_zones` | BC-2.16.020 §Post §1 / EC-016-020-011 | `[zone_name asc]` | AC-004-organization_zones-sort_by | VHS (success + error) |
| AC-005 | `claroty_organization_zone_policies` | BC-2.16.020 §Post §2 / EC-016-020-012 | `[policy_name asc]` | AC-005-zone_policies-sort_by | VHS (success + error) |
| AC-006 | `claroty_organization_firewall_groups` | BC-2.16.021 §Post §1 / EC-016-021-011 | `[firewall_group_name asc]` | AC-006-firewall_groups-sort_by | VHS (success + error) |
| AC-007 | `claroty_organization_firewall_policies` | BC-2.16.021 §Post §2 / EC-016-021-012 | `[policy_name asc]` | AC-007-firewall_policies-sort_by | VHS (success + error) |

**Coverage: 7/7 ACs. All acceptance criteria have both success and error path recordings.**

---

## Recording Index

### Per-AC Recordings (success + error paths)

Each per-AC recording demonstrates two paths:

1. **Success path** (`show_sort_by.py <table>`): Reads `claroty.sensor.toml` via `tomllib`, extracts the `body_template` for the named fetch step, parses the embedded JSON, and pretty-prints the `sort_by` array. This proves the deterministic sort_by is present in the TOML body_template for that table.

2. **Error path** (`demo_error_path.py <table>`): Demonstrates the pre-fix defect condition. Shows a synthetic body_template WITHOUT `sort_by` (representing the state before this fix), then runs the same assertion logic as RG-001..RG-007. Outputs `DEFECT DETECTED: 'sort_by' absent...` proving the detection mechanism would catch a regression.

| File | AC | What it shows |
|------|----|---------------|
| `AC-001-vulnerabilities-sort_by.gif` | AC-001 | vulnerabilities sort_by = `[adjusted_vulnerability_score desc, name asc]` + error detection |
| `AC-001-vulnerabilities-sort_by.webm` | AC-001 | (archival copy) |
| `AC-002-audit_logs-sort_by.gif` | AC-002 | audit_logs sort_by = `[timestamp asc]` (timestamp-only canonical, RETIRED note shown) + filter_by coexistence + error detection |
| `AC-002-audit_logs-sort_by.webm` | AC-002 | (archival copy) |
| `AC-003-server_interfaces-sort_by.gif` | AC-003 | server_interfaces composite sort_by = `[server_name asc, interface_name asc]` + error detection |
| `AC-003-server_interfaces-sort_by.webm` | AC-003 | (archival copy) |
| `AC-004-organization_zones-sort_by.gif` | AC-004 | organization_zones sort_by = `[zone_name asc]` + error detection |
| `AC-004-organization_zones-sort_by.webm` | AC-004 | (archival copy) |
| `AC-005-zone_policies-sort_by.gif` | AC-005 | zone_policies sort_by = `[policy_name asc]` + error detection |
| `AC-005-zone_policies-sort_by.webm` | AC-005 | (archival copy) |
| `AC-006-firewall_groups-sort_by.gif` | AC-006 | firewall_groups sort_by = `[firewall_group_name asc]` + error detection |
| `AC-006-firewall_groups-sort_by.webm` | AC-006 | (archival copy) |
| `AC-007-firewall_policies-sort_by.gif` | AC-007 | firewall_policies sort_by = `[policy_name asc]` + error detection |
| `AC-007-firewall_policies-sort_by.webm` | AC-007 | (archival copy) |

### Wire-Level Recording (covers all 7 ACs)

| File | Covers | What it shows |
|------|--------|---------------|
| `AC-WIRE-001-obs-wire-sort_by-serialized.gif` | All 7 ACs | 3 OBS wire tests passing via wiremock — proves the serialized POST bodies sent by `build_request` contain `sort_by` (offset/limit injection does NOT clobber it) |
| `AC-WIRE-001-obs-wire-sort_by-serialized.webm` | All 7 ACs | (archival copy) |

Wire tests run:
- `test_obs1_vulnerabilities_build_request_emits_sort_by` — POST to `/api/v1/vulnerabilities/` asserts `sort_by` (2-element, DESC-first), `offset` 0, and `limit` 1000 in serialized body
- `test_obs1_audit_logs_build_request_emits_sort_by_with_filter_and_pagination` — POST to `/api/v1/audit_log/get` asserts all four keys (`filter_by`, `sort_by`, `offset`, `limit`) coexist simultaneously
- `test_obs3_remaining_tables_build_request_emits_sort_by` — wire-level coverage for `server_interfaces`, `organization_zones`, `zone_policies`, `firewall_groups`, `firewall_policies`

---

## AC-002 Audit Note: timestamp-only canonical

AC-002 (`claroty_audit_logs`) uses the **timestamp-only canonical** sort:
`[{"field":"timestamp","order":"asc"}]`

**The compound form with `id` is RETIRED** — live-validated 2026-09-02 on the monroe tenant:
- Both compound variants (`[timestamp asc, id asc]` and `[timestamp asc, id desc]`) returned HTTP 200 with **0 rows** against the live xDome audit_log endpoint.
- `id` is NOT a valid sort field in `GetAuditLogParameters` (documented sortable fields: `category`, `action`, `user_display_name`, `note`, `timestamp`, `details`).
- Including `id` in `sort_by` causes xDome to silently return 0 rows rather than a 4xx response.

The AC-002 recording (`show_sort_by.py audit_logs`) shows the "RETIRED" note explicitly in its output.

**Accepted residual non-determinism for audit_logs:** No valid unique tiebreaker exists in the `audit_logs` API's documented sort field set. Equal-timestamp ties resolve in API-dependent order. The 7-day time-window filter (via `filter_by`) bounds the practical blast radius. This residual is accepted, not a defect (BC-2.16.013 §Sort-by postcondition).

---

## Helper Scripts (not recordings — used to generate demo evidence)

| File | Purpose |
|------|---------|
| `show_sort_by.py` | Reads `claroty.sensor.toml` via `tomllib`, extracts `sort_by` for each table's fetch step. Called in the success-path portion of AC-001..007 recordings. Live-data-free. |
| `demo_error_path.py` | Demonstrates the pre-fix defect using synthetic body_templates (no sort_by). Called in the error-path portion of AC-001..007 recordings. No real data. |

---

## Tape Scripts (VHS source)

| File | Produces | AC(s) |
|------|----------|-------|
| `AC-001-vulnerabilities-sort_by.tape` | AC-001 GIF + WEBM | AC-001 |
| `AC-002-audit_logs-sort_by.tape` | AC-002 GIF + WEBM | AC-002 |
| `AC-003-server_interfaces-sort_by.tape` | AC-003 GIF + WEBM | AC-003 |
| `AC-004-organization_zones-sort_by.tape` | AC-004 GIF + WEBM | AC-004 |
| `AC-005-zone_policies-sort_by.tape` | AC-005 GIF + WEBM | AC-005 |
| `AC-006-firewall_groups-sort_by.tape` | AC-006 GIF + WEBM | AC-006 |
| `AC-007-firewall_policies-sort_by.tape` | AC-007 GIF + WEBM | AC-007 |
| `AC-WIRE-001-obs-wire-sort_by-serialized.tape` | WIRE-001 GIF + WEBM | All 7 ACs |

---

## sort_by Values Summary (verified from TOML)

```
claroty_vulnerabilities:
  [{"field":"adjusted_vulnerability_score","order":"desc"},
   {"field":"name","order":"asc"}]

claroty_audit_logs:
  [{"field":"timestamp","order":"asc"}]
  (filter_by coexists: true — MUST NOT be displaced)
  (note: compound form with id RETIRED — live-validated 2026-09-02)

claroty_server_interfaces:
  [{"field":"server_name","order":"asc"},
   {"field":"interface_name","order":"asc"}]
  (composite PK = total order)

claroty_organization_zones:
  [{"field":"zone_name","order":"asc"}]

claroty_organization_zone_policies:
  [{"field":"policy_name","order":"asc"}]

claroty_organization_firewall_groups:
  [{"field":"firewall_group_name","order":"asc"}]

claroty_organization_firewall_policies:
  [{"field":"policy_name","order":"asc"}]
```
