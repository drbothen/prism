# Coverage Audit: ocsf-proto-gen

## Method

Grep-driven coverage audit of all 19 source files against all 15 prior analysis files. Every file in the repo was read; every analysis file was read. References were counted via ripgrep pattern matching of filenames across the analysis corpus.

---

## File Inventory (19 files)

| # | File | Category | Lines |
|---|------|----------|-------|
| 1 | `src/main.rs` | Source (entry) | 165 |
| 2 | `src/lib.rs` | Source (entry) | 36 |
| 3 | `src/codegen.rs` | Source (core) | 640 |
| 4 | `src/schema.rs` | Source (core) | 389 |
| 5 | `src/type_map.rs` | Source (core) | 231 |
| 6 | `src/error.rs` | Source (support) | 46 |
| 7 | `tests/integration.rs` | Test | 603 |
| 8 | `Cargo.toml` | Config | 37 |
| 9 | `.github/workflows/ci.yml` | CI | 67 |
| 10 | `.github/workflows/release.yml` | CI | 50 |
| 11 | `.github/workflows/validate-codeowners.yml` | CI | 29 |
| 12 | `.github/CODEOWNERS` | Config | 1 |
| 13 | `.gitignore` | Config | 2 |
| 14 | `README.md` | Docs | 158 |
| 15 | `CLAUDE.md` | Docs | ~87 |
| 16 | `CHANGELOG.md` | Docs | 30 |
| 17 | `CONTRIBUTING.md` | Docs | 40 |
| 18 | `INGESTION.md` | Docs | 613 |
| 19 | `LICENSE` | Legal | 22 |

---

## Coverage Matrix

Legend:
- **Y** = Covered (file content analyzed, key behaviors extracted)
- **P** = Partial (file mentioned but content not deeply analyzed)
- **N** = Not covered (file not referenced or only listed by name)
- Pass columns: 0=Inventory, 1=Architecture, 2=Domain Model, 3=Behavioral Contracts, 4=NFRs, 5=Conventions

| File | Pass 0 | Pass 1 | Pass 2 | Pass 3 | Pass 4 | Pass 5 | Refs | Verdict |
|------|--------|--------|--------|--------|--------|--------|------|---------|
| `src/main.rs` | Y | Y | Y | Y | Y | Y | 58 | FULL |
| `src/lib.rs` | Y | Y | Y | P | P | Y | 25 | FULL |
| `src/codegen.rs` | Y | Y | Y | Y | Y | Y | 87 | FULL |
| `src/schema.rs` | Y | Y | Y | Y | Y | Y | 47 | FULL |
| `src/type_map.rs` | Y | Y | Y | Y | P | Y | 33 | FULL |
| `src/error.rs` | Y | Y | Y | Y | P | Y | 15 | FULL |
| `tests/integration.rs` | Y | P | Y | Y | P | Y | 49 | FULL |
| `Cargo.toml` | Y | Y | P | P | Y | P | 22 | FULL |
| `.github/workflows/ci.yml` | Y | P | N | N | Y | P | 4 | ADEQUATE |
| `.github/workflows/release.yml` | Y | P | N | N | P | P | 7 | ADEQUATE |
| `.github/workflows/validate-codeowners.yml` | Y | N | N | N | N | N | 7 | ADEQUATE |
| `.github/CODEOWNERS` | Y | N | N | N | N | N | 10 | ADEQUATE |
| `.gitignore` | Y | N | N | N | N | N | 4 | ADEQUATE |
| `README.md` | Y | N | N | N | P | N | 4 | ADEQUATE |
| `CLAUDE.md` | Y | N | N | N | N | Y | 10 | ADEQUATE |
| `CHANGELOG.md` | Y | N | N | N | N | P | 3 | ADEQUATE |
| `CONTRIBUTING.md` | Y | N | N | N | N | P | 6 | ADEQUATE |
| `INGESTION.md` | Y | N | N | N | N | N | 4 | ADEQUATE |
| `LICENSE` | Y | N | N | N | N | P | 9 | ADEQUATE |

---

## Coverage Verdicts Explained

### FULL Coverage (7 files: all source + tests)

All 6 source files and the integration test file have deep coverage across multiple passes:

- **`src/codegen.rs`** (87 refs): The most-referenced file. Every function (15 private + 1 public) cataloged in Pass 2. All generation behaviors captured as behavioral contracts (Pass 3). Output structure, determinism, enum handling, object resolution all deeply analyzed. The DFS-not-BFS documentation bug was discovered in Pass 5 R2.

- **`src/schema.rs`** (47 refs): All 6 struct types fully documented with serde annotations (Pass 2). Load and download paths traced (Pass 2 R2). 7 behavioral contracts extracted (Pass 3). Tolerant parsing pattern documented (NFR-7).

- **`src/main.rs`** (58 refs): CLI types (`Cli`, `Commands`) documented in Pass 2. Async/sync boundary analyzed in Pass 1. Quiet flag behavior, error chain printing, class name parsing all captured as contracts (Pass 3). Env var support documented.

- **`src/type_map.rs`** (33 refs): Complete type mapping table in broad sweep and verified in Pass 3 with 7 HIGH-confidence contracts. All 5 public functions cataloged with test evidence. Edge cases documented (triple non-alphanum, extension prefix).

- **`src/error.rs`** (15 refs): All 7 variants documented with triggers, format strings, and traits (Pass 2). Feature-gated `Download` variant analyzed. Auto-conversion via `#[from]` documented.

- **`src/lib.rs`** (25 refs): Correctly identified as pure re-export hub with no logic (Pass 1). Module visibility analysis confirmed no functions or types defined.

- **`tests/integration.rs`** (49 refs): All 8 tests used as primary evidence for behavioral contracts. Test helpers (`test_schema`, `default_attr`, `tempdir`, `walkdir`) documented in Pass 2 and Pass 3 R2. Test schema fixture fully characterized (1 class, 3 objects, 8 attribute types exercised).

### ADEQUATE Coverage (12 files: config, CI, docs)

These files are non-behavioral (configuration, documentation, governance). They are inventoried in Pass 0 with line counts and purposes, and referenced where relevant in other passes. They do not need deep behavioral analysis because they contain no executable logic.

- **CI files** (ci.yml, release.yml, validate-codeowners.yml): Job structure, triggers, timeout values, and tool versions documented in Pass 0 R1 and verified in R2. CI enforcement model documented in NFR-6. Release pipeline documented in Pass 5.

- **Config files** (Cargo.toml, .gitignore, CODEOWNERS): Cargo.toml extensively analyzed for dependency versions, features, dual target configuration. CODEOWNERS and .gitignore documented in Pass 0.

- **Documentation files** (README.md, CLAUDE.md, CHANGELOG.md, CONTRIBUTING.md, INGESTION.md, LICENSE): All inventoried in Pass 0. README's ocsf-tool comparison table and type mapping table noted. CHANGELOG gap (missing v0.1.2) identified. CLAUDE.md documented in Pass 5 as a convention. LICENSE verified as MIT, copyright 1898 & Co.

---

## Blind Spot Analysis

### Potential Gaps Investigated

1. **README.md ocsf-tool comparison table**: The README documents 6 specific issues with the community `ocsf-tool` that this project fixes. The broad sweep mentions ocsf-proto-gen replaces ocsf-tool (1 reference) but does not reproduce the specific comparison table. **Assessment**: The comparison is marketing/context material, not behavioral specification. The specific fixes (deprecated fields, json_t handling, string-keyed enums, enum type refs, determinism, version pinning) are ALL independently captured in behavioral contracts BC-7.03.001, BC-3.01.005, BC-5.02.001, BC-5.01.001, BC-7.04.001, and BC-8.01.001. **No gap.**

2. **README library usage example**: Shows `default-features = false` usage for library consumers without network deps. This is captured in Pass 1 R1 (feature-gated compilation model) and the Prism implications note. **No gap.**

3. **CHANGELOG v0.1.1 details**: Lists 9 specific changes. These are all represented in the analysis (type mapping additions, test isolation fix, integration tests, CLAUDE.md, release workflow). **No gap.**

4. **CONTRIBUTING.md code standards**: 4 rules documented. All captured in Pass 5 (doc comments, no panic/unwrap exception, thiserror, test unwrap). **No gap.**

5. **INGESTION.md**: This file is a copy of the broad sweep analysis itself (613 lines matching the broad sweep content). It is an artifact of the analysis being committed to the repo. **Not a content source -- skip.**

6. **validate-codeowners.yml fork handling**: The workflow has conditional logic for fork PRs (line 19-20: `if github.event.pull_request.head.repo.full_name == github.repository` for full check vs syntax-only for forks). This is documented in Pass 0 R1 but not analyzed further. **Assessment**: This is CI governance detail, not behavioral specification. **No gap for semport purposes.**

7. **README protoc verification command**: Shows how to validate output with protoc. NFR-10 (Pass 4) notes that generated protos are not validated via protoc. **Captured.**

### Substantive Blind Spots Found

**None.** After exhaustive file-by-file grep-driven audit and content comparison:

- All 6 source files have function-level coverage across domain model, behavioral contracts, and conventions
- All 603 lines of integration tests are used as evidence in Pass 3
- All configuration and CI files are inventoried with relevant details extracted
- All documentation files are inventoried and cross-referenced where they contain technical content

---

## Cross-Pass Consistency Checks

| Check | Result |
|-------|--------|
| Do Pass 3 behavioral contracts reference the correct line numbers? | VERIFIED in Pass 3 R2 and Pass 5 R2 (hallucination audits) |
| Does Pass 2 domain model match actual struct definitions? | VERIFIED -- all serde annotations, field types, and derives confirmed |
| Does Pass 1 architecture match actual module dependencies? | VERIFIED -- corrected in R1 (main.rs direct module access, lib.rs is gateway) |
| Does Pass 0 inventory match the actual file tree? | VERIFIED -- 7 missing files added in R1, line counts corrected in R2 |
| Does Pass 4 NFR evidence match actual code locations? | VERIFIED in R2 hallucination audit |
| Does Pass 5 convention catalog match actual patterns? | VERIFIED in R2 -- BFS/DFS terminology corrected, map_err count corrected |

---

## Quantitative Summary

| Metric | Value |
|--------|-------|
| Total source files | 19 |
| Files with FULL coverage | 7 (all executable code + tests) |
| Files with ADEQUATE coverage | 12 (config, CI, docs) |
| Files with NO coverage | 0 |
| Behavioral contracts extracted | 67 |
| HIGH confidence contracts | 42 (63%) |
| MEDIUM confidence contracts | 22 (33%) |
| LOW confidence contracts | 3 (4%) |
| Analysis references to source files | 478 total |
| Untested behavioral gaps identified | 11 (all edge cases, not core paths) |
| Corrections made across all rounds | 8 (line counts, DFS terminology, map_err count, test naming, diagnostic count, LICENSE lines, CHANGELOG gap, lib.rs characterization) |

---

## Verdict

**PASS.** The coverage audit finds no substantive blind spots. All 19 files in the repository are accounted for. All 7 executable code files have deep multi-pass coverage. The 12 non-executable files (config, CI, docs) are appropriately inventoried without unnecessary deep analysis. The 11 identified untested behavioral gaps are all edge cases (depth-2+ transitive object graph, extension-prefixed object lookup, tier-3 linear scan, >10 classes truncation, UNSPECIFIED zero-value, missing object warning, multiple categories, "all" keyword, network download, variant name triple non-alphanum, object proto enum import) and do not represent missing core functionality analysis.

The analysis corpus is complete and ready for downstream spec crystallization.
