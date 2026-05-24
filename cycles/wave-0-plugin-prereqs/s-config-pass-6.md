---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-24T00:00:00Z
cycle: "wave-0-plugin-prereqs"
story: "S-CONFIG-MULTI-TENANT-OVERRIDE-001"
pass: 6
traces_to: convergence-trajectory.md
---

# S-CONFIG-MULTI-TENANT-OVERRIDE-001 — LOCAL Adversary Pass-6

**Pass Date:** 2026-05-24
**Feature HEAD:** `3416eea6` (fix-burst-6 implementer; OBS-LP5-001 corrective incomplete — state-manager D-814 burst)
**Streak Before Pass:** 0/3
**Result:** 4 findings (2 MED + 1 LOW + 1 LOW) — meta-recurrence of OBS-LP5-001 narrative drift inside the very burst that codified lesson 42

---

## Part A — Fix-Burst-6 Closure Verification

All 3 pass-5 findings (F-LP5-MED-001 + F-LP5-LOW-001 + F-LP5-LOW-002) verified CLOSED:

- **F-LP5-MED-001 DURABLE:** BC-2.06.016 line 108 E-SPEC-020 `expected_instance_id` message template — `{expected}` placeholder confirmed; `{sensor_id}@{org_slug}` paraphrase removed (PO 513ee6b8).
- **F-LP5-LOW-001 DURABLE (partially):** overlay.rs doc-comment forward-pointer corrections confirmed at 3 of 5 sites (`make_e_spec_019_unknown_extends`, `make_e_spec_020_instance_id_mismatch`, `make_e_spec_021_tables_in_overlay`). NOTE: 2 sibling sites (`e_spec_022_unknown_org_slug`, `make_e_spec_023_unrecognized_field`) retain paraphrased templates — F-LP6-LOW-001 below captures this sibling-sweep gap.
- **F-LP5-LOW-002 DURABLE:** S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001 v0.1→v0.2 AC-006 Suggestion field source-of-truth adjudication confirmed (architect 4ef6c650 Option B).

### SAP-1 Probe (tracing emission catalog)

Grepped `event_type =` across crates/ — no new emission sites in this story's branch. No SAP-1 findings.

### SAP-2 Probe (DTU↔TOML schema parity)

Not applicable — this pass does not touch `.prism/specs/sensors/*.toml` files.

---

## Part B — New Findings

### Finding Summary

| ID | Severity | Category | Description |
|----|----------|----------|-------------|
| F-LP6-MED-001 | MEDIUM | [process-gap] narrative-drift recurrence | s-config-fix-burst-6.md F-LP5-LOW-001 closure section cites non-existent function names: `make_e_spec_019_instance_id_mismatch` (actual: `make_e_spec_019_unknown_extends`) and `make_e_spec_022_unknown_org_slug` as a fixed site (actual: site NOT in fix-burst-6; fix-burst-6 fixed `make_e_spec_020_instance_id_mismatch`). Same OBS-LP5-001 class as found in pass-5 narrative, now in the closure record itself. |
| F-LP6-MED-002 | MEDIUM | [process-gap] narrative-drift recurrence | lessons.md entry 41 bullets (1) and (2) still contain paraphrase drift despite OBS-LP5-001 meta-correction note in header. Bullet (1): "BC-2.06.013 §Postconditions used colon where canonical template uses em-dash (F-LP4-MED-001)" — WRONG: BC-2.06.013 v1.1 changelog states "E-SPEC-021 message at line 73 — replaced paraphrase (semicolon-separated...) with canonical (period-separated...)". Bullet (2): "BC-2.06.013 §Error Cases used `{overlay_path}` vs canonical `{file}` (F-LP4-MED-002)" — WRONG: BC-2.06.013 v1.1 changelog states "E-SPEC-023 message at line 82 — replaced paraphrase (`{field}` placeholder) with canonical (`{field_name}` placeholder)". |
| F-LP6-LOW-001 | LOW | sibling-sweep gap | overlay.rs fix-burst-6 applied forward-pointer doc-comment style to `make_e_spec_019_unknown_extends` + `make_e_spec_020_instance_id_mismatch` + `make_e_spec_021_tables_in_overlay` but did NOT apply to the 2 remaining canonical-error-template builders: `e_spec_022_unknown_org_slug` (impl block method, overlay.rs) and `make_e_spec_023_unrecognized_field` (free function, overlay.rs). Both still carry paraphrased templates with per-fix drift potential. Standing Rule 3 §1b sibling-sweep completion gap. |
| F-LP6-LOW-002 | LOW | [spec-gap] | BC-2.06.016 EC-016-003 ("all five error codes fire in the same boot") is ambiguous: silent on whether codes could originate from the same file or require different files. Code analysis of `validate_overlay_toml` structural-check early-return in `prism-spec-engine/src/overlay.rs` confirms: structural errors (E-SPEC-021/E-SPEC-023) cause early-return before deserialization, making E-SPEC-019/E-SPEC-020 unreachable for the same file. EC-016-003 should specify "each from a DIFFERENT overlay file or directory". Also: no EC documents the within-file suppression boundary explicitly. |

---

## Part C — Standing Probe Results

### OBS-LP6-001 — Meta-Recurrence of OBS-LP5-001

**Observation:** Pass-6 adversary confirmed that the D-814 state-manager burst — the same burst that codified lesson 42 to prevent narrative drift — produced F-LP6-MED-001 (fix-burst-6.md wrong function names) and F-LP6-MED-002 (lessons.md entry 41 body still wrong). Both are instances of the SAME OBS-LP5-001 class: narrative authored from memory/summary rather than reading source.

This is strong empirical evidence that codification-only mitigation is insufficient. The pattern recurred immediately inside its own codification burst, suggesting the authoring step requires mechanical enforcement (lint hook or mandatory grep gate) rather than verbal-only codification.

**Routing:** state-manager corrective in fix-burst-7 (narrative scope, state-manager domain). F-LP6-LOW-001 routing: implementer. F-LP6-LOW-002 routing: product-owner (BC amendment). Lessons entry 43 to capture meta-recurrence [process-gap] [codified].

---

## Verdict

**CLEAN (strict):** no
**CLEAN (PR-merge):** no
**Streak:** 0/3 → 0/3 (BLOCKED)
**Findings:** 2 MED + 2 LOW = 4 total
**Root cause (F-LP6-MED-001+MED-002):** D-814 state-manager burst authored fix-burst-6 closure record and lessons.md entry 41 from memory/summary rather than reading overlay.rs source and BC-2.06.013 v1.1 changelog — the same failure mode that lesson 42 was authored to prevent. Meta-recurrence inside codification burst.

## Fix-burst Routing

Fix-burst 7 dispatched:
- **PO** (455f9fbb): BC-2.06.016 v1.2→v1.3 — EC-016-003 cross-file aggregation scope clarification + EC-016-005 within-file structural-suppresses-semantic boundary (F-LP6-LOW-002 closed).
- **Implementer** (d600f7f4): overlay.rs `e_spec_022_unknown_org_slug` + `make_e_spec_023_unrecognized_field` forward-pointer doc-comment style applied (F-LP6-LOW-001 closed).
- **State-manager (this burst, D-815):** F-LP6-MED-001 corrective (s-config-fix-burst-6.md rewritten with byte-quoted function names from overlay.rs); F-LP6-MED-002 corrective (lessons.md entry 41 bullets rewritten with byte-quoted text from BC-2.06.013 v1.1 changelog); BC-INDEX v5.50→v5.51; lessons entry 43 [process-gap] [codified].
