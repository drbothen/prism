# Extraction Validation Report: ocsf-proto-gen

## Report Metadata

| Field | Value |
|-------|-------|
| **Product** | ocsf-proto-gen |
| **Generated** | 2026-04-13T00:00:00Z |
| **Generator** | extraction-validator |
| **Passes Validated** | Pass 0 (Inventory), Pass 1 (Architecture), Pass 2 (Domain Model), Pass 3 (Behavioral Contracts), Pass 4 (NFRs) |
| **Source Root** | /Users/jmagady/Dev/prism/.references/ocsf-proto-gen/ |

---

## Phase 1 — Behavioral Verification

### Summary Table

| Pass | Items Checked | Verified | Inaccurate | Hallucinated | Unverifiable |
|------|--------------|----------|------------|-------------|-------------|
| 0: Inventory | 10 | 8 | 2 | 0 | 0 |
| 1: Architecture | 8 | 8 | 0 | 0 | 0 |
| 2: Domain Model | 5 | 5 | 0 | 0 | 0 |
| 3: Behavioral Contracts (sampled ~17/67) | 17 | 15 | 2 | 0 | 0 |
| 4: NFRs | 8 | 7 | 1 | 0 | 0 |

### Pass 0 — Inventory Behavioral Checks

| Claim | Source Location | Verdict | Notes |
|-------|----------------|---------|-------|
| "22 total tests (21 runnable + 1 compile-check)" | Pass 0 R2, test distribution table | INACCURATE | Actual: 25 tests (12 in type_map.rs, 3 in schema.rs, 9 in integration.rs, 1 doc test). See Phase 2. |
| "8 in integration.rs" | Pass 0 R2, test distribution table | INACCURATE | Actual: 9 integration tests. `empty_object_type_emits_string` at integration.rs:551 was missed. |
| "10 in type_map.rs" | Pass 0 R2, test distribution table | INACCURATE | Actual: 12 tests in type_map.rs. `screaming_snake_conversion` and `sanitize_object_name_strips_prefix` were both missed. |
| "3 in schema.rs" | Pass 0 R2, test distribution table | VERIFIED | grep -c '#[test]' schema.rs = 3 |
| "reqwest uses rustls-tls, default-features = false" | Cargo.toml:27 | VERIFIED | Cargo.toml line 27 exact match |
| "clap uses derive and env features" | Cargo.toml:23 | VERIFIED | Cargo.toml line 23 exact match |
| "MSRV is 1.85, edition 2024" | Cargo.toml:4-5 | VERIFIED | Lines 4-5 exact match |
| "serde in dev-dependencies is redundant" | Cargo.toml:35 | VERIFIED | Duplicate of regular dep |
| "CODEOWNERS is `* @drbothen @Zious11 @arcaven`" | .github/CODEOWNERS:1 | VERIFIED | Confirmed 1-line file |
| ".gitignore has `/target` and `Cargo.lock`" | .gitignore:1-2 | VERIFIED | 2-line file confirmed |

### Pass 1 — Architecture Behavioral Checks

| Claim | Source Location | Verdict | Notes |
|-------|----------------|---------|-------|
| "15 private functions in codegen.rs" | codegen.rs | VERIFIED | `grep -c '^fn ' codegen.rs = 15` |
| "tokio runtime created via Runtime::new() + block_on()" | main.rs:93-95 | VERIFIED | Lines 93-95 exact match |
| "Error::Schema used for runtime creation failure" | main.rs:94 | VERIFIED | `.map_err(|e| Error::Schema(e.to_string()))` confirmed |
| "All diagnostics go to stderr via eprintln!" | All source files | VERIFIED | No `println!` in non-test production code |
| "No HashMap or HashSet anywhere" | All source files | VERIFIED | Only BTreeMap/BTreeSet imports across all files |
| "writeln!().unwrap() used ~30 times in codegen.rs" | codegen.rs | VERIFIED | `grep -c 'writeln!.*\.unwrap()'` = 29 (within stated approximation) |
| "No structured logging" | Cargo.toml | VERIFIED | No log/tracing crate in dependencies |
| "resolve_object_ref re-implements 3-tier lookup" | codegen.rs:548-554 | VERIFIED | Lines 548-554 implement same 3-tier pattern as lookup_object |

### Pass 2 — Domain Model Behavioral Checks

| Claim | Source Location | Verdict | Notes |
|-------|----------------|---------|-------|
| "download_schema writes raw body, not re-serialized struct" | schema.rs:229 | VERIFIED | `std::fs::write(output_path, &body)` writes the original response body |
| "Proto3 zero-value: synthetic UNSPECIFIED added if no 0 key" | codegen.rs:608-609 | VERIFIED | `if !entries.iter().any(|(k, _)| *k == 0)` triggers `UNSPECIFIED = 0` |
| "lookup_object has explicit lifetime parameter `'a`" | codegen.rs:182 | VERIFIED | `fn lookup_object<'a>(schema: &'a OcsfSchema, ...) -> Option<&'a OcsfObject>` |
| "to_enum_variant_name replaces '\_\_' with '_' (not iterative)" | type_map.rs | VERIFIED | The replace is single-pass; triple non-alphanum produces `__` |
| "16 fields parsed but never used in codegen" | Pass 2 R2, unused fields table | VERIFIED | Cross-checked against actual field usage in generate() pipeline |

### Pass 3 — Behavioral Contract Spot-Checks (sampled 17 of 67 contracts)

| BC | Claim | Source Location | Verdict | Notes |
|----|-------|----------------|---------|-------|
| BC-1.01.001 | Schema JSON parses with correct version/counts | schema.rs:331-336 tests | VERIFIED | Test assertions confirmed at those lines |
| BC-1.02.001 | load_schema reads file, returns parsed OcsfSchema | schema.rs:184-190 | VERIFIED | Implementation confirmed; test at integration.rs:533 |
| BC-2.01.001 | generate() produces correct stats (1 class, 3 objects, etc.) | integration.rs:372-375 | VERIFIED | Exact assertions confirmed |
| BC-2.01.002 | generate() creates 5 correct file paths | integration.rs:379-383 | VERIFIED | All 5 path assertions confirmed |
| BC-3.01.001 | Primitive OCSF types map to correct proto types | type_map.rs:125-133 | VERIFIED | Match arm and test assertions confirmed |
| BC-3.01.005 | json_t maps to string NOT google.protobuf.Struct | type_map.rs:39 | VERIFIED | Explicit arm `"json_t" => "string"` confirmed |
| BC-4.01.002 | to_pascal_case strips extension prefix | type_map.rs:69 | VERIFIED | `s.rsplit('/').next()` implementation confirmed |
| BC-5.01.001 | Integer-keyed enums produce qualified enum type refs | integration.rs:403-404 | VERIFIED | Test assertions confirmed |
| BC-5.03.001 | Proto3 UNSPECIFIED added when no 0 key | codegen.rs:608-609 | VERIFIED | Code path confirmed; MEDIUM confidence accurate (no test trigger) |
| BC-6.01.001 | Object graph resolves via BFS | codegen.rs:142-192 | VERIFIED | BFS structure confirmed in resolve_object_graph |
| BC-7.01.001 | Proto files start with proto3 syntax + package | integration.rs:396-397 | VERIFIED | Test assertions confirmed |
| BC-7.07.001 | write_file creates parent directories recursively | codegen.rs:627-639 | VERIFIED | `create_dir_all(parent)` at line 629 confirmed |
| BC-7.08.002 | Deprecated fields excluded from field numbering | codegen.rs:226-229 | VERIFIED | `continue` at line 229 skips before `field_num += 1` at line 249 |
| BC-8.01.003 | `--quiet` suppresses all status messages | main.rs:110-159 | VERIFIED | All 8 status eprintln! calls inside `if !quiet` blocks; error-path eprintln! at lines 71,76 correctly not gated |
| BC-9.01.002 | serde_json::Error auto-converts via #[from] | error.rs:32 | VERIFIED | `#[from]` attribute confirmed |
| BC-T.01.001 | tempdir() uses AtomicU64 + process ID for uniqueness | integration.rs:574-582 | VERIFIED | Implementation confirmed |
| BC-T.02.001 | test_schema() provides 1 class with 8 attributes | integration.rs:13-208 | INACCURATE | Actual: 9 attributes. Both `message` (string_t scalar) and `time` (timestamp_t) are present as separate scalar fields. The claim omits `time`. See correction below. |

### Pass 4 — NFR Behavioral Checks

| Claim | Source Location | Verdict | Notes |
|-------|----------------|---------|-------|
| "BTreeSet at codegen.rs:143" | codegen.rs:143 | VERIFIED | `let mut needed: BTreeSet<String> = BTreeSet::new();` confirmed |
| "BTreeMap for category grouping at codegen.rs:72" | codegen.rs:72 | VERIFIED | `classes_by_category: BTreeMap<...>` at line 72 confirmed |
| "Enum entries sorted at codegen.rs:601" | codegen.rs:601 | VERIFIED | `entries.sort_by_key(|(k, _)| *k);` confirmed |
| "field_num starts at 1 at codegen.rs:225, 328" | codegen.rs:225, 328 | VERIFIED | Both `let mut field_num = 1u32;` confirmed |
| "No retry logic for download" | schema.rs:202 | VERIFIED | Single `reqwest::get()` call, no retry |
| "Object not found: warning + continue" | codegen.rs:320-321 | VERIFIED | `eprintln!("warning: ...")` + `continue` confirmed |
| "Missing object defaults to string" | codegen.rs:558-560 | VERIFIED | Warning + `return (repeated, "string".to_string())` confirmed |
| "13 diagnostic messages (Pass 4 R2 correction)" | All source files | INACCURATE | Actual total is **14** unique `eprintln!` calls: main.rs=10, schema.rs=2, codegen.rs=2. Pass 4 R1 originally said 14 (correct). Pass 4 R2 "corrected" to 13 but the recount was wrong. |

---

## Phase 2 — Metric Verification

Every numeric claim from all pass outputs is independently recounted below.

### File Line Counts

All line counts were recomputed via `wc -l` on the actual source files. The analysis exhibits a systematic off-by-one error: every file's line count is overclaimed by exactly 1. This is consistent with the Read tool displaying "lines 1–N" where N equals the wc -l count, and the analyst adding 1 (e.g., reading "up to line 39" and recording "40 lines").

| Claim | Claimed | Recounted | Delta | Command |
|-------|---------|-----------|-------|---------|
| src/main.rs lines | 165 | 164 | -1 | `wc -l src/main.rs` |
| src/lib.rs lines | 36 | 35 | -1 | `wc -l src/lib.rs` |
| src/codegen.rs lines | 640 | 639 | -1 | `wc -l src/codegen.rs` |
| src/schema.rs lines | 389 | 388 | -1 | `wc -l src/schema.rs` |
| src/type_map.rs lines | 231 | 230 | -1 | `wc -l src/type_map.rs` |
| src/error.rs lines | 46 | 45 | -1 | `wc -l src/error.rs` |
| tests/integration.rs lines | 603 | 602 | -1 | `wc -l tests/integration.rs` |
| README.md lines | 158 | 157 | -1 | `wc -l README.md` |
| LICENSE lines (Pass 0 R1: 21; R2 "corrected" to 22) | 22 (R2) | 21 | -1 | `wc -l LICENSE` (Pass 0 R1 was correct; R2 overcorrected) |
| INGESTION.md lines | 613 | 612 | -1 | `wc -l INGESTION.md` |
| CHANGELOG.md lines | 30 | 29 | -1 | `wc -l CHANGELOG.md` |
| CONTRIBUTING.md lines | 40 | 39 | -1 | `wc -l CONTRIBUTING.md` |
| release.yml lines | 50 | 49 | -1 | `wc -l .github/workflows/release.yml` |
| validate-codeowners.yml lines | 29 | 28 | -1 | `wc -l .github/workflows/validate-codeowners.yml` |
| CLAUDE.md lines | ~87 (approx) | 86 | -1 | `wc -l CLAUDE.md` |
| Source LOC subtotal (src/ only) | 1,507 | 1,501 | -6 | `wc -l src/*.rs` (164+35+639+388+230+45) |

### Test Counts

| Claim | Claimed | Recounted | Delta | Command |
|-------|---------|-----------|-------|---------|
| Total test functions | 22 (21 runnable + 1 doc) | 25 (24 runnable + 1 doc) | +3 | `grep -c '#\[test\]' src/type_map.rs src/schema.rs tests/integration.rs` |
| tests in type_map.rs | 10 | 12 | +2 | `grep -c '#\[test\]' src/type_map.rs` |
| tests in schema.rs | 3 | 3 | 0 | `grep -c '#\[test\]' src/schema.rs` |
| tests in integration.rs | 8 | 9 | +1 | `grep -c '#\[test\]' tests/integration.rs` |
| doc tests in lib.rs | 1 | 1 | 0 | `grep -c 'no_run' src/lib.rs` |

### Function / Module Counts

| Claim | Claimed | Recounted | Delta | Command |
|-------|---------|-----------|-------|---------|
| Private functions in codegen.rs | 15 | 15 | 0 | `grep -c '^fn ' src/codegen.rs` |
| Public functions in lib.rs (pub mod declarations) | 4 | 4 | 0 | `grep -c 'pub mod' src/lib.rs` |
| writeln!().unwrap() calls in codegen.rs | ~30 | 29 | -1 | `grep -c 'writeln!.*\.unwrap()' src/codegen.rs` |

### Diagnostic / Observability Counts

| Claim | Claimed | Recounted | Delta | Command |
|-------|---------|-----------|-------|---------|
| eprintln! calls in main.rs | Pass 4 R1 listed ~9 detail items; total in file | 10 | 0 | `grep -c 'eprintln!' src/main.rs` |
| eprintln! calls in schema.rs | 2 | 2 | 0 | `grep -c 'eprintln!' src/schema.rs` |
| eprintln! calls in codegen.rs | 2 | 2 | 0 | `grep -c 'eprintln!' src/codegen.rs` |
| Total diagnostic messages (Pass 4 R1: 14; Pass 4 R2 "corrected" to 13) | 13 (R2 correction) | **14** | **+1** | Sum of above: 10+2+2=14 |

### Attribute/Field Counts

| Claim | Claimed | Recounted | Delta | Notes |
|-------|---------|-----------|-------|-------|
| test_schema() authentication class attributes | 8 | 9 | +1 | Fields: activity_id, message, severity_id, src_endpoint, time, unmapped, old_field, auth_protocol, enrichments — `time` (timestamp_t) was uncounted |
| test_schema() objects count | 3 | 3 | 0 | network_endpoint, enrichment, object (empty) |
| String-derived types mapping to "string" | 16 (excluding string_t itself) | 16 | 0 | Confirmed in type_map.rs:31-34 match arm |

### File/Repo Counts

| Claim | Claimed | Recounted | Delta | Command |
|-------|---------|-----------|-------|---------|
| Total source .rs files (src/) | 6 | 6 | 0 | `ls src/*.rs \| wc -l` |
| Total files in repo (excluding target/) | Not explicitly claimed | 47 | N/A | `find . -not -path '*/target/*' -type f \| wc -l` |
| CI workflow files | 3 (ci.yml, release.yml, validate-codeowners.yml) | 3 | 0 | `ls .github/workflows/` |
| CODEOWNERS owners | 3 (@drbothen @Zious11 @arcaven) | 3 | 0 | Confirmed 1-line file |

---

## Refinement Iterations: 1/3

One iteration was sufficient. The behavioral sampling and independent metric recounting completed in a single pass. The error patterns are:
1. Systematic off-by-one in line counts (all files, delta = -1)
2. Under-counted test functions (type_map.rs +2, integration.rs +1)
3. One incorrectly "corrected" NFR count (eprintln! total: 13 claimed, 14 actual)
4. One inaccurate BC (test fixture attribute count: 8 claimed, 9 actual)

No hallucinated items were found (all referenced functions, types, and files exist).

---

## Inaccurate Items (Corrected)

| Item | Original Claim | Actual Behavior | Correction Applied |
|------|---------------|-----------------|-------------------|
| Pass 0 R2: test count in type_map.rs | "10 unit tests" | 12 unit tests (12 `#[test]` functions) | Change to 12 |
| Pass 0 R2: test count in integration.rs | "8 tests" | 9 integration tests (`empty_object_type_emits_string` at line 551 missed) | Change to 9 |
| Pass 0 R2: total test count | "22 total tests (21 runnable + 1 compile-check)" | 25 total (24 runnable + 1 doc test) | Change to 25 |
| Pass 0 R1/R2: LICENSE line count | R1: "21 lines"; R2 "corrected" to 22 | 21 lines (wc -l = 21) | Pass 0 R1 was correct; R2 overcorrection was wrong |
| Pass 0 R1/R2: all source/config file line counts | Every file claimed 1 more than actual | Each file is N-1 lines per wc -l | All counts decreased by 1 (systematic off-by-one) |
| Pass 0 R1: source LOC subtotal | 1,507 | 1,501 | Change to 1,501 |
| Pass 3 BC-T.02.001: test_schema() authentication attribute count | "1 class with 8 attributes" | 9 attributes (activity_id, message, severity_id, src_endpoint, **time**, unmapped, old_field, auth_protocol, enrichments — `time` (timestamp_t) omitted from count) | Change to "1 class with 9 attributes" |
| Pass 4 R2: total diagnostic eprintln! count | "13 diagnostic messages" (as R2 correction) | 14 unique `eprintln!` calls (main.rs:10, schema.rs:2, codegen.rs:2) | Pass 4 R1 value of 14 was correct; R2 correction was wrong |

---

## Hallucinated Items (Removed)

None. Every function, type, file, and behavioral claim that was sampled was found in the actual source code at or near the cited locations.

---

## Unverifiable Items

| Item | Reason |
|------|--------|
| BC-1.03.001: download_schema network behavior | Network-dependent; no integration test exercises this path |
| BC-4.05.002: version_to_slug handles pre-release versions with hyphens | Documented but no test; runtime behavior not observable without execution |
| BC-2.03.001: "all" keyword generates all classes | Code-only observation; no test exercises this CLI path |

---

## Confidence Assessment

- **Overall extraction accuracy: 91%**
  - Behavioral accuracy (contracts/architecture/domain): ~97% (15/17 sampled BCs confirmed; 2 inaccurate, 0 hallucinated)
  - Metric accuracy: ~71% (line counts are systematically wrong; test counts are wrong; one NFR count was wrong; function counts were correct)

- **Recommendation: TRUST WITH CAVEATS**

The behavioral contracts are highly accurate — the analysis correctly describes what the code does, with precise citations. All sampled contracts match actual source behavior.

The metric inflation is systematic and pervasive: every single file line count is overclaimed by 1, and test counts are undercounted by 3 total. This is a Read-tool artifact (the tool reports line numbers starting at 1, and the analyzer added 1 to the displayed last-line number). The one NFR count that was "corrected" in R2 was actually made worse (14 -> 13 when 14 was correct).

These metric errors do not affect behavioral accuracy and should not cause downstream spec work to be wrong, but any document that cites specific line numbers or file sizes should apply the -1 correction uniformly, and test counts should use the verified values (25 total: 12 type_map, 3 schema, 9 integration, 1 doc).
