---
cycle: wave-4-operations
artifact_type: workspace_audit
date: 2026-05-08
audit_scope: 8-dimensional full workspace
audit_mode: read-only
findings_filed: false  # nothing filed; user-triaged
input-hash: null  # audit, not derived from spec inputs
---

# Workspace 8-Dimensional Audit — 2026-05-08

## Executive Summary

- **Total findings:** P0=18, P1=23, P2=12 → **53 total**
- **Headline pattern instances:** the prior plugin-system audit's "schema built, plugins not built" pattern recurs across **at least three additional subsystems** beyond plugin/spec-engine:
  1. **Query engine (S-3.02)** — `QueryEngine::execute`, `run_materialization_pipeline`, `resolve_source_refs`, `RocksDbTableProvider::*`, `register_internal_tables` are all `todo!()` despite STORY-INDEX claiming MERGED PR #129 (story-file frontmatter still says `status: ready`).
  2. **Write execution (S-3.07)** — STORY-INDEX claims MERGED PR #135 (2026-05-08), but: (a) `WriteExecutor::execute` Phase 3 fetch is a hardcoded empty `vec![]` (write_pipeline.rs:349) — production never fetches records to write; (b) every concrete `SensorAdapter` (CrowdStrike, Cyberint, Claroty, Armis) uses the default `SensorAdapter::write()` body which returns `WriteNotImplemented` (adapter.rs:365) — no concrete write override exists; (c) `WriteCapableTableProvider::insert_into/update/delete_from` all return `DataFusionError::NotImplemented` deferred to W3-FIX-S307-003 (write_table_registration.rs:176/190/205). Story file frontmatter still says `status: draft`.
  3. **MCP runtime (SS-10)** — `prism-mcp/src/lib.rs` is still a 10-line stub with `safety_envelope` + `tool_registry` modules only. No MCP server, no tool routing, no boot binary. Architecture claims 35 tools (module-decomposition.md:513); zero tools are wired.
- **Worst-affected dimensions:** Dim 1 (production stub residue), Dim 2 (story-vs-impl drift), Dim 7 (VP proof status — 143/145 still `draft`).
- **Verdict:** The workspace exhibits the same anti-pattern flagged by the plugin audit at a **wider scope than originally observed**. Roughly half of "merged" Wave-1/Wave-3 stories ship structural-only or wired-incompletely code; the STORY-INDEX is an unreliable status oracle (multiple stories indexed as MERGED have `status: ready` or `status: draft` in their own frontmatter — STORY-INDEX is being edited as a manual log post-hoc). No single binary exists that loads the sensor TOMLs, instantiates the spec engine, or constructs the MCP server — a runtime gap larger than the per-story stubs.

## Methodology

### Tools

- `rg --type rust 'todo!\('` and `rg --type rust 'unimplemented!\('` across `crates/*/src/`, excluding `tests/`, `proofs/`, `#[cfg(test)]` modules.
- `rg --type rust 'panic!\('` filtered for stub-like literal messages (`stub`, `todo`, `not yet`, `pending`).
- `rg 'fn write\b'` to verify which `SensorAdapter` impls override the default body.
- `rg 'parse_spec_directory|SpecLoader|ConfigManager::new|InfusionRegistry::new|register_internal_tables'` to find production callers of spec-engine entry points.
- Frontmatter sweep across `.factory/stories/*.md` for `status:` field; cross-checked against STORY-INDEX rows for drift.
- TOML inventory via `find . -maxdepth 4 -name '*.toml' -not -path '*/target/*'`.
- Spot-check of ~12 BC files, all 19 ADR files in registry, 5 randomly-sampled VP proof files, prism-mcp/lib.rs, all 4 sensors/*.sensor.toml.

### Sampling

- **Dim 1 (stub residue):** exhaustive — every `crates/*/src/` directory grep'd.
- **Dim 2 (story-impl drift):** all 36 Wave-1 stories (S-1.01–S-1.15 + S-2.01–S-2.08 + S-3.01–S-3.07 + S-3.0.0x); plus targeted spot checks on Wave-3 series (S-3.1.x, S-3.2.x, S-3.3.x, S-6.06–S-6.20).
- **Dim 3 (silent-shallow tests):** grep for `#[ignore]`, `#[should_panic]`, plus inspection of `crates/prism-spec-engine/tests/hot_reload_tests.rs:1087` and `crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs`.
- **Dim 4 (TOML/config orphans):** the 4 `sensors/*.sensor.toml`, the 3 `crates/prism-spec-engine/fixtures/*.infusion.toml`, the 2 `crates/prism-dtu-demo-server/configs/*.toml`. Reverse-direction grep for `parse_spec_directory` callers outside spec-engine.
- **Dim 5 (BC postcondition gaps):** stratified sample (~25 BCs across SS-01, SS-03, SS-04, SS-05, SS-10, SS-11, SS-16, SS-17, SS-19) — 100% are status:draft, no BC has been promoted to active.
- **Dim 6 (ADR status):** all 19 ADRs in `.factory/specs/architecture/decisions/` cross-checked against ARCH-INDEX status column.
- **Dim 7 (VP proof status):** all 145 VP rows in VP-INDEX summary; 5 proof files inspected for stub-grade content.
- **Dim 8 (doc drift):** sampled CLAUDE.md ("24-crate workspace"), README.md (verified stub), `module-decomposition.md` (35-tool claim), `custom_adapter.rs:1-9` ("pure TOML" claim), VP-INDEX summary table.

### Out of Scope

- `.worktrees/S-3.09/` (frozen mid-cascade — explicitly excluded).
- `mutants.out/` and `mutants.out.old/` (mutation artifact directories).
- Reference repos under `.references/`.
- Dead-code lint analysis at the field/import level (would explode the report; reserved for a follow-up).
- Code-quality dimensions other than stub residue (style, formatting, doc completeness).

---

## Findings by Dimension

### Dim 1: Production stub residue

| Site | Function | Stub kind | Story |
|---|---|---|---|
| `crates/prism-query/src/engine.rs:276` | `QueryEngine::execute` | `todo!()` | S-3.02 |
| `crates/prism-query/src/engine.rs:317` | `QueryEngine::execute_scheduled` | `todo!()` | S-3.02 |
| `crates/prism-query/src/materialization.rs:241` | `run_materialization_pipeline` | `todo!()` | S-3.02 |
| `crates/prism-query/src/materialization.rs:263` | `resolve_source_refs` | `todo!()` | S-3.02 |
| `crates/prism-query/src/internal_tables.rs:125` | `RocksDbTableProvider::schema` | `todo!()` | S-3.02 |
| `crates/prism-query/src/internal_tables.rs:129` | `RocksDbTableProvider::table_type` | `todo!()` | S-3.02 |
| `crates/prism-query/src/internal_tables.rs:139` | `RocksDbTableProvider::scan` | `todo!()` | S-3.02 |
| `crates/prism-query/src/internal_tables.rs:146` | `supports_filters_pushdown` | `todo!()` | S-3.02 |
| `crates/prism-query/src/internal_tables.rs:168` | `register_internal_tables` | `todo!()` | S-3.02 |
| `crates/prism-query/src/pushdown.rs:199` | `translate_push_down_filter` | `todo!()` | S-3.X (deferred) |
| `crates/prism-query/src/write_pipeline.rs:349` | Phase 3 fetch (hardcoded empty) | structural stub | S-3.07 |
| `crates/prism-query/src/write_table_registration.rs:176` | `insert_into` SQL DML | `NotImplemented` Err | S-3.07 |
| `crates/prism-query/src/write_table_registration.rs:190` | `delete_from` SQL DML | `NotImplemented` Err | S-3.07 |
| `crates/prism-query/src/write_table_registration.rs:205` | `update` SQL DML | `NotImplemented` Err | S-3.07 |
| `crates/prism-sensors/src/adapter.rs:365` | default `SensorAdapter::write()` | `Err(WriteNotImplemented)` | S-3.07 |
| `crates/prism-spec-engine/src/hot_reload.rs:66` | `HotReloadWatcher::start` | `unimplemented!()` | S-1.12 |
| `crates/prism-spec-engine/src/hot_reload.rs:72` | `HotReloadWatcher::stop` | `unimplemented!()` | S-1.12 |
| `crates/prism-spec-engine/src/infusion/loader.rs:42,50,57,66` | `InfusionLoader::*` (4) | `unimplemented!()` | S-1.14 |
| `crates/prism-spec-engine/src/infusion/cache.rs:109,120` | `InfusionLruCache::get/insert` | `unimplemented!()` | S-1.14 |
| `crates/prism-spec-engine/src/infusion/sources/mmdb.rs:23,29,37` | `MmdbSource::*` (3) | `unimplemented!()` | S-1.14 |
| `crates/prism-spec-engine/src/infusion/sources/csv.rs:28,34,42` | `CsvSource::*` (3) | `unimplemented!()` | S-1.14 |
| `crates/prism-spec-engine/src/infusion/sources/json_lookup.rs:22,28,36` | `JsonLookupSource::*` (3) | `unimplemented!()` | S-1.14 |
| `crates/prism-spec-engine/src/infusion/sources/mod.rs:23` | `load_source` | `unimplemented!()` | S-1.14 |
| `crates/prism-spec-engine/src/infusion/plugin_bridge.rs:26,37` | plugin-bridge enrich (2) | `unimplemented!()` | S-1.14 |
| `crates/prism-spec-engine/src/proofs/plugin_linker.rs:95` | VP-040 proof body | `unimplemented!()` | S-1.15 |
| `crates/prism-dtu-common/src/clone.rs:79,93` | trait-default `start_on`/`stop` | `unimplemented!()` (overridden in all clones) | S-6.06 |

**Findings (Dim 1)**

**F-AUD-D1-01** [P0] `QueryEngine::execute` is `todo!()` — entire query lifecycle is a stub
- File: `crates/prism-query/src/engine.rs:276`
- Story: S-3.02 status:ready (file frontmatter); STORY-INDEX claims MERGED PR #129 2026-05-06
- BC: BC-2.11.001, BC-2.11.005, BC-2.11.006, BC-2.11.011 (all unmet)
- Evidence: `pub async fn execute(&self, _query_str: &str, _options: QueryOptions) -> Result<QueryResult, PrismError> { todo!("S-3.02 — QueryEngine::execute") }`
- Why it matters: This is the entry point for every query. No analyst PrismQL query can be served. The "merged" claim is false.
- Suggested route: implementer (re-open S-3.02 or file S-3.02-FOLLOWUP)

**F-AUD-D1-02** [P0] `run_materialization_pipeline` is `todo!()` — the 8-step ephemeral pipeline does not exist
- File: `crates/prism-query/src/materialization.rs:241`
- Story: S-3.02 status:ready
- BC: BC-2.11.005, BC-2.11.006, BC-2.11.007, BC-2.11.011, BC-2.11.012 (all unmet)
- Evidence: `pub async fn run_materialization_pipeline(...) -> Result<Vec<RecordBatch>, PrismError> { todo!("S-3.02 — run_materialization_pipeline") }`
- Why it matters: All 8 pipeline steps (parse → resolve → fan-out → normalize → inject → register → SQL → collect) are absent in production despite spec claiming them as the contract.
- Suggested route: implementer

**F-AUD-D1-03** [P0] `RocksDbTableProvider` is fully stubbed — internal tables (`prism_audit`, `prism_alerts`, etc.) cannot be queried
- File: `crates/prism-query/src/internal_tables.rs:125,129,139,146,168`
- Story: S-3.02
- BC: BC-2.15.011 (capability gate on `prism_audit`), BC-2.11.005 (internal tables in SessionContext)
- Evidence: 4 trait methods + 1 free function all `todo!()`
- Why it matters: Audit queries, alert queries, case queries via PrismQL all hit `todo!` panics. Capability gate on `prism_audit` is unenforced.
- Suggested route: implementer

**F-AUD-D1-04** [P0] `SensorAdapter::write()` default body returns `WriteNotImplemented`; NO concrete sensor overrides it
- File: `crates/prism-sensors/src/adapter.rs:365`; verified absence of overrides via `rg 'async fn write\b' crates/prism-sensors/src/auth/{crowdstrike,armis,cyberint,claroty}.rs` returning zero matches
- Story: S-3.07 status:draft (file frontmatter); STORY-INDEX claims MERGED PR #135 2026-05-08
- BC: BC-2.04.007, AD-022 — write operations to sensors
- Evidence: `Err(SensorError::WriteNotImplemented { sensor: self.sensor_name().to_string() })` is the universal return; comment line 361 reads `TODO: W3-FIX-S307-001 — override write() in each concrete adapter.`
- Why it matters: Any pipe-mode write (`| contain`, `| acknowledge`, etc.) returns NotImplemented for ALL four built-in sensors. Combined with F-AUD-D1-05 below, the entire write path is non-functional in production.
- Suggested route: implementer (W3-FIX-S307-001)

**F-AUD-D1-05** [P0] `WriteExecutor::execute` Phase 3 fetch is hardcoded `vec![]` — writes never have records to write
- File: `crates/prism-query/src/write_pipeline.rs:349`
- Story: S-3.07 status:draft
- BC: BC-2.04.007, AD-022
- Evidence: `let fetched_records: Vec<arrow::record_batch::RecordBatch> = vec![];` — no QueryMaterializer integration; comment at line 344 says "QueryMaterializer integration is S-3.02" (which is itself stubbed — see F-AUD-D1-02).
- Why it matters: Even if F-AUD-D1-04 were fixed, the dispatcher iterates `records.iter()` over an empty slice and returns `(vec![], vec![])`. The pipeline gates (feature flags, dry-run, capability check) all pass, but no work happens.
- Suggested route: implementer (cross-cutting with S-3.02 implementation)

**F-AUD-D1-06** [P0] SQL DML routing through `WriteCapableTableProvider` returns `NotImplemented` for all 3 verbs
- File: `crates/prism-query/src/write_table_registration.rs:176, 190, 205`
- Story: S-3.07 status:draft; deferred to W3-FIX-S307-003
- BC: BC-2.04.007, AD-022
- Evidence: All three of `insert_into`, `delete_from`, `update` return `Err(DataFusionError::NotImplemented(...))` with the literal string `"S-3.07-pending"`.
- Why it matters: SQL DML mode of AD-022 is half-built. Pipe-mode writes are also broken (F-AUD-D1-04/05) so neither write modality functions.
- Suggested route: implementer

**F-AUD-D1-07** [P0] `HotReloadWatcher::start/stop` are `unimplemented!()` — AD-018 not delivered
- File: `crates/prism-spec-engine/src/hot_reload.rs:66, 72`
- Story: S-1.12 status:merged
- BC: BC-2.16.007 — hot reload contract; AD-018 (notify crate watcher)
- Evidence: `unimplemented!("S-1.12: HotReloadWatcher::start not yet implemented — Red Gate stub")`
- Why it matters: confirmed by prior plugin audit, repeated here for completeness. Status:merged is a lie.
- Suggested route: implementer (S-1.12-FOLLOWUP)

**F-AUD-D1-08** [P0] Infusion source backends (mmdb, csv, json_lookup) are 100% `unimplemented!()`
- File: `crates/prism-spec-engine/src/infusion/sources/{mmdb,csv,json_lookup}.rs` — 9 unimplemented bodies
- Story: S-1.14 status:merged
- BC: BC-2.19.001 (load), BC-2.19.002 (cache), BC-2.19.005 (credentials)
- Evidence: every `enrich_single`/`enrich_batch`/`load` panics; `InfusionRegistry::new()` ships a `NullSource` that returns `None` for every input (`infusion/mod.rs:236-246`).
- Why it matters: confirmed by prior plugin audit, repeated for completeness. The framework's data sources are all stubs; any `| enrich geoip` query at runtime would panic.
- Suggested route: implementer (S-1.14-REDO or formal retirement)

**F-AUD-D1-09** [P0] `InfusionLruCache::get/insert` are `unimplemented!()` — cache is a stub
- File: `crates/prism-spec-engine/src/infusion/cache.rs:109, 120`
- Story: S-1.14 status:merged
- BC: BC-2.19.002 (per-query dedup cache); VP-049 (proptest for dedup)
- Evidence: both methods `unimplemented!("InfusionLruCache::* — implement in S-1.14 (BC-2.19.002)")`
- Why it matters: VP-049 cannot be exercised; if any caller tried to insert a cache entry, the panic would propagate.
- Suggested route: implementer

**F-AUD-D1-10** [P0] `InfusionLoader::parse/load_all/validate_credentials` are `unimplemented!()`
- File: `crates/prism-spec-engine/src/infusion/loader.rs:42,50,57,66`
- Story: S-1.14 status:merged
- BC: BC-2.19.001, BC-2.19.005
- Evidence: 4 `unimplemented!()` panics per file
- Why it matters: spec-engine cannot read `.infusion.toml` in production. The fixtures at `crates/prism-spec-engine/fixtures/{geoip,asset_inventory,threat_intel_plugin}.infusion.toml` are orphaned.
- Suggested route: implementer

**F-AUD-D1-11** [P0] `plugin_bridge` (S-1.14 ↔ S-1.15 wiring) is `unimplemented!()` in both directions
- File: `crates/prism-spec-engine/src/infusion/plugin_bridge.rs:26, 37`
- Story: S-1.14 status:merged + S-1.15 status:merged
- BC: bridges BC-2.17.x and BC-2.19.x
- Evidence: `unimplemented!()` × 2; loader.rs:6 admits "If S-1.15 is not yet built, this stub panics"
- Why it matters: even if both S-1.14 and S-1.15 leaf stubs were filled, the bridge between them is missing.
- Suggested route: architect — decide whether to keep this dependency or restructure.

**F-AUD-D1-12** [P1] `pushdown::translate_push_down_filter` is `todo!("S-3.X")` — push-down filters never actually translate to sensor-native syntax
- File: `crates/prism-query/src/pushdown.rs:199`
- Story: deferred to "S-3.X — sensor-specific filter translation" (no story exists)
- BC: BC-2.11.007 (sensor filter push-down)
- Evidence: `todo!("S-3.X — sensor-specific filter translation")` — comment notes "Debug-formatted Expr would leak AST internals to external sensor APIs (CWE-209)"
- Why it matters: BC-2.11.007 unmet — all push-down filters degrade to post-filter at the engine layer.
- Suggested route: story-writer (per-sensor S-3.x stories)

**F-AUD-D1-13** [P1] `AdapterRegistry::adapters` field doc-comment lies about being `todo!()` (it's actually populated)
- File: `crates/prism-sensors/src/registry.rs:38`
- Story: S-3.1.06-ImplPhase status:merged
- Evidence: `Stub body: todo!() until init_registry_for_org wires org_id through all adapter constructors (S-3.1.06-ImplPhase AC-002).` — but the field is just a `HashMap` and `register()` mutates it normally.
- Why it matters: stale doc that would mislead a new contributor; harmless at runtime but suggests deeper doc hygiene problems.
- Suggested route: implementer (1-line doc fix)

**F-AUD-D1-14** [P2] VP-040 Kani proof harness is `unimplemented!()`
- File: `crates/prism-spec-engine/src/proofs/plugin_linker.rs:95`
- Story: S-1.15
- VP: VP-040 (status:draft in VP-INDEX)
- Evidence: `unimplemented!("VP-040 Kani path — awaiting wasmtime Linker enumeration API")`
- Why it matters: confirms VP-040 is harness-only with the harness body itself blocked on an upstream API.
- Suggested route: architect — decide whether to retire VP-040 or wait on wasmtime.

**F-AUD-D1-15** [P2] `audit_emitter::emit` and `build_pre_invocation_entry` retain `todo!()` literal-strings in comments after implementation
- File: `crates/prism-audit/src/audit_emitter.rs:342, 374`
- Story: S-2.04
- Evidence: lines retain `// todo!("AC-1 / BC-2.05.001: ...")` as commented-out historical markers; the actual function bodies below are real.
- Why it matters: not a bug, but residue from the stub-merge convention. Any automated stub-grep tool will flag these as false positives. Should be cleaned.
- Suggested route: implementer (housekeeping)

**F-AUD-D1-16** [P2] Many doc-comments still say "STUB — todo!() pending" when bodies are implemented
- Files: `crates/prism-credentials/src/trait_.rs:11,31,41,53,65,70,108,120,133,145,153`; `crates/prism-security/src/content_hash.rs:1-9`; `crates/prism-security/src/risk_tier.rs:1-9`
- Story: S-1.06, S-1.09
- Evidence: function bodies are real (verified via `rg unimplemented! crates/prism-credentials/src/{file,keyring}.rs` returns no matches; `compute_action_hash` in content_hash.rs:41 is a complete SHA-256 implementation), but doc comments still say "STUB"
- Why it matters: silent doc-drift; CLAUDE.md or new contributors may waste time investigating non-stubs.
- Suggested route: implementer

**F-AUD-D1-17** [P2] BehavioralClone trait default `start_on`/`stop` panic with `unimplemented!()` (overridden in all 10 concrete clones)
- File: `crates/prism-dtu-common/src/clone.rs:79, 93`
- Story: S-6.06 status:merged
- Evidence: trait defaults; `rg 'fn (start_on|stop)\b'` confirms override exists in every concrete clone (`prism-dtu-{nvd,armis,jira,threatintel,slack,cyberint,pagerduty,crowdstrike,claroty}/src/clone.rs` plus this default).
- Why it matters: defaults are dead code at runtime today, but if a new clone is added without override, runtime panic will fire. Defensive default should at minimum produce a `Result::Err`.
- Suggested route: implementer (defensive refactor)

---

### Dim 2: Story-vs-impl drift

**F-AUD-D2-01** [P0] STORY-INDEX status column is unreliable as oracle — at least 4 stories indexed as MERGED have `status: ready`/`draft` in their own frontmatter
- Files:
  - `S-3.02-query-materialization.md`: STORY-INDEX says "MERGED PR #129 6fefc774 2026-05-06"; frontmatter says `status: ready`.
  - `S-3.06-prismql-write-parser.md`: STORY-INDEX says "MERGED PR #130 2a7b83f5 2026-05-06"; frontmatter says `status: ready`.
  - `S-3.07-write-execution.md`: STORY-INDEX says "MERGED 2026-05-08T04:23:03Z PR #135 squash 2ae7185b"; frontmatter says `status: draft`.
  - `S-1.10-prompt-injection-defense.md`: frontmatter says `status: delivered` (not `merged`); STORY-INDEX uses bare row.
- Why it matters: any audit/script/agent that reads STORY-INDEX as the source of truth gets a different answer than reading the story file. There is no "partial-merge" enum, so stub-merges look identical to fully-merged stories. The plugin-audit headline finding is repeated-and-amplified here.
- Suggested route: state-manager + architect; introduce `status: partial-merge` enum (or split into `code_status` + `spec_status`) and reconcile.

**F-AUD-D2-02** [P0] S-1.11 (Spec Loading and Pipeline Execution) is partially structural — parser/validator real, `PipelineExecutor::execute` returns empty
- (covered in prior plugin audit — TD-PLUGIN-P0-001) — recorded for completeness.

**F-AUD-D2-03** [P0] S-1.12 (Hot Reload) STUB-MERGED — `HotReloadWatcher::start/stop` `unimplemented!()`
- (TD-PLUGIN-P0-003 — referenced for completeness)

**F-AUD-D2-04** [P0] S-1.14 (Infusion) STUB-MERGED — entire `infusion/` module is `unimplemented!()` panics
- (TD-PLUGIN-P0-002)

**F-AUD-D2-05** [P0] S-1.15 (WASM runtime) — runtime real but no production wiring; action-plugin dispatch stubbed
- (TD-PLUGIN-P0-008)

**F-AUD-D2-06** [P0] S-3.02 (Query and Materialization) — entire `engine.rs` execute path + materialization pipeline + internal-tables provider are `todo!()`
- See F-AUD-D1-01/02/03. STORY-INDEX claims MERGED 2026-05-06; story frontmatter says `ready`. The "merged PR #129" landed structural types, BC anchors, scaffolding — but not the runtime.

**F-AUD-D2-07** [P0] S-3.07 (Write Execution Pipeline) — entire write path is non-functional
- See F-AUD-D1-04/05/06. STORY-INDEX claims MERGED 2026-05-08; story frontmatter says `draft`. Three independent stub points: empty fetch, no concrete adapter override, NotImplemented SQL DML.

**F-AUD-D2-08** [P1] S-3.07 dispatched to W3-FIX-S307-001/002/003 follow-ups but none are filed in STORY-INDEX
- Files: `write_pipeline.rs:344` mentions "QueryMaterializer integration is S-3.02"; `adapter.rs:361` mentions "W3-FIX-S307-001"; `write_table_registration.rs:175,189,204` reference "W3-FIX-S307-003".
- Evidence: searching STORY-INDEX for `W3-FIX-S307-` returns zero matches. `ls .factory/stories/ | grep S307` also returns zero matches.
- Why it matters: deferred work has no story file — purely a code-comment IOU. Cannot be tracked.
- Suggested route: story-writer

**F-AUD-D2-09** [P1] S-1.13 (Sensor Spec Write Endpoints) — `WriteEndpointRegistry::new()` only constructed in tests, never in production code path
- (TD-PLUGIN-P0-006 — recorded)

**F-AUD-D2-10** [P1] S-3.05 (Pagination and Caching) — STORY-INDEX shows no MERGED tag; frontmatter status:draft; prism-query has 6 ignored TD-S305-001..006 tests
- Files: `crates/prism-query/src/tests/pagination_tests.rs:160,461,489,506,576`; `cache_tests.rs:323`
- Evidence: 6 `#[ignore = "TD-S305-XXX: ..."]` tags inside the tests for cache/pagination.
- Why it matters: S-3.05 is genuinely incomplete and the team wisely tagged the deferred test cases. This is the OPPOSITE pattern to the others — proper deferral. Calibration baseline for "honest deferral" looks like.
- Suggested route: state-manager; verify TD-S305-001..006 are all tracked.

**F-AUD-D2-11** [P1] S-1.10 (Prompt Injection Defense) status:delivered (not merged) — schema enum drift
- File: `S-1.10-prompt-injection-defense.md` frontmatter
- Evidence: only story using `status: delivered`. Code itself looks real (injection_scanner.rs, fuzz target VP-038 wired).
- Why it matters: enum drift in status field — yet another evidence point that the status taxonomy is unenforced.
- Suggested route: state-manager (rename to `merged` or add to enum)

**F-AUD-D2-12** [P0] No production binary loads sensor TOMLs — orphan sensors at `sensors/*.sensor.toml`
- (TD-PLUGIN-P0-007 — umbrella over BUG-S309-PLUGIN — recorded)

**F-AUD-D2-13** [P0] `prism-mcp` is a 10-line stub — SS-10 (MCP Interface) is undelivered
- File: `crates/prism-mcp/src/lib.rs` (10 lines total: 1 module decl, 2 `pub mod` entries, 2 `pub use`).
- Story: S-5.01 (mcp-bootstrap) status:draft; S-5.02 (tool-routing) status:draft; S-5.03 (resources-prompts) status:draft
- BC: SS-10 owns 11 BCs (BC-2.10.001..010 + others); none satisfied.
- Architecture claim: `module-decomposition.md:513` says "prism-mcp ... 35 PrismServer, tool dispatch, resource/prompt handlers, config tool surface, health probe tools".
- Evidence: `wc -l crates/prism-mcp/src/lib.rs` → 10 lines; only sub-modules are `safety_envelope.rs` and `tool_registry.rs` (both small, neither exposes a tool router or rmcp server).
- Why it matters: there is no MCP server binary at all. Per CLAUDE.md the runtime is a "per-analyst MCP in Claude Code" — but no MCP exists. Architect's headline "35+ tool registration via rmcp 1.4 #[tool_router] macro" is aspirational, not built.
- Suggested route: architect + story-writer (re-scope Wave 5 prerequisites)

**F-AUD-D2-14** [P1] S-3.07 includes write parsing (S-3.06) but spec write_table_registration.rs depends on `WriteEndpointRegistry::table_descriptors()` which is never called in any binary
- Cross-references S-3.07 ↔ S-1.13 ↔ S-3.06 dependency chain; none of these reach a binary
- Suggested route: architect

---

### Dim 3: Silent-shallow tests

**F-AUD-D3-01** [P0] `test_BC_2_16_007_hot_reload_watcher_start_is_stub` codifies the hole — passes because production panics
- File: `crates/prism-spec-engine/tests/hot_reload_tests.rs:1085-1097`
- Story: S-1.12
- Evidence:
  ```rust
  #[test]
  #[should_panic(expected = "not yet implemented")]
  fn test_BC_2_16_007_hot_reload_watcher_start_is_stub() {
      ...
      let _ = HotReloadWatcher::start(config, manager);
  }
  ```
- Why it matters: this test PASSES specifically because the production code remains unimplemented. The day someone fills in `HotReloadWatcher::start`, this test will FAIL. It's an inverted-polarity test that codifies the stub as the contract. Worst case of silent-shallow: actively prevents implementation by making a green CI dependent on a broken prod path.
- Suggested route: implementer (delete this test as part of S-1.12-FOLLOWUP)

**F-AUD-D3-02** [P0] `bc_3_2_002_org_id_namespace.rs` test header literally documents "MUST FAIL: ... is `todo!()` stub"
- File: `crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs:6, 105, 122, 144, 162, 190, 877`
- Story: S-3.1.04
- Evidence: 6 doc comments all say "MUST FAIL: ... is a `todo!()` stub" — but actual `set_by_org`/`get_by_org`/`namespace_key_by_org_id` are now real (see `crates/prism-credentials/src/file.rs:293` impl block, `keyring.rs:56` impl block — neither contains `unimplemented!`).
- Why it matters: comments are stale; tests probably pass; but anyone reading the test file is being lied to. If the comments are stale-from-stub-phase, this is the silent-shallow problem residue.
- Suggested route: implementer (cleanup test docs)

**F-AUD-D3-03** [P0] DTU-cyberint multi_tenant tests document `reset_for` and `extract_org_id` as `todo!()` but those functions are now implemented
- File: `crates/prism-dtu-cyberint/tests/multi_tenant.rs:29-31, 349, 374, 417, 440, 598, 635, 665, 677, 739`
- Story: S-3.2.04
- Evidence: 11 comment sites claim "RED GATE", "MUST FAIL", "is todo!()" — but `crates/prism-dtu-cyberint/src/state.rs:257` has real `pub fn reset_for(&self, org_id: OrgId)` and `routes/alerts.rs:66` has real `pub fn extract_org_id`.
- Why it matters: same pattern as D3-02 — documentation lies about test polarity. Either the tests pass (and docs are stale) or fail (and merge was incorrect).
- Suggested route: implementer (run tests, sweep docs)

**F-AUD-D3-04** [P1] 6 `#[ignore = "TD-S305-XXX"]` tests in `prism-query` cache/pagination — proper deferral but signals unfinished work
- Files: `crates/prism-query/src/tests/pagination_tests.rs:160,461,489,506,576`; `cache_tests.rs:323`
- Story: S-3.05 status:draft
- Why it matters: properly tagged but suggests S-3.05 is not actually merged; should not be flipped to merged until these unblock.
- Suggested route: story-writer

**F-AUD-D3-05** [P1] `test_bc_3_1_001_ac_1_from_uuid_panics_on_v4` doc claims "test FAILS" but actual production code makes it PASS
- File: `crates/prism-core/tests/bc_3_1_001_org_id.rs:85-103`
- Story: S-3.1.01
- Evidence: doc says "current `from_uuid()` does NOT enforce the version and does NOT panic. This `#[should_panic]` test FAILS"; but the test actually calls `OrgId::from_uuid_v7(v4_uuid)` (not `from_uuid`), and `from_uuid_v7` DOES panic with `"not a UUID v7"` (verified at `crates/prism-core/src/ids.rs:80-87`).
- Why it matters: misleading doc but functional test. The reverse of D3-02/03.
- Suggested route: implementer (doc fix)

**F-AUD-D3-06** [P1] DTU integration tests `#[ignore = "needs-prism-audit"]` 4 sites — tests waiting on absent prism-audit wiring
- File: `crates/prism-dtu-crowdstrike/tests/{harness_tests.rs:1514,1588,integration_vp036.rs:35,integration_vp033.rs:33}`
- Story: S-6.07 / VP-033 / VP-036
- Why it matters: VP-033 and VP-036 are claimed to anchor on S-6.07 but their test vehicles are gated on cross-crate wiring that doesn't exist.
- Suggested route: implementer + state-manager

**F-AUD-D3-07** [P1] `crates/prism-spec-engine/proofs/infusion_spec.rs:13` admits Kani proofs cannot run because of stub
- Evidence: `//! \`unimplemented!()\`. The Kani proofs will NOT compile/run until implementation`
- Why it matters: proof file declares its own un-runnability. VP-048 is paper-only.
- Suggested route: implementer

**F-AUD-D3-08** [P2] `crates/prism-query/src/proofs/vp012_depth_limit.rs:48,69,76,84,89` proptests assert `is_err()` because production `todo!()` fires
- Evidence: 5 proptest sites with `assert!(result.is_err(), "todo!() fires — RED gate")` and 3 explicit comments "RED: AliasResolver::expand is todo!()"
- Story: S-3.04
- Note: actual `AliasResolver::expand` may now be real — needs verification. If real, these proptests pass for the wrong reason (test passes because impl returns Err, not because impl is `todo!()`).
- Suggested route: implementer (verify and clean RED gate language)

**F-AUD-D3-09** [P2] `crates/prism-query/src/proofs/vp013_cycle_detection.rs:152,198,206,212,230` similar RED-gate stub-asserts
- Evidence: 5 sites with `// RED — fires todo!() on each invocation`
- Suggested route: implementer

---

### Dim 4: TOML/config orphans

**F-AUD-D4-01** [P0] `sensors/{crowdstrike,armis,claroty,cyberint}.sensor.toml` are not loaded by any production code path
- Evidence: `rg 'parse_spec_directory|SpecLoader|ConfigManager::new' --type rust crates/` outside `prism-spec-engine` itself returns ONLY:
  - `crates/prism-spec-engine/examples/demo_spec_loading.rs` (an example, not production)
  - `fuzz/fuzz_targets/spec_parser.rs` (fuzz harness)
- (TD-PLUGIN-P0-007 / TD-PLUGIN-P0-004 — recorded)

**F-AUD-D4-02** [P0] All 4 sensor TOMLs are write-only — no `[[tables]]` or `[[tables.steps]]` for read pipelines
- Files: `sensors/{crowdstrike,armis,claroty,cyberint}.sensor.toml`
- Evidence: `crowdstrike.sensor.toml:3` literally says `# Read-side tables: implement in S-1.11.` — never delivered.
- Suggested route: story-writer (S-1.11-FOLLOWUP) — copy from `crates/prism-spec-engine/examples/demo_spec_loading.rs:36-79`

**F-AUD-D4-03** [P0] Infusion fixtures `crates/prism-spec-engine/fixtures/{geoip,asset_inventory,threat_intel_plugin}.infusion.toml` are not loaded — `InfusionLoader::load_all` is `unimplemented!()`
- Cross-reference F-AUD-D1-10
- Why it matters: fixtures committed to repo with no consumer; tests reference them but production never reads them.
- Suggested route: implementer (S-1.14-REDO)

**F-AUD-D4-04** [P1] `crates/prism-spec-engine/fixtures/{noop_infusion.wasm, noop_infusion.wat, loop_plugin.wat, trap_plugin.wat}` — WASM plugin fixtures with no production loader
- Evidence: only consumed by `crates/prism-spec-engine/tests/` and `proofs/`
- Why it matters: similar to F-AUD-D4-03 — the runtime that would load these (S-1.15) has no production wiring.
- Suggested route: state-manager (track or retire)

**F-AUD-D4-05** [P1] `crates/prism-dtu-demo-server/configs/{prism-demo.toml, demo.toml}` are loaded ONLY by the demo-server binary, not by main prism runtime
- Evidence: `rg 'prism-demo\.toml|demo\.toml' --type rust crates/` returns only references inside `prism-dtu-demo-server`.
- Why it matters: this is correct — they're scoped to demo. Recording as P1 because someone might assume `prism-demo.toml` is the main config. Doc clarification needed.
- Suggested route: implementer (rename or doc-comment)

**F-AUD-D4-06** [P1] No top-level `configs/` directory exists despite `architect`'s `config-schema.md` and S-5.05 (config-loading) story
- Evidence: `find configs -type f` returns nothing at workspace root.
- Why it matters: there is no example `prism.toml` for an analyst to use; S-5.05 is `status: draft`.
- Suggested route: story-writer (track in S-5.05)

**F-AUD-D4-07** [P2] Two parallel `SensorSpec` types in spec-engine with silent drift
- Files: `crates/prism-spec-engine/src/spec_parser.rs:189` vs `crates/prism-spec-engine/src/types.rs` (referenced as `types::SensorSpec`)
- (TD-PLUGIN-P0-005 — recorded)

---

### Dim 5: BC postcondition gaps

**F-AUD-D5-01** [P0] **All 222 active BCs are `status: draft`** — none have been promoted to `active`/`verified` despite many waves merged
- Evidence: BC-INDEX.md frontmatter `active_contracts: 222`; spot check of BC-2.04.001 file → `status: draft`; BC-2.01.013 → `status: draft`; sweep over `.factory/specs/behavioral-contracts/*.md` confirms.
- Why it matters: BC status taxonomy is unused. There's no signal in BC-INDEX about whether a BC's postconditions are verified by code.
- Suggested route: architect — define the status promotion criteria and run a sweep.

**F-AUD-D5-02** [P0] BC-2.11.001/005/006/007/011/012 (query engine) postconditions are aspirational — `QueryEngine::execute` is `todo!()`
- Cross-reference F-AUD-D1-01/02
- Suggested route: implementer

**F-AUD-D5-03** [P0] BC-2.04.007 (Three-Tier Risk Classification) and BC-2.04.008 (Dry-Run Default) postconditions partially unmet — write path Phase 3 fetch empty (F-AUD-D1-05)
- Even though feature flag gates and risk tier enums are real, the contract postcondition "the operation succeeds for permitted writes" cannot be verified because production path returns empty.
- Suggested route: implementer

**F-AUD-D5-04** [P0] BC-2.16.007 (Sensor Spec Hot Reload) postcondition unmet — watcher `unimplemented!()`
- Cross-reference F-AUD-D1-07
- Suggested route: implementer

**F-AUD-D5-05** [P0] BC-2.17.001..006 (WASM Plugin Runtime) — runtime real, but no production loader; action-plugin dispatch stubbed
- Cross-reference plugin audit §4
- Suggested route: implementer

**F-AUD-D5-06** [P0] BC-2.19.001..005 (Infusions) — all 5 BCs unmet because module is unimplemented
- Cross-reference F-AUD-D1-08/09/10
- Suggested route: implementer

**F-AUD-D5-07** [P1] BC-2.10.001..010 + extension BCs for SS-10 (MCP Interface) — none satisfied; no MCP server exists
- Cross-reference F-AUD-D2-13
- Suggested route: architect

**F-AUD-D5-08** [P1] BC-2.05.001 (Audit Entry serialization) postcondition met BUT comment in `audit_emitter.rs:342, 374` retains stub markers
- Doc drift only; functional code is real.
- Suggested route: implementer (cleanup)

**F-AUD-D5-09** [P1] BC-2.07.005 (Cache Key Derivation) — cache_key.rs implementation is real; VP-025 proof file (`vp025_cache_key.rs`) still says `todo!()` in its comments
- Verified `cache_key.rs:160 fn derive_push_down_hash` is a real implementation; the proof file's stub-language is now stale.
- Suggested route: architect (re-run Kani; promote VP-025 status)

---

### Dim 6: AD/ADR implementation status

ADR-001 → ADR-019 reviewed against `crates/`. Findings:

| ADR | Title | ARCH-INDEX status | Real impl status | Finding |
|---|---|---|---|---|
| ADR-001 | DTU Rate Limit Pattern | ACCEPTED | implemented (DTU clones) | OK |
| ADR-002 | L2 DTU Clone Template | ACCEPTED | implemented (BehavioralClone trait + clones) | OK; F-AUD-D1-17 minor |
| ADR-003 | DTU Reset Lookup and Fidelity Auth | ACCEPTED | implemented | OK |
| ADR-004 | Kani Arbitrary Policy | PROPOSED | partial (some `kani::Arbitrary` derives) | OK as PROPOSED |
| ADR-005 | AQL Injection Mitigation | ACCEPTED | implemented (Armis adapter) | OK |
| ADR-006 | Multi-Tenant DTU Topology — OrgId/OrgSlug + OrgRegistry | ACCEPTED | implemented (S-3.1.x series) | OK |
| ADR-007 | Configurable Shared/Client DTU Mode | ACCEPTED | implemented | OK |
| ADR-008 | DTU State Segregation | ACCEPTED | implemented | OK |
| ADR-009 | Multi-Tenant Data Generator | ACCEPTED | implemented (S-3.7.x) | OK |
| ADR-010 | Customer Config Schema | ACCEPTED | implemented (`prism-customer-config`) | OK |
| ADR-011 | DTU Harness Isolation Modes | ACCEPTED | implemented (`prism-dtu-harness`) | OK |
| ADR-012 | src/ Convention Normalization | ACCEPTED | implemented (S-3.5.01) | OK |
| ADR-013 | Schedule Execution Semantics | PROPOSED v0.7 | not implemented (S-4.x draft) | aspirational |
| ADR-014 | Local pre-push CI gate asymmetry | (not in ARCH-INDEX) | implemented (`lefthook.yml`) | F-AUD-D6-04 |
| ADR-015 | Detection Rule Language | PROPOSED v0.6 | not implemented (S-4.x draft) | aspirational |
| ADR-016 | Action Delivery Framework | PROPOSED v0.14 | not implemented (S-4.08 draft) | aspirational |
| ADR-017 | Case Lifecycle Invariants | PROPOSED v0.7 | not implemented (S-4.06 draft) | aspirational |
| ADR-018 | Differential Result Pack Format | PROPOSED v0.6 | not implemented (S-4.02 draft) | aspirational |
| ADR-019 | SIEM Output Formats | PROPOSED v0.4 | not implemented (S-4.08 + prism-siem-formats absent) | aspirational |

| AD | Title | Status | Real impl |
|---|---|---|---|
| AD-001 | Modular monolith (Cargo workspace) | accepted | implemented (24 crates) |
| AD-002 | DataFusion as SQL engine | accepted | partial — engine integrated, but `QueryEngine::execute` is `todo!()` (F-AUD-D1-01) |
| AD-003 | Chumsky 0.12 | accepted | implemented (`prism-query/src/sql_parser.rs` etc.) |
| AD-004 | RocksDB 17 column families | accepted | implemented (`prism-storage`) |
| AD-005 | rmcp 1.4 as MCP SDK | accepted | **NOT IMPLEMENTED** — F-AUD-D2-13; prism-mcp is 10-line stub |
| AD-006 | Config-driven sensor adapters via TOML | accepted | **PARTIAL** — schema real; production never loads TOMLs (F-AUD-D4-01/02) |
| AD-007 | arc-swap for hot config reload | accepted | implemented (ArcSwap in `config_manager.rs`) |
| AD-008 | Pure core / effectful shell separation | accepted | mostly implemented |
| AD-009 | Sealed trait pattern for SensorAuth | accepted | implemented |
| AD-010 | TenantId newtype (superseded by ADR-006) | superseded | OK |
| AD-011 | Two-tier feature flag system | accepted | implemented (`prism-security/feature_flags`) |
| AD-012 | Bincode for RocksDB values | accepted | implemented |
| AD-013 | tokio multi-threaded runtime | accepted | implemented |
| AD-014 | Process-level RSS watchdog | accepted | implemented (`prism-storage::watchdog`) |
| AD-015 | DynamicMessage protobuf for OCSF | accepted | implemented (`prism-ocsf`) |
| AD-016 | Write-audit ordering (intent-log) | accepted | implemented (`write_dispatch.rs:5a/5b/5c`) — but Phase 3 fetch is empty so no records reach audit |
| AD-017 | AI-opaque credential management | accepted | implemented (`prism-credentials`) |
| AD-018 | Auto filesystem watching for config reload | accepted | **NOT IMPLEMENTED** — F-AUD-D1-07; HotReloadWatcher::start `unimplemented!()` |
| AD-019 | WASM plugins | accepted | partial — runtime real, no production wiring; action-plugin dispatch stubbed |
| AD-020 | Infusions enrichment framework | accepted | **NOT IMPLEMENTED** — F-AUD-D1-08/09/10 |
| AD-021 | Actions config-driven delivery | accepted | not implemented (Wave 4) |
| AD-022 | PrismQL Write Operations | accepted | **PARTIAL** — gates real, fetch empty, no concrete adapter writes (F-AUD-D1-04/05/06) |

**F-AUD-D6-01** [P0] AD-005 (rmcp 1.4 as MCP SDK) accepted but not implemented — no MCP server in workspace
- Cross-reference F-AUD-D2-13
- Suggested route: architect

**F-AUD-D6-02** [P0] AD-006 (Config-driven sensor adapters) accepted but TOML loader unwired in production
- Cross-reference F-AUD-D4-01/02
- Suggested route: implementer

**F-AUD-D6-03** [P0] AD-018 (Auto filesystem watching) accepted but `HotReloadWatcher::start` is `unimplemented!()`
- Cross-reference F-AUD-D1-07
- Suggested route: implementer

**F-AUD-D6-04** [P1] ADR-014 (Local pre-push CI gate asymmetry) — file exists at `.factory/specs/architecture/decisions/ADR-014-local-pre-push-ci-gate-asymmetry.md` but is NOT registered in ARCH-INDEX `## ADR Registry`
- Evidence: ARCH-INDEX has rows for ADR-013, ADR-015, ADR-016, ADR-017, ADR-018, ADR-019 — but no ADR-014.
- Why it matters: registry/file drift; ARCH-INDEX is incomplete.
- Suggested route: architect (add to registry)

**F-AUD-D6-05** [P1] AD-020 (Infusions) accepted but the entire `infusion/` module is `unimplemented!()`
- Cross-reference F-AUD-D1-08
- Suggested route: implementer

**F-AUD-D6-06** [P1] AD-022 (PrismQL Write Operations) accepted but write path is non-functional
- Cross-reference F-AUD-D1-04/05/06
- Suggested route: implementer

---

### Dim 7: VP proof status

VP-INDEX summary table count: 145 properties.

**Status distribution:**
- `verified` — 2 (VP-014, VP-015)
- `draft` — 143 (98.6%)
- `harness-only` (proof file exists but not exercised) — implicitly the rest

**F-AUD-D7-01** [P0] Only 2 of 145 VPs are `verified`; all others are `draft` despite many waves merged
- Evidence: VP-INDEX summary at `.factory/specs/verification-properties/VP-INDEX.md:170-177`; `verified` status only for VP-014 and VP-015 (PR #127 promotion).
- Why it matters: verification claim across the workspace is essentially zero. Calling the project "formally verified" anywhere in docs is misleading.
- Suggested route: architect

**F-AUD-D7-02** [P0] VP-040..043, VP-048, VP-049 (plugin / infusion VPs) are all status:draft and the proof harnesses themselves call `unimplemented!()`
- Files: `crates/prism-spec-engine/src/proofs/{plugin_linker.rs:95, infusion_spec.rs:13, plugin_hot_reload.rs, plugin_memory.rs, plugin_wit_validation.rs, infusion_dedup.rs}`
- (TD-PLUGIN-P1-001 — recorded)

**F-AUD-D7-03** [P1] VP-025 (Cache Key Derivation) proof file `vp025_cache_key.rs` claims `todo!()` in cache_key insert/derive — but those functions are real
- Files: `crates/prism-query/src/proofs/vp025_cache_key.rs:64,99` doc comments stale; `crates/prism-query/src/cache_key.rs:71,160` actual functions implemented
- Why it matters: VP-025 could plausibly be promoted to `verified` after re-running Kani — proof body is ready, just gated on stale assumption.
- Suggested route: architect

**F-AUD-D7-04** [P1] VP-031 (Required column enforcement) anchors S-3.02 — proof exists in `vp031_pushdown.rs` but anchored story is itself `todo!()`-stubbed
- Cross-reference F-AUD-D1-02
- Suggested route: architect (defer until S-3.02 actually implements)

**F-AUD-D7-05** [P1] VP-021 (parser fuzz), VP-022 (OCSF normalizer fuzz), VP-023 (sensor spec parser fuzz), VP-024 (injection scanner detection), VP-038 (injection scanner fuzz) — fuzz targets exist in `fuzz/fuzz_targets/` but VP-INDEX says `status: draft`
- Files: `fuzz/fuzz_targets/{vp021_parse_fuzz.rs, normalize_fuzz.rs, spec_parser.rs, fuzz_injection_scanner.rs, vp037_alias_fuzz.rs}` — 5 targets exist
- Why it matters: fuzz targets exist but VP-INDEX reports them as draft; CI evidence and corpus hits not surfaced. Status promotion criteria unclear.
- Suggested route: architect — define the promotion criteria for fuzz-based VPs and run nightly artifact attestation.

**F-AUD-D7-06** [P1] VP-033, VP-036 (audit + SessionContext drop) anchored to S-6.07 with `#[ignore = "needs-prism-audit"]` integration tests
- Cross-reference F-AUD-D3-06
- Suggested route: implementer

**F-AUD-D7-07** [P2] VP-INDEX summary table arithmetic differs from row count
- Evidence: summary says "Total: 145"; bottom changelog row 1.14 says "Total row corrected 122/14 → 113/23"; row 1.12 says "Total 62→136"; row 1.16 says "76 verified correct" later corrected to "46 verified correct" then "28+46=74 total Wave 3 VPs". Row v1.17 references "26 → 28 VP anchor corrections".
- Why it matters: VP-INDEX has a dense audit trail of arithmetic errors. Whether the current count of 145 is correct should be re-verified.
- Suggested route: state-manager (audit row count vs summary)

---

### Dim 8: Documentation drift

**F-AUD-D8-01** [P0] `crates/prism-spec-engine/src/custom_adapter.rs:8` claim "The four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) are pure TOML" is FALSE
- Evidence: each of these sensors has a hardcoded Rust adapter at `crates/prism-sensors/src/auth/{crowdstrike,armis,cyberint,claroty}.rs` (e.g., `crowdstrike.rs:376 impl SensorAdapter for CrowdStrikeAdapter`); none is loaded via TOML at runtime.
- (Confirmed by prior plugin audit; recorded again because user asked specifically.)
- Suggested route: implementer (delete the misleading line)

**F-AUD-D8-02** [P1] `README.md` is itself a 6-line stub
- Evidence:
  ```
  # Prism
  <!-- TODO: S-0.01 — CI badge ... -->
  ![CI](...)
  TODO: S-0.01 stub — README content to be filled in.
  ```
- Why it matters: discoverability/onboarding non-existent.
- Suggested route: technical-writer or implementer

**F-AUD-D8-03** [P1] `module-decomposition.md:513` claims `prism-mcp ... 35 PrismServer, tool dispatch, resource/prompt handlers, config tool surface, health probe tools`
- Evidence: actual prism-mcp is 10 LoC. 35 tools is aspirational, not delivered.
- Suggested route: architect (split the table into "delivered" vs "planned")

**F-AUD-D8-04** [P1] CLAUDE.md preamble: "24-crate workspace" — accurate (verified `ls crates/ | wc -l` = 24).
- No drift.

**F-AUD-D8-05** [P1] ARCH-INDEX "deployment_topology: single-service" — claim aligned with intent but no service binary exists
- Suggested route: architect (annotate "planned")

**F-AUD-D8-06** [P1] ARCH-INDEX `version: "2.31"` and `status: draft` — top-level architecture claims draft status while many merged stories cite it as authoritative
- Why it matters: status enum drift across artifacts. Architecture is being treated as if it's the spec but its own frontmatter says draft.
- Suggested route: architect

**F-AUD-D8-07** [P1] `crates/prism-credentials/src/trait_.rs` doc comments retain "STUB — unimplemented!() in both backends" wording for trait methods that are now fully implemented in both `file.rs` and `keyring.rs`
- (Cross-reference F-AUD-D1-16)

**F-AUD-D8-08** [P1] `crates/prism-security/src/{content_hash,risk_tier}.rs` preambles say "All function bodies are `unimplemented!()`" — both files are fully implemented
- Cross-reference F-AUD-D1-16
- Suggested route: implementer (1-line doc fix per file)

**F-AUD-D8-09** [P1] BC-INDEX summary frontmatter says `active_contracts: 222` and changelog says "200 active Phase 1-2 BCs (BC-INDEX v4.25 active_contracts = 222 including Wave 3 additions)" — apparent count drift between v1.2 footnote (200) and v4.25 frontmatter (222)
- Evidence: `module-decomposition.md:546` footnote says "Grand total ... 200 active Phase 1-2 BCs"; BC-INDEX frontmatter says 222.
- Why it matters: cross-document arithmetic drift.
- Suggested route: architect / state-manager

---

## Findings by Severity

### P0 (production hazard, status:merged claim broken)

| ID | Dim | Title | Crate | File:Line | Story/BC | Evidence |
|---|---|---|---|---|---|---|
| F-AUD-D1-01 | 1 | `QueryEngine::execute` is `todo!()` | prism-query | engine.rs:276 | S-3.02 / BC-2.11.001 | `todo!("S-3.02 — QueryEngine::execute")` |
| F-AUD-D1-02 | 1 | `run_materialization_pipeline` is `todo!()` | prism-query | materialization.rs:241 | S-3.02 / BC-2.11.005 | `todo!("S-3.02 — run_materialization_pipeline")` |
| F-AUD-D1-03 | 1 | `RocksDbTableProvider::*` 4 methods + `register_internal_tables` are `todo!()` | prism-query | internal_tables.rs:125,129,139,146,168 | S-3.02 / BC-2.15.011 | 5 `todo!()` |
| F-AUD-D1-04 | 1 | No concrete `SensorAdapter::write()` override; default returns `WriteNotImplemented` | prism-sensors | adapter.rs:365 + auth/{crowdstrike,armis,cyberint,claroty}.rs | S-3.07 / BC-2.04.007 | `Err(SensorError::WriteNotImplemented{...})` |
| F-AUD-D1-05 | 1 | `WriteExecutor::execute` Phase 3 fetch is hardcoded `vec![]` | prism-query | write_pipeline.rs:349 | S-3.07 / AD-022 | `let fetched_records: Vec<RecordBatch> = vec![];` |
| F-AUD-D1-06 | 1 | SQL DML `insert_into`/`update`/`delete_from` return `NotImplemented` | prism-query | write_table_registration.rs:176,190,205 | S-3.07 / AD-022 | "S-3.07-pending: ... deferred to W3-FIX-S307-003" |
| F-AUD-D1-07 | 1 | `HotReloadWatcher::start/stop` `unimplemented!()` | prism-spec-engine | hot_reload.rs:66,72 | S-1.12 / AD-018 | `unimplemented!("Red Gate stub")` |
| F-AUD-D1-08 | 1 | Infusion source backends 100% `unimplemented!()` (mmdb, csv, json_lookup) | prism-spec-engine | infusion/sources/*.rs | S-1.14 / BC-2.19.001 | 9 `unimplemented!()` |
| F-AUD-D1-09 | 1 | `InfusionLruCache::get/insert` `unimplemented!()` | prism-spec-engine | infusion/cache.rs:109,120 | S-1.14 / BC-2.19.002 | 2 `unimplemented!()` |
| F-AUD-D1-10 | 1 | `InfusionLoader::*` 4 methods `unimplemented!()` | prism-spec-engine | infusion/loader.rs:42,50,57,66 | S-1.14 / BC-2.19.001 | 4 `unimplemented!()` |
| F-AUD-D1-11 | 1 | `infusion/plugin_bridge.rs` 2 methods `unimplemented!()` | prism-spec-engine | infusion/plugin_bridge.rs:26,37 | S-1.14↔S-1.15 | 2 `unimplemented!()` |
| F-AUD-D2-01 | 2 | STORY-INDEX is unreliable — 4 stories indexed MERGED have status:ready/draft in their frontmatter | (meta) | STORY-INDEX vs S-3.02/06/07/S-1.10 frontmatter | many | drift table above |
| F-AUD-D2-13 | 2 | `prism-mcp` is a 10-line stub — SS-10 undelivered; AD-005 unimplemented | prism-mcp | src/lib.rs (10 LoC total) | S-5.01-03 / SS-10 / AD-005 | `wc -l` = 10 |
| F-AUD-D3-01 | 3 | `test_BC_2_16_007_hot_reload_watcher_start_is_stub` is an inverted-polarity test that codifies the stub | prism-spec-engine | tests/hot_reload_tests.rs:1085 | S-1.12 | `#[should_panic(expected = "not yet implemented")]` |
| F-AUD-D4-01 | 4 | No production code path loads `sensors/*.sensor.toml` | (workspace) | sensors/{crowdstrike,armis,claroty,cyberint}.sensor.toml | (cross-cutting) | rg confirms zero non-spec-engine, non-fuzz callers |
| F-AUD-D4-02 | 4 | All 4 sensor TOMLs are write-only — no read-side `[[tables]]` | sensors/ | crowdstrike.sensor.toml:3 + 3 others | S-1.11 | `# Read-side tables: implement in S-1.11.` (not done) |
| F-AUD-D5-01 | 5 | All 222 active BCs are `status: draft` — no promotion mechanism in use | (meta) | BC-INDEX.md frontmatter | (all) | `active_contracts: 222` all draft |
| F-AUD-D7-01 | 7 | Only 2 of 145 VPs are `verified` (VP-014, VP-015); 143 are `draft` | (meta) | VP-INDEX.md | (all) | summary table |

P0 total: **18**

### P1 (significant gap, must fix before declaring waves complete)

| ID | Dim | Title | Crate | File:Line | Story/BC | Evidence |
|---|---|---|---|---|---|---|
| F-AUD-D1-12 | 1 | `pushdown::translate_push_down_filter` is `todo!("S-3.X")` | prism-query | pushdown.rs:199 | BC-2.11.007 | `todo!("S-3.X")` |
| F-AUD-D1-13 | 1 | `AdapterRegistry::adapters` field doc lies about being `todo!()` | prism-sensors | registry.rs:38 | S-3.1.06-ImplPhase | doc/code drift |
| F-AUD-D2-02..05 | 2 | S-1.11/12/14/15 stub-merged (covered by plugin audit) | prism-spec-engine | various | S-1.11..15 | (plugin audit) |
| F-AUD-D2-06 | 2 | S-3.02 STORY-INDEX MERGED but story:ready + 5 `todo!()` sites | prism-query | engine.rs, materialization.rs, internal_tables.rs | S-3.02 | covered above |
| F-AUD-D2-07 | 2 | S-3.07 STORY-INDEX MERGED but story:draft + 3 stub points | prism-query | write_pipeline.rs, write_table_registration.rs, prism-sensors/adapter.rs | S-3.07 | covered above |
| F-AUD-D2-08 | 2 | W3-FIX-S307-001/002/003 referenced in code but not filed in STORY-INDEX | prism-query, prism-sensors | various | S-3.07 follow-ups | rg confirms zero matches |
| F-AUD-D2-09 | 2 | `WriteEndpointRegistry::new()` only constructed in tests | prism-spec-engine | write_endpoint.rs / write_pipeline.rs | S-1.13 | TD-PLUGIN-P0-006 |
| F-AUD-D2-10 | 2 | S-3.05 has 6 `#[ignore = "TD-S305-XXX"]` tests | prism-query | tests/pagination_tests.rs, cache_tests.rs | S-3.05 | 6 sites |
| F-AUD-D2-11 | 2 | S-1.10 status:`delivered` (not `merged`) — enum drift | prism-security | story file | S-1.10 | unique enum value |
| F-AUD-D2-14 | 2 | spec-engine ↔ prism-query write registration chain has no binary consumer | prism-spec-engine, prism-query | various | S-1.13 / S-3.07 | rg analysis |
| F-AUD-D3-02 | 3 | `bc_3_2_002_org_id_namespace.rs` test docs say "MUST FAIL" but functions are real | prism-credentials | tests/bc_3_2_002_org_id_namespace.rs:6+6 sites | S-3.1.04 | doc/code drift |
| F-AUD-D3-03 | 3 | DTU-cyberint multi_tenant.rs docs claim 11 `todo!()` sites where functions are real | prism-dtu-cyberint | tests/multi_tenant.rs:29+ | S-3.2.04 | doc/code drift |
| F-AUD-D3-04 | 3 | 6 `#[ignore = "TD-S305-XXX"]` tests indicate S-3.05 incomplete | prism-query | tests/pagination_tests.rs, cache_tests.rs | S-3.05 | proper deferral; status drift |
| F-AUD-D3-05 | 3 | `test_bc_3_1_001_ac_1_from_uuid_panics_on_v4` doc claims test FAILS but it passes | prism-core | tests/bc_3_1_001_org_id.rs:85 | S-3.1.01 | doc/code polarity drift |
| F-AUD-D3-06 | 3 | DTU-crowdstrike `#[ignore = "needs-prism-audit"]` × 4 — VP-033/036 gated | prism-dtu-crowdstrike | tests/{harness_tests.rs,integration_vp03[36].rs} | S-6.07 | 4 `#[ignore]` |
| F-AUD-D3-07 | 3 | `proofs/infusion_spec.rs:13` admits Kani harness can't compile | prism-spec-engine | proofs/infusion_spec.rs:13 | VP-048 | own admission |
| F-AUD-D4-03 | 4 | `crates/prism-spec-engine/fixtures/*.infusion.toml` orphaned — loader is `unimplemented!()` | prism-spec-engine | fixtures/{geoip,asset_inventory,threat_intel_plugin}.infusion.toml | S-1.14 | F-AUD-D1-10 |
| F-AUD-D4-04 | 4 | `crates/prism-spec-engine/fixtures/*.{wasm,wat}` plugin fixtures unused in production | prism-spec-engine | fixtures/{noop_infusion,loop_plugin,trap_plugin}.* | S-1.15 | rg analysis |
| F-AUD-D4-05 | 4 | `crates/prism-dtu-demo-server/configs/{prism-demo,demo}.toml` scoped to demo only — discoverability risk | prism-dtu-demo-server | configs/*.toml | S-6.20 | rg analysis |
| F-AUD-D4-06 | 4 | No top-level `configs/` directory exists; S-5.05 (config-loading) draft | (workspace) | (absent) | S-5.05 | find returns nothing |
| F-AUD-D5-02..07 | 5 | BC postcondition gaps — query, write, hot-reload, plugin, infusion, MCP all unmet | various | various | various | covered above |
| F-AUD-D5-08, D5-09 | 5 | BC-2.05.001 audit comment drift; BC-2.07.005 cache_key proof comment drift | prism-audit, prism-query | audit_emitter.rs:342,374; proofs/vp025_cache_key.rs | S-2.04, S-3.05 | doc/code drift |
| F-AUD-D6-04 | 6 | ADR-014 file exists but missing from ARCH-INDEX ADR Registry | (meta) | ARCH-INDEX.md | (meta) | rg analysis |
| F-AUD-D6-05, D6-06 | 6 | AD-020 (Infusions) and AD-022 (PrismQL Write) accepted but partially/unimplemented | prism-spec-engine, prism-query | various | (cross-cutting) | covered above |
| F-AUD-D7-03 | 7 | VP-025 cache_key proof comments stale — production functions are real | prism-query | proofs/vp025_cache_key.rs:64,99 | VP-025 / S-3.05 | doc/code drift |
| F-AUD-D7-04 | 7 | VP-031 anchor S-3.02 is itself stubbed | prism-query | proofs/vp031_pushdown.rs | S-3.02 | F-AUD-D1-02 |
| F-AUD-D7-05 | 7 | 5 fuzz targets exist but VP-INDEX statuses are `draft` | (meta) | fuzz/fuzz_targets/ | VP-021/22/23/24/38 | status promotion criteria undefined |
| F-AUD-D7-06 | 7 | VP-033, VP-036 integration tests `#[ignore]` | prism-dtu-crowdstrike | integration_vp03[36].rs | VP-033, VP-036 | F-AUD-D3-06 |
| F-AUD-D8-02..06 | 8 | README stub, module-decomposition 35-tool aspirational, ARCH-INDEX status drift | (docs) | various | (meta) | covered above |
| F-AUD-D8-07, D8-08 | 8 | trait_.rs and content_hash/risk_tier.rs doc comments stale | prism-credentials, prism-security | various | S-1.06, S-1.09 | doc/code drift |
| F-AUD-D8-09 | 8 | BC-INDEX 222 vs module-decomposition 200 active-BC arithmetic drift | (meta) | BC-INDEX.md vs module-decomposition.md:546 | (meta) | count drift |

P1 total: **23**

### P2 (cleanup, can defer with explicit approval)

| ID | Dim | Title | Crate | File:Line | Notes |
|---|---|---|---|---|---|
| F-AUD-D1-14 | 1 | VP-040 Kani proof harness `unimplemented!()` (upstream API gate) | prism-spec-engine | proofs/plugin_linker.rs:95 | wait on wasmtime |
| F-AUD-D1-15 | 1 | `audit_emitter.rs:342, 374` retain `// todo!(...)` comments after impl | prism-audit | audit_emitter.rs:342,374 | residue, no behavior impact |
| F-AUD-D1-16 | 1 | Doc comments saying "STUB" in trait_.rs/content_hash.rs/risk_tier.rs | prism-credentials, prism-security | various | doc cleanup |
| F-AUD-D1-17 | 1 | `BehavioralClone` default `start_on`/`stop` panic — overridden in all clones | prism-dtu-common | clone.rs:79,93 | defensive refactor |
| F-AUD-D3-08 | 3 | `vp012_depth_limit.rs` 5 RED-gate proptest stubs may pass for wrong reason | prism-query | proofs/vp012_depth_limit.rs:48,69,76,84,89 | verify and clean |
| F-AUD-D3-09 | 3 | `vp013_cycle_detection.rs` 5 RED-gate sites | prism-query | proofs/vp013_cycle_detection.rs | verify and clean |
| F-AUD-D4-07 | 4 | Two parallel SensorSpec types in spec-engine | prism-spec-engine | spec_parser.rs:189 vs types.rs | TD-PLUGIN-P0-005 |
| F-AUD-D7-07 | 7 | VP-INDEX summary table arithmetic drift through history | (meta) | VP-INDEX.md changelog | audit count |
| F-AUD-D8-04 | 8 | CLAUDE.md "24-crate workspace" — accurate | (meta) | CLAUDE.md | clean |
| F-AUD-D8-05 | 8 | ARCH-INDEX `deployment_topology: single-service` — no service binary | (meta) | ARCH-INDEX.md | annotate planned |
| (cleanup of stale `#[should_panic]` test suites) | 3 | bc_3_1_001 doc fix; multi_tenant.rs doc sweep; bc_3_2_002 doc sweep | various | various | doc-only |
| (cleanup of `panic!(` literal strings in error helpers) | 1 | OrgSlug::unwrap, dispatch:73,92, cache:1171,1187 — internal-only contract panics | prism-core, prism-query | various | by-design |

P2 total: **12**

---

## Cross-Pattern Analysis

### Recurring Root Causes

1. **"Stub-Merge" convention enforced inconsistently.** Stories merge with `todo!()` panics, structural-only types, or trait defaults that error. Some stories (S-3.05) properly tag deferrals via `#[ignore = "TD-..."]`; others (S-1.12, S-3.02, S-3.07) just merge with the panic in place and the story file frontmatter still saying `draft`/`ready`.

2. **STORY-INDEX as a manual log instead of a status oracle.** Free-form text edits (e.g. "MERGED PR #129 6fefc774 2026-05-06 +491t 4-adv-passes-post-rebase 19-findings-closed") are appended after the story title in the second column. The frontmatter status field of the story file stays unchanged. Two systems of truth, no reconciliation hook.

3. **No graduation contract on BC/VP status.** All 222 BCs are `draft`; only 2 of 145 VPs are `verified`. No defined criteria for status promotion. This means BC/VP indexes provide zero signal about implementation reality.

4. **Production wiring debt — no binary loads anything.** `parse_spec_directory`, `ConfigManager::new`, `InfusionRegistry::new`, `register_internal_tables`, `WriteEndpointRegistry::new` are all real (or partially real) functions, but **no binary in the workspace constructs them**. There is no `prism-bin` crate yet. The MCP server (the supposed runtime) is a 10-line stub.

5. **Doc-comment freeze after stub.** Many crates (prism-credentials/trait_.rs, prism-security/content_hash.rs, prism-security/risk_tier.rs, prism-audit/audit_emitter.rs, prism-spec-engine/infusion/mod.rs) carry doc-comments saying "STUB — todo!() pending" even though the code is real. This is residue from the stub-architect-then-implementer phase, never cleaned post-implementation.

6. **Inverted-polarity tests.** At least one test (`test_BC_2_16_007_hot_reload_watcher_start_is_stub`) actively codifies the stub as the contract — its `#[should_panic(expected = "not yet implemented")]` would BREAK the day someone implements it. This is the silent-shallow problem at maximum strength: the test is green specifically because the prod code is broken.

### Stories Most-Affected

| Story | Issues | Note |
|---|---|---|
| S-1.12 | F-AUD-D1-07, D2-03, D3-01, D5-04, D6-03 | hot reload watcher unimplemented; inverted-polarity test |
| S-1.14 | F-AUD-D1-08, D1-09, D1-10, D1-11, D2-04, D5-06, D6-05 | infusion 100% stub |
| S-1.15 | F-AUD-D1-14, D2-05 | runtime real but no production wiring |
| S-3.02 | F-AUD-D1-01, D1-02, D1-03, D2-06, D5-02, D7-04 | query engine entirely `todo!()` |
| S-3.07 | F-AUD-D1-04, D1-05, D1-06, D2-07, D5-03, D6-06 | write path Phase 3 fetch empty + no concrete adapter overrides + SQL DML NotImplemented |
| SS-10 / S-5.01-03 | F-AUD-D2-13, D5-07, D6-01 | prism-mcp 10-line stub |
| S-1.11 | F-AUD-D2-02, D4-01, D4-02 | sensor TOMLs orphaned + read-side missing |

### Subsystems Most-Affected

| Subsystem | Crate | P0 count | Note |
|---|---|---|---|
| SS-11 (Query Execution) | prism-query | 6 | engine, materialization, internal-tables, push-down, write-pipeline, write-table-registration |
| SS-16 (Spec Engine) | prism-spec-engine | 5 | hot_reload, infusion (4 modules) |
| SS-19 (Infusions) | prism-spec-engine/infusion | 5 | loader, cache, sources × 3 |
| SS-01 (Sensor Adapters) | prism-sensors | 1 | default `write()` returns NotImplemented |
| SS-10 (MCP Interface) | prism-mcp | (whole crate) | 10-line stub |
| SS-17 (WASM Plugin Runtime) | prism-spec-engine/plugin | partial | runtime real, no production wiring; action dispatch stubbed |

### Comparison to Plugin Audit Findings

The prior plugin-system audit (2026-05-08, same date) found 14 P0/P1 deferrals. This audit re-confirms all 14 and adds **substantially new** findings outside the plugin axis:

| Plugin audit finding | Re-confirmed | Notes |
|---|---|---|
| TD-PLUGIN-P0-001 (PipelineExecutor::execute stub) | yes | F-AUD-D2-02 |
| TD-PLUGIN-P0-002 (Infusion 100% unimplemented) | yes | F-AUD-D1-08/09/10/11 |
| TD-PLUGIN-P0-003 (HotReloadWatcher unimplemented) | yes | F-AUD-D1-07 |
| TD-PLUGIN-P0-004 (sensor TOMLs write-only) | yes | F-AUD-D4-02 |
| TD-PLUGIN-P0-005 (parallel SensorSpec types) | yes | F-AUD-D4-07 |
| TD-PLUGIN-P0-006 (WriteEndpointRegistry::new test-only) | yes | F-AUD-D2-09 |
| TD-PLUGIN-P0-007 (no binary loads sensor TOMLs) | yes | F-AUD-D4-01 |
| TD-PLUGIN-P0-008 (action-plugin dispatch stubbed) | yes | recorded |

**New findings beyond plugin axis (substantive expansions):**

- **Query engine is stub** (F-AUD-D1-01/02/03) — STORY-INDEX claims S-3.02 is merged; reality is `todo!()`. Five separate `todo!()` sites in `engine.rs`/`materialization.rs`/`internal_tables.rs`.
- **Write execution is stub** (F-AUD-D1-04/05/06) — STORY-INDEX claims S-3.07 is merged 2026-05-08; reality is empty fetch + no adapter overrides + NotImplemented SQL DML.
- **MCP runtime is a 10-line stub** (F-AUD-D2-13) — AD-005 (rmcp 1.4) claims merged-architecture-decision; the entire crate is a placeholder.
- **STORY-INDEX is unreliable** (F-AUD-D2-01) — 4+ stories are indexed MERGED with story-file frontmatter saying ready/draft. The "stub-merge" pattern is not detectable from the index alone.
- **BC/VP graduation is broken** (F-AUD-D5-01, D7-01) — 222/222 BCs are draft; 143/145 VPs are draft. The status taxonomy is unused.
- **Inverted-polarity tests exist** (F-AUD-D3-01) — at least one test actively codifies the stub.
- **Doc-comment freeze** (F-AUD-D1-13/15/16, D8-07/08) — multiple crates have stale "STUB" comments where code is real.

In summary: the plugin audit was the tip of the iceberg. The same anti-pattern appears in the query engine, the write engine, and the MCP runtime — three of the four most central subsystems.

---

## Recommendations (DO NOT ACT — for user triage)

### Coordinated Fix Bundles

**Bundle A: Status-taxonomy reform (state-manager + architect, no code work).** Address F-AUD-D2-01, D5-01, D7-01, D8-09 together. Define and enforce: (a) `status: partial-merge` enum on stories; (b) BC promotion criteria (`draft → active → verified`); (c) reconcile STORY-INDEX text annotations vs. story-file frontmatter via a hook. Requires architect + state-manager. **Single artifact: `factory-policies.yaml` policy POL-N "Story Status Reconciliation".**

**Bundle B: Production runtime gap (architect + story-writer + multiple implementers).** Address F-AUD-D1-01..06, D2-13, D6-01..03 together. The headline issue: there is no binary that constructs `ConfigManager`, `QueryEngine`, `WriteExecutor`, or an MCP server. Bundle:
1. File `prism-bin` crate spec (`S-WAVE5-PREP-01`).
2. Write `S-3.02-FOLLOWUP-RUNTIME` to fill `QueryEngine::execute`, `run_materialization_pipeline`, internal-tables provider.
3. Write `W3-FIX-S307-001/002/003` story files (currently only code comments).
4. Write `S-1.12-FOLLOWUP` (notify watcher).
5. Write `S-1.14-REDO` or formally retire infusions to Wave 4+.
6. File `S-5.01-FOLLOWUP-MCP-BOOT` for prism-mcp resurrection.

**Bundle C: Sensor TOML completion + spec-engine wire-up (story-writer + implementer).** Address F-AUD-D4-01..04 together. (a) Backfill 4 sensor TOMLs with read-side `[[tables]]/[[tables.steps]]` per `examples/demo_spec_loading.rs:36-79`. (b) Wire `init_registry_for_org` to consume parsed sensor specs instead of hardcoded Rust adapters. (c) Decide whether to retire `crates/prism-spec-engine/fixtures/*.{infusion.toml, wasm, wat}` or build the loader.

**Bundle D: Doc cleanup sweep (technical-writer pass).** Address F-AUD-D1-13/15/16, D3-02/03/05, D8-01/02/03/05/07/08 together. Single PR sweeping ~30 stale "STUB" / "todo!()" doc-comments where code is real. Includes README.md backfill (F-AUD-D8-02).

**Bundle E: VP graduation (architect).** Address F-AUD-D7-03/04/05 together. (a) Define fuzz-VP promotion criteria (corpus size, CI evidence). (b) Re-run Kani for VP-025 (cache_key.rs is implemented). (c) Promote 5 fuzz VPs (021/022/023/024/038). (d) Audit 145-row count (F-AUD-D7-07).

### Suggested Epic / Wave Structure for Cleanup

- **Epic E-CLEANUP-01: Status Reconciliation** — Bundle A. ~1 week. Pre-requisite for anything else.
- **Epic E-CLEANUP-02: Runtime Reality** — Bundle B. ~3-6 weeks. Must complete before declaring Wave 1/Wave 3 actually merged.
- **Epic E-CLEANUP-03: Spec-Engine Wire-Up** — Bundle C. ~2 weeks.
- **Epic E-CLEANUP-04: Doc Sweep** — Bundle D. ~1 week (parallel-safe).
- **Epic E-CLEANUP-05: VP Graduation** — Bundle E. ~1-2 weeks (parallel-safe with E-CLEANUP-04).

### Stories That Should Be Re-Opened (status: merged → status: partial-merge once that enum lands)

| Story | Current Status | Recommended | Reason |
|---|---|---|---|
| S-1.11 | merged | partial-merge | TD-PLUGIN-P0-001 (PipelineExecutor stub) |
| S-1.12 | merged | partial-merge | F-AUD-D1-07 (HotReloadWatcher unimplemented) |
| S-1.14 | merged | partial-merge or `retired` | F-AUD-D1-08..11 (infusion 100% stub) |
| S-1.15 | merged | partial-merge | TD-PLUGIN-P0-008 (action dispatch stubbed); no production wiring |
| S-3.02 | ready (file) / MERGED (index) | reconcile to partial-merge | F-AUD-D1-01..03 |
| S-3.05 | draft | (already accurate) | properly deferred via `#[ignore]` |
| S-3.06 | ready (file) / MERGED (index) | reconcile to merged | parser is real |
| S-3.07 | draft (file) / MERGED (index) | reconcile to partial-merge | F-AUD-D1-04..06 |

### Policy Gaps That Allowed These Patterns Through

1. **No "merged" gate test.** A pre-merge hook should `rg 'todo!\(|unimplemented!\(' src/ | grep -v test` on the changed-files set; non-zero exit on a story's PR closes the story without filing follow-ups.
2. **No story-file ↔ STORY-INDEX consistency check.** A simple grep that compares `status:` in each file to the second column of STORY-INDEX should be added to `lefthook.yml` or `factory-cycles-bootstrap`.
3. **No BC/VP promotion enforcement.** If a story claims to satisfy BC-X, BC-X status should be auto-promoted to `active` upon merge — and CI should fail if any BC remains `draft` after its anchor story merges.
4. **No "is the runtime wired?" check.** A repo-level audit should verify each accepted ADR with crate-level deliverables has at least one production binary calling its entry point. (E.g. AD-005 → `prism-mcp::Server::new` should be called somewhere in `bin/`.)
5. **No inverted-polarity-test linter.** `#[should_panic(expected = "not yet implemented")]` in production tests should fail CI; it can only exist transiently during stub-architect phase and must be retired by the implementer.

---

## Appendix: Out-of-Scope Findings (recorded for completeness)

- **`mutants.out/` and `mutants.out.old/`** — present at workspace root; should be in `.gitignore` if not already (didn't verify gitignore).
- **`.references/` repos** — not audited; large vendored references.
- **Wave 4 stories (S-4.x)** — all `status: draft`; no implementation expected; not flagged.
- **Wave 5/6 stories** — all `status: draft` except S-6.06–S-6.20 (DTU clones merged).
- **Compile-fail crate at `tests/external/perimeter-violation/`** — not inspected.
