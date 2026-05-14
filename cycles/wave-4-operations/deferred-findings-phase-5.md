---
document_type: deferred-findings-phase-5
cycle: wave-4-operations
status: active
created: 2026-05-13
producer: state-manager
rationale: "Findings that are out-of-perimeter for story-scoped fix-bursts but require resolution before Phase 5 (Adversarial Refinement, post-implementation cascade) closes. Per CLAUDE.md Canonical Principle Rule 3: not added to tech-debt-register (no human direction, no concrete future dependency, no story anchor). Phase 5 PO-led adjudication is the correct routing for cross-document governance gaps."
---

# Phase-5 Deferred Findings — Wave 4 Operations Cycle

This file accumulates findings that are out-of-perimeter for story-scoped fix-bursts during Phase 3
(TDD Implementation). Each finding is routed here when:
1. The defect is in a cross-cutting artifact (error-taxonomy.md, BC pair governance) rather than in the
   story body under review.
2. The fix requires PO adjudication or architectural decision that goes beyond story-writer scope.
3. The CLAUDE.md boundaries clause applies: "expanding into a new domain that requires new specs or
   new architecture decisions" requires explicit scope expansion request.

Findings here MUST be addressed before Phase 5 (Adversarial Refinement) convergence is declared.
They are NOT tech-debt-register entries (no human-directed deferral; these are production-grade gaps
awaiting the correct phase gate).

---

## F-LP12-OBS-001 — E-PLUGIN-008 Dual-Semantic Reuse

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP12-OBS-001 |
| **Severity** | OBS (out-of-perimeter for story scope; substantive for phase-5) |
| **Confidence** | HIGH |
| **Story source** | S-PLUGIN-PREREQ-D |
| **Surfaced at** | Pass-12 (adversary fresh-context audit) |
| **Date routed** | 2026-05-13 |
| **Target** | Phase-5 product-owner-led error namespace adjudication |

### Evidence

- **BC-2.17.005** anchors E-PLUGIN-008 to hot-reload WASM compilation failure (message: retry with new binary).
- **BC-2.17.006** anchors the same E-PLUGIN-008 code to boot-time `Component::from_binary` failure on corrupt `.prx` bytes (message: initial-load failure, no previous version to retain).
- **error-taxonomy.md** message template for E-PLUGIN-008: "Plugin '{plugin_id}' hot-reload failed: WASM compilation error: {error}. Previous version retained." — anchored ONLY to BC-2.17.005 hot-reload context; misleading and incorrect at boot-time initial-load where no previous version exists to retain.

### Why It Matters

An implementer coding EC-D-007 (boot-time corrupt bytes scenario per BC-2.17.006) would use E-PLUGIN-008 per the story spec — correct per BC-2.17.006 anchor. But the error-taxonomy.md message template would produce a message saying "Previous version retained" when there is no previous version. At boot-time initial-load, retaining a previous version is not possible.

Story EC-D-007 (line 126) is internally consistent (cites E-PLUGIN-008 per BC-2.17.006) — the STORY is not the defect location. The gap is cross-doc governance between BC-2.17.005, BC-2.17.006, and error-taxonomy.md.

### Fix Options (for Phase-5 PO adjudication)

**Option A — Split E-PLUGIN-008**: Create E-PLUGIN-008a (hot-reload, retain current message template "hot-reload failed: WASM compilation error") + new E-PLUGIN-N (initial-load, message "initial-load failed: WASM compilation error; no previous version available"). POL-1 append-only new code; existing E-PLUGIN-008 hot-reload usage in BC-2.17.005 preserved.

**Option B — Conditional message template**: Update error-taxonomy.md template to context-aware messaging covering both anchors (e.g., template parameterized by context: hot-reload vs initial-load). Single code, two message forms.

**Option C — Re-anchor BC-2.17.006**: Assign a distinct new E-PLUGIN-N code to BC-2.17.006 initial-load failure; preserve E-PLUGIN-008 as hot-reload-only semantics. Simplest canonical split; highest BC-drift risk (BC-2.17.006 + story EC-D-007 both need updating).

### Pre-existing Gap Age

11 passes (story created for pass-1; gap existed from story creation but only surfaced at pass-12 via fresh-context deep audit).

### Resolution Criteria

Before Phase 5 convergence can be declared: error-taxonomy.md E-PLUGIN-008 entry updated so that the message template is not misleading for the boot-time initial-load context, OR a distinct error code for that context is established, with BC-2.17.005 and BC-2.17.006 updated accordingly.

---

## F-LP16-OBS-001 — Workspace Edition Inconsistency (`prism-bin` edition 2021 vs. canonical 2024)

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP16-OBS-001 |
| **Severity** | OBS (out-of-perimeter for story scope; substantive for phase-5) |
| **Confidence** | HIGH |
| **Story source** | S-PLUGIN-PREREQ-D |
| **Surfaced at** | Pass-16 (adversary fresh-context audit) |
| **Date routed** | 2026-05-13 |
| **Target** | Workspace-wide edition unification (phase-5 architect adjudication) |

### Evidence

- `crates/prism-bin/Cargo.toml:4` declares `edition = "2021"`.
- CLAUDE.md §Toolchain states the canonical edition is `"2024"` (rust-toolchain.toml, resolver 2, edition 2024).
- `crates/prism-spec-engine/Cargo.toml` correctly uses `edition = "2024"`.
- Other crates were not surveyed in this pass (full workspace audit is a phase-5 sweep task).

### Why It Matters

Inconsistent crate editions create friction for cross-crate feature usage (e.g., `let`-chains, raw-pointer patterns, and other edition-gated syntax are available in edition 2024 but not edition 2021). Future Rust edition deprecation cycles will make edition 2021 a lag item. As prism-bin is the binary entry point for the platform, it should be on the canonical edition.

### Fix Options

**Option A — Workspace-wide edition sweep:** Update all crates to `edition = "2024"` simultaneously. Verify each crate's MSRV compatibility (run `cargo check` after each bump to catch edition-gated breakage).

**Option B — Per-crate migration:** Incremental edition bump as each crate is touched in a feature cycle. Track in tech-debt register. Lowest disruption but longest tail.

**Option C — Architect decision on canonical timeline:** Architect issues an ADR decision or decision log entry specifying "all crates on edition 2024 by Wave 5" (or equivalent). Option A or B executes under that mandate.

### Resolution Criteria

Phase-5 architect adjudication picks an option and either executes the sweep or schedules it with a specific wave anchor. Re-check at session-reviewer post-cycle. The finding is RESOLVED when `crates/prism-bin/Cargo.toml:4` reads `edition = "2024"` (and any other crates found to lag are updated, per the chosen option).

---

## F-LP19-LOW-002 — VP-INDEX VP-PLUGIN-004 Dual-Emission Framing Diverges from BC-2.16.002 v1.12 Catalog Single-Emission Discipline

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP19-LOW-002 |
| **Severity** | LOW |
| **Confidence** | LOW |
| **Story source** | S-PLUGIN-PREREQ-D pass-19 |
| **Surfaced at** | Pass-19 (adversary fresh-context audit) |
| **Date routed** | 2026-05-14 |
| **Target** | Cross-doc framing reconciliation (VP-INDEX line 187 vs BC-2.16.002 v1.12 catalog single-emission discipline) |

### Evidence

- VP-INDEX v1.34 VP-PLUGIN-004 entry (line 187 area) contains prose framing that predates the Path B BC-2.16.002 universal-catalog scope decision established at fix-burst-8 (commit 4ed96e06, pass-9 F-LP9-MED-001 resolution).
- BC-2.16.002 v1.12 (SHA 84f58565) defines single-emission discipline for plugin events in its §Structured Event Catalog scope statement.
- The VP-INDEX VP-PLUGIN-004 summary prose describes a dual-emission verification pattern that may diverge from the catalog's single-emission discipline as currently codified in BC-2.16.002 v1.12.

### Why It Matters

VP-PLUGIN-004 (VP-149) is a verification property referenced by S-PLUGIN-PREREQ-D. If the VP-INDEX framing of VP-PLUGIN-004 predates or diverges from the BC-2.16.002 v1.12 catalog's current single-emission discipline, implementers and verifiers may apply inconsistent verification expectations — one derived from the VP-INDEX framing and one derived from the BC catalog. The divergence is LOW confidence because the VP-INDEX prose may represent an intentional scope distinction rather than a true conflict.

### Why It Is Out-of-Perimeter

VP-INDEX editing requires spec-steward or architect adjudication (cross-document governance). Story-scoped fix-bursts are not authorized to amend VP-INDEX content that does not trace directly to the story body under review. This finding routes to phase-5 per the CLAUDE.md boundaries clause.

### Resolution Criteria

Phase-5 architect or PO review of VP-INDEX VP-PLUGIN-004 framing against BC-2.16.002 v1.12 catalog single-emission discipline. Specific target: reconcile VP-INDEX line 187 prose to accurately reflect either (a) the single-emission discipline per BC-2.16.002 v1.12 §Catalog, or (b) document the intentional divergence with explicit rationale. Finding is RESOLVED when VP-INDEX framing is verified consistent with BC-2.16.002 v1.12 or the divergence is explicitly justified.

---

## F-LP22-OBS-001 — `PluginError` enum lacks `#[non_exhaustive]` despite story adding 4 new variants

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP22-OBS-001 |
| **Severity** | OBS (out-of-perimeter for story scope; substantive for phase-5) |
| **Confidence** | MEDIUM |
| **Story source** | S-PLUGIN-PREREQ-D pass-22 |
| **Surfaced at** | Pass-22 (adversary fresh-context audit) |
| **Date routed** | 2026-05-14 |
| **Target** | Architectural review — `PluginError` enum at `crates/prism-core/src/error.rs:983-984` lacks `#[non_exhaustive]` despite CLAUDE.md Conventions "All public TOML-deserialized types and pub-API surface types require `#[non_exhaustive]`" requirement |

### Evidence

- Story S-PLUGIN-PREREQ-D adds 4 new variants to `PluginError`: E-PLUGIN-013 (`PluginError::ManifestNameMissing`), E-PLUGIN-014 (`PluginError::ManifestVersionMalformed`), E-PLUGIN-015, E-PLUGIN-016.
- `SpecEngineError` at `crates/prism-spec-engine/src/error.rs` carries `#[non_exhaustive]`.
- `PluginError` at `crates/prism-core/src/error.rs:983-984` does **NOT** carry `#[non_exhaustive]`.
- CLAUDE.md Conventions: "All public TOML-deserialized types and pub-API surface types require `#[non_exhaustive]`."
- The current compile-fail gate at `tests/external/perimeter-violation/` enforces `EXPECTED=30` (30 `#[non_exhaustive]` types; established by S-PLUGIN-PREREQ-C AC-5). Adding `#[non_exhaustive]` to `PluginError` would change the count to 31, requiring a gate update.

### Why It Matters

`PluginError` is a pub-API surface type in `prism-core` that accepts new variants across story cycles (this story adds 4). Without `#[non_exhaustive]`, downstream match expressions on `PluginError` in external crates are NOT required to include a wildcard arm — meaning future variant additions to `PluginError` (in Wave 5, 6, etc.) will be source-breaking changes for external consumers. This is the exact asymmetry that `#[non_exhaustive]` is designed to prevent. `SpecEngineError` (prism-spec-engine) already carries this attribute; the asymmetry with `PluginError` (prism-core) creates inconsistent API stability guarantees across the two primary error types.

### Why It Is Out-of-Perimeter

Adding `#[non_exhaustive]` to `PluginError` is scope expansion into `prism-core` — the story's primary crate targets are `prism-spec-engine` and `prism-bin`. The compile-fail gate EXPECTED count (30 → 31) impact requires architect evaluation before execution. Story-scoped fix-bursts are not authorized to amend `prism-core` internals that affect the compile-fail gate count without explicit architect adjudication of the gate impact.

### Fix Options (for Phase-5 architect adjudication)

**Option A — Add `#[non_exhaustive]` to `PluginError` + update gate:** Add `#[non_exhaustive]` attribute to `PluginError` at `crates/prism-core/src/error.rs:983`. Update the compile-fail gate `EXPECTED` from 30 → 31 in `tests/external/perimeter-violation/`. Run `just check` to verify the gate update is coherent. This is the CLAUDE.md-compliant path.

**Option B — Explicit decision to keep `PluginError` exhaustive:** Architect issues a decision log entry documenting the rationale for keeping `PluginError` exhaustive (e.g., `PluginError` is considered a closed set with no external-crate consumers expected; internal-to-workspace match exhaustiveness is enforced via Rust compiler). This option requires explicit documentation that CLAUDE.md Conventions §`#[non_exhaustive]` does NOT apply to `PluginError` and why.

**Option C — Workspace-wide pub-API enum audit:** Before resolving `PluginError` in isolation, audit all pub-API enums in `prism-core` and `prism-spec-engine` for `#[non_exhaustive]` compliance. Update the gate EXPECTED count once for all discovered gaps rather than incrementally per story. This option provides the most complete resolution.

### Resolution Criteria

Phase-5 architect adjudication picks one of the three options above and executes it. The finding is RESOLVED when either: (a) `PluginError` carries `#[non_exhaustive]` and the compile-fail gate EXPECTED is updated to reflect the new count; OR (b) a decision log entry explicitly documents why `PluginError` is exempt from the CLAUDE.md `#[non_exhaustive]` requirement with a concrete rationale.

---

## F-LP25-OBS-001 — BC-2.17.002 v1.5 EC-17-007 Becomes Vacuously True Under Vec<String> Contract

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP25-OBS-001 |
| **Severity** | OBS (out-of-perimeter for story scope; cross-wave governance concern for phase-5) |
| **Confidence** | MEDIUM |
| **Story source** | S-PLUGIN-PREREQ-D pass-25 |
| **Surfaced at** | Pass-25 (adversary fresh-context idempotency audit) |
| **Date routed** | 2026-05-13 |
| **Target** | Phase-5 product-owner adjudication — BC-2.17.002 v1.5 §EC-17-007 cross-story/wave-gate concern |

### Evidence

BC-2.17.002 v1.5 §EC-17-007 (line 85) reads: "Plugin calls `host::http_request` when no allowlist is configured | Request allowed to any URL (open by default); audit log entry created"

Under the PREREQ-D Vec<String> field-type contract (established at fix-burst-4, enforced through fix-burst-22 via F-LP23-HIGH-001 8-site type-contract correction):

- `allowed_urls` field type is `Vec<String>` (non-Option, always present)
- An empty `Vec<String>` (`vec![]`) represents "no URLs allowed" (all requests blocked per AC-7 canonical framing)
- A non-empty `Vec<String>` represents an explicit allowlist
- There is no representational state for "no allowlist configured" — the Vec is always present in the configuration structure

Therefore, EC-17-007's framing of "no allowlist configured" becomes representationally impossible after PREREQ-D ships: the `allowed_urls` field cannot be absent; it is always configured (always a Vec<String>). EC-17-007 describes a state the Vec<String> type contract makes impossible to express.

### Why It Matters

If EC-17-007 is not updated to reflect the Vec<String> contract, an implementer reading BC-2.17.002 §EC-17-007 post-PREREQ-D may believe there is an "open by default" mode that is achievable via configuration — but the Vec<String> type makes this mode unreachable. The gap is a cross-BC governance inconsistency between:

1. PREREQ-D AC-7 / BC-2.17.002's Vec<String> field-type contract (enforced)
2. BC-2.17.002 §EC-17-007 prose framing that describes an "absent allowlist" state (contradicted by the Vec<String> contract)

### Why It Is Out-of-Perimeter

BC-2.17.002 amendment requires PO adjudication and is a cross-story governance gap. The story-scoped fix-burst cannot amend BC-2.17.002 without explicit scope expansion. This finding is also not blocking PREREQ-D implementation since the Vec<String> contract itself is correctly specified in the story.

### Fix Options (for Phase-5 PO adjudication)

**Option A — Update EC-17-007 to reflect Vec<String> semantics:** Rewrite EC-17-007 to describe the "empty Vec<String>" state (not "absent allowlist"). New framing: "Plugin calls `host::http_request` when `allowed_urls` is empty (`vec![]`) | All URLs blocked (empty list = deny-all); `plugin_http_request_denied` audit log entry emitted." This accurately describes the post-PREREQ-D behavior for the "no-URLs-configured" case.

**Option B — Remove EC-17-007 as vacuously obsolete:** After PREREQ-D ships, EC-17-007 describes an unreachable state. Remove EC-17-007 from BC-2.17.002 and update any story cross-references that cite it.

**Option C — Grandfather EC-17-007 with an explicit note:** Add a note to EC-17-007 stating "NOTE: With PREREQ-D Vec<String> contract, this condition is representationally impossible; EC-17-007 is preserved for historical audit trail only. See AC-7 for current allowlist behavior."

### Resolution Criteria

Before Phase 5 convergence can be declared: BC-2.17.002 §EC-17-007 entry is updated so that its framing is consistent with the PREREQ-D Vec<String> type contract, OR an explicit decision log entry documents why EC-17-007 is intentionally retained in its current form despite the type-contract contradiction.

---

## F-LP28-OBS-001 [process-gap] — E-INT-001 Absent from error-taxonomy.md

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP28-OBS-001 |
| **Severity** | OBS (out-of-perimeter for story scope; substantive for phase-5) |
| **Confidence** | HIGH |
| **Story source** | S-PLUGIN-PREREQ-D |
| **Surfaced at** | Pass-28 (adversary fresh-context audit) |
| **Date routed** | 2026-05-13 |
| **Target** | Phase-5 product-owner adjudication — error-taxonomy.md E-INT namespace entirely absent |

### Evidence

- Story S-PLUGIN-PREREQ-D body line 393 cites E-INT-001 referencing `error.rs:881-883`.
- Grep confirms E-INT-001 IS present in `crates/prism-spec-engine/src/error.rs` at lines 881-883 (real code — not fabricated).
- E-INT-001 does NOT appear in `.factory/specs/prd-supplements/error-taxonomy.md`.
- The E-INT-NNN namespace is entirely absent from error-taxonomy.md (no E-INT rows, no E-INT section).

### Why It Matters

error-taxonomy.md is the canonical cross-reference for all project error codes (per CLAUDE.md §Error handling). If E-INT-001 exists in production code but is absent from the taxonomy, implementers cannot look up the code semantics, audit role, or handling guidance via the spec. The gap means the error taxonomy is incomplete with respect to the current codebase.

This is a pre-existing gap — E-INT-001 was not introduced by S-PLUGIN-PREREQ-D. The story correctly cites E-INT-001 as an existing code from the codebase. The defect is in the taxonomy artifact, not in the story.

### Why It Is Out-of-Perimeter

Adding a new namespace (E-INT) to error-taxonomy.md requires product-owner adjudication and is broader than the story scope. A full E-INT namespace survey (how many E-INT-NNN codes exist in the codebase, their semantics, audit roles) is a cross-cutting taxonomy task. Story-scoped fix-bursts cannot add new namespaces to the taxonomy without explicit PO authorization.

### Fix Options (for Phase-5 PO adjudication)

**Option A — E-INT namespace audit + taxonomy append:** Survey all E-INT-NNN codes in the codebase (`grep -r "E-INT-" crates/`). Add a new E-INT namespace section to error-taxonomy.md with a row for each discovered code. This is the CLAUDE.md-compliant path.

**Option B — Architectural rename:** If E-INT codes are internal-to-crate error variants that are not intended to be user-facing or cross-crate, determine whether they should be renamed to a more appropriate namespace (E-INTERNAL, E-PIPELINE, etc.) and then added to the taxonomy under the correct name.

**Option C — Decision to keep E-INT internal (not in taxonomy):** If the architect determines that E-INT codes are purely internal implementation detail not subject to error-taxonomy documentation, record an explicit decision log entry stating this exception. This would require updating CLAUDE.md §Error handling to carve out internal codes.

### Resolution Criteria

Before Phase 5 convergence can be declared: either (a) E-INT-001 (and any other discovered E-INT-NNN codes) are added to error-taxonomy.md with complete entries (code, message template, subsystem, BC anchor, audit role); OR (b) a decision log entry documents why E-INT codes are exempt from the error taxonomy requirement with concrete rationale.

---

## OBS-LP35-001 — Pre-AC-7 "not-None" Option-Semantics in verification-architecture.md:282 + ADR-023:732-733 (Out-of-Perimeter — Architecture Layer)

| Field | Value |
|-------|-------|
| **Finding ID** | OBS-LP35-001 |
| **Severity** | OBS (out-of-story-perimeter for S-PLUGIN-PREREQ-D cascade; substantive for phase-5) |
| **Confidence** | HIGH |
| **Story source** | S-PLUGIN-PREREQ-D |
| **Surfaced at** | Pass-35 (adversary fresh-context audit) |
| **Date routed** | 2026-05-14 |
| **Target** | Phase-5 architect adjudication — verification-architecture.md:282 + ADR-023:732-733 carry pre-AC-7 Option-semantics for VP-PLUGIN-007 `allowed_urls` |

### Evidence

- **verification-architecture.md:282** — VP-PLUGIN-007 prose description in the verification
  architecture carries "not-None" or "allowlist not-None" Option-presence framing for
  `allowed_urls`. This framing predates AC-7, which established `allowed_urls: Vec<String>`
  (never `Option`).
- **ADR-023-plugin-only-sensor-architecture.md:732-733** — ADR-023 prose discussion of the
  `allowed_urls` type contract retains the same pre-AC-7 Option-semantics framing at lines
  732-733.

### Propagation Context

The pre-AC-7 "not-None" / "allowlist not-None" phrasing for `allowed_urls` has been propagated
across 7 sites total (see OBS-LP35-001 in pass-35 report for full table). The in-perimeter
sites (VP-INDEX, story §References, BC-2.17.007) are being addressed by fix-burst-32/33.
These two architecture-layer sites are out-of-story-perimeter and require architect adjudication.

### Why It Is Out-of-Perimeter

Architecture documents (verification-architecture.md, ADR-023) are owned by the `architect`
specialist. Their amendment requires architect adjudication and is outside the story-scope
fix-burst discipline. They cannot be corrected by product-owner or story-writer dispatch
during this cascade.

### Fix (for Phase-5 Architect Adjudication)

Architect rewrites verification-architecture.md:282 and ADR-023:732-733 to replace
Option-presence framing ("not-None", "allowlist not-None") with Vec<String>-semantics framing
consistent with AC-7 + AC-17:
- "Empty Vec<String> → deny-all (default)"
- "Non-empty Vec<String> → URL-set active"
- Never `Option<Vec<String>>`; the None state is type-system-impossible.

### Resolution Criteria

Before Phase 5 adversarial convergence: architect rewrites both locations to use the
Vec<String>-semantics framing established by AC-7 + AC-17. The verification-architecture
VP-PLUGIN-007 entry and ADR-023 prose should match the phrasing in BC-2.17.007 (post
fix-burst-33 v1.3) and VP-INDEX (post D-533 v1.35).

---

## OBS-LP36-002 — BC-INDEX prose vs frontmatter count drift (System-level; Pre-existing)

| Field | Value |
|-------|-------|
| **Finding ID** | OBS-LP36-002 |
| **Severity** | OBS (system-level; pre-existing; deferred phase-5) |
| **Confidence** | HIGH |
| **Story source** | S-PLUGIN-PREREQ-D |
| **Surfaced at** | Pass-36 (adversary fresh-context audit) |
| **Date routed** | 2026-05-14 |
| **Target** | Phase-5 architect adjudication — BC-INDEX.md has three independent count claims that disagree with each other |

### Evidence

BC-INDEX.md frontmatter line 4 reads:
`total_contracts: 236, active_contracts: 229, draft_contracts: 6, deprecated_contracts: 3`

Note: 229 + 6 + 3 = 238, not 236 — the frontmatter subcounts do not sum to the total.

BC-INDEX.md prose at lines 17 and 19-20 reads:
`235 contracts total (227 active + 6 draft + 2 deprecated)`

Three independent count claims disagree:
- **Frontmatter declared total**: 236 (with internal inconsistency: subcounts sum to 238)
- **Prose total**: 235
- **Prose subcounts**: 227 active + 6 draft + 2 deprecated = 235

### Why It Matters

BC-INDEX is the authoritative count source for all BC-count-propagation sweeps (TD-VSDD-060
discipline). If the three count fields disagree, count-propagation sweeps across STATE.md,
ARCH-INDEX, STORY-INDEX, and prd.md may anchor to any of the three values and produce
inconsistent results. The count-propagation sweep in D-535 anchored to `total_contracts: 236`
(frontmatter) — but a verifier reading the prose would see 235.

### Why It Is Out-of-Perimeter

A correct fix requires a workspace-wide BC enumeration (`find .factory/specs/behavioral-contracts -name "BC-*.md" | wc -l`) to determine the authoritative count. This is a system-level architect task that is broader than story-scope fix-bursts. The drift is pre-existing and was not introduced by this cascade.

### Fix Options (for Phase-5 Architect Adjudication)

**Option A — Workspace enumeration + single-source reconciliation:** Run `find .factory/specs/behavioral-contracts -name "BC-*.md" | wc -l` to get the actual count. Update frontmatter + prose to match. Reconcile subcounts (active/draft/deprecated) by examining BC frontmatter `lifecycle_status` fields. This is the CLAUDE.md-compliant path.

**Option B — Frontmatter-wins policy:** Declare frontmatter as the authoritative source; update prose to match frontmatter values. Run the workspace enumeration to verify frontmatter is correct.

**Option C — Prose-wins policy:** Declare prose as the authoritative source; update frontmatter to match prose values. Run the workspace enumeration to verify prose is correct.

### Resolution Criteria

Before Phase 5 adversarial convergence: BC-INDEX.md has exactly one consistent count claim across frontmatter and prose, and the declared count matches the actual count of BC-*.md files in `.factory/specs/behavioral-contracts/`.
