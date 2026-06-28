---
document_type: working-note
status: working
do_not_execute: true
iterate_later: true
gated_on: "§5.1 brief-reframe (positioning fold deferred)"
produced_by: architect
timestamp: "2026-06-28"
provenance: >
  out-of-band side-analysis working note; NOT yet folded into the positioning candidates;
  touches-no-live-artifacts.
sources_read:
  - specs/matured-vision-day2-requirements.md §17.6, §17.7, §17.9, §17.12, §17.13, §17.14
  - specs/day2-design-decisions/ADR-PROP-active-query-devices.md (C14 D-C14-1/2, OQ-C14-*)
  - specs/day2-design-decisions/ADR-PROP-positioning-problem-framed.md (§8 P4/P5, Pillar B)
  - specs/day2-design-decisions/positioning-executive-narrative.md (Theme B)
  - research/detection-reshape-protocol-dissectors-2026-06-26.md (R5, R6, R9)
  - research/federated-ingestion-collector-connectors-2026-06-26.md
---

# Working Note — Prism as a Passive OT Sensor (PCAP Capture + Native Dissector)

> **NOT a ratified decision. NOT a positioning change. This note records a finding,
> a correction, and the open threads to iterate on later. Do not fold into live specs
> until §5.1 brief-reframe sign-off and explicit human direction.**

---

## §1 — The Finding: Passive OT Sensor Capability Is Decided, Not Roadmap-Vague

Two capabilities that together constitute a passive OT sensor are DECIDED in the
matured-vision capture and are not gated on any open question:

**§17.6 DECIDED 2026-06-26 (human) — Full-packet PCAP retrieval is in day-2 scope.**
The mechanism is an Arkime/Moloch-model disk-bounded rolling packet buffer at the deepest
edge node: `libpcap` / `AF_PACKET` capture, rotating on disk-fill or time boundary,
indexed by session ID. The queryable surface is flow/session metadata (Zeek conn.log /
Suricata EVE flow / Arkime SPI) normalized to OCSF Network Activity (class_uid 4001).
The S2 console gains a "download PCAP" action on flow result rows. Epic: E-COLLECTOR-PCAP-001.
(Source: §17.6, matured-vision-day2-requirements.md.)

**§17.12 DECIDED 2026-06-26 (human) — Prism embeds a NATIVE Spicy-style declarative
parser-generator engine ("the keystone").**
Prism does not depend on running external Zeek or Suricata as a process. Protocol grammars
(Spicy-style `unit`/field/hook definitions plus an interface-definition binding that
compiles to a parser) are first-class prism artifacts. The decision is on the model, not
the embed-vs-reimplement choice: either way the dissection engine is prism-owned and
prism-native with no external process dependency. Declarative grammars produce
bounds-checked parsers (memory-safe, fuzzable). A dissector-backed packet sensor becomes
"just another collector" (`FROM cache.<collector>`); new protocols or OT dialects are new
declarative grammar plugins with no core change.

The dissector emits three things per session: (a) OCSF Network Activity (class 4001,
L3/4 envelope); (b) native schema-on-read OT/protocol semantics (Modbus function codes,
DNP3 object-groups/points, S7 block/variable access, GOOSE dataset-refs/state-numbers,
IEC-104 ASDU types — queried as prism native tables per §13.6); (c) the Community ID
session key linking normalized metadata to pinned raw packets.

Epics: E-DISSECTOR-NATIVE-001 (embedded runtime + core ICSNPP grammars + Community ID +
§17 stage-3 integration), E-DISSECTOR-OT-001 (OT grammar gaps: EtherNet/IP, OPC-UA,
BACnet, MQTT, IEC-61850 MMS/SV; per-site grammar plugin lifecycle; safety/passivity
enforcement). (Source: §17.12, matured-vision-day2-requirements.md.)

**§17.13 DECIDED 2026-06-26 (human) — OT/ICS is the flagship native-schema-on-read case.**
OCSF has no first-class ICS/OT event classes as of 2026 (open proposal: ocsf/ocsf-schema
issue #1515). OT protocol semantics must live in native structured tables queried
schema-on-read. The dissector runs on the OT-layer Satellite (§3.2) under STRICT
PASSIVITY: TAP preferred over SPAN (SPAN can drop time-critical L2 frames such as GOOSE/SV/
PROFINET RT under congestion); placement per Purdue layer / IEC 62443 zones-and-conduits;
L0–L1 process-bus protocols need a sensor on that segment with L2 access; parsing must be
lightweight and deterministic; the dissector must never inject onto an OT segment.

**Domain framing:** The incumbent passive OT-monitoring vendors — Claroty, Nozomi, Dragos —
are fundamentally passive traffic capture (TAP/SPAN) plus deep OT-protocol dissection.
That is precisely the mechanism captured here. This is established, validated technology.
The research confirms it (R6 in `research/detection-reshape-protocol-dissectors-2026-06-26.md`:
"OT safety is non-negotiable and read-only … the OT ecosystem has converged on passive
analysis as the primary telemetry source").

---

## §2 — The Two-Path OT-Standalone Model and the Correction

The OT "stands on its own / customer can forego existing OT-platform investment" story
has TWO distinct paths. An earlier positioning framing conflated them.

**PASSIVE PATH (PCAP capture + Spicy dissector, §17.6/§17.12): DECIDED; SAFE by design.**
This is the TAP/SPAN-at-the-wire model: listening, not touching devices. Strict passivity
is a hard invariant in §17.13 ("prism never injects onto an OT segment under any
configuration"). No device is contacted; no active-polling safety gate applies. This path
is the established mode of operation for every incumbent passive OT sensor — Prism
implementing it makes Prism capable of the same standalone OT-monitoring role
without relying on Claroty/Nozomi/Dragos being present in the customer environment.

**ACTIVE PATH (C14 Reading B — direct OT-protocol device polling): GATED.**
This path speaks OT protocols (Modbus/OPC-UA/DNP3/SNMP) directly to field devices from the
Edge Satellite. It is gated on three open questions:
- OQ-C14-SAFETY-LIABILITY: legal/insurance — if a Prism polling query contributes to a
  controller fault, who owns the risk? Not an engineering-resolvable question before ship.
- OQ-C14-CADENCE-NUMBERS: safe poll-cadence defaults; no published standards numbers exist
  (IEC 62443, NIST SP 800-82, CISA all define principles but no specific cadence values);
  requires non-production OT environment validation.
- OQ-C14-PACKAGING: WASM-compilability of Rust OT-protocol crates at morph.

**THE CORRECTION recorded here:** The OT-standalone claim in positioning is STRONGER than
"gated." It conflates the two paths. The correct statement is:

> "Prism CAN BE a passive OT sensor right now (decided, safe, incumbent-equivalent
> mechanism — TAP/SPAN + deep dissection). The gate governs only the ACTIVE-POLLING
> frontier (direct field-device polling, Reading B), which is a separate capability
> with a different risk profile."

The positioning documents that currently imply OT-standalone is only the carefully-gated
direct-polling path need a targeted correction when the positioning fold happens. This
correction must NOT be made to the live specs until §5.1 sign-off — but it must be
recorded now so it is not lost.

Specifically, the affected passages are:

1. `ADR-PROP-positioning-problem-framed.md` §8 P5 ("Reading B (gated)") and Pillar B
   §3 ("The honest boundary: Native discovery (Reading B) is gated — do NOT headline
   'Prism discovers your OT'"). These are accurate about the active path but are silent
   about the passive path as an independent, safe, decided capability. The Pillar B honest
   boundary needs a second sentence: "Prism can serve as a passive OT sensor (TAP/SPAN +
   native dissector, §17.12/§17.13, DECIDED); that path is safe and has no gate."

2. `positioning-executive-narrative.md` Theme B — "Know your environment." The current
   copy says: "If you do not yet have an OT discovery platform, Prism can directly check
   field devices itself as a fallback option (note: direct device polling is a capability
   we are approaching carefully…)." This is accurate about Reading B but omits the passive
   path entirely. The narrative should acknowledge that Prism can also BE the passive OT
   traffic monitor — the same TAP/SPAN-listen-and-dissect mechanism that Claroty, Nozomi,
   and Dragos use as their core collection mechanism.

---

## §3 — Honest Strains and Caveats (Do Not Minimize, Per §17.9)

These are carried verbatim from §17.9 and §17.12/§17.14 for completeness. They must
accompany any positioning lift.

| Strain | Description |
|--------|-------------|
| Build-stage | All of §17.12/§17.13 is day-2 scope, not shipping product. "Prism plans to be a passive OT sensor" is the honest present tense, not "Prism is." |
| Heaviest dissector build | §17.12 flags the native Spicy-style dissector as "the heaviest dissector build of any approach." This is the tradeoff for owning the engine with no external process dependency. |
| Two heavy native engines | Prism now owns TWO heavy native engines: the Spicy-style declarative dissector AND the windowed continuous operator (§17.7 Phase 2). The resource envelope (edge compute, edge DevOps, development capacity) compounds. |
| OT protocol coverage breadth | Building Modbus/DNP3/S7comm/GOOSE/PROFINET from ICSNPP is the starting set. The GAP protocols (EtherNet/IP, OPC-UA, BACnet, MQTT, IEC-61850 MMS/SV) require prism-authored Spicy grammars — this is the real coverage effort and is likely a phased roadmap. |
| Encrypted OT | OPC-UA (TCP 4840/443) and MQTT-TLS are often encrypted. Passive capture of encrypted OT traffic yields only L3/4 metadata — no OT-protocol semantics. DECIDED 2026-06-26: encrypted-OT = metadata-only by default; bounded-decrypt opt-in at gateway chokepoints is a LATER capability (explicitly tensioned with strict passivity, default-OFF, requires explicit per-site authorization). |
| Full-take PCAP volume | Second storage regime, NOT RocksDB/Iceberg. TB/day at 10 Gbps. Requires its own sizing, retention policy, and residency governance (§17.6). |
| Continuous-operator PHASED | §17.7 Phase 2 (native continuous windowed operator) is "the single most expensive item" — ordered later as a whole feature. Phase 1 NRT-over-cache reuses §14 detection-as-query. |
| Deployment dependency | The passive sensor requires TAP/SPAN access to the relevant OT network segments. Deploying at L0–L1 (GOOSE/SV/PROFINET) requires a sensor physically on that segment with L2 access. This is an infrastructure and site-access constraint, not just a software question. |
| Un-pressure-tested | The passive-OT-sensor capability (§17.12/§17.13) was decided on 2026-06-26 and was NOT covered by the 2026-06-28 mandated-7 adversarial pressure-test. It is decided-but-un-graded. Confidence in the claim should be qualified accordingly until a targeted adversarial pass is run. |

---

## §4 — Positioning Implication (For Later Fold; NOT Applied Here)

The "reuse-or-replace" framing for OT splits cleanly along the two paths:

> "Federate the OT sensors you already have — OR let Prism BE your OT sensor, passively,
> the same way the established tools work — with active device polling as the carefully-
> gated frontier."

This framing:

- Strengthens Pillar B of `ADR-PROP-positioning-problem-framed.md` (problem #4 IT watches
  OT + problem #5 what devices exist) by adding a credible standalone path that does not
  require owning a Claroty/Nozomi/Dragos license.
- Corrects the executive narrative's Theme B, which currently implies that the only
  standalone path is the carefully-gated active-polling fallback.
- Does not weaken the "federate existing platforms" claim — Reading A (federate northbound
  REST APIs from existing OT platforms) remains the zero-safety-risk, immediately decided
  path. The passive-sensor path is additive, not a replacement for the federation story.
- Should be framed with the build-stage honest caveat: day-2 planned capability, not
  shipping product.

The positioning fold is BLOCKED on §5.1 brief-reframe sign-off. Do not apply until then.

---

## §5 — Open Threads to Iterate On Later

- [ ] **OT protocol-coverage priority and phasing.** Which protocols ship first (likely
  Modbus/DNP3 from ICSNPP as the safest adopt/extend path)? What is the phasing roadmap
  for the GAP protocols (EtherNet/IP, OPC-UA, BACnet, MQTT, IEC-61850 MMS/SV)? And what
  is the embed-vs-reimplement-Spicy-runtime decision called out in §17.12 as deferred to
  morph?

- [ ] **Reconcile §17 (passive PCAP+dissector ingestion) with C14 (active-query devices)
  into ONE coherent OT-ingestion narrative.** Currently two separate captures in the
  matured-vision document — they sit in different sections (§17 collector class vs §16.x
  C14 decisions). They should be unified: passive as default / always-safe; active as
  gated / opt-in frontier. A single OT-ingestion ADR at morph time should draw this line
  explicitly.

- [ ] **Adversarial pressure-test of "Prism can be a passive OT sensor."** The claim is
  decided in §17 but was not in scope for the 2026-06-28 mandated-7 adversary pass. Before
  this capability anchors external positioning, run a dedicated adversarial pass against:
  (a) build feasibility of the native Spicy-style engine; (b) OT-protocol coverage maturity
  vs incumbent breadth; (c) TAP/SPAN deployment practicality and site-access constraints;
  (d) encrypted-OT visibility limits; (e) edge-compute footprint vs OT passivity ceiling.

- [ ] **Fold the correction into live positioning artifacts** (only after §5.1 sign-off):
  `ADR-PROP-positioning-problem-framed.md` §8 P5 + Pillar B honest boundary; and
  `positioning-executive-narrative.md` Theme B "What Prism does" paragraph. The fold
  should add the passive-sensor path without removing the federation-first framing.

- [ ] **Relationship to C20 NERC CIP.** Passive monitoring (TAP/SPAN, no device interaction,
  no control-path) is the CIP-friendly OT posture. §17.13 strict passivity + SF-3 (lighter
  classification by default, passive-read-only = no control path → avoids full EACMS/BCS
  CIP weight) are directly synergistic. A passive-OT-sensor deployment that never contacts
  a device is the lowest-classification, ESP-friendly posture. This synergy should be
  noted explicitly in the C20 narrative when the fold happens (CIP-005 ESP + passive/
  one-way/data-diode edge mode, per §3.2 and C20 SF-3 in matured-vision-day2-requirements.md
  line 3769–3776).

- [ ] **Competitive angle.** What is the correct competitive framing for Prism-as-passive-
  OT-sensor vs Claroty/Nozomi/Dragos? These vendors have years of grammar coverage,
  production-hardened parsers, and purpose-built OT-protocol stacks. Honest parity claim
  requires scoping the comparison: "passive monitor for the protocols we dissect natively"
  vs "all OT-protocol coverage a purpose-built sensor provides." The maturity gap is real
  and must be acknowledged — this capability is an architectural foundation, not a
  day-one parity claim. The competitive angle should wait until the protocol-coverage
  phasing decision (thread 1 above) is settled.

---

## Footer

Working note. `do_not_execute: true`. NOT yet folded into any positioning candidate,
live ADR, behavioral contract, or story. Iteration substrate for the passive-OT-sensor
capability thread. Brief-reframe §5.1 is still pending; no positioning artifact may
be modified until that sign-off. State-manager may commit this file as a standalone
artifact burst.

*Sources: matured-vision-day2-requirements.md §17.6/§17.7/§17.9/§17.12/§17.13/§17.14;
ADR-PROP-active-query-devices.md (C14); ADR-PROP-positioning-problem-framed.md (§8 Pillar B,
P4/P5); positioning-executive-narrative.md (Theme B); research/detection-reshape-protocol-dissectors-2026-06-26.md
(R5, R6, R9/synthesis).*
