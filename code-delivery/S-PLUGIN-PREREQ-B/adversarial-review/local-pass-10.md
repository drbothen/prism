---
pass: 10
story: S-PLUGIN-PREREQ-B
head_sha: f5746553
base_sha: 90d7c80f
factory_sha_at_pass: 465af091
verdict: BLOCKED-soft
streak_target: 1/3
streak_actual: 0/3
findings_summary: 0C / 0H / 2M / 2L / 3O
adversary_run_date: 2026-05-11
novelty_score: 5/5 = 1.0
---

# Adversarial Review — LOCAL Pass 10 (S-PLUGIN-PREREQ-B)

## Executive Summary

Fix-burst-9 closures are functionally CORRECT and the two new tests are load-bearing per TD-VSDD-059. All four closures (F-LP9-MED-001 BC↔impl drift, F-LP9-MED-002 multi-array warn, F-LP9-LOW-001 dead-mock deletion, OBS-LP9-003 UTF-8 boundary) are CLEAN.

However, pass-10 surfaces TWO new MEDIUM defects that block streak advancement:

1. **F-LP10-MED-001 — TD-VSDD-060 sibling-sweep gap.** Fix-burst-9's sibling sweep for OBS-LP9-003 was scoped to pipeline.rs only and missed `validation.rs:126` — `&spec.base_url[..spec.base_url.len().min(200)]` is the SAME byte-slice anti-pattern in a sibling file. Multi-byte UTF-8 base_url panics validator.

2. **F-LP10-MED-002 — find_fan_out_array Object-stringification silent corruption.** Parallel to F-LP8-LOW-001 for Object values. Object-typed step variables get JSON-stringified into URLs silently. Validator Category 2b and runtime warn BOTH only consider arrays.

Streak does NOT advance. Pass-10 is **0/3** (unchanged).

## Part A — Closure Verification of fix-burst-9

| Finding | Closure status | Load-bearing? | Notes |
|---------|----------------|---------------|-------|
| F-LP9-MED-001 (BC v1.6→v1.7) | CLEAN | YES | BC line 71 enumerates three events; matches impl at pipeline.rs:143-174 and :466-500. Test from fix-burst-8 covers all branches. Changelog v1.7 dated 2026-05-11. POL-7 H1 unchanged. POL-8 frontmatter unchanged. |
| F-LP9-MED-002 (fanout_ambiguous_multi_array warn) | CLEAN | YES | pipeline.rs:959-1013 collects all array vars with seen_keys dedup, warns when >=2. Test at pipeline_http_integration.rs:2535. Pre-fix code would FAIL assertion. Combined coverage: validator + runtime covers array surface fully. (Object surface gap → F-LP10-MED-002.) |
| F-LP9-LOW-001 (dead Mock 1 deletion) | CLEAN | YES | Mock 1 removed. Test still passes. Sibling sweep on test files clean. |
| OBS-LP9-003 (UTF-8 boundary fix) | CLEAN | YES | pipeline.rs:894-900 uses char_indices().nth(100). Test exercises 30 emoji cursor. Pre-fix panics. Sibling sweep claim **INCOMPLETE** — see F-LP10-MED-001. |

**Result:** All four fix-burst-9 closures LOAD-BEARING and CORRECT. The fix-burst-9 sibling-sweep claim for OBS-LP9-003 is INCOMPLETE; surfaced as F-LP10-MED-001.

## Part B — New Dimension Findings

### F-LP10-MED-001 — Byte-slice anti-pattern in validation.rs:126 (TD-VSDD-060 sibling-sweep gap)

- Severity: MEDIUM | Confidence: HIGH | Dimension: Sibling-sweep gap on OBS-LP9-003
- Evidence: validation.rs:126 — `&spec.base_url[..spec.base_url.len().min(200)]` — byte-index slicing on user-controlled base_url (TOML-parsed).
- Panic vector: TOML `base_url = "x🎯🎯🎯..."` with 51 emoji = 205 bytes; byte 200 falls mid-codepoint → panic when constructing error message after starts_with("http://") check fails.
- The starts_with check at line 120 rejects this path → error message construction at line 123-127 PANICS instead of returning ValidationError. The validator's BC contract (BC-2.16.009 line 12-14) says "no panic; collect all errors" — violated.
- Fix-burst-9 sibling-sweep claim (fix-burst-9.md:56): "grep '\[\.\..*\]' pipeline.rs ... Zero other byte-slice operations on user-controlled strings" — sweep was scoped to pipeline.rs ONLY. TD-VSDD-060 mandate is "same-type files in the same architectural layer" — validation.rs is clearly in scope.
- **Recommendation:** Replace with char-boundary-safe `s.chars().take(200).collect::<String>()` or `char_indices().nth(200)` pattern. Red Gate test with non-ASCII base_url failing starts_with check. Sibling sweep MUST extend to spec_parser.rs, validation.rs, interpolation.rs, infusion/*, types.rs.

### F-LP10-MED-002 — find_fan_out_array Object-stringification silent corruption (P10-J)

- Severity: MEDIUM | Confidence: HIGH | Dimension: P10-J (silent data corruption parallel to F-LP8-LOW-001)
- Evidence:
  - pipeline.rs:979 — `step_vars.get(&key).filter(|v| v.is_array())` ONLY catches arrays
  - interpolation.rs:217 — `other.to_string()` stringifies Value::Object as JSON literal
  - validation.rs:259-265 Category 2b heuristic ALSO arrays-only
- Concrete corruption: step1 `response_path: "$.metadata"`, response `{"metadata":{"host_id":"abc","region":"us-east"}}` → step_vars["step1.metadata"] = Object. step2 `path_template: "/api/devices/${step1.metadata}/lookup"`. Interpolator stringifies → URL `/api/devices/%7B%22host_id%22%3A%22abc%22%2C%22region%22%3A%22us-east%22%7D/lookup`. No warn. No validator rejection. Pipeline silently issues malformed request.
- Survives 10 passes because F-LP8-LOW-001 + F-LP9-MED-002 both scoped to arrays only. The cursor terminal-type handler DOES enumerate Object/Array/Bool as unsupported (good!) but the fan-out source classifier does not. **Inconsistent discipline within the same file.**
- Real-world hit probability HIGH: most REST APIs return nested objects under `$.data` / `$.metadata`. Spec author error (`${step1.metadata}` instead of `${step1.metadata.host_id}`) gets no validation error and no warn — only opaque upstream 400/404.
- **Recommendation:** find_fan_out_array detects Value::Object → emit `tracing::warn!` with `event_type = "fanout_invalid_source_type"`. OR interpolation.rs::value_to_string when context is UrlPath emits warn for Object/Array. Add Red Gate test exercising object-stringification.

### F-LP10-LOW-001 — Inconsistent #[non_exhaustive] discipline across pub types

- Severity: LOW | Confidence: HIGH | Dimension: P10-C (public API surface stability)
- Evidence:
  - HAVE `#[non_exhaustive]`: pipeline.rs:40 FetchContext, pipeline.rs:69 PipelineResult, error.rs:13 SpecEngineError
  - LACK `#[non_exhaustive]`: spec_parser.rs CredentialRef:204, SensorSpec:212, SensorTableDescriptor:242, FetchStep:58, ColumnSpec:81, TableSpec:101, PaginationConfig:38, AuthType:24
- The fix-burst-9 worktree DID add `#[non_exhaustive]` to FetchContext/PipelineResult but missed spec_parser.rs's TOML-deserialized types. Discipline drift: ad-hoc rather than crate-wide.
- PREREQ-C plans to extend SensorSpec/FetchStep. Every field addition is a SemVer break for external struct-literal constructors.
- **Recommendation:** File TD for PREREQ-C scope. Add `#[non_exhaustive]` to all public TOML-deserialized types in spec_parser.rs and types.rs. Severity LOW (no SemVer break shipped yet at crate version 0.5.0).

### F-LP10-LOW-002 — MockAuthProvider.token is pub-mut field

- Severity: LOW | Confidence: HIGH | Dimension: P10-B (test-helper hygiene)
- Evidence: auth_provider.rs:148 — `pub token: String`.
- External test crates with `features = ["test-helpers"]` can do `let mut p = MockAuthProvider::new("t"); p.token = "different".to_string();`. No obvious test scenario where this is necessary. Creates non-obvious shared-state behavior if Arc-wrapped and mutated from another thread.
- call_count is AtomicU32 — author KNEW about interior mutability. Choice to leave .token plain-pub looks oversight, not design.
- **Recommendation:** Make `MockAuthProvider.token` private; expose via `pub fn token(&self) -> &str`. If mid-test mutability IS intended, wrap in `Mutex<String>` + `pub fn set_token()`. Tag (pending intent verification). Severity LOW because test-helpers is feature-gated.

## Observations (non-blocking)

### OBS-LP10-001 — Category 2b validator error doesn't cite offending variable names

- Dimension: P10-A
- validation.rs:294-302 cites step names but NOT the `${...}` references that fired the heuristic. Spec author must manually re-read TOML to identify culprits.
- **Recommendation:** Extend message to enumerate offending references. Non-blocking; defer.

### OBS-LP10-002 — Workspace dependency version bounds

- Dimension: P10-K
- All bounds reasonable. tokio dev-deps vs dep features differ (sync missing in dev) — could dedupe.
- wasmtime 44 advisory currency through 2026-04-15; verify as of 2026-05-11.
- **Recommendation:** Defer to TD-S-PLUGIN-PREREQ-B-005 (existing unused-deps TD; expand scope to cargo audit).

### OBS-LP10-003 — `tokio::time::timeout` cancellation behavior is unspecified for execute()

- Dimension: P10-G
- BC-2.16.002 Error Conditions table doesn't enumerate caller-imposed timeout (tokio::time::timeout dropping the future). Technically correct semantic (Err(Elapsed) → all records discarded → matches partial-record-discard postcondition) but text doesn't say so.
- **Recommendation:** Amend BC-2.16.002 v1.8 under PREREQ-D scope to enumerate caller-timeout as an error condition.

## Findings Table

| ID | Severity | Dimension | Category | Summary | Actionable |
|----|----------|-----------|----------|---------|-----------|
| F-LP10-MED-001 | MEDIUM | sibling-sweep gap | UTF-8 panic | validation.rs:126 byte-slice on user-controlled base_url | YES |
| F-LP10-MED-002 | MEDIUM | P10-J | silent data corruption | Object-stringification in fan-out source | YES |
| F-LP10-LOW-001 | LOW | P10-C | API surface | Inconsistent #[non_exhaustive] | YES (defer to PREREQ-C) |
| F-LP10-LOW-002 | LOW | P10-B | test-helper hygiene | MockAuthProvider.token pub-mut field | YES (bundle into fix-burst-10) |
| OBS-LP10-001 | OBS | P10-A | error actionability | Category 2b error omits variable names | NO |
| OBS-LP10-002 | OBS | P10-K | dependency hygiene | All bounds reasonable; defer cargo audit | NO |
| OBS-LP10-003 | OBS | P10-G | BC text | BC doesn't enumerate caller-timeout | NO (defer to PREREQ-D) |

## Process-Gap Findings

None this pass. F-LP10-MED-001 is a content defect from under-scoped sibling sweep. If recurs, promote to [process-gap] for codification: "fix-burst reports MUST enumerate the list of sibling files swept and the grep/search patterns used."

## Recommendations

### Fix-burst-10 scope (REQUIRED)

1. **F-LP10-MED-001:** Fix validation.rs:126 byte-slice. Add Red Gate test (non-ASCII base_url failing starts_with). Sibling sweep ALL byte-slice operations on user-controlled strings in: spec_parser.rs, validation.rs, interpolation.rs, infusion/*, types.rs.

2. **F-LP10-MED-002:** Add Object/Array detection in find_fan_out_array OR interpolation.rs::value_to_string. Emit `tracing::warn!` `event_type = "fanout_invalid_source_type"`. Add Red Gate test.

3. **F-LP10-LOW-002 (pending intent verification):** Convert MockAuthProvider.token to private + getter, OR document mid-test mutability intent.

### Deferred

- F-LP10-LOW-001: file TD-S-PLUGIN-PREREQ-B-016 for #[non_exhaustive] crate-wide discipline → PREREQ-C scope.
- OBS-LP10-001..003: defer per recommendations above.

After fix-burst-10, dispatch pass-11. Target: streak 0/3 → 1/3.

## Novelty Self-Check

All findings on dimensions never previously examined. 5/5 = 1.0 novelty.

**Verdict: BLOCKED-soft. Streak 0/3. Fix-burst-10 required.**
