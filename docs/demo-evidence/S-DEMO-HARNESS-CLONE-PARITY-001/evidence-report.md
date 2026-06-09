# Demo Evidence Report — S-DEMO-HARNESS-CLONE-PARITY-001

**Story:** S-DEMO-HARNESS-CLONE-PARITY-001  
**Title:** In-process harness DTU clone route parity (Armis GET /api/v1/search + Claroty POST /api/v1/audit_log/get)  
**BC:** BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY  
**Convergence status:** LOCAL 3-CLEAN (ready for PR)  
**Evidence generated:** 2026-06-08  
**VHS version:** 0.10.0  
**Font:** FiraCode Nerd Font Mono  

---

## Coverage Summary

| AC | Description | Load-bearing test | Status | Recording |
|----|-------------|-------------------|--------|-----------|
| AC-001 | Armis GET /api/v1/search: 200 with real admin token, 403 without Bearer | `test_BC_2_16_013_armis_harness_search_returns_200_with_bearer_403_without` | PASS | AC-001-armis-search-auth.gif / .webm |
| AC-001/C-3 | Armis search 401 on present-but-wrong token (F-P2-LOW-001) | `test_BC_2_16_013_armis_harness_search_401_on_wrong_token` | PASS | AC-001-armis-search-auth.gif / .webm |
| AC-002 | Armis AQL routing: in:devices vs in:alerts return distinct non-empty arrays | `test_BC_2_16_013_armis_harness_search_aql_in_devices_returns_device_records` | PASS | AC-002-armis-search-aql-routing.gif / .webm |
| AC-003 | Claroty POST /api/v1/audit_log/get in router() AND network_router() | `test_BC_2_16_013_claroty_harness_audit_log_returns_200_with_bearer_401_without` | PASS | AC-003-claroty-audit-log-auth.gif / .webm |
| AC-004 | Claroty audit_log response envelope: {audit_log:[...], total:N} + 5-column check | `test_BC_2_16_013_claroty_harness_audit_log_response_envelope_matches_standalone` | PASS | AC-004-claroty-audit-log-envelope.gif / .webm |
| AC-005 | Module-doc route tables: routes registered in armis::router() and both claroty routers | Source inspection via grep | PASS | AC-005-route-table-inspection.gif / .webm |

**All 5 acceptance criteria fully covered. 5 parity tests pass.**

---

## Acceptance Criterion Detail

### AC-001 — Armis GET /api/v1/search authentication parity

**Recording:** `AC-001-armis-search-auth.gif` / `AC-001-armis-search-auth.webm`  
**Tape source:** `AC-001-armis-search-auth.tape`

**What is demonstrated:**
- `test_BC_2_16_013_armis_harness_search_returns_200_with_bearer_403_without` — The Armis harness clone registers `GET /api/v1/search`. A request with the real admin token (obtained via `harness.admin_token_for()`) returns HTTP 200. A request with no `Authorization` header returns HTTP 403 (Armis auth model — missing Bearer is 403, NOT 401).
- `test_BC_2_16_013_armis_harness_search_401_on_wrong_token` (F-P2-LOW-001) — A request bearing a present-but-wrong bearer token (`"definitely-not-the-admin-token"`) returns HTTP 401 (token mismatch). This closes the false-pass vulnerability where a regression to the Claroty "accept any non-empty bearer" model would have silently passed the 200/403 test.

**Auth model distinction:**
- Armis: missing Bearer → 403; present wrong token → 401; correct token → 200
- Claroty (see AC-003): any non-empty Bearer → 200; missing/empty → 401

**Red Gate:** Before implementation, `GET /api/v1/search` was not registered in `armis::router()`, so all requests returned 404.

---

### AC-002 — Armis AQL routing structural parity

**Recording:** `AC-002-armis-search-aql-routing.gif` / `AC-002-armis-search-aql-routing.webm`  
**Tape source:** `AC-002-armis-search-aql-routing.tape`

**What is demonstrated:**
- `test_BC_2_16_013_armis_harness_search_aql_in_devices_returns_device_records` — The `GET /api/v1/search?aql=in:devices` request returns `{"data": {"results": [...], "total": N}}` with a non-empty results array from `DEVICES_FIXTURE` (25 device records). The `GET /api/v1/search?aql=in:alerts` request returns a distinct non-empty results array from `ALERTS_FIXTURE` (12+ alert records). The EC-004 discriminator check confirms the first device record and first alert record are different values.

**C-7 structural parity:** The harness serves raw `Vec<Value>` from embedded fixtures; the standalone DTU uses typed `DeviceRecord`/`AlertRecord` + time-window filtering. The test verifies structural shape (envelope, non-empty array, numeric total, distinct entity types) not byte-exact field equality.

**Red Gate:** Before implementation, `GET /api/v1/search?aql=in:devices` returned 404 (route not registered).

---

### AC-003 — Claroty POST /api/v1/audit_log/get in both routers

**Recording:** `AC-003-claroty-audit-log-auth.gif` / `AC-003-claroty-audit-log-auth.webm`  
**Tape source:** `AC-003-claroty-audit-log-auth.tape`

**What is demonstrated:**
- `test_BC_2_16_013_claroty_harness_audit_log_returns_200_with_bearer_401_without` — Tests BOTH `router()` (logical mode) and `network_router()` (network mode) per C-4:
  - Logical mode Part A: `POST /api/v1/audit_log/get` with `Bearer any-non-empty-token` → HTTP 200; without `Authorization` header → HTTP 401
  - Network mode Part B: Same route in `network_router()` with bearer → HTTP 200; without → HTTP 401

**Claroty auth model:** Any non-empty Bearer is accepted (returns 200). Missing or empty Bearer returns 401. This is distinct from Armis (403 for missing, 401 for wrong).

**Red Gate:** Before implementation, `POST /api/v1/audit_log/get` was not registered in either router, returning 404 for all requests.

---

### AC-004 — Claroty audit_log response envelope matches standalone

**Recording:** `AC-004-claroty-audit-log-envelope.gif` / `AC-004-claroty-audit-log-envelope.webm`  
**Tape source:** `AC-004-claroty-audit-log-envelope.tape`

**What is demonstrated:**
- `test_BC_2_16_013_claroty_harness_audit_log_response_envelope_matches_standalone` — The response envelope is `{"audit_log": [...], "total": N}` where `"audit_log"` matches the `claroty.sensor.toml` `response_path="$.audit_log"`. The `audit_log` array is non-empty (5 entries from the embedded fixture). `total` equals `audit_log.len()`. Every entry contains all 5 required columns: `id`, `action`, `actor`, `timestamp`, `resource` — all non-empty strings. The first entry's `timestamp` is ISO 8601 format (contains `T`).

**C-1/C-8 compile-time embedding:** The fixture is embedded via `include_str!("../../../prism-dtu-claroty/fixtures/audit-log.json")` at compile time. The harness does NOT use `prism_dtu_common::load_fixture` (the standalone DTU's runtime pattern).

**Red Gate:** Before implementation, `POST /api/v1/audit_log/get` returned 404.

---

### AC-005 — Module-doc route tables (source inspection)

**Recording:** `AC-005-route-table-inspection.gif` / `AC-005-route-table-inspection.webm`  
**Tape source:** `AC-005-route-table-inspection.tape`

**What is demonstrated:** Source inspection via grep confirms both routes are registered in the router constructor functions.

**Armis `armis.rs` router() route table excerpt (lines 1039–1068):**

```rust
pub fn router(state: Arc<ArmisHarnessState>) -> Router {
    Router::new()
        .route("/api/v1/devices", get(get_devices))
        .route("/api/v1/devices", post(post_devices))
        .route("/api/v1/devices/:device_id/activity", get(get_device_activity))
        .route("/api/v1/devices/:device_id/risk", get(get_device_risk))
        .route("/api/v1/alerts", get(get_alerts))
        .route("/api/v1/search", get(get_search))          // <-- AC-001/AC-002 new route
        .route("/api/v1/devices/:device_id/tags/", post(post_device_tag))
        .route("/api/v1/devices/:device_id/tags/:tag_key", delete(delete_device_tag))
        .route("/dtu/aql-log", get(get_aql_log))
        .route("/dtu/configure", post(dtu_configure))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/health", get(dtu_health))
        // ... crash detection test hooks
        .with_state(state)
}
```

**Claroty `claroty.rs` router() and network_router() audit_log route:**

```rust
// In router() (line 984):
.route("/api/v1/audit_log/get", post(list_audit_log))
// Comment: "// Audit log endpoint (INV-HARNESS-ROUTE-PARITY — S-DEMO-HARNESS-CLONE-PARITY-001 AC-003)"

// In network_router() (line 1238):
.route("/api/v1/audit_log/get", post(list_audit_log))
// Comment: "// Audit log endpoint: same plain check_bearer_auth as sibling alert/vuln routes"
// "// (INV-HARNESS-ROUTE-PARITY — S-DEMO-HARNESS-CLONE-PARITY-001 AC-003 C-4)"
```

**Module-doc route tables (lines 17–56 of each file):** Both `armis.rs` and `claroty.rs` maintain complete `//!` module-level route tables listing every registered endpoint. `GET /api/v1/search` appears in the armis table; `POST /api/v1/audit_log/get` appears in both claroty tables (logical and network mode).

---

## Test Run Output (live capture at evidence generation time)

```
Nextest run ID a8781c73-9c75-4772-879a-4f5bf07c63eb
    Starting 5 tests across 1 binary (8 binaries skipped)
        PASS [   0.053s] prism-dtu-harness::harness_clone_parity_test test_BC_2_16_013_armis_harness_search_401_on_wrong_token
        PASS [   0.054s] prism-dtu-harness::harness_clone_parity_test test_BC_2_16_013_claroty_harness_audit_log_response_envelope_matches_standalone
        PASS [   0.054s] prism-dtu-harness::harness_clone_parity_test test_BC_2_16_013_armis_harness_search_returns_200_with_bearer_403_without
        PASS [   0.054s] prism-dtu-harness::harness_clone_parity_test test_BC_2_16_013_armis_harness_search_aql_in_devices_returns_device_records
        PASS [   0.055s] prism-dtu-harness::harness_clone_parity_test test_BC_2_16_013_claroty_harness_audit_log_returns_200_with_bearer_401_without
     Summary [   0.056s] 5 tests run: 5 passed, 0 skipped
```

**Command:** `cargo nextest run -p prism-dtu-harness --features dtu -E 'binary(harness_clone_parity_test)' --no-fail-fast`

---

## Path Hygiene (DRIFT-SEC-TAPE-PATH-001 / DRIFT-D904-002)

All `.tape` files use relative `Output` paths (`docs/demo-evidence/S-DEMO-HARNESS-CLONE-PARITY-001/AC-NNN-*.gif`). The `cd` path inside the `Hide` block is necessarily absolute (VHS has no cwd concept), but no `/Users/<literal-name>/` paths appear in any `Output` directive, making the evidence paths portable across machines.

---

## Files Produced

| File | Type | Size |
|------|------|------|
| `AC-001-armis-search-auth.tape` | VHS script | source |
| `AC-001-armis-search-auth.gif` | GIF recording | 230 KB |
| `AC-001-armis-search-auth.webm` | WebM recording | 923 KB |
| `AC-002-armis-search-aql-routing.tape` | VHS script | source |
| `AC-002-armis-search-aql-routing.gif` | GIF recording | 137 KB |
| `AC-002-armis-search-aql-routing.webm` | WebM recording | 425 KB |
| `AC-003-claroty-audit-log-auth.tape` | VHS script | source |
| `AC-003-claroty-audit-log-auth.gif` | GIF recording | 139 KB |
| `AC-003-claroty-audit-log-auth.webm` | WebM recording | 431 KB |
| `AC-004-claroty-audit-log-envelope.tape` | VHS script | source |
| `AC-004-claroty-audit-log-envelope.gif` | GIF recording | 146 KB |
| `AC-004-claroty-audit-log-envelope.webm` | WebM recording | 451 KB |
| `AC-005-route-table-inspection.tape` | VHS script | source |
| `AC-005-route-table-inspection.gif` | GIF recording | 166 KB |
| `AC-005-route-table-inspection.webm` | WebM recording | 160 KB |
| `evidence-report.md` | This report | — |
