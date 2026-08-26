---
document_type: adr
adr_id: "ADR-059"
title: "H2 Flow-Control Window Sizing for Large-Response Sensor APIs"
status: WITHDRAWN
date: "2026-08-26"
modified: "2026-08-26"
version: "1.2"
producer: architect
subsystems_affected: [SS-01, SS-16]
supersedes: []
superseded_by: null
withdrawn_reason: "D7 h2 flow-control window hypothesis falsified by live wire evidence (2026-08-26). reqwest/hyper 1.9 defaults (2 MiB stream + 5 MiB connection) exceed the observed ~1.2 MB page size — the initial-window hypothesis was numerically impossible. Direct h2 transport to api.claroty.com confirmed healthy across single-page and multi-page fetches. No client transport change adopted."
amends: null
anchor_stories:
  - S-ENGINE-H2-LARGE-RESPONSE-001   # re-scoped to canary regression + diagnostics; no longer cites D7
related_adrs: [ADR-050]
related_bcs: [BC-2.16.002, BC-2.16.015]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-059: H2 Flow-Control Window Sizing for Large-Response Sensor APIs

## Status

WITHDRAWN v1.2 (2026-08-26) — D7 is WITHDRAWN. The h2 flow-control window hypothesis was
falsified by live wire evidence. No client transport change is adopted. The direct negotiated-h2
transport to api.claroty.com is confirmed healthy and ships as-is. See §Withdrawal Rationale below.

Prior accepted versions (historical record only — not active mandates):
- v1.1 accepted D7 (fixed 4 MiB stream + connection windows, `http2_adaptive_window` omitted).
- v1.0 initial D7 h2 window sizing decision.

---

## Withdrawal Rationale

### Evidence That Falsified the D7 Hypothesis

**reqwest/hyper 1.9 defaults exceed the observed page size.** The §Context hypothesis assumed
reqwest 0.12 / h2 0.4 used the RFC 7540 default initial window sizes (65,535 bytes). Live
investigation confirmed that reqwest 0.12.28 with the `http2` feature resolves through hyper 1.x,
whose h2 layer uses 2 MiB stream window and 5 MiB connection window by default — both exceeding
the ~1.2 MB `claroty_vulnerabilities` page. The numeric precondition for window exhaustion did not
hold. The initial-window hypothesis was arithmetically impossible against the actual defaults.

**Direct h2 confirmed healthy in multiple reproduction paths:**
- `curl --http2` against POST `api.claroty.com/api/v1/vulnerabilities/` (~1.2 MB/page): HTTP 200,
  full body received, ~0.9s — faster than HTTP/1.1 for the same payload.
- Byte-faithful reqwest 0.12.28 reproduction of the production client builder (no window
  overrides): succeeded with clean END_STREAM; no stall observed.
- 6-page / ~7 MB multi-page fetch: drained correctly with observed WINDOW_UPDATE replenishment;
  the connection-window hypothesis was also falsified.
- Pre-fix prism binary (BEFORE any D7 changes): fetched live CVE rows across 10 paginated pages
  in ~17s with zero E-QUERY-004 / timeout errors when run directly on 2026-08-26.

**Diagnosis: transient network/edge condition.** The original 30s stall observation is most
consistent with a transient condition at `api.claroty.com`'s network edge at the time of the
live monroe validation, since resolved. A client-code root cause is ruled out: the same unmodified
binary succeeds in direct reproduction.

### Decision on Withdrawal

- **D7 is NOT adopted.** Do not add `http2_initial_stream_window_size` or
  `http2_initial_connection_window_size` overrides to production client builders. The defaults are
  sufficient for all observed sensor page sizes.
- **Do NOT force HTTP/1.1.** `http1_only()` on any production client would be a paper-fix for a
  non-problem and would degrade h2 multiplexing on capable endpoints (§Context §Option A remains
  correctly rejected).
- **Do NOT add per-sensor transport tuning.** §Context §Option C remains correctly rejected.
- **The interim relay is decommissioned.** The Python urllib HTTP/1.1 relay served as bisection
  evidence during investigation; it is not a production path and is already decommissioned.
- **Production-grade recurrence guard:** add a live canary regression and improved E-QUERY-004
  timeout diagnostics as the operational response to transient edge stalls. This is tracked in the
  re-scoped story S-ENGINE-H2-LARGE-RESPONSE-001 (canary + diagnostics, not D7 implementation).

### Downstream Impact

- **BC-2.16.002 §Postconditions "H2 Flow-Control Window Sizing":** any transcribed D7 mandate
  must be removed by the product-owner in the same session burst.
- **S-ENGINE-H2-LARGE-RESPONSE-001:** must be re-scoped by the story-writer from D7 implementation
  to live canary regression + E-QUERY-004 diagnostics. The D7 Red Gate assertion (SETTINGS frame
  check) is retired; the story's new acceptance criteria govern canary/diagnostic behavior only.

---

## Context

> **FALSIFIED HYPOTHESIS — HISTORICAL RECORD ONLY.** The root-cause analysis and mechanism
> selection below were produced before live wire evidence was available. The D7 decision derived
> from this context is WITHDRAWN (see §Withdrawal Rationale). This section is preserved for
> audit continuity; it does NOT represent an active mandate or a correct diagnosis.

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

**Option B — H2 fixed window sizing** (`.http2_initial_stream_window_size`,
`.http2_initial_connection_window_size`): ACCEPTED. Sets the initial flow-control window to
4 MiB (4 194 304 bytes), which covers the observed maximum page size (~1.1 MB) with a 3.7× safety
margin. `http2_adaptive_window` is NOT included: setting it to `true` overrides both explicit
window setters by resetting them to SPEC_WINDOW_SIZE = 65,535 bytes and enabling BDP estimation
from that baseline — silently negating the fix. Fixed windows guarantee the server can transmit a
full 1.1 MB/page with zero WINDOW_UPDATE round-trips, directly eliminating the peer flow-control
deadlock. This fix is surgical: it applies to the reqwest ClientBuilder and has zero behavioral
impact on sensors with small pages (the larger initial window just means fewer WINDOW_UPDATE round
trips, which is a performance improvement, not a regression).

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

**D7 — All production outbound sensor reqwest client builders MUST configure fixed 4 MiB h2
stream and connection windows; `http2_adaptive_window` MUST be omitted (amends ADR-050)**

Every production `reqwest::Client::builder()` chain that produces an outbound sensor/plugin HTTP
client — enumerated in ADR-050 §D6 — MUST also include:

```rust
.http2_initial_stream_window_size(4 * 1024 * 1024)      // 4 MiB stream window
.http2_initial_connection_window_size(4 * 1024 * 1024)  // 4 MiB connection window
// NOTE: http2_adaptive_window(true) MUST NOT be added — it overrides the above setters
// back to 65,535 bytes (SPEC_WINDOW_SIZE), silently negating this fix.
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
- 8 MB+: over-provisioning; no page in the observed corpus exceeds 4 MiB. Pages >4 MiB would
  still require WINDOW_UPDATE frames, which is an accepted, documented limit.

**`http2_adaptive_window` is explicitly excluded:** reqwest 0.12.28 applies builder setters in
fixed order at `.build()` time — window-size setters first, adaptive window last. When
`http2_adaptive_window(true)` is set, both `http2_initial_stream_window_size` and
`http2_initial_connection_window_size` are overridden back to `SPEC_WINDOW_SIZE = 65,535` bytes,
and the BDP estimator grows from that 64 KiB baseline — silently negating the fix. The adaptive
window also cannot resolve DEFECT-1: BDP estimation requires successful WINDOW_UPDATE exchanges
to grow, but DEFECT-1 is precisely a stall in the WINDOW_UPDATE path. Fixed 4 MiB windows bypass
the WINDOW_UPDATE path entirely for all observed page sizes.

---

## Rationale

The h2 flow-control window stall is a known class of defect in HTTP/2 client implementations.
h2 crate 0.4.18 uses the spec minimum (65535 bytes) as its default, which is correct for protocol
compliance but operationally inadequate for APIs that return MB-scale pages in a single response
(which is common for vulnerability and device inventory APIs in the OT/ICS sensor space).

Increasing the initial window to 4 MiB eliminates the stall for all sensor page sizes observed in
the prism corpus (max: ~1.1 MB for `claroty_vulnerabilities`). No adaptive window is used; the
fixed 4 MiB guarantee is sufficient for the observed corpus and avoids the WINDOW_UPDATE
dependency that is the root cause of DEFECT-1.

**Red Gate assertion** (required for story RG, replaces the discarded loopback-latency experiment):
A loopback h2 server cannot reproduce DEFECT-1: on 127.0.0.1 RTT is microseconds, so WINDOW_UPDATE
round-trips are free and a compliant h2 server delivers any response in milliseconds with ANY
initial window — the timing assertion passes before and after the fix. DEFECT-1 is a peer
flow-control deadlock, not generic slowness.

The deterministic gate is a SETTINGS frame assertion:

1. Bind a plain TCP listener on localhost (plain TCP, no TLS; use `.http2_prior_knowledge()` on
   the reqwest client in the test to bypass ALPN negotiation).
2. Spawn the production reqwest client under test with the D7 builder chain applied.
3. Have the client issue any GET request to the listener.
4. Server-side: call `h2::server::handshake(stream).await` — `h2` at v0.4.18 is already a
   transitive dependency of `reqwest`; add `h2 = "0.4"` to the test crate's `[dev-dependencies]`.
5. After the handshake future resolves, read
   `conn.remote_settings().initial_window_size()` — this returns the
   `SETTINGS_INITIAL_WINDOW_SIZE` value from the client's initial SETTINGS frame.
6. Assert `initial_window_size == Some(4_194_304)`.

**This test FAILS before the fix** (default `SETTINGS_INITIAL_WINDOW_SIZE` = 65,535 bytes) and
**PASSES after** (the two explicit `http2_initial_stream_window_size` setters inject 4,194,304
into the SETTINGS frame). `h2::server::Connection::remote_settings()` is available immediately
after `.handshake()` resolves; `Settings::initial_window_size()` returns `Option<u32>`.

---

## Consequences

### Positive
- `claroty_vulnerabilities` (and any future large-page sensor table) no longer stalls under direct
  h2 HTTPS — the root defect motivating S-CLAROTY-VULNS-001 is resolved.
- All current sensors with small pages (claroty_alerts ~5 KB, crowdstrike ~50 KB typical) are
  unaffected; larger initial window simply means fewer WINDOW_UPDATE frames — a performance
  improvement, not a behavioral change.
- Pages up to 4 MiB are transmitted with zero WINDOW_UPDATE overhead. Pages >4 MiB (none in the
  current corpus) would still require WINDOW_UPDATE frames; this is an accepted, documented limit.

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
active stream, because the connection window would be exhausted. Both must be set to 4 MiB.

**Alt-E: `http2_adaptive_window(true)` only (drop explicit setters)** — Rejected. This was v1.0's
unintended effective configuration. `http2_adaptive_window(true)` overrides both explicit window
setters back to SPEC_WINDOW_SIZE = 65,535 bytes and enables BDP estimation from that 64 KiB
baseline. BDP estimation requires successful WINDOW_UPDATE exchanges to grow the window; DEFECT-1
is precisely a stall in the WINDOW_UPDATE path — the peer is waiting for WINDOW_UPDATE frames that
are delayed. Fixed 4 MiB windows (Option A) bypass that path entirely for all observed page sizes.

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
| 1.2 | 2026-08-26 | architect | WITHDRAWN. D7 hypothesis falsified by live wire evidence: reqwest/hyper 1.9 defaults (2 MiB stream + 5 MiB connection) exceed the observed page size; direct h2 transport confirmed healthy end-to-end. No client transport change adopted. §Status rewritten, §Withdrawal Rationale added, §Context/§Decision annotated as falsified. Story re-scoped to canary + diagnostics. |
| 1.1 | 2026-08-26 | architect | Corrects v1.0 internal contradiction: `http2_adaptive_window(true)` overrides explicit window setters back to 65,535 bytes; dropped. D7 now specifies fixed 4 MiB stream + connection windows only. Red Gate redesigned from loopback-timing experiment to deterministic `SETTINGS_INITIAL_WINDOW_SIZE` assertion via `h2`-crate server. Aligns with Option A selection. |
| 1.0 | 2026-08-26 | architect | Initial — D7 h2 window sizing decision, extends ADR-050. |
