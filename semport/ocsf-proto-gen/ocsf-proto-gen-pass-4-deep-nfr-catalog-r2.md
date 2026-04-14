# Pass 4 Deep Dive Round 2: NFR Catalog -- ocsf-proto-gen

## Objective

Hallucination audit of Round 1 NFR claims. Verify code locations and behavioral assertions. Cross-reference with other passes.

---

## Hallucination Audit

### NFR-1: Determinism Claims

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "BTreeMap at schema.rs:8" | VERIFIED | `use std::collections::BTreeMap;` at line 8 |
| "BTreeSet at codegen.rs:143" | VERIFIED | `let mut needed: BTreeSet<String> = BTreeSet::new();` at line 143 |
| "BTreeMap for category grouping at codegen.rs:72" | VERIFIED | `let mut classes_by_category: BTreeMap<String, Vec<&OcsfClass>> = BTreeMap::new();` at line 72 |
| "Enum entries sorted at codegen.rs:601" | VERIFIED | `entries.sort_by_key(|(k, _)| *k);` at line 601 |
| "field_num starts at 1 at codegen.rs:225, 328" | VERIFIED | Line 225: `let mut field_num = 1u32;` and line 328: `let mut field_num = 1u32;` |
| "No HashMap/HashSet anywhere" | VERIFIED | Cross-checked with Pass 1 R2 audit |
| "deterministic_output test at integration.rs:500-516" | VERIFIED | Lines 499-516 (test starts at line 499 with `#[test]`, function at 500) |

### NFR-3: Security Claims

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "rustls-tls feature on reqwest" | VERIFIED | Cargo.toml:27 |
| "URL overridable via env var" | VERIFIED | main.rs:36: `env = "OCSF_SCHEMA_URL"` |
| "Path traversal via version string" | VERIFIED RISK | main.rs:92, 109: `output_dir.join(&ocsf_version)` -- user-supplied version string used in path. Risk is real but mitigated by CLI context. |
| "No input size limits on download" | VERIFIED | schema.rs:213-216: `response.text().await` with no size check |
| "Schema validated before writing" | VERIFIED | schema.rs:219: `serde_json::from_str::<OcsfSchema>(&body)` before write at line 229 |

### NFR-4: Reliability Claims

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "No retry logic for download" | VERIFIED | schema.rs:198-242: single `reqwest::get()` call, no retry loop |
| "No transaction/rollback on partial failure" | VERIFIED | codegen.rs `generate()` writes files incrementally; no cleanup on error |
| "create_dir_all is idempotent" | VERIFIED | std::fs::create_dir_all is documented as idempotent in Rust std |
| "Object not found produces warning, continues" | VERIFIED | codegen.rs:320-321: `eprintln!("warning: ..."); continue;` |
| "Missing object defaults to string" | VERIFIED | codegen.rs:558-560: `eprintln!("warning: ..."); ... return (repeated, "string".to_string());` |

### NFR-5: Observability Claims

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "14 diagnostic messages cataloged" | RECOUNTED | Counting all `eprintln!` calls in non-test code: main.rs has 9 (lines 71, 76, 111, 115-120, 130, 136-139, 140-143, 144-148, 149-154, 158), schema.rs has 2 (lines 200, 234-240), codegen.rs has 2 (lines 320, 558). Total: 13, not 14. **Correction: 13 diagnostic messages.** The "Done." message (main.rs:158) was counted but it is inside the `if !quiet` block that also contains the stats print, so it is a single `eprintln!` call. Recounting carefully: 13 unique `eprintln!` calls. |
| "download subcommand has no quiet flag" | VERIFIED | main.rs:22-38: no `quiet` field in `DownloadSchema` |
| "codegen warnings not suppressible" | VERIFIED | codegen.rs:320, 558 -- no quiet parameter |

### NFR-6: Maintainability Claims

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "All 5 CI jobs use rust-cache" | PARTIALLY CORRECT | check, clippy, test, doc use `Swatinem/rust-cache@v2`. fmt does NOT (it only needs rustfmt, no compile). Round 1 correctly noted "except fmt which has no compile step". |
| "All public functions have doc comments" | VERIFIED | `generate()`, `load_schema()`, `download_schema()`, all 5 type_map functions -- all have `///` comments |
| "No unsafe code" | VERIFIED | No `unsafe` keyword in any source file |
| "RUSTDOCFLAGS: -D warnings in doc job" | VERIFIED | ci.yml:65-66 |

### NFR-7: Compatibility Claims

| Claim | Verdict | Evidence |
|-------|---------|---------|
| "deny_unknown_fields is NOT set" | VERIFIED | No `#[serde(deny_unknown_fields)]` on any struct |
| "No required keyword in output" | VERIFIED | No `required` string appears in any `writeln!` in codegen.rs |
| "No optional keyword in output" | VERIFIED | No `optional` keyword in any proto generation code |
| "MSRV 1.85" | VERIFIED | Cargo.toml:5 |

---

## Cross-Reference with Pass 3

Pass 3 BC-7.03.002 asserts "No google.protobuf.Struct references in output" -- this aligns with NFR-7 compatibility (proto3 compliance without well-known type dependencies).

Pass 3 BC-7.04.001 (deterministic output) is the behavioral test for NFR-1.

Pass 3 gaps #9 (network download untested) aligns with NFR-4 noting no retry logic and NFR-3 noting no input size limits.

---

## Corrections to Round 1

1. **Diagnostic message count**: 13, not 14.
2. **CI cache detail**: fmt job does NOT use rust-cache (Round 1 stated this correctly but the phrasing "All 5 CI jobs use rust-cache" in the table was misleading -- the exception was noted in parenthetical).

---

## Delta Summary
- New items added: 0
- Existing items refined: Diagnostic message count corrected (13 not 14), CI cache phrasing clarified
- Remaining gaps: None

## Novelty Assessment
Novelty: NITPICK
Round 2 is entirely verification. One minor count correction (13 vs 14 diagnostic messages). No new NFRs discovered. Cross-references with Pass 3 confirm consistency. Removing these findings would not change how you would spec the system.

## Convergence Declaration
Pass 4 has converged -- findings are nitpicks, not gaps. The NFR catalog is complete and verified.

## State Checkpoint
```yaml
pass: 4
round: 2
status: complete
timestamp: 2026-04-13T23:40:00Z
novelty: NITPICK
```
