---
document_type: demo-scope
level: ops
producer: state-manager
version: "1.0"
timestamp: 2026-06-13T18:00:00Z
project: prism
---

# DEMO SCOPE — Multi-Client SOC-Analyst Live Demo (Authoritative)

> **THIS IS THE SINGLE SOURCE OF TRUTH** for "everything we are including in the demo."
> Referenced by SESSION-HANDOFF.md §ACTIVE OBJECTIVE and `.factory/objectives/multi-client-soc-demo-tasks.md`.
> A zero-context restart MUST read this file to understand what the demo includes, what is already built, and what the honest gaps are.

---

## Status Legend

- **MERGED** — code on `develop`; spec/contracts active
- **IN CONVERGENCE** — code complete locally; PR-LEVEL adversarial cascade in progress; not yet merged
- **SCOPED-NOT-BUILT** — designed and planned; no code yet

---

## The Frame

Prism runs as a **per-analyst MCP server** inside Claude Code. The analyst works multiple client orgs, queries their security sensors in PrismQL; prism fans out to sensor APIs (DTU clones standing in for real vendors for the demo), normalizes to OCSF, returns a unified result.

**READ-ONLY.** Write-back / actions (TDE — Threat Detection Engineer workflow) is DEFERRED. Reason: requires the absent `prism-operations` crate + wiring the dead write path (E-SENSOR-070 / TODO W3-FIX-S307-001).

---

## MERGED — Built and Merged on `develop`

### 6-Sensor DTU Fleet (behavioral clones of real vendor APIs)

Four **operational sensors**:
- **CrowdStrike** — devices, hosts, alerts, detections
- **Cyberint** — alerts
- **Claroty** — devices, audit_log
- **Armis** — devices/hosts, alerts/detections

Two **enrichment sources**:
- **ThreatIntel** — IOC lookup (threat intelligence)
- **NVD** — CVE lookup (vulnerability database)

These are behavioral HTTP clones of real vendor APIs. The demo queries them instead of live vendor endpoints.

### Per-Client Distinct Seeded Data (Story A / BC-2.06.018)

Merged as **PR #181** (`develop@c287b00d`).

Each client org's sensors return **different** devices/alerts. Org isolation holds: `devices(OrgA) ∩ devices(OrgB) = ∅`. This is what makes it a believable **multi-client** demo — not just routing, but genuinely different data per client.

BC-2.06.018 v1.6 is **active** (promoted at merge per POL-14).

### Review-Cycle Hardening

Three fix-PRs that hardened the fleet for demo quality:
- **PR #182** (`develop@939f36ce`) — DTU fleet lane: DTU schema parity, Postcondition-5 propagation, Armis isolation, CrowdStrike parity
- **PR #183** (`develop@f88b10e3`) — query-core lane: E-QUERY taxonomy splits, watchdog grace period, cache/hot-reload fixes
- **PR #184** (`develop@c200d5a2`) — mcp-boot lane: fail-closed write audit, reload_config WriteTool, capability fields, security polish

All 43 CI checks pass on `develop@939f36ce`.

---

## IN CONVERGENCE — T5 (PR #185, S-DEMO-DTU-LIVE-SCENARIO-001-B)

> The **unfolding-attack centerpiece** of the demo. PR #185 OPEN. Story B HEAD: `15bedc12`. PR-LEVEL cascade streak 0/3; pass 25 NEXT.

### Deterministic 5-Stage Attack Progression Over Wall-Clock Time

**Stages:** Baseline → Recon → Lateral Movement → Exfil → Containment

- Same seed + same clock-offset = same timeline (reproducible for demo replay)
- Stage transitions happen at real wall-clock time during the demo — the analyst can query prism WHILE the attack progresses and see the picture grow

BCs: **BC-2.06.019 v1.7** (scenario progression) — draft; promotes to active at merge (POL-14).

### Stage-Gated Visibility (StageMask)

As the attack unfolds, querying sensors **mid-scenario shows a GROWING picture**:
- Stage 0 (Baseline): clean baseline, normal devices/alerts only
- Stage 1 (Recon): compromised device `dev-deadbeef-…` appears in sensor data
- Stage 2 (Lateral Movement): lateral spread devices appear; IOC hashes surface
- Stage 3 (Exfil): IOCs expand to IPs/domains; exfil alerts fire
- Stage 4 (Containment): containment activates; isolation flags set on devices

**Earlier stages legitimately show less** — this is the demo narrative: "watch the investigation expand as the attack progresses."

### Cross-DTU Coherence

The **same compromised device** (`dev-deadbeef-…`) appears consistently across Armis + CrowdStrike + Claroty at the correct stage. Correlating across sensors tells ONE coherent story per client.

The analyst can pivot: "this device in Armis — what does CrowdStrike say about it? what does Claroty say?"

### Enrichment Correlation (D-1117 Enhancement)

Every scenario IOC resolves in **ThreatIntel** as Malicious (threat score ≥ 75). Every scenario CVE resolves in **NVD** at HIGH CVSS 8.1.

Per user direction (D-1117):
- Every synthetic CVE is **collision-safe**: `CVE-9999-{:04}` format — never matches a real advisory
- Cyberint alert CVEs correlate end-to-end: a Cyberint alert's CVE genuinely resolves in NVD via the real `lookup_and_count` pivot (proven by integration test VP-020-K)
- Enrichment data is scenario-seeded per-client — not random fixture noise

BCs: **BC-2.06.020 v1.6** (enrichment correlation) — draft; promotes to active at merge (POL-14).

### Acceptance Criteria and Demo Evidence

19 acceptance criteria, all with recorded VHS demo evidence (19/19 complete).

---

## SCOPED-NOT-BUILT — Honest Gaps

> Critical to not forget. These items are designed and scoped but have NO code yet.

### THE `enrich` QUERY PATH IS NOT WIRED YET (Priority Gap)

ThreatIntel + NVD are seeded with correlated data and the DTU clones **return it**, but the analyst **CANNOT yet pivot** `| enrich nvd(cve_id)` **through prism** in a PrismQL query. The infusion engine (`S-1.14`) is partial-merge / unimplemented.

**What this means in the demo:**
- Enrichment correlation is **REAL at the data layer** (integration tests prove IOCs/CVEs resolve in the clones)
- But the **in-prism analyst pivot** (`| enrich threatintel(ioc_hash)` / `| enrich nvd(cve_id)`) is FUTURE WORK

**The infusion chain (S-DEMO-ENRICHMENT-PIVOT-001/002/003):**

Designed in WO-D1109 at `.factory/specs/architecture/work-orders/WO-D1109-enrichment-pivot.md`. Three DRAFT stories:
- **S-DEMO-ENRICHMENT-PIVOT-001** — infusion engine plugin-bridge (forward-subset of S-1.14-REDO)
- **S-DEMO-ENRICHMENT-PIVOT-002** — `threatintel.infusion.toml` + `nvd.infusion.toml` grounded against DTU clone routes; IOC-stamping so sensor surfaces carry pivotable IOCs/CVEs
- **S-DEMO-ENRICHMENT-PIVOT-003** — canonical `enrich` pivot queries + demo integration

This is **THE FLAGSHIP `enrich` FEATURE**. User chose the DESIGN-FAITHFUL infusion path (not a sensor-pragmatic shortcut). Slots AFTER the capability-discovery block, BEFORE T11 launcher.

### T6 — Multi-Tenant DTU Overlay

**Story:** `S-DEMO-MULTI-TENANT-DTU-001` (BC-2.06.017 draft; story status: ready v1.2)

Per-instance multi-address binding for fuller multi-tenant testing. Independent of Story A/B; deliver after T5 convergence + merge.

### Capability-Discovery Block (D-1107)

Four stories opted in (D-1107 2026-06-12):
- **S-5.02** — Tool routing / errors / client scoping (depends_on S-5.01)
- **S-5.03** — Resources and prompts (hard dep of S-5.04; depends_on S-5.02)
- **S-5.04** — Sensor health subsystem (depends_on S-5.03 + S-DEMO-001)
- **S-3.13** — Dynamic table availability (parallel after PO authors dedicated BCs; depends_on S-3.02 + S-1.12)

All four require `dclaude:remove-uncertainty` before TDD delivery (D-1110 standing rule).

### T11 — Launcher Consolidation

**Story:** `S-DEMO-LAUNCHER-CONSOLIDATION-001` (draft stub; depends_on S-DEMO-003 SATISFIED)

Pending human launcher-lifecycle decision. Story-writer materialization needed.

### T13 — Capstone

**Story:** Multi-client SOC-analyst narrative story (not yet named or authored). Owner: product-owner + story-writer. After data layer + tooling exist.

---

## What the Demo CAN SHOW Today (Post-T5-Merge)

An analyst in Claude Code, querying **multiple client orgs** through prism, watching an attack **unfold in real time** across CrowdStrike / Armis / Claroty / Cyberint — devices appearing, IOCs surfacing, containment triggering — with **cross-sensor correlation** telling one coherent story per client.

The enrichment pivot (alert → IOC/CVE → ThreatIntel/NVD answer) is demonstrable at the DTU level and becomes a TRUE in-prism analyst pivot once the PIVOT-001/002/003 infusion chain lands.

---

## Build Sequence (Where Each Piece Sits)

```
T1–T4 done
  → T4-A MERGED (PR #181) — per-client distinct seeded data
  → T5 IN CONVERGENCE (PR #185) — unfolding-attack scenario (5 stages, cross-DTU coherence, enrichment correlation at data layer)
  → T6 multi-tenant DTU overlay (S-DEMO-MULTI-TENANT-DTU-001)
  → T8 architect + PO reconcile (S-DEMO-004 formal story file)
  → capability-discovery block: S-5.02 → S-5.03 → S-5.04 (+ S-3.13 parallel)
  → S-DEMO-ENRICHMENT-PIVOT-001 → PIVOT-002 → PIVOT-003 (infusion `enrich` pivot)
  → T11 launcher consolidation (S-DEMO-LAUNCHER-CONSOLIDATION-001)
  → T13 capstone (multi-client SOC-analyst narrative story)
  → T14 demo recording
```

---

## Cross-References

| Document | Role |
|----------|------|
| `.factory/objectives/multi-client-soc-demo-tasks.md` | Granular task tracker (task status, next action, story sequence, CURRENT POINTER) |
| `.factory/SESSION-HANDOFF.md §ACTIVE OBJECTIVE` | Resume-oriented narrative + build sequence mirror |
| `.factory/STATE.md` | Live pipeline state (current phase, active decision rows) |
| `.factory/stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md` | Story B spec (Story B v2.16, BCs 019+020) |
| `.factory/specs/architecture/work-orders/WO-D1109-enrichment-pivot.md` | Enrichment pivot design (PIVOT-001/002/003 architecture) |
| `.factory/specs/behavioral-contracts/BC-2.06.019.md` | Scenario progression contract (v1.7 draft) |
| `.factory/specs/behavioral-contracts/BC-2.06.020.md` | Enrichment correlation contract (v1.6 draft) |
