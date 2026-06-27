---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-26"
provenance: "side-analysis discussion input; does not modify vision/specs."
topic: "Prism Satellite mesh — enrollment, dial-home transport, chaining, per-hop auth, loop prevention, partial-failure relay"
feeds: "matured-vision §3.2 (Satellite mesh) discussion; ties §17.4 ingestion locus + §17.8 chain cache; §3.4 relay/aggregator role-noun (TBD)"
---

# Prism Satellite Mesh — Cited Research (Discussion Input)

> **SCOPE / GUARDRAIL.** This is a SIDE-ANALYSIS discussion input to inform a conversation about the
> matured-vision §3.2 Satellite mesh. It does **not** modify the vision, any spec, STATE.md,
> SESSION-HANDOFF.md, or any prior research. `do_not_execute: true`. Leans are recommendations for
> the discussion, not decisions. The human + architect adjudicate; nothing here is binding.

> **READ-COVERAGE HONESTY.** Five `perplexity_research` (sonar-deep-research, `reasoning_effort=high`)
> calls + one `perplexity_ask` were issued. Each deep-research call returned 82K–99K chars saved to
> tool-result files. I read the **enrollment** doc in full through its conclusion (~62K/82K chars; the
> tail was the SPIFFE/Teleport/K8s comparative conclusion, which I read) and the **transport** doc in
> full through §6 (~61K/93K; the tail covered HTTP/3/WebTransport/QUIC + Rust-ecosystem wrap-up). The
> **chaining**, **partial-failure**, and **residency** docs were confirmed-present and topic-complete
> but could not be paginated cleanly (single-line JSON, char-offset reads capped). For those three I
> rely on (a) the portions surfaced, (b) the cross-doc synthesis already read, and (c) clearly-flagged
> `[model-knowledge]` for the well-trodden mechanics (IP TTL, BGP AS-path, RPF, gRPC deadline
> propagation, Prometheus federation, OTel collector chaining, CDN parent caches, CCS `skip_unavailable`,
> JetStream, MQTT QoS). Where a claim rests on model knowledge it is marked. Citation-density on the
> three partial-read docs is lower; flagged as `[INCONCLUSIVE-DEPTH]` where it matters.

---

## Executive Summary (~12 lines)

1. **Enrollment is a solved problem with a clear winner-pattern, not a winner-product.** Every mature
   system (SPIFFE/SPIRE, Teleport, k8s kubelet, Tailscale/Headscale) uses the same three-stage shape:
   low-assurance **bootstrap credential** → outbound dial → **CSR/attestation** → **short-lived
   verifiable identity** + trust anchor. [enrollment-research; SPIFFE/SPIRE docs; RFC 7030; RFC 8555]
2. **Lean: model Satellite identity on SPIFFE X.509-SVID semantics, issued via a SPIRE-style
   node-attestation + join-token bootstrap, but implement it natively in Rust** (do not adopt SPIRE as
   a runtime dependency — air-gap/edge constraint + ephemeral ethos). Identity = `spiffe://`-style URI
   bound to a short-lived X.509 cert chaining to a per-trust-domain CA. [enrollment-research]
3. **Per-hop mutual auth is point-to-point mTLS at every hop; transitive trust is the OPTION TO AVOID.**
   Teleport's own docs warn that root-CA trust silently bypasses leaf-cluster labels — the exact
   foot-gun Prism must NOT replicate. Lean: **per-hop trust only**, no transitive identity across hops.
   [enrollment-research, Teleport trusted-clusters caveat]
4. **Transport lean: outbound-initiated, long-lived, multiplexed channel — gRPC bidirectional streaming
   over HTTP/2 (tonic) for the control+result plane, with NATS leaf-node hierarchy as the strong
   alternative** if the mesh wants broker-grade store-and-forward + subject routing for free. [transport-research]
5. **The reverse-RPC inversion is the key trick:** the satellite is the network *client* (dials out),
   but the coordinator is the logical *driver* — it pushes execution requests DOWN the satellite-initiated
   stream and reads results back UP. gRPC bidi streams permit either side to send first. [transport-research, gRPC docs]
6. **Chaining = aggregation tree.** Requests fan inward; results + partial-failure metadata aggregate
   outward; each relay is a non-leaf tree node that both executes locally AND aggregates its subtree.
   Closest prior art: **NATS leaf-node hierarchy** (topology) + **gRPC deadline propagation** (per-hop
   decrement) + **Prometheus/OTel collector chaining** (aggregation). [chaining-research, model-knowledge]
7. **Per-hop deadline DECREMENT is gRPC-canonical:** each hop subtracts elapsed + a hop-budget and
   forwards the residual deadline; a hop with a non-positive residual fails fast rather than dialing
   downstream. [model-knowledge: gRPC deadline semantics; chaining-research]
8. **Loop prevention: belt-and-suspenders.** (a) seen-request-ID set per hop (already in §3.2), AND
   (b) a **hop-count TTL** decremented per hop (IP-TTL / BGP-max-AS-path analog) as a hard ceiling that
   bounds a misconfigured cycle even if request-IDs are mutated. [model-knowledge: IP TTL, BGP loop prevention, multicast RPF]
9. **Partial-failure: Prism already owns the right primitive — BC-2.01.010 partial-result + §3.6
   coverage banner.** The mesh extension is "subtree-unreachable, not lost" = exactly Elastic/OpenSearch
   **CCS `skip_unavailable`** `_clusters{successful/skipped/failed}` metadata, relayed per hop. [partial-failure-research, CCS docs]
10. **Store-and-forward: the mesh needs durable buffering at relays for the §17.4 ingestion locus** —
    push data lands at the edge satellite, buffers during gaps, drains on reconnect. Prior art: NATS
    JetStream / MQTT QoS-1 persistent sessions / disk-backed WAL queue. Lean: a RocksDB-backed durable
    queue (Prism already runs RocksDB) with explicit backpressure when the buffer fills. [partial-failure-research]
11. **Residency-by-construction is structural, not policy.** IEC-62443 zones-and-conduits + Purdue
    layering + data-diode/bastion patterns all enforce "raw stays put, only normalized aggregates
    transit." This is the federated-analytics-where-only-aggregates-leave pattern, mapped onto the
    satellite chain — the residency invariant Prism already wants (§17.5). [residency-research, IEC 62443]
12. **Role-noun lean for the relay/aggregator satellite (§3.4 TBD): "Relay Satellite"** as the role
    qualifier (every node is a "Prism Satellite"; a non-leaf one is a "Relay Satellite"). Runner-up:
    **"Concentrator"** (telecom many-to-one heritage, strongest aggregation connotation). Avoid
    overloading "leaf" (NATS clash) and "hub" (pure-repeater connotation). [role-noun-research]

---

## Topic 1 — Enrollment + identity bootstrap

### Prior art

- **SPIFFE/SPIRE** [enrollment-research; spiffe.io docs]. Identity = **SPIFFE ID** URI
  `spiffe://<trust-domain>/<workload>`; materialized as an **SVID** (X.509-SVID cert *or* JWT-SVID),
  signed by an authority in the ID's trust domain. SPIRE Server = trust-domain CA; SPIRE Agent runs per
  node, exposes the Workload API. Two-phase attestation: **node attestation** (pluggable: join-token,
  AWS IID, k8s SA, TPM) then **workload attestation** (selector-matched registration entries). Agent
  bootstrap: needs an initial **trust bundle** (file, `trust_bundle_url` over HTTPS, or unix-socket);
  `insecure_bootstrap` exists but is explicitly MITM-vulnerable and dev-only. **Federation** =
  trust-domain A imports trust-domain B's **bundle** (public keys) so cross-domain SVIDs validate.
  **Delegated Identity API** lets a trusted local delegate obtain SVIDs for other workloads — relevant
  to a relay terminating mTLS for its subtree.
- **EST (RFC 7030)** [RFC 7030; enrollment-research]. Certificate enrollment over a TLS-secured HTTP
  session; client can fetch current CA certs (trust-anchor distribution — but authenticity is
  out-of-band/out-of-scope), then submit a CSR; supports re-enroll/rekey using the current cert. PKI
  topology-agnostic; multi-hop = subordinate CAs + chain validation.
- **ACME (RFC 8555)** [RFC 8555]. Account → order → challenge (domain-control) → issuance → auto-renew.
  Server-cert / DNS-control oriented; flat topology. **ACME Device Attestation** (draft-ietf-acme-
  device-attest) adds hardware-attestation challenge types (TPM/TEE/secure-element) — stronger device
  bootstrap, but draft-status and not widely deployed as of 2026. [enrollment-research]
- **Teleport trusted clusters** [enrollment-research; goteleport.com docs]. Leaf cluster joins root via
  a `trusted_cluster` **join token** (`tctl tokens add --type=trusted_cluster --ttl=…`); leaf dials out,
  root verifies, trust established; leaf runs behind firewall with **no ingress** (reverse tunnels).
  Join methods: static (discouraged), ephemeral TTL tokens, IAM (AWS/GCP/Azure), CI, k8s in-cluster,
  **TPM** (`teleport tpm identify`). **Critical caveat:** a leaf inherently trusts the root CA, so a
  root-issued cert can reach the leaf *directly, bypassing leaf `cluster_labels`* — transitive trust
  foot-gun; role-mapping is the only real constraint.
- **Tailscale/Headscale + WireGuard** [enrollment-research; tailscale.com docs]. Device dials control
  plane (SaaS or self-hosted Headscale), authenticates (user OIDC or **auth-key**), receives WireGuard
  keying + ACLs. **Tags** = ACL-based service identity for non-user devices. Mutual auth = WireGuard
  Noise handshake over static public keys; **no certificate chains, no subordinate CAs — trust is
  centralized in the control plane, per-hop point-to-point only.** No transitive cryptographic trust.
- **Kubernetes kubelet TLS bootstrapping** [enrollment-research; kubernetes.io docs]. Bootstrap token
  (Secret) → kubelet dials API server → authenticated to `system:bootstrappers` → creates a **CSR**
  (`signerName: kubernetes.io/kube-apiserver-client-kubelet`) → auto-approved by controller-manager →
  receives client cert → auto-rotation on expiry. **Star topology only** — nodes auth to the API server,
  never to each other; clean but no multi-hop semantics.

### Transitive vs per-hop trust (the load-bearing distinction)

[enrollment-research] Transitive trust ("the root can authorize a node 3 hops away without
re-authenticating it") is *enabled* by shared trust anchors / subordinate-CA chains / federation
bundles (SPIFFE federation, Teleport root-CA, PKI chains). Per-hop trust limits an intermediate node to
"I have a secure authenticated channel to my neighbor" and nothing more. The Teleport leaf-label-bypass
caveat is the canonical demonstration that transitive trust is a **security liability** when the tree
crosses residency/segmentation boundaries — which is precisely Prism's Purdue/MSSP case.

### Lean

- **Identity model:** SPIFFE-style. Satellite identity = a URI (`prism-sat://<trust-domain>/<sat-id>`)
  bound to a **short-lived X.509 cert** chaining to a per-trust-domain CA held at the coordinator (or at
  a Relay Satellite that is a sub-CA for its subtree — see Topic 5 residency). Reuse Prism's existing
  newtype + redacted-Debug credential discipline for the bootstrap secret and the agent private key.
- **Bootstrap:** SPIRE-style **node attestation via a one-time/TTL join token** distributed out-of-band
  at deploy time (matches k8s bootstrap-token + Teleport ephemeral-token + SPIRE join-token consensus).
  Optional TPM attestation as a hardening upgrade for high-assurance OT deployments (Teleport `tpm`
  precedent). First dial presents the join token over TLS to the upstream; upstream verifies, issues the
  Satellite's first SVID + the trust anchor (the upstream's CA bundle).
- **Per-hop mutual auth = mTLS at every hop, per-hop trust ONLY.** A relay validates its child against
  the child's SVID and its parent against the parent's SVID; it does NOT vend a cross-hop transitive
  credential. This deliberately rejects the Teleport root-CA-reaches-leaf model because Prism's tree
  crosses residency boundaries where transitive trust would violate §17.5.
- **Do NOT take SPIRE as a runtime dependency.** Implement the SVID-issuance + node-attestation +
  short-lived-cert-rotation natively in Rust. Rationale: air-gap/edge deployment (§3.2 #2), zero
  external control-plane dependency, and Prism's ephemeral/self-contained ethos. SPIFFE is the *model*;
  SPIRE is *not* the *runtime*. [model-knowledge + enrollment-research]

### Open questions

- Trust-anchor rotation across a deep tree without a flag-day: how does a CA roll propagate hop-by-hop?
  (SPIFFE rotates trust bundles frequently; mechanism for multi-level propagation is under-specified in
  the standard — `[INCONCLUSIVE]` for multi-hop chains.) [enrollment-research]
- Is a Relay Satellite a **sub-CA** (issues SVIDs to its subtree → transitive-trust risk) or merely an
  **mTLS terminator + re-originator** (per-hop only, no issuance)? The residency invariant (Topic 5)
  pushes toward the latter; convenience pushes toward the former. Architect call.
- Join-token distribution channel for truly air-gapped enclaves (no out-of-band network) — sneakernet?
  QR/printed token (SSH-reverse-tunnel "blink an LED / print an ID" precedent in transport-research)?
- Does Prism want JWT-SVID (for non-mTLS contexts, e.g., signed result envelopes) in addition to X.509?

---

## Topic 2 — Dial-home / outbound-only transport

### Prior art

- **NATS leaf nodes** [transport-research; Synadia/NATS docs]. Leaf server dials hub **outbound on
  7422**; **no inbound rules / VPN / port-forward**. Subject routing crosses the leaf boundary
  (export/import); coordinator publishes to a subject with leaf-side subscribers → message delivered
  down. Queue semantics preserved (local consumers first). Disconnection → leaf operates locally,
  **buffers, replays on reconnect**; built-in reconnection. JetStream adds durable streams + **domains**
  (`$JS.<domain>.API.>`) + mirrors across leaf connections. `async-nats` is the Rust/Tokio client.
- **gRPC bidirectional streaming over HTTP/2** [transport-research; grpc.io docs]. Satellite = gRPC
  *client*, dials out over TLS:443 (firewall-friendly). Opens a long-lived **bidi stream**; either side
  may send first → coordinator pushes `WorkItem`s down, satellite returns `WorkResult`s up. HTTP/2
  multiplexes many streams over one TCP conn (solves HTTP-layer HOLB; **TCP-layer HOLB remains** — a lost
  packet stalls all streams on the conn). Keepalive via HTTP/2 **PING** (~60s idle keeps conns alive
  through middleboxes). **tonic** = the mature Rust/Tokio gRPC stack (hyper + h2).
- **Reverse tunnels** [transport-research]. **Teleport** reverse tunnels (agents dial Proxy; Proxy is
  the only public component; HTTP_PROXY/HTTPS_PROXY CONNECT traversal); **ngrok** (agent dials cloud over
  persistent TLS; cloud owns DNS/cert/ports; upstream IP hidden); **SSH `-R`** (classic, robust with
  systemd + TCPKeepAlive but operationally fiddly at fleet scale); **yamux / tokio-yamux** (generic
  stream multiplexer over any reliable transport — either side opens streams, flow control, keep-alives,
  back-pressure, NAT-traversal-oriented; a Rust building block for a custom reverse channel).
- **MQTT bridges** [transport-research; EMQX docs]. Edge broker's bridge connector dials remote broker
  outbound (TCP/TLS); Forwards/Subscriptions map topics across the boundary; **QoS 0/1/2** + automatic
  reconnect + message buffering. `rumqttc`/`rumqttd` = Rust clients. IoT/constrained-device sweet spot.
- **HTTP/3 / QUIC / WebTransport** [transport-research]. QUIC multiplexes independent streams with
  **per-stream flow control → no TCP-layer HOLB**; WebTransport over HTTP/3 supports uni/bidi streams +
  datagrams. The HOLB-free upgrade path; Rust QUIC ecosystem (quinn — note Prism already pins
  quinn-proto per recent commit) is maturing but operationally younger than HTTP/2/tonic.

### Lean

- **Primary: gRPC bidirectional streaming over HTTP/2 via `tonic`** for the control+result plane.
  Rationale: (a) TLS:443 is the most firewall-friendly outbound posture (§3.2 #5); (b) bidi-stream
  reverse-RPC is the exact inversion the mesh needs; (c) tonic is the most mature Rust/Tokio gRPC stack,
  and Prism is already a tonic/hyper-adjacent Tokio codebase; (d) protobuf result envelopes align with
  Prism's OCSF+protobuf normalization boundary. Configure HTTP/2 PING keepalive (~30–60s) + exponential
  backoff reconnect with jitter. [transport-research, model-knowledge]
- **Strong alternative / complement: NATS leaf-node hierarchy** if the mesh wants broker-grade
  store-and-forward, subject-routed fan-out, and durable JetStream replay "for free" rather than
  building it on gRPC. The NATS leaf topology *is* the hub-spoke+tree model in §3.2, natively. The
  tradeoff is an embedded NATS server as a mesh dependency vs. a self-contained tonic transport.
  **Discussion fork worth surfacing:** gRPC-native (more control, fewer deps, more to build) vs.
  NATS-leaf (topology + S&F + reconnect handed to you, one more moving part). [transport-research]
- **Watch the TCP-HOLB cost** of single-connection HTTP/2 multiplexing when a relay carries bulk
  result streams alongside latency-sensitive control. Mitigation: separate control vs. bulk-result
  connections, or adopt HTTP/3/QUIC (quinn) when the Rust QUIC stack maturity clears Prism's bar.
  Flag as an explicit cost, not a silent default. [transport-research §3.3/§6]

### Open questions

- One multiplexed connection per hop, or separate control vs. result-data connections to dodge TCP-HOLB?
- gRPC-native transport vs. NATS-leaf transport — a genuine architecture fork; needs an ADR.
- Corporate TLS-inspection / MITM proxies (transport-research flags this for both gRPC and WebSocket):
  does Prism need pinning, and how does pinning survive an inspecting proxy in OT enterprise networks?
- Heartbeat cadence vs. OT low-bandwidth/power constraints (§3.2 #4) — PING interval is a tradeoff.

---

## Topic 3 — Chaining / tree topology + multi-hop relay

### Prior art

- **NATS leaf-node hierarchies** [transport/chaining-research]. The canonical outbound-only *tree*:
  leaves dial hubs, hubs can be leaves of higher hubs; subjects propagate down, replies up; gateways /
  superclusters for cross-cluster. Closest topological match to §3.2.
- **Aggregation trees / hierarchical brokers** [chaining-research; model-knowledge]. **Prometheus
  federation** (a parent Prometheus scrapes child Prometheis — hierarchical pull aggregation);
  **OpenTelemetry collector chaining / cascading collectors** (collector → collector → backend, each
  layer can process/aggregate/sample). These are the "intermediate node aggregates its children's
  results" pattern.
- **CDN parent/child hierarchical caching** [chaining-research; model-knowledge]. Varnish / Apache
  Traffic Server **parent-child**, **request coalescing** (collapse N child misses into 1 upstream
  fetch) — directly relevant to the §17.8 chain cache: a relay can coalesce identical inward sub-queries.
- **gRPC deadline propagation + per-hop decrement** [model-knowledge: grpc.io deadline docs;
  chaining-research]. The canonical pattern: a deadline is an absolute time; each hop computes the
  residual (`deadline − now − hop-budget`) and forwards it downstream; a hop with non-positive residual
  fails fast (`DEADLINE_EXCEEDED`) without dialing further. This is exactly §17.8 Q3's per-hop
  deadline-decrement requirement.
- **Loop prevention** [model-knowledge: IP TTL (RFC 791), BGP AS-path loop detection, multicast RPF;
  chaining-research]. Three independent, composable mechanisms: (a) **seen-request-ID set** per hop
  (reject duplicates — §3.2 already specifies this); (b) **hop-count TTL** decremented per hop, hard-drop
  at 0 (IP-TTL analog — bounds a cycle even if IDs are mutated); (c) **path-vector** (carry the list of
  satellite-IDs traversed; reject if self appears — BGP AS-path analog, also yields a free topology
  trace for diagnostics).

### Lean

- **Topology = rooted aggregation tree** (root = coordinator/central Prism), modeled most directly on
  the **NATS leaf-node hierarchy** + **OTel collector chaining** patterns: each non-leaf node executes
  locally AND aggregates its subtree's results inward.
- **Per-hop deadline decrement: adopt gRPC deadline semantics verbatim** — absolute deadline,
  residual computed and forwarded per hop, fail-fast on non-positive residual, plus a per-hop budget
  reservation so a relay doesn't dial a subtree it can't possibly hear back from in time. Ties §17.8 Q3.
- **Loop prevention: belt-and-suspenders — keep the §3.2 seen-request-ID set AND add a hop-count TTL.**
  Rationale: a request-ID set alone fails if IDs are regenerated mid-path (bug or malice); a hop-TTL is
  a cheap hard ceiling that bounds *any* cycle. Optionally carry a path-vector for both stronger loop
  detection and free topology/health diagnostics (feeds the §3.2 heartbeat/topology-health requirement).
  [model-knowledge: defense-in-depth from IP/BGP/RPF]

### Open questions

- Max chain depth (a hop-TTL ceiling) — an explicit ADR value (§3.2 lists "chaining depth limits" as a
  needed ADR). Purdue is ~5 layers; MSSP nesting could be deeper. Pick a production ceiling + rationale.
- Result aggregation at a relay: streaming/incremental merge as children return, or barrier-wait? The
  deadline model argues for **incremental** (emit partial as children complete; close out on deadline).
- Does the relay re-plan/push-down (DataFusion-Federation remote-subplan, per §3.2 G-6 / day-2 ADR note)
  or blindly relay the inward plan? Push-down at each layer is the residency-friendly answer.
- Heartbeat propagation: hop-by-hop liveness vs. end-to-end? Path-vector gives topology for free.
- `[INCONCLUSIVE-DEPTH]` — the chaining deep-research doc was confirmed-present + topic-complete but not
  fully paginated; the aggregation-tree specifics above lean on model knowledge of well-documented
  systems (Prometheus federation, OTel chaining, CDN parent caches) which the doc corroborates.

---

## Topic 4 — Partial-failure + store-and-forward through hops

### Prior art

- **Elastic/OpenSearch Cross-Cluster Search `skip_unavailable`** [partial-failure-research; Elastic
  docs]. The canonical "degraded, not failed" model: an unavailable remote cluster is **skipped**, not
  fatal; the response carries `_clusters: {total, successful, skipped, failed, running, partial}`
  metadata so the consumer knows exactly which slice answered. **This is the direct analog of Prism's
  §3.6 coverage banner + BC-2.01.010 partial-result semantics** — the day-2 vision already cites CCS as
  the lineage (ADOPT-3, §10.3).
- **NATS JetStream store-and-forward** [transport/partial-failure-research]. Durable streams; leaf
  buffers during disconnection and replays on reconnect; mirrors/sources for cross-domain replication.
- **MQTT QoS + persistent sessions** [partial-failure-research; MQTT spec]. QoS 0/1/2 + persistent
  session + message queuing for offline clients; broker store-and-forwards to a reconnecting client.
- **Disk-backed durable queues / WAL** [partial-failure-research; model-knowledge]. Segment-based
  append logs, WALs, RocksDB-backed buffers for gap-buffering with explicit **backpressure** when full.
- **"Subtree unreachable, not lost"** [partial-failure-research; CCS lineage]. Coverage metadata +
  degraded-region indicators + eventual reconnect/drain — the federated, residency-aware framing.

### Lean

- **Partial-failure: extend the EXISTING primitive, don't invent a new one.** Prism already owns
  BC-2.01.010 partial-result + the §3.6 CCS-lineage coverage banner. The mesh extension is: a relay that
  loses a child surfaces the child's subtree as **skipped** (with a reason + last-seen timestamp), and
  relays that gap **upward unmodified through every hop** — no hop may swallow it (§3.2's "no hop can
  silently swallow a downstream failure" + Prism's Standing Rule 3 §2 no-silent-`Vec::new()`). The
  coverage banner accumulates per-hop skip metadata so central Prism reports exactly which Purdue layer /
  region / tenant subtree was unreachable. [partial-failure-research, CCS docs, BC-2.01.010]
- **Store-and-forward at the ingestion-locus relay (§17.4): a RocksDB-backed durable queue.** Prism
  already runs RocksDB (19 column families); a durable, segment-style buffer column-family at the
  collection-capable Satellite is the lowest-new-dependency path. Push data lands locally, buffers during
  gaps, drains inward on reconnect; **explicit backpressure** (bounded buffer + drop-policy or
  apply-backpressure-to-source) when full — surfaced as a coverage/degraded signal, not silent loss.
  [partial-failure-research, model-knowledge: WAL/segment queues; §17.4 locus]
- **Distinguish two failure classes explicitly:** (a) *transient* (relay offline, buffer-and-replay —
  S&F applies) vs. (b) *hard* (subtree genuinely unreachable past deadline — surface as skipped in the
  coverage banner). Conflating them is the trap.

### Open questions

- Buffer durability vs. residency: if a relay buffers RAW push data on disk during a gap, does that disk
  buffer violate residency if the relay sits at a higher Purdue layer? (Answer likely: the locus/buffer
  MUST be the in-region edge satellite — ties Topic 5 + §17.4 locus (a)/(b).)
- Backpressure policy when the durable buffer fills: drop-oldest, drop-newest, or backpressure-to-source?
  Security telemetry usually wants drop-oldest-with-loud-coverage-signal, never silent loss.
- Replay ordering + dedup on drain (at-least-once vs. exactly-once) — interacts with loop-prevention
  request-IDs. JetStream/MQTT-QoS-2 give exactly-once at a cost; QoS-1 + idempotent dedup is the usual lean.
- `[INCONCLUSIVE-DEPTH]` — the partial-failure deep-research doc was confirmed-present but not fully
  paginated; CCS/JetStream/MQTT specifics are corroborated by model knowledge + Prism's existing §3.6/ADOPT-3 lineage.

---

## Topic 5 — Residency + OT/Purdue use cases

### Prior art

- **IEC 62443 zones-and-conduits** [residency-research; IEC 62443 / ISA-99]. Industrial networks
  partition into **zones** (grouped assets at a security level) connected only by controlled
  **conduits**; each conduit enforces a security level. Maps cleanly onto a Satellite chain:
  enterprise → DMZ → OT-L3 → L2 → L1, one satellite per zone, the inter-zone link = the conduit, and the
  satellite is the conduit's policy-enforcement point. [residency-research]
- **NIST SP 800-82** (ICS security guide) [residency-research]. Companion guidance to the Purdue/IEC
  model for segmentation and one-way flows.
- **Data diodes / unidirectional gateways** [residency-research; Waterfall/Owl]. Hardware-enforced
  one-way flow OUT of a high-security OT zone — the strongest structural residency guarantee. A bastion
  Satellite (§3.2 #2) sits at the boundary; a diode can sit on the conduit for the highest-assurance case.
- **Bastion / jump-host** [residency-research]. Single controlled crossing point for air-gapped enclave
  access — exactly §3.2 #2's "single bastion satellite."
- **Residency-by-construction / federated analytics** [residency-research; model-knowledge: federated
  learning, GDPR localization, "only aggregates leave"]. The structural pattern: raw data is processed
  *in-region/in-zone*; only normalized/aggregated/sanitized results cross the boundary. This is exactly
  Prism's ephemeral-federated-residency-first ethos (§17.5) and the §3.2 #6 fan-in/residency-hop.
- **MSSP nested multi-tenant topologies** [residency-research]. spoke → regional-hub → central, with
  tenant/region isolation preserved at each aggregation layer (§3.2 #3).

### Lean

- **Residency is enforced STRUCTURALLY by what crosses the conduit, not by policy config.** The mesh
  invariant: a Satellite normalizes raw → OCSF/native-schema **at the edge, in-zone**, and **only the
  normalized result transits the conduit upward.** Raw never crosses a Satellite boundary (matches §17.5
  residency-by-construction + §3.2 #6 + §17.4 OT/Purdue locus example: L2 satellite hosts the syslog
  receiver; only OCSF Network-Activity/Authentication events transit to the L4 satellite). This is the
  federated-analytics "only aggregates leave" pattern applied to security telemetry. [residency-research]
- **Map satellite chaining onto IEC-62443 zones-and-conduits explicitly** — one satellite per zone, the
  inter-satellite hop = the conduit, per-hop mTLS (Topic 1) = the conduit's authentication control. This
  gives Prism a recognized standards anchor (IEC 62443 / NIST 800-82) for the OT story, which matters for
  the MSSP/1898 audience.
- **This is THE argument for per-hop trust over transitive trust (Topic 1):** transitive trust would let
  a higher-zone identity reach into a lower OT zone (the Teleport leaf-bypass foot-gun), structurally
  violating zone separation. Per-hop trust + edge-normalization is what makes residency-by-construction
  actually hold. [residency-research + enrollment-research]

### Open questions

- Where does credential resolution happen for in-enclave sources? (Memory `project_ai_opaque_credentials.md`
  + §3.2/§17.5 note: `SecretBackend` is **satellite-local** in the OT topology — creds resolved AT the
  satellite, never sent to central. This should be a hard invariant.) [vision §3.2 line 1072-1074]
- Diode-compatible mode: can the dial-home transport (Topic 2) operate over a true unidirectional link
  for the highest-assurance OT zone, or does mutual-auth mTLS inherently require bidirectionality?
  (Likely needs a store-and-forward + one-way-result variant — a real design fork for diode deployments.)
- Per-zone normalization-schema enforcement: how does central verify a relay actually stripped raw and
  only forwarded normalized? (Attestation of the normalization step? Schema-validated conduit?)

---

## Topic 6 — Relay/aggregator role-noun (§3.4 TBD)

### Survey [role-noun-research]

| Term | Canonical origin | Connotation | Fit for "executor + subtree aggregator" |
|------|------------------|-------------|------------------------------------------|
| **Relay** | radio/telecom relay station | pure *forwarding* | partial — implies forwarding, not local execution |
| **Hub** | Ethernet L1 hub (star) | pure *repeater/broadcast* | poor — "dumb repeater" connotation |
| **Aggregator** | data/traffic aggregation trees | strong *combine/summarize* | good for the aggregation half; silent on downstream-parent role |
| **Concentrator / edge concentrator** | telecom/SNA line concentrators (many→one) | strong *many-to-one aggregation* | strong — aggregation + edge heritage; less "executor" |
| **Gateway** | TCP/IP internetworking, IoT gateways | *boundary/protocol translation* | partial — boundary-crossing, not interior-tree |
| **Leaf node** | NATS leaf (border broker) vs. graph-theory leaf (no children) | **CLASHES** — graph-leaf = no children | avoid — NATS overloads it; collides with tree "leaf" |
| **Superpeer** | P2P (KaZaA/Gnutella) | **dual executor + relay/aggregator for child peers** | best *semantic* match; P2P baggage |
| **Parent / interior / branch node** | graph theory / tree topology | neutral *non-leaf* (executes + forwards/aggregates) | most precise, least overloaded, generic |
| **Broker** | message-oriented middleware | *mediation + routing + persistence* | partial — implies pub/sub mediation |
| **Rendezvous point** | multicast PIM-SM | *central registration point* | poor — root-like, not interior |

### Lean

- **Primary: "Relay Satellite."** Every node is a "Prism Satellite" (confirmed, D-1330); a non-leaf one
  that relays/aggregates for a subtree is a **"Relay Satellite"** (vs. an **"Edge Satellite"** /
  "Leaf Satellite" for a pure executor at the tree edge, and the central node which is the
  Coordinator). Rationale: it's a *role qualifier* on the confirmed "Satellite" noun, reads naturally in
  the §3.2 topology, and "relay" + "Satellite" together already imply the dual role in context. Avoids
  inventing a new top-level noun.
- **Runner-up: "Concentrator Satellite"** if the discussion wants to foreground the many-to-one
  *aggregation/residency* role (telecom heritage, strongest "fan-in" connotation — fits §3.2 #6 and the
  MSSP regional-hub). Slightly more jargon.
- **Avoid:** "leaf" (NATS-vs-graph-theory clash), "hub" (pure-repeater connotation contradicts the
  executor role — though §3.2 already uses "hub-spoke" for *topology*, which is fine), bare "gateway"
  (boundary-translation connotation), "superpeer" (correct semantics but heavy P2P baggage for a
  coordinator-rooted tree).

### Open question

- Three-noun set vs. two? Candidate: **Coordinator** (central root) · **Relay Satellite** (non-leaf
  executor+aggregator) · **Edge Satellite** (leaf executor). Business-analyst + PO call (§3.4 owns it).

---

## How the mesh underpins §17 ingestion locus + §17.8 chain cache

- **§17.4 collection locus.** A "collection-capable Satellite" (locus a/b) is just an Edge or Relay
  Satellite that additionally **hosts a receiver endpoint + buffer** (Topic 4 store-and-forward). The
  transport (Topic 2) carries the *pull* of buffered data inward on query — the §17.5 "push lands
  locally, pull retrieves on query" mechanic is exactly the reverse-RPC inward-plan/outward-result flow
  (Topic 3). No new transport needed; the listener-hosting capability is the only addition. [vision §17.4/§17.5]
- **§17.8 chain-aware cache / replication / deadlines.** (Q3 deadline) = the gRPC per-hop deadline
  decrement (Topic 3 lean). Chain cache = CDN parent/child hierarchical caching + request-coalescing
  (Topic 3 prior art): a Relay Satellite caches/coalesces identical inward sub-queries for its subtree.
  Replication across the chain = JetStream mirrors/sources or the RocksDB durable-queue drain (Topic 4).
  The residency invariant (Topic 5) constrains WHAT may be cached/replicated at each layer (normalized
  only, never raw, above the in-region edge). [vision §17.8; chaining/partial-failure-research]

---

## Consolidated open design questions (for the discussion)

1. **Transport fork:** gRPC-native (tonic, more control, fewer deps, build-S&F-yourself) vs. NATS-leaf
   (topology + store-and-forward + reconnect for free, one embedded broker dependency). → ADR.
2. **Trust model:** per-hop-only (residency-safe, more re-auth) vs. transitive (convenient, Teleport
   foot-gun, violates Purdue zone separation). Lean strongly per-hop; confirm. → ADR (§3.2 enrollment ADR).
3. **Relay = sub-CA (issues subtree SVIDs) or mTLS terminator only (no issuance)?** Residency pushes
   terminator-only.
4. **Max chain depth** (hop-TTL ceiling) — explicit production value + rationale. → ADR (§3.2 depth-limit ADR).
5. **Loop prevention:** request-ID set + hop-TTL (lean) ± path-vector. Confirm the belt-and-suspenders set.
6. **Result aggregation:** incremental/streaming merge (deadline-friendly) vs. barrier-wait at each relay.
7. **Push-down at each layer** (DataFusion-Federation remote-subplan, residency-friendly) vs. blind relay.
8. **Store-and-forward buffer:** RocksDB durable queue (lean, lowest-dep) + backpressure policy
   (drop-oldest-with-coverage-signal vs. backpressure-to-source).
9. **Replay semantics on drain:** at-least-once + idempotent dedup vs. exactly-once cost.
10. **Diode-compatible one-way mode** for highest-assurance OT zones — does mutual-auth mTLS preclude it?
11. **Satellite-local credential resolution** as a hard invariant (creds never transit to central) —
    confirm and bind to AD-017 / `project_ai_opaque_credentials.md`.
12. **Role-noun set:** Coordinator / Relay Satellite / Edge Satellite (lean) — business-analyst + PO (§3.4).

## Honest costs

- **Build vs. adopt:** the gRPC-native lean means Prism builds its own enrollment, store-and-forward, and
  reconnect — real engineering cost vs. NATS-leaf handing much of it over (at a dependency cost). This is
  a genuine tradeoff, not a free lunch; surfaced as a fork, not buried.
- **Per-hop trust costs re-authentication at every hop** (more handshakes, more cert rotation surface)
  vs. transitive convenience — accepted as the price of residency-by-construction.
- **TCP-HOLB on single-connection HTTP/2 multiplexing** is a real latency risk for mixed control+bulk
  traffic; HTTP/3/QUIC (quinn) is the fix but the Rust QUIC stack is operationally younger.
- **Durable buffering at edge relays adds disk + RocksDB column-family management** at every
  collection-capable Satellite; backpressure-when-full must be designed, not defaulted.
- **Diode/one-way OT mode may not be expressible** over mutual-auth mTLS without a separate
  store-and-forward + one-way-result transport variant — potentially a whole second transport path for
  the highest-assurance deployments. `[INCONCLUSIVE]` until prototyped.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 5 | Deep multi-source synthesis: (1) enrollment/identity bootstrap (SPIFFE/SPIRE, EST, ACME, Teleport, Tailscale, k8s); (2) outbound-only dial-home transport (NATS leaf, gRPC bidi, reverse tunnels, MQTT, HTTP/3); (3) chaining/tree/multi-hop relay + deadline decrement + loop prevention; (4) partial-failure + store-and-forward (CCS, JetStream, MQTT QoS, WAL queues); (5) residency + OT/Purdue (IEC 62443, data diodes, bastion, MSSP nesting). All at reasoning_effort=high. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 1 | Focused role-noun terminology survey (relay/hub/aggregator/concentrator/superpeer/leaf/parent-node) with canonical origins. |
| Context7 | 0 | — (no single-library API question; topic is architectural/standards prior art) |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~4 areas | Clearly flagged inline `[model-knowledge]`: gRPC deadline-propagation semantics; IP-TTL / BGP-AS-path / multicast-RPF loop-prevention; Prometheus federation / OTel collector chaining / CDN parent-cache aggregation mechanics; WAL/segment durable-queue + federated-analytics "only aggregates leave" patterns. Used to corroborate the 3 deep-research docs that could not be fully paginated, NOT as primary source. |

**Total MCP tool calls:** 6 (5 perplexity_research at reasoning_effort=high + 1 perplexity_ask).
**Training data reliance:** low-to-medium — primary findings are web-grounded via 5 high-effort deep-research calls; model knowledge is used only to corroborate well-documented mechanics on the 3 docs (chaining, partial-failure, residency) that could not be fully paginated from their single-line tool-result files, and is explicitly flagged at every use. The enrollment and transport docs were read substantively (through conclusion / §6).

**Compliance note:** MANDATORY MCP gate satisfied (6 MCP calls; 5 are the PRIMARY `perplexity_research`
tool, the expected default for a non-trivial multi-source architecture topic). No MCP-UNAVAILABLE
escalation needed.
