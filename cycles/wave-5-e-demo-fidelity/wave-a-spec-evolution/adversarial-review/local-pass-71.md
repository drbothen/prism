---
document_type: adversarial-review
pass: 71
scope: local
frozen_head: "5e1381e1a"
timestamp: 2026-07-30T00:00:00Z
CLEAN_strict: "no"
CLEAN_PR_merge: "no"
finding_counts: {HIGH: 5, MED: 7, LOW: 3, total: 15}
novelty: HIGH
---

# LOCAL Adversary Pass 71 — Wave-A Spec-Evolution Cascade

**CLEAN (strict): no**
**CLEAN (PR-merge): no**

**15 findings: 5 HIGH · 7 MEDIUM · 3 LOW.** Novelty: **HIGH**. Frozen HEAD reviewed: `5e1381e1a`.

Pass-71 independently confirmed 6 pass-70 closures by recomputing them from source — notably the `&limit=200` pagination arithmetic (`get_search` `size=200`, `start_offset=0`, `.take(200)` reaches tombstones at positions 180..200) and the exhaustive seven-caller `build_asset` status enumeration.

---

## Findings Ledger

### F-WASE-P71-HIGH-001 — BC-2.02.014 retains 12 LIVE stale `ADR-057 v0.7` pins; one is a content falsification (POL-29 9a twin asymmetry)
**Severity:** HIGH
**Artifact:** `BC-2.02.014-armis-device-activity-filter-required-push-down-fetch.md` (v1.8)
**Occurrence count:** 12 LIVE normative + 4 immutable-changelog.

ADR-057 is at **v0.9**. All 12 LIVE sites read `v0.7`. One is a content falsification: a TOML block claiming "transcribed verbatim from v0.7 §D5" while containing a `required_filters` line first added in v0.8.

Per-site (12 LIVE normative):
1. §Description — `…confirmed capability gap (ADR-057 v0.7 §D6)`
2. §Architecture Anchors — `…push-down-grammar.md (v0.7) — authority for the ${query.filter.device_id} grammar`
3. §Story Anchor — `UNBLOCKED per ADR-057 v0.7 §Consequences (C1)`
4. §Preconditions — response_path cite (v0.7)
5–12. Additional §Postconditions and §TOML Contract sites (8 further occurrences).

**Root cause:** HIGH-001 is the empirical proof of HIGH-003 — the governance burst retired POL-23's grep before POL-39's replacement exists, and 12 stale pins survived.

**Closing leg:** → **FB100 CLOSED**

---

### F-WASE-P71-HIGH-002 — ADR-057 §D7 and §C7 assert E-SPEC-029 registration as still pending; discharged in the same commit
**Severity:** HIGH
**Artifact:** `ADR-057-armis-activity-per-device-push-down-grammar.md` (v0.9)
**Occurrence count:** 2 LIVE normative sites.

§D7 and §C7 both contained language asserting an E-SPEC registration obligation that was discharged in the same commit (FB95/FB97 registered E-SPEC-029 in error-taxonomy.md).

**Closing leg:** → **FB99 CLOSED** (both marked DISCHARGED against ACTIVITY-001 AC-004/RG-004)

---

### F-WASE-P71-HIGH-003 — [process-gap] POL-23 detector retired before POL-39 L11 gate exists; ~5,679 grandfathered pins with zero detection mechanism
**Severity:** HIGH
**Artifact:** `policies.yaml` (v1.40) — POL-23 `description` + `verification_steps[0]`; POL-39 `enforced_by` + `implementation_target`
**Occurrence count:** 1 policy amendment, 2 LIVE normative sites.

POL-23's `description` stated it was "Retired by POL-39" on the false premise that no prose pins remained. POL-39's own rationale documents ~5,679 grandfathered pins across 219 files — those have zero detection now. `records-lint-L11` sits in POL-39's `enforced_by` array indistinguishable from live mechanisms, but the check does not exist.

**Fix:** Reinstate POL-23's grep step as ACTIVE-DURING-TRANSITION. Retirement condition: retires ONLY when BOTH `S-MAINT-ANTIPIN-SWEEP-001` AND `S-MAINT-ANTIPIN-SWEEP-002` have merged AND records-lint L11 is deployed.

**Closing leg:** → **FB98 CLOSED** (reinstated ACTIVE-DURING-TRANSITION, pattern-based + bidirectional, retirement conditioned on BOTH sweep stories merging AND L11 deployed)

---

### F-WASE-P71-HIGH-004 — SPEC-001 T-01 instructs `device_type`; BC wire-key `type` fix (MED-001) applied to BC copy-source only, not downstream executable copy (POL-29 9b)
**Severity:** HIGH
**Artifact:** `S-WAVE-A-ARMIS-SPEC-001-armis-sensor-spec-six-column-additions-and-device-cves-first-source-path.md` (v1.8) §Tasks → T-01
**Occurrence count:** 1 LIVE normative site.

§Tasks T-01 still instructs the implementer to write `device_type` in their TOML output. The wire-key correction (`device_type` → `type` per serde rename) was applied to BC-2.02.006 §TOML Contract in F-WASE-P70-MED-001 but the downstream executable copy in the story was not swept.

**Closing leg:** → **FB101 CLOSED**

---

### F-WASE-P71-HIGH-005 — AC-016/RG-016 unsatisfiable: a single `ArmisState` cannot produce both `status: "Normal"` and `status: "At Risk"` records
**Severity:** HIGH
**Artifact:** `BC-2.02.006-armis-field-mapping.md` (v1.16) §Generated-Records Path Coverage — AC-016 anchor entry; `S-WAVE-A-ARMIS-SPEC-001` (v1.8) — AC-016/RG-016 anchor
**Occurrence count:** 2 LIVE normative sites (BC + story).

AC-016 requires ONE `GET /api/v1/search?aql=in:devices` response against a single `ArmisState` to satisfy both: (1) a record with `risk_factors` present (`status: "At Risk"` branch) and (2) a record with `risk_factors` absent (`status: "Normal"` branch). A single `ArmisState` carries one archetype; the two status classes are strictly disjoint. No single state can satisfy both obligations.

**Fix:** Two-`ArmisState` construction — one per archetype — with an ordered `.chain()` call.

**Closing leg:** → **FB100 (BC) + FB101 (story) CLOSED** via a two-`ArmisState` construction

---

### F-WASE-P71-MED-001 — POL-39 self-non-compliant: `description` contains the literal banned form `ADR-028 v1.28`
**Severity:** MEDIUM
**Artifact:** `policies.yaml` (v1.40), POL-39 `description`
**Occurrence count:** 1 LIVE normative + 1 immutable-record.

POL-39's negative-example statement contained `ADR-028 v1.28` — the exact banned form the policy prohibits. Fix: replace with `ADR-028 §D1` (section-anchor form).

**Closing leg:** → **FB98 CLOSED** (placeholder convention applied)

---

### F-WASE-P71-MED-002 — [process-gap] POL-39 `enforced_by` names a nonexistent gate with no `-proposed` marker and no interim manual-verification step; POL-31 is the correct exemplar
**Severity:** MEDIUM
**Artifact:** `policies.yaml` (v1.40), POL-39 `enforced_by` + `verification_steps`
**Occurrence count:** 2 LIVE normative sites.

`records-lint-L11` in `enforced_by` is indistinguishable from live mechanisms but does not exist. POL-31 in the same registry is the correct exemplar — it uses a `-proposed` suffix and provides an explicit interim manual step.

**Closing leg:** → **FB98 CLOSED** (mirrored POL-31's three conventions; `records-lint-L11` → `records-lint-L11-proposed`)

---

### F-WASE-P71-MED-003 — POL-39 exemption tiers narrower than `S-MAINT-L11-GATE-001` ACs; will produce false-positive L11 firings
**Severity:** MEDIUM
**Artifacts:** `policies.yaml` POL-39 `verification_steps` (exemptions 2 and 3); `S-MAINT-L11-GATE-001` AC-003, AC-004, EC-004
**Occurrence count:** 2 divergences.

POL-39 exempts "changelog rows" while the story exempts "changelog rows AND §Changelog section headers"; POL-39 exempts "version frontmatter fields" while the story exempts "frontmatter fields (any key) plus entire `---` frontmatter blocks." The narrower policy will cause false-positive firings on the very artifacts the burst created.

**Closing leg:** → **FB98 CLOSED** (exemption tiers aligned to story ACs)

---

### F-WASE-P71-MED-004 — Superseded FB93 blockquote retained verbatim in both BC-2.02.006 and BC-2.02.014 with no in-artifact supersession marker
**Severity:** MEDIUM
**Artifacts:** `BC-2.02.006` (v1.16) §TOML Contract; `BC-2.02.014` (v1.8) §TOML Contract
**Occurrence count:** 2 LIVE normative sites (byte-identical).

The FB93 blockquote instructing future authors to consult BC-2.02.006 §TOML Contract "as the authoritative copy" was superseded by the version-free anchor convention in the same burst but no in-artifact marker was added to either BC.

**Closing leg:** → **FB100 CLOSED** (supersession marker added in-scope; full removal stays with `S-MAINT-ANTIPIN-SWEEP-002` AC-005)

---

### F-WASE-P71-MED-005 — ADR-023 and ADR-028 `anchor_stories` annotations misquote story §Authority text and pin superseded ADR versions
**Severity:** MEDIUM
**Artifacts:** `ADR-023` (v1.25) frontmatter `anchor_stories` annotation; `ADR-028` (v1.29) frontmatter `anchor_stories` annotation
**Occurrence count:** 2 LIVE normative sites.

The SAC-2 verification annotations for `S-WAVE-A-ARMIS-SPEC-001` quoted §Authority text that FB96 revised in the same commit, and pinned `v1.25`/`v1.29` which the burst itself superseded.

**Closing leg:** → **FB99 CLOSED**

---

### F-WASE-P71-MED-006 — BC-2.02.014 §Architecture Anchors cites ADR-057 §D4/§D5 but omits §D7, the ratified authority for the required-filter gate the BC now contracts
**Severity:** MEDIUM
**Artifact:** `BC-2.02.014` (v1.8) §Architecture Anchors
**Occurrence count:** 1 LIVE normative site (omission).

§Architecture Anchors describes ADR-057 as authority for the push-down grammar and pre-seed mechanism (§D4, §D5) only. §D7 — which ratifies the `required_filters` gate and `E-SPEC-029` — was added in v0.8 and is now the authority for the contract's required-filter obligation. Its omission leaves the BC citing a stale architectural scope.

**Closing leg:** → **FB100 CLOSED**

---

### F-WASE-P71-MED-007 — Dependency-graph non-reciprocity: `S-MAINT-L11-GATE-001` declares `blocks` for both ANTIPIN stories, but neither carries the reciprocal `depends_on`
**Severity:** MEDIUM
**Artifacts:** `S-MAINT-L11-GATE-001` frontmatter `blocks`; `S-MAINT-ANTIPIN-SWEEP-001` frontmatter `depends_on`; `S-MAINT-ANTIPIN-SWEEP-002` frontmatter `depends_on`
**Occurrence count:** 2 missing reciprocal edges of 3 declared.

| Declared edge | Reciprocal present? |
|---|---|
| L11-GATE-001 `blocks: S-MAINT-CAPREF-SWEEP-001` | ✅ CAPREF `depends_on: [S-MAINT-L11-GATE-001]` |
| L11-GATE-001 `blocks: S-MAINT-ANTIPIN-SWEEP-001` | ❌ ANTIPIN-001 `depends_on: [S-MAINT-CAPREF-SWEEP-001]` only |
| L11-GATE-001 `blocks: S-MAINT-ANTIPIN-SWEEP-002` | ❌ ANTIPIN-002 `depends_on: [S-MAINT-ANTIPIN-SWEEP-001]` only |

**Closing leg:** → **FB101 CLOSED**

---

### F-WASE-P71-LOW-001 — ACTIVITY-001 mis-dates the §D7 ratification as 2026-07-27; ADR-057 §D7 says 2026-07-30
**Severity:** LOW
**Artifact:** `S-WAVE-A-ARMIS-ACTIVITY-001` (v1.8) §Tasks → T-IMPL-02
**Occurrence count:** 1 LIVE normative site.

T-IMPL-02 referenced `ADR-057 §D7 (ratified 2026-07-27)` while ADR-057's frontmatter and §D7 changelog row both record `2026-07-30`.

**Closing leg:** → **FB101 CLOSED**

---

### F-WASE-P71-LOW-002 — `S-MAINT-L11-GATE-001` AC-007 CLAUDE.md self-compliance has no mechanical verification; no RG-007 exists
**Severity:** LOW
**Artifact:** `S-MAINT-L11-GATE-001` (v1.0) AC-007, MERGE-GATE-CLAUDE-MD-SELF-COMPLIANT
**Occurrence count:** 1 structural gap.

AC-007 states the L11 gate must be self-compliant (no narrative version pins in CLAUDE.md) but carries no RG entry and no mechanical verification step. The AC↔RG bijection is broken for this acceptance criterion.

**Closing leg:** → **FB101 CLOSED** (RG-007 added, probe total ≥41, AC↔RG bijection restored)

---

### F-WASE-P71-LOW-003 — ADR-028 `anchor_stories` retains 7 entries with no §Authority grounding; pending intent verification
**Severity:** LOW (pending intent verification)
**Artifact:** `ADR-028` (v1.29) frontmatter `anchor_stories`
**Occurrence count:** 7 entries.

`anchor_stories: [PLUGIN-MIGRATION-001-D, PLUGIN-MIGRATION-001-A, PLUGIN-MIGRATION-001-B, PLUGIN-MIGRATION-001-C, PLUGIN-MIGRATION-001-E, S-DEMO-001, S-DEMO-002, S-WAVE-A-ARMIS-SPEC-001]`. Verified: only 11 story files corpus-wide contain a `## Authority` section at all, and none of the seven pre-existing entries is among them. SAC-2 rule 2 requires population from §Authority citations.

**Adjudication:** The seven predate the §Authority convention entirely — deliberate legacy-anchor records rather than prose-mention pollution. Retain with per-entry legacy annotations.

**Closing leg:** → **FB99 ADJUDICATED RETAIN** with per-entry legacy annotations naming each ADR body section

---

## Probe Verdicts

**SAP-1** PASS (perimeter `.factory/`-only; zero `crates/` files touched, so no new emission sites — the adversary explicitly did NOT re-verify the 232 pre-existing `event_type=` occurrences and makes no claim about them).

**SAP-2** PASS (DTU files read directly: `types.rs`, `routes/devices.rs`, `routes/search.rs`, `generator.rs`, `clone.rs`, `fixtures/device-activity.json`, `crates/prism-sensors/specs/armis.sensor.toml`; activity surface Rule-6-verified at the emission site; fixture IDs `d-001 d-002 d-005 d-013 d-015 d-020 d-023 d-024` confirmed against 10 records over exactly those 8 IDs).

**SAP-3** PASS except RG-016 (HIGH-005 — AC-016/RG-016 unsatisfiable as written).

**SAC-1** PASS across six strict stories; banned-construct probe CLEAN (5 `RED_RATIO` mentions all legitimate — all are orchestrator-deferred-metric form, none are authored-time computed values).

**SAC-2** PASS on the FB94 additions.

**TD-VSDD-091/L9** PASS.

**POL-24** PASS (E-SPEC-029 byte-identical across taxonomy, ADR §D7 Rule 2, §C7).

**POL-36** PASS.

**POL-29 as-found: 9a FAIL ×2, 9b FAIL 1-of-3, 9c PASS with one undischarged-marker gap** — all remediated by FB98–FB101.

---

## Structural Lesson

**De-pinning bursts create inverted twin asymmetry.** When a burst removes pins from one twin, a value-based POL-29 9a grep returns zero hits in the *clean* member and cannot detect that the *dirty* member was never swept. Detection requires grepping the pin *pattern* in both twins independently — absence of the changed string in the clean twin is the failure mode, not evidence of cleanliness. Now encoded in POL-23's amended ACTIVE-DURING-TRANSITION step.

**Codification candidate:** a policy-registry invariant that `enforced_by` may not name a nonexistent mechanism unless a predecessor remains ACTIVE or an interim-manual step is present (POL-31 exemplar).

---

## Counts NOT Verified (do not treat as established)

- CAPREF-SWEEP-001's "102 citations / 69 rows / 72 files" — verified only for internal arithmetic consistency, not against the corpus.
- POL-39's "~2,434 / 364 / 15 / ~2,866" pin figures — verified only for internal arithmetic consistency (5,679 across 219 files), not against the corpus.
