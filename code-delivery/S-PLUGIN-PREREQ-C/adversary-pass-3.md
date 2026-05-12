# Adversarial Review — S-PLUGIN-PREREQ-C — LOCAL Pass 3

**HEAD:** 4bf3dfdd
**Reviewer:** adversary (fresh-context)
**Date:** 2026-05-12
**Subject:** Fix-burst-2 closure verification (5 commits d3ea7a0b → 4bf3dfdd) of pass-2's 8 findings + new-finding sweep
**Streak status:** this pass is attempt 3 toward 3-CLEAN cascade (was 0/3)

---

## Closure Verification Table

| Pass-2 Finding | Closure Status | Evidence |
|---|---|---|
| F-LP2-CRIT-001 (CI threshold hardcoded to 8 while log claimed 14) | **REAL** | `.github/workflows/ci.yml` sets `EXPECTED=29`. Assertion `if [ "${TOTAL_COUNT}" -lt "${EXPECTED}" ]`. Per-symbol enumeration: 18 E0639 struct violations + 11 E0004 enum violations = 29. Per-file rustc cap dodged by splitting into `enum_violations.rs` + `struct_violations.rs`. |
| F-LP2-HIGH-001 (sibling-sweep missing 11+ types) | **REAL** | All claimed siblings verified via grep: write_endpoint.rs (3), infusion/mod.rs (7), types.rs (5). All annotated. |
| F-LP2-HIGH-002 (verify-workflow-structure reachability check missing) | **REAL** | Reachability grep guard added: `grep -qE 'non-exhaustive-violation-compile-fail' .github/workflows/ci.yml`. Positive-coverage log on success. |
| F-LP2-MED-001 (stale tenant.rs doc-comment) | **REAL** | Doc-comment rewritten: explicit "No production caller of `new_unchecked` remains" + History section + audit-guardrail call-out. |
| F-LP2-MED-002 (volatile pin citations decayed) | **REAL** | `grep \.rs:[0-9]+` against pipeline.rs returns ZERO matches. |
| F-LP2-OBS-001 (no local just check-non-exhaustive recipe) | **REAL** | Justfile `check-non-exhaustive` recipe invoking `scripts/check-non-exhaustive.sh` + `scripts/count-non-exhaustive-errors.py`. Scripts have shebangs, use --message-format=json to dodge rustc cap. |
| F-LP2-OBS-003 (stale TD-008 comment in pipeline.rs) | **REAL** | pipeline.rs TD-008 reference updated to reflect closure; TD-006 same. No stale "no escape mechanism" claim. |
| F-LP2-OBS-002 (types.rs duplicate type families) | **NOT-CLOSED (LOW deferral acceptable)** | Architectural deferral; not in fix-burst-2 scope. |

**Closure summary: 7 REAL, 1 deferred-LOW (acceptable). Zero paper-fixes detected.**

---

## Positional-Constructor Anti-Pattern Recurrence Check

CRIT-002 (pass-1) cited positional `::new()` as defeating `#[non_exhaustive]`. Audited every new `::new()` introduced for the 15+ types added in fix-burst-2:

| Constructor | Positional Args | Default impl Available? | Verdict |
|---|---|---|---|
| `WriteStep::new(method, url, body_template, response_path)` | 4 | YES (derived `Default`) | SAFE — `..Default::default()` available |
| `WriteEndpointSpec::new(pipe_verb, sql_table, risk_tier, capability_path, batch_limit, batch_mode, record_id_field, steps)` | 8 | YES (explicit `impl Default`) | SAFE |
| `InfusionSourceConfig::new(source_type, file_path, key_column, refresh_interval_secs)` | 4 | YES (explicit `impl Default`) | SAFE |
| `CredentialRef::new(field_name, env_var)` (infusion) | 2 | YES (derived `Default`) | SAFE |
| `InfusionField::new(name, input_field, input_type, output_type)` + `::with_all(...)` | 4 + 6 | YES (derived `Default`) | SAFE |
| `PipeStageConfig::new(adds_columns)` | 1 | YES (derived `Default`) | SAFE |
| `PluginConfig::new(plugin_path)` | 1 | YES (derived `Default`) | SAFE |
| `InfusionSpec::new(infusion_id, name, infusion_type, fields, source_path)` | 5 | YES (explicit `impl Default`) | SAFE |
| `SensorTableDescriptor::new(...)` (types.rs) | 4 | YES (explicit `impl Default`) | SAFE |
| `SensorSpec::new_hot_reload(...)` (types.rs) | 8 | YES (explicit `impl Default`) | SAFE |

**Verdict: CRIT-002 anti-pattern does NOT recur.** Every positional `::new()` is paired with a `Default` impl. The canonical external pattern remains struct-literal + `..Default::default()`, which IS forward-compatible.

---

## New Findings

### F-LP3-MED-001 — AC-5 narrative in story v1.1 still cites "8 types" in 4 places — POL-7 source-of-truth drift
- **Severity**: MEDIUM
- **Category**: spec-drift / POL-7 `[process-gap]`
- **Subject**: Story v1.1 still uses "8 types" language in AC-5 narrative even though fix-burst-1 expanded scope to 14 and fix-burst-2 expanded to 29:
  - Token Budget: `(AC-5 #[non_exhaustive] on 8 types) | ~2,000`
  - Tasks Step 1: `Apply #[non_exhaustive] to all 8 types in spec_parser.rs`
  - File Structure Requirements: `add #[non_exhaustive] to 8 types`
  - Demo Evidence: `grep #[non_exhaustive] output showing all 8 types annotated`
  - AC-5 body table STILL lists only the original 8 types, not the 21 sibling types.
- **Why MEDIUM**: Story narrative is POL-7 source-of-truth. Drift creates uncertainty about AC-5 scope.
- **Recommended fix**: Bump story v1.1 → v1.2. Replace AC-5 table to enumerate all 29 types (or reference CI EXPECTED count as authoritative).

### F-LP3-MED-002 — Sibling sweep STILL incomplete: 9 pub Deserialize types in types.rs lack `#[non_exhaustive]`
- **Severity**: MEDIUM
- **Category**: spec-compliance / sibling-sweep (TD-VSDD-060)
- **Subject**: `crates/prism-spec-engine/src/types.rs` has 9 pub Deserialize types lacking `#[non_exhaustive]`:
  - `SensorSpecEntry`, `ConfigSnapshot`, `ValidationError`, `ModeChange`, `ReloadResult`, `ReloadStatus`, `ModifiedSpec`, `AddSensorSpecResult`, `ListSensorSpecsResult`
  - Also: `AddSensorSpecArgs`, `ListSensorSpecsArgs` (MCP request DTOs, no Default, not non_exhaustive)
- **Why MEDIUM (not HIGH)**: Adjudication-dependent — these are MCP-wire result/event types, not TOML-input config types. AC-5's letter says "pub TOML-deserialized" — these qualify. Pragmatic interpretation: output/result types may legitimately be exhaustive.
- **Recommended fix**: Either (a) extend AC-5 audit to cover these 9-11 types, OR (b) explicitly DOCUMENT in story v1.2 that AC-5 audit scope is limited to TOML-config-input types and add doc-comment to each result type stating "exhaustive by design — MCP response stability governed by protocol spec." Recommendation: (b) — protocol types should be exhaustive.

### F-LP3-LOW-001 — `WriteStep::new` and `WriteEndpointSpec::new` doc-comments contradict the actual recommended external pattern
- **Severity**: LOW
- **Category**: documentation drift
- **Subject**: Doc-comments claim `::new()` IS the forward-compatible pattern, but positional `::new()` with fixed arg count is NOT forward-compatible (adding a field requires changing the arg count = breaking semver change).
- **Why LOW**: Default available, so escape hatch exists; doc just steers toward less-forward-compat pattern.
- **Recommended fix**: Update each `::new()` doc-comment: "This convenience constructor takes fixed positional arguments and is NOT forward-compatible by itself. For forward-compatible external construction across future field additions, use struct-literal + `..Default::default()`."

### F-LP3-OBS-001 — Justfile new recipe not wired into `just check` meta-recipe
- **Severity**: LOW
- **Category**: ci-as-code / dev-experience `[process-gap]`
- **Subject**: `just check` (canonical pre-push gate) does NOT include `check-non-exhaustive`. Only CI catches regressions locally.
- **Recommended fix**: Append `@scripts/check-non-exhaustive.sh` to the `check` recipe body, OR document deliberate exclusion.

### F-LP3-OBS-002 — `non-exhaustive-violation-compile-fail` CI job lacks `timeout-minutes` headroom
- **Severity**: LOW
- **Category**: ci-as-code / POL-15
- **Subject**: Job has `timeout-minutes: 12` — same class as proven-stable perimeter-compile-fail. Tight ceiling but failure mode is loud.
- **Recommended fix**: No immediate action. Monitor; bump to 20 min if approaching limit.

---

## Total Findings by Severity

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 2 | F-LP3-MED-001, F-LP3-MED-002 |
| LOW (OBS) | 3 | F-LP3-LOW-001, F-LP3-OBS-001, F-LP3-OBS-002 |
| **Total** | **5** | |

---

## Trajectory

PREREQ-C: 18 (pass-1) → 8 (pass-2) → **5 (pass-3)**.
- CRIT: 3 → 1 → **0**
- HIGH: 8 → 2 → **0**
- MED: 0 → 2 → **2** (different findings — pass-2 MEDs all closed; pass-3 MEDs newly surfaced)
- LOW/OBS: 7 → 3 → **3**

---

## 3-CLEAN Streak Status

**1/3.** Zero CRITICAL + zero HIGH findings. Streak advances 0/3 → 1/3.

MEDs are non-blocking for streak advancement, but actionable. Recommended to address before pass-4 to lock the streak.

---

## Verdict

**CLEAN — streak advances 0/3 → 1/3.**

Pass-3 detects no CRITICAL or HIGH findings. Fix-burst-2 successfully closed all 7 in-scope pass-2 findings with no paper-fix surface.

**Recommended next-step:** Optional fix-burst-3 to address F-LP3-MED-001 (story v1.1 → v1.2 narrative reconciliation to 29 types) and F-LP3-MED-002 (adjudicate AC-5 scope for 9 result/wire types). Then pass-4 expecting streak 2/3.
