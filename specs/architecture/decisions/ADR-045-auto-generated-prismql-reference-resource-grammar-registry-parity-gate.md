---
document_type: adr
adr_id: "ADR-045"
title: "Auto-Generated prismql://reference — Grammar-Registry Parity Gate via Runtime Assembly"
status: proposed
date: "2026-06-24"
version: "1.0"
producer: architect
subsystems_affected: [SS-10, SS-11]
supersedes: null
superseded_by: null
amends: ADR-041
anchor_stories: []
related_adrs: [ADR-041, ADR-042, ADR-043, ADR-044]
related_bcs: [BC-2.10.014, BC-2.11.001, BC-2.11.004]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-045: Auto-Generated `prismql://reference` — Grammar-Registry Parity Gate via Runtime Assembly

## Status

PROPOSED v1.0 (2026-06-24). Architect decision following grammar usability audit
(GRAMMAR-008, GRAMMAR-009, GRAMMAR-011, GRAMMAR-017) and pre-flight audit
(DISCOVERABILITY-GAP-001). Amends ADR-041 §L3 by specifying the generation strategy for
`prismql://reference`. Human ratification required on the build-time vs runtime tradeoff
(see §Human Ratification Gate). This ADR supersedes the static `pql_reference.md` file
approach implied by ADR-041 §L3 in favor of a hybrid model.

---

## Context

`prismql://reference` (`crates/prism-mcp/src/pql_reference.md`, 161 lines, ~6.5KB) is
the canonical PQL grammar reference that the `query` tool description directs analysts to
read. The grammar audit found:

- The BNF is wrong: it documents SQL clauses as pipe stages (line 43-44), the direct
  opposite of what the parser implements (GRAMMAR-009).
- The multi-stage pipeline example does not parse (GRAMMAR-009, confirmed live).
- The Datetime Arithmetic section documents `NOW()`/`INTERVAL` which do not exist in
  the parser (GRAMMAR-011, before ADR-044 lands).
- The Operators table omits CONTAINS/ICONTAINS, `=~`, `IN CIDR`, HAS, MISSING, BETWEEN,
  wildcard auto-promotion, percentile, distinct_count, dedup, `fields +/-` (GRAMMAR-007).
- Enrichment (`enrich`, pipe stages, UDF registry) is completely absent (GRAMMAR-008,
  DISCOVERABILITY-GAP-001).
- Composite sources (`EVENTS`/`ALERTS`/`DEVICES`), internal tables, virtual fields, scope
  model, quoting rules are absent (GRAMMAR-017).

The root cause of GRAMMAR-009 and GRAMMAR-011 is that `pql_reference.md` was authored
by hand against an aspirational model and was never regenerated or validated against the
actual grammar. The fix is structural: the reference must have a mechanical connection to
its sources of truth such that drift is impossible or immediately detected.

**ADR-041 §L3 context:** ADR-041 specifies `prismql://reference` as "server-authored
content" providing "BNF-style PQL grammar, operator semantics, error code quick-reference."
It does not specify how the content is produced or how parity is enforced. This ADR fills
that gap.

**The two viable strategies:**
- **Build-time generation (Option A):** A codegen step runs at `cargo build` time (via
  a `build.rs` or a `xtask`) and generates a Rust source file (via `include_str!`) or
  embeds the reference content as a `const`. The reference is baked into the binary.
- **Runtime assembly (Option B):** The `prismql://reference` resource handler assembles
  the reference content at serve time from live sources (AST doc-comments, operator tables,
  InfusionRegistry). The content is dynamic and reflects the current binary's live state.

The complication: some content IS per-deployment (infusion names — these come from the
live `InfusionRegistry` which is populated by WASM plugins loaded at startup, per ADR-040).
Other content is per-binary (grammar, operators, pipe stages — these are static Rust code).
A purely build-time approach cannot include per-deployment infusions without a runtime
component.

---

## Decision

**We adopt Option B (runtime assembly) for the grammar-invariant sections and a mixed
approach for the infusion section, with a CI parse-round-trip gate as the correctness
enforcer.**

**D1 — The static `pql_reference.md` file is REPLACED by a `build_reference_content`
function in `crates/prism-mcp/src/resources.rs`.** The function signature is:
```rust
pub fn build_reference_content(infusion_registry: Option<&InfusionRegistry>) -> String
```
This function is called by the `read_resource` handler when the URI is `prismql://reference`.
It returns a fully-assembled reference string. `InfusionRegistry` is passed in so that
the per-deployment enrichment section reflects live registered infusions. If
`infusion_registry` is `None` (e.g., at test time or before boot step 9 completes), the
enrichment section shows a placeholder "Call `list_infusions` to see available enrichment
functions for your deployment."

**D2 — Reference sections and their sources of truth:**

| Section | Source of truth | Generation strategy |
|---------|----------------|---------------------|
| Mode overview (Filter/SQL/Pipe) | This ADR + ADR-043 prose | Static Rust string constant in `resources.rs` |
| SQL mode BNF | `ast.rs` `SqlQuery` fields | Static Rust string constant (hand-written, validated by CI gate) |
| Pipe mode BNF + pipe stages | `pipe_parser.rs` doc-comment §Grammar | Static Rust string constant (hand-written, validated by CI gate) |
| SQL→Pipe composition (ADR-043) | ADR-043 §Decision D2 | Static Rust string constant, added after ADR-043 lands |
| Operators table | `ast.rs` `Predicate` enum doc-comment §4 table | Static Rust string constant (regenerated when `Predicate` changes) |
| Aggregates / stats | `pipe_parser.rs` `stats_stage` doc-comment | Static Rust string constant |
| Temporal grammar (ADR-044) | ADR-044 §D1-D3 | Static Rust string constant, added after ADR-044 lands |
| Virtual fields + scope model | `ast.rs` `VirtualField` enum | Static Rust string constant |
| Error code quick-reference | `error.rs` / `error-taxonomy.md` | Static Rust string constant (synced with error-taxonomy) |
| Enrichment section | `InfusionRegistry` (live) | Runtime: iterate registered infusions, emit names + call signatures |
| Query examples | Static, validated by CI gate | Static Rust string constant; each example is a string that MUST round-trip through `PrismQlParser::parse` |

**D3 — CI parse-round-trip gate (the parity enforcer).** A unit test in
`crates/prism-mcp/src/resources.rs` (or `crates/prism-query/tests/`) calls
`build_reference_content(None)`, extracts all PQL code blocks (delimited by
` ```\n…\n``` ` fences), and runs each non-placeholder example through
`PrismQlParser::parse`. Any `PARSE_ERR` result is a test failure. This gate would have
caught GRAMMAR-009 and GRAMMAR-011 mechanically. It must run in CI on every push.

**D4 — Static sections are Rust string constants in `resources.rs`, not a `.md` file.**
The `include_str!("pql_reference.md")` pattern is retired. All sections are inlined as
Rust string constants. This is intentional: it makes the reference content a first-class
code artifact, reviewed in PRs like code rather than as a doc file that drifts silently.

**D5 — InfusionRegistry access in `read_resource` handler.** The `PrismServer` already
holds the `InfusionRegistry` at runtime (wired in the boot sequence per ADR-022). The
`read_resource` handler (currently in `server.rs`) passes `Some(&self.infusion_registry)`
to `build_reference_content` when serving `prismql://reference`. No architectural change
is needed; this is a parameter-threading change.

**D6 — Reload awareness (ADR-042 alignment).** If new infusions are loaded via
`reload_infusion` (hot-reload), the `InfusionRegistry` is updated in the `ArcSwap`
(per ADR-042 pattern). The next `read_resource prismql://reference` call will receive
the updated registry. No caching of the assembled reference content is needed because
the resource is assembled on each read (it is not a hot path — it is called once per
query session setup, not per query).

---

## Rationale

1. **Runtime assembly is the only approach that can include per-deployment infusion
   names.** Build-time generation cannot know which WASM plugins will be loaded at a
   specific customer deployment. The `InfusionRegistry` is populated at runtime after
   WASM plugins are scanned. The only way to include live infusion names in the reference
   is to assemble the infusion section at request time.

2. **The CI parse-round-trip gate is the mechanical correctness enforcer.** The audit
   identified that GRAMMAR-009 and GRAMMAR-011 could have been caught automatically. The
   gate is a one-time test that runs forever. Static string constants + a parse gate is
   strictly superior to a static `.md` file with no gate.

3. **Static Rust constants are reviewed in PRs, unlike `.md` files.** The `pql_reference.md`
   file drifted for multiple story cycles without anyone noticing because it was a
   documentation asset outside the normal code-review attention boundary. Inlining the
   content as Rust constants makes every reference change a code-diff that triggers the
   same review discipline as production code changes.

4. **The `InfusionRegistry` already exists and is already wired.** Passing it to
   `build_reference_content` is a 2-line change. The alternative (maintaining a separate
   static infusion list) would immediately drift from the live registry and reproduce the
   same problem the audit found.

5. **Aligns with ADR-042 reload awareness.** The reference reflects the current live
   state of the binary (grammar) and the current live infusion set (registry), consistent
   with the ADR-042 principle that schema-adjacent state should be reload-aware.

---

## Consequences

### Positive

- `prismql://reference` can never silently diverge from the parser again — the CI gate
  catches it.
- The enrichment section lists actually-available infusions for the deployment.
- Every reference update is a code PR with review and CI coverage.
- The circular dependency (reference teaches examples that don't parse) is broken at the
  structural level.

### Negative / Trade-offs

- The static `pql_reference.md` file is deleted; all content migrates to Rust string
  constants. This is a one-time migration cost.
- `build_reference_content` must be kept in sync with `ast.rs` operator/stage changes.
  The CI gate is the enforcer, but it requires discipline to update the constants when the
  grammar evolves. This is an ongoing maintenance obligation.
- The assembled reference is slightly larger than the static file because it includes
  sections that were absent (enrichment, pipe modes, operators). The `prismql://reference`
  resource will grow from ~6.5KB to an estimated ~12-16KB. This is acceptable (the
  resource is fetched on demand, not injected on every turn per ADR-041 §L3).

### Status as of v1.0 (2026-06-24)

PROPOSED. The static `pql_reference.md` continues to exist at current HEAD. This ADR
gates the reference rewrite story.

---

## Human Ratification Gate

**HRG-1 — Build-time vs runtime assembly:** The human should confirm that runtime
assembly (Option B, D1) is acceptable versus build-time generation (Option A). The
architect's recommendation is Option B because of the per-deployment infusion names
requirement (D2). If the human prefers build-time generation with a runtime overlay for
infusions, a hybrid approach is feasible: bake the grammar sections at build time, append
the live infusion section at runtime. The functional outcome is identical; the difference
is implementation complexity. The recommendation stands with Option B (pure runtime).

---

## Alternatives Considered

- **Option A — Build-time codegen via `build.rs` or `xtask`:** Generate a Rust source
  file from `ast.rs` proc-macro reflection or doc-comment extraction at `cargo build`
  time. Rejected for the primary reason: build-time generation cannot include
  per-deployment infusion names. A hybrid (build-time grammar + runtime infusions)
  is functional but adds `build.rs` complexity. The runtime approach (Option B) is
  simpler and achieves the same outcome.

- **Keep `pql_reference.md` as a static file and add a CI round-trip gate only:**
  Keep the `.md` file, add a test that parses all its examples. Rejected because: (a) the
  file will continue to be maintained as documentation outside the normal code-review
  attention boundary; (b) the file cannot include live infusion names; (c) new sections
  (enrichment, pipe modes, virtual fields) still need to be written into it, which is the
  same effort as writing Rust constants with the added risk of future drift.

---

## Source / Origin

- Grammar usability audit: `.factory/research/prismql-grammar-usability-audit-2026-06-24.md`
  §GRAMMAR-008, §GRAMMAR-009, §GRAMMAR-011, §GRAMMAR-017, §3 Bucket 2.
- ADR-041 §L3 — Full Grammar Reference Resource (`prismql://reference`) — this ADR
  amends that section with the generation strategy.
- ADR-042 — Reload-Aware `resolved_spec_map` — alignment on reload-aware runtime state.
- `crates/prism-mcp/src/pql_reference.md` — the static file this ADR replaces.
- `crates/prism-mcp/src/resources.rs` — existing resource handler infrastructure.
- `crates/prism-query/src/ast.rs` — `Predicate` enum §4 operator table (source of truth
  for the operators section).
