---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 36
target_sha: 95d46be2
story_content_sha: TBD (story v1.32; recompute after commit)
bc_2_17_007_content_sha: TBD (v1.3; post-fix-burst-33 edit)
bc_2_17_002_content_sha: 898ad6282b8f514e5b378b483932ea40f3a05a2c
bc_2_16_002_content_sha: 84f58565
error_taxonomy_content_sha: TBD (v1.22; post-fix-burst-33 edit)
base_sha: 95d46be2
verdict: BLOCKED
streak: "0/3 HOLD (pass-36 BLOCKED: 0 CRIT + 0 HIGH + 1 MED + 1 LOW + 2 OBS)"
finding_summary: {CRITICAL: 0, HIGH: 0, MEDIUM: 1, LOW: 1, OBS: 2}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7, pass-8, pass-9, pass-10, pass-11, pass-12, pass-13, pass-14, pass-15, pass-16, pass-17, pass-18, pass-19, pass-20, pass-21, pass-22, pass-23, pass-24, pass-25, pass-26, pass-27, pass-28, pass-29, pass-30, pass-31, pass-32, pass-33, pass-34, pass-35]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6, fix-burst-7, fix-burst-8, fix-burst-9, fix-burst-10, fix-burst-11, fix-burst-12, fix-burst-13, fix-burst-14, fix-burst-15, fix-burst-16, fix-burst-17, fix-burst-18, fix-burst-19, fix-burst-20, fix-burst-21, fix-burst-22, fix-burst-23, fix-burst-24, fix-burst-25, fix-burst-26, fix-burst-27, fix-burst-28, fix-burst-29, fix-burst-30, fix-burst-31, fix-burst-32, fix-burst-33]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → 5 → 1 → 1 → 3 → 4 → 5 → 5 → 5 → 2"
idempotency_check: false
post_fix_check: true
post_fix_target: "fix-burst-33 (F-LP35-MED-001 + F-LP35-MED-002 — all 2 in-scope closed)"
trajectory_note: "Pass-36 drops to 2 findings (0 CRIT + 0 HIGH + 1 MED + 1 LOW + 2 OBS). This is a DROP from 5 to 2 — the first genuine decrease in 3 passes. Fix-burst-33 closures verified HELD: BC-2.17.007:138+161 pre-AC-7 'not-None'/'allowlist not-None' Option-semantics successfully rewritten to post-AC-7 Vec<String>-semantics framing; error-taxonomy.md:464 superseded §Canonical Structured Event Catalog form correctly replaced with §Postconditions ancestry form. New findings are: F-LP36-MED-001 — fix-burst-33 itself introduced frontmatter staleness on BC-2.17.007 (TD-VSDD-060 sibling-site sweep gap on the frontmatter axis: modified: + timestamp: fields not updated when v1.2→v1.3 was bumped; Codification #17 pattern applied to frontmatter coherence, POL-23 candidate extension); F-LP36-LOW-001 — BC-2.17.007:138 VP-PLUGIN-007 gate-rationale description semantically mis-anchors to AC-7 (downstream consumer) instead of canonical AC-5 (manifest gate). OBS-LP36-001 is a process-gap codification candidate #24 (frontmatter-modified sibling-sweep — 2nd recurrence of this specific miss). OBS-LP36-002 is BC-INDEX prose vs frontmatter count drift — a pre-existing system-level gap deferred phase-5. Trajectory 5→5→5→2 DROP is the first convergence signal since pass-32 reset. Fix-burst-34 routes product-owner single-file (BC-2.17.007 v1.3→v1.4 frontmatter modified+timestamp + line 138 anchor rewrite + v1.4 changelog row)."
producer: "adversary (vsdd-factory; reified by state-manager per established cascade convention)"
---

# Adversarial Pass 36 — S-PLUGIN-PREREQ-D

**Verdict: BLOCKED (0 CRIT + 0 HIGH + 1 MED + 1 LOW + 2 OBS)**

**Context:** This is a post-fix-burst-33 fresh-context pass. Fix-burst-33 closed 2 in-scope
findings from pass-35 (F-LP35-MED-001 BC-2.17.007 v1.2→v1.3 lines 138+161 pre-AC-7
"allowed_urls = None"/"allowlist not-None" Option-semantics rewritten to post-AC-7 Vec<String>
framing; F-LP35-MED-002 error-taxonomy.md v1.21→v1.22 line 464 §Canonical Structured Event
Catalog phantom-section anchor rewritten to §Postconditions ancestry form) via product-owner
edits with state-manager BC-INDEX bump. The expected outcome was CLEAN (0/3 → 1/3).
Actual: BLOCKED by 1 MED + 1 LOW + 2 OBS. Net in-scope actionable: 2 findings.
Streak holds at 0/3 per BC-5.39.001.

Trajectory pass-25..pass-36: 4 → 1 → 4 → 5 → 1 → 1 → 3 → 4 → 5 → 5 → 5 → **2** —
DROP from 5 to 2. This is the first genuine finding-count decrease in 3 passes (passes 34,
35, 36 had been 5/5/5 flat). The trajectory DROP is a convergence signal.

**Fix-burst-33 closure verification:**

Both closures from fix-burst-33 held:
- F-LP35-MED-001 (BC-2.17.007 pre-AC-7 "not-None" / "allowlist not-None" Option-semantics):
  BC-2.17.007 lines 138+161 now use post-AC-7 Vec<String>-semantics framing:
  - Line 138: "explicit allowed_urls: Vec<String> field — manifest omission is a hard load
    rejection (E-PLUGIN-013) per AC-5 manifest gate; default-deny consumer is AC-7"
    (note: the fix correctly updated the description body at line 161 to Vec<String> framing,
    but the VP-PLUGIN-007 gate-rationale citation at line 138 still uses "per AC-7 default-deny"
    as the anchor — see F-LP36-LOW-001 below for this residual semantic issue)
  - Line 161: "explicit list under AC-7 default-deny semantics" CLEAN — Vec<String>-semantics
    framing confirmed.
- F-LP35-MED-002 (error-taxonomy.md:464 §Canonical Structured Event Catalog phantom anchor):
  Line 464 now reads "§Postconditions (Canonical Structured Event Catalog bullet, v1.12) row
  pipeline_max_requests_exceeded". CLEAN — §Postconditions ancestry form confirmed.

**Propagation analysis (F-LP35-MED-001 closure scope):**

The closure scope for F-LP35-MED-001 was: BC-2.17.007 body lines 138 and 161 only.
The fix-burst-33 product-owner correctly rewrote the substantive Vec<String> content at
both targeted body lines and bumped the version from v1.2 to v1.3 with a §Changelog row
dated 2026-05-14. However, the frontmatter fields `modified:` and `timestamp:` were not
updated to match. A sibling-sweep on BC-2.17.007 after the v1.3 bump reveals:

- **Line 14**: `modified: 2026-05-13` — stale (v1.2 date; v1.3 changelog row dated 2026-05-14)
- **Line 7**: `timestamp: 2026-05-13T00:00:00Z` — stale (v1.2 timestamp; v1.3 changelog row
  dated 2026-05-14)

Canonical pattern confirmation: sibling BC-2.17.002:14 reads `modified: 2026-05-14` (set at
fix-burst-30, D-539), confirming that BC version bumps DO update the `modified:` frontmatter
field. The pattern was not applied to BC-2.17.007 v1.3. This is a TD-VSDD-060 sibling-site
sweep gap on the frontmatter axis — the product-owner's sweep covered body lines (correctly)
but did not extend to the YAML frontmatter fields.

---

## Findings

### F-LP36-MED-001 — BC-2.17.007 frontmatter `modified:` + `timestamp:` stale relative to v1.3 fix-burst-33 edit

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP36-MED-001 |
| **Severity** | MEDIUM |
| **Confidence** | HIGH |
| **Location** | `.factory/specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md` |
| **Lines** | 7 (`timestamp: 2026-05-13T00:00:00Z`), 14 (`modified: 2026-05-13`) |
| **Policy** | Codification #17 (entity existence / frontmatter-body coherence); POL-23 candidate extension (frontmatter axis) |

**Evidence:**

Fix-burst-33 (D-535) bumped BC-2.17.007 v1.2→v1.3 with a §Changelog row dated 2026-05-14
at line 179 of the file. The body content edits (lines 138+161) are correct and reflect the
post-AC-7 Vec<String> type contract. However the frontmatter fields remain at the v1.2 values:

```
# Line 7 (stale)
timestamp: 2026-05-13T00:00:00Z

# Line 14 (stale)
modified: 2026-05-13
```

The v1.3 §Changelog row explicitly states the change date as 2026-05-14. The two frontmatter
fields have not been updated to match.

**Canonical pattern confirmation:**

BC-2.17.002 was edited during fix-burst-30 (D-539 equivalent). Its frontmatter reads:
`modified: 2026-05-14`. This confirms the canonical pattern: edits bump `modified:` to the
edit date. BC-2.17.007:14 should read `modified: 2026-05-14` and BC-2.17.007:7 should read
`timestamp: 2026-05-14T00:00:00Z`.

**Root cause:**

TD-VSDD-060 sibling-site sweep gap on the frontmatter axis. Fix-burst-33 correctly swept the
body lines (138, 161) but did not extend the sweep to the YAML frontmatter fields
(`modified:`, `timestamp:`). This is the 2nd recurrence of the frontmatter-modified-field miss
pattern (1st: fix-burst-7-stage-1A lifecycle_status frontmatter field miss).

**Proposed fix (product-owner):**

Update BC-2.17.007:
- Line 7: `timestamp: 2026-05-14T00:00:00Z`
- Line 14: `modified: 2026-05-14`

No version bump required (this is a frontmatter-coherence correction of an already-committed
v1.3 edit; the v1.3 §Changelog row already correctly records the date). However, if the
product-owner determines a v1.4 changelog row is cleaner than a silent frontmatter fix
under v1.3, that is also acceptable — see F-LP36-LOW-001 which requires a v1.4 bump anyway,
so the combined fix at v1.4 is the natural path.

**Routing:** product-owner (BC-frontmatter ownership). Dispatch in fix-burst-34.

---

### F-LP36-LOW-001 — BC-2.17.007:138 VP-PLUGIN-007 description semantically mis-anchors gate-rationale to AC-7 instead of canonical AC-5

| Field | Value |
|-------|-------|
| **Finding ID** | F-LP36-LOW-001 |
| **Severity** | LOW |
| **Confidence** | MEDIUM-HIGH |
| **Location** | `.factory/specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md:138` |
| **Policy** | Codification #11 (lexical-vs-semantic); POL-4 (semantic_anchoring_integrity) |

**Evidence:**

BC-2.17.007 line 138 (VP-PLUGIN-007 row in the verification-properties table) reads:

```
| VP-PLUGIN-007 | After PREREQ-D lands, every loaded .prx plugin in PluginRuntime registry
carries an explicit allowed_urls: Vec<String> field — manifest omission is a hard load
rejection (E-PLUGIN-013) per AC-7 default-deny | ...
```

The gate-rationale at the end of the description anchors to "AC-7 default-deny." However:

- **AC-5** is BC-2.17.007's canonical story anchor for manifest-gate enforcement (per BC
  §Story Anchor at line 157: "AC-5 anchors to this BC"). AC-5 is where the manifest schema
  validation occurs — the `allowed_urls` field presence check, the manifest omission rejection
  (E-PLUGIN-013), and the hard load rejection are ALL at AC-5.
- **AC-7** is the downstream `host_http_request` allowlist enforcement, anchored to
  BC-2.17.002. AC-7 is a CONSUMER of the `allowed_urls` field established at AC-5. Its
  default-deny behavior applies at HTTP request time, not at manifest load time.

The fix-burst-33 rewrite of line 138 correctly updated the Option-semantics framing ("per
AC-7 default-deny" replaced "allowed_urls = None") — this was the F-LP35-MED-001 fix.
However, the replacement text retained "per AC-7 default-deny" as the gate-rationale anchor,
which is semantically incorrect: the manifest load rejection (what VP-PLUGIN-007 verifies)
is a manifest-gate event at AC-5, not an HTTP-request event at AC-7.

**Why it matters:**

A verifier reading VP-PLUGIN-007's gate-rationale as "per AC-7 default-deny" would expect to
find the enforcement in the AC-7 `host_http_request` path. The actual enforcement is in AC-5
at manifest load time. The mislabeling creates a discoverability gap: if VP-PLUGIN-007 fails
in testing, the developer looks at AC-7 (HTTP allowlist) rather than AC-5 (manifest gate).

**Proposed fix (Path A — product-owner):**

Rewrite the VP-PLUGIN-007 description at line 138 to use the canonical AC-5 anchor with
AC-7 as the downstream consumer:

```
per AC-5 manifest gate; default-deny consumer is AC-7
```

This preserves the AC-7 reference (legitimate: AC-7 IS the default-deny consumer of the
Vec<String> field established at AC-5) while correctly labeling the gate-rationale anchor.

**Combined fix-burst-34 scope:**

Because F-LP36-MED-001 (frontmatter staleness) and F-LP36-LOW-001 (line 138 semantic anchor)
both require edits to BC-2.17.007, and F-LP36-MED-001 naturally pairs with a v1.3→v1.4 bump,
fix-burst-34 should dispatch product-owner for:
1. Frontmatter: `timestamp: 2026-05-14T00:00:00Z` + `modified: 2026-05-14`
2. Line 138: "per AC-7 default-deny" → "per AC-5 manifest gate; default-deny consumer is AC-7"
3. Version: v1.3→v1.4 (version bump for the line 138 semantic fix; frontmatter coherence
   correction does not strictly require a bump but the line 138 edit does)
4. §Changelog: add v1.4 row documenting the two fixes

**Routing:** product-owner (BC semantic content ownership). Dispatch in fix-burst-34.

---

## Observations

### OBS-LP36-001 — [process-gap] frontmatter `modified:` + `timestamp:` sibling-sweep on BC version bump (2nd recurrence)

| Field | Value |
|-------|-------|
| **Finding ID** | OBS-LP36-001 |
| **Severity** | OBS [process-gap] |
| **Pattern** | BC version-bump fix-burst forgets to update frontmatter `modified:` + `timestamp:` fields |
| **Recurrence** | 2nd instance in this cascade |

**Instances:**

1. **fix-burst-7-stage-1A** (D-479 lifecycle_status sweep) — frontmatter `lifecycle_status`
   fields not updated on 6 BCs during the initial sweep.
2. **fix-burst-33** (D-535 BC-2.17.007 v1.2→v1.3) — frontmatter `modified:` and `timestamp:`
   fields not updated after v1.2→v1.3 version bump.

**Proposed codification (POL-23 extension):**

Extend POL-23 to enumerate frontmatter `modified:` and `timestamp:` as required sibling-sweep
targets on every BC version bump. Current POL-23 text addresses body-content sibling sweep
discipline; the frontmatter axis is a gap. Codification candidate #24.

**Routing:** cycle-close session-reviewer adjudication. Add as codification candidate #24.

---

### OBS-LP36-002 — BC-INDEX prose vs frontmatter count drift (DEFERRED — system-level)

| Field | Value |
|-------|-------|
| **Finding ID** | OBS-LP36-002 |
| **Severity** | OBS [system-level; deferred] |
| **Confidence** | HIGH |
| **Location** | `.factory/specs/behavioral-contracts/BC-INDEX.md` frontmatter line 4 vs lines 17/19-20 prose |

**Evidence:**

BC-INDEX.md frontmatter line 4 reads: `total_contracts: 236, active_contracts: 229, draft_contracts: 6, deprecated_contracts: 3`

BC-INDEX.md prose at lines 17 and 19-20 reads: `235 contracts total (227 active + 6 draft + 2 deprecated)`

Three independent count claims disagree:
- Frontmatter: 236 total, 229 active, 6 draft, 3 deprecated (229 + 6 + 3 = 238 ≠ 236 also)
- Prose line 17: 235 total
- Prose lines 19-20: 227 active + 6 draft + 2 deprecated = 235 total

This is a pre-existing multi-field count drift. It was not introduced by this cascade or
by fix-burst-33. A proper fix requires a workspace-wide BC enumeration to determine the
authoritative count, which is a system-level architect task.

**Routing:** phase-5 deferred-findings (architect adjudication, system-level workspace
enumeration required). Appended to deferred-findings-phase-5.md this burst (8th deferred
finding).

---

## Convergence Position

Pass-36 brings the finding count to 2 (from 5). Trajectory pass-25..36:
`4→1→4→5→1→1→3→4→5→5→5→2`

The DROP from 5 to 2 is the first genuine decrease in 3 passes. This is a favorable
convergence signal. Fix-burst-34 targets a single-file single-agent scope (BC-2.17.007
v1.3→v1.4 via product-owner only). If fix-burst-34 closes both F-LP36-MED-001 and
F-LP36-LOW-001 cleanly, pass-37 should achieve CLEAN (0/3 → 1/3 streak advance).

The 2 in-scope findings (MED + LOW) are both in BC-2.17.007 and require product-owner
dispatch. The 2 OBS are both non-blocking: OBS-LP36-001 is codification candidate routing
(no fix-burst action needed); OBS-LP36-002 is deferred phase-5 (no fix-burst action needed).

**Fix-burst-34 dispatch template (product-owner ONLY — single-agent, single-file):**

1. BC-2.17.007 line 7: `timestamp: 2026-05-13T00:00:00Z` → `timestamp: 2026-05-14T00:00:00Z`
2. BC-2.17.007 line 14: `modified: 2026-05-13` → `modified: 2026-05-14`
3. BC-2.17.007 line 138: `per AC-7 default-deny` → `per AC-5 manifest gate; default-deny consumer is AC-7`
4. BC-2.17.007 version: v1.3→v1.4; add §Changelog row documenting the two fixes

**State-manager (same burst per TD-VSDD-053):**
- BC-INDEX v4.74→v4.75 (minor bump for BC-2.17.007 v1.4)
- STATE.md + SESSION-HANDOFF.md D-537 closure

**Minimum pass-37 target:** 0 findings (CLEAN; streak 0/3 → 1/3).
