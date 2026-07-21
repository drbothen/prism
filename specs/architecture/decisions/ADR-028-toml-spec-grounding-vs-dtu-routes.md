---
document_type: adr
adr_id: "ADR-028"
title: "TOML Spec URLs and auth_type Ground Against DTU Clone Routes (Real-API Canonical), Not Production Rust Adapter URLs"
status: Proposed
date: "2026-05-20"
modified: "2026-07-21"  # see §Changelog top row
version: "1.23"
producer: architect
subsystems_affected: [SS-01, SS-07, SS-16, SS-17]
supersedes: ["ADR-026 §D3 (partial — auth_type_name() return values for Cyberint/Claroty/Armis non-CrowdStrike sensors)"]
superseded_by:
  - "ADR-031 §D4 (partial — §D12 only: Cyberint cookie auth DTU-shortcut acceptance reversed by DTU=true-DTU principle 2026-05-29)"
  - "ADR-053 §D1/§D2/§D5 (partial — grounding order §D1/§D2/§D5 superseded: spec grounds FROM vendor OpenAPI, not DTU; Armis LOCKED auth_type D-747 superseded; Cyberint LOCKED auth_type D-747 superseded; authorized D-1889 2026-07-20; final ADR approval gate pending)"
amends: null
amended_by:
  - "ADR-054 §D2/D5/D10 (partial — §D13 oauth2_client_credentials: PluginAuthProvider (WASM) path is spec-load-rejected per D10(b) E-SPEC-028(b) unconditional rejection for auth_type ∈ {oauth2_client_credentials, token_exchange} + auth_plugin present; DeclarativeHttpAuthProvider (native) is the sole live path; crowdstrike-oauth2.prx plugin retired; §D2 + §D13 Armis blockquotes updated from custom_via_plugin to token_exchange; effective on ADR-054 acceptance)"
anchor_stories: [PLUGIN-MIGRATION-001-D, PLUGIN-MIGRATION-001-A, PLUGIN-MIGRATION-001-B, PLUGIN-MIGRATION-001-C, PLUGIN-MIGRATION-001-E, S-DEMO-001, S-DEMO-002]
related_adrs: [ADR-003, ADR-023, ADR-027, ADR-053, ADR-054]
related_bcs: [BC-2.16.013, BC-2.16.001, BC-2.16.009, BC-2.01.016]
locked_decisions: ["D-737 Decision 1", "D-737 Decision 4"]
wiring_deferred_to: null
---

# ADR-028: TOML Spec URLs and auth_type Ground Against DTU Clone Routes (Real-API Canonical), Not Production Rust Adapter URLs

## Status

Proposed 2026-05-20, v1.0 (initial proposal version; current version per §Changelog top row). Locks D-737 Decisions 1 and 4 as a durable architectural principle. Will be promoted to ACCEPTED after PLUGIN-MIGRATION-001-D LOCAL adversarial cascade reaches 3-CLEAN convergence per ADR-021 promotion lifecycle.

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

> **[ADR-053 §D2 SUPERSEDES this Armis row (2026-07-20, D-1889); ADR-054 D1 amends auth
> mechanism (2026-07-21, D-1895):]** Armis is reclassified from `bearer_static` to
> `auth_type = "token_exchange"` with native `DeclarativeHttpAuthProvider` (ADR-054 D1/D4 —
> no WASM plugin), `header_scheme = "raw"`, and a `[[credential_refs]]` block with `name = "secret_key"`. The real
> Armis v1 API uses token-exchange auth (POST `secret_key` → short-lived `access_token`) and
> raw-token Authorization header injection with NO "Bearer" prefix. `auth_type = "bearer_static"`
> for Armis is superseded and MUST NOT be used. The operative contract is ADR-053 §D2 +
> ADR-054 D1. D-1889 authorization; D-1895 native-provider ruling.

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

### D7 — Per-File §Changelog Convention Lock

**Adjudicated in FB-IMPL-P17-ARCH (2026-05-20), closing F-LP17-HIGH-002 (12th coherence-axis class: sample-biased sibling-convention closures).**

Each ADR's §Changelog table ordering convention (ascending oldest-to-newest, or descending newest-to-oldest) is **locked at the ADR's authoring time**. Subsequent fix-bursts MUST preserve the file's existing order. POL-26 monotonic-ordering enforcement targets ROW POSITIONS within the established convention — it does not authorize unilateral flipping. **Exception:** POL-32 (`changelog_monotonic_descending`) supersedes the per-file lock when an ADR explicitly reorders under that policy's authorization. ADR-026 v1.35 (2026-07-20) exercised this exception — see table row below.

Project does NOT have a single canonical §Changelog direction. Observed per-file conventions:

| ADR | Authoring Convention | Evidence |
|-----|---------------------|---------|
| ADR-019 | DESCENDING (newest top) | v0.4 at top, v0.1 at bottom |
| ADR-022 | DESCENDING (newest top) | v1.12 top → v1.0 bottom; 6 explicit POL-26 "repaired to strict descending" enforcement records (D-611/D-628/D-635/D-659/D-670/D-671) |
| ADR-026 | **DESCENDING (newest top) as of v1.35 (2026-07-20)** | Reordered from ascending (v1.0–v1.34) to descending per POL-32 `changelog_monotonic_descending` authorization; prior ascending convention (v1.0 top → v1.34 bottom) documented here for historical reference |
| ADR-027 | ASCENDING (oldest top) | v1.0 top → v1.9 bottom |
| ADR-028 | DESCENDING (newest top) | v1.0 was sole row at authoring; each new row must be prepended above the previous |

**FB-IMPL-P16-ARCH's ascending flip of ADR-028 was a sample-biased error.** It surveyed ADR-025/026/027 (three files, all ascending) without enumerating ADR-022's six-precedent DESCENDING enforcement chain. This erroneously concluded a project-wide ascending convention and flipped ADR-028. This v1.8 burst REVERTS that flip.

**Rule:** Before closing any POL-26 or convention-alignment finding by claiming a "project convention," the closer MUST exhaustively enumerate ALL ADRs and their authoring conventions. Declaring a project-wide rule from a sample of fewer than all ADRs is a sample-biased sibling-convention closure — the 12th coherence-axis class.

### D8 — Timestamp Grammar Extension: Option A LOCKED (FB-IMPL-1, 2026-05-21)

**Adjudicated in FB-IMPL-1 (D-FB-IMPL-1-OPT-A), closing F-LP1-HIGH-002/003.**

BC-2.16.013 §O-001 LOCKED to **Option A (grammar extension)**. The WASM transformer plugin path (Option B) is NOT in scope for PLUGIN-MIGRATION-001-D. Rationale:

1. **WASM transformer runtime does not yet exist.** BC-2.17.* governs the WASM plugin sandbox lifecycle, but no runtime exists for loading `.prx` column-transformer plugins from within `PipelineExecutor`. Choosing Option B would add an undeclared story dependency (the transformer loader) that is not yet scheduled.

2. **Option A is fully bounded.** Adding `timestamp_formats: Vec<String>` and `timestamp_fallback_chain: Vec<String>` to `ColumnSpec` with `#[serde(default)]` is a self-contained grammar extension expressible in `spec_parser.rs`. No new crates, no WASM, no new story prerequisites.

3. **Option A keeps specs human-readable.** The timestamps fields and fallback chain are visible and auditable in the TOML without an opaque binary blob reference.

#### D8-A — Canonical Format List for Cyberint `created_at`

DTU evidence (`crates/prism-dtu-cyberint/src/types.rs`): `Alert.created_at` is typed as `serde_json::Value` — it accepts any JSON value. Fixture coverage must include:

| Format | Example | Notes |
|--------|---------|-------|
| ISO 8601 UTC | `"2024-01-15T10:30:00Z"` | Primary format; most common |
| ISO 8601 with offset | `"2024-01-15T10:30:00+02:00"` | Cyberint sometimes emits timezone-offset form |
| Unix epoch (integer) | `1705311000` | Legacy alert payloads use epoch seconds |

The TOML spec declares: `timestamp_formats = ["iso8601", "unix_epoch_seconds"]` on the `created_at` column. The normalization layer (in `PipelineExecutor`) tries each format in order and uses the first successful parse. On all-formats failure: return `SpecEngineError::TimestampParseFailure` (error code `E-SPEC-018`, registered in error-taxonomy.md by this fix-burst's architect scope).

#### D8-B — Canonical Fallback Chain for Armis Timestamp

DTU evidence (`crates/prism-dtu-armis/src/types.rs`): `DeviceRecord` has `last_seen: Option<String>` (primary) and `first_seen: Option<String>` (secondary). Fixture doc (`DeviceRecord` doc comment): `d-001` has `last_seen: null` and `first_seen: "2024-01-15T10:00:00Z"` to exercise the fallback.

Canonical fallback chain (locked, v1.10 amendment per F-LP2-HIGH-004): `first_seen` → `DateTime::now()` (fetch-time UTC).

**Amendment rationale (FB-IMPL-2):** The original §D8-B locked chain listed `["last_seen", "first_seen"]`. Listing the primary column name (`last_seen`) as the first chain entry is a semantic no-op: the fallback chain executes ONLY when `last_seen` is already confirmed null/absent, so re-fetching `last_seen` from the same row yields the same null. The doc-comment "Skip the primary field itself when it appears in the chain" in the `PipelineExecutor` timestamp fallback-chain loop (`pipeline.rs`) was false — no such skip existed. The correct canonical chain is `["first_seen"]` only. The pipeline MUST also add a defensive skip guard (`if fb_field == &col.name { continue; }`) to protect against future TOML authors who mistakenly include the primary column in the chain.

The TOML spec declares `timestamp_fallback_chain = ["first_seen"]` on the primary `last_seen` column. The normalization layer applies the chain in order; if all named columns are null/absent, falls back to `DateTime::now()` (fetch-time UTC). A `tracing::warn!` is emitted when the `now()` fallback is taken, preserving the existing audit signal (BC-2.16.013 §Postconditions §1 Armis).

#### D8-C — Implementation Contract for Implementer

The following grammar extension is in scope for PLUGIN-MIGRATION-001-D (no sub-story needed):

**Field additions to `ColumnSpec` in `crates/prism-spec-engine/src/spec_parser.rs`:**

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSpec {
    pub name: String,
    pub column_type: ColumnType,
    pub ocsf_field: Option<String>,
    #[serde(default)]
    pub options: Vec<ColumnOptions>,
    /// Ordered list of timestamp format names to try when parsing this column.
    /// Only valid when `column_type == ColumnType::Datetime`.
    /// Supported format names: `"iso8601"`, `"unix_epoch_seconds"`, `"unix_epoch_millis"`.
    /// Empty vec (default) means the column is treated as a single well-known ISO 8601 string.
    #[serde(default)]
    pub timestamp_formats: Vec<String>,
    /// Ordered list of source field names to try when the primary field is null/absent.
    /// The pipeline executor tries each field in order; if all are null/absent, falls back to
    /// `DateTime::now()` (UTC) and emits `tracing::warn!(event_type = "timestamp.fallback_to_now")`.
    /// Only meaningful when `column_type == ColumnType::Datetime`.
    /// Empty vec (default) means no fallback chain — null primary → null output column.
    #[serde(default)]
    pub timestamp_fallback_chain: Vec<String>,
}
```

**Normalization layer — where in the pipeline:**

The normalization runs inside `PipelineExecutor` during the response-to-Arrow materialization step (after HTTP response is parsed, before `PipelineResult` is returned). Specifically, per-column normalization is applied when materializing `ColumnType::Datetime` columns:

1. If `timestamp_formats` is non-empty, iterate the formats in order; use the first successful parse result. On complete failure: emit `E-SPEC-018`.
2. If `timestamp_fallback_chain` is non-empty and the primary field is null/absent: try each fallback field in order using the same `timestamp_formats` (or ISO 8601 default if formats is empty). If all fallback fields are also null/absent: use `DateTime::now()` UTC and emit `tracing::warn!(event_type = "timestamp.fallback_to_now", column = %col_name)`.

**Default behavior (backward compatible):** Both fields default to empty `Vec` via `#[serde(default)]`. Existing TOML specs that do not declare these fields parse identically to current behavior (ISO 8601 expected, no fallback chain, no multi-format retry). This is a strictly additive grammar extension — no existing spec is invalidated.

**Error code:** `E-SPEC-018` — `TimestampParseFailure` — added to error-taxonomy.md by this fix-burst. Emitted when `timestamp_formats` is non-empty and no format successfully parsed the field value. Existing `E-SPEC-NNN` codes are unchanged.

**Validation gate (BC-2.16.009):** Add validation rule: if `timestamp_formats` contains unrecognized format names (not in `["iso8601", "unix_epoch_seconds", "unix_epoch_millis"]`), emit `E-SPEC-001` validation error at load time. Recognized format names are a closed set as of this ADR; additions require a new §D8 amendment.

### D9 — Documented-Gap Entries in Spec Are Permitted with Explicit DTU-EXT-NNN Blocker Reference

**Adjudicated in FB-IMPL-1 (D-FB-IMPL-1-MED-001), closing F-LP1-MED-001. Scope boundary clarified in FB-IMPL-2 (F-LP2-MEDIUM-001).**

ADR-028 §D5 ("spec entry for a URL path that has no corresponding DTU route registration is an architectural violation") is clarified: this prohibition applies to **active** spec entries that the implementer intends to exercise in parity tests. It does NOT prohibit **documented-gap entries** that meet all three of:

1. The spec table entry is explicitly marked in the TOML with an inline comment referencing the DTU-EXT-NNN gap ID (e.g., `# DTU-EXT-001: no DTU route registered; parity test is SKIP`).
2. The corresponding parity test is unconditionally `#[ignore]`-tagged with the gap message per EC-016-013-006.
3. The gap is catalogued in BC-2.16.013 §Known Gaps with the recommended resolution.

**§D9 scope boundary (FB-IMPL-2 clarification — F-LP2-MEDIUM-001):** The documented-gap exception covers **table-level gaps** (an entire table endpoint has no DTU route). It does NOT cover **parameter-level projections** — query parameters or pagination fields declared in TOML that the DTU route struct does not accept. A parameter with no DTU validation backing is not a "documented gap" under §D9; it is a spec overclaim under §D1 and must be removed until a DTU-EXT-NNN entry provides the corresponding DTU route/struct extension. Concretely: `page_size = 100` in `cyberint.sensor.toml` pagination block is removed (see implementer handoff below) because `AlertListParams` in `crates/prism-dtu-cyberint/src/routes/alerts.rs` has no `page_size` field. DTU-EXT-005 is registered in BC-2.16.013 §Known Gaps to track the future DTU extension.

Under this clarification, the CrowdStrike `incidents` table REMAINS in `crowdstrike.sensor.toml` as a documented-gap entry. Story AC-001's `tables.len() == 3` count (detections + devices + incidents) remains correct. The incidents table is architecturally forward-looking — the real Falcon Detects/Incidents API exists and is in scope for a follow-up DTU-EXT-001 story.

**Architect-handoff note for PO (F-LP1-MED-001 AC amendment):** Story AC-001 already states `tables.len() == 3` with an explanation that incidents is DTU-EXT-001 gated. No AC text change is needed; the existing AC-001 PASS criterion note accurately describes the documented-gap behavior. PO need not amend AC-001.

### D10 — Co-Merge Contract: PLUGIN-MIGRATION-001-D and PLUGIN-MIGRATION-001-A

**Adjudicated in FB-IMPL-1 (D-FB-IMPL-1-MED-005), closing F-LP1-MED-005.**

PLUGIN-MIGRATION-001-D spec declares Claroty `auth_type = "bearer_static"` (DTU-grounded, per §D2). The LIVE adapter until PLUGIN-MIGRATION-001-A merges returns `"cookie_roundtrip"` from `ClarotyAuth::auth_type_name()`. ADR-023 Rule 2 enforcement (INV-AUTH-OPEN-003 / BC-2.01.016) checks auth_type at spec-load; if the spec's declared `auth_type` does not match the registered SensorAuth implementation's `auth_type_name()`, `E-SPEC-012` is emitted.

This creates a REAL runtime regression risk: anyone running `prism start` on a develop build that has PLUGIN-MIGRATION-001-D merged but NOT PLUGIN-MIGRATION-001-A merged will encounter `E-SPEC-012` on the Claroty sensor spec load.

**Decision: Option (a) — Co-Merge Contract.**

PLUGIN-MIGRATION-001-D and PLUGIN-MIGRATION-001-A MUST be deployed to production simultaneously. The story dependency graph is amended:

- `PLUGIN-MIGRATION-001-D.blocks` gains: `PLUGIN-MIGRATION-001-A` (already present per INV-PARITY-001)
- `PLUGIN-MIGRATION-001-D.postconditions` gains: explicit co-deploy annotation (see story §Postconditions amendment below)
- `PLUGIN-MIGRATION-001-A.depends_on` gains: `PLUGIN-MIGRATION-001-D` (already structurally implied; made explicit)

ADR-028 §D6 documents the auth migration window. §D10 closes the window by specifying that the production deployment MUST be atomic across both stories. Development and CI builds may have either story merged independently (CI does not run prism start with production credentials); the regression risk is PRODUCTION deployment only.

Feature flags (Option b) are rejected: they add runtime branch complexity to solve a deployment-sequencing problem. The sequencing is already enforced by the story dependency graph and the co-merge contract stated here.

### D11 — OAuth2 Credential Substitution Model for Plugin Dispatch (PLUGIN-MIGRATION-001-E)

**Adjudicated 2026-05-24 (PLUGIN-MIGRATION-001-E PR-LEVEL CRIT #2, user-authorized Option A fix-in-scope).**

#### Problem

`boot.rs::validate_and_construct_auth_providers` constructs `credential_handle = format!("sensor:{sensor_id}")` (e.g., `"sensor:crowdstrike"`) — an opaque keyring reference per AD-017. The handle is forwarded to `dispatch_plugin_acquire_token`, which places it in `PluginConfigMap` under key `"credential_handle"`. The WASM guest `acquire_token` then executes:

```rust
let form_body = format!("{}&grant_type=client_credentials", credential_handle);
```

This produces `sensor:crowdstrike&grant_type=client_credentials` — not a valid OAuth2 form body. The real API rejects this with 4xx. Tests mask the bug because they pass literal `"client_id=my-id&client_secret=my-secret"` as `credential_handle`.

#### Decision: Option C — Host Resolves at Dispatch Time via PluginConfigMap Injection

Before calling `dispatch_plugin_acquire_token`, the host resolves `credential_handle` to `(client_id, client_secret)` via `prism_credentials::resolve_credential` and populates `PluginConfigMap` with explicit keys `"client_id"` and `"client_secret"`. The WASM guest reads these via `host::get-config` and builds the OAuth2 form body itself.

**Option A (host_http_request sentinel substitution) — Rejected.** Sentinel parsing (`${credential:handle}` pattern in POST body) is fragile: any body escaping or field ordering change by the guest silently breaks substitution. The parsing is implicit magic buried in the HTTP layer, invisible to callers and reviewers. The sentinel approach also mixes two separate concerns (HTTP execution and credential resolution) in the same function, violating single-responsibility.

**Option B (WIT param expansion to client_id/client_secret strings) — Rejected.** Passing resolved credential values as WIT string parameters exposes them to any wit-bindgen trace, debug log, or WASM memory inspector. AD-017 prohibits credential values from transiting AI context; wit-bindgen logging of WIT call parameters would be an AD-017 violation. Additionally, this requires a WIT versioning change and ADR amendment across BC-2.17.006, which has wider blast radius.

**Option C is chosen** because:

1. **Explicit data flow.** `get-config("client_id")` and `get-config("client_secret")` are direct, readable, and grep-able. No implicit substitution magic.
2. **Lowest blast radius.** `PluginConfigMap` is the existing plumbing; no WIT changes, no new host functions, no signature changes to `dispatch_plugin_acquire_token`.
3. **Bounded credential exposure.** Credentials live in `HostState.config` (a `HashMap<String,String>` wrapped in `Arc`) for the duration of the single wasmtime Store call. The Store is dropped when `dispatch_plugin_acquire_token` returns — credentials are not retained between calls.
4. **Tracing audit compliance (AD-017).** `HostState.config` is never logged. `host_get_config` returns `Option<String>` to the guest — the guest can use the value in a format string (the POST body) but cannot emit it to the tracing subscriber (which is host-owned; guests call `host::log`, which uses the message string, not config values). The host must NOT add tracing of config values in `host_get_config`.

#### AD-017 Compliance Analysis

| Criterion | Option C assessment |
|-----------|---------------------|
| Credential values never in tracing logs | PASS — `host_get_config` must not emit the returned value; existing implementation returns silently |
| Credential values never in error messages | PASS — `AuthError` variants carry structural descriptions, not values |
| Credential values never in KV store | PASS — only `token` (the *result* bearer token) and `expires_at_secs` are KV-stored; never `client_secret` |
| Credential values not retained across calls | PASS — `Arc<PluginConfigMap>` is constructed per-dispatch in `dispatch_plugin_acquire_token`; Store drop deallocates the Arc copy |
| Credential values not readable from guest WASM linear memory after call | PASS — the WASM Store is dropped after `func.call`; linear memory is deallocated |

**One required guard:** The host MUST NOT call `tracing::debug!` or any `tracing::*!` macro that would emit `client_id` or `client_secret` values retrieved from `PluginConfigMap`. The `host_get_config` implementation is currently silent — that MUST remain true.

**TD-S-PLUGIN-PREREQ-B-002 note:** The `AuthToken` zeroize gap (bearer token in heap after drop) is pre-existing and tracked separately. Option C does not worsen it — `client_id` and `client_secret` strings in `PluginConfigMap` are analogously subject to the same gap, but they are short-lived (dropped on Store drop) and the existing TD scope already covers `AuthToken` zeroize as a future hardening task.

#### Data Flow (Production)

```
boot.rs::validate_and_construct_auth_providers
  ↳ PluginAuthProvider::new(runtime, plugin_id, "sensor:crowdstrike", token_endpoint)
      → stores credential_handle = "sensor:crowdstrike"

PipelineExecutor calls auth_provider.acquire_token()
  ↳ PluginAuthProvider::acquire_token
      ↳ resolve_credential("client_id_or_org", "crowdstrike", "client_id")
             → SecretString("actual-client-id")
        resolve_credential("client_id_or_org", "crowdstrike", "client_secret")
             → SecretString("actual-client-secret")
        PluginConfigMap {
          "client_id"      → "actual-client-id",
          "client_secret"  → "actual-client-secret",
          "token_endpoint" → "https://api.crowdstrike.com/oauth2/token",
        }
      ↳ runtime.dispatch_plugin_acquire_token(plugin_id, &config)

plugin guest acquire_token:
  client_id     = host::get_config("client_id")     → "actual-client-id"
  client_secret = host::get_config("client_secret") → "actual-client-secret"
  form_body     = format!("client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials")
  → POST /oauth2/token with valid form body
```

#### Affected Files

| File | Change |
|------|--------|
| `crates/prism-bin/src/boot.rs` | `validate_and_construct_auth_providers`: `PluginAuthProvider::new` signature change — replace `credential_handle: String` with explicit `(client_id_cred_name, client_secret_cred_name)` names OR add credential resolution call-site; see §Implementation Contract |
| `crates/prism-spec-engine/src/auth_provider.rs` | `PluginAuthProvider::acquire_token`: resolve credentials before dispatch; pass resolved `client_id` and `client_secret` in PluginConfigMap |
| `crates/prism-spec-engine/src/plugin/mod.rs` | `dispatch_plugin_acquire_token`: accept `config: &PluginConfigMap` (or add `client_id` and `client_secret` params) replacing the `credential_handle: &str` param |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` | `acquire_token`: replace `format!("{}&grant_type=client_credentials", credential_handle)` with `get_config("client_id")` + `get_config("client_secret")` reads; build form body explicitly |
| Test files in `prism-spec-engine` | Transition literal-form-body tests: `credential_handle = "client_id=test&client_secret=test"` → explicit config map entries `"client_id" → "test"`, `"client_secret" → "test"` |

#### Implementation Contract for Implementer

**Signature change in `dispatch_plugin_acquire_token`** — replace the `credential_handle` param with a resolved config map:

```rust
pub fn dispatch_plugin_acquire_token(
    &self,
    plugin_id: &str,
    config: &PluginConfigMap,   // contains "client_id", "client_secret", "token_endpoint"
) -> Result<String, PluginError>
```

The `PluginConfigMap` passed in MUST contain at minimum:
- `"client_id"` — resolved OAuth2 client ID (never an opaque handle)
- `"client_secret"` — resolved OAuth2 client secret (never an opaque handle)
- `"token_endpoint"` — full URL for POST /oauth2/token

**Guest `acquire_token` change** — replace the credential_handle usage:

```rust
pub(crate) fn acquire_token(
    host: &impl HostInterface,
    token_endpoint: &str,
) -> Result<String, AuthError> {
    let client_id = host.get_config("client_id")
        .ok_or_else(|| AuthError::Internal("client_id absent from host config (EC-006b)".to_string()))?;
    let client_secret = host.get_config("client_secret")
        .ok_or_else(|| AuthError::Internal("client_secret absent from host config (EC-006c)".to_string()))?;
    let form_body = format!(
        "client_id={}&client_secret={}&grant_type=client_credentials",
        client_id, client_secret
    );
    // ... remainder unchanged
}
```

Remove `credential_handle: &str` from the function signature. The WIT `acquire-token` export also loses the `credential-handle` param (or it is retained as an ignored compatibility stub — implementer decides based on WIT versioning constraints, but no production code path may use it to build a form body).

**Credential resolution in `PluginAuthProvider::acquire_token`:**

```rust
// In prism-spec-engine/src/auth_provider.rs
async fn acquire_token(&self) -> Result<AuthToken, SpecEngineError> {
    // Resolve credentials from prism-credentials resolution chain (BC-2.03.006).
    let client_id = prism_credentials::resolve_credential(
        &self.client_id_or_org,     // org/tenant scoping
        &self.sensor_id,
        "client_id",
    ).await.map_err(|e| SpecEngineError::AuthRefreshFailed {
        sensor_id: self.sensor_id.clone(),
        detail: e.to_string(),  // detail is structural (not a credential value)
    })?;

    let client_secret = prism_credentials::resolve_credential(
        &self.client_id_or_org,
        &self.sensor_id,
        "client_secret",
    ).await.map_err(|e| SpecEngineError::AuthRefreshFailed {
        sensor_id: self.sensor_id.clone(),
        detail: e.to_string(),
    })?;

    let config = PluginConfigMap::from([
        ("client_id".to_string(), client_id.expose_secret().to_string()),
        ("client_secret".to_string(), client_secret.expose_secret().to_string()),
        ("token_endpoint".to_string(), self.token_endpoint.clone()),
    ]);

    let token_str = self.runtime.dispatch_plugin_acquire_token(&self.plugin_id, &config)?;
    Ok(AuthToken::new(token_str))
}
```

`expose_secret()` is called at the `PluginConfigMap` construction boundary — this is the sole location where credential values are materialized from `SecretString`. The `PluginConfigMap` lifetime is bounded to the `dispatch_plugin_acquire_token` call frame.

**Test transition strategy:**

Existing tests pass `credential_handle = "client_id=test&client_secret=test"` to `acquire_token`. After this change:
1. Unit tests for `acquire_token` (in `lib.rs`) pass explicit config entries via `MockHost::get_config` returning `Some("test")` for `"client_id"` and `"client_secret"`.
2. The existing `test_acquire_token_form_body_contains_required_params` is renamed/updated to verify `client_id=my-id` appears in the form body when `get_config("client_id")` returns `"my-id"`.
3. Integration tests for `dispatch_plugin_acquire_token` (in `mod.rs`) pass an explicit `PluginConfigMap { "client_id" → "...", "client_secret" → "...", "token_endpoint" → "..." }`.

No existing test behavior is lost — the same assertions on form body content and error paths remain; only the setup changes from a pre-formatted string to explicit config keys.

**EC-006 error code extension:**
- `EC-006` (existing): `token_endpoint` absent from host config → `AuthError::Internal`.
- `EC-006b` (new): `client_id` absent from host config → `AuthError::Internal("client_id absent from host config (EC-006b)")`.
- `EC-006c` (new): `client_secret` absent from host config → `AuthError::Internal("client_secret absent from host config (EC-006c)")`.

**BC-2.16.002 catalog entries required** (SAP-1):
No new `event_type` values are introduced by this change. The existing `plugin.auth_token_parse_error` emission (BC-2.16.002 row 37) remains unchanged. No new catalog rows required.

### D12 — Cyberint Cookie Auth: Real-API vs DTU Model Divergence (S-DEMO-001)

> **[SUPERSEDED by ADR-031 §D4 2026-05-29 — DTU=true-DTU principle adoption]**
> §D12's acceptance of the `cyberint_session` DTU shortcut is reversed. The correct
> cookie name is `access_token` (matching poller-express real-API behavior). The correct
> prism implementation is `StaticCookieAuthProvider` (no login step), not `CookieLoginAuthProvider`.
> The `build_request` dispatch for `CookieRoundtrip` must inject `Cookie: access_token={token}`.
> This body is preserved for traceability (POL-1 append-only). Do NOT implement per §D12 contract.
> Implement per ADR-031 §D3 contract.

**Adjudicated 2026-05-29 (architect, S-DEMO-001 v1.1 gap analysis; reference: poller-express semport).**

#### Background

The `auth_type = "cookie_roundtrip"` label for Cyberint (§D2, D-737 LOCKED) correctly identifies that Cyberint uses cookie-based auth. However the label name is ambiguous: it implies an actual round-trip login exchange, but the real Cyberint API uses a simpler pattern.

**Real Cyberint API (poller-express reference):** The Go reference implementation (`poller-express`, `.factory/semport/poller-express/poller-express-broad-sweep.md §2.1`) injects the API key as a static `access_token` cookie on every HTTP request via a `cookieTransport` (`http.RoundTripper`). There is NO login step. The credential is an API key loaded from `CYBERINT_API_KEY` (or file-backed variant), not a session token. The cookie name is `access_token`.

**DTU clone model (`prism-dtu-cyberint`):** The DTU implements a stateful session model — `POST /login` generates a UUID session token and issues it as `Set-Cookie: cyberint_session={uuid}`. Subsequent requests present this session cookie. The cookie name is `cyberint_session`. This is a deliberate simplification that tests cookie-handling behavior without requiring a real credential exchange.

#### Cookie Name Reconciliation

| Context | Cookie name | Mechanism |
|---------|-------------|-----------|
| Real Cyberint API (poller-express) | `access_token` | Static API key injection; no login step |
| DTU clone (`prism-dtu-cyberint`) | `cyberint_session` | UUID session token from `POST /login`; per-session |

The two names are intentionally different. The DTU's `cyberint_session` name exercises the session-cookie pattern; the real API's `access_token` name reflects static-credential injection. Both are `cookie_roundtrip` auth_type under the current taxonomy.

#### S-DEMO-001 Scope Decision

For the live demo (which runs against the DTU clone), the **DTU model governs**:
- `CookieLoginAuthProvider` performs `POST {base_url}/login` → parses `Set-Cookie: cyberint_session={token}` → returns the token string.
- `PipelineExecutor::build_request` (amended in S-DEMO-001) injects `Cookie: cyberint_session={token}` for `AuthType::CookieRoundtrip`.
- The `CookieLoginAuthProvider` MUST use the `base_url` from `ResolvedSensorSpec` (with per-org overlay applied) — not the raw type-spec `base_url`. Failure here breaks demo routing to the DTU clone (see S-DEMO-001 EC-005).

#### Production Path (Future Story)

For production Cyberint auth (real API), the correct model is static-cookie injection matching poller-express:
- A future `StaticCookieAuthProvider` injects `Cookie: access_token={api_key}` on every request without any login step.
- The `api_key` comes from the credential store at fetch time (not held at construction per AD-017).
- `build_request` already dispatches on `AuthType` — adding `AuthType::CookieStatic` requires a new enum variant and a new `StaticCookieAuthProvider` in a separate follow-up story.
- Alternatively, a new TOML field `auth_cookie_name` (MEDIUM complexity) could parameterize the cookie name under the existing `cookie_roundtrip` type, eliminating the need for a new auth_type variant.

**Decision for whether to add `auth_cookie_name` or `AuthType::CookieStatic`:** Deferred to the production Cyberint auth story. S-DEMO-001 MUST NOT change the `auth_type` enum or add `auth_cookie_name` to the TOML grammar — that is a scope-expanding cross-cutting change. S-DEMO-001 ships only the DTU-model path; the auth_type taxonomy amendment is a follow-up.

#### `build_request` Pipeline Amendment Scope

`PipelineExecutor::build_request` currently injects ALL tokens as `Authorization: Bearer {token}`. For `AuthType::CookieRoundtrip`, it must inject `Cookie: cyberint_session={token}` instead. The amended function signature gains `auth_type: &AuthType` (passed from `issue_request_with_retry` which already has access to `spec`). Dispatch table:

| AuthType | Header injected |
|----------|----------------|
| `CookieRoundtrip` | `Cookie: cyberint_session={token}` |
| `BearerStatic` | `Authorization: Bearer {token}` |
| `Oauth2ClientCredentials` | `Authorization: Bearer {token}` |
| `CustomViaPlugin` | `Authorization: Bearer {token}` |

The cookie name `cyberint_session` is hardcoded for the `CookieRoundtrip` variant in S-DEMO-001. When `auth_cookie_name` or `CookieStatic` lands, this dispatch will be generalized.

**Invariant:** `build_request` must dispatch by auth_type, not assume all tokens are bearer tokens. The pre-S-DEMO-001 behavior (always `Authorization: Bearer`) was an implementation gap, not an architectural decision.

### D13 — BearerStaticCredentialAuthProvider Pattern: Retirement of Bare AdapterAuthStrategy::BearerStatic (S-DEMO-002)

**Adjudicated 2026-06-03 (architect, S-DEMO-002 ADV-SDEMO002-P01-CRIT-001 disposition). Clarifying note — no new ADR warranted.**

#### Background

S-DEMO-002 introduced `BearerStaticCredentialAuthProvider`, which implements the `AuthProvider` trait and resolves bearer token credentials asynchronously via `resolve_credential` at `acquire_token` time. This mirrors the existing `StaticCookieAuthProvider` pattern established in S-DEMO-001 (see §D12 and ADR-031 §D3).

ADV-SDEMO002-P01-CRIT-001 identified that the prior path — a bare `AdapterAuthStrategy::BearerStatic` variant consuming a synchronous `ProductionCredentialResolver::resolve` return — was a defect: `ProductionCredentialResolver` was a placeholder that performed no real async resolution, resulting in fail-open behavior on missing credentials. This violated the fail-closed credential invariant (DI-002, BC-2.03.002).

#### Decision

**`bearer_static` sensors resolve credentials via `BearerStaticCredentialAuthProvider`, held as `AdapterAuthStrategy::Plugin`.**

The implementation contract:

1. `BearerStaticCredentialAuthProvider` implements `AuthProvider`. Its `acquire_token` method calls `resolve_credential(org_slug, sensor_id, "bearer_token")` asynchronously. On credential absence, it returns `SpecEngineError::CredentialNotFound` (fail-closed). On success, it returns `AuthToken::new(resolved_value)`.

2. The sensor adapter holds the provider as `AdapterAuthStrategy::Plugin(Arc<dyn AuthProvider>)` — the same variant used by `StaticCookieAuthProvider`. No new `AdapterAuthStrategy` variant is introduced.

3. `PipelineExecutor::issue_request_with_retry` dispatches `AdapterAuthStrategy::Plugin` uniformly for both bearer-static and cookie-based auth. The `build_request` auth-type-aware dispatch (established in S-DEMO-001, §D12 invariant) remains correct — `AuthType::BearerStatic` sensors still inject `Authorization: Bearer {token}`; the change is in HOW the token is obtained (via `BearerStaticCredentialAuthProvider::acquire_token`), not in how it is injected.

4. **Retired path:** The bare `AdapterAuthStrategy::BearerStatic(token_string)` constructor pattern — where a credential resolver return was synchronously unpacked and stored as a plain string in the strategy — is RETIRED. This path was a defect: it performed resolution at construction time (wrong: resolution must happen at request time, per BC-2.03.002 §Postconditions) and relied on a `ProductionCredentialResolver` placeholder that was not wired to the actual prism-credentials chain.

#### Canonical Credential Reference Name

The canonical `credential_ref` name for `bearer_static` sensors is `bearer_token`. Operator environment variable convention: `<SENSOR_ID_UPPER>_BEARER_TOKEN` (e.g., `CLAROTY_BEARER_TOKEN`). This follows the four-tier per-client resolution chain defined in BC-2.06.003.

> **[ADR-032 SUPERSEDES env-var format (per-client convention):]** The `<SENSOR_ID_UPPER>_BEARER_TOKEN`
> global format above is the LEGACY env-var convention from before ADR-032. ADR-032 (per-client
> credential format) introduced `PRISM_CLIENTS_{ORG}_SENSORS_{SENSOR}_BEARER_TOKEN` as the
> canonical operator env-var format (e.g., `PRISM_CLIENTS_ACMECORP_SENSORS_CLAROTY_BEARER_TOKEN`).
> New operator documentation and sensor spec comments MUST use the ADR-032 per-client format;
> the global format is accepted only for backward-compat. The `ARMIS_BEARER_TOKEN` example
> previously in this section is now stale — Armis is reclassified to
> `auth_type = "token_exchange"` + native `DeclarativeHttpAuthProvider` +
> `[[credential_refs]]` block with `name = "secret_key"` by ADR-053 §D2 + ADR-054 D1 (D-1895);
> `CLAROTY_BEARER_TOKEN` remains valid as a legacy alias only.

#### Consistency with Existing AuthProvider Patterns

| Sensor auth_type | AuthProvider | credential_ref | resolve_credential key |
|------------------|--------------|----------------|------------------------|
| `oauth2_client_credentials` | `DeclarativeHttpAuthProvider` (native — sole live path; per ADR-054 D2/D5). `PluginAuthProvider` (WASM) via `auth_plugin` is **spec-load-rejected** post-ADR-054: `auth_type = "oauth2_client_credentials"` + `auth_plugin` present → E-SPEC-028(b) unconditional rejection at spec-load per ADR-054 D10(b). The "when `[auth_acquisition]` present" conditional framing is superseded; there is no live WASM dispatch path. (`crowdstrike-oauth2.prx` retired by ADR-054 D5.) | `client_id`, `client_secret` (resolved via per-org credential chain per §D11) | `client_id`, `client_secret` |
| `cookie_roundtrip` (Cyberint) | `StaticCookieAuthProvider` | `access_token` (per ADR-031 §D3) | `access_token` |
| `bearer_static` (Claroty) | `BearerStaticCredentialAuthProvider` | `bearer_token` | `bearer_token` |

> **[ADR-053 §D2 SUPERSEDES Armis row (2026-07-20, D-1889); ADR-054 D1 amends auth
> mechanism (2026-07-21, D-1895):]** Armis is reclassified to `auth_type = "token_exchange"`
> with native `DeclarativeHttpAuthProvider` (no WASM plugin), `header_scheme = "raw"`,
> `[[credential_refs]]` block with `name = "secret_key"`. The `bearer_static` row above is narrowed to Claroty only.
> Implementers building Armis MUST NOT use `bearer_static` + `BearerStaticCredentialAuthProvider`.
> The operative contract is ADR-053 §D2 + ADR-054 D1.

All `AuthProvider` implementations are fail-closed: a missing credential at `acquire_token` time returns an error that propagates to `PipelineExecutor` and surfaces as `E-SENSOR-NNN` to the caller. No partial or default token values are returned.

---

## Rationale

The DTU clone routes are the correct and only viable grounding reference for TOML sensor spec URLs and auth_type values for four compounding reasons:

1. **Legacy adapter code is demonstrably wrong.** Pass-4 cross-checks confirmed all four sensor adapters carry URL simplification bugs (e.g., `/queries/detections` vs. the real `/detects/queries/detects/v1`). Grounding specs against a buggy reference propagates the bug into every downstream BC, story, holdout scenario, and parity test — defeating the purpose of spec-first development.

2. **DTU clones derive from real API documentation (ADR-003).** DTU routes are authored against real third-party API specs, not against Rust adapter code. They are the closest executable proxy to the real APIs that CI can reach. Grounding against DTU routes is equivalent to grounding against real API docs; where they diverge, a DTU extension story is the correct resolution (§D5).

3. **DTU routes are executable and CI-verifiable.** Real API documentation is not executable in CI. Legacy adapter code is deleted by PLUGIN-MIGRATION-001-A. The fixture JSON and DTU clone routes are the only durable, CI-verifiable reference surfaces available after 001-A merges.

4. **CLAUDE.md §Source-of-Truth Precedence #7 extends to reference selection.** The SPEC wins on code-vs-spec conflicts. This principle extends one layer up: when the code used to author a spec is itself wrong, the spec must ground against the correct reference (DTU routes) rather than canonizing the latent bug. The purity of the parity tests depends on this grounding decision being correct.

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

## Source / Origin

This ADR was authored during the PLUGIN-MIGRATION-001-D LOCAL adversarial cascade, pass-4 (2026-05-20), when the fresh-context adversary surfaced a systemic regression: TOML specs had been re-grounded against production Rust adapter URLs during pass-3 closures, but those adapters themselves carried latent URL simplification bugs. The root cause was the absence of an explicit grounding rule — each spec author chose a reference independently and chose the wrong one.

D-737 (STATE.md v7.424 `architectural_decisions_locked`) records the user-adjudicated Decisions 1 and 4 that established the DTU-grounding principle and prohibited adapter-as-reference. This ADR is the durable spec codification of those locked decisions.

Subsequent amendments (§D6 through §D13) were authored during PLUGIN-MIGRATION-001-A through PLUGIN-MIGRATION-001-E and S-DEMO-001/002 cascades as the principle was extended to cover auth_type grounding, credential substitution, co-merge contracts, and the BearerStatic auth provider pattern. Each amendment is attributable to a specific cascade pass and adjudication decision in STATE.md.

ADR-053 §D1/§D2/§D5 (2026-07-20, D-1889) supersedes the core §D1/§D2/§D5 grounding rules of this ADR: the new authority chain is vendor OpenAPI → spec → DTU (replacing DTU → spec), with dtu-validator scoring DTU fidelity against OpenAPI rather than DTU serving as the ground-truth source.

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
| 1.23 | 2026-07-21 | architect | FIX-BURST 9 (OBS-1): `modified:` frontmatter inline comment `# v1.18 HIGH-2 (FIX-BURST): …` removed — version-pinned narrative in frontmatter fields is the same self-cite volatility class closed at PG-ADR-STATUS-SELFCITE-001; replaced with non-volatile `# see §Changelog top row`. POL-29 class sweep: only this file carried the defect in the Wave-A perimeter; full decisions/ sweep confirms no other frontmatter-field version-pinned inline comments exist. |
| 1.22 | 2026-07-21 | architect | FIX-BURST 7 (OBS-1): §D2 Armis supersession blockquote and §D13 env-var blockquote + §D13 Armis consistency-table blockquote — scalar `credential_ref = "secret_key"` replaced with canonical `[[credential_refs]]` block form with `name = "secret_key"` (3 occurrences; `credential_ref` is the old scalar grammar; `[[credential_refs]]` with `name =` is the canonical array-of-tables form per ADR-054 §D3). POL-29 sweep: zero live scalar `credential_ref = "secret_key"` hits remain in live content sections. `modified` comment updated. |
| 1.21 | 2026-07-21 | architect | OBS-1: §Status stale self-cite corrected — "current frontmatter v1.10 per §Changelog" replaced with non-volatile form "current version per §Changelog top row" (permanently retires this staleness class). |
| 1.20 | 2026-07-21 | architect | MED-2: §D13 oauth2_client_credentials consistency-table row updated — `PluginAuthProvider` (WASM) path marked spec-load-rejected per ADR-054 D10(b) (E-SPEC-028(b) unconditional rejection for auth_type ∈ {oauth2_client_credentials, token_exchange} + auth_plugin present); `DeclarativeHttpAuthProvider` (native) is the sole live path; "when [auth_acquisition] present" conditional framing removed (superseded by D10(b)'s unconditional rule). Frontmatter `amended_by` framing updated to reflect D10(b) unconditional rejection. |
| 1.19 | 2026-07-21 | architect | MED-3: §D7 ADR-026 convention-table row updated — ADR-026 reordered from ascending to descending at v1.35 (2026-07-20) per POL-32 `changelog_monotonic_descending`; table row and §D7 lock rule amended to reflect POL-32 as authorizing policy for deliberate convention reorders. MED-4: §D13 "three-tier resolution chain" corrected to "four-tier" (BC-2.06.003 is authoritatively four-tier; consistent with ADR-053/054). |
| 1.18 | 2026-07-21 | architect | HIGH-2 (FIX-BURST): §D2 Armis inline supersession blockquote corrected — `custom_via_plugin` (token-exchange via `armis-token-exchange.prx` WASM plugin) → `auth_type = "token_exchange"` with native `DeclarativeHttpAuthProvider` (ADR-054 D1/D4, D-1895); no WASM plugin. §D13 env-var blockquote corrected — `custom_via_plugin + credential_ref = "secret_key"` → `token_exchange + native DeclarativeHttpAuthProvider + credential_ref = "secret_key"`. §D13 oauth2_client_credentials consistency-table row corrected — `PluginAuthProvider (WASM)` → `DeclarativeHttpAuthProvider (native)` when `[auth_acquisition]` present (ADR-054 D2/D5; crowdstrike-oauth2.prx retired). §D13 Armis consistency-table blockquote corrected — `custom_via_plugin (armis-token-exchange.prx, ...)` → `token_exchange, native DeclarativeHttpAuthProvider` (ADR-054 D1/D4). MED-1: `amended_by` back-ref for ADR-054 added to frontmatter; ADR-054 added to `related_adrs`. |
| 1.17 | 2026-07-20 | architect | OBS-2 (ADR-053 pass-3): §D13 "Canonical Credential Reference Name" — ADR-032 per-client env-var supersession note added. The legacy `<SENSOR_ID_UPPER>_BEARER_TOKEN` global format (e.g., `ARMIS_BEARER_TOKEN`) is retired in favour of `PRISM_CLIENTS_{ORG}_SENSORS_{SENSOR}_BEARER_TOKEN`; new specs/docs must use per-client format. `ARMIS_BEARER_TOKEN` example annotated stale (Armis reclassified to `custom_via_plugin` + `secret_key` by ADR-053 §D2); `CLAROTY_BEARER_TOKEN` valid only as legacy alias. |
| 1.16 | 2026-07-20 | architect | LOW-1: §D2 Armis row — inline supersession blockquote added pointing to ADR-053 §D2 (2026-07-20, D-1889). Armis `bearer_static` row now carries explicit at-point warning: reclassified to `custom_via_plugin` + token-exchange + `header_scheme = "raw"` + `credential_ref = "secret_key"`; `bearer_static` MUST NOT be used for Armis. Closes pass-2 adversary LOW-1 finding. |
| 1.15 | 2026-07-20 | architect | ADR-053 §D1/§D2/§D5 supersession linkage: `superseded_by:` converted to YAML list with ADR-053 §D1/§D2/§D5 entry (grounding order superseded by OpenAPI-first; Armis LOCKED auth_type D-747 + Cyberint LOCKED auth_type D-747 superseded; D-1889 2026-07-20). `related_adrs` updated to include ADR-053. §Source/Origin + §Rationale sections added (template compliance). TD-VSDD-091 volatile cite remediated at §D8-B (stable behavioral anchor substituted). §D13 consistency table Armis row narrowed to Claroty-only (HIGH-1 fix): ADR-053 §D2 supersedes Armis `bearer_static`; Armis reclassified to `custom_via_plugin` (`armis-token-exchange.prx`, `header_scheme = "raw"`, `credential_ref = "secret_key"`). |
| 1.14 | 2026-06-03 | architect | §D13 ADDED — BearerStaticCredentialAuthProvider pattern clarifying note (ADV-SDEMO002-P01-CRIT-001 disposition). Documents that `bearer_static` sensors now resolve credentials via `BearerStaticCredentialAuthProvider` (AuthProvider pattern, async `acquire_token` → `resolve_credential("bearer_token")`, fail-closed on missing credential), held as `AdapterAuthStrategy::Plugin`. Retires bare `AdapterAuthStrategy::BearerStatic` constructor path (defect: sync placeholder resolver, resolution at construction time rather than request time). Canonical `credential_ref` name `bearer_token`, operator env var `<SENSOR>_BEARER_TOKEN`. Consistency table added. anchor_stories += S-DEMO-002. |
| 1.13 | 2026-05-29 | architect | §D12 SUPERSEDED — ADR-031 §D4 reverses the DTU-shortcut acceptance. §D12 body annotated `[SUPERSEDED by ADR-031 §D4 2026-05-29 — DTU=true-DTU principle adoption]`. Correct contract is now ADR-031 §D3: `access_token` cookie (not `cyberint_session`), `StaticCookieAuthProvider` (no login step), `build_request` dispatches `CookieRoundtrip → Cookie: access_token={token}`. frontmatter `superseded_by:` updated. frontmatter `version:` bumped v1.12→v1.13. |
| 1.12 | 2026-05-29 | architect | §D12 ADDED — Cyberint cookie auth real-API vs DTU model divergence (S-DEMO-001). Documents `access_token` (real API, static injection per poller-express) vs `cyberint_session` (DTU, POST /login session token) divergence. Locks DTU model for S-DEMO-001 (`CookieLoginAuthProvider` + `build_request` amendment to `Cookie: cyberint_session={token}`). Documents production path via future `StaticCookieAuthProvider` or `auth_cookie_name` TOML field. Documents `build_request` auth-type-aware dispatch invariant. anchor_stories += S-DEMO-001. |
| 1.11 | 2026-05-24 | architect | §D11 ADDED — OAuth2 Credential Substitution Model for Plugin Dispatch (PLUGIN-MIGRATION-001-E PR-LEVEL CRIT #2, user-authorized fix-in-scope). Locks Option C (host resolves credential_handle → client_id + client_secret via prism_credentials::resolve_credential; PluginConfigMap injection before dispatch). Options A (host_http_request sentinel) and B (WIT param expansion) rejected with rationale. Full AD-017 compliance analysis, data flow diagram, affected file list, implementer contract (dispatch signature change, guest acquire_token change, test transition strategy), and EC-006b/EC-006c error code extensions. BC-2.01.016 added to related_bcs. Closes F-LP12-PR-CRIT-2. |
| 1.10 | 2026-05-21 | architect | FB-IMPL-2 architect adjudication: §D8-B AMENDED — canonical Armis fallback chain corrected from `["last_seen", "first_seen"]` to `["first_seen"]` (F-LP2-HIGH-004: `last_seen` self-reference is a semantic no-op; false doc-comment "Skip the primary field itself" had no code implementation; implementer must add defensive skip guard `if fb_field == &col.name { continue; }` and fix doc-comment). §D9 SCOPE CLARIFIED — documented-gap exception covers table-level gaps only, NOT parameter-level projections; `page_size = 100` in cyberint.sensor.toml removed per §D1 (`AlertListParams` struct has no `page_size` field confirmed at alerts.rs:38-40); DTU-EXT-005 registered in BC-2.16.013 §Known Gaps (F-LP2-MEDIUM-001). §Status self-cite advanced to v1.10. BC-2.16.013 v1.12→v1.13. |
| 1.9 | 2026-05-21 | architect | (D-FB-IMPL-1-OPT-A) FB-IMPL-1 architect adjudication: §D8 LOCKS Option A (grammar extension) for BC-2.16.013 §O-001 — `timestamp_formats` + `timestamp_fallback_chain` fields added to `ColumnSpec`; Cyberint canonical formats iso8601+unix_epoch_seconds documented; Armis fallback chain `last_seen → first_seen → now()` locked; implementer contract for spec_parser.rs changes specified; E-SPEC-018 registered. §D9 clarifies §D5 documented-gap exception: incidents table REMAINS in crowdstrike.sensor.toml per documented-gap policy; AC-001 `tables.len() == 3` stands. §D10 co-merge contract: 001-D + 001-A MUST deploy to production simultaneously to prevent E-SPEC-012 regression on Claroty bearer_static vs live cookie_roundtrip; feature-flag Option b rejected; story §Postconditions annotated. |
| 1.8 | 2026-05-20 | architect | Pass-17 FB-IMPL-P17-ARCH: §Changelog rows REVERTED to descending (project per-file convention locks at authoring; ADR-028 was authored at v1.0 with descending order). FB-IMPL-P16-ARCH's ascending flip was based on sample-biased 3-ADR enumeration that missed ADR-022's 6-precedent DESCENDING enforcement chain (D-611/D-628/D-635/D-659/D-670/D-671). F-LP17-HIGH-002 closure. 12th coherence-axis class (sample-biased sibling-convention closures) codified: convention closures MUST exhaustively enumerate ALL ADRs before declaring project rule. §D7 (Per-File §Changelog Convention Lock) added. §Status self-cite advanced to v1.8. |
| 1.7 | 2026-05-20 | architect | Pass-16 FB-IMPL-P16-ARCH: §Changelog rows reordered descending→ascending to match project convention (ADR-026/025/027) per F-LP16-MED-001 (POL-26 sibling-asymmetric convention). Closes 9th coherence-axis class. Content of all prior rows preserved verbatim — only ordering changed. |
| 1.6 | 2026-05-20 | architect | FB-IMPL-P14-ARCH: F-LP14-MED-002 closure — §Status self-cite "current frontmatter v1.4" advanced to "current frontmatter v1.6" (stale after v1.5 bump in P13; same defect class as F-LP10-LOW-001). F-LP14-MED-003 closure — §D6 Action 3 parenthetical rewritten: future-tense "applied in the PLUGIN-MIGRATION-001-A merge burst" replaced with realized past-tense "applied simultaneously with this §D6 authoring in FB-IMPL-P13-ARCH; reflected in ADR-026 v1.30 frontmatter". POL-29 self-verification greps: CLEAN. |
| 1.5 | 2026-05-20 | architect | Pass-13 FB-IMPL-P13-ARCH per user Path A adjudication (D-747): `supersedes:` frontmatter field added (ADR-026 §D3 partial — non-CrowdStrike `auth_type_name()` returns). §D2 supersession prefix paragraph added: explicitly supersedes ADR-026 §D3 for Cyberint/Claroty/Armis `auth_type_name()` return values effective at PLUGIN-MIGRATION-001-A merge; CrowdStrike `"oauth2_client_credentials"` unchanged. New §D6 documents PLUGIN-MIGRATION-001-A scope expansion: rewrite three `Auth::auth_type_name()` returns to DTU-grounded values + amend Red Gate `test_BC_2_01_016_003` + bidirectional supersession linkage. F-LP13-HIGH-001 closure. |
| 1.4 | 2026-05-20 | architect | Pass-7 FB-IMPL-P7 — §Context cyberint symbol-path mis-anchor corrected: `cyberint.rs::CyberintAuth::get_page` (HALLUCINATION — wrong type namespace; method belongs to `CyberintAdapter`, not `CyberintAuth`) → `cyberint.rs::CyberintAdapter::new()` (cookie-store `reqwest::Client::builder().cookie_store(true).build()` construction) + `::get_page()` consumption. Semantic claim corrected: cookie-store is BUILT in `CyberintAdapter::new()` (not "established in per-page fetch loop"). F-LP7-HIGH-001 closure. Root cause: FB-IMPL-P6 propagated wrong type namespace from pass-6 review without grep-verifying symbol against codebase. Going-forward discipline: ALL symbol-path anchor replacements MUST be grep-verified against `crates/` before commit (TD-VSDD-059 paper-fix variant; TD-VSDD-091 anti-volatile-pin). Pass-10 FB-IMPL-P10 — §Status historical-anchor disambiguation: "Proposed 2026-05-20, v1.0" → "Proposed 2026-05-20, v1.0 (initial proposal version; current frontmatter v1.4 per §Changelog)" to prevent reader confusion about current revision. F-LP10-LOW-001 closure (POL-29 body-frontmatter coherence axis). |
| 1.3 | 2026-05-20 | architect | Pass-6 FB-IMPL-P6 — POL-25 expanded sibling-anti-pattern sweep. §D2 Armis row: `crates/prism-dtu-armis/src/lib.rs:16-17` → `crates/prism-dtu-armis/src/lib.rs` module-level `//!` doc-comment (Armis Centrix BearerStatic contract). §Context cyberint cite: `crates/prism-sensors/src/auth/cyberint.rs:155` → `::CyberintAuth::get_page` symbol path. TD-VSDD-091 anti-volatile-pin. F-LP6-LOW-001 closure (architect scope; PO + SW closing sibling sites in parallel bursts). |
| 1.2 | 2026-05-20 | architect | Pass-5 fix-burst — §D2 Cyberint row symbol-anchored: `prism-dtu-cyberint/src/routes/alerts.rs:43-46` → `::extract_session_token()` per TD-VSDD-091. F-LP5-LOW-001 closure (POL-25 sibling sweep — BC-2.16.013/HS-015 already fixed by PO this burst). |
| 1.1 | 2026-05-20 | architect | Pass-5 fix-burst — §D2 Armis row corrected from `"api_key"` (legacy adapter `auth_type_name()` return) to `"bearer_static"` (DTU `Authorization: Bearer` enforcement per `prism-dtu-armis/src/lib.rs:16-17`). The original v1.0 §D2 Armis row was itself the latent label bug §D2 was authored to immunize against — fresh-context adversary surfaced the contradiction. F-LP5-HIGH-001. |
| 1.0 | 2026-05-20 | architect | Initial version — locks D-737 Decisions 1 and 4; enumerates 5 decision rules; documents 3 DTU gap follow-ups |
