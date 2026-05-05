---
document_type: uncertainty-map
level: L4
story_id: S-3.01
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: 2026-05-04T00:00:00Z
status: draft
---

# S-3.01 Uncertainty Map — PrismQL Parser (Filter + SQL + Pipe)

## Summary

**Verdict: RED — DO NOT START IMPLEMENTATION YET**

S-3.01 has multiple HIGH-severity blockers that will derail TDD if implementation begins as written. The single most disruptive finding is a **crate-name collision**: the `prism-query` crate **already exists in the workspace** with completely different content (S-2.08 virtual-field injection + S-3.2.08 CrowdStrike pagination session ID generation). The story plans to "Create" `crates/prism-query/Cargo.toml` and `lib.rs`, but doing so will overwrite the in-tree implementation that other stories depend on. There is also a hard inconsistency between the story (Chumsky 0.12), the grammar reference (Chumsky 0.10), the architecture decision text (mixes 0.10 + 0.12), and an external research signal: as of mid-2025, Chumsky 0.10.x was the documented stable line on crates.io with major API changes vs 0.9.x — Chumsky **0.12 may not exist yet** (RESEARCH-NEEDED, but this is the highest-confidence indicator that the version pin is stale or fabricated). Several VPs reference contradictory limits (depth 32 vs 64) and the story declares VPs (VP-014/015/021) that are themselves still in `draft` and were last touched 2026-04-19 — older than the story's last edit but predicated on the same stale Chumsky claim.

| Category | HIGH | MEDIUM | LOW |
|----------|------|--------|-----|
| Library Versions | 3 | 1 | 1 |
| API References | 2 | 1 | 0 |
| Architecture Pattern Drift | 1 | 1 | 0 |
| Crate Path Validity | 2 | 1 | 0 |
| Sister-Spec Consistency | 2 | 2 | 0 |

---

## Library Versions

| Library | Story-Assumed | Current Stable (claim) | Risk Level | Notes |
|---------|---------------|------------------------|------------|-------|
| chumsky | 0.12.x (story §Library & Framework, §Tasks, §Architecture Mapping; AD-003) | RESEARCH-NEEDED — based on training data, Chumsky 0.10 was the actively maintained line; 0.11/0.12 status unclear | **HIGH** | Story, AD-003, dependency-graph.md all say 0.12. But `domain-spec/prismql-grammar.md` (frontmatter line 7) says **Chumsky 0.10**, and `query-engine.md` line 103 says "axiathon uses Chumsky 0.10... Chumsky 0.12 adds..." — implying a future upgrade not yet validated. Lockfile contains NEITHER chumsky 0.10 nor 0.12. Implementation must verify the latest crates.io release **before** writing the dep line. |
| ariadne | 0.4.x (story §Library & Framework; dep-graph.md) | RESEARCH-NEEDED | **MEDIUM** | Ariadne's API for Source/Span builders has shifted across 0.3→0.4. Story says only "0.4.x" with no specific minor — needs verification that the Chumsky-recommended integration path still works against current ariadne. |
| kani | "(dev)" — no version pin (story §Library & Framework) | RESEARCH-NEEDED | **HIGH** | Story does not pin a kani version. Kani has been in active development with frequent breaking changes to the Arbitrary derive macro and proof harness attribute syntax. Two Kani proofs (VP-014, VP-015) are AC-10 gating items — unpinned tooling means the proofs could fail to compile day one. |
| libfuzzer-sys | 0.4.x (story §Library & Framework) | 0.4.x is the long-running stable line | **LOW** | Stable; widely used. Confirm during implementation only. |
| chumsky/ariadne in workspace | NOT IN LOCKFILE | — | **HIGH** | Confirmed via `grep chumsky\|ariadne /Users/jmagady/Dev/prism/Cargo.lock` — neither crate is currently a transitive dep. First introduction will require a fresh resolution; any version drift between story claim and crates.io reality bites here. |

---

## API References

| API Pattern | Source | Risk Level | Concern |
|-------------|--------|------------|---------|
| `&str -> Result<Ast, Vec<RichError>>` (query-engine.md AD-003 §Consequences) | architecture/query-engine.md:104 | **HIGH** | `RichError` is the Chumsky 0.10+ error type. Chumsky 0.9 used `Simple<T>`. The architecture text mixes 0.10 (axiathon ref) and 0.12 (target) — implementation must verify whether `RichError`/`Rich` is still the canonical type in the version actually picked. The story's parser modules and `error.rs` will be authored against this assumption. |
| Chumsky `recover_with`, `skip_then_retry_until`, `nested_delimiters` (story Task 7) | story line 158-160 | **HIGH** | Chumsky's recovery API was reworked between 0.9 → 0.10 (function names same, but signatures changed; some moved to a `recovery` submodule). If the implementer reaches for these without re-reading the changelog, the parser will not compile. |
| `text::ident()` with case-insensitive comparison, citing `zesterer/chumsky#699` | grammar.md:32 | **MEDIUM** | The cited GitHub issue/community pattern may be obsolete in 0.10+. Verify the recommended idiom for case-insensitive keywords in the version actually selected. |
| Chumsky `Stream`, `Span`, `Recursive<...>` as referenced indirectly via grammar (e.g. recursive `predicate`, "boolean expression tree") | story Task 3-5; grammar §2 | **MEDIUM** | Recursive parser combinator pattern in Chumsky 0.10 requires `Recursive::declare()` + `define()`; signatures are stricter than 0.9. Story does not call out the right pattern explicitly. |

---

## Architecture Pattern Drift

**ADR-006 / ADR-008 (multi-tenant OrgId):** ADR-006 states `OrgId(Uuid v7)` flows through "query plan construction (org boundary constraint carries OrgId)". The story makes **no mention** of org_id threading — neither in AST types (`SqlQuery`, `FilterExpr`, `PipeQuery` lack any tenant scope field), in `SourceRef` definition, nor in security checks. **This is a HIGH concern** because S-3.02 and downstream stories assume the parser produces ASTs that the executor can re-key by org. Two consistent positions exist: (a) the parser is intentionally tenant-agnostic and the executor injects scope at planning time; or (b) the parser's `SourceRef` should accept `{org_slug}:{sensor}.{table}` syntax. The story currently implies (a) by silence but does not commit to it. Without clarity, the work cannot proceed without either rework or an architect ruling.

**AD-022 / S-3.06 split (write parser):** The story scope is read-only PrismQL. AD-022 (write operations) is split into S-3.06 (`prismql-write-parser`) which depends on S-3.01. The story correctly excludes write verbs but the AST design must leave room for them: `PipeStage` enum and `SqlQuery` variants need to be `#[non_exhaustive]` or otherwise extension-safe so S-3.06 does not require a breaking refactor. Story does not specify this. **MEDIUM**.

**axiathon reference codebase:** AD-003 cites `axiathon-query` as the reference implementation but the story does not include a path to the reference crate or specify which patterns to lift. If axiathon was part of the brownfield ingestion list, that path must be discoverable; otherwise "axiathon-proven" is an ungrounded claim. **MEDIUM** — RESEARCH-NEEDED.

---

## Crate Path Validity

| Path / Module | Story Claim | Filesystem Reality | Risk Level |
|---------------|-------------|--------------------|------------|
| `crates/prism-query/Cargo.toml` | "Create" (Task 1, File Structure Requirements row 1) | **Already exists** — owned by S-2.08; describes "virtual field injection and sensor table materialization (S-2.08). DataFusion TableProvider integration added in S-3.02." | **HIGH** — collision. S-3.01 cannot "Create" a file that already exists; needs to be reframed as "Extend" with explicit instructions to preserve existing modules (`types`, `materialization`, `org_scoped_session_id`). |
| `crates/prism-query/src/lib.rs` | "Create" (File Structure row 2) | **Already exists** — declares `pub mod org_scoped_session_id; pub mod materialization; pub mod types;`. | **HIGH** — story will overwrite real code unless rewritten. |
| `crates/prism-query/src/ast.rs`, `filter_parser.rs`, `sql_parser.rs`, `pipe_parser.rs`, `security.rs`, `error.rs` | "Create" (File Structure rows 3-8) | Do not exist | **LOW** (paths are fine, but nest under a now-mixed crate purpose) |
| Workspace member `crates/prism-query` in `Cargo.toml` workspace | Implied present | Confirmed: line 25 of root Cargo.toml | **LOW** — already a member. |
| `crates/prism-query/proofs/` and `crates/prism-query/fuzz/` | "Create" (File Structure rows 11-12) | Do not exist | **MEDIUM** — placement convention: this repo's existing kani/fuzz layout (if any) must be checked. Story does not reference a workspace-level fuzz harness pattern. |
| `prism-core` import (story Task 1: "prism-core workspace dependencies") | Required | Crate exists at `crates/prism-core` | **LOW** — confirmed. |
| `prism-storage`, `prism-spec-engine` (existing prism-query deps) | Story strips them implicitly by listing only `prism-core`, `chumsky`, `ariadne` | Existing Cargo.toml has both as deps | **MEDIUM** — story Task 1 wording suggests a from-scratch dep list; if implementer overwrites the existing Cargo.toml, S-2.08/S-3.2.08 code in this same crate breaks because its `prism-storage` + `prism-spec-engine` deps disappear. |

---

## Sister-Spec Consistency

| Item | Finding | Risk |
|------|---------|------|
| VP-031 | User prompt referred to VP-031 as "this story's anchor VP". **VP-031 is anchored to S-3.02, not S-3.01** (file frontmatter line, body §Source Contract). Story's actual VPs are VP-014, VP-015, VP-021. | **MEDIUM** (prompt-level error; story itself is internally correct) |
| VP-014/015/021 | All exist on disk; VP-014/021 are anchored to S-3.01 explicitly; VP-015 also anchors to S-3.01. All three are `lifecycle_status: active`, `status: draft`. | **LOW** — green |
| VP-015 depth limit | VP-015 §Property Statement says depth ceiling **"e.g. 32"**; BC-2.11.006 and story EC-002 + Task 6 say **64**; PRD/grammar reference §"Security limits" says nesting 128 -> 64 (DI-019). | **HIGH** — VP-015 spec text is inconsistent with the canonical limit. Implementation will compile against 64 (story-driven) but the Kani harness language will say 32 unless the VP is corrected. |
| BC-2.11.001…012 referenced by user prompt | All 12 BCs exist on disk and BC-INDEX shows them all `draft` (active). BCs 013/014/015 also exist (alias mgmt). Story actually only declares dependence on BC-2.11.{002,003,004,006}. | **LOW** — story scope is well-bounded. |
| Mode auto-detection rule | Story §Dev Notes says "filter mode requires `\|` immediately after the source ref with no SELECT keyword; SQL mode starts with SELECT; pipe mode's stages use lowercase keywords after the initial `\|`." But grammar.md §2.1 says **"Starts with `FROM` → pipe mode"** and PRD has yet a third version (mode unified by SELECT presence). | **MEDIUM** — three documents with three different mode-detection rules. Architect must pick one before parser is written, otherwise grammar tests will diverge from BC text. |
| Pipe stage list | Story Task 5 lists 9 stages (7 BC-required + `join` + `enrich`). BC-2.11.004 must be cross-checked to confirm `join`/`enrich` are in scope for the parser, not the executor. | **MEDIUM** |

---

## Recommended Spec Updates (BEFORE implementation begins)

1. **Resolve the prism-query crate collision.** Either: (a) rewrite S-3.01 File Structure & Tasks to "Extend existing crate" with explicit no-overwrite guarantees for `types.rs`, `materialization.rs`, `org_scoped_session_id.rs`, and a merged Cargo.toml dep list (`prism-core`, `prism-storage`, `prism-spec-engine`, `serde`, `serde_json`, `uuid`, `chumsky`, `ariadne`); or (b) move the parser into a new crate `prism-query-parser` and update AD-001's crate count. Option (a) is preferred — keeps SS-11 in one crate.
2. **Pin Chumsky to a verified version.** Run `cargo search chumsky` (or equivalent research) to confirm the latest stable. Update story §Library & Framework, AD-003 (query-engine.md line 93/103), and `domain-spec/prismql-grammar.md` frontmatter line 7 to a single agreed-upon version. The current three-way disagreement (0.10 / 0.12 / unstated) is a recipe for review thrash.
3. **Pin Kani version explicitly.** Add `kani = "<x.y>"` (dev-dep) to story §Library & Framework. Without this, AC-10 ("VP-014 and VP-015 Kani proofs pass under `cargo kani`") is non-deterministic.
4. **Reconcile depth limit (32 vs 64).** Update VP-015 §Property Statement to drop the "e.g. 32" hedge and pin to 64 (per BC-2.11.006 / DI-019). Otherwise the proof harness skeleton will encode the wrong constant.
5. **Clarify org_id threading.** Add a §Architecture Compliance bullet stating explicitly that the parser is org-agnostic (executor responsibility) — and add a note that AST types must NOT include hardcoded tenant scope. This is a no-op for code but a hard guarantee for cross-wave consistency with ADR-006.
6. **Mark AST types `#[non_exhaustive]`.** Update Task 2 to require this for `PipeStage`, `SqlQuery`, `FilterExpr`, `Expr` so S-3.06 (write parser) and S-3.07 (write execution) can extend without breaking SemVer in this internal crate's downstream consumers.
7. **Pick ONE mode-detection rule** across story §Dev Notes, `query-engine.md`, and `prismql-grammar.md §2.1`. Right now the three docs disagree.
8. **Specify axiathon reference path.** Either include a path under brownfield/ingested-references or remove the "axiathon-proven" rationale from AD-003 and replace with a documented internal proof-of-concept.
9. **Resolve Chumsky error type name** (`Rich`, `RichError`, `Simple`, `Cheap`) consistently across story, AD-003, and parser modules.
10. **Confirm `prism-storage` and `prism-spec-engine` dep retention** in the merged Cargo.toml — these are required by existing in-tree code and were silently elided from the story's dep list.

---

## Implementation Risk Assessment

If implementation starts now, the first 30 minutes will be spent in a foreseeable disaster: the implementer creates a TDD test, runs `cargo test -p prism-query`, and discovers the crate already has materialization and session code. Two outcomes follow — they either (a) overwrite `lib.rs` and `Cargo.toml` from the story spec verbatim, breaking S-2.08-derived code in `materialization.rs` and `org_scoped_session_id.rs` and orphaning the existing test suite (a downstream Wave 2/3 regression), or (b) try to merge by intuition, producing a hybrid Cargo.toml whose dep set has not been reviewed. Either path will surface in CI as a structural failure rather than a clean spec-vs-code diff. The Chumsky version question compounds this: the very first `cargo build` may fail to resolve `chumsky = "0.12"` if 0.12 is not yet on crates.io, leaving the implementer to guess a version on their own. The depth-limit mismatch (32 vs 64) becomes a minor annoyance compared to the crate collision but will surface during VP-015 proof authoring. The org_id silence is the slowest-to-detect bomb: if the parser ships without an architect-confirmed "executor injects org scope" rule, S-3.02 will need an emergency AST extension, breaking the "S-3.01 unblocks 12 stories" assumption. **Net: lockstep RED, blocked on six spec edits, est. 1-2 hours of architect/PO time before TDD can safely begin.**

