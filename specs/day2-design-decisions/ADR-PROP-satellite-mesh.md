---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C2-1: Transport (gRPC bidi streaming vs NATS leaf-node — prototype bake-off gated)"
  - "ADR-PROP-C2-2: Relay trust role (mTLS terminator/re-originator only)"
  - "ADR-PROP-C2-3: Role nouns (Coordinator / Relay Satellite / Edge Satellite)"
  - "ADR-PROP-C2-4: Diode / true one-way OT mode (DEFERRED explicit open question)"
  - "ADR-PROP-C2-5: Identity model (SPIFFE-model native Rust, no SPIRE runtime)"
  - "ADR-PROP-C2-6: Trust model (per-hop mTLS only, no transitive trust)"
  - "ADR-PROP-C2-7: Bootstrap (join-token + optional TPM attestation)"
  - "ADR-PROP-C2-8: Loop prevention (request-ID set + hop-count TTL + optional path-vector)"
  - "ADR-PROP-C2-9: Per-hop deadline decrement (gRPC model verbatim)"
  - "ADR-PROP-C2-10: Store-and-forward (RocksDB durable queue, drop-oldest-loud)"
  - "ADR-PROP-C2-11: Partial-failure relay (extend BC-2.01.010 + CCS skip_unavailable lineage)"
  - "ADR-PROP-C2-12: Residency enforcement (structural, edge-normalization, IEC-62443 mapping)"
  - "ADR-PROP-C2-13: Max chain depth (hop-TTL ceiling with rationale)"
produced_by: architect
timestamp: "2026-06-27"
provenance: "side-analysis C2 capture; human-confirmed decisions 2026-06-27 session. Research basis: research/satellite-mesh-2026-06-26.md (5 perplexity_research sonar-deep-research calls at reasoning_effort=high + 1 perplexity_ask). Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact."
traces_to:
  - matured-vision-day2-requirements.md §3.2 (Satellite mesh; D-1330 name confirmation)
  - matured-vision-day2-requirements.md §16.4 (C2 decisions log entry)
  - matured-vision-day2-requirements.md §17.4 (ingestion locus; collection-capable Satellite)
  - matured-vision-day2-requirements.md §17.5 (push-lands-locally, pull-retrieves-on-query)
  - matured-vision-day2-requirements.md §17.8 (chain-aware cache / Q3 deadline model)
  - day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (RocksDB at Satellite; SQLite local control-plane)
  - domain-spec/invariants.md (per-hop mutual auth non-negotiable; residency-by-construction)
  - research/satellite-mesh-2026-06-26.md (primary research basis — all six topics)
  - CLAUDE.md (AD-017 AI-opaque credentials; Standing Rule 3 §2 no-silent-Vec::new(); newtype + redacted-Debug)
---

# ADR-PROP — Prism Satellite Mesh: C2 Control+Communication Layer

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C2
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/satellite-mesh-2026-06-26.md` — five `perplexity_research`
> (sonar-deep-research, `reasoning_effort=high`) calls covering enrollment/identity, outbound
> transport, chaining/tree topology, partial-failure/store-and-forward, and residency/OT/Purdue,
> plus one `perplexity_ask` for role-noun terminology. All load-bearing claims are source-grounded
> in that research document. Claims corroborated from model knowledge are explicitly flagged
> `[model-knowledge]` in the research doc.

> **Scope reminder:** "Prism Satellite" noun confirmed by human as D-1330. This ADR-PROP covers
> the C2 control+communication layer design only — not the §3.4 role-noun finalization (BA + PO
> scope), not story decomposition, not live BCs.

---

## Context

A Prism Satellite is a remote query executor deployed at a client site, plant, or network enclave.
The central Prism service acts as coordinator/planner; satellites act as remote executors.
Communication is outbound-only from Satellite to central (dial-home), compatible with strict
firewall policies that permit only upstream connections. Satellites can chain in a tree:

```
Coordinator (central Prism)
  └── Relay Satellite (e.g., enterprise DMZ)
        └── Relay Satellite (e.g., OT L3/SCADA)
              └── Edge Satellite (e.g., OT L2/field devices)
```

The five driving use cases (from matured-vision §3.2) are:
1. OT/ICS Purdue-model layered segmentation — chain traverses Purdue layers.
2. Air-gapped enclaves — bastion Satellite bridges the gap.
3. MSSP nested topology — spoke → regional-hub → central.
4. Remote/intermittent/low-bandwidth edges — store-and-forward + reconnect.
5. Fan-in and data-residency hops — only normalized results transit upward.

The decisions below cover all thirteen C2 sub-decisions confirmed on 2026-06-27. Each decision
is numbered D-C2-N, cross-references the research, and documents the reasoning.

---

## Decision Ledger

### D-C2-1 — Transport: gRPC Bidi Streaming PRIMARY; NATS Leaf-Node STRONG ALTERNATIVE

**DECIDED 2026-06-27 (human).**

**PRIMARY / default:** gRPC bidirectional streaming over HTTP/2:443 via `tonic`.

**Mechanism (reverse-RPC inversion):** The Satellite is the network *client* — it dials out to the
coordinator. The coordinator is the logical *driver* — it pushes execution requests DOWN the
satellite-initiated bidi stream and reads results back UP. gRPC bidi streams permit either side to
send first; this inversion is the key trick. [research/satellite-mesh-2026-06-26.md §Topic 2]

**Rationale for PRIMARY:**
- TLS:443 is the most firewall-friendly outbound posture (§3.2 #5 "permits only upstream").
- `tonic` is the mature Rust/Tokio gRPC stack; Prism is already a tonic/hyper-adjacent Tokio
  codebase. No net-new transport framework.
- Protobuf result envelopes align with Prism's existing OCSF+protobuf normalization boundary.
- HTTP/2 PING keepalive (~30–60s idle) keeps connections alive through middleboxes.
- Exponential backoff reconnect with jitter handles transient link interruption.

**STRONG ALTERNATIVE — NATS leaf-node hierarchy:**
NATS gives hub-spoke+tree topology, JetStream store-and-forward, subject routing, and automatic
reconnect "for free" at the cost of an embedded NATS broker dependency. The NATS leaf topology is
the §3.2 architecture natively, with no bespoke store-and-forward code.
[research/satellite-mesh-2026-06-26.md §Topic 2 lean]

**DEFERRED to prototype bake-off:** the final either/or between gRPC-native (more control, fewer
deps, build store-and-forward yourself) and NATS-leaf (topology + S&F + reconnect for free, one
more embedded process) is gated on a prototype. This is a genuine architecture fork; a prototype
is the right adjudication mechanism. Real ADR to be written post-prototype.

**Explicit known cost — TCP Head-Of-Line Blocking (TCP-HOLB):**
A single HTTP/2 connection multiplexes all streams over one TCP connection. HTTP-layer HOLB is
solved; TCP-layer HOLB is NOT — a single lost packet stalls all streams on the connection. This
is a real latency risk when a relay carries mixed control (latency-sensitive) and bulk result
traffic on the same connection.

**TCP-HOLB mitigations (in order of preference):**
1. Separate control vs. bulk-result connections per hop.
2. HTTP/3 / QUIC via `quinn` (Prism already pins `quinn-proto` per recent commit) — independent
   per-stream flow control eliminates TCP-HOLB entirely. This is the eventual upgrade path.
   Noted; NOT adopted now; `quinn` Rust ecosystem is operationally younger than `tonic`.

---

### D-C2-2 — Relay Trust Role: mTLS Terminator / Re-Originator ONLY

**DECIDED 2026-06-27 (human).**

A Relay Satellite terminates the mTLS connection from its children and opens a fresh mTLS
connection to its parent. It does NOT act as a sub-CA and it vends NO cross-hop credential.

This means a Relay Satellite:
- Holds its own X.509-SVID (issued by the coordinator's CA or its immediate upstream's CA).
- Authenticates its children by validating THEIR SVIDs against the trust bundle for the
  relevant trust domain.
- Does NOT issue SVIDs to its children — that issuance path remains with the coordinator.
- Does NOT relay the child's identity credential upstream — it presents its OWN identity
  to its parent.

**Why not sub-CA:** Allowing a Relay to issue SVIDs to its subtree creates transitive trust —
a root credential could reach into a lower Purdue zone without re-authentication at the relay
boundary. This is exactly the Teleport root-CA-reaches-leaf foot-gun (documented in the
research at §Topic 1). Prism's tree crosses residency and zone boundaries (OT L2 ↔ L3 ↔
enterprise) where transitive trust directly violates IEC-62443 zone separation (D-C2-6 below).
[research/satellite-mesh-2026-06-26.md §Topic 1, transitive vs per-hop trust section]

---

### D-C2-3 — Role Nouns: Coordinator / Relay Satellite / Edge Satellite

**DECIDED 2026-06-27 (human) — ARCHITECTURAL LEAN.**

The three-noun role set, qualifying the confirmed "Prism Satellite" base noun (D-1330):

| Role | Definition |
|------|-----------|
| **Coordinator** | Central root — the central Prism service in its topology-driving role. Not a Satellite; it is the tree root. |
| **Relay Satellite** | Interior executor + aggregator. Executes queries against its own local sources AND aggregates results from its subtree. Dials the coordinator or a parent Relay Satellite. |
| **Edge Satellite** | Leaf executor. Executes queries against its own local sources. Has no children. Dials its parent Relay Satellite or the coordinator directly. |

**Rationale:** "Relay Satellite" and "Edge Satellite" are role qualifiers on the confirmed
"Prism Satellite" noun. They read naturally in context, avoid NATS-vs-graph-theory clash on
"leaf", and avoid the pure-repeater connotation of "hub". [research/satellite-mesh-2026-06-26.md
§Topic 6]

**Scope boundary:** §3.4 of matured-vision owns the final role-noun finalization. Business
Analyst + PO adjudicate. This capture records the architectural lean; the real decision lives
in §3.4.

Runner-up: "Concentrator Satellite" — strongest many-to-one aggregation connotation, telecom
heritage. Retained as alternative if §3.4 discussion prefers to foreground the aggregation role.

---

### D-C2-4 — Diode / True One-Way OT Mode: DEFERRED — Explicit Open Question

**DECIDED 2026-06-27 (human): Day-2 = bidirectional mTLS mesh only.**

True one-way (data-diode) OT mode is DEFERRED. It is recorded as an explicit open design
question / future epic, gated on a prototype.

**Why DEFERRED:** Mutual-auth mTLS is inherently bidirectional — the TLS handshake requires a
round-trip to authenticate both parties. A true unidirectional link (hardware data diode,
Waterfall/Owl style) likely precludes mTLS without a separate transport variant: for example,
a store-and-forward relay that accepts OCSF results via a one-way channel and buffers them
for the upstream to collect. This is a second transport path for the highest-assurance OT
deployments, not a configuration flag on the existing mTLS path.

**Status:** `[INCONCLUSIVE]` per research — prototyping required. The design fork is real.

**Future epic placeholder:** E-SATELLITE-DIODE-001 (one-way OT diode mode — post-day-2).

---

### D-C2-5 — Identity: SPIFFE-Model, Native Rust, SPIRE NOT a Runtime Dependency

**DECIDED 2026-06-27 (human).**

**Identity model:** SPIFFE-style.

- Satellite identity = a URI `prism-sat://<trust-domain>/<sat-id>` bound to a **short-lived
  X.509 cert** (SVID) chaining to a per-trust-domain CA held at the coordinator.
- Cert lifetime is short (hours, configurable); automatic rotation before expiry.
- The coordinator (or the upstream Relay Satellite in its role as mTLS terminator — but NOT
  as sub-CA, per D-C2-2) holds and operates the CA for its immediate downstream trust domain.

**Bootstrap secret + private key credential handling:**
Reuse Prism's existing newtype + redacted-Debug credential discipline (CLAUDE.md §Conventions)
for the bootstrap secret and the agent private key. These are sensitive types; they MUST use
newtypes with redacted `Debug` impls and MUST NOT transit AI context (AD-017 /
`project_ai_opaque_credentials.md`).

**SPIRE is NOT a runtime dependency:**
SPIFFE is the *identity model*. SPIRE is one *implementation* of that model. Prism implements
the SVID-issuance + node-attestation + short-lived-cert-rotation natively in Rust.

**Rationale for native Rust:**
- Air-gap / edge deployment constraint (§3.2 #2): a Satellite must be self-contained, no
  external control-plane sidecar process.
- Ephemeral ethos: the Prism binary carries everything it needs.
- Operational simplicity at OT edge nodes: zero external process dependencies.

[research/satellite-mesh-2026-06-26.md §Topic 1 lean]

---

### D-C2-6 — Trust Model: Per-Hop Mutual mTLS ONLY; No Transitive Trust

**DECIDED 2026-06-27 (human).**

Every inter-Satellite hop and every Satellite-to-coordinator hop uses **mutual mTLS**. There is
no transitive trust across hops.

**What this means:**
- A node at hop N can only authenticate its immediate neighbors (hop N-1 and hop N+1).
- A node at hop N cannot assert identity claims on behalf of a node at hop N+2.
- A compromised or misconfigured node at hop N cannot impersonate any node it is not directly
  connected to.

**Why no transitive trust:**
Transitive trust (shared root CA that can authorize any node in the tree) is explicitly rejected
because:
1. **IEC-62443 zone separation requirement:** Prism's tree crosses Purdue zones
   (enterprise ↔ DMZ ↔ OT L3 ↔ L2 ↔ L1). Transitive trust would allow a higher-zone
   credential to reach into a lower OT zone, violating the zone separation invariant that
   IEC-62443 mandates.
2. **Teleport root-CA foot-gun:** Teleport's own documentation warns that root-CA trust
   silently bypasses leaf-cluster labels — a node with a root-issued cert can reach the leaf
   directly, bypassing all label-based policy gates. This is the exact attack surface Prism
   must NOT replicate.
3. **D-C2-2 consistency:** per-hop-only trust is the direct corollary of "relay = mTLS
   terminator, not sub-CA."

[research/satellite-mesh-2026-06-26.md §Topic 1, transitive vs per-hop trust; §Topic 5 lean]

---

### D-C2-7 — Bootstrap: SPIRE-Style Join Token + Optional TPM Attestation

**DECIDED 2026-06-27 (human).**

**Bootstrap mechanism:**
1. Operator generates a one-time / TTL-bounded **join token** for the new Satellite.
2. Token is distributed **out-of-band at deploy time** (e.g., during provisioning, via
   secure operator workflow — not via network auto-discovery).
3. On first boot, the Satellite dials its configured upstream (coordinator or parent Relay)
   and presents the join token over TLS.
4. Upstream verifies the token (TTL check + one-time consumption), issues the Satellite's
   first X.509-SVID, and delivers the trust anchor (CA bundle for the Satellite's trust domain).
5. Token is burned (single-use). Subsequent reconnections use the SVID, not the token.

This matches the consensus pattern across SPIRE join-token, k8s bootstrap token, and Teleport
ephemeral TTL token. [research/satellite-mesh-2026-06-26.md §Topic 1, lean]

**Optional hardening — TPM attestation:**
For high-assurance OT deployments, the bootstrap can be upgraded to TPM attestation: the
Satellite uses a TPM key to sign the bootstrap request, cryptographically binding the
identity to the hardware device (Teleport `tpm` method precedent). This is a hardening
upgrade; it is NOT required for the base satellite mesh.

**Open question:** distribution of join tokens to truly air-gapped enclaves with no out-of-band
network path (sneakernet, printed QR / ID). The answer is operational procedure, not a
transport protocol change. Flagged for operational runbook at morph.

---

### D-C2-8 — Loop Prevention: Belt-and-Suspenders (Request-ID + Hop-TTL + Optional Path-Vector)

**DECIDED 2026-06-27 (human).**

Three independent, composable mechanisms — belt-and-suspenders. The existing §3.2 specification
mandates seen-request-ID sets; this decision adds the hop-count TTL as a hard ceiling and
records path-vector as an optional diagnostic enhancement.

| Mechanism | Description | Binding |
|-----------|-------------|---------|
| **Seen-request-ID set** | Each hop rejects a request whose ID it has already seen (ring buffer or TTL-expiring set). Rejects loops where the same original request ID circulates. | Existing §3.2 spec — retained. |
| **Hop-count TTL** | An integer decremented at each hop; a node receiving TTL ≤ 0 rejects the request without forwarding. IP-TTL analog. [model-knowledge: RFC 791] | NEW — added by this decision. |
| **Path-vector (OPTIONAL)** | The request carries the list of Satellite IDs traversed. A node rejects the request if its own ID appears in the path. BGP AS-path analog. [model-knowledge: BGP loop detection] | OPTIONAL — recommended as a diagnostic enhancement. Yields a free topology trace for heartbeat/health. |

**Why both request-ID set AND hop-TTL:**
A request-ID set alone fails if IDs are regenerated mid-path (bug, restart, or deliberate
mutation). A hop-TTL is a cheap hard ceiling that bounds any cycle, regardless of how IDs
behave. Defense-in-depth from IP/BGP/RPF prior art.
[research/satellite-mesh-2026-06-26.md §Topic 3 lean]

**Production default hop-TTL ceiling:** see D-C2-13 for the concrete value.

---

### D-C2-9 — Per-Hop Deadline Decrement: Adopt gRPC Semantics Verbatim

**DECIDED 2026-06-27 (human).**

**Mechanism:** Per-hop deadline decrement using gRPC deadline semantics (absolute deadline,
not per-hop timeout). [model-knowledge: gRPC deadline propagation; research/satellite-mesh-2026-06-26.md §Topic 3]

At each hop:
1. Receive the request with an absolute deadline timestamp.
2. Compute residual: `residual = deadline − now − hop_budget_reservation`.
3. If `residual ≤ 0`: fail fast with `DEADLINE_EXCEEDED` — do NOT dial downstream.
4. Forward the request with the residual as the new absolute deadline.
5. On deadline expiry during subtree execution: surface results received so far plus a
   partial/coverage signal for the timed-out subtree (via the partial-failure path, D-C2-11).

**Per-hop budget reservation:**
Each relay reserves a hop budget (configurable, e.g., 50–200ms) before forwarding. This
prevents a relay from forwarding a request it cannot possibly receive a response for in time.

**Ties to §17.8 Q3:** This is the v1 deadline model described in matured-vision §17.8.3.
The "full budget-aware planner" (query-plan-aware resource allocation) is ordered later;
gRPC + partial+coverage is the production-grade v1.

---

### D-C2-10 — Store-and-Forward: RocksDB-Backed Durable Queue; Drop-Oldest-Loud

**DECIDED 2026-06-27 (human).**

**Implementation:** A new RocksDB column family at collection-capable Satellites (Edge and
Relay Satellites operating in collection/locus mode per §17.4). RocksDB is already present at
every Satellite (ADR-PROP-storage-engine-taxonomy.md: RocksDB everywhere). Adding a new CF
is the lowest-new-dependency path.

**Behavior:**
- Push data (inbound sensor events / collector receipts) lands locally in the durable queue CF.
- Data buffers in the queue during connectivity gaps to the upstream.
- On reconnect, the queue drains inward (pulled or pushed, per transport model).
- The queue is bounded. When it fills:
  - **Drop-oldest** policy — preserve the most recent data.
  - **Loud coverage signal** — the dropped range MUST surface as a coverage gap in the
    partial-failure / coverage banner (D-C2-11). NEVER silent loss.
  - `event_time` TTL from §3.3 applies — data exceeding its TTL is evicted regardless of
    queue pressure.

**Two failure classes (must be distinguished in implementation):**

| Class | Description | Response |
|-------|-------------|---------|
| **Transient** | Connectivity gap; upstream temporarily unreachable. Buffer absorbs the gap; drain on reconnect. | Store-and-forward applies. |
| **Hard** | Subtree genuinely unreachable past deadline (hop TTL exhausted, upstream permanently unavailable). | Surface as skipped in coverage banner (D-C2-11); do NOT wait indefinitely. |

**Buffer placement and residency:**
The durable queue MUST reside at the in-region Edge Satellite (or the innermost Relay in the
zone). A Relay at a higher Purdue zone MUST NOT buffer raw inbound data that should be
normalized before it reaches that layer — the residency invariant (D-C2-12) requires
normalization AT the edge, in-zone, before any upward transit.

**Replay semantics:** lean is **at-least-once delivery + idempotent dedup** on drain. This
matches the loop-prevention request-ID set (D-C2-8) — request IDs serve double duty as
dedup keys. Exactly-once delivery costs (JetStream QoS-2 / MQTT QoS-2) are NOT accepted
as a default; QoS-1 + dedup is the production-grade lean.

---

### D-C2-11 — Partial-Failure: Extend BC-2.01.010 + CCS skip_unavailable Lineage; No Hop Swallows

**DECIDED 2026-06-27 (human).**

**Principle:** Prism already owns the right partial-failure primitive — BC-2.01.010 partial-result
+ §3.6 CCS-lineage coverage banner (matured-vision cites Elastic/OpenSearch Cross-Cluster Search
`skip_unavailable` as ADOPT-3, §10.3). The mesh extension does NOT invent a new mechanism; it
EXTENDS the existing one to the multi-hop case.

**Mesh extension rule:**
When a Relay Satellite loses a child (or its subtree is unreachable), it surfaces the child's
entire subtree as a **skipped segment** in the coverage banner, with:
- Reason (transient connectivity loss, deadline exceeded, hard failure, explicit skip).
- Last-seen timestamp for the child.
- Which sensors / sources within the subtree were lost.

This gap is relayed **upward unmodified through every hop**. No intermediate hop may swallow or
aggregate-away a downstream failure signal. This is an extension of:
- Prism Standing Rule 3 §2: no silent `Vec::new()` return where partial-failure data should
  propagate (CLAUDE.md).
- IEC-62443 conduit integrity: a conduit that silently drops failure metadata is not a
  controlled conduit.

**Coverage accumulation:** The coverage banner accumulates per-hop skip metadata so the
Coordinator can report exactly which Purdue layer / region / tenant subtree was unreachable
for a given query.

**Partial results on deadline:** When a subtree times out (D-C2-9 deadline), the relay surfaces
results received so far from that subtree plus a `timed_out: true` flag in the coverage metadata.
This matches the Elastic CCS `_clusters.timed_out` behavior.
[research/satellite-mesh-2026-06-26.md §Topic 4; CCS docs; BC-2.01.010]

---

### D-C2-12 — Residency: Structural Enforcement; IEC-62443 Zone Map; Satellite-Local Credential Resolution

**DECIDED 2026-06-27 (human).**

**Residency is enforced STRUCTURALLY by what crosses the conduit — not by policy configuration.**

The hard invariant: a Satellite normalizes raw sensor data → OCSF / native-schema **at the
edge, in-zone**, and **only the normalized result transits the conduit upward**. Raw data NEVER
crosses a Satellite boundary.

**IMPORTANT — OCSF normalization governs data FORMAT, not PII content (ADS conformance
2026-06-27; P-ADS-03; CONFLICT-7 resolution):** D-C2-12 means raw vendor API responses are
normalized to OCSF before transit — it does NOT mean the conduit traffic is PII-safe.
OCSF-normalized events carry first-class PII in standard fields: hostnames, IP addresses,
user account names, process names, file paths. An implementer who reads "only OCSF results
transit" must NOT conclude the conduit is PII-safe. OQ-DEPLOY-2(a) result-transit residency
governance applies to all conduit traffic including OCSF-normalized results. The in-transit
path is encrypted (per-hop mTLS per D-C2-6), but data jurisdiction / residency-of-results
policy is a separate concern that must be addressed per OQ-DEPLOY-2(a). (ADS conformance 2026-06-27)

This is not a policy flag that can be turned off. It is enforced by:
1. The Satellite code processing raw data before placing anything in the outbound stream.
2. The conduit (inter-Satellite connection) never accepting raw-schema frames from downstream.
3. The policy DSL (§17.8.2) making raw-forward across a residency boundary inexpressible.

**IEC-62443 zones-and-conduits mapping:**

| IEC-62443 concept | Prism mapping |
|-------------------|--------------|
| Security zone | One zone per Satellite (or per Satellite group at the same Purdue level) |
| Conduit | Inter-Satellite hop (one connection, mTLS authenticated) |
| Conduit security level | Per-hop mTLS (D-C2-6) is the conduit's authentication control |
| Zone boundary | Satellite boundary — the normalization point |

This gives Prism a recognized standards anchor (IEC-62443 / NIST SP 800-82) for the OT story,
which matters for the MSSP / 1898 & Co audience.
[research/satellite-mesh-2026-06-26.md §Topic 5 lean]

**NIST SP 800-82 companion:** The IEC-62443 zones-and-conduits model is companion-aligned with
NIST SP 800-82 ICS security guidance. Reference both in the OT positioning.

**Satellite-local credential resolution — HARD INVARIANT:**
Every Satellite resolves its sensor credentials (API keys, bearer tokens, service accounts)
AT the Satellite using its local SecretBackend. Credentials are NEVER sent to the coordinator
or to any upstream hop. The outbound stream carries only OCSF-normalized query results.

This binds directly to:
- AD-017 / `project_ai_opaque_credentials.md` — credentials never transit AI context.
- §11.1 SecretBackend satellite-local posture — the Satellite's SecretBackend resolves
  creds locally, not by calling back to central SS-26.
- The residency invariant — a credential value is as sensitive as raw sensor data.

---

### D-C2-13 — Max Chain Depth: Hop-TTL Ceiling = 8 Hops; Rationale

**DECIDED 2026-06-27 (human): Production default hop-TTL ceiling = 8.**

**Rationale:**

| Reference | Layer count |
|-----------|------------|
| Purdue Reference Model (IEC 62443 / ISA-99) | 5 levels: L0 (field) → L1 → L2 → L3 → L4 (enterprise), plus enterprise cloud/remote = 6 with DMZ |
| MSSP nesting | spoke → regional → national → global coordinator = 4 hops from edge to coordinator |
| Real-world maximum observed | Prism's deepest expected topology: OT L1 → L2 → L3 → enterprise → DMZ → regional MSSP hub → national MSSP → coordinator = 7 hops |
| Safety margin | +1 hop above the expected maximum to accommodate unforeseen nesting |

**Production ceiling: 8 hops.** A request delivered to the coordinator at hop-TTL = 8 started
at the edge. A misconfigured loop will be hard-dropped at hop 8 regardless of request-ID state.

**Configurable:** The ceiling is a configuration parameter (Satellite TOML), defaulting to 8.
Operators with simpler topologies (2–3 hops) SHOULD set a tighter ceiling to minimize the
blast radius of a misconfiguration. Operators with unusually deep nesting (unlikely but possible
in large MSSP hierarchies) may raise it with documented justification.

**Interaction with D-C2-9 (deadline):** hop-TTL and deadline are independent safeguards. A
request can be dropped by the hop-TTL ceiling before the deadline fires (misconfiguration),
or by the deadline before the hop-TTL fires (deep subtrees over slow links). Both must be
surfaced as distinct failure reasons in the coverage banner (D-C2-11).

---

## Design Notes (from research — not new decision forks)

### Incremental / Streaming Result Aggregation at a Relay

Result aggregation at a relay SHOULD be incremental/streaming rather than barrier-wait
(wait-for-all-children-then-merge). Rationale: the per-hop deadline model (D-C2-9) naturally
produces partial results as children complete or time out. Incremental aggregation emits partial
batches as each child returns, with the deadline as the final flush trigger. This minimizes
coordinator wait time and aligns with the coverage-banner model (D-C2-11 timed_out handling).
[research/satellite-mesh-2026-06-26.md §Topic 3 open questions; §17.8.3 lean]

### Push-Down at Each Relay (DataFusion-Federation Pattern)

The residency-friendly execution model is for each Relay Satellite to receive an inward
sub-plan (DataFusion-Federation remote-subplan style) and execute it locally, rather than
blindly relaying the full coordinator plan. Push-down at each layer:
- Reduces data volume crossing each conduit (relay normalizes + aggregates before forwarding).
- Enforces residency-by-construction (raw data never enters the relay's outbound stream).
- Enables per-layer predicate pushdown (e.g., filter to sensors in-zone before fan-out).

Lean: push-down is the preferred execution model. Blind relay (relay forwards the full plan
without local optimization) is acceptable as a v1 simplification, with push-down as the
upgrade path. [research/satellite-mesh-2026-06-26.md §Topic 3 open questions]

### §17.4 Collection Locus Integration

A "collection-capable Satellite" (§17.4 locus modes a/b) is an Edge or Relay Satellite
that additionally hosts a receiver endpoint and the RocksDB durable-queue buffer (D-C2-10).
The reverse-RPC transport (D-C2-1) carries the inward pull of buffered data on query. The
§17.5 "push lands locally, pull retrieves on query" mechanic maps directly to the reverse-RPC
flow: push = sensor writes to the local queue; pull = coordinator sends a WorkItem down the
bidi stream; Satellite drains the queue and sends results UP the stream.
No new transport is needed for the locus capability.

### §17.8 Chain-Aware Cache Integration

The §17.8 chain-aware cache (Q1/Q2/Q3 model) overlays directly on the mesh:
- **Q1 (declarative policy tiering):** the per-collector replication policy controls what may
  transit each conduit and at what tier — the policy runs AT the Satellite, enforced
  before forwarding.
- **Q2 (residency-first per-field policy language):** residency enforcement ordered BEFORE
  destination selection, per-field (§17.8.2 transform-ordering). The policy DSL makes
  raw-forward across a residency boundary inexpressible.
- **Q3 (deadline budget):** the per-hop gRPC deadline decrement (D-C2-9) IS the Q3 v1
  deadline mechanism.
- **Request coalescing at a relay:** a Relay Satellite can coalesce identical inward sub-queries
  from its children (CDN request-collapsing / single-flight pattern). Feeds into the §17.8
  "warm hub" cache at an intermediate relay.

---

## Remaining Open Questions

These are genuine open questions, not deferred decisions. Captured here for the architect's
consumption at morph time.

| # | Question | Domain | Notes |
|---|---------|--------|-------|
| OQ-C2-1 | **Trust-anchor rotation across a deep tree without a flag-day.** SPIFFE rotates trust bundles frequently; how does a CA roll propagate hop-by-hop across N relays without a simultaneous cutover? | Identity / enrollment | `[INCONCLUSIVE]` in research. Multi-hop propagation is under-specified in SPIFFE standard for multi-level chains. |
| OQ-C2-2 | **Transport fork bake-off:** gRPC-native (tonic) vs. NATS-leaf — prototype + ADR. | Transport | D-C2-1 gated. The fork is genuine; pick the better implementation path empirically. |
| OQ-C2-3 | **Join-token distribution to truly air-gapped enclaves** — no out-of-band network path. Sneakernet / QR printed token? | Bootstrap | Operational runbook question more than protocol design. |
| OQ-C2-4 | **Separate control vs. bulk-result connections** to mitigate TCP-HOLB per D-C2-1? Or accept single-connection until HTTP/3 upgrade? | Transport | ADR-PROP-C2-1 real ADR to resolve. |
| OQ-C2-5 | **Corporate TLS-inspection / MITM proxy survival.** Does Prism need client cert pinning, and how does pinning survive an inspecting proxy in OT enterprise networks? | Transport / security | Transport research flags this as an open issue for both gRPC and WebSocket. |
| OQ-C2-6 | **Replay ordering and dedup on drain.** At-least-once is the lean; full dedup requires the loop-prevention request-ID ring buffer to also serve as a dedup filter at the receiver. Interaction with in-flight duplicate suppression? | Store-and-forward | D-C2-10 records the lean; implementation detail to resolve at morph. |
| OQ-C2-7 | **Per-zone normalization-schema enforcement.** How does the coordinator verify that a Relay actually stripped raw and only forwarded normalized results? Attestation of the normalization step? Schema-validated conduit? | Residency / security | Structural residency enforcement (D-C2-12) is the answer in principle; the verification mechanism is open. |
| OQ-C2-8 | **Diode-compatible one-way transport variant** for highest-assurance OT zones (D-C2-4). | OT / security | Full design fork; future epic E-SATELLITE-DIODE-001. |
| OQ-C2-9 | **Path-vector overhead.** The path-vector (D-C2-8 optional) grows with chain depth (up to 8 entries). Encoding in the request metadata — protobuf repeated field? Acceptable overhead for an 8-hop chain? | Protocol | Implementation detail; likely negligible but confirm at ADR authorship. |

---

## Honest Costs

| Cost | Description |
|------|-------------|
| **Build vs. adopt (gRPC-native lean)** | Choosing gRPC-native means Prism builds its own store-and-forward queue management, enrollment protocol, cert-rotation, and reconnect logic. NATS-leaf hands these to the broker. The research estimates this as weeks of engineering. Not a free lunch; surfaced as the explicit bake-off fork. |
| **Per-hop re-authentication** | Per-hop mTLS (D-C2-6) costs more TLS handshakes and more certificate rotation surface than transitive trust. This is the accepted price of residency-by-construction. |
| **RocksDB column family addition per Satellite** | Every collection-capable Satellite gains a store-and-forward durable-queue CF (D-C2-10). RocksDB CF management at every edge node; backpressure-when-full policy must be designed and configured. |
| **Native Rust identity implementation** | Not taking SPIRE as a runtime dependency means implementing SVID-issuance, node-attestation, cert-rotation natively. This is real engineering (days–weeks). The upside is zero external-process dependency at the edge. |
| **Hop-TTL ceiling is a configuration surface** | The ceiling must be documented, tunable, and tested. An 8-hop ceiling is large; a misconfigured topology with TTL=8 can exhibit substantial latency before a loop is detected. Per the rationale above, 8 is justified by the deepest expected MSSP topology; tighter defaults for simpler deployments are recommended. |
| **Diode OT mode absent from day-2** | The highest-assurance OT deployments (hardware data diode, Waterfall/Owl-class) are NOT supported by the bidirectional mTLS mesh. This is a known gap, documented as D-C2-4. |

---

## Alternatives Considered and Rejected

### Alternative A: NATS-leaf as PRIMARY (not STRONG ALTERNATIVE)

NATS leaf-node hierarchy maps directly to the §3.2 hub-spoke+tree topology and provides
store-and-forward, subject routing, and reconnect as built-in capabilities.

**Not chosen as PRIMARY because:** the prototype bake-off has not been run. Both gRPC-native
and NATS-leaf are viable; the ADR-PROP records gRPC as the default lean based on the existing
`tonic` dependency and fewer net-new deps, but explicitly defers the final choice to a
prototype. Choosing NATS as PRIMARY now without a prototype would skip the empirical
adjudication that the genuine fork deserves.

### Alternative B: Teleport or Tailscale for enrollment + transport

Teleport provides reverse tunnels, ephemeral tokens, TPM attestation, and multi-hop topology
out of the box. Tailscale/Headscale provides WireGuard mesh + auth-key bootstrap.

**Rejected because:**
- Teleport's root-CA-reaches-leaf transitive trust (D-C2-6) is incompatible with IEC-62443
  zone separation.
- Both Teleport and Tailscale are external control-plane dependencies, incompatible with
  air-gap / edge / self-contained deployment (D-C2-5 rationale).
- Tailscale's centralized control plane (even Headscale as self-hosted) adds an operational
  dependency that contradicts the Satellite's self-sufficiency requirement.

### Alternative C: Single root CA for the entire tree (transitive trust)

A shared root CA issues certs to all Satellites; any Satellite can authenticate any other
by validating against the root.

**Rejected because:** this is exactly the Teleport foot-gun (D-C2-6). A compromised root CA
— or any node presenting a root-CA-signed cert — could reach into any OT zone in the tree,
directly violating the IEC-62443 zone-separation invariant. Per-hop trust is the mandatory
model for a topology that crosses residency boundaries.

---

## Ripple Effects (must be picked up at morph time)

| Affected area | Ripple |
|---------------|--------|
| **ADR-PROP-storage-engine-taxonomy.md** | RocksDB at every Satellite now gains a store-and-forward durable-queue CF (D-C2-10). The taxonomy table `Engine: RocksDB / Lane: Satellite` row should note the S&F CF. |
| **§3.2 matured-vision — decision block** | The §3.2 section receives a "DECIDED 2026-06-27" decision block (appended, not rewriting existing prose) — see this document's associated matured-vision update. |
| **§3.4 matured-vision** | Role-noun finalization (D-C2-3 lean: Coordinator / Relay Satellite / Edge Satellite) is for BA + PO at morph. |
| **BC families (PO scope at morph)** | New BC families needed: Satellite enrollment; Satellite dial-home reconnect; per-hop deadline propagation; per-hop partial-failure relay; store-and-forward drain on reconnect; Satellite-local credential resolution. |
| **Architecture — new subsystem SS-## Satellite Mesh** | Satellite mesh entity (§3.2 spec: Satellite topology node, trust anchor, endpoint, health state) needs a subsystem entry in ARCH-INDEX.md at morph. |
| **domain-spec/entities.md** | New entity: Satellite (topology node, trust anchor, chain-depth, health state, parent endpoint). |
| **domain-spec/invariants.md** | New invariants: per-hop mTLS non-negotiable; residency-by-construction (raw-never-transits-conduit); satellite-local credential resolution hard invariant; no hop swallows downstream failure. |
| **E-SATELLITE-MESH-001** | The proposed epic (§3.2 spec) now has the C2 decisions as its architecture backing. ADR-PROP-satellite-mesh.md is the input to the epic's ADR slots. |
| **E-SATELLITE-DIODE-001** | New future epic for D-C2-4 one-way OT diode mode. Out of day-2 scope; flagged for post-day-2 roadmap. |
