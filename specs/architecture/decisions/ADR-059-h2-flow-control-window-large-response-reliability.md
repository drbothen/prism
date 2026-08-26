---
document_type: adr
adr_id: "ADR-059"
title: "H2 Flow-Control Window Sizing for Large-Response Sensor APIs"
status: ACCEPTED
date: "2026-08-26"
modified: "2026-08-26"
version: "1.0"
producer: architect
subsystems_affected: [SS-01, SS-16]
supersedes: []
superseded_by: null
amends: "ADR-050"
anchor_stories:
  - S-ENGINE-H2-LARGE-RESPONSE-001   # story to be created; §Authority will cite this ADR
related_adrs: [ADR-050]
related_bcs: [BC-2.16.002, BC-2.16.015]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-059: H2 Flow-Control Window Sizing for Large-Response Sensor APIs

## Status

ACCEPTED v1.0 (2026-08-26) — D7: h2 initial stream/connection window sizing and adaptive window
enabled on all production outbound sensor reqwest clients. Extends ADR-050 §D5 without
superseding any prior decision.

---

## Context

### Defect Evidence

Live monroe validation of S-CLAROTY-VULNS-001 (claroty `vulnerabilities` table, 2026-08-26)
revealed that fetching `claroty_vulnerabilities` (~1.1 MB/page) hangs with no bytes arriving
over 30 seconds. The query times out with E-QUERY-004. Concurrent observations:

- `claroty_alerts` (~5 KB/page): returns instantly on the same direct HTTPS connection.
- `claroty_vulnerabilities` (~1.1 MB/page): zero bytes received in 30s over direct HTTPS.
- Identical requests relayed via a Python `urllib` HTTP/1.1 proxy return full data (five 200
  OK responses, correct pagination, correct OCSF output).
- The live xDome API (`api.claroty.com`) returns HTTP 200; no HTTP 500 is involved.
- Field projection and TOML spec are confirmed valid per the xDome OpenAPI (spike findings).

ADR-050 §D5 (DEFECT-ADAPTER-TLS-XDOME-LIVE-001) added the `http2` reqwest feature to production
crates to enable h2 ALPN negotiation. That fix restored direct communication for small-response
tables. The current stall is a SECOND distinct defect: h2 flow-control window exhaustion on
large responses, not an h2-absence problem.

### Root Cause Hypothesis

reqwest 0.12.28 / h2 crate 0.4.18 (Cargo.lock) uses the RFC 7540 default initial window sizes:
65535 bytes (~64 KB) per stream and per connection. A 1.1 MB API page requires 17+ WINDOW_UPDATE
round trips to receive completely. If the h2 runtime delays generating WINDOW_UPDATE frames
(async executor scheduling, Nagle-analogue behavior, or a latent flow-control bug in h2 0.4.18),
the server stalls waiting for permission to transmit more DATA frames. The relay sidesteps this by
using HTTP/1.1, which has no application-level flow control window.

A second, independent explanation — WAF throttling of large h2 responses — is less likely because
the relay and direct path share the same TCP/IP path to `api.claroty.com`. Flow-control stall is
the simpler and sufficient explanation.

### Mechanism Selection

Three options were evaluated:

**Option A — Force HTTP/1.1 globally** (`.http1_only()`): Rejected. ADR-050 §Alt-E explicitly
rejected forcing h1 on all clients for the previous fix. Doing so here would (1) degrade
performance on h2-capable endpoints by eliminating h2 multiplexing, (2) potentially reintroduce
the WAF fingerprint issue for sensors behind h2-preferring cloud edges (AWS GA, Cloudflare), and
(3) not address the root cause (window sizing), leaving the defect class open for any future
HTTP/1.1 → h2 migration on another sensor backend.

**Option B — H2 window tuning** (`.http2_initial_stream_window_size`, `.http2_initial_connection_window_size`,
`.http2_adaptive_window`): ACCEPTED. Sets the initial flow-control window to 4 MB (4 194 304 bytes),
which covers the observed maximum page size (~1.1 MB) with a 3.7× safety margin. Enables the h2
crate's bandwidth-delay-product (BDP) adaptive window so the window expands further at runtime for
even larger future pages. This fix is surgical: it applies to the reqwest ClientBuilder and has
zero behavioral impact on sensors with small pages (the larger initial window just means fewer
WINDOW_UPDATE round trips, which is a performance improvement, not a regression).

**Option C — Per-sensor transport opt-in via TOML**: Rejected. H2 window sizing is a client-side
transport parameter, not a sensor-protocol concern. The defect arises from a client default that is
too small for ANY sensor returning large pages; fixing it globally eliminates the defect class for
all current and future sensors. Per-sensor config adds maintenance surface and means new sensors
inherit the broken default until explicitly configured.

### ADR-050 Compatibility

This decision extends ADR-050 D5 (h2 feature required) with concrete h2 ClientBuilder parameters.
It does NOT:
- Introduce `native-tls` (D1/D2 unchanged)
- Remove the `http2` feature (D5 unchanged)
- Remove `.user_agent(...)` (D6 unchanged)
- Create a per-sensor/per-spec mechanism (Option C rejected per §Rationale)

---

## Decision

**D7 — All production outbound sensor reqwest client builders MUST configure h2 initial window
sizes and enable adaptive window (amends ADR-050)**

Every production `reqwest::Client::builder()` chain that produces an outbound sensor/plugin HTTP
client — enumerated in ADR-050 §D6 — MUST also include:

```rust
.http2_initial_stream_window_size(4 * 1024 * 1024)      // 4 MiB stream window
.http2_initial_connection_window_size(4 * 1024 * 1024)  // 4 MiB connection window
.http2_adaptive_window(true)                             // BDP-based expansion at runtime
```

**Scope (same as ADR-050 §D6):**
- `build_http_client_with_custom_timeout` in `crates/prism-bin/src/spec_driven_adapter.rs`
  (canonical factory; `DeclarativeHttpAuthProvider` inherits via BC-2.16.014)
- Both `reqwest::Client::builder()` sites in `crates/prism-bin/src/boot.rs` for `PluginRuntime`
- `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs`
  (infusion `HttpLookupSource` outbound factory)

DTU test clients (wiremock/plain HTTP) are excluded — ALPN h2 negotiation does not apply over
plain HTTP, and DTU client construction must not be altered.

**Window size justification:**
- 64 KB (RFC default): forces WINDOW_UPDATE on every ~64KB of data; stall risk for MB-scale pages.
- 1 MB: marginal — still requires WINDOW_UPDATE on 1.1 MB responses.
- **4 MB (selected):** covers the observed 1.1 MB page with 3.7× margin and handles any page size
  up to 4 MB without any mid-response WINDOW_UPDATE overhead.
- 8 MB+: over-provisioning; h2 adaptive window covers any exceptional future cases.

**`http2_adaptive_window(true)` justification:** The h2 BDP algorithm increases the window
dynamically based on measured RTT and throughput. On a high-latency WAN path (e.g., `api.claroty.com`
behind AWS Global Accelerator), BDP will push the window above the 4 MB initial value
automatically, preventing future stalls on pages larger than 4 MB without requiring another ADR
amendment.

---

## Rationale

The h2 flow-control window stall is a known class of defect in HTTP/2 client implementations.
h2 crate 0.4.18 uses the spec minimum (65535 bytes) as its default, which is correct for protocol
compliance but operationally inadequate for APIs that return MB-scale pages in a single response
(which is common for vulnerability and device inventory APIs in the OT/ICS sensor space).

Increasing the initial window to 4 MB eliminates the stall for all sensor page sizes observed in
the prism corpus (max: ~1.1 MB for `claroty_vulnerabilities`). The adaptive window provides a
self-tuning safety net for sensors not yet onboarded.

**Confirmation experiment** (required for story RG): spin up a local hyper h2 server using
`.http2_prior_knowledge()` on plain TCP (avoids TLS/ALPN setup in tests while exercising h2
flow control at full fidelity), serve a 2 MB response body, assert the response arrives in < 5s
with the tuned client and either stalls or exceeds 5s with the default 64 KB window. This
provides a deterministic local proof of the fix without the live tenant.

---

## Consequences

### Positive
- `claroty_vulnerabilities` (and any future large-page sensor table) no longer stalls under direct
  h2 HTTPS — the root defect motivating S-CLAROTY-VULNS-001 is resolved.
- All current sensors with small pages (claroty_alerts ~5 KB, crowdstrike ~50 KB typical) are
  unaffected; larger initial window simply means fewer WINDOW_UPDATE frames — a performance
  improvement, not a behavioral change.
- The adaptive window prevents recurrence for future sensors with larger pages.

### Negative / Trade-offs
- Very slightly higher initial memory per h2 stream (4 MB send budget allocated by the remote
  server, not by prism). In practice, most servers will not fill the full window before prism
  sends WINDOW_UPDATE; the memory cost is the server's to manage.
- No known regression risk across the 4 existing Claroty tables (alerts ~5 KB, audit_logs ~20 KB,
  devices ~200 KB, vulnerabilities ~1.1 MB) or any other sensor.

---

## Alternatives Considered

**Alt-A: `.http1_only()` global** — Rejected. See §Mechanism Selection §Option A. ADR-050 §Alt-E
already rejected this for the D5 decision; the same reasoning applies here.

**Alt-B: Upgrade h2 crate** — h2 0.4.18 is the Cargo.lock-resolved transitive dep of reqwest
0.12.28. Upgrading would require bumping reqwest (Cargo.toml semver pin) or patching. The
ClientBuilder API fix (D7) addresses the symptom deterministically without requiring a dep upgrade
path. If h2 0.4.x has a known bug, an upgrade is a valid complement but not a substitute.

**Alt-C: Per-sensor window config** — Rejected. See §Mechanism Selection §Option C.

**Alt-D: Increase only stream window (not connection window)** — Rejected. A large stream window
with a small connection window (64 KB) would still stall when a single large page is the only
active stream, because the connection window would be exhausted. Both must be set to 4 MB.

---

## Source / Origin

DEFECT-1 (S-CLAROTY-VULNS-001 live monroe validation, 2026-08-26). Bisection evidence: Python
urllib HTTP/1.1 relay succeeds (5× 200 OK, full pagination); direct reqwest/h2 stalls for 30s
with zero bytes received. `claroty_alerts` (~5 KB, same credentials, same direct HTTPS) succeeds
instantly, confirming auth and routing are not the cause.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-08-26 | architect | Initial — D7 h2 window sizing decision, extends ADR-050. |
