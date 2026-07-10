---
document_type: research
level: ops
decision_anchor: D-1650
topic: DEFECT-CSDEVICES-EMPTY-PIPELINE-001 root-cause investigation
produced: 2026-07-10
producer: state-manager (orchestrator-directed)
status: complete
---

# DEFECT-CSDEVICES-EMPTY-PIPELINE-001 — Root-Cause Investigation

**Date:** 2026-07-10 | **Anchor:** D-1650 | **Exposed by:** AUDIT-COVERAGE-001 live gate H7

---

## Executive Summary

`DEFECT-CSDEVICES-EMPTY-PIPELINE-001` decomposes into **two independent sub-defects**:

1. **Symptom 1 (0 rows) = SENSOR TOML SPEC DEFECT** — `crowdstrike.sensor.toml` `fetch_devices` step has NO interpolation variable reference, so `find_fan_out_array()` returns `None` and the GET `/devices/entities/devices/v2` call is issued with zero `ids` params, yielding an empty response from the DTU.
2. **Symptom 2 (JOIN "Internal error") = PRODUCT-CODE DEFECT (independent)** — `materialization.rs` skips DataFusion MemTable registration for 0-batch tables; when a mixed JOIN occurs (one side registered, one not), DataFusion planning fails with `DataFusionError::Plan("table not found")` → catch-all `-32000 "Internal error"`.
3. **Symptom 3 (self-join 0 rows): NOT CONFIRMED** — cascade artifact of Symptoms 1+2; re-verify after fixes land.

Architect ratification is **PENDING** on the recommended fix path (Option 1) for Symptom 1.

---

## Symptom 1 — 0 Rows: Sensor TOML Spec Defect

### Observed Behavior

`FROM crowdstrike_devices | limit 3` returns 0 rows live against the DTU (which serves 50 device records).

### Root Cause

In `crowdstrike.sensor.toml`, the `fetch_devices` pipeline step (approx. lines 246–262) declares:

```toml
fan_out_batch_size = 100
```

…but its `path` or `body_template` field contains **no `${...}` variable reference** to the preceding step's output. As a result:

- The engine's `find_fan_out_array()` routine returns `None` (no interpolation anchor found).
- The engine issues a GET to `/devices/entities/devices/v2` with **zero `ids` query params**.
- The DTU route `prism-dtu-crowdstrike/src/routes/hosts.rs` `parse_ids_from_query()` correctly returns an empty vec when no `ids` params are present.
- Zero records are returned; the engine correctly materializes an empty result.

### Working Contrast

The `fetch_detections` step for `crowdstrike_devices` (or the analogous `detections` table pipeline) uses a POST with `body_template` containing `${query_detection_ids.resources}`, which correctly triggers `find_fan_out_array()` and passes the batch IDs through.

### Complication: UrlPath Array Encoding

A naive TOML fix such as:

```toml
path = "/devices/entities/devices/v2?ids=${fetch_device_ids.resources}"
```

**will NOT work** with the current engine. The engine's `UrlPath` interpolation renders arrays as a single percent-encoded JSON blob (`%5B%22id1%22%2C%22id2%22%5D`), which the CrowdStrike API does not accept as a repeated `ids=` param style.

### Fix Options

| Option | Description | Engine Changes | Recommended |
|--------|-------------|----------------|-------------|
| **Option 1 (RECOMMENDED)** | Convert `fetch_devices` to POST, mirroring the `detections` pipeline; add a POST route to `prism-dtu-crowdstrike/src/routes/hosts.rs`; update TOML `method = "POST"` + `body_template` to embed `${fetch_device_ids.resources}`. Matches the real CrowdStrike Devices POST variant in production. | None | YES — requires architect ratification |
| Option 2 | Extend engine's `UrlPath` interpolation to emit repeated `?ids=v1&ids=v2&...` query-param style for array values | Engine changes (non-trivial) | No — larger blast radius |
| Option 3 | Add DTU tolerance for the blob-encoded single-param form (parse `%5B...%5D` as a JSON array) | None | Rejected — breaks fidelity contract (DTU clones must match real-API behavior) |

**Option 1 requires architect ratification** before the implementer proceeds (new DTU route + TOML method/body_template changes are architectural decisions).

### Spec Anchor

BC-2.16.002 fan-out precondition (step variable references must be present for `find_fan_out_array()` to return a non-None result).

---

## Symptom 2 — JOIN "Internal error": Product-Code Defect (Independent)

### Observed Behavior

Any SQL JOIN where one side is an empty-result table (e.g., `crowdstrike_devices LEFT JOIN armis_devices`) fails with MCP error `-32000 "Internal error"`.

### Root Cause

In `crates/prism-query/src/engine.rs` (or `materialization.rs`, approx. lines 1008–1022), the engine **skips DataFusion MemTable registration for tables that returned 0 result batches**. When a mixed JOIN is attempted:

- `armis_devices` was registered (had results).
- `crowdstrike_devices` was NOT registered (0 batches → skipped).

DataFusion's query planner then raises `DataFusionError::Plan("table 'crowdstrike_devices' not found")`, which the engine's catch-all handler maps to `-32000 "Internal error"`.

### Impact

This defect is **independent of Symptom 1**. Any legitimately-empty sensor result in a JOIN position will trigger it — e.g., if a sensor returns 0 rows for a valid query with no matching data.

### Correct Fix

Register a **schema-only empty `MemTable`** from the sensor's declared spec columns when the result batch count is 0. This allows DataFusion to plan the JOIN against a known schema, producing a graceful 0-row result per BC-2.01.010 (partial-failure handling) and BC-2.11.005 (empty-table JOIN semantics).

### Spec Anchors

- BC-2.01.010: partial-failure propagation — empty result ≠ error.
- BC-2.11.005: JOIN with empty table must yield 0 rows, not an error.

### Owner

Implementer (TDD cascade required; RED gate test for empty-MemTable registration then GREEN fix in `materialization.rs` or equivalent registration site).

---

## Symptom 3 — Self-Join 0 Rows: Unconfirmed

**Status:** Likely a cascade artifact of Symptoms 1+2. Re-verify after both fixes are merged. If self-join on a populated table still yields 0 rows after fixes, escalate as a new distinct defect.

---

## Bonus Finding: T13 Section-B False-Positive Gate

The T13 audit script `section-B` check for `crowdstrike_devices` currently reports `PASS: 0 rows` (because 0 rows match the assertion `>= 0 rows`). This is a **false-positive gate** — it passes without actually verifying data delivery.

**Registered task:** section-B hardening is registered against the parked `AUDIT-COVERAGE-001` branch (`fix/T13-audit-coverage` @317b6e25). It must land before that branch merges. The T13 demo capstone is NOT blocked by this (T13 exercises neither `crowdstrike_devices` nor JOINs in its current 70-check form).

---

## Fix Work Plan

### Track A — Symptom 1 (TOML Spec Defect)

1. **PENDING:** Architect ratification of Option 1 (POST conversion).
2. After ratification:
   - Product-owner: amend `crowdstrike.sensor.toml` `fetch_devices` step (`method = "POST"`, `body_template` with `${...}` reference).
   - Implementer: add POST route to `prism-dtu-crowdstrike/src/routes/hosts.rs` (new handler mirroring detections POST route).
3. Adversary LOCAL 3-CLEAN → fix-PR (may be combined with Track B at architect's discretion).

### Track B — Symptom 2 (Product-Code Defect)

1. Implementer: RED gate test — `JOIN` with one empty-result side → expect 0 rows (not error).
2. Implementer: GREEN fix — register schema-only empty `MemTable` for 0-batch tables in `materialization.rs`.
3. Adversary LOCAL 3-CLEAN → fix-PR.

### Track C — Section-B T13 Hardening

1. AFTER both Tracks A+B PRs merge, un-park `AUDIT-COVERAGE-001` branch.
2. Rebase onto develop.
3. Add section-B 0-rows hardening check (assert actual DTU delivery, not merely `>= 0 rows`).
4. Full live audit — expect 0 FAIL / 0 WARN.
5. LOCAL 3-CLEAN → PR.

---

## Decision Record

- **D-1650**: Root-cause investigation confirmed. Two sub-defects (TOML spec + product-code). Architect ratification pending for Option 1. Track B (empty-MemTable) can proceed autonomously.
- **DEFECT-CSDEVICES-EMPTY-PIPELINE-001** status: ROOT-CAUSED (in STATE.md Drift Items).
- **DEFECT-EQUERY042-GROUPBY-DEADARM-001** status: RED-COMPLETE @49e07a29 (independent fix, `.worktrees/FIX-EQUERY042-GROUPBY` on `fix/equery042-groupby-deadarm`). See also worktree notes in STATE.md.
