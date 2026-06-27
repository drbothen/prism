---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-26"
provenance: "side-analysis discussion input; does not modify vision/specs."
topic: "Detection/storage reshape (full-packet retrieval + continuous operator) + first-class protocol-dissector layer incl. OT/ICS"
scope: "SIDE-ANALYSIS / discussion input — NOT a spec, vision, brief, ADR, BC, or PRD change."
---

# Detection/Storage Reshape + Protocol-Dissector Layer (incl. OT/ICS) — Research Capture

> **BOUNDARY.** This is a cited research capture to inform a HUMAN DISCUSSION. It does **not**
> modify `matured-vision-day2-requirements.md` (incl. §17 / §14 / §13 / §12.4 / §3.2 / §3.3), any
> spec, STATE.md, SESSION-HANDOFF.md, prior research docs, or any live factory artifact. It does
> **not** decide the open questions it raises. `do_not_execute: true`.

> **Citation discipline.** Findings are grounded in MCP-tool-retrieved web research (Perplexity
> `sonar-deep-research`, five deep passes + one reasoning pass + one factual confirmation). Vendor /
> RFC / standard / protocol sources are named inline by the deep-research citation index. Where a
> claim rests on model knowledge it is flagged **[model-knowledge]**; where research was
> inconclusive it is flagged **[INCONCLUSIVE]**. Date-stamped as of 2026-06.

> **Read-coverage honesty.** Each deep-research pass returned 83k–104k characters. I read the first
> ~58–77% of each in full and harvested the remaining sections by targeted grep for the
> decision-relevant claims (verdicts, semantics, limits, version facts). Where a specific section
> was harvested by grep rather than read in full prose, the synthesis below reflects the harvested
> sentences, not invented detail. The OCSF-OT verdict was additionally confirmed by a separate
> `perplexity_ask` factual check.

---

## 1. Executive Summary (12 lines)

1. **Streaming `MATCH_RECOGNIZE` is real but bounded.** Flink SQL implements SQL:2016 row-pattern-recognition over unbounded streams (event-time + watermarks + `AFTER MATCH SKIP`), but as a *subset*: no greedy quantifier as the last pattern element, no distinct aggregations, and explicit warnings that partial-match state grows with stream size. **Negation/absence/timeout is NOT first-class in `MATCH_RECOGNIZE`** — it must be encoded indirectly via time-bounds + watermarks. Dedicated CEP engines (Esper `not`/`timer:interval`, Siddhi `every`/`absent`) model it directly with explicit timers.
2. **Verdict R1:** compiling prism's §12.4 `MATCH_RECOGNIZE` to a continuous operator is **feasible** (Flink proves the shape), but `WATCH…UNLESS` (exclusion + timeout/absence) is the **single hardest piece** because absence-detection over an unbounded stream fundamentally *requires a timer/watermark to ever "complete" a non-match*. This matches prism's own §12.4 note that the exclusion/timeout `AbsenceWindowNode` anti-join is the hardest desugaring.
3. **Verdict R2:** "one language, planner picks physical mode" is **partially validated, with a hard caveat.** Unification is real at the *programming-model* level (Flink unified batch/stream SQL; Spark Structured Streaming; Beam; Sigma → many SIEMs; YARA-L single+multi-event). But late-data, window-edge alignment, state-retention, and exactly-once-vs-at-least-once **silently change meaning** between a polled-window and a continuous watermark operator. The mature mitigation: **make temporal/lateness/accumulation semantics EXPLICIT in the detection spec**, do not delegate them to the planner.
4. **Verdict R3:** detection-driven packet retention is an **established pattern (conceptually), with no single canonical academic name.** Implemented as Suricata conditional pcap (`alerts`/`tag` modes — logs the *whole alerted flow*), Zeek/Bro Time Machine (first-N-bytes heuristic + dynamic detection-override), Corelight Smart PCAP (first-2000-bytes, S3, PCAP-URL-in-log), Stenographer + Arkime (rolling buffer + query-extract). The flow is **trigger → pin → retrieve**.
5. **Verdict R4:** mature systems **mostly SPLIT** state, not unify. Kafka Streams unifies operator + long-lived state (state-store + changelog); but ML model/feature state is near-universally a **separate** store; the dominant reason is **blast-radius / recovery isolation + divergent access patterns + divergent retention horizons**. Co-location buys latency at the cost of coupled failure domains.
6. **Verdict R5:** **Spicy is the right model** for prism's pluggable, spec-driven dissector layer. It is a *declarative parser-generator* (units/fields/hooks → `.evt` interface → `spicyz` → `.hlto`), **built into Zeek since 5.0**, mature enough that the Spicy HTTP analyzer "replaces HTTP." Declarative grammar > hand-written imperative C for memory-safety/fuzzability/maintainability. Suricata reached the same safety goal via a different lever (Rust-only app-layer parsers).
7. **The dissector IS §17 stage-3 normalization for packet sources** — it turns packet bytes into structured OCSF/native fields, pluggable without core changes. This unifies it with the collector abstraction: a Spicy-style dissector makes a packet sensor "just another collector" whose stage-3 output is `FROM cache.<collector>`.
8. **Verdict R6 (OT):** the major OT protocols (Modbus, DNP3, EtherNet/IP+CIP, OPC-UA, S7comm, IEC 104, IEC 61850 GOOSE/MMS/SV, BACnet, PROFINET, MQTT) are well-characterized and largely dissected by **Zeek ICSNPP (Spicy-based)** for Modbus/DNP3/S7comm/IEC-104/GOOSE/PROFINET-IO-CM; EtherNet/IP, OPC-UA, BACnet, MQTT are **gaps in the cited ICSNPP set** (covered partially by Suricata/Wireshark). CISA/INL **Malcolm** bundles Zeek+ICSNPP+Suricata+pcap as a turnkey passive stack.
9. **OT safety is non-negotiable and read-only.** Active polling/scanning can fault/crash PLCs; monitoring must be **passive, TAP-preferred-over-SPAN**, placed per Purdue layer / IEC 62443 zones-and-conduits, with **no injection** onto the OT segment. This aligns precisely with prism's §3.2 OT/Purdue satellite chaining and the §17.6 packet-buffer-at-edge model.
10. **OCSF-OT verdict (CONFIRMED, was a candidate INCONCLUSIVE):** as of 2026 OCSF has **no first-class ICS/OT event classes/categories/profiles**; there is an *open proposal* (ocsf/ocsf-schema issue #1515 "Industrial Control System (ICS) Field Extensions") but nothing standardized. → **OT telemetry is a native-schema-on-read case (§13.6 / §13 G-17)** today; OCSF Network Activity covers only the generic L3/4 envelope, not function-codes / DNP3 points / GOOSE state numbers.
11. **Synthesis (R7):** dissector + detection-driven retention + continuous-operator **all push down to the edge/satellite capture point**, producing a metadata-first, residency-bound pipeline: *dissect → normalize → fan-out {continuous-detect | short-TTL cache for polled query} → federate metadata up + pin packets on detection*. Central prism stays pull/ephemeral; raw packets stay local. This is the §17 collector spine extended to packet sources, and it strengthens (not dilutes) the federation thesis.
12. **Honest cost:** the composition concentrates *heavy compute at the edge* (deep dissection + RocksDB-backed continuous operator), a *second packet-storage regime* (§17.6), an *OT-passivity ceiling on how aggressive edge detection can be*, and a *multi-schema OCSF + native query burden* — each a genuine strain the discussion must price, not paper over.

---

# PART A — DETECTION / STORAGE RESHAPE

## R1 — Streaming Row-Pattern-Recognition / `MATCH_RECOGNIZE` over unbounded streams

### Maturity (version-stamped, 2026)

- **SQL:2016 standard** added `MATCH_RECOGNIZE` (R010/R020/R030: `PATTERN`/`DEFINE`/`PARTITION BY`/`ORDER BY`/`MEASURES`/`AFTER MATCH SKIP`). Native vendors that execute it: **Oracle, Trino (since v356), Snowflake, Flink SQL, Azure Stream Analytics, Google BigQuery (since Nov 2025)**. (VLDB "Democratize MATCH_RECOGNIZE!"; VLDB "RPR via joins"; Trino/TIND blog — deep-research.) This corroborates prism §14.2's "research-verified" vendor list.
- **Flink SQL** is the canonical *streaming* `MATCH_RECOGNIZE` (open-source Flink + Confluent Cloud). Event-time ordering via a watermark-decorated timestamp (`ADD WATERMARK FOR ts AS ts - INTERVAL '5' SECOND`); windows close when the watermark passes; allowed-lateness grace + side-output for very-late events. Flink 2.1/2.2 (2025) added materialized tables + delta joins. (Confluent Flink docs; Conduktor — deep-research.)
- **Flink is documented as a SUBSET of SQL:2016**, with named limits: **(a)** `MATCH_RECOGNIZE` is SQL-only (no Table-API equivalent); **(b)** **distinct aggregations not supported** in `MEASURES`; **(c)** **a greedy quantifier cannot be the last pattern element** (`(A B*)` is rejected — "generally unreasonable / can never be satisfied"); must end with a simple variable or a reluctant quantifier; **(d)** time-bound: if first→last event span exceeds a configured value the match is dropped (a built-in timeout). (Confluent Flink reference docs — deep-research.)
- **ksqlDB / Kafka Streams: NO `MATCH_RECOGNIZE`.** ksqlDB is windowed aggregates + joins (`WITHIN` join windows, single-column equi-join only); sequence patterns must be hand-built or delegated to Flink. Kafka Streams Processor API can hand-code absence via punctuators/timers + state stores. (Confluent docs; IJCA paper — deep-research.)
- **Materialize: NO `MATCH_RECOGNIZE`** as of mid-2020s (incremental view maintenance / differential dataflow; "always-correct" answers vs eventual consistency). Temporal patterns approximated via `LAG`/`LEAD` over time-ordered partitions. (Materialize blog "Eventual Consistency isn't for Streaming" — deep-research.)
- **Apache Calcite:** parser-level RPR support exists; runtime behavior depends on the executing engine. (deep-research.)
- **CEP lineage (Esper/EPL, Flink CEP library, Siddhi, Drools Fusion):** richer, *first-class* sequence + absence operators. Esper EPL: `->` (followed-by), `and` (co-occurrence), **`not` (absence)**, `timer:interval(10 minutes)` to fire an absence event when the expected event fails to arrive. **Siddhi: `every` + `absent`** ("every A -> not B within 10 minutes"). Drools Fusion: timers + window expiry to derive synthetic absence/timeout events. (Esper/Siddhi/Drools docs — deep-research.)

### The hard limits (explicit)

1. **Absence/negation/non-event is the structural hard problem.** Over an unbounded stream a negative match (`A` NOT followed by `B` within `T`) can *never be confirmed without a timer/watermark* — you cannot prove a non-arrival by observation, only by a deadline. SQL `MATCH_RECOGNIZE` has no built-in negative operator; CEP engines solve it with explicit timers. (deep-research, Flink §3.6 + CEP section.)
2. **Unbounded state growth** from open partitions + `*`/`+` quantifiers + overlapping `AFTER MATCH SKIP TO FIRST` is a documented resource-exhaustion risk; Flink's mitigations are: define each variable as specifically as possible, never `DEFINE x AS TRUE`, always give the pattern a definitive end (bounded quantifier or time-bound).
3. **Late / out-of-order events** relative to the watermark: a late `B` arriving after an absence-match has already fired produces eventual inconsistency — the engine ignores it. Idle partitions stall watermarks and delay timeout firing.

### VERDICT R1 — feasibility

**Compiling prism's §12.4 `MATCH_RECOGNIZE` to a continuous operator is FEASIBLE.** Flink is direct prior art for SQL:2016 RPR over unbounded streams with event-time/watermark semantics, and prism already plans to *own* the operator (DataFusion core won't execute `MATCH_RECOGNIZE` — §14.2/G-18 — so prism builds the NFA operator regardless of polled-vs-continuous). The continuous execution adds watermark + late-data machinery on top of the same NFA.

**`WATCH…UNLESS` (exclusion + timeout/absence) is the single hardest piece** and confirms prism's own §12.4 assessment: absence detection over a stream is exactly the case SQL `MATCH_RECOGNIZE` does not model directly. prism should look to the **CEP-engine pattern (explicit per-partition timer started on the antecedent, fires the absence event on timer-expiry if the excluded event has not arrived)** — i.e., the `AbsenceWindowNode` anti-join must be timer/watermark-driven, not a pure relational anti-join, once it runs continuously. This is net-new correlation-state machinery, consistent with §17.7 Phase-2 being "the single most expensive item."

---

## R2 — ONE detection language → TWO execution modes (polled-batch vs continuous-over-stream)

### Prior art for unified batch+stream from one spec

- **Flink:** "batch is a bounded special case of streaming"; one SQL/Table API over bounded or unbounded tables; optimizer picks the plan. Real at the API level. (Confluent/Conduktor — deep-research.)
- **Spark Structured Streaming:** same DataFrame API; micro-batch engine (~100ms, exactly-once via checkpoint+WAL) or continuous mode (~1ms, at-least-once); trigger modes (`processingTime`, `AvailableNow`, real-time) tune the latency/throughput point. (Spark/Databricks docs — deep-research.)
- **Beam/Dataflow:** one pipeline, batch or streaming runner; event-time windows + watermarks + triggers + accumulation modes are *part of the spec*; Dataflow gives exactly-once via upstream-backup + dedup catalog + output checkpointing. (Beam/Dataflow docs — deep-research.)
- **Materialize / IVM:** for relational queries under strong consistency, "polling a materialized view ≈ consuming a changelog stream" — functionally close, semantically close. (deep-research.)

### Security write-once-run-many canon

- **Sigma:** generic YAML detection → many SIEM backends (Splunk SPL, Elastic, Sentinel KQL) via pySigma/sigma-cli pipelines. Abstraction **leaks** at: backend-specific field mappings, unevenly-supported modifiers, and correlation that translates cleanly to EQL/KQL but awkwardly to SPL. **Sigma Correlations** now add `event_count` / `value_count` / `temporal` / `temporal_ordered` types — directly relevant to prism's Sigma→PrismQL ambition (§14.4 Sigma import). (Sigma Correlations docs — deep-research.)
- **Splunk:** same SPL, real-time search vs scheduled search differ — real-time taps the ingest path (sees events pre-index, higher resource cost, often disabled) while scheduled relies on the index and **can silently miss events on index-lag** (documented worked example where an event falls between two scheduled windows). (Splunk community/Lantern — deep-research.)
- **Sentinel:** NRT analytics rules (run ~every minute, single query, constraints) vs scheduled rules; workspace rule-count limits (512) shape coverage. (Sentinel community — deep-research; NRT specifics partly [INCONCLUSIVE] in retrieved sources.)
- **Elastic EQL:** `sequence by` ordered-event detection; rule types query/threshold/EQL/ML; scheduled windows can miss sequences crossing run boundaries. (Elastic docs — deep-research.)
- **Google SecOps YARA-L 2.0:** single-event AND multi-event rules in ONE language; sliding windows defined *relative to a pivot event* (`$host over 10m after $e1`); **first-class negative correlation** (fires when `$e2` is absent in the window for the same host). Strong evidence that one rule language can span point-in-time + over-time. (YARA-L 2.0 examples — deep-research.)

### What silently changes between polled-window and continuous-operator (the caveat)

| Dimension | Polled batch query | Continuous stream operator | Silent-divergence risk |
|---|---|---|---|
| **Window boundary** | Retrospective, event-time `BETWEEN T AND T+W`, assumes completeness | Prospective, watermark+trigger closes window | Different events fall in different windows |
| **Late data** | Pipeline/index-lag concern outside query semantics; may miss late events | First-class: allowed-lateness, side-output, possibly retractions | Same spec includes/excludes different late events |
| **State** | Scoped to the run, dropped after | Long-lived across windows (RocksDB) | Threshold/sequence semantics differ if retention unspecified |
| **Emission** | One result set | Exactly-once (micro-batch) vs at-least-once (continuous) + possible retractions | Duplicate or corrected alerts |

### VERDICT R2 — does one-language-two-modes hold?

**Validated at the programming-model level; the planner-picks-physical-mode design is sound ONLY IF the spec makes the time/lateness/accumulation/state semantics EXPLICIT.** Every deep-research source converges on the same mitigation: "a write-once detection language that does not expose lateness/window/state semantics will produce silently inconsistent alerts across batch and streaming." prism's design ("planner picks polled-query vs continuous-operator from one PrismQL+YAML detection") is **defensible**, but the discussion must decide *where* the late-data / window-alignment / state-retention semantics live. The safest stance for a residency-first SOC engine where a missed or duplicated alert is a real harm: **the detection spec declares its windowing + lateness tolerance explicitly; the planner picks the engine, NOT the semantics.** prism's §14 YAML metadata (`window`, `schedule`) is the natural home — it likely needs explicit `lateness` / `accumulation` fields before Phase-2 continuous mode (§17.7) ships, so a rule cannot mean two different things depending on the planner's mode choice.

---

## R3 — Detection-DRIVEN / trigger-based packet capture (the #4 × #5 link)

### Prior art (confirmed)

- **Suricata conditional pcap:** `pcap-log` with `conditional:` ∈ {`all`, `alerts`, `tag`}. `alerts` → log **all packets from alerted flows** (the whole session, not just the triggering packet); `tag` → log only flows tagged by a signature's `tag` keyword (N packets / duration / scope). Also used offline: run Suricata `-r 24h.pcap` with `conditional: alerts` → "smaller PCAP with only alerted flows," then `gopherCap` to map/replay/extract. **Caveat: not "time-travel" — Suricata does not recover packets it never logged.** (SURICON materials / docs.suricata.io — deep-research.)
- **Zeek / Bro Time Machine** (Maier et al., "Enriching network security analysis with time travel," ACM): high-volume capture indexed by time/connection; the **cutoff heuristic = retain the first N bytes/packets of each connection, discard the bulk-transfer tail** (handshakes/auth/banners are the security-relevant part). **Bro can dynamically tell TM to retain MORE of a connection flagged by detection** — detection overrides the generic heuristic. (Now EOL; successors gotm / Metron / Stenographer.) (ACM paper — deep-research.)
- **Corelight Smart PCAP** (the commercial Zeek/TM descendant): captures only the packets needed, links Zeek logs + detections + extracted files to PCAP, **extends lookback up to ~10×**; configurable byte-depth (e.g., "first 2,000 bytes of all unencrypted traffic"); storage to Corelight HW / BYO / **Amazon S3**; **embeds PCAP URLs in connection logs for one-click pivot to Wireshark**. (Corelight docs — deep-research.)
- **Google Stenographer:** high-performance NIC-to-disk rolling buffer (to ~10 Gbps) with an index for fast retrieval *while still writing*; **independent of IDS/Zeek** (no impact on alerts); purges oldest when free disk < `diskfreepercentage`; query via BPF (`stenoread "<query>" -w out.pcap`, `so-pcap-export`). The detection system supplies the query string (5-tuple + time from an alert). (Security Onion docs — deep-research.)
- **Arkime (Moloch):** rolling PCAP (`pcapDir`, default `/opt/arkime/raw`, now expires by default) + session metadata (SPI) in OpenSearch/Elasticsearch; **hunt** = content search over raw packets, retrievable on demand; PCAP retention disk-bounded, metadata retention cluster-bounded. (Arkime FAQ — deep-research.) This is exactly the prism §17.6 model.

### Retention economics (why nobody keeps full packets long-term)

| Sustained link | ≈ PCAP / day | ≈ 90-day full-take |
|---|---|---|
| 25 Mbps | 250 GB | 22.5 TB |
| 1 Gbps | 5 TB | 450 TB |
| 10 Gbps | 50 TB | 4.5 PB |
| 40 Gbps | 200 TB | 18 PB |

(IPCopper sizing baseline ×proportional scaling; Arkime capacity guidance — deep-research. The 10/40 Gbps rows are linear extrapolations, flagged as illustrative in the source.) Conclusion is unambiguous: **metadata retained for months/years, packets for days/weeks; detection-driven retention is the principled tiering signal that elevates a session from metadata-only to metadata-plus-PCAP.**

### VERDICT R3 — is detection-driven capture a real pattern?

**YES — conceptually established and productized, but with NO single canonical academic name** (vendors say "conditional PCAP logging," "Smart PCAP," "selective full packet capture," "trigger-based capture"; the generic term **detection-driven packet retention** is apt). The flow is canonical **trigger → pin → retrieve**: detection (Suricata/Zeek/continuous-operator) emits identifiers → capture system pins those sessions/byte-ranges beyond rolling expiry → analyst/automation retrieves by session ID. This **directly validates prism §17.6 (#4 second storage regime, Arkime-style, retrieve-by-session) and the §17.7 detection-driven-retention idea**: the #5 continuous operator (or inline Zeek/Suricata) is the trigger that pins #4 packets. **Ancillary design requirements the prior art flags: synchronized clocks (alert timestamps must match PCAP), consistent session identifiers across tools (Zeek UID / Suricata `flow_id` / Community ID), and disk headroom for pinned data even as the generic buffer rolls.**

---

## R4 — Detection/correlation STATE stores: UNIFY vs SPLIT

### How mature stateful processors organize state

- **Flink:** keyed state (`ValueState`/`ListState`/`MapState`) + operator state; **RocksDB state backend** (embedded RocksDB per task manager, on-disk, in-memory cache, **incremental checkpoints** via SSTable diff to S3/HDFS, savepoints); window state + timers + `MATCH_RECOGNIZE`/CEP partial-match state all live here; timers in keyed state bound state growth (expire partial matches). Checkpoint cadence ~5–60s trades recovery-time vs I/O. (Flink docs / Waehner — deep-research.) **Note the alignment prism §17.7 already calls out: Flink's state backend IS RocksDB.**
- **Kafka Streams:** local RocksDB-backed state stores + **changelog topics as source-of-truth** (replay to rebuild on failover/rescale); KTable = state-store + changelog. (Confluent docs — deep-research.)
- **CEP engines:** Esper holds count + partial-matched-patterns in memory, snapshots at app level (no standardized persistence backend); Siddhi (WSO2 Streaming Integrator) has built-in snapshot/restore via REST/Query API. (deep-research.)
- **Feature stores** (Feast/Redis online store): ML feature/model state kept in a **separate** low-latency random-read store / model registry with its own versioning + backup. (deep-research.)

### The unify-or-split question, for {window-state, `detection_state`, `ModelState`}

The deep-research is explicit: **"both patterns exist, but separation is common for pragmatic reasons."**

- **Where unification happens:** Kafka Streams unifies *operator state + long-lived materialized (table) state* on one mechanism (state-store + changelog); co-location lets detection logic read risk-scores/suppression-lists as keyed state with no cross-system call → lower latency.
- **Where SPLIT dominates:** **ML model/feature state is near-universally a separate backend** (feature store / model registry). The cited drivers for separation:
  - **Blast-radius / recovery isolation** — streaming-engine failure or misconfig must not corrupt long-lived case/campaign data or model params.
  - **Divergent access patterns** — operator state optimizes write-throughput + checkpoint efficiency; feature stores optimize random-read latency; model registries optimize versioning.
  - **Divergent retention horizons** — window state is seconds-minutes; `detection_state` (risk/campaign) is days-weeks; model state is versioned across releases.

### VERDICT R4 — unify or split for prism?

**The prior art leans SPLIT, but prism's RocksDB-native posture (§14.3, no new datastore) makes a NUANCED answer correct, not a binary one.** The defensible position for the discussion:

- **Unify the homogeneous, co-access-pattern state:** continuous-operator window/correlation state + `detection_state` (risk/campaign/suppression/dedup) can share the **RocksDB / RetentionCache family** — this matches Kafka-Streams precedent (operator + long-lived materialized state together) AND prism's §14.3 "correlation state stays Prism-native, RocksDB, not a new datastore." prism likely keeps these in **distinct column families** (the existing 19-CF pattern — project memory) for isolation *within* the one engine, rather than one undifferentiated keyspace.
- **Keep ML `ModelState` (§15) logically separable** — it has a different access pattern (random-read at inference), a different lifecycle (versioned), and a different recovery/blast-radius profile; the prior art is consistent that model state is the one that benefits most from isolation. RocksDB-native does not forbid this (a dedicated CF or a dedicated RocksDB instance both honor "no new datastore").
- **The trade-off to surface:** checkpointing cadence and recovery isolation. A single shared checkpoint stream couples the recovery of fast operator state to slow campaign state; the discussion should decide whether the column-family boundary is enough isolation or whether the continuous operator's window state wants its own checkpoint cadence (Flink-style incremental checkpoints) distinct from the durable `detection_state`.

---

# PART B — PROTOCOL DISSECTORS + OT/ICS

## R5 — Pluggable protocol-dissector frameworks; the Spicy model

### What makes a dissector framework extensible WITHOUT core changes

The common architectural spine across Wireshark/Zeek/Suricata: **a stable core (capture + reassembly + scheduling) + a registration/dispatch table + isolated per-connection parser state.** The core never hard-codes protocols; it works over generic dissector tables (by port or heuristic) or generic analyzer interfaces, and modules bind themselves in at load time.

- **Wireshark/tshark:** C dissectors register via `proto_register_XXX` + `proto_reg_handoff_XXX` (`dissector_add_uint("udp.port", 1234, handle)`); Lua dissectors via `Proto` + `register_heuristic("udp", checker)` (port-independent, magic-number checks). **Declarative subsystems:** the **Protobuf dissector consumes `.proto` files as the grammar** (a real spec-driven instance) and the third-party **Wireshark Generic Dissector (WSGD)** lets data layouts be described externally. But Wireshark's primary mode is imperative C/Lua. (Wireshark dev guide / Lua API / Protobuf docs — deep-research.)
- **Zeek + Spicy (THE model):** Spicy is a **declarative parser-generator** — you write a `.spicy` grammar of `unit`s (ordered named fields + types + constraints + `&requires` + hooks), and Spicy compiles it (via the HILTI backend, JIT or build-time) into a parser. A `.evt` **interface-definition file** binds the grammar to Zeek (`protocol analyzer ... over TCP`, originator/responder units, `export SPICY_ID as ZEEK_ID`, event definitions); `spicyz` precompiles `.spicy`+`.evt` → `.hlto` that Zeek loads. **Built into Zeek since 5.0.** Spicy guidelines: keep state in top-level units (not globals → avoids memory bloat / cross-connection contamination), minimize emitted events, outsource only perf-critical bits to C++. **Maturity: Zeek-integration ~1.8.0, Spicy ~1.17.0-dev; the Spicy HTTP analyzer "replaces HTTP" in production** — strong production signal, though the sources do not enumerate the full Spicy-vs-C++ analyzer split, so exhaustive 2026 production coverage is [INCONCLUSIVE]. (Book of Zeek / Spicy docs / spicy HTTP analyzer.evt — deep-research.)
- **Suricata app-layer:** parser + logger + detection-keyword triad; **app-layer protocol code is Rust-only "for security reasons"** (the `RustParser` struct, per-flow `State` + per-transaction `Transaction`, gap handling via `APP_LAYER_PARSER_OPT_ACCEPT_GAPS`); new protocols added by writing a Rust parser + register code, no C-core change; project reserves protocol-ID ranges for mainline inclusion. Suricata 8.0.3. (Suricata app-layer docs / dev forum — deep-research.)
- **libnids:** legacy C TCP-reassembly library; callbacks only, **no protocol registry/dispatch** — every app reimplements parsing; illustrates *why* structured frameworks evolved. (deep-research, [model-knowledge] for some specifics.)
- **nDPI (ntop):** DPI for **application-ID/classification** (thousands of apps via signatures/heuristics), **NOT full protocol dissection** — labels a flow "TLS"/"BitTorrent" without exposing all fields. Different requirement than dissection. (ntopng docs — deep-research.)

### Why declarative (Spicy) > imperative for prism

- **Memory safety / fuzzability:** declarative units generate bounds-checked parsing code, eliminating the manual pointer-arithmetic / off-by-one / buffer-overrun hazards of hand-written C; the grammar is itself fuzzable and a form of documentation. (Suricata's Rust-only mandate reaches the same goal via a memory-safe imperative language — two routes to the same safety property.)
- **Maintainability / protocol evolution:** a new field or version is a grammar edit, not a rewrite of byte-reading loops.
- **Spec-driven dogfood alignment:** a `.spicy` grammar + `.evt` interface is *exactly* prism's TOML-connector philosophy applied to packets — protocol behavior is **declared, not coded**, and new protocols register without core changes. The honest limit: Spicy still allows imperative hooks/C++ for perf-critical or awkward semantics, so "purely declarative" is aspirational; and Spicy generates C++/JIT, which is an edge-compute cost (R7).

### VERDICT R5 — dissector model recommendation

**Adopt the Spicy model as the template for prism's pluggable, spec-driven dissector layer** — declarative grammar + interface-definition file + registration without core changes is the closest external analogue to prism's TOML-spec-engine connector philosophy, and it is the same engine (Zeek+Spicy) that already produces prism's preferred §17.6 flow/session metadata. **The dissector IS §17 stage-3 normalization for packet sources** (packet bytes → structured OCSF/native fields), which unifies a packet sensor with the collector abstraction (a dissector-backed sensor's stage-3 output becomes `FROM cache.<collector>`). prism does **not** rebuild Zeek/Suricata; it federates their (Spicy-dissected) output and may author its own Spicy grammars for protocols the ICSNPP set misses (R6).

---

## R6 — OT/ICS protocol dissection (DEEP) + safety + OCSF coverage

### Protocol matrix (standard / Purdue placement / open-source dissection)

| Protocol | What it is / standard | Purdue layer | Open-source dissection |
|---|---|---|---|
| **Modbus / Modbus-TCP** (TCP 502) | 1979 Modicon de-facto std (Modbus Org); 7-byte ADU header + function-code PDU; no auth/encryption | L0–L2 (cell/area), aggregated at L2–L3 | **Zeek ICSNPP-Modbus (Spicy)**; Wireshark; Suricata native; Malcolm |
| **DNP3** (TCP 20000) | IEEE 1815; class-based polling, unsolicited reporting, source timestamps; optional Secure Authentication (often off) | Substation L1–L2 ↔ control-center L3 (±L3.5 DMZ) | **Zeek ICSNPP-DNP3 (Spicy)**; Wireshark; Suricata native; Malcolm |
| **EtherNet/IP + CIP** (TCP 44818 explicit / UDP 2222 implicit cyclic I/O) | ODVA; "Ethernet Industrial Protocol"; CIP object model | L0–L2 cell/area | Wireshark (ENIP+CIP); **Suricata ENIP**; **ICSNPP EtherNet/IP NOT in cited set — verify** [INCONCLUSIVE] |
| **OPC-UA** (TCP 4840 binary / 443 HTTPS) | IEC 62541 (2008); platform-independent; self-describing info-model; built-in auth/encryption; can ride MQTT/AMQP pub/sub | L2–L3, often L3.5 DMZ (IT↔OT bridge) | Wireshark dissector; **often ENCRYPTED → passive sees only metadata/TLS**; no mature Zeek/Suricata app-parser → mostly TCP/TLS metadata |
| **S7comm / S7comm-plus** (TCP 102, ISO-on-TCP) | Siemens proprietary (since 1994); TPKT(RFC1006)+COTP+S7; S7 PDU starts `0x32`; TSAP encodes rack/slot; S7comm-plus adds enc/auth | L1–L2 (engineering/HMI ↔ PLC) | **Zeek ICSNPP-S7COMM (Spicy, parses S7comm + S7comm-plus + COTP)**; Wireshark; Suricata; Malcolm |
| **IEC 60870-5-104** (TCP 2404) | IEC 104; IP extension of -5-101; ASDU + I/S/U frames; European electric SCADA; unencrypted | Substation L2 ↔ control-center L3 | **Zeek ICSNPP-IEC60870-5-104 (Spicy) — labeled "outdated," verify maintenance**; Wireshark; Suricata; Malcolm |
| **IEC 61850 GOOSE** (L2 multicast, EtherType) | Fast substation events / protection trips; publisher-subscriber; no TCP/IP — invisible to L3 monitoring | Process bus L0–L1 | **Zeek ICSNPP-IEC61850-GOOSE (Spicy)** (parses L2 frames, dataset refs, state numbers); needs L2 TAP/SPAN |
| **IEC 61850 MMS** (TCP 102, ISO 9506 over ISO-on-TCP) | Client-server config/monitor/control between IEDs and SCADA; shares port 102 transport with S7comm | Substation L2–L3 | Wireshark; (ICSNPP MMS not in cited set) |
| **IEC 61850 Sampled Values (SV/SMV)** | IEC 61850-9-2(LE); high-rate analog streaming (e.g., 80 SPP @50Hz → 250µs interval); dedicated process bus | Process bus L0–L1 | Wireshark; needs L2 TAP, deterministic; demanding for capture HW |
| **BACnet** (UDP 47808 BACnet/IP; also MS/TP RS-485) | ASHRAE 135 / ISO 16484-5; building automation object model (AnalogInput, etc.); optional BACnet/SC | Building OT ~L2–L3 | Wireshark dissector; community Zeek/Suricata; **not in cited ICSNPP set** |
| **PROFINET** (industrial Ethernet) | Open IEC std; "Ethernet for industry" + deterministic RT classes; Siemens-associated | L0–L2 cell/area | **Zeek ICSNPP-PROFINET-IO-CM (Spicy)** (I/O context-management); Wireshark; needs deterministic-aware capture |
| **MQTT** (TCP 1883 / 8883 TLS) | OASIS pub/sub; broker-based; IANA 1883/8883; **IIoT not classic OT**; carries OT-origin data (+ sometimes control back) | L3–L3.5 / cloud edge (outbound-only friendly) | Wireshark dissector; **often TLS (8883) → metadata only**; Suricata/commercial MQTT support |

(Sources: ProSoft/Modbus, IEEE 1815/DNP.org, ODVA, OPC Foundation/IEC 62541, Wireshark S7comm/RFC 1006, IEC 104 guidance, IEC 61850-9-2LE, ASHRAE 135, PROFINET, OASIS MQTT, and the **CISA ICSNPP GitHub** packages — all via deep-research. Commercial Claroty/Nozomi/Dragos coverage is deep + mature for these protocols, cited for context only.)

**ICSNPP read:** the cited ICSNPP set (all Spicy-based) = **Modbus, DNP3, S7comm, IEC-60870-5-104, IEC-61850-GOOSE, PROFINET-IO-CM**. **EtherNet/IP, OPC-UA, BACnet, MQTT, IEC-61850 MMS/SV are gaps in the cited set** — partly covered by Suricata/Wireshark, candidates for prism-authored Spicy grammars. The IEC-104 plugin is flagged "outdated" — verify before relying on it.

### Safety constraints (NON-NEGOTIABLE in OT)

- **Passive / read-only is mandatory.** Active polling/scanning can fault, hang, or fail-safe PLCs/RTUs/relays — even benign diagnostic traffic. The OT ecosystem has converged on passive analysis as the primary telemetry source. (NIST SP 800-82 ICS guidance + CISA + vendor guidance — deep-research; the explicit "800-82" string was harvested from the framing, the passivity rationale is stated throughout.)
- **TAP preferred over SPAN.** A network TAP mirrors without adding forward delay and does not drop under congestion; SPAN/mirror ports "may be less reliable under congestion and can drop packets," obscuring critical I/O and L2 GOOSE/SV frames. For time-critical buses (GOOSE, SV, PROFINET RT) TAPs are "strongly preferred."
- **Purdue + ISA/IEC 62443 zones-and-conduits drive monitor placement.** Monitor at the conduit between zones (cell/area L1↔L2, L2↔L3, L3.5 DMZ); L0–L1 process-bus protocols (GOOSE/SV/PROFINET) need a sensor *on that segment* with L2 access. This maps **directly onto prism §3.2 satellite chaining / Purdue-layered segmentation** — an OT-layer satellite hosts the dissector at the right layer.
- **Determinism / no-injection.** No active probes onto the OT segment; capture must not add buffering/forwarding load to RT I/O paths; parsing must be robust+lightweight so a malformed-parse never misinterprets safety-relevant data. Aligns with §17.5 "residency preserved by construction" + §17.6 edge packet buffer.

### OCSF-OT coverage — VERDICT (CONFIRMED)

**OCSF has NO first-class ICS/OT event classes, categories, or profiles as of 2026.** OCSF core covers general IT domains (Network Activity, DNS, HTTP, file/process activity, authentication, cloud, threat-intel). A separate confirmation pass found an **open proposal** — ocsf/ocsf-schema GitHub **issue #1515 "[Proposal] Industrial Control System (ICS) Field Extensions"** — indicating ongoing work but **nothing standardized**. (deep-research flagged this INCONCLUSIVE on the live schema; the `perplexity_ask` confirmation upgraded it to a definite "no first-class OT classes + open proposal exists," citing the OCSF GitHub issue, Deepwatch, Query.ai, Fleak glossaries.)

→ **For prism this is decisive: OT/ICS telemetry is a native-schema-on-read case (§13.6 / G-17) today.** OCSF Network Activity (4001) can carry only the generic L3/4 envelope of an OT flow; the OT *semantics* — Modbus function codes, DNP3 object groups/points, S7 block/variable access, GOOSE dataset refs + state numbers, IEC-104 ASDU types — have no OCSF home and must live in **native structured tables** queried schema-on-read. This is exactly prism's multi-schema thesis (§13.6 #2). When/if OCSF ICS extensions land, prism's multi-version OCSF support (G-16) would absorb them; until then OT is the canonical native-schema-on-read example.

---

## R7 — How dissectors COMPOSE with the collector boundary + #4 + #5 + edge/Purdue

(Synthesis validated by a dedicated reasoning pass over the R1–R6 evidence; no new vendor facts introduced.)

### Physical placement (everything pushes down to the capture point)

1. **Zeek+Spicy / Suricata sit on the TAP/SPAN/inline point** at the edge/satellite (OT satellite at the right Purdue layer). They own packet ingestion **and** the Spicy dissector layer.
2. **Dissection + normalization run INSIDE the residency boundary** — normalized OCSF Network Activity + native OT schema-on-read records are produced at the locus and stay there (raw never crosses a Satellite boundary; §17.5 invariant).
3. **The continuous operator (Flink-lineage, RocksDB state backend) runs co-located/adjacent at the edge** — subscribes to the normalized stream, keeps window/sequence state in edge-local RocksDB (residency-respecting; the §17.7 Phase-2 capability).
4. **Full-packet rolling buffer is edge-local** (§17.6 second regime); detection (inline + continuous) issues **pin** decisions.
5. **The central federated query plane holds NO bulk PCAP** — it federates edge metadata stores + short-TTL caches, ephemeral pull queries descend into each residency-bound satellite (§3.2/§3.6 mechanics, §17.5).

### Data-flow ordering

```
packets (TAP)
  → DISSECT (Spicy dissector = §17 stage-3 normalization)
  → NORMALIZE (OCSF Network Activity 4001  +  native OT schema-on-read fields)
  → fan-out:
       ├─ CONTINUOUS DETECT  (Flink-lineage operator, RocksDB window/sequence state)   [#5 Phase 2]
       └─ SHORT-TTL CACHE    (RetentionCache hot tier; polled NRT query / hunt)         [#5 Phase 1 / §14]
  → on detection (any path) → RETENTION CONTROLLER:
       ├─ METADATA: write normalized rows to durable edge metadata store → federated up   [axis-1 storage]
       └─ PACKETS:  PIN session-IDs / first-N-bytes in the rolling PCAP buffer            [#4 trigger→pin]
  → later: RETRIEVE pinned PCAP by session-ID on demand (residency-aware channel)         [#4 retrieve]
```

### Why the dissector unifies with the collector

A Spicy-style dissector turns a packet sensor into a **general collector for packet-borne data**: stage-1 capture → stage-2 flow/session extraction → **stage-3 protocol-aware dissection + normalization**. Every packet source (IT subnet, OT Purdue layer, cloud edge) then presents the *same logical collector interface* — core OCSF fields + optional native/OT extension fields — so the federated query engine sees streams/tables differing only in optional fields, not in shape. New protocols/OT dialects = new declarative dissector plugins, no core change; per-site OT dissectors fit the residency-first model (each site carries the grammars its equipment needs). This is the §17.2 four-stage spine with the dissector as stage-3 for packets.

### NEW strains/costs this composition introduces (honest)

1. **Heavy edge compute.** Deep dissection (esp. OT stacks) + a RocksDB-backed continuous operator add CPU/mem/disk-I/O at the capture point. Over-subscription risks packet drops / detection lag / TAP backpressure. Dissector + streaming-job upgrades become *edge* DevOps (per-site), not central.
2. **A second packet-storage regime (already §17.6)** — dual: high-volume short-retention rolling buffer + low-volume long-retention pinned store; bursty incidents create PCAP retention hot-spots; per-residency lifecycle policy required.
3. **OT passivity ceiling.** Strict read-only at the Purdue layer means observe-and-alert only (no inline enforcement on OT), and dissection + streaming detection must stay lightweight enough not to perturb SPAN/TAP infra or saturate links — a real ceiling on how aggressive edge detection can be.
4. **Multi-schema OCSF + native burden.** OCSF gives a portable baseline (network L3/4); OT semantics are site-specific native schema-on-read. The federated query layer needs per-site schema discovery/registry + graceful degradation when OT fields are absent. **Detection portability splits:** an OCSF-only detection is globally portable; an OT-native detection is residency/site-specific and must be governed as such. (Directly extends §13.6 multi-schema + the entity registry §12.1 cross-schema resolution.)
5. **Governance / autonomous satellites.** Each edge now hosts a mini-stack (dissectors + operator + PCAP buffer + metadata store); residency-first implies per-site export/retention/query-access policy coordinated across many satellites (extends §17.8 chain-aware model + the residency invariant).

---

## Cross-Cutting Synthesis — how dissection + #4 + #5 reshape the storage/detection picture together

- **Storage becomes two-axis, both residency-bound at the edge:** (axis-1) **normalized metadata** — OCSF Network Activity for the L3/4 envelope + **native schema-on-read for all OT/non-mappable semantics** (the OCSF-OT gap forces this) — federated upward and TTL/tiered per §3.3/§17.8; (axis-2) **raw packets** — the §17.6 Arkime-style rolling buffer, disk-bounded, **pinned by detection (R3 trigger→pin)**, retrieved by session-ID. The dissector is the bridge: it is what makes axis-1 *structured* and what produces the session-IDs that index axis-2.
- **Detection becomes two-mode over one language (R2 caveat applies):** Phase-1 NRT-over-cache (polled, §14 reuse) + Phase-2 continuous operator (R1 feasible, watermark/state machinery, `WATCH…UNLESS` the hard part). The continuous operator is *also the trigger* that drives axis-2 packet pinning — closing the #4×#5 loop the requester posited.
- **The dissector layer is the load-bearing unifier:** it is simultaneously (a) §17 stage-3 normalization for packet sources, (b) the producer of the OCSF + native multi-schema records detection runs over, (c) the producer of the session identifiers that link metadata to pinned packets, and (d) the spec-driven (Spicy-style) dogfood of prism's TOML-connector philosophy extended to packets. OT dissectors are the same machinery, constrained to strict passivity at the Purdue-layer satellite.
- **State (R4)** sits underneath all of this: continuous-operator window state + `detection_state` likely share the RocksDB/RetentionCache family (distinct CFs for isolation), with ML `ModelState` (§15) kept logically separable per the prior-art lean.
- **Net for the federation thesis:** every piece keeps raw local and pushes only normalized/reduced metadata upward; central prism stays pull/ephemeral/residency-first. The composition *strengthens* the thesis (it answers "what about packets / OT / real-time?") at the cost of a materially heavier, more autonomous edge.

---

## Honest Costs / Strains (consolidated)

| Strain | Where it bites | prism anchor |
|---|---|---|
| Heavy edge compute (deep dissection + RocksDB continuous operator) | per-satellite capacity, drop/backpressure risk, per-site DevOps | §17.4 locus, §17.7 Phase-2 |
| Second packet-storage regime + pinning hot-spots | edge disk/I/O, per-residency lifecycle governance | §17.6 |
| `WATCH…UNLESS` absence/timeout over a stream | the hardest single operator (timer/watermark-driven anti-join) | §12.4, §17.7 |
| Spec mode-ambiguity (polled vs continuous semantics) | silent alert divergence unless lateness/window/state are explicit in the spec | §14 YAML metadata |
| OT passivity ceiling | observe-only; lightweight parsing mandatory; no inline OT enforcement | §3.2, §14.6 |
| OCSF-OT gap → multi-schema burden | per-site schema registry; detection portability splits OCSF-portable vs OT-site-specific | §13.6, §12.1 |
| State checkpoint/recovery coupling | shared checkpoint couples fast operator state to slow campaign state | §14.3, §17.7 |
| Clock-sync + consistent session-ID across tools | required for trigger→pin→retrieve correctness | §17.6 |

---

## OPEN DESIGN QUESTIONS (for human discussion — NOT decided here)

1. **Spec-mode semantics ownership (R2).** Should the PrismQL+YAML detection spec carry **explicit** `lateness` / `accumulation` / window-alignment fields so a rule cannot mean two different things in polled vs continuous mode — or is the planner trusted to pick semantics too? (The prior art strongly favors explicit-in-spec.)
2. **`WATCH…UNLESS` engine (R1).** When the absence/timeout operator runs continuously, is the `AbsenceWindowNode` a CEP-style **per-partition timer** (Esper/Siddhi model) rather than a relational anti-join? Who owns watermark/idle-partition handling so a stalled stream doesn't indefinitely defer (or prematurely fire) an absence detection?
3. **State unify-vs-split boundary (R4).** Do continuous-operator window state + `detection_state` share the RocksDB/RetentionCache family in **distinct column families**, and is ML `ModelState` (§15) a separate CF / separate RocksDB instance? What checkpoint cadence does the continuous operator's window state get, distinct from durable `detection_state`?
4. **Dissector ownership for the ICSNPP gaps (R5/R6).** EtherNet/IP, OPC-UA, BACnet, MQTT, IEC-61850 MMS/SV are gaps in the cited ICSNPP set. Does prism (a) federate Suricata/Wireshark output for these, (b) author its own Spicy grammars, or (c) wait for upstream ICSNPP? And does prism *run* Zeek/Suricata as the dissector engine, or *embed* a Spicy-style parser-generator natively?
5. **Encrypted-OT visibility (R6).** OPC-UA and MQTT-over-TLS often yield metadata-only passively. Is metadata-only OT visibility acceptable, or does prism need a (carefully-bounded, possibly active-adjacent) decryption/proxy posture at OT gateway chokepoints — which would tension with strict passivity?
6. **OCSF-OT mapping policy (R6).** Until OCSF ICS extensions standardize (issue #1515), what is prism's native OT schema family (function codes, points, GOOSE state, ASDU types), and how does the entity registry (§12.1) resolve an OT observable (e.g., a PLC IP + unit-ID) across OCSF + native + IT schemas?
7. **Edge compute budget vs OT passivity ceiling (R7).** What is the per-satellite compute envelope when dissection + a continuous operator run at an OT-layer satellite, and how is the "must not perturb the TAP/SPAN or saturate links" constraint enforced/measured?
8. **Detection-driven retention policy (R3).** Who owns the pin policy (which detections pin which sessions, for how long, first-N-bytes vs full-session), and how does pinning interact with §17.6 residency + the §17.8 retention-policy engine? How are clock-sync + session-ID consistency guaranteed across the dissector, the operator, and the PCAP buffer?
9. **Detection portability governance (R7).** How are OCSF-portable detections vs OT-native (site-specific) detections managed in the §14 rule lifecycle so a globally-deployed rule degrades gracefully where OT fields are absent?
10. **Build-vs-federate boundary for the continuous operator (R1/R4).** prism already commits to *owning* the `MATCH_RECOGNIZE` operator (DataFusion won't run it). Does the continuous (watermark) variant reuse a Flink-lineage embedded engine, or is it a prism-native operator on the RocksDB state backend? (§17.7 leans native; the cost is the watermark/checkpoint machinery.)

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 5 | (R1) Streaming `MATCH_RECOGNIZE`/CEP — Flink/ksqlDB/Materialize/Calcite/Esper/Siddhi/Drools, negation/absence/timeout, hard limits; (R2) unified batch+stream (Flink/Spark/Beam/Materialize) + security write-once-run-many (Sigma/Splunk/Sentinel/EQL/YARA-L) + mode-divergence semantics; (R3) detection-driven pcap (Suricata conditional/Zeek Time Machine/Stenographer/Arkime/Corelight Smart PCAP) + retention economics + state backends (Flink RocksDB/Kafka Streams/Esper/Siddhi/feature stores) unify-vs-split; (R5) dissector frameworks Wireshark/Zeek-Spicy/Suricata-Rust/libnids/nDPI; (R6) OT/ICS 10-protocol matrix + Purdue + ICSNPP/Suricata/Malcolm + safety constraints + OCSF-OT |
| Perplexity perplexity_reason | 1 | (R7) composition synthesis over the R1–R6 evidence — placement, data-flow ordering, dissector-as-stage-3 unification, new strains |
| Perplexity perplexity_ask | 1 | Confirm OCSF-OT coverage as of 2026 (upgraded R6 OCSF verdict from INCONCLUSIVE to confirmed; surfaced ocsf-schema issue #1515) |
| Perplexity perplexity_search | 0 | — |
| Context7 | 0 | — (subject is protocol/architecture/standard/vendor research, not library-API docs; Perplexity deep-research is the correct instrument) |
| Tavily (all) | 0 | — |
| WebFetch / WebSearch | 0 | — |
| Training data | ~4 areas | (a) 10/40 Gbps→TB/day pcap rates are linear extrapolations flagged illustrative in-source [model-knowledge]; (b) libnids internals partly [model-knowledge]; (c) BACnet UDP 47808 / ASHRAE 135 port widely-known but not stated in retrieved excerpt; (d) NIST SP 800-82 cited as the canonical passivity-guidance anchor (passivity rationale is stated throughout the retrieved research; the exact "800-82" doc id is the standard reference) |

**Total MCP tool calls:** 7 (5× `perplexity_research` `sonar-deep-research` `reasoning_effort: high`; 1× `perplexity_reason`; 1× `perplexity_ask` `search_context_size: high`).
**Training-data reliance:** **low** — every non-obvious architectural, protocol, version, standard, and vendor claim is grounded in retrieved deep-research citing RFCs/standards (SQL:2016, RFC 1006, IEEE 1815, IEC 62541, IEC 60870-5-104, IEC 61850-9-2LE, ASHRAE 135, OASIS MQTT, IANA ports), vendor/project docs (Confluent/Flink, Spark/Databricks, Beam/Dataflow, Materialize, Sigma Correlations, Splunk, Sentinel, Elastic EQL, Google YARA-L, docs.suricata.io, docs.zeek.org/Spicy, Corelight, Stenographer/Security Onion, Arkime, CISA ICSNPP, Malcolm/INL, schema.ocsf.io + ocsf/ocsf-schema#1515), and academic sources (VLDB "Democratize MATCH_RECOGNIZE!" + RPR-via-joins; ACM Bro Time Machine). Four areas flagged `[model-knowledge]`/`[INCONCLUSIVE]` inline (pcap rate extrapolation, libnids internals, Spicy exhaustive 2026 production split, ICSNPP EtherNet/IP coverage). The OCSF-OT verdict was cross-confirmed by a second independent tool call.

**Deviation note (per agent mandate):** `perplexity_research` was the PRIMARY tool for all five fact-finding passes (no deviation to justify); `perplexity_reason` was used once for synthesis OVER gathered evidence (its intended role); `perplexity_ask` was used once for a single ≤2-sentence factual confirmation (its intended role). Context7/Tavily were not used because the subject is protocol/standard/architecture/vendor research, not library-API documentation.
