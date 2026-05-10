---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T12:00:00
phase: 5
pass: 1
previous_review: null
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/cycles/wave-4-operations/audits/plugin-only-violations-2026-05-10.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/domain-spec/invariants.md"
  - ".factory/policies.yaml"
  - "crates/prism-spec-engine/src/custom_adapter.rs"
  - "crates/prism-spec-engine/src/plugin/host_functions.rs"
  - "crates/prism-spec-engine/src/plugin/loader.rs"
  - "crates/prism-spec-engine/src/spec_parser.rs"
input-hash: "[live-state]"
target_artifact: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
target_artifact_sha_at_review: "1a915530"
findings_total: 26
findings_by_tier:
  CRIT: 4
  HIGH: 9
  MED: 7
  LOW: 4
  OBS: 5
process_gap_findings: 5
convergence_status: NOT_CLEAN
fix_burst_required: true
related_tasks: [94, 95]
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 1)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix — `W4OPS` (wave-4-operations)
- `<PASS>`: Two-digit pass number — `P01`
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

This is pass 1 of ADR-023 adversarial review — all findings are new (Part A not applicable).

---

## Part A — Fix Verification (pass >= 2 only)

Not applicable — this is pass 1.

---

## Part B — New Findings (or all findings for pass 1)

26 findings total: 4 CRIT / 9 HIGH / 7 MED / 4 LOW / 5 OBS.

ADR is structurally sound but requires major revision before pass-2. It correctly identifies the plugin-only architectural mandate and provides a reasonable migration scaffold. However, it has critical contract coherence gaps: it retires a Rust trait and un-seals an auth abstraction without amending the behavioral contracts and domain invariants that currently mandate those constructs. It also contains verification properties that reference types and crates that do not exist in the codebase, and a migration wave ordering that creates a broken-develop window.

### CRITICAL

#### ADV-W4OPS-P01-CRIT-001: ADR retires CustomAdapter trait but does not amend the rust-escape-hatch behavioral contract that mandates it

- **Severity:** CRITICAL
- **Category:** contradictions
- **Location:** ADR-023 §C Rule 5; the rust-escape-hatch behavioral contract (`BC-2.16.004-rust-escape-hatch.md`)
- **Description:** ADR-023 Rule 5 states "Retire CustomAdapter Rust trait — `.prx` WASM is the sole escape hatch." The rust-escape-hatch behavioral contract currently mandates that `CustomAdapter` be available as a compile-time extensibility mechanism with specific `DataSourceAdapter` trait bounds. The ADR body contains no `amends_bcs:` frontmatter field and no "Retired/Amended Contracts" section documenting the retirement.
- **Evidence:** The behavioral contract is the behavioral ground truth per the VSDD hierarchy. An ADR that contradicts it without amending it creates a split-brain specification. An implementer dispatched to execute Wave 0/E (S-PLUGIN-PREREQ-E) will read the rust-escape-hatch behavioral contract, find it mandates a trait that this ADR retires, and have no authoritative guidance on which document wins.
- **Proposed Fix:** Add `amends_bcs: [BC-2.16.004]` and `retires_bcs: [BC-2.16.004]` to ADR-023 frontmatter. Add a "Retired and Amended Contracts" section to the ADR body documenting the retirement with effective date and rationale. The rust-escape-hatch behavioral contract must be updated (lifecycle_status: deprecated, deprecated_by: ADR-023) before Wave 0/E can dispatch.

#### ADV-W4OPS-P01-CRIT-002: ADR un-seals SensorAuth but does not amend the sealed-auth-trait domain invariant

- **Severity:** CRITICAL
- **Category:** contradictions
- **Location:** ADR-023 §C Rule 2; the sealed-auth-trait domain invariant (invariants.md, INV-AUTH-SEALED)
- **Description:** ADR-023 Rule 2 states "Un-seal SensorAuth entirely; remove `private::Sealed` marker." The sealed-auth-trait domain invariant in invariants.md mandates that `SensorAuth` implementations must not be constructible outside the `prism-sensors` crate, enforced via the `private::Sealed` marker trait. The ADR contains no `amends_dis:` frontmatter field and no documentation of this invariant's downgrade from compile-time to runtime enforcement.
- **Evidence:** Any static-analysis or compliance tool checking invariants.md will flag a SensorAuth un-sealing as an invariant violation. Security review agents checking the un-sealing PR against the domain invariant will BLOCK it.
- **Proposed Fix:** Add `amends_dis: [INV-AUTH-SEALED]` to ADR-023 frontmatter. Document the downgrade from compile-time to runtime invariant enforcement. Specify what runtime validation rules replace the sealed trait (e.g., which cross-sensor auth-composition patterns are explicitly rejected by the runtime registry and why).

#### ADV-W4OPS-P01-CRIT-003: Forbidden Patterns enforcement cites a crate that does not exist

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** ADR-023 §D "Forbidden Patterns" enforcement section
- **Description:** The Forbidden Patterns section includes a lint rule citing `prism-operations` as a crate to scan for violations. Enumeration of the workspace via `cargo metadata` shows no `prism-operations` crate. The actual crates containing sensor dispatch sites are `prism-sensors`, `prism-query`, `prism-ocsf`, and `prism-bin`.
- **Evidence:** A CI enforcement step scanning `prism-operations` will silently succeed (no files to scan → zero violations found) while the real violation sites documented in `plugin-only-violations-2026-05-10.md` go unchecked. Silent CI false-pass.
- **Proposed Fix:** Replace the nonexistent crate reference with an explicit list of actual crate names derived from `cargo metadata`. Minimum crate list for sensor-dispatch enforcement: `prism-sensors`, `prism-query`, `prism-ocsf`, `prism-spec-engine`, `prism-bin`.

#### ADV-W4OPS-P01-CRIT-004: Forbidden Patterns require a PR template that does not exist

- **Severity:** CRITICAL
- **Category:** missing-edge-cases
- **Location:** ADR-023 §D "Forbidden Patterns" enforcement section
- **Description:** The Forbidden Patterns section requires that any PR touching sensor code include a completed "plugin-migration-checklist" PR template. No such template exists at `.github/PULL_REQUEST_TEMPLATE/plugin-migration-checklist.md` (verified by directory listing).
- **Evidence:** The first PR in Wave 1 (PLUGIN-MIGRATION-001-A) cannot comply with the stated requirement. Either the pr-manager skips the checklist (silent non-compliance) or the PR is blocked on an undefined deliverable.
- **Proposed Fix:** Move PR template creation to Wave 0/F as an explicit deliverable. The ADR's Forbidden Patterns section should reference the template by its target path and note that it is created in Wave 0/F before any Wave 1 PR can proceed.

---

### HIGH

#### ADV-W4OPS-P01-HIGH-001: OCSF Hybrid 80/20 boundary not testable as specified

- **Severity:** HIGH
- **Category:** verification-gaps
- **Location:** ADR-023 §C Rule 1 (OCSF hybrid TOML/WASM boundary)
- **Description:** The 80/20 split (TOML `ocsf_field` column for 80% of fields, in-repo `.prx` WASM transformers for 20% complex cases) is stated as an architectural principle but has no closed-grammar definition for what constitutes an `ocsf_field` value, no machine-readable catalog of which fields are TOML-expressible vs WASM-required, and no verification property governing the boundary.
- **Evidence:** A WASM author has no principled way to decide which fields belong in TOML vs in the transformer. No VP currently governs the boundary.
- **Proposed Fix:** Add a closed BNF/PEG grammar for the `ocsf_field` column syntax. Add a WASM-required catalog specifying the OCSF field patterns that cannot be expressed in TOML `ocsf_field`. Add a verification property that validates no WASM transformer handles a mapping that could be expressed in `ocsf_field`.

#### ADV-W4OPS-P01-HIGH-002: Plugin signing infrastructure entirely unspecified

- **Severity:** HIGH
- **Category:** security-surface
- **Location:** ADR-023 §E Prerequisites (PREREQ-D)
- **Description:** PREREQ-D lists "`.prx` build/sign/load pipeline" as a prerequisite but provides no specification of the signing infrastructure: no key management model, no trust anchor, no signature scheme, no verification step in the loader, no revocation mechanism, no threat model for unsigned plugins.
- **Evidence:** A developer implementing S-PLUGIN-PREREQ-D cannot implement signing from this specification.
- **Proposed Fix (per user decision 2026-05-10):** Defer signing to v1.0+1. Remove signing from PREREQ-D scope for v1.0. Add boot warning requirement: `prism-bin` MUST emit a WARN-level log at startup and MUST write an audit log entry recording the unsigned-plugin security posture. Track as TD-PLUGIN-SIGNING-001 (P0, v1.0+1). Add v1.0 security exposure to Negative Consequences.

#### ADV-W4OPS-P01-HIGH-003: Verification property `VP-PLUGIN-001` names types that do not exist in the codebase

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** ADR-023 §F Verification Properties section
- **Description:** The verification property for plugin migration references `CrowdStrikeAdapter` as a type that must NOT appear in the compiled binary. The actual type in `crates/prism-spec-engine/src/custom_adapter.rs` is `CrowdStrikeAuth` (not `CrowdStrikeAdapter`). Similarly for `ArmisAdapter` (actual: `ArmisAuth`), and analogously for Claroty and Cyberint.
- **Evidence:** A Kani proof or grep-based CI check using the names in the verification property will pass vacuously because those type names do not exist and were never compiled in.
- **Proposed Fix:** Reconcile the verification property's banned-symbol list against actual codebase types confirmed by `cargo metadata` and source grep. Correct type names: `CrowdStrikeAuth`, `ArmisAuth`, `ClarotyAuth`, `CyberintAuth`. Also verify that `init_registry_for_org` is correctly cited.

#### ADV-W4OPS-P01-HIGH-004: `VP-PLUGIN-003` byte-level parity self-contradicts the ADR architecture

- **Severity:** HIGH
- **Category:** verification-gaps
- **Location:** ADR-023 §F Verification Properties section, `VP-PLUGIN-003`
- **Description:** The verification property requires "byte-level output parity" between TOML-driven sensor specs and legacy Rust adapter outputs for DTU-parity tests. This is unachievable: TOML-driven specs go through a different execution path than Rust adapters; timestamp precision, field ordering, and null-handling may legitimately differ; the OCSF 80/20 hybrid means some fields are computed by WASM transformers that produce semantically equivalent but not byte-identical output.
- **Evidence:** Requiring byte-level parity makes the DTU test impossible to pass without byte-for-byte reimplementing the legacy Rust adapter behavior, defeating the purpose of the migration.
- **Proposed Fix:** Replace "byte-level parity" with "schema-identical output (same OCSF fields present), row-count within 5% tolerance, and canonical-projection value parity (all fields in the canonical OCSF projection have equal values after timestamp normalization to second precision)."

#### ADV-W4OPS-P01-HIGH-005: `VP-PLUGIN-005` sandbox model contradicts existing host function capability

- **Severity:** HIGH
- **Category:** contradictions
- **Location:** ADR-023 §F Verification Properties section; `crates/prism-spec-engine/src/plugin/host_functions.rs`
- **Description:** The verification property describes a WASM sandbox model that prohibits network access from plugins. However, `host_functions.rs` already implements `host_http_request` — a host function that intentionally allows plugins to make HTTP requests through the host runtime.
- **Evidence:** The sandbox model directly contradicts the existing plugin capability model. A security reviewer reading this VP against the actual host_functions.rs will flag an apparent security regression that does not exist.
- **Proposed Fix:** Rewrite the sandbox section to align with the existing `host_http_request` allowlist model: plugins MAY make HTTP requests through the declared `host_http_request` host function (subject to sensor-spec-declared `allowed_hosts` allowlist). Direct WASI network syscalls are prohibited. The sandbox property is: all network I/O must flow through the declared host function interface.

#### ADV-W4OPS-P01-HIGH-006: C3 grammar extensions overstate the new work required

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** ADR-023 §C Rule 3 (TOML grammar extensions) / PREREQ-C story estimate
- **Description:** ADR-023 C3 lists `batch`, `retry`, `two-step-fetch`, `ocsf_field`, `cache_ttl`, and `table_name` as new TOML grammar extensions. Examination of `crates/prism-spec-engine/src/spec_parser.rs` shows that `retry`, `cache_ttl`, and `table_name` are already parsed. Only `batch`, `two-step-fetch`, and `ocsf_field` are genuinely new.
- **Evidence:** The story point estimate for S-PLUGIN-PREREQ-C (5-8 SP) was set assuming all 6 extensions are new work. This overestimates the scope and may cause mis-sequenced dependencies on non-existent gaps.
- **Proposed Fix:** Re-author C3 with an explicit NEW vs already-present split. Mark `retry`/`cache_ttl`/`table_name` as "already present in spec_parser.rs — verify behavior matches spec." Mark `batch`/`two-step-fetch`/`ocsf_field` as new work. Revise S-PLUGIN-PREREQ-C story estimate to reflect only new grammar work (likely 2-4 SP, not 5-8 SP).

#### ADV-W4OPS-P01-HIGH-007: Wave 1 ordering creates a broken-develop window

- **Severity:** HIGH
- **Category:** missing-edge-cases
- **Location:** ADR-023 §G Migration Plan, Wave 1 ordering
- **Description:** The migration plan orders Wave 1 as: A (delete 4 named Rust auth modules) → B (convert prism-query dispatch sites) → C (merge OCSF mappers) → D (author 4 production TOMLs) → E (CrowdStrike OAuth2 WASM plugin). This deletes the Rust adapters before the replacement TOML specs + plugins are authored and parity-tested. Between Wave 1/A merge and Wave 1/D merge, develop has no functioning sensor adapters.
- **Evidence:** CrowdStrike, Armis, Claroty, and Cyberint are all non-functional in develop for the duration between those two PRs. Any integration test or demo run during this window will fail.
- **Proposed Fix (per user decision 2026-05-10):** Reorder Wave 1: D → E → A → B → C. At every PR boundary in Wave 1, all four sensors must remain functional via either the legacy Rust path OR the new TOML+plugin path. The cutover commit (Wave 1/A) deletes Rust adapters only after DTU-parity tests pass for all four sensors.

#### ADV-W4OPS-P01-HIGH-008: Negative Consequences omit six categories of real risk

- **Severity:** HIGH
- **Category:** missing-edge-cases
- **Location:** ADR-023 §H Negative Consequences
- **Description:** The Negative Consequences section lists 3 items (complexity, performance, vendor lock-in). It omits: (1) cold-start latency for WASM plugin loading at boot; (2) observability gap — WASM stack traces are not human-readable in current Wasmtime configurations; (3) rollback difficulty — once Rust adapters are deleted, rolling back requires reverting multiple merged PRs; (4) version-skew risk — WASM ABI between `prism-spec-engine` host and `.prx` plugin; (5) debugging complexity — stepping into WASM is not supported by standard Rust tooling; (6) panic isolation — a WASM plugin panic currently propagates to the host runtime as a Wasmtime trap.
- **Evidence:** An architect or future decision-maker reading this ADR has an incomplete picture of the trade-offs accepted.
- **Proposed Fix:** Expand Negative Consequences with all 6 omitted items plus the unsigned-plugin security exposure from HIGH-002 user decision. Each should include a one-line mitigation or acceptance rationale.

#### ADV-W4OPS-P01-HIGH-009: Amendment to the production-runtime-wiring decision is prose-only; that decision body remains unchanged

- **Severity:** HIGH
- **Category:** contradictions
- **Location:** ADR-023 §G Story 3 / the production-runtime-wiring decision record
- **Description:** ADR-023 §G Story 3 documents that the production-runtime-wiring decision's §G Story 3 (PluginRuntime wiring) is superseded by ADR-023. However, the production-runtime-wiring decision itself has not been amended — it still describes Story 3 as if ADR-023 did not exist. Cross-document navigation from the superseded document to the superseding document is broken.
- **Evidence:** A developer reading the production-runtime-wiring decision record gets no indication that its Story 3 has been superseded. The production-runtime-wiring §G Story 3 will be implemented twice — once per its own spec, once per ADR-023 — unless the supersession is visible in that document.
- **Proposed Fix:** Schedule the production-runtime-wiring decision for a v1.2 amendment during the same wave-gate burst as ADR-023's status flip from PROPOSED to COMMITTED. Add `superseded_by_partial: ADR-023 (§G Story 3 only)` to the production-runtime-wiring frontmatter and an inline note directing readers to ADR-023.

---

### MEDIUM

#### ADV-W4OPS-P01-MED-001: Rule 5 conflicts with existing consumer code in spec_parser.rs

- **Severity:** MEDIUM
- **Category:** contradictions
- **Location:** ADR-023 §C Rule 5; `crates/prism-spec-engine/src/spec_parser.rs`
- **Description:** `spec_parser.rs` contains call sites that instantiate `CustomAdapterRegistry` and invoke `custom_adapter.rs` exports. Rule 5 retires the `CustomAdapter` trait but provides no transition guidance for these call sites. If Wave 0/E deletes `custom_adapter.rs` without first migrating these call sites, the crate fails to compile.
- **Evidence:** Direct grep of `spec_parser.rs` confirms `CustomAdapterRegistry` references.
- **Proposed Fix (per user decision 2026-05-10):** Confirm Rule 5. ADR Rule 5 stays. Add a note to Wave 0/E (S-PLUGIN-PREREQ-E) story scope: "Migrate `spec_parser.rs` `CustomAdapterRegistry` call sites to `PluginRegistry` before deleting `custom_adapter.rs`."

#### ADV-W4OPS-P01-MED-002: ADR claims DTU clones exist for all 4 sensors but Cyberint DTU is incomplete

- **Severity:** MEDIUM
- **Category:** verification-gaps
- **Location:** ADR-023 §E Prerequisites / PREREQ-B
- **Description:** PREREQ-B states DTU clones exist for all four sensors and parity tests can be run against them. The DTU assessment notes that the Cyberint DTU clone in `prism-dtu-cyberint` has known gaps in its API coverage (specifically the `incidents` endpoint pagination behavior). Authoring TOML specs with DTU-parity tests against an incomplete clone may produce false-passing parity tests.
- **Evidence:** DTU assessment document confirms Cyberint clone completeness gap.
- **Proposed Fix:** Annotate PREREQ-B with the known Cyberint DTU gap. Add a task to Wave 1/D: "Verify Cyberint DTU clone covers `incidents` endpoint pagination before authoring parity test."

#### ADV-W4OPS-P01-MED-003: `host_functions.rs:30` import list will drift from WASM ABI as host functions evolve

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** `crates/prism-spec-engine/src/plugin/host_functions.rs:30`
- **Description:** The host function import list is manually maintained. As new host functions are added (e.g., `host_metrics_emit` planned in PREREQ-D), the import list must be manually synchronized with the WASM ABI. There is no generated or validated source of truth; drift produces a link-time error caught late in the build cycle.
- **Evidence:** Examination of `host_functions.rs` confirms manual maintenance pattern without a synchronization assertion.
- **Proposed Fix:** Add a build-time assertion or `#[cfg(test)]` test that enumerates declared host functions against the `wasmtime::Linker` registration list. This is PREREQ-D scope.

#### ADV-W4OPS-P01-MED-004: Wave 0/A estimate does not account for proc-macro-generated dispatch sites

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** ADR-023 §G Wave 0, Story A estimate (13-18 SP)
- **Description:** S-PLUGIN-PREREQ-A migrates `SensorType` (a closed enum) to `SensorId(Arc<str>)` (an open string type). The estimate does not account for macro-generated code in `prism-query` that pattern-matches on `SensorType` variants. Any proc-macro or `strum`-derived implementation will produce compile errors not caught by a simple grep.
- **Evidence:** `SensorType` uses `strum` derives in `prism-core`. Any `strum`-generated match code is not visible to a line-level grep.
- **Proposed Fix:** Add a pre-implementation task to S-PLUGIN-PREREQ-A: "Enumerate all pattern-match sites using `SensorType` including proc-macro generated code via `cargo expand`." Revise SP estimate after enumeration.

#### ADV-W4OPS-P01-MED-005: No deprecation notice planned for `custom_adapter.rs` during the interim period

- **Severity:** MEDIUM
- **Category:** missing-edge-cases
- **Location:** ADR-023 §G Wave 0, Story E; `crates/prism-spec-engine/src/custom_adapter.rs`
- **Description:** The migration plan schedules deletion of `custom_adapter.rs` in Wave 0/E, potentially weeks after ADR-023 is committed. Any external consumer who reads the code during this window gets no signal that the trait is being retired.
- **Evidence:** ADR-023 may reach COMMITTED status well before Wave 0/E dispatches, creating a window where the code contradicts the committed decision record.
- **Proposed Fix:** Add a task to Wave 0/F: add `#[deprecated(since = "next", note = "Use .prx WASM plugins instead")]` to the `CustomAdapter` trait definition immediately when ADR-023 moves to COMMITTED status.

#### ADV-W4OPS-P01-MED-006: `loader.rs` WASM validation does not verify plugin manifest format version

- **Severity:** MEDIUM
- **Category:** security-surface
- **Location:** `crates/prism-spec-engine/src/plugin/loader.rs`
- **Description:** The plugin loader validates WASM binaries (magic bytes, exports) but does not verify the plugin manifest's `format_version` field. A plugin built against a future manifest format will load without error and potentially exhibit undefined behavior at runtime.
- **Evidence:** Examination of `loader.rs` confirms no `format_version` check in the validation path.
- **Proposed Fix:** Add manifest `format_version` validation to the loader. Reject plugins with `format_version > CURRENT_SUPPORTED_VERSION` with a clear error. Track the supported version as a crate constant. This is PREREQ-D scope.

#### ADV-W4OPS-P01-MED-007: Wave 0 has no story for behavioral contract and domain-invariant amendments before code changes begin

- **Severity:** MEDIUM
- **Category:** missing-story
- **Location:** ADR-023 §G Wave 0 migration plan
- **Description:** Wave 0 dispatches implementation stories (SensorId migration, PluginRuntime wiring, SensorAuth un-sealing) without first amending the behavioral contracts and domain invariants that govern the pre-migration architecture. An implementer executing Wave 0/E without the amendment story having landed will be implementing against contradictory behavioral contracts.
- **Evidence:** The rust-escape-hatch behavioral contract (CRIT-001), the sealed-auth-trait domain invariant (CRIT-002), and the datasource-trait-adapter-pattern behavioral contract all remain authoritative until explicitly amended.
- **Proposed Fix (per user decision 2026-05-10):** Add Wave 0/F as a new prerequisite story (S-PLUGIN-PREREQ-F, 3-5 SP) that lands FIRST in Wave 0: (1) deprecate the rust-escape-hatch behavioral contract; (2) amend the datasource-trait-adapter-pattern behavioral contract; (3) amend the sealed-auth-trait domain invariant; (4) sweep sensor-named behavioral contracts. All Wave 0 code stories (A-E) depend on PREREQ-F.

---

### LOW

#### ADV-W4OPS-P01-LOW-001: ADR status PROPOSED but migration plan uses imperative-present tense

- **Severity:** LOW
- **Category:** ambiguous-language
- **Location:** ADR-023 frontmatter `status: PROPOSED`; §G body text
- **Description:** The status is correctly PROPOSED, but the migration plan body uses imperative-present tense ("Delete 4 named auth modules", "Wire PluginRuntime into boot") rather than future-conditional tense appropriate for a PROPOSED decision record.
- **Evidence:** This creates ambiguity about whether the steps are already in progress.
- **Proposed Fix:** Convert §G migration plan language to future-conditional tense consistent with PROPOSED status.

#### ADV-W4OPS-P01-LOW-002: Changelog lacks an initial entry for the PROPOSED version

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** ADR-023 Changelog section
- **Description:** The ADR body has no Changelog section, or the Changelog has no initial v1.0 entry documenting the PROPOSED version authoring.
- **Evidence:** All other ADRs in `.factory/specs/architecture/decisions/` have Changelog sections with initial entries.
- **Proposed Fix:** Add Changelog section with v1.0 initial entry: `v1.0 | 2026-05-10 | Initial PROPOSED version — plugin-only sensor architecture mandate`.

#### ADV-W4OPS-P01-LOW-003: PREREQ-D story estimate does not account for unsigned-plugin boot warning requirement

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** ADR-023 §E PREREQ-D / HIGH-002 user decision
- **Description:** The user decision on HIGH-002 adds a new requirement to PREREQ-D: `prism-bin` must emit a WARN-level log and audit log entry at boot when unsigned plugins are loaded. This was not in the original PREREQ-D scope.
- **Evidence:** The 8-13 SP estimate for S-PLUGIN-PREREQ-D should be reviewed against the additional boot warning work.
- **Proposed Fix:** Add the boot warning requirement to S-PLUGIN-PREREQ-D story scope. Revise SP estimate if needed.

#### ADV-W4OPS-P01-LOW-004: Verification property story citation uses inconsistent naming convention

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** ADR-023 §F `VP-PLUGIN-002`
- **Description:** The verification property is linked to a story ID that may not match the story naming convention in STORY-INDEX exactly. Wave 0 stories use "S-PLUGIN-PREREQ-*" while Wave 1 stories use "PLUGIN-MIGRATION-001-*".
- **Evidence:** Minor but worth correcting for STORY-INDEX consistency and to prevent stale story citations.
- **Proposed Fix:** Verify that all verification property story citations match story IDs in STORY-INDEX exactly. Update any stale citations.

---

### OBSERVATIONS / PROCESS GAPS

All 5 observations are tagged `[process-gap]`. These are deficiencies in the ADR template or the audit-to-ADR transition protocol, not defects in ADR-023 itself. They do NOT block ADR-023 convergence.

#### ADV-W4OPS-P01-OBS-001: ADR template lacks amendment-traceability fields `[process-gap]`

The ADR template frontmatter has no `amends_bcs:`, `retires_bcs:`, `amends_dis:`, or `amends_caps:` fields. CRIT-001 and CRIT-002 would have been caught at ADR-authoring time if the template prompted for these fields. Track as TD-ADR-AMEND-001.

#### ADV-W4OPS-P01-OBS-002: No Audit → ADR coverage matrix `[process-gap]`

The transition from `plugin-only-violations-2026-05-10.md` (21 findings) to ADR-023 has no coverage matrix verifying that each audit finding is addressed by at least one ADR decision or wave story. CRIT-003 and CRIT-004 may have been caught if such a matrix existed. Track as TD-AUDIT-ADR-001.

#### ADV-W4OPS-P01-OBS-003: User decisions captured as paraphrase, not verbatim `[process-gap]`

User decisions that drove ADR-023's content are paraphrased in the ADR body rather than quoted verbatim with timestamps. If a decision is later contested, the ADR provides insufficient attribution evidence. Track as TD-USER-DECISION-001.

#### ADV-W4OPS-P01-OBS-004: Plugin signing has a dimension-rich threat model; cannot be one PREREQ bullet `[process-gap]`

PREREQ-D's one-bullet "build/sign/load pipeline" understates the threat-model complexity of plugin signing (key custody, rotation, revocation, supply-chain attestation). HIGH-002 resolved this by deferring signing, but the template pattern of compressing security prerequisites into single bullets is a structural gap. Track as TD-SIGNING-PREREQ-001.

#### ADV-W4OPS-P01-OBS-005: ADR template lacks an "Open Questions" section `[process-gap]`

ADR-023 has unresolved questions (WASM ABI version pin, canonical `host_http_request` allowlist schema, format_version numbering scheme) that should be tracked in the ADR itself. The current template has no designated section for this. Track as TD-ADR-OPEN-Q-001.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 4 |
| HIGH | 9 |
| MEDIUM | 7 |
| LOW | 4 |
| OBS (process-gap) | 5 |
| **Total** | **26** |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision before pass-2

---

## User Decisions on Key Findings (2026-05-10)

The user reviewed the 4 highest-impact findings and made these calls:

1. **MED-001 → Confirm Rule 5:** "Retire CustomAdapter Rust trait — `.prx` WASM is the sole escape hatch." User-stated, durable. Eat own dog food. ADR Rule 5 stays. The `spec_parser.rs` call sites must be migrated to the plugin registry path as part of Wave 0/E.

2. **HIGH-002 → Defer signing to v1.0+1:** v1.0 ships unsigned plugins with explicit security warning at boot and an audit log entry. Tracked as TD-PLUGIN-SIGNING-001 (P0, v1.0+1, security). ADR-023 Negative Consequences must honestly document the v1.0 security exposure.

3. **HIGH-007 → Reorder Wave 1:** Wave 1/E (CrowdStrike `.prx` plugin) and Wave 1/D (TOMLs with DTU parity tests) MUST land and pass parity tests BEFORE Wave 1/A (delete Rust adapters). At every PR boundary in Wave 1, all four sensors must remain functional via either legacy Rust path OR new TOML+plugin path. Cutover commit deletes Rust path only after spec path passes parity test.

4. **MED-007 → Add Wave 0/F:** New story S-PLUGIN-PREREQ-F: deprecate the rust-escape-hatch behavioral contract, amend the datasource-trait-adapter-pattern behavioral contract, amend the sealed-auth-trait domain invariant, sweep sensor-named behavioral contracts. Lands FIRST in Wave 0 before any code changes. Estimated 3-5 SP.

---

## Fix-Burst Plan

### Step 1: architect dispatch — ADR-023 v1.1 amendment

Closes all CRIT + HIGH + MED + LOW findings. Key changes:
- Add `amends_bcs:`, `retires_bcs:`, `amends_dis:` frontmatter
- Add "Retired/Amended Contracts" section (CRIT-001/002)
- Fix crate reference in Forbidden Patterns (CRIT-003)
- Move PR template creation to Wave 0/F (CRIT-004)
- Add `ocsf_field` closed grammar + WASM-required catalog (HIGH-001)
- Document signing deferral + boot warning requirement + TD (HIGH-002)
- Reconcile verification property symbol list with actual codebase types (HIGH-003)
- Replace byte-level parity with testable parity criterion (HIGH-004)
- Rewrite sandbox model to align with `host_http_request` allowlist (HIGH-005)
- Re-author C3 with NEW vs already-present split (HIGH-006)
- Reorder Wave 1: D → E → A → B → C with parity-test gate (HIGH-007)
- Expand Negative Consequences with 6 omitted risk categories (HIGH-008)
- Add Wave 2/G note for production-runtime-wiring decision v1.2 amendment (HIGH-009)
- Apply individual remediations for MED-001 through MED-007
- Apply individual remediations for LOW-001 through LOW-004
- Add Wave 0/F to migration plan (per user decision on MED-007)

5 process-gap OBS findings tracked as TD items — do NOT block ADR convergence.

### Step 2: product-owner dispatch — Wave 0/F behavioral contract and domain invariant amendments

After ADR-023 v1.1 lands:
- Deprecate the rust-escape-hatch behavioral contract
- Amend the datasource-trait-adapter-pattern behavioral contract
- Amend the sealed-auth-trait domain invariant (compile-time → runtime enforcement)
- Sweep sensor-named behavioral contracts for plugin-only language

### Step 3: architect dispatch — production-runtime-wiring decision v1.2 amendment

After ADR-023 v1.1 lands:
- Add `superseded_by_partial: ADR-023 (§G Story 3 only)` to the production-runtime-wiring frontmatter
- Add inline note at top of §G Story 3 directing readers to ADR-023 and PLUGIN-PREREQ-F

### Step 4: adversary dispatch — ADR-023 pass-2

After fix-burst completes:
- Verify all 4 CRIT closed
- Verify all 9 HIGH closed
- Verify all 7 MED closed
- Verify all 4 LOW closed
- 5 process-gap OBS: tracked as TD items, NOT blocking
- Pass-2 must be CLEAN to advance toward 3-CLEAN convergence target

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 26 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (26 / (26 + 0)) |
| **Median severity** | 2.5 (between HIGH and MED) |
| **Trajectory** | 26 (pass 1 baseline) |
| **Verdict** | FINDINGS_REMAIN |

<!--
  This section is MANDATORY. The validate-novelty-assessment hook
  blocks adversarial review files missing this section or its required fields.
  
  Novelty score = new / (new + duplicate). Converged when < 0.15 for 2+ passes.
  See CONVERGENCE.md Dimension 1 for the full quantitative criteria.
-->
