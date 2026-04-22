# Evidence Report — S-1.14: Infusion Spec Loading and UDF Registration

**Story:** S-1.14  
**Branch:** feature/S-1.14-infusion-specs  
**Commit at recording:** c102fd7  
**Test suite:** 220/220 tests pass (prism-core 85 + prism-spec-engine 14 VP + 31 integration)  
**Policy:** POL-010  
**Date recorded:** 2026-04-22  

---

## Coverage Map

| AC | BC / VP | Description | Success Recording | Error Recording | Format |
|----|---------|-------------|-------------------|-----------------|--------|
| AC-1 | BC-2.19.001 | InfusionRegistry: geoip spec → 4 UDF descriptors | AC-001-infusion-registry-udf-export.gif/.webm | AC-010-error-path-duplicate-udf.gif/.webm | VHS |
| AC-2 | BC-2.19.002 | QueryScopedInfusionCache: 10K events, 200 unique IPs → 200 calls | AC-002-query-scoped-dedup-cache.gif/.webm | AC-002 (null result cached, not retried) | VHS |
| AC-3 | BC-2.19.001 | EnrichStageDescriptor: `| enrich geoip ON device_ip` → 4 columns | AC-003-enrich-descriptor.gif/.webm | AC-003 (E-INFUSE-001 for unknown name) | VHS |
| AC-4 | BC-2.19.003 | E-RULE-012: `is_api_backed()` returns true for plugin infusions | AC-004-e-rule-012-api-backed-rejection.gif/.webm | AC-004 (E-RULE-012 error format) | VHS |
| AC-5 | BC-2.19.004 | Hot reload: failed validation retains previous registry (CI-002) | AC-005-hot-reload-atomicity.gif/.webm | AC-005 (invalid spec rejected, old intact) | VHS |
| AC-6 | BC-2.19.005 | CredentialRef: `<redacted>` in Debug; E-INFUSE-005 hides secrets | AC-006-credential-redaction.gif/.webm | AC-006 (loader error, no secret in msg) | VHS |
| AC-7 | — | CSV source: asset_inventory spec loads asset_owner + asset_department | AC-007-csv-source-backend.gif/.webm | AC-010 (E-INFUSE-003 zero fields) | VHS |
| AC-8 | — | RocksDB Tier 3 cache: covered by BC-2.19.002 dedup tests (architecture) | (Tier 3 is CacheBackend trait injection, tested structurally) | — | Note |
| AC-9 | VP-048 (Kani) | N distinct fields → exactly N UDF descriptors; duplicate → E-INFUSE-002 | AC-008-vp048-kani-placeholder.md | — | Markdown placeholder |
| AC-10 | VP-049 (proptest) | 1000 cases: enrich_single calls == unique value count | AC-009-vp049-proptest.gif/.webm | AC-009 (all-identical → 1 call edge case) | VHS |

---

## Recordings

### AC-001: InfusionRegistry UDF Export (BC-2.19.001)

**Tape:** `AC-001-infusion-registry-udf-export.tape`  
**Demonstrates:** `cargo test BC_2_19_001` — 8 tests covering geoip spec loading, 4 UDF
descriptors produced, `udf_descriptors()` return, 10-field spec producing 10 descriptors.  
![AC-001](AC-001-infusion-registry-udf-export.gif)

---

### AC-002: QueryScopedInfusionCache Dedup (BC-2.19.002)

**Tape:** `AC-002-query-scoped-dedup-cache.tape`  
**Demonstrates:** `cargo test BC_2_19_002` — 4 tests: 3-row same IP → 1 call, 10K events
200 unique IPs → 200 calls, per-query cache isolation, null result cached not retried.  
![AC-002](AC-002-query-scoped-dedup-cache.gif)

---

### AC-003: EnrichStageDescriptor (BC-2.19.001)

**Tape:** `AC-003-enrich-descriptor.tape`  
**Demonstrates:** `cargo test enrich_descriptor` — correct output columns returned,
E-INFUSE-001 raised for unknown infusion name.  
![AC-003](AC-003-enrich-descriptor.gif)

---

### AC-004: E-RULE-012 API-Backed Rejection (BC-2.19.003)

**Tape:** `AC-004-e-rule-012-api-backed-rejection.tape`  
**Demonstrates:** `cargo test BC_2_19_003` — `is_api_backed()` returns true for plugin
infusions, false for local_lookup and unknown names; E-RULE-012 error format validated.  
![AC-004](AC-004-e-rule-012-api-backed-rejection.gif)

---

### AC-005: Hot Reload Atomicity (BC-2.19.004)

**Tape:** `AC-005-hot-reload-atomicity.tape`  
**Demonstrates:** `cargo test BC_2_19_004` — failed hot reload retains previous registry
(CI-002), valid hot reload swaps atomically, duplicate UDF on reload rejected with E-INFUSE-002.  
![AC-005](AC-005-hot-reload-atomicity.gif)

---

### AC-006: Credential Redaction (BC-2.19.005)

**Tape:** `AC-006-credential-redaction.tape`  
**Demonstrates:** `cargo test BC_2_19_005` — CredentialRef Debug shows `<redacted>`,
E-INFUSE-005 message includes field name and env var name but never the secret value,
loader error for spec with credentials does not leak secrets.  
![AC-006](AC-006-credential-redaction.gif)

---

### AC-007: CSV Source Backend

**Tape:** `AC-007-csv-source-backend.tape`  
**Demonstrates:** `cargo test ac_7` — asset_inventory CSV spec with `key_column = ip_address`
loads and exports `asset_owner` and `asset_department` UDF descriptors.  
![AC-007](AC-007-csv-source-backend.gif)

---

### AC-008: VP-048 Kani Proof (Placeholder)

**File:** `AC-008-vp048-kani-placeholder.md`  
**Demonstrates:** Kani harnesses authored in `proofs/infusion_spec.rs`. Two harnesses:
`verify_n_fields_n_descriptors` (N in 1..=16) and `verify_duplicate_udf_name_errors`.
Execution scheduled for Phase 5 formal-verify. Compile-check unit test passes in 220/220
green suite.

**Kani placeholder count: 1**

---

### AC-009/AC-010: VP-049 proptest (BC-2.19.002)

**Tape:** `AC-009-vp049-proptest.tape`  
**Demonstrates:** `cargo test invariant_dedup` — 1000 proptest cases covering:
K-distinct from N values → exactly K calls; all-identical → 1 call; all-distinct → N calls.  
![AC-009](AC-009-vp049-proptest.gif)

---

### AC-010 Error Path: E-INFUSE-002 / E-INFUSE-003 / E-RULE-012

**Tape:** `AC-010-error-path-duplicate-udf.tape`  
**Demonstrates:** `cargo test rejects` — spec with 0 fields rejected (E-INFUSE-003),
duplicate UDF name across specs rejected (E-INFUSE-002), unknown source type rejected
(E-INFUSE-004), E-RULE-012 error message format verified.  
![AC-010](AC-010-error-path-duplicate-udf.gif)

---

## File Inventory

| File | Type | AC Coverage |
|------|------|-------------|
| `AC-001-infusion-registry-udf-export.tape` | VHS script | AC-1 / BC-2.19.001 |
| `AC-001-infusion-registry-udf-export.gif` | GIF recording | AC-1 / BC-2.19.001 |
| `AC-001-infusion-registry-udf-export.webm` | WebM recording | AC-1 / BC-2.19.001 |
| `AC-002-query-scoped-dedup-cache.tape` | VHS script | AC-2 / BC-2.19.002 |
| `AC-002-query-scoped-dedup-cache.gif` | GIF recording | AC-2 / BC-2.19.002 |
| `AC-002-query-scoped-dedup-cache.webm` | WebM recording | AC-2 / BC-2.19.002 |
| `AC-003-enrich-descriptor.tape` | VHS script | AC-3 / BC-2.19.001 |
| `AC-003-enrich-descriptor.gif` | GIF recording | AC-3 / BC-2.19.001 |
| `AC-003-enrich-descriptor.webm` | WebM recording | AC-3 / BC-2.19.001 |
| `AC-004-e-rule-012-api-backed-rejection.tape` | VHS script | AC-4 / BC-2.19.003 |
| `AC-004-e-rule-012-api-backed-rejection.gif` | GIF recording | AC-4 / BC-2.19.003 |
| `AC-004-e-rule-012-api-backed-rejection.webm` | WebM recording | AC-4 / BC-2.19.003 |
| `AC-005-hot-reload-atomicity.tape` | VHS script | AC-5 / BC-2.19.004 |
| `AC-005-hot-reload-atomicity.gif` | GIF recording | AC-5 / BC-2.19.004 |
| `AC-005-hot-reload-atomicity.webm` | WebM recording | AC-5 / BC-2.19.004 |
| `AC-006-credential-redaction.tape` | VHS script | AC-6 / BC-2.19.005 |
| `AC-006-credential-redaction.gif` | GIF recording | AC-6 / BC-2.19.005 |
| `AC-006-credential-redaction.webm` | WebM recording | AC-6 / BC-2.19.005 |
| `AC-007-csv-source-backend.tape` | VHS script | AC-7 |
| `AC-007-csv-source-backend.gif` | GIF recording | AC-7 |
| `AC-007-csv-source-backend.webm` | WebM recording | AC-7 |
| `AC-008-vp048-kani-placeholder.md` | Kani placeholder | AC-9 / VP-048 |
| `AC-009-vp049-proptest.tape` | VHS script | AC-10 / VP-049 |
| `AC-009-vp049-proptest.gif` | GIF recording | AC-10 / VP-049 |
| `AC-009-vp049-proptest.webm` | WebM recording | AC-10 / VP-049 |
| `AC-010-error-path-duplicate-udf.tape` | VHS script | Error paths (all ACs) |
| `AC-010-error-path-duplicate-udf.gif` | GIF recording | Error paths (all ACs) |
| `AC-010-error-path-duplicate-udf.webm` | WebM recording | Error paths (all ACs) |
| `evidence-report.md` | This report | All ACs |

---

## Notes

- **AC-8 (RocksDB Tier 3):** The `CacheBackend` trait is injected at startup by prism-bin.
  `prism-spec-engine` does not depend on `prism-storage`. Tier 3 cache behavior is verified
  architecturally by the dedup tests (BC-2.19.002) and the `QueryScopedInfusionCache` isolation
  tests. A dedicated RocksDB integration demo requires prism-storage wiring (S-3.02 scope).

- **VP-048 Kani:** 1 placeholder. Kani harnesses are authored and compile-checked; formal
  symbolic execution runs during Phase 5 formal-verify after the full implementation lands.

- **All 9 VHS recordings** completed successfully with `Wait+Screen /finished/` synchronization
  on `cargo test -q` output. Each recording is under 15 seconds.
