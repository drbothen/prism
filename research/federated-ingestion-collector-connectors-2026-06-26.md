---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-26"
provenance: "side-analysis discussion input; does not modify vision/specs."
topic: "Federated ingestion of push/stream/capture data — collector / stream-connector class"
scope: "SIDE-ANALYSIS / discussion input — NOT a spec, vision, brief, ADR, BC, or PRD change."
---

# Federated Ingestion of Push / Stream / Capture Data — The Collector / Stream-Connector Class

> **BOUNDARY.** This is a cited research capture to inform a HUMAN DISCUSSION. It does **not**
> modify `matured-vision-day2-requirements.md`, any spec, STATE.md, SESSION-HANDOFF.md, or any
> live factory artifact. It does **not** decide the open questions it raises. The working
> hypothesis stated by the requester is **pressure-tested**, not adopted. `do_not_execute: true`.

> **Citation discipline.** Findings are grounded in MCP-tool-retrieved web research (Perplexity
> `sonar-deep-research`, four deep passes). Vendor/RFC sources are named inline. Where a claim
> rests on model knowledge rather than a retrieved source, it is flagged **[model-knowledge]**.
> Where research was inconclusive, it is flagged **[INCONCLUSIVE]**. Date-stamped as of 2026-06.

---

## 1. Executive Summary (10 lines)

1. Push/stream/capture data (syslog, pcap, NetFlow/IPFIX, Kafka, webhooks, object-drop) has **no queryable API to federate in place** — the source initiates, the receiver cannot pull on demand. This is the structural reason it cannot be treated like a CrowdStrike/Splunk API connector.
2. Every mature edge-pipeline product (Cribl Stream/Edge, Vector, Fluent Bit/Fluentd, Logstash/Beats) implements the **same four-part abstraction**: a *receiver/listener endpoint* → a *local buffer (store-and-forward)* → a *boundary normalization* stage → a *queryable/forwardable surface*. This abstraction maps cleanly onto Prism's Satellite + RetentionCache + connector-taxonomy pillars.
3. Cribl's "neutral incentive" framing (no ingest-priced analytics backend, so reduction-and-route is a *selling point* not a conflict) is real and is the **closest external analogue to Prism's own positioning** — it strengthens, not weakens, the federation thesis.
4. **Syslog** is two problems: a transport problem (UDP loss / no backpressure / TCP framing / TLS / RELP reliability) and a schema problem (free-form `MSG`, vendor formats, CEF/LEEF). Firewall/auth/network-device logs map to OCSF Network Activity / Authentication classes with real gaps.
5. **pcap** validates the requester's hypothesis precisely: Arkime/Moloch **splits full packets (rolling disk buffer) from session metadata (searchable index)**; you query metadata, retrieve raw packets on-demand by session ID. Zeek/Suricata turn packets into metadata logs. Nobody retains full packets long-term — the economics forbid it.
6. The honest reframe survives scrutiny: **a collector is a SOURCE and a buffer, not a central sink.** "No ingestion" was never literally true even for pull connectors (the RetentionCache already buffers); a collector is the same pattern with a *receiver endpoint* bolted on the front.
7. Where it **genuinely strains**: full-take pcap volume (TB/day at 10Gbps), long retention of high-volume streams, and the need to hold *streaming correlation state* (windows/watermarks) that a pure pull-then-cache model does not naturally express.
8. **Streaming detection** (Flink/Kafka Streams windowed operators; Zeek scripts; Suricata inline sliding-window) is mature prior art for "detect as it arrives." Prism's detection-as-query over a short-TTL RetentionCache is a *near-real-time scheduled-query* variant (à la Splunk real-time / Sentinel NRT) — weaker than a continuous operator but far simpler.
9. **Collection locus** (satellite-edge / standalone collector / central-managed) trades off residency-vs-egress, volume/cost, latency, air-gap fit, and failure modes — the research finds **no single winner**; it is a per-deployment decision.
10. **Top recommendation for the discussion:** treat "collector" as a *first-class connector subtype with a receiver endpoint*, reuse the Satellite + RetentionCache + capability-descriptor machinery rather than building new, and decide explicitly whether streaming-correlation-state is in scope (the one place the pull-then-cache model does not stretch cleanly).

---

## 2. The Collector-Class Abstraction (the unifying spine)

The single most load-bearing finding across all five push/stream/capture source types is that
they collapse to **one abstraction with four stages** (Perplexity deep-research synthesis over
NetFlow/IPFIX, sFlow, Kafka, webhooks, and S3/object-drop; sources include RFC 7011, RFC 3954,
AWS S3/SQS/SNS docs, Confluent/Fluentd docs):

| Stage | What it is | Per-source instantiation |
|-------|-----------|--------------------------|
| **1. Receiver / listener endpoint** | An externally addressable ingress surface the source pushes to | UDP port (NetFlow 2055 / sFlow / syslog 514); TCP/TLS port (syslog 6514); HTTP path (webhook); Kafka consumer-group subscription; SQS queue ARN / SNS subscription (S3 events); a file-system / SMB path (drop dir) |
| **2. Local buffer (store-and-forward)** | Bridges bursty arrival against controlled downstream processing | Kernel NIC ring + UDP socket receive buffer (NetFlow/sFlow/UDP syslog — *lossy*); disk-backed persistent queue (Cribl PQ, Vector disk buffer, Fluentd/Logstash persistent queue); Kafka's persisted partition log + consumer fetch buffer; SQS/SNS managed queue; staging directory (file-drop) |
| **3. Boundary normalization** | Heterogeneous wire format → common schema (OCSF for security telemetry; native schema-on-read otherwise) | Parse syslog header + MSG (+ embedded CEF/LEEF); template-decode NetFlow v9 / IPFIX records; sessionize packets → flow/session metadata (Arkime SPI, Zeek conn.log, Suricata flow); parse webhook/S3 JSON |
| **4. Queryable / forwardable surface** | Where detection/analytics/query runs | Time-series DB / search index (Arkime→OpenSearch); SIEM/lake; **or** — in Prism's model — the RetentionCache + PrismQL `FROM cache.<name>` |

**Why this matters for Prism:** Prism already has stages 2–4 (RetentionCache buffer, OCSF/native
normalization boundary, PrismQL queryable surface). The *only genuinely new* element a collector
adds is **stage 1 — a receiver endpoint** — plus the consequences of accepting push semantics
(no backpressure to the source; loss handling; store-and-forward durability).

### 2.1 PUSH vs PULL — the structural reason push cannot be "federated in place"

- **Pull (today's Prism connectors):** the receiver issues a request on demand against a stable, queryable API/DB. Prism federates "in place" because it can *ask* the source for exactly the time-bounded slice a query needs, when the query runs. No data lands until demand exists. (Aligns with the data-virtualization model — Perplexity synthesis.)
- **Push (the collector class):** the source autonomously initiates transmission to an endpoint. The receiver **cannot ask for arbitrary historical subsets** and **cannot replay** a transmission. NetFlow/sFlow exporters send on their own timers; webhooks fire on events; S3 notifies on PutObject. There is no API to query "in place."
- **Consequence:** push sources **must be landed** (buffered/persisted) before they can participate in a federated query. This is not an implementation choice — it is intrinsic to push transport. **This is the crux the discussion must confront:** accepting push data necessarily means accepting a *landing buffer*, which is exactly what the RetentionCache is.
- **Kafka is the interesting hybrid:** producers *push* to brokers, but consumers *pull* from the durable log with explicit offsets. Backpressure manifests as *consumer lag* (data waits in Kafka) rather than *drop* (UDP). A Kafka topic is therefore the *most* federatable push source — it is closer to a queryable, replayable surface than any UDP exporter (Confluent/Fluentd docs; Perplexity synthesis).

---

## 3. Per-Source-Type Findings

### 3.1 Prior art — edge collection / pipeline products

**Common pattern (all six tools):** sources/inputs → transforms/filters → sinks/outputs, with an
explicit decoupling of source from destination plus a buffer. None *requires* a single central
store; all can route/reduce to many destinations. (Cribl docs/blog; Vector docs; Fluent Bit
manual; Elastic docs — Perplexity deep-research.)

| Tool | Edge vs central | Buffering / store-and-forward | Central-store requirement | Notes |
|------|-----------------|-------------------------------|---------------------------|-------|
| **Cribl Stream** | Central pipeline (worker cluster) | **Persistent Queues (PQ)** disk-backed at destinations; **shared PQ** over NFS/S3 so a surviving worker drains a failed worker's queue (orphan management: `pq.orphan_detected`, `pq.shared_storage_bytes_out`) | No — routes "any source to any tool"; vendor-agnostic | Closest analogue to Prism's neutral stance |
| **Cribl Edge** | Edge agent (per-host, fleet-managed) | Local buffering inferred; **[INCONCLUSIVE]** — Cribl's docs do not fully specify Edge's on-agent disk-queue durability under prolonged disconnect | Recommends (not requires) shipping to Stream first; can deliver direct to object stores | "0-cost ingest" via Cribl HTTP — reduction is a *feature*, not a cost |
| **Vector** | Both (sidecar agent or aggregator) | In-memory buffer (~100 events between components, ~500 at sinks); **disk buffers** for durability; `drop_newest` vs blocking backpressure; **end-to-end acknowledgements** (batch notifier) — but only for sources that support ack | No — sources→transforms→sinks over gRPC; multi-destination | **syslog source is explicitly `state: stateless`, `delivery: best effort`, `acknowledgements: no`** — supports RFC 6587/5424/3164. Cannot ack back to syslog senders |
| **Fluent Bit** | Lightweight edge agent | Memory + filesystem buffer plugins | No — pluggable inputs→filters→outputs | Designed for constrained hosts / k8s |
| **Fluentd** | Central router | Buffer plugins: memory or **file-backed** chunks | No | Kafka input/output plugin → Kafka-as-source pattern |
| **Logstash + Beats** | Beats = edge shipper; Logstash = central processor | Logstash **persistent queue** (disk, sized by events/bytes) acts as store-and-forward before Elasticsearch | Technically can route to non-Elastic; in practice Elastic-aligned | Beats has processors for edge drop/enrich |

**Reduction/routing they perform:** drop verbose/health-check events, strip/redact fields,
sample high-volume streams, aggregate over windows, normalize formats, route the same stream to
multiple destinations with different transforms. (Cribl/Vector docs.)

**Cribl "neutral incentive" — verified and directly relevant (§2.5 of the vision):** an analyst
report and Cribl's own messaging confirm Cribl's revenue is *not* ingest-priced — it has no
large analytics backend whose economics depend on maximizing stored/searched volume, so
aggressive *reduction* is a selling point. This is the **same incentive structure the vision
claims for Prism** (sensor-API-native, no ingestion revenue). The federate-OR-route stance reads
as credible precisely because there is no conflicted incentive. (Perplexity deep-research;
corroborates vision §2.5 without modifying it.)

### 3.2 Syslog (DEEP)

Syslog is **two orthogonal problems** — transport reliability and schema normalization.

**Transport / framing (RFCs verified against rfc-editor.org via deep-research):**

| Spec | Focus | Transport / framing | Reliability | Security |
|------|-------|---------------------|-------------|----------|
| **RFC 3164** (BSD, legacy) | Observed behavior, *not* prescriptive | UDP/514; PRI+HEADER+free-form MSG; **1024-byte cap** | Best-effort; truncation | None |
| **RFC 5424** (IETF, structured) | Message semantics + VERSION + **STRUCTURED-DATA** (SD-ID/SD-PARAM) + precise RFC3339 timestamp | Any transport via mapping | Per-transport | Per-transport |
| **RFC 6587** (syslog/TCP) | Framing only | **octet-counting** (transparent, length-prefixed — preferred) vs **non-transparent** (newline-delimited — breaks on multi-line) | TCP in-order; **no app-level ack** | None |
| **RFC 5425** (syslog/TLS) | TLS transport mapping | TLS over TCP, **port 6514 "syslog-tls"** | Inherits TCP | Confidentiality + integrity + mutual-auth (X.509) |
| **RELP** (rsyslog/librelp) | App-level reliable delivery | Command–response model, per-message ack, retransmit on reconnect | **Strongest** — survives server restart/network break | None natively (combine with TLS) — **not an IETF RFC**, de-facto = librelp/rsyslog |

**UDP operational realities (syslog-ng, Rapid7, Axoflow via deep-research):** silent message
loss (NIC ring + kernel `UdpRcvbufErrors` overflow); **no backpressure** (sender ignores
collector state — "send and forget"); weak ordering (reorder + no sequence numbers); time-skew /
clock-drift (RFC 3164 lacks year/TZ — must infer); 1024-byte truncation drops attack-relevant
fields. Mitigations: bigger socket buffers, multiple collectors, **local rsyslog aggregator that
converts UDP→TLS/RELP for hot paths**, NTP discipline, store both device-time and receipt-time.

**Schema → OCSF mapping (schema.ocsf.io via deep-research; firewall/auth/network/audit):**
- **Maps cleanly:** firewall connection logs → OCSF **Network Activity** (4001) when CEF/LEEF or RFC 5424 structured-data provide src/dst/port/proto/action; auth/PAM → **Authentication**; SSH → **SSH Activity**; DNS → **DNS Activity**.
- **Embedded schemas help:** CEF (`CEF:0|Vendor|Product|...|key=val`) and LEEF give consistent field names that translate to OCSF (`src`→source IP, `suser`→user). Two-layer parse (syslog envelope, then embedded schema).
- **Does NOT map cleanly [flagged INCONCLUSIVE / gap]:** free-form RFC 3164 `MSG` with vendor-specific bodies; many config-change / system-health / device-internal events have **no clean OCSF home** → these are §13.6 "native schema-on-read" candidates, not OCSF. The deep-research explicitly flagged several mappings as ambiguous and noted some OCSF field structures could not be field-verified against schema.ocsf.io from the retrieved excerpts. **[INCONCLUSIVE on exact OCSF attribute paths — verify against schema.ocsf.io before any spec work.]**

### 3.3 pcap (DEEP) — validates the requester's hypothesis

**Arkime / Moloch (arkime.com, GitHub, Moloch FAQ via deep-research) — the metadata-vs-payload split is CONFIRMED:**
- **Three components:** `capture` (C app: sniffs interface, writes raw **PCAP to local disk** in `pcapDir`, parses packets → **Session Profile Information (SPI)** metadata, sends SPI to the DB); `viewer` (Node.js UI + REST API); **OpenSearch/Elasticsearch** (stores SPI metadata, time-indexed).
- **Query workflow (CONFIRMED two-step):** (1) query the **SPI metadata index** (sessions, fields, time ranges); (2) retrieve the **raw PCAP on demand by session ID + node** via the viewer API. If PCAP retention has elapsed, metadata still queryable but full packets are gone.
- **Storage split is enforced:** Arkime refuses to run with `pcapDir` and the ES data dir on the same filesystem. **PCAP retention is bounded by sensor disk (rolling buffer)**; **SPI metadata retention is bounded by ES cluster size** (ILM/ISM policies, `db.pl expire`). The exact internal PCAP-rotation algorithm was **[INCONCLUSIVE]** in the docs, but disk-bounded rolling-buffer behavior is explicit.
- **This is exactly the requester's hypothesis:** *federate the metadata (flow/session/protocol events), keep full packets as a short-TTL on-demand blob.*

**Zeek / Bro (docs.zeek.org, Corelight via deep-research):** packets → **structured metadata logs** — `conn.log`, `dns.log`, `http.log`, `ssl.log`, `files.log`. Metadata-not-payload by design. Each connection gets a **UID**; **Community ID** (Corelight spec) is a cross-tool flow hash for pivoting. Malcolm (Arkime+Zeek) maps Zeek logs into Arkime's session schema via Community ID + UID → `rootId`.

**Suricata / OISF (docs.suricata.io via deep-research):** IDS/IPS emitting **EVE JSON** event types — `flow`, `alert`, `http`, `dns`, `tls`, `fileinfo`, `stats`. `flow_id` correlates. Inline IPS mode inspects a **sliding window** over the TCP stream (`stream.inline`) — detection at capture time, not after store.

**Capture strategies + economics (deep-research):** full-take vs rolling-buffer vs trigger-based/selective. Full packet capture at 10Gbps is on the order of **TB/day** [model-knowledge for the exact rate; the research confirms "enormous" volume forcing the metadata/payload split]. **Nobody retains full packets long-term** — metadata is retained for months/years, packets for days/weeks. Privacy/regulatory pressure reinforces metadata-first.

**OCSF mapping:** flow/session metadata → **Network Activity (class_uid 4001)** and OCSF network categories; Zeek UID / Suricata `flow_id` / Community ID → flow/session identifier fields. Exact OCSF attribute paths **[INCONCLUSIVE]** — flagged by the research as general-schema knowledge not field-verified against schema.ocsf.io.

### 3.4 The broader collector class (surveyed)

| Source | Receiver endpoint | Buffer / backpressure | Record content | Federatability |
|--------|-------------------|------------------------|----------------|----------------|
| **NetFlow v5** | UDP (e.g. 2055) | NIC+socket buffer; **loss on overflow, no backpressure** | Fixed 7-tuple (ingress-if, src/dst IP, proto, src/dst port, ToS) + bytes/packets/flags/timestamps/next-hop | Push — must land |
| **NetFlow v9** | UDP | same | **Template-based** (exporter sends template, then data refs template-ID); template loss → misparse until refresh | Push — must land |
| **IPFIX (RFC 7011)** | UDP/TCP/SCTP | UDP lossy; TCP/SCTP flow-controlled | IETF-standard Information Elements; observation domains | Push — must land |
| **sFlow** | UDP | same as NetFlow | **Sampled** packet headers + counters (e.g. 1/16384); estimator must scale by sample rate | Push, lossy + sampled |
| **Kafka topic** | Consumer-group subscription | **Durable log + consumer lag** (no drop within retention) | Arbitrary (JSON/protobuf) | **Hybrid push/pull — most federatable** (replayable, offset-addressable) |
| **Webhook / HTTP-push** | HTTP endpoint | App-level queue; **HTTP 2xx ack**, sender retries → must be **idempotent** (event-ID dedup) | Arbitrary JSON | Push — must land |
| **S3 / object-drop** | SQS queue / SNS sub / EventBridge on PutObject | **Durable managed queue** (SQS visibility timeout, at-least-once) | S3 event JSON (`Records[].s3`...) then fetch object | Push-notify + pull-object — semi-federatable |
| **SMB / file-drop** | File-system / share path | Staging dir; inotify or poll | CSV / log lines | Mostly pull (poll) |

**Common abstraction = §2 (receiver + buffer + normalization + queryable surface).** Backpressure
spectrum: **UDP (silent drop) ← worst ... → HTTP/SQS (ack + retry) ... → Kafka (lag, no loss) ←
best.** This spectrum is the key engineering variable for any Prism collector.

---

## 4. Collection LOCUS — compared evenly (no prior lean)

The research finds **no universal winner**; locus is a per-deployment decision. (Synthesis over
Cribl Edge/Stream topology, Arkime NPB/sensor placement, satellite/dial-home pillar §3.2.)

| Dimension | (a) Satellite-edge collection | (b) Standalone dedicated collector component | (c) Central-side / cloud-delivered collection |
|-----------|-------------------------------|----------------------------------------------|-----------------------------------------------|
| **Residency / no-egress** | **Strongest** — raw lands in-region; only normalized/sanitized transits (matches vision §3.2 #6, §2.3 sovereignty) | Strong if deployed in-region; depends on placement | **Weakest** — raw egresses to central/cloud before reduction |
| **Volume / cost** | Reduce at source → less transits (Cribl Edge "0-cost ingest" pattern) | Reduce at the collector tier | Pay to move raw, then reduce centrally (Splunk-style ingest cost) |
| **Latency** | Low to source; adds satellite→central hop | One hop | Lowest infra hops if sources are cloud-native, but WAN for on-prem |
| **Air-gap / OT fit** | **Best** — Purdue-layer chaining, bastion satellite, dial-home (vision §3.2 #1/#2/#5) | Possible inside enclave | Poor — needs inbound or egress through the air gap |
| **Operational complexity** | Highest — fleet mgmt, enrollment, per-hop auth, store-and-forward (vision §3.2) | Medium — one tier to operate | Lowest for the customer; vendor operates it |
| **Failure modes** | Mid-chain offline → subtree drops (partial-result §3.6); store-and-forward buffers gaps | Single collector = single buffer (mitigate w/ shared-PQ like Cribl) | Central outage loses in-flight push (UDP) unless buffered at edge |

**Reading:** for Prism's stated use cases (MSSP, OT/ICS, residency, air-gap), **(a) satellite-edge
aligns most naturally with existing pillars** — but the research deliberately does not pick;
hybrids are common (edge reduce + central correlate). **Decision deferred to the discussion.**

---

## 5. Reconciliation with the Federation Thesis

**Does accepting push/capture data violate "no ingestion, no duplication, no egress" (vision §2.3 #1)?**

- **The honest answer: partially, and the vision already conceded the relevant ground.** The RetentionCache (§3.3) is *already* a demand-driven landing buffer — "no ingestion" was always "no *store-everything* ingestion," not "never persist a byte." A collector extends the same buffer with a *receiver endpoint*.
- **How Cribl-like vendors frame it:** *collection ≠ ingestion*. Collecting/routing/reducing at the edge is a telemetry **service**; "ingestion" is the act of loading into a priced analytics store. Cribl's neutral incentive lets it call reduction a feature. **Prism can use the identical reframe:** a collector is **a SOURCE and a buffer, not a central sink.** The data is landed at the edge (satellite), reduced/normalized, TTL'd, and exposed as *just another queryable source* — central Prism stays pull-based and ephemeral against that source.
- **The reframe holds for:** syslog, NetFlow/IPFIX, webhooks, Kafka-consume, object-drop, and pcap-**metadata**. In each, the edge buffer is bounded (TTL/RETAIN/detection-window) and central stays pull.
- **Where it GENUINELY STRAINS (the honest cost — do not paper over):**
  1. **Full-take pcap volume.** TB/day cannot be a "short-TTL cache" in any normal RetentionCache budget. The only honest model is Arkime's: metadata federated + full packets a *separate, disk-bounded, short rolling buffer* retrieved on demand. This is a *second storage regime*, not the RocksDB/Iceberg tiers.
  2. **High-volume stream retention.** A busy syslog/NetFlow feed at sustained rate will blow a 200MB hot budget in seconds. Either heavy edge reduction, or a sized-per-deployment buffer, or accept loss. "Configurable memory budget, server-sized" (DC-004) helps but does not eliminate the tension.
  3. **Streaming detection state.** Detection over a *stream as it arrives* wants a continuous windowed operator holding correlation state (watermarks, late-arrival handling). A pull-then-cache-then-query model expresses this only as *near-real-time repeated query over a short-TTL buffer* — weaker semantics. This is the one place the pull thesis does not stretch cleanly (see §6).

---

## 6. Streaming Detection — prior art and the fit to Prism

(Flink, Kafka Streams, ksqlDB, Dataflow, Zeek, Suricata, Splunk ES, Sentinel — deep-research.)

- **Continuous streaming operators (Flink / Kafka Streams / ksqlDB):** detection logic = standing computation over an unbounded stream; **tumbling / sliding(hopping) / session windows**; **event-time vs processing-time** with **watermarks** ("all events ≤ t have arrived") to close windows and handle late data; correlation state in **RocksDB state backend** (Flink) or **state stores + changelog topics** (Kafka Streams) for fault tolerance. Lowest latency; highest operational complexity (monitor consumer lag, query saturation).
- **Inline at capture time:** **Zeek** scripts maintain per-connection keyed state and timers (implicit windows) over the live packet stream — detection without first storing. **Suricata IPS** inspects a **sliding window** over reassembled TCP and can *drop* packets inline.
- **Near-real-time scheduled query (the SIEM compromise):** **Splunk ES** real-time/continuous correlation searches and **Microsoft Sentinel NRT** rules (run every ~1 minute over freshly ingested data). The research explicitly characterizes these as **"detection-as-query over a short-TTL buffer"** — *repeatedly query the last W minutes*, conceptually a polled windowed aggregation. **This is exactly Prism's detection-as-query-over-RetentionCache model (§3.3 / §14).**
- **The trade-off table (verified):** streaming operators = low latency + continuous in-memory/RocksDB state + explicit watermarks, but heavy ops + stream-engine expertise. Query-after-store / NRT = higher latency + state reconstructed from the buffer on each run + accessible SIEM query language, but cannot express arbitrary stateful patterns as cleanly.

**Fit verdict for the discussion:** Prism's "detection-as-query over a short-TTL RetentionCache"
is the **Splunk-real-time / Sentinel-NRT lineage**, not the Flink lineage. That is a legitimate,
simpler design — **but** if a collector ingests a high-rate stream and a detection needs tight
windowed correlation with late-arrival correctness, the NRT-over-cache model inherits the SIEM
limitation: it is a polled window, not a watermark-driven continuous operator. **Open question
whether Prism wants the continuous-operator capability at the edge (Zeek/Suricata/Flink-like) or
stays NRT-over-cache.**

---

## 7. Mapping to Prism's Existing Pillars

| Collector-class element (research) | Prism pillar it reuses (vision) | Fit / gap |
|------------------------------------|----------------------------------|-----------|
| **Receiver/listener endpoint** (§2 stage 1) | *New* — no existing pillar; closest is the Satellite as host | **GAP** — Satellite today is a remote *executor* (pull), not a *listener* (push-receive). Adding a receiver endpoint is the core new capability |
| **Local buffer / store-and-forward** (§2 stage 2) | **Satellite** store-and-forward (§3.2) + **RetentionCache** hot/cold (§3.3) | Strong reuse. Satellite already does store-and-forward for *results*; here it buffers *incoming push data* |
| **Boundary normalization** (§2 stage 3) | **Connector taxonomy** §3.4 (sensor→OCSF; connector→native schema-on-read) + multi-schema §13.6 | Strong reuse — syslog/flow→OCSF Network Activity/Authentication; non-mappable → native schema-on-read |
| **Queryable surface** (§2 stage 4) | **PrismQL** + `FROM cache.<name>` (§3.3 P2) + detection-as-query §14 | Strong reuse — collector buffer becomes a `FROM cache.<collector>` source |
| **pcap metadata/payload split** (§3.3) | RetentionCache for metadata; **no pillar for the raw-packet rolling buffer** | **GAP** — full-packet blob store is a distinct regime (Arkime model), not RocksDB-hot/Iceberg-cold |
| **Capability descriptor** (push has *no* pushdown) | Per-connector capability descriptor (§3.4 addendum / §10.3 ADOPT-1) | A collector's descriptor declares "push source, no pushdown, buffer-backed" — a new descriptor *class* |
| **Static vs dynamic schema** (§13) | Connector schema axis (§13.1) | Syslog/flow = mostly *static-ish* (known formats + vendor variance); webhook/Kafka payloads = *dynamic* (introspect + configure-schema) |
| **Streaming detection state** (§6) | Detection-as-query over RetentionCache (§14) is NRT-not-continuous | **GAP/decision** — continuous windowed operator is not currently a pillar |

---

## 8. OPEN DESIGN QUESTIONS (for human discussion — NOT decided here)

1. **Is "collector" a connector *subtype* with a receiver endpoint, or a separate component class?** The research says the abstraction is unified (§2) — but the *receiver endpoint* is genuinely new vs. Prism's pull connectors. Does it live in the connector taxonomy (§3.4/§13) or get its own pillar?
2. **Collection locus** — satellite-edge (a), standalone collector (b), or central-managed (c)? Or a declared-per-deployment hybrid? (§4 finds no universal winner.)
3. **What is the durability contract for push data Prism cannot ack?** UDP syslog/NetFlow/sFlow have *no backpressure and silent loss*. Does Prism (a) accept best-effort + record loss-metrics (`UdpRcvbufErrors`-style), (b) require TLS/RELP/TCP hot-path for critical sources, or (c) front everything with a local aggregator? What is the partial-result/loss semantics vs. BC-2.01.010?
4. **pcap: metadata-only, or metadata + on-demand packet retrieval (Arkime model)?** If the latter, the full-packet rolling buffer is a **distinct storage regime** outside RocksDB-hot/Iceberg-cold — is that in scope, and who owns its retention/residency?
5. **Streaming detection: NRT-over-cache (Splunk/Sentinel lineage) or continuous windowed operator (Flink/Zeek/Suricata lineage)?** (§6.) The former reuses §14 as-is; the latter is new correlation-state machinery (watermarks, event-time, late arrivals).
6. **Where does normalization run for push data — edge or central?** Edge normalization preserves residency and reduces transit (Cribl Edge pattern) but pushes compute to the satellite; central normalization is simpler but egresses raw.
7. **Buffer budget for high-rate streams.** A 200MB hot cap is seconds of busy syslog. Per-collector sized buffers? Mandatory edge reduction policy? Explicit backpressure/shedding policy? How does DC-004 server-sizing interact?
8. **Schema-on-read coverage for non-OCSF push data.** Which syslog/device/flow events have *no clean OCSF home* and become §13.6 native tables? Requires field-level verification against schema.ocsf.io (flagged INCONCLUSIVE in §3.2/§3.3).
9. **Kafka-as-source first?** Kafka is the *most federatable* push source (replayable, offset-addressable, lag-not-loss). Is consuming a Kafka topic the lowest-risk first collector to prototype the abstraction, before UDP syslog/pcap?
10. **Idempotency / dedup for webhook + at-least-once queues.** SQS/webhook retries deliver duplicates. Where does event-ID dedup live, and how does it interact with detection-window correlation?
11. **Capability-descriptor semantics for a no-pushdown push source.** A collector cannot push down predicates to the *source* — pushdown applies to the *buffer*. Does the descriptor model need a new "buffer-backed push source" class with its own join-guard implications (§12.2)?

---

## 9. What this would ADD to Prism's value-prop — and the honest cost

**Adds:**
- **Closes the "but we have syslog/NetFlow/pcap" objection** that pure-pull federation cannot answer — these are ubiquitous in real SOC/MSSP and have no queryable API. A federated engine that *also* federates the metadata of push/capture sources is materially more complete.
- **Reinforces, not dilutes, the neutral-incentive story.** Collecting-and-reducing-at-the-edge is the *same* Cribl-credible posture: a collector is a source/buffer, not a priced sink. "Federate or replace" extends to "federate the API, collect-and-federate the stream."
- **Reuses 3 of 4 abstraction stages** (Satellite buffer, OCSF/native normalization, PrismQL surface). The marginal new build is the receiver endpoint + push-loss semantics — not a new product.
- **OT/air-gap differentiation** — satellite-edge collection with dial-home + store-and-forward is exactly where central-cloud collectors are weakest (§4).

**Honest cost (do not minimize):**
- **A second storage regime for full-packet pcap** if on-demand retrieval is in scope — disk-bounded rolling buffer, distinct from RocksDB/Iceberg, with its own residency/retention governance.
- **Push-loss is real and un-fixable at the transport layer** for UDP sources. Prism must own an explicit best-effort-vs-reliable posture and surface loss metrics — silence here would be a SOUL.md #4 (no silent partial-failure) violation.
- **Buffer-budget pressure.** High-rate streams break a small hot cap; mandatory edge reduction or per-collector sizing is required, and that is operational complexity the pure-pull model never had.
- **Streaming-correlation-state is the one capability that does not stretch from the pull-then-cache model.** If tight windowed detection over the incoming stream is wanted, it is net-new machinery (watermarks, event-time, fault-tolerant state) — the most expensive single item in this whole space.
- **Operational surface area** — receiver endpoints are listeners (attack surface, auth, rate-limit, DoS), unlike outbound pull connectors. New security-review scope.

**Net:** the requester's working hypothesis ("collect at edge → OCSF-normalize → TTL'd buffer →
queryable source; pcap = metadata-federated + short-TTL packet blob") is **validated by the
prior art** (Cribl/Vector edge model; Arkime metadata/payload split; Cribl neutral incentive). It
is the right shape. The three honest strains — full-take pcap volume, high-rate buffer budget,
and streaming-correlation-state — are where the discussion's hardest decisions live.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 5 | (1) Edge-pipeline products Cribl/Vector/Fluent Bit/Fluentd/Logstash/Beats — architecture, buffering, neutral incentive; (2) Syslog deep — RFC 3164/5424/5425/6587 + RELP + UDP realities + OCSF mapping; (3) pcap deep — Arkime/Zeek/Suricata + capture economics + OCSF Network Activity 4001; (4) Collector class — NetFlow/IPFIX(RFC 7011)/sFlow/Kafka/webhook/S3-object-drop + push-vs-pull abstraction; (5) Streaming detection — Flink/Kafka Streams/ksqlDB/Dataflow + Zeek/Suricata inline + Splunk/Sentinel NRT + windows/watermarks |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (library-API docs not the subject; protocol/vendor research is Perplexity's strength) |
| Tavily tavily_search | 0 | — |
| Tavily tavily_research | 0 | — |
| Tavily tavily_extract | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~3 areas | (a) exact 10Gbps→TB/day pcap rate [model-knowledge, flagged]; (b) Sigma rule format genericity (research itself flagged this as community-knowledge); (c) generic webhook/idempotency framing where vendor docs were thin |

**Total MCP tool calls:** 5 (all `perplexity_research`, `sonar-deep-research`, `reasoning_effort: high` ×4 + `medium` ×1).
**Training-data reliance:** **low** — every non-obvious architectural, protocol, and vendor claim is grounded in retrieved deep-research output citing RFCs (rfc-editor.org RFC 3164/5424/5425/6587/7011/3954), vendor docs (cribl.io, vector.dev, fluentbit/fluentd, elastic.co, arkime.com, docs.zeek.org, docs.suricata.io/OISF, schema.ocsf.io, AWS S3/SQS/SNS, Confluent/Flink/ksqlDB, Splunk ES, Microsoft Sentinel). Three areas flagged `[model-knowledge]` / `[INCONCLUSIVE]` inline: exact pcap volume rate, Sigma framing, and several exact OCSF attribute paths (which MUST be field-verified against schema.ocsf.io before any downstream spec work).

**Deviation note (per agent mandate):** `perplexity_research` was used for all five passes (the
preferred PRIMARY tool) — no deviation to justify. Context7/Tavily were not used because the
subject is protocol/architecture/vendor-positioning research, not library-API documentation;
Perplexity deep-research with high reasoning effort is the correct instrument and produced
source-grounded, citation-backed output for each pass.
```
