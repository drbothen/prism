---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 1
target_sha: 72687483
base_sha: 95d46be2
verdict: BLOCKED-hard
streak: "0/3 → 0/3"
finding_summary:
  CRITICAL: 1
  HIGH: 5
  MEDIUM: 5
  LOW: 3
  OBS: 2
prior_passes: []
producer: adversary
timestamp: 2026-05-13T06:32:00Z
input-hash: "4beaa73"
inputs:
  - .factory/stories/S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
  - .factory/specs/behavioral-contracts/BC-2.17.001-*.md
  - .factory/specs/behavioral-contracts/BC-2.17.002-*.md
  - .factory/specs/behavioral-contracts/BC-2.17.003-*.md
  - .factory/specs/behavioral-contracts/BC-2.17.004-*.md
  - .factory/specs/behavioral-contracts/BC-2.17.005-*.md
  - .factory/specs/behavioral-contracts/BC-2.17.006-*.md
  - .factory/specs/behavioral-contracts/BC-2.22.001-*.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/stories/STORY-INDEX.md
  - .factory/policies.yaml
---

# Adversarial Pass 1 — S-PLUGIN-PREREQ-D

**Target:** S-PLUGIN-PREREQ-D v1.0 (prism-bin/prism-spec-engine: Wire PluginRuntime into Boot Sequence; .prx Load Pipeline)
**Target SHA:** 72687483 (factory-artifacts)
**Base develop SHA:** 95d46be2
**Verdict:** BLOCKED-hard
**Streak:** 0/3 → 0/3 (reset per BC-5.39.001: 1 CRITICAL present)
**Total findings:** 16 (1 CRIT + 5 HIGH + 5 MED + 3 LOW + 2 OBS)
**KUDOs:** 6

---

## Summary

S-PLUGIN-PREREQ-D v1.0 reviewed against ADR-023 §C4 + 7 BCs (BC-2.17.001–006, BC-2.22.001) + 2 VPs (VP-PLUGIN-004/VP-PLUGIN-007 via VP-INDEX named-alias table) + 16 active policies.

One CRITICAL finding blocks progression: the VP-INDEX named-alias rows for VP-PLUGIN-004 and VP-PLUGIN-007 semantically describe the wrong properties (TOML grammar and CustomAdapter retirement respectively, which are PREREQ-C and PREREQ-E scope). The story anchors to these named aliases, making the VP traceability chain incorrect until either the VP-INDEX aliases are corrected or the story re-anchors to numeric IDs (VP-149/VP-152).

Two process-gaps codified: VP-INDEX semantic-sync discipline (no standing policy) and manifest validation BC coverage gap (no BC for plugin manifest schema validation).

Fix-burst routing: architect (VP-INDEX named-alias correction, F-LP1-CRIT-001) + product-owner (manifest validation BC anchor, F-LP1-HIGH-004) in parallel; story-writer (all story-content findings) sequential after architect + PO complete; state-manager (VP-INDEX semantic-sync POL amendment) last.

---

## Findings

### F-LP1-CRITICAL-001 — VP-INDEX named-alias entries semantically describe wrong properties

**Severity:** CRITICAL
**Policy refs:** POL-4, POL-9
**Owner-specialist:** architect (VP-INDEX is architect-owned per routing table)

VP-INDEX line 186 entry for VP-PLUGIN-004 names it as the TOML grammar extension VP — that is PREREQ-C scope (VP-149 numeric alias). The story S-PLUGIN-PREREQ-D relies on VP-PLUGIN-004 named alias expecting it to describe "boot warning emitted when plugin_dir not configured." VP-INDEX line 189 entry for VP-PLUGIN-007 names it as the CustomAdapter retirement VP — that is PREREQ-E scope. The story expects VP-PLUGIN-007 to describe "allowlist not-None post-boot assertion."

The numeric aliases VP-149 and VP-152 carry the correct semantics per their sequential VP rows. The named-alias table diverges from the sequential rows.

**Required action:** Correct VP-INDEX named-alias rows for VP-PLUGIN-004 (must read: "boot warning emitted when plugin_dir not configured") and VP-PLUGIN-007 (must read: "plugin allowlist not-None post-boot assertion"), OR re-anchor the story frontmatter and body to numeric aliases VP-149, VP-152 exclusively and drop the VP-PLUGIN-004/VP-PLUGIN-007 named references.

**Impact if unresolved:** Story's VP traceability chain is incorrect. Any downstream verification that uses the named-alias table to locate coverage will map to the wrong VPs.

---

### F-LP1-HIGH-002 — STORY-INDEX row title diverges from story H1 title

**Severity:** HIGH
**Policy refs:** POL-7, POL-13
**Owner-specialist:** story-writer

Story H1 reads: "prism-bin/prism-spec-engine: Wire PluginRuntime into Boot Sequence; .prx Load Pipeline (...)"

STORY-INDEX line 392 reads: "...into boot step 7..." — incorrect (step 7 is Storage init; plugin-load is a NEW step inserted after step 7, not step 7 itself).

STORY-INDEX also reads ".prx Build/Sign/Load Pipeline" — incorrect; build and signing are explicitly out-of-scope per story body constraints.

Additionally, "Allowlist Enforcement; PR Template" is absent from the STORY-INDEX title summary, omitting key story scope.

**Required action:** Sync STORY-INDEX row title to H1 source of truth (POL-7). Correct "step 7" language to reflect new boot step insertion. Remove "Build/Sign" from scope description. Add "Allowlist Enforcement; PR Template" to summary.

---

### F-LP1-HIGH-003 — Per-plugin reqwest::Client semantics conflict with single boot-time client in AC-9

**Severity:** HIGH
**Policy refs:** internal contradiction
**Owner-specialist:** story-writer

AC-9 states a singular reqwest::Client is constructed once at boot time and shared. The Architecture Mapping section states make_host_state is Pure (no I/O side effects). If clients are per-plugin (each plugin gets its own client), then make_host_state cannot be Pure because it must perform I/O to initialize each client. If clients are shared (one client across all plugins), the story does not say so explicitly, leaving the implementer to infer a critical architectural decision.

**Required action:** Clarify in story body whether reqwest::Client is: (a) constructed once at boot and cloned/Arc-wrapped per plugin call, or (b) constructed per plugin instance. If (a), document the Arc<reqwest::Client> threading model. If (b), clarify that make_host_state is not pure and explain the side-effect boundary.

---

### F-LP1-HIGH-004 — BC-2.17.006 over-anchored: manifest field validation is NOT a BC-2.17.006 postcondition

**Severity:** HIGH
**Policy refs:** POL-4, POL-7
**Owner-specialist:** product-owner (BCs are product-owner-owned)

BC-2.17.006 H1 title is "WIT Interface Validation Before Plugin Registration" — its postconditions cover WIT interface conformance only.

Story AC-5 anchors E-PLUGIN-013 (missing allowed_urls field), E-PLUGIN-014 (invalid format_version), and name/version validation to BC-2.17.006. These are manifest schema validations, not WIT interface validations.

**Options:**
- (a) Author a new BC-2.17.007 "Plugin Manifest Schema Validation Before Registration" covering manifest field presence, format_version range, name/version format
- (b) Amend BC-2.17.006 scope to explicitly include manifest field validation as a precondition gate (WIT validation depends on valid manifest fields being present)

Either option resolves the over-anchoring. Process-gap: no existing BC covers manifest schema validation — this is a coverage gap in the BC corpus independent of which option is chosen.

---

### F-LP1-HIGH-005 — Three test name conventions used inconsistently across story

**Severity:** HIGH
**Policy refs:** (project naming convention, no explicit policy ID)
**Owner-specialist:** story-writer

The story mixes three test naming conventions:
1. BC-prefixed: `test_BC_2_22_001_...`
2. VP-prefixed: `test_VP_PLUGIN_004_...`
3. Bare names (no prefix)

Additionally, AC-12 and AC-13 wording in the Acceptance Criteria table does not match the corresponding Red Gate test names — implementers will have ambiguity about which test closes which AC.

**Required action:** Adopt a single convention throughout. Recommend BC-prefixed for BC-anchored tests, VP-prefixed for VP-anchored tests, and story-prefixed (e.g., `test_S_PLUGIN_PREREQ_D_...`) for story-level integration tests with no BC/VP anchor. Align AC-12/AC-13 AC wording to Red Gate test names exactly.

---

### F-LP1-HIGH-006 — make_host_state signature cascade not fully enumerated (TD-VSDD-060 sibling-site sweep)

**Severity:** HIGH
**Policy refs:** TD-VSDD-060 (sibling-site sweep discipline)
**Owner-specialist:** story-writer

Task 2 changes the make_host_state signature to accept allowed_urls. The Match-Site Inventory does not enumerate:
- mod.rs:202 (call site)
- mod.rs:279 (call site)

Additionally, three remaining TODO(S-4.08) sites at mod.rs:395, mod.rs:419, and mod.rs:442 (fire-alert/fire-case/fire-report) are ambiguous — the story does not clarify whether these are closed by this story or remain open for S-4.08.

**Required action:** Add mod.rs:202 and mod.rs:279 to the Match-Site Inventory. Add a disposition row for TODO(S-4.08) sites clarifying: if these sites are NOT closed by S-PLUGIN-PREREQ-D, explicitly label them "OUT OF SCOPE — S-4.08" in the inventory.

---

### F-LP1-MEDIUM-007 — "plugin sandbox + lifecycle contracts" misdescription

**Severity:** MEDIUM
**Policy refs:** POL-7
**Owner-specialist:** story-writer

The story Overview section describes the BC-2.17.* cluster as "plugin sandbox + lifecycle contracts." BC-2.17.* are isolation contracts (WIT interface isolation, capability enforcement, fuel limits), not lifecycle contracts. Lifecycle is BC-2.22.*. The mislabeling will cause future adversary passes to flag BC routing errors.

**Required action:** Rename the cluster description to "plugin isolation contracts" or "plugin capability + WIT isolation contracts."

---

### F-LP1-MEDIUM-008 — Fixture Strategy singular vs. plural contradiction

**Severity:** MEDIUM
**Policy refs:** internal consistency
**Owner-specialist:** story-writer

Fixture Strategy prose states "commit minimal.prx singular" (implying one fixture). The fixture table below lists 4 .prx artifacts: minimal.prx, no_allowlist.prx, malformed_manifest.prx, bad_wit.prx.

**Required action:** Update Fixture Strategy prose to "commit 4 .prx test fixtures" or consolidate to a single fixture if the table is aspirational.

---

### F-LP1-MEDIUM-009 — TD-B-005 closure crate ownership unspecified

**Severity:** MEDIUM
**Policy refs:** POL-9 (traceability)
**Owner-specialist:** story-writer

The story closes TD-B-005 (reqwest::Client construction). TD-B-005 states the Client must be constructed in one canonical location. The story does not specify whether the canonical constructor lives in prism-bin (boot sequence crate) or prism-spec-engine (plugin executor crate).

**Required action:** Add one sentence: "TD-B-005 closure: reqwest::Client is constructed in [prism-bin::boot | prism-spec-engine::executor] and passed to PluginRuntime via [Arc<Client> | owned transfer]."

---

### F-LP1-MEDIUM-010 — Hot-reload watcher install out of scope but BC-2.17.005 anchored

**Severity:** MEDIUM
**Policy refs:** POL-14 (BC promotion discipline)
**Owner-specialist:** story-writer

BC-2.17.005 (hot-reload watcher lifecycle) is listed in the story's BC frontmatter as anchored. The story Constraints section explicitly states "hot-reload watcher install is out of scope for this story." POL-14 auto-promotes a BC to active when its story is marked complete. Anchoring BC-2.17.005 will cause it to promote prematurely — the watcher is not installed by this story.

**Required action:** Either (a) drop BC-2.17.005 from the frontmatter BC list and add a note "BC-2.17.005 deferred to [story ID]", or (b) document a partial-promotion rationale: "BC-2.17.005 promoted to partial-active; watcher registration satisfied; watcher activation deferred."

---

### F-LP1-MEDIUM-011 — Edge case EC-D-008 not testable without dedicated Red Gate test

**Severity:** MEDIUM
**Policy refs:** POL-8 (AC-to-test traceability)
**Owner-specialist:** story-writer

Edge case EC-D-008 describes "duplicate plugin ID: first registered wins." No Red Gate test `test_BC_2_17_006_duplicate_plugin_id_first_wins` (or equivalent) appears in the story's test name list.

**Required action:** Add `test_BC_2_17_006_duplicate_plugin_id_first_wins` to the Red Gate test list, or cite a pre-existing test in the test corpus that covers this scenario by name and location.

---

### F-LP1-LOW-012 — sha2 workspace dependency status unverified

**Severity:** LOW
**Policy refs:** POL-12 (dependency audit)
**Owner-specialist:** story-writer

The story adds `sha2` as a dependency for .prx manifest hash verification. It is not confirmed whether sha2 is already a workspace-level dep (Cargo.toml workspace.dependencies) or must be added fresh.

**Required action:** Check workspace Cargo.toml for sha2; if present, note "sha2 already in workspace." If absent, note "sha2 NEW — add to workspace.dependencies."

---

### F-LP1-LOW-013 — wasmtime "17 RUSTSEC advisories" claim lacks citation

**Severity:** LOW
**Policy refs:** POL-9 (evidence)
**Owner-specialist:** story-writer (minor)

The story notes "wasmtime has 17 open RUSTSEC advisories." This number appears to be an estimate or stale figure. No citation date or advisory list reference is provided.

**Required action:** Replace with a cited count: "As of [date], wasmtime has [N] open RUSTSEC advisories per `cargo audit`" or simply remove the numeric claim and replace with "wasmtime has known RUSTSEC advisories — run `cargo audit` for current count."

---

### F-LP1-LOW-014 — Token Budget table omits fixture .prx files

**Severity:** LOW
**Policy refs:** completeness
**Owner-specialist:** story-writer (minor)

The Token Budget table accounts for source files and test files but does not include the 4 .prx fixture files committed to the repository. These contribute to binary size and build artifact scope.

**Required action:** Add a "Fixtures" row to the Token Budget table: "4 × .prx fixture files — committed to tests/fixtures/plugins/."

---

### F-LP1-OBS-015 — HostState struct #[non_exhaustive] status for allowed_urls field

**Severity:** OBS
**Policy refs:** CLAUDE.md #[non_exhaustive] discipline
**Owner-specialist:** story-writer

Task 2 adds `allowed_urls: Vec<Url>` to HostState. The story does not confirm whether HostState carries #[non_exhaustive]. If it does, adding the field is a safe semver addition. If it does not, adding the field is a breaking change for downstream crates constructing HostState with struct literal syntax.

**Required action:** Note in story body: "HostState is #[non_exhaustive] (confirmed [source reference]) — field addition is non-breaking" or "HostState lacks #[non_exhaustive] — TD-XXX filed to add it before this field lands."

---

### F-LP1-OBS-016 — PRISM_DISABLE_PLUGIN_LOAD precedence vs config-file plugin_dir under-specified

**Severity:** OBS
**Policy refs:** completeness
**Owner-specialist:** story-writer

The story defines PRISM_DISABLE_PLUGIN_LOAD env var as a kill-switch. It does not specify precedence when both PRISM_DISABLE_PLUGIN_LOAD=1 AND a plugin_dir is configured in prism.toml. Is the env var an absolute override, or does plugin_dir take precedence if set?

**Required action:** Add one sentence to the env-var section: "PRISM_DISABLE_PLUGIN_LOAD=1 overrides any plugin_dir configuration; no .prx files are loaded regardless of prism.toml settings."

---

## Process-Gaps

### PG-LP1-001 — VP-INDEX named-alias semantic-sync has no standing policy

The VP-INDEX named-alias table can drift from the sequential VP rows (as demonstrated by F-LP1-CRIT-001). No current policy requires semantic verification that named-alias rows match the sequential rows they alias. Recommend: POL-9 amendment or new policy "Every VP-INDEX named-alias row MUST semantically match the description of its sequential VP row — verified at every VP-INDEX edit."

### PG-LP1-002 — Manifest validation has no BC anchor

Plugin manifest schema validation (format_version range, allowed_urls presence, name/version format) is not covered by any existing BC. BC-2.17.006 covers WIT interface validation only. Either BC-2.17.007 "Plugin Manifest Schema Validation Before Registration" or a BC-2.17.006 scope amendment is required. This is a coverage gap independent of F-LP1-HIGH-004 routing.

---

## KUDOs

1. **Comprehensive AC-to-BC traceability table** — every AC row is anchored to a BC or VP; best-practice per POL-8.
2. **Match-Site / Stub Replacement Inventory** — enumerates production stubs with file:line references; best-practice per POL-12.
3. **Edge Cases table thorough** — 11 EC scenarios including env-var precedence corners (EC-D-005/EC-D-006), duplicate plugin ID (EC-D-008), malformed manifest (EC-D-003), and WIT mismatch (EC-D-004).
4. **Structured Event Catalog Additions section** — 7 event_type rows with schema + PG-LP11-001 SOP reference; establishes clear observability contract for boot sequence.
5. **Forbidden Dependencies callout** — explicitly lists disallowed crates at story level (architectural perimeter enforcement per CLAUDE.md discipline).
6. **Fixture Strategy explicit COMMIT decision** — avoids cargo component build bootstrap by committing pre-built .prx artifacts; reduces CI complexity.

---

## Convergence Position

**Streak:** 0/3 → 0/3 (BLOCKED-hard). Per BC-5.39.001, streak resets on any CRITICAL finding.

**Fix-burst routing:**

| Agent | Findings | Priority |
|-------|----------|----------|
| architect | F-LP1-CRIT-001 (VP-INDEX named-alias correction) | FIRST (parallel with PO) |
| product-owner | F-LP1-HIGH-004 (new BC or BC-2.17.006 amendment for manifest validation) | FIRST (parallel with architect) |
| story-writer | F-LP1-HIGH-002/003/005/006 + F-LP1-MED-007/008/009/010/011 + F-LP1-LOW-012/013/014 + F-LP1-OBS-015/016 | SECOND (after architect + PO complete) |
| state-manager | Codify VP-INDEX semantic-sync process-gap as POL-9 amendment or new policy | LAST |

**Next dispatch:** Fix-burst-1 — architect + product-owner in parallel.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 16 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 16 / (16 + 0) = 1.00 |
| **Median severity** | 3.0 (HIGH) |
| **Trajectory** | 16 |
| **Verdict** | FINDINGS_REMAIN |
