# Armis Schema Derivation Note

**Story:** S-3.7.00  
**BC:** BC-3.4.002 (precondition 2), BC-3.4.003 (PaginationEdgeCases baseline)  
**Date:** 2026-04-28  
**Author:** implementer agent  

---

## 1. Source SDK + Version

| Item | Value |
|------|-------|
| Go SDK | `github.com/1898andCo/armis-sdk-go/v2 v2.0.1` |
| Consumer repo | `.references/poller-coaster` |
| Go version | 1.25.7 |
| SDK import alias | `centrix` |
| Key SDK type used | `centrix.SearchData`, `centrix.SearchResult` |

The Armis SDK is not vendored locally; Go module cache was unavailable in this
environment. The `SearchResult` struct was reconstructed entirely from field
accesses in the poller-coaster collector files:

- `internal/collector/alert_collector.go`
- `internal/collector/activity_collector.go`
- `internal/collector/audit_collector.go`
- `internal/collector/risk_factor_collector.go`
- `internal/collector/connection_collector.go`
- `internal/collector/device_collector.go`
- `internal/collector/vulnerability_collector.go`

S-3.7.04 implementers should validate field names against the live SDK if/when the
Go module cache becomes available.

---

## 2. default_page_size

```
default_page_size = 100
```

**Source:** `internal/config/config.go` in poller-coaster — all seven data-source
limits default to `100`:

```go
AlertLimit:         100,
ActivityLimit:      100,
AuditLogLimit:      100,
RiskFactorLimit:    100,
ConnectionLimit:    100,
DeviceLimit:        100,
VulnerabilityLimit: 100,
```

These values are environment-variable-overridable (`ARMIS_ALERT_LIMIT`, etc.) but 100
is the conservative safe default used in production deployments.

**API max:** The Armis Search API supports larger page sizes, but the poller caps
at 100 to avoid response-size timeouts on large environments. The `PaginationEdgeCases`
archetype baseline (BC-3.4.003) should use `default_page_size = 100` for Armis.

---

## 3. Go Struct → Rust Type Field Mapping

### 3.1 `ArmisSearchResult` ← `centrix.SearchResult`

| Go field (inferred) | Go type (inferred) | Rust field | Rust type | Nullable handling |
|--------------------|--------------------|-----------|-----------|------------------|
| `ID` | `DeviceID` (custom int type) | `id` | `Option<ArmisId>` | Nullable — zero means absent |
| `Title` | `string` | `title` | `Option<String>` | Empty string treated as absent |
| `AlertID` | `int64` | `alert_id` | `Option<i64>` | Zero means absent |
| `PolicyID` | `string` | `policy_id` | `Option<String>` | Empty string treated as absent |
| `Time` | `string` | `time` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `LastAlertUpdateTime` | `string` | `last_alert_update_time` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `LastSeen` | `string` | `last_seen` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `FirstSeen` | `string` | `first_seen` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `ActivityUUIDs` | `[]string` | `activity_uuids` | `Option<Vec<String>>` | Absent for non-activity results |
| `StartTimestamp` | `string` | `start_timestamp` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `EndTimestamp` | `string` | `end_timestamp` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `LastDetected` | `string` | `last_detected` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `FirstDetected` | `string` | `first_detected` | `Option<String>` | RFC-3339 or RFC-3339Nano |
| `PublishedDate` | `string` | `published_date` | `Option<String>` | RFC-3339 or RFC-3339Nano |

### 3.2 `ArmisAsset` ← device-subset of `centrix.SearchResult`

Captures device/asset-specific fields requested by the `DeviceLimit` AQL query.
Default fields from `config.go` DeviceFields:

```go
DeviceFields: []string{"id", "name", "type", "lastSeen", "firstSeen", "ipAddress", ...}
```

### 3.3 `ArmisAlert` ← alert-subset of `centrix.SearchResult`

Alert-specific fields from `AlertFields` default in `config.go`:

```go
AlertFields: []string{"title", "status", "alertId", "policyId", ...}
```

---

## 4. Nullable → `Option<T>` Handling

All string fields from the Armis API are treated as `Option<String>` because:

1. The Go SDK uses empty string as "not present" — the collectors use
   `strings.TrimSpace(field) == ""` to detect absent values.
2. JSON nulls and absent JSON keys both deserialize to `None` in Rust with `#[serde(default)]`.
3. Numeric fields like `AlertID` use zero-value to mean "absent" in Go — mapped to
   `Option<i64>` with `#[serde(default)]`.

---

## 5. Polymorphic ID Handling (EC-001)

The Armis API returns IDs as either JSON integers or JSON strings depending on context:

- In `device_collector.go`: `id = string(result.ID)` — cast from a custom integer type
- In `alert_collector.go`: `result.AlertID != 0` — pure integer comparison
- In `sink.go`: all results serialized uniformly as JSON (Go marshal handles the cast)

**Decision:** Use `ArmisId(serde_json::Value)` as a newtype wrapping `Value`. This
accepts both `"123"` (string) and `123` (integer) from JSON without a custom
`Deserialize` impl, at the cost of type safety. The `#[serde(transparent)]` attribute
means `ArmisId` serializes/deserializes identically to `Value`.

If the fixture generator (S-3.7.04) determines that a specific field is always
integer, replace `ArmisId` with `i64` for that field.

---

## 6. Go Fields Omitted from Rust Translation

| Go field | Reason for omission |
|----------|---------------------|
| `CveUid` (vulnerability) | Not accessed as a primary field; captured in `extra` HashMap |
| Internal SDK metadata | Any unexported fields have no JSON representation |
| `Sample` in SearchData | Optional diagnostic blob; mapped to `Option<Value>` |

---

## 7. `AqlResponse<T>` and `ArmisPage<T>` Rationale

The collector calls `client.GetSearch()` which returns `centrix.SearchData` directly —
pagination is managed cursor-side, not response-side. However:

- `AqlResponse<T>` is included per AC-001 story requirement (generic API envelope).
- `ArmisPage<T>` captures the conceptual pagination shape for S-3.7.04 fixture use.

S-3.7.04 implementers should use `ArmisPage<ArmisAsset>` or
`ArmisPage<ArmisSearchResult>` as their fixture schema root.

---

## 8. Interface{} → `serde_json::Value` Decisions (EC-003)

Any Go `interface{}` or `map[string]interface{}` field is translated to
`serde_json::Value`. In `ArmisSearchResult`, the `extra` field uses `#[serde(flatten)]`
to absorb additional API fields not explicitly captured in the struct. This is the
standard Rust pattern for extensible JSON schemas.
