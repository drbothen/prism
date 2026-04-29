# CrowdStrike Schema Derivation Note

**Story:** S-3.7.00  
**BC:** BC-3.4.002 (precondition 2), BC-3.4.003 (PaginationEdgeCases baseline)  
**Date:** 2026-04-28  
**Author:** implementer agent  

---

## 1. Source SDK + Version

| Item | Value |
|------|-------|
| Go SDK | `github.com/crowdstrike/gofalcon v0.18.0` |
| Consumer repo | `.references/poller-cobra` |
| Go version | 1.25.7 |
| Key SDK packages | `falcon/client`, `falcon/client/alerts`, `falcon/models` |
| Internal API client | `internal/crowdstrike/api.go` (`HTTPClient`) |

Secondary sources (authoritative per ADR-009 §1.2):
- `crates/prism-dtu-crowdstrike/fixtures/hosts-detail.json` — empirical device fields
- `crates/prism-dtu-crowdstrike/fixtures/detections-detail.json` — empirical detection fields
- `crates/prism-dtu-crowdstrike/fixtures/hosts-ids.json` — Step-1 ID list (30 host IDs)
- `crates/prism-dtu-crowdstrike/fixtures/detections-ids.json` — Step-1 ID list (50 detection IDs)
- `crates/prism-dtu-crowdstrike/src/state.rs` — stateful schema (`containment_store`, `detection_status_store`)

---

## 2. default_page_size

```
default_page_size = 100
```

**Source:** `poller-cobra/internal/crowdstrike/source.go`:

```go
limit := cfg.Limit
if limit <= 0 {
    limit = 100
}
```

And `poller-cobra/internal/crowdstrike/api.go` for each `Fetch*` method:

```go
if limit <= 0 {
    limit = 100
}
```

**API max:** The CrowdStrike Falcon API supports up to 500 IDs per query step and
5,000 records per entity fetch page, but the poller caps at 100 as a conservative
default. The `PaginationEdgeCases` archetype baseline (BC-3.4.003) should use
`default_page_size = 100` for CrowdStrike.

---

## 3. Go Struct → Rust Type Mapping

### 3.1 `OAuth2TokenResponse` ← gofalcon OAuth2 response

The gofalcon SDK handles OAuth2 token exchange transparently via `falcon.NewClient()`.
The raw API response from `POST /oauth2/token` follows the standard OAuth2 token format.
The DTU (prism-dtu-crowdstrike) emulates this endpoint for `auth_mode=accept/reject` testing.

| Go field (gofalcon) | Rust field | Rust type | Nullable handling |
|--------------------|-----------|-----------|------------------|
| `AccessToken` `*string` | `access_token` | `Option<String>` | Pointer → Option |
| `TokenType` `*string` | `token_type` | `Option<String>` | Pointer → Option |
| `ExpiresIn` `*int64` | `expires_in` | `Option<i64>` | Pointer → Option |
| `Error` `*string` | `error` | `Option<String>` | Pointer → Option |
| `ErrorDescription` `*string` | `error_description` | `Option<String>` | Pointer → Option |

### 3.2 `IdPage` ← gofalcon `MsaQueryResponse` / `MsaspecQueryResponse` (Step-1 results)

The 2-step CrowdStrike query pattern (EC-002):
- `alerts.QueryV2` returns `MsaQueryResponse` with `Payload.Resources []string`
- `alerts.PostEntitiesAlertsV1` takes those IDs and returns full records

The DTU fixtures confirm the pattern:
- `fixtures/hosts-ids.json`: array of 30 string IDs (`"h-001"` … `"h-030"`)
- `fixtures/detections-ids.json`: array of 50 string IDs (`"det-001"` … `"det-050"`)

| Go field | Rust field | Rust type |
|----------|-----------|-----------|
| `Payload.Resources []string` | `resources` | `Option<Vec<String>>` |
| `Payload.Errors []*MsaAPIError` | `errors` | `Option<Vec<ApiError>>` |
| `Payload.Meta *MsaMetaDataResponse` | `meta` | `Option<ResponseMeta>` |

### 3.3 `FalconDevice` ← poller-cobra `Host` + fixture `hosts-detail.json`

| Go field (api.go Host) | Fixture field (hosts-detail.json) | Rust field | Rust type | Nullable handling |
|------------------------|-----------------------------------|-----------|-----------|------------------|
| `ID string` | `device_id` | `device_id` | `Option<String>` | Always present in fixture |
| `Hostname string` | `hostname` | `hostname` | `Option<String>` | Always present in fixture |
| `Raw map[string]any` | `platform_name` | `platform_name` | `Option<String>` | EC-004: fixture only |
| `Raw map[string]any` | `os_version` | `os_version` | `Option<String>` | EC-004: fixture only |
| `Raw map[string]any` | `status` | `status` | `Option<String>` | EC-004: fixture only |
| `Raw map[string]any` | `containment_status` | `containment_status` | `Option<String>` | state.rs ContainmentStatus.status |
| `Raw map[string]any` | `last_seen` | `last_seen` | `Option<String>` | EC-004: fixture only |
| `Raw map[string]any` | `external_ip` | `external_ip` | `Option<String>` | EC-004: fixture only |
| `Raw map[string]any` | `local_ip` | `local_ip` | `Option<String>` | EC-004: fixture only |
| `Raw map[string]any` | `agent_version` | `agent_version` | `Option<String>` | EC-004: fixture only |

`FalconDevice.device_id` is the HashMap key for `containment_store` in state.rs.

### 3.4 `FalconDetection` ← poller-cobra `Detection` + `alertToMap()` + fixture `detections-detail.json`

| Source | Field | Rust field | Rust type |
|--------|-------|-----------|-----------|
| api.go `Detection.ID` | `id` | `detection_id` | `Option<String>` |
| fixture | `detection_id` | `detection_id` | `Option<String>` |
| api.go `Detection.Status` | `status` | `status` | `Option<String>` |
| fixture | `severity` | `severity` | `Option<String>` |
| fixture | `created_timestamp` | `created_timestamp` | `Option<String>` |
| fixture | `updated_timestamp` | `updated_timestamp` | `Option<String>` |
| fixture | `device.device_id` | `device.device_id` | `Option<String>` |
| fixture | `device.hostname` | `device.hostname` | `Option<String>` |
| fixture | `behaviors[].tactic` | `behaviors[].tactic` | `Option<String>` |
| fixture | `behaviors[].technique` | `behaviors[].technique` | `Option<String>` |
| alertToMap | `composite_id` | `composite_id` | `Option<String>` |
| alertToMap | `tactic`, `tactic_id` | `tactic`, `tactic_id` | `Option<String>` |
| alertToMap | `technique`, `technique_id` | `technique`, `technique_id` | `Option<String>` |
| alertToMap | `product`, `platform` | `product`, `platform` | `Option<String>` |
| alertToMap | `agent_id` | `agent_id` | `Option<String>` |
| alertToMap | `sha256`, `md5` | `sha256`, `md5` | `Option<String>` |

`FalconDetection.detection_id` is the HashMap key for `detection_status_store` in state.rs,
with `FalconDetection.status` as the HashMap value.

### 3.5 `ContainmentResponse` ← state.rs `ContainmentStatus`

The DTU exposes `POST /devices/entities/devices-actions/v2` for containment actions.
State is stored as `containment_store: Mutex<HashMap<String, ContainmentStatus>>`.

| state.rs field | Rust struct field | Rust type |
|----------------|------------------|-----------|
| HashMap key | `ContainedDevice.device_id` | `Option<String>` |
| `ContainmentStatus.status` | `ContainedDevice.status` | `Option<String>` |
| `ContainmentStatus.updated_at` | `ContainedDevice.updated_at` | `Option<String>` |

---

## 4. Nullable → `Option<T>` Handling

All pointer fields in gofalcon Go types (`*string`, `*int64`, `*time.Time`) are
mapped to `Option<T>` in Rust. Rationale:

1. gofalcon uses pointer semantics for nullable JSON fields — `nil` pointer → JSON
   `null` or absent key.
2. The `safeString(*string)` helper in api.go returns `""` for nil pointers; in
   Rust we model this as `Option<String>` to distinguish absent from empty.
3. All fixture fields are always present in the static JSON; however, production
   API responses may omit optional fields, so `Option<T>` is the safe choice.

---

## 5. Polymorphic Field Handling (EC-002 and EC-003)

**EC-002 — 2-step IDs→detail pattern:**

CrowdStrike's API does not return resource details directly from query endpoints.
The pattern used by `FetchAlerts` in api.go:

```
Step 1: alerts.QueryV2(params)       → IdPage ([]string of IDs)
Step 2: alerts.PostEntitiesAlertsV1(body{Ids: alertIDs}) → []FalconDetection
```

The `IdPage` struct in `types.rs` captures Step-1. S-3.7.05 must generate both
Step-1 (ID-only) and Step-2 (detail) fixture responses.

**EC-003 — Go `interface{}` / `map[string]interface{}`:**

`Host.Raw map[string]any` and `Detection.Raw map[string]interface{}` in api.go
capture the full raw API response. The fixture JSON shows specific field names
that S-3.7.05 needs to generate. These are modeled as:
- Explicit `Option<String>` fields for known fixture fields (device_id, hostname, etc.)
- `#[serde(flatten)] pub extra: HashMap<String, Value>` for unknown extension fields

---

## 6. Go Fields Omitted from Rust Translation

| Go field | Reason for omission |
|----------|---------------------|
| `Alert.SeverityName *string` (numeric) | Redundant with `severity` string field; captured via `alertToMap` |
| `Alert.Cmdline`, `Alert.Filename` | Captured in `FalconDetection.cmdline/filename` |
| gofalcon internal pagination cursor | Managed by SDK; DTU uses session_registry (state.rs) |
| `SessionData` (state.rs) | Internal DTU state; not an API wire type |

---

## 7. state.rs Field Name Authority

Per ADR-009 §1.2, `crates/prism-dtu-crowdstrike/src/state.rs` is authoritative
for the stateful schema:

```rust
// state.rs
pub containment_store: Mutex<HashMap<String, ContainmentStatus>>,
pub detection_status_store: Mutex<HashMap<String, String>>,
```

- `FalconDevice.device_id` → key in `containment_store`
- `ContainedDevice.status` → value in `containment_store` (`ContainmentStatus.status`)
- `FalconDetection.detection_id` → key in `detection_status_store`
- `FalconDetection.status` → value in `detection_status_store`

S-3.7.05 fixture generators MUST use these field names to ensure round-trip
compatibility with the DTU state store.
