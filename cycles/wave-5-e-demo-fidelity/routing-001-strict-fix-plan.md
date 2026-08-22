---
document_type: strict-fix-plan
story_id: S-ADR058-OCSF-ROUTING-001
version: "1.0"
created: 2026-08-22
decision: D-2273
status: current
---

# S-ADR058-OCSF-ROUTING-001 — Strict-Fix Plan

Re-cascade pass-1 findings; human: "fix everything strictly" 2026-08-22.

Frozen code HEAD at plan time: `396af5722` (feature branch, pushed origin).
`just check` GREEN 5805. BC-5.39.001 LOCAL streak: 0/3.

Dependency check: prism-query / prism-bin / prism-mcp all already depend on
prism-spec-engine — no cycle risk moving helpers there.

---

## LOW-1 + OBS-1 — Shared OCSF Projection Helper

**Finding summary:** Zero-column OCSF tables (no Tier-1 columns) fall through the ST gate
without registering `class_uid` + `_sensor`. Projection logic is duplicated across
table_registry, engine (two MT sites), prism-mcp describe, and prism-bin record_batch with
no single authoritative impl.

### Implementation changes

**ADD** to `crates/prism-spec-engine/src/column_mapping.rs` two `pub` fns (after
`ocsf_field_to_arrow_name`):

1. `ocsf_projected_column_names(tbl: &TableSpec, ocsf_column_naming: bool) -> Vec<String>`
   - `flag=true`: Tier-1 `ocsf_field_to_arrow_name` names + `"class_uid"` + `"_sensor"` +
     `"raw_extensions"` iff any Tier-2 column exists (`ocsf_field == None`). Zero-column
     OCSF table (no Tier-1 at all) → `["class_uid", "_sensor"]`.
   - `flag=false`: raw `col.name` list.

2. `ocsf_projected_column_types(tbl: &TableSpec, ocsf_column_naming: bool) -> HashMap<String, ColumnType>`
   - `flag=true`: Tier-1 `(arrow_name, col.column_type)` pairs + `class_uid=Integer` +
     `_sensor=String` + `raw_extensions=Json` iff Tier-2 exists.
   - `flag=false`: `(col.name, col.column_type)` pairs.

**crates/prism-query/src/engine.rs:** make the private
`ocsf_or_raw_column_names_for_table` a thin forward to
`prism_spec_engine::column_mapping::ocsf_projected_column_names`. Both MT call sites
unchanged: `check_column_availability` MT arm; `get_initial_available_columns` MT arm.

**crates/prism-query/src/table_registry.rs** (`register_sensor` — LOW-1 fix): REMOVE the
outer `if !table.columns.is_empty()` guard for the OCSF branch. When
`spec.ocsf_column_naming`, always insert `columns_by_table` + `column_types_by_table` via
the two shared fns (so zero-column OCSF tables register `class_uid` / `_sensor`). Keep the
`!columns.is_empty()` guard ONLY on the non-OCSF `else` branch (preserve fail-open there).

**`build_ocsf_column_descriptors`** (prism-mcp/prism_describe.rs) and
**`pipeline_result_to_record_batch`** (prism-bin/spec_driven_adapter.rs): CANNOT fully
delegate (need Arrow types / descriptor text). Leave inline; add a doc-comment referencing
the shared invariant. `RG-Q-015` enforces name-set agreement between the registry and these
sites. `check_operator_type_compatibility` MT arm stays inline (per-column `effective_name`;
correct as-is).

---

## OBS-2 — Spec-Load Collision Validation

**Finding summary:** §J1/§J2/§J4 OCSF collision guards live only in the runtime
`pipeline_result_to_record_batch` — unreachable in normal operation. Collision detection
belongs at spec-load time so invalid sensor TOMLs are rejected at boot / hot-reload.

### Implementation changes

**ADD** to `crates/prism-spec-engine/src/add_sensor_spec.rs` a private fn:

```
validate_ocsf_column_collisions(spec: &SensorSpec, source_path: &Path) -> Vec<String>
```

Enforcing:
- **§J2**: Tier-1 `ocsf_field_to_arrow_name(ocsf_field)` equals a reserved synthesized
  name (`class_uid`, `category_uid`, `_sensor`, `raw_extensions`) — E-SPEC-027 + §J2 tag.
- **§J4**: Two Tier-1 columns in the same table flatten to the same arrow name (duplicate
  arrow name) — E-SPEC-027 + §J4 tag.
- **§J1**: A Tier-1 arrow name equals another column's raw `col.name` within the same table
  (shadow collision) — E-SPEC-027 + §J1 tag.

Plug into `parse_and_validate_spec_toml` as "Validation Rule 8" (after Rule 7). Error
string carries new code **E-SPEC-027** (+ §J tag + sensor ID / table name). `ValidationError`
uses `Vec<String>` — no new enum variant needed. Boot path: `ConfigInvalid → exit 2`.
Hot-reload: keeps prior spec on error (existing behavior).

Runtime `pipeline_result_to_record_batch` §J guard stays as defense-in-depth (now
unreachable in prod, but correct to keep for belt-and-suspenders).

Add E-SPEC-027 comment entry to:
- `crates/prism-core/src/error.rs` (comment block, not a new variant)
- `.factory/specs/prd-supplements/error-taxonomy.md` (new E-SPEC-027 row)

---

## New Red Gate Tests

SAP-3 end-to-end coverage required where possible.

- **RG-Q-010** `test_BC_2_11_016_zero_col_ocsf_table_st_gate_accepts_class_uid_and_sensor`
  (prism-query): zero-column OCSF table ST gate accepts `class_uid` and `_sensor` queries;
  traces to BC-2.11.016 EC-11-079 (§J5 zero-column clause).

- **RG-Q-011** `test_BC_2_11_016_zero_col_ocsf_table_st_gate_rejects_raw_col_name`
  (prism-query): available_columns == `["_sensor", "class_uid"]` only — no raw col.name.
  Traces to BC-2.11.016 EC-11-079.

- **RG-Q-012** `test_BC_2_16_003_ocsf_collision_j2_reserved_name_rejected_at_spec_load`
  (prism-spec-engine): TOML with a Tier-1 `ocsf_field` that flattens to a reserved name →
  `parse_and_validate_spec_toml` returns `Err` containing `E-SPEC-027` + `§J2`.

- **RG-Q-013** `test_BC_2_16_003_ocsf_collision_j4_intra_table_duplicate_rejected_at_spec_load`
  (prism-spec-engine): two Tier-1 columns in same table with same `ocsf_field` → error with
  `E-SPEC-027` + `§J4`.

- **RG-Q-014** `test_BC_2_16_003_ocsf_collision_j1_shadow_rejected_at_spec_load`
  (prism-spec-engine): Tier-1 arrow name shadows another column's raw `col.name` → error with
  `E-SPEC-027` + `§J1`.

- **RG-Q-015** `test_ocsf_projected_names_all_surfaces_agree`
  (prism-query table_registry): `registry.columns_for_table` == `ocsf_projected_column_names`
  output, byte-equal sorted. Enforces the shared-helper invariant across registry + engine.

---

## Spec Additions Required

### ADR-058 (architect — three new clauses)

(a) **Consolidated-Projection Invariant:** `ocsf_projected_column_names` /
`ocsf_projected_column_types` (prism-spec-engine) are the single authoritative projection
impl. `build_ocsf_column_descriptors` (prism-mcp) and `pipeline_result_to_record_batch`
(prism-bin) are documented shape-exception sites bound by RG-Q-015.

(b) **§J5** zero-column OCSF tables: when `ocsf_column_naming = true` and a table has no
Tier-1 columns, the table registers `class_uid` + `_sensor` (no `raw_extensions` because no
Tier-2 exists). ST gate accepts queries against these two names.

(c) **Spec-load §J collision validation:** `parse_and_validate_spec_toml` enforces §J1/§J2/§J4
via `validate_ocsf_column_collisions`. E-SPEC-027 on violation; boot exit 2; hot-reload
keeps prior spec. Runtime `pipeline_result_to_record_batch` §J guard = defense-in-depth.

### BC-2.11.016 (product-owner — new EC)

New EC: zero-column OCSF table presents `class_uid` / `_sensor` in the plan-gate
`available` set (anchor: RG-Q-010 / RG-Q-011). Traces to ADR-058 §J5.

### BC-2.16.003 (product-owner — new EC)

New EC: `parse_and_validate_spec_toml` rejects §J1/§J2/§J4 collisions with E-SPEC-027; boot
exit 2; hot-reload keeps prior spec. Defense-in-depth: runtime `pipeline_result_to_record_batch`
§J guard remains but is unreachable in prod. Anchor: RG-Q-012/013/014.

### error-taxonomy.md (product-owner — new error code)

E-SPEC-027: OCSF column collision at spec-load time. Discriminators: §J1 (shadow), §J2
(reserved-name), §J4 (intra-table duplicate arrow name). Boot path: ConfigInvalid → exit 2.

---

## Delivery Sequence (Next Session)

### Step 1 — Spec burst (before any code)

1. **architect**: ADR-058 — add three clauses (consolidated-projection invariant; §J5;
   spec-load collision validation via E-SPEC-027).
2. **product-owner**: BC-2.11.016 new EC (zero-column; anchor RG-Q-010/011). BC-2.16.003
   new EC (spec-load collision; anchor RG-Q-012/013/014). error-taxonomy.md E-SPEC-027 row.
3. **story-writer**: ROUTING-001 story — add ACs for LOW-1-FIX / OBS-1-FIX / OBS-2-FIX;
   enumerate RG-Q-010..015 in SAC-1 section; recount density.
4. **state-manager**: single-commit spec burst to factory-artifacts.

### Step 2 — test-writer: RG-Q-010..RG-Q-015 (RED)

All six must be failing before any implementation code is written (BC-5.38.001 / SAC-1).

### Step 3 — implementer: shared helpers + LOW-1 + OBS-2 collision validation

Sequence within implementer:
1. Add `ocsf_projected_column_names` + `ocsf_projected_column_types` to prism-spec-engine.
2. Refactor `ocsf_or_raw_column_names_for_table` in engine.rs as a thin forward.
3. Fix `register_sensor` in table_registry.rs (remove outer empty-guard for OCSF branch).
4. Add `validate_ocsf_column_collisions` to add_sensor_spec.rs; wire into
   `parse_and_validate_spec_toml` as Rule 8.
5. Add E-SPEC-027 comment to prism-core/error.rs.
6. Run `just iter prism-spec-engine` + `just iter prism-query` — all 44 RGTs green (38 prior
   + 6 new RG-Q-010..015). Run `just check` at end.

### Step 4 — re-run LOCAL 3-CLEAN cascade on frozen HEAD (after `just check` GREEN)

BC-5.39.001 requires 3 consecutive CLEAN(strict) passes. Streak resets to 0/3 on new HEAD.

### Step 5 — holdout re-gate (FRESH scenarios — HS-022 group consumed)

HS-022 group (4 scenarios) was CONSUMED at D-2270 (1 pass / 3 fail). Product-owner MUST
author FRESH holdout scenarios for this new HEAD before re-gate. Re-gate executes AFTER
LOCAL 3-CLEAN. Any holdout fail resets LOCAL streak per CLAUDE.md story-level holdout gate.

### Step 6 — demo → push → pr-manager 9-step PR → merge → post-merge state burst

After LOCAL 3-CLEAN + holdout PASS:
- demo-recorder per AC evidence
- push feature branch
- pr-manager 9-step PR cycle (PR-LEVEL adversary 3-CLEAN, holdout, security review, merge)
- state-manager post-merge burst (POL-14 auto-promotion for any new/updated BCs)

---

## Key Cross-References

| Artifact | Version at Plan Time | Notes |
|----------|---------------------|-------|
| ADR-058 | v2.28 | Will gain 3 new clauses |
| BC-2.11.016 | v1.28 | Will gain zero-column EC |
| BC-2.16.003 | v1.23 | Will gain spec-load collision EC |
| BC-2.16.002 | v2.33 | No change expected |
| ROUTING-001 story | v1.51 | Will gain OBS-2-FIX AC + RG-Q-010..015 |
| error-taxonomy.md | current | Will gain E-SPEC-027 row |
| Code HEAD | 396af5722 | Feature branch; just check GREEN 5805 |
| develop HEAD | 362e4f85 | Unchanged; PRs #240+#241 merged |

---

_This document is self-sufficient. The transcript for the session ending 2026-08-22
is cleared; resume entirely from this plan._
