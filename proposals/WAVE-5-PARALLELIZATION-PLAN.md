---
document_type: parallelization-plan
title: "Wave 5 Parallelization Plan — wave-5-e-demo-fidelity"
producer: state-manager
version: "1.0"
timestamp: "2026-05-31T00:00:00Z"
project: prism
status: CURRENT
anchor: D-910
---

# Wave 5 Parallelization Plan

**Produced:** 2026-05-31 (D-910 state-manager durability burst)
**Wave:** wave-5-e-demo-fidelity
**Baseline:** PLUGIN-MIGRATION-001-A/E merged ✅; S-DTU-CYBERINT-AUTH-FIDELITY-001 merged PR #164 develop@e798e67c ✅
**Immediate blocker:** S-CONFIG-MULTI-TENANT-OVERRIDE-001 is the SOLE remaining hard gate to S-DEMO-001 keystone.

---

## 1. Dependency Tiers

### Tier 0 — Dispatchable now

| Story | Points | Priority | Notes |
|-------|--------|----------|-------|
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 | 8 | P0 | **Sole hard gate to S-DEMO-001 keystone. Dispatch first.** |
| S-DEMO-ARMIS-AQL-001 | 5 | P1 | Solo Armis crate; no file conflicts with other Tier 0 work. PO flags SUFFICIENT (see §3). |
| S-DEMO-CROWDSTRIKE-MULTIREGION-001 | 2 | P2 | Solo CrowdStrike crate. **Gated on ${env.VAR} prereq (see §4 CRITICAL).** |
| S-DEMO-CLAROTY-AUDIT-DTU-001 | 5 | P1 | Claroty lane story 1 of 3 (serialize within lane). |
| S-DEMO-CLAROTY-TRAILING-SLASH-001 | 3 | P1 | Claroty lane story 2 of 3; soft-dep on AUDIT-DTU-001. |
| S-DEMO-CLAROTY-PAGINATION-001 | 5 | P1 | Claroty lane story 3 of 3. |
| S-5.04-FIX-001 | 1 | P2 | Factory-only; no crate changes. Dispatch anytime. |

### Tier 1 — After S-CONFIG merges

| Story | Points | Priority | Notes |
|-------|--------|----------|-------|
| S-DEMO-001 (KEYSTONE) | 11 | P0-KEYSTONE | Requires S-CONFIG + S-DTU-CYBERINT-AUTH-FIDELITY-001 (both merged). |
| S-DEMO-MULTI-TENANT-DTU-001 | TBD | P2 | Stub; needs story-writer materialization. Non-blocking for single-tenant demo. |

### Tier 2 — After S-DEMO-001 merges

| Story | Points | Priority |
|-------|--------|----------|
| S-DEMO-002 | 11 | P0 |

### Tier 3 — After S-DEMO-002 merges

| Story | Points | Priority |
|-------|--------|----------|
| S-DEMO-003 | 5 | P1 |

**Key:** Cyberint merged ⇒ S-CONFIG-MULTI-TENANT-OVERRIDE-001 is the SOLE remaining hard gate to the S-DEMO-001 keystone.

---

## 2. File-Overlap Conflict Map

The real parallelism limiter is file-overlap, not logical deps.

| Files | Contended By | Risk Level | Resolution |
|-------|-------------|-----------|------------|
| `claroty.sensor.toml` | S-DEMO-CLAROTY-AUDIT-DTU-001 + S-DEMO-CLAROTY-TRAILING-SLASH-001 + S-DEMO-CLAROTY-PAGINATION-001 | HIGH | Serialize the 3 Claroty stories within the lane |
| `crates/prism-dtu-claroty/src/routes/` | S-DEMO-CLAROTY-AUDIT-DTU-001 (new audit_log route) + S-DEMO-CLAROTY-TRAILING-SLASH-001 (normalize_path) | HIGH | Serialize: AUDIT first, TRAILING-SLASH second |
| `crates/prism-spec-engine/src/` (env/config) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 + ${env.VAR} prereq work | WATCH | Coordinate if env-var prereq folded into S-CONFIG |
| `crates/prism-dtu-armis/` | S-DEMO-ARMIS-AQL-001 only | LOW | Distinct crate — parallel safe |
| `crates/prism-dtu-crowdstrike/` | S-DEMO-CROWDSTRIKE-MULTIREGION-001 only | LOW | Distinct crate — parallel safe |
| `crates/prism-spec-engine/` (config overlay) | S-CONFIG-MULTI-TENANT-OVERRIDE-001 only | LOW | Distinct from Armis/CrowdStrike crates |

---

## 3. Recommended Conflict-Aware Parallel Tracks (Tier 0)

### Track A — P0: S-CONFIG-MULTI-TENANT-OVERRIDE-001
**Run first — unblocks keystone.** Per-org overlay loading per ADR-029. Solo spec-engine crate. No file conflicts with other tracks.

### Track B — P1: S-DEMO-ARMIS-AQL-001
Solo `prism-dtu-armis` crate. AQL endpoint fidelity. Can run in parallel with Track A.
**PO flags disposition:** Both flags SUFFICIENT (see §3 — PO Flag Dispositions). Clear to dispatch (modulo ${env.VAR} prereq for parity tests, but non-blocking for story start).

### Track C — P2: S-DEMO-CROWDSTRIKE-MULTIREGION-001
Solo `prism-dtu-crowdstrike` crate. Multi-region base_url fidelity.
**BLOCKED** by ${env.VAR} prereq — parity tests cannot pass until env-var interpolation is implemented. Dispatch only after ${env.VAR} prereq resolved.

### Track D — Claroty Lane SERIALIZED: AUDIT → TRAILING-SLASH → PAGINATION
Shared `claroty.sensor.toml` + `crates/prism-dtu-claroty/` require serialization. Three stories in sequence:
1. S-DEMO-CLAROTY-AUDIT-DTU-001 (new route first)
2. S-DEMO-CLAROTY-TRAILING-SLASH-001 (normalize_path depends on route existing)
3. S-DEMO-CLAROTY-PAGINATION-001 (pagination offset/limit)

### Track E — P2: S-5.04-FIX-001
Factory-only (no crate changes). Dispatch anytime; zero conflict with any other track.

**Ceiling: ~4–5 concurrent stories max** (Claroty collapsed to one serial lane; CrowdStrike gated on env-var prereq).

---

## 4. Critical Path (Spine) vs Parallel Fidelity

```
[CRITICAL PATH — SPINE]
S-CONFIG → S-DEMO-001 (keystone) → S-DEMO-002 → S-DEMO-003

[PARALLEL — fidelity stories]
Track B (Armis-AQL)  ─┐
Track D (Claroty ×3) ─┤─ all parallel to spine, NOT depends_on for S-DEMO-001
Track C (CrowdStrike) ─┘  RECOMMEND: add as soft-deps of S-DEMO-002 for faithful demo
Track E (S-5.04)      ─ anytime
```

The 3 fidelity stories are parallel to the spine — they do NOT block S-DEMO-001 dispatch. However, for a fully faithful multi-sensor demo (S-DEMO-002 is the 3-org multi-sensor E2E smoke), recommend adding them as soft-deps of S-DEMO-002 so the demo exercises real fidelity behaviors.

---

## 5. CRITICAL SHARED PREREQUISITE — ${env.VAR} Interpolation (HUMAN DECISION NEEDED)

**Severity:** BLOCKS all fidelity story parity tests.

### Problem

`${env.VAR}` interpolation in sensor TOML fields (e.g., `base_url = "${env.ARMIS_INSTANCE_URL}"`) is currently a **DEAD LETTER** — `prism-spec-engine` does NOT resolve it. The value stays as the literal string `"${env.ARMIS_INSTANCE_URL}"`. This affects:

| Sensor | Field | Current state |
|--------|-------|--------------|
| Armis | `base_url` | `${env.ARMIS_INSTANCE_URL}` → literal string (not resolved) |
| Claroty | `base_url` | `${env.CLAROTY_INSTANCE_URL}` → literal string |
| CrowdStrike | `base_url` | `${env.CROWDSTRIKE_BASE_URL}` → literal string |

Blocks parity tests for ALL 3 fidelity stories (base_url won't resolve, DTU requests hit the literal string as a URL).

### Required implementation

1. **Implement `${env.VAR}` resolution in `spec_parser`** — post-parse pass over string fields; sibling-sweep all sensors per TD-VSDD-060.
2. **Register `E-SPEC-024` in error-taxonomy.md** — missing/empty env var at load → structured error (no panic, no value leaked to logs).
3. **Add AC to BC-2.16.009** (or new BC) — covers the env-var resolution and error path.

### Human decision needed

**Where does this prereq land?**

| Option | Tradeoff |
|--------|----------|
| A — Standalone prereq story (new `S-SPEC-ENV-VAR-001` or similar) | Cleanest; explicit gate; adds story to Wave 5 backlog |
| B — Fold into S-CONFIG-MULTI-TENANT-OVERRIDE-001 | S-CONFIG already touches spec-engine config; may fit; broadens S-CONFIG scope |
| C — Implement in first fidelity story (Armis-AQL or Claroty-AUDIT) | Minimal scope expansion; but couples fidelity story to env-var plumbing |

**CRITICAL PATH IMPACT:** This gate MUST close before any fidelity story's parity tests pass. CrowdStrike-multiregion story is BLOCKED until resolved. Armis-AQL and Claroty-AUDIT stories can START (non-parity work) but parity AC verification blocks.

---

## 6. PO New-BC Flag Dispositions

Per orchestrator evaluation 2026-05-31 (D-911).

| Story | Flag | Disposition | Action at Dispatch |
|-------|------|-------------|-------------------|
| S-DEMO-ARMIS-AQL-001 | Flag 1 (AQL validation BC) | **SUFFICIENT** — AQL opaque per ADR-031 §D8-a/R-DTU-002; no new BC required | story-writer adds BC-2.16.013 to `behavioral_contracts` frontmatter |
| S-DEMO-ARMIS-AQL-001 | Flag 2 (AQL push-down parity EC) | **SUFFICIENT** — BC-2.16.013 covers (closes DTU-EXT-003/004); existing ACs are sufficient | story-writer adds BC-2.16.013 to `behavioral_contracts` frontmatter |
| S-DEMO-CLAROTY-TRAILING-SLASH-001 | Flag 3 (trailing-slash parity coverage) | **SUFFICIENT** — BC-2.16.013 covers; no new AC required for POST-path trailing-slash normalization | story-writer adds BC-2.16.013 to `behavioral_contracts` frontmatter |
| S-DEMO-CROWDSTRIKE-MULTIREGION-001 | Flag 4 (env-var-resolution BC coverage) | **NEW-AC-AT-DISPATCH** — ${env.VAR} prereq; needs E-SPEC-024 + BC-2.16.009 new AC + impl | GATES story: resolve ${env.VAR} prereq first |
| S-DEMO-CROWDSTRIKE-MULTIREGION-001 | Flag 5 (missing-env E-SPEC-error BC) | **NEW-AC-AT-DISPATCH** — E-SPEC-024 (next free code; E-SPEC-023 is current max, 015/016 retired) + BC-2.16.009 AC + impl | GATES story: resolve ${env.VAR} prereq first |

**Readiness summary:**

| Story | Ready to dispatch? |
|-------|-------------------|
| S-DEMO-ARMIS-AQL-001 | YES — flags SUFFICIENT; add BC-2.16.013 frontmatter at dispatch; parity tests gated on ${env.VAR} prereq |
| S-DEMO-CLAROTY-TRAILING-SLASH-001 | YES — flag SUFFICIENT; add BC-2.16.013 frontmatter at dispatch; soft-dep on AUDIT-DTU-001 merge |
| S-DEMO-CROWDSTRIKE-MULTIREGION-001 | NOT CLEAR — gated on ${env.VAR} prereq + new E-SPEC-024 + BC-2.16.009 AC |

---

## 7. Practical Dispatch Ceiling

Each story runs full TDD → LOCAL 3-CLEAN → PR → PR-LEVEL 3-CLEAN → merge (heavy pipeline). Realistic concurrency: 2–3 stories demanding real orchestrator attention simultaneously. Trivial ones (S-5.04, CrowdStrike once unblocked) are cheap.

**Merge serialization:** PRs merge one-at-a-time. Low-overlap tracks (Armis, CrowdStrike, S-CONFIG) rebase clean. Claroty lane serializes within lane.

---

*Anti-volatile-pin (TD-VSDD-091): all citations use story-ID/BC-ID/finding-ID/function-name anchors. No file:line-number citations.*
