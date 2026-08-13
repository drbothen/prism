---
document_type: adr
adr_id: "ADR-050"
title: "Workspace reqwest TLS Backend — rustls-tls Mandatory, native-tls Forbidden, http2 and User-Agent Required for Sensor/Plugin Clients"
status: ACCEPTED
date: "2026-07-02"
modified: "2026-08-13"
version: "2.2"
producer: architect
subsystems_affected: [SS-01, SS-16, SS-17, SS-22]
supersedes: []
superseded_by: null
amends: null
anchor_stories:  # SAC-2 ground truth: a story belongs here only when its own ## Authority section cites this ADR
  - S-DEMO-FIDELITY-REMEDIATION-001  # §Authority verified: "ADR-050 v1.2 §D1/§D3/§D4" — scoped to AC-TLS only
  - DEFECT-ADAPTER-TLS-XDOME-LIVE-001  # §Authority verified: cites ADR-050 §D1/§D2 in §Authority table; extends to §D5/§D6 after v2.0
related_adrs: [ADR-014, ADR-049]
related_bcs: [BC-2.06.019, BC-2.16.002, BC-2.16.014]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-050: Workspace reqwest TLS Backend — rustls-tls Mandatory, native-tls Forbidden, http2 and User-Agent Required for Sensor/Plugin Clients

## Status

ACCEPTED v1.1 (2026-07-03) — original decisions D1–D4: rustls-tls mandatory, native-tls forbidden.

ACCEPTED v2.0 (2026-08-12) — D5/D6 added: `http2` feature required for sensor/plugin outbound
production clients; `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` required on all
sensor/plugin client builders. Triggered by DEFECT-ADAPTER-TLS-XDOME-LIVE-001: live xDome HTTPS
failed against AWS Global Accelerator (h2-preferring edge) because prism was HTTP/1.1-only and sent
no User-Agent, matching a WAF block fingerprint.

ACCEPTED v2.1 (2026-08-13) — §D6 scope extended to include `build_http_client_with_timeout` in
`crates/prism-spec-engine/src/pipeline.rs` (infusion `HttpLookupSource` outbound factory;
sibling-sweep gap in v2.0 enumeration). §D6 header clarified from "sensor and plugin outbound" to
"all outbound third-party HTTP" to make the universal scope and enumerated list consistent.
WAF-fingerprint-coherence reasoning applies to ALL outbound third-party HTTP, not only
sensor/plugin adapters. Anchored to DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOCAL adversary pass-2 OBS-4.

ACCEPTED v2.2 (2026-08-13) — §D5 prism-bin production entry count corrected: one
production `[dependencies]` reqwest entry (S-PLUGIN-PREREQ-D AC-9 shared outbound
client), not two; total production entries three (prism-spec-engine, prism-sensors,
prism-bin). Prism-bin `[dev-dependencies]` reqwest entry also carries `http2` (Cargo
feature unification, harmless; DTU dev-deps remain out of scope). Records-only — no
decision or mechanism change. Anchored to DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOCAL
adversary pass-4 F-3.

---

## Context

### D1–D4 Background — Keychain Init Cost in Test Processes

`reqwest` defaults to `native-tls` when built without explicit feature flags. On
macOS, `native-tls` uses the Security.framework backend, which triggers Keychain
initialization at TLS client construction. Cold-start cost: **~65 seconds per test
process** on the development machines used by this project.

Prism's DTU stage-0 scenario tests (BC-2.06.019) establish a `reqwest::Client` to
make HTTP calls against WireMock-backed DTU clones during Stage 0. The stage-0
timing window is 50 seconds. A 65-second Keychain init deterministically exceeds
that budget for every test binary that carried native-tls.

This was the root cause of 4 `#[ignore]`-quarantined tests across `prism-dtu-armis`
(×3) and `prism-dtu-crowdstrike` (×1). They were not flaky under load — they were
systematically timing out due to TLS backend mismatch.

The security review (S-DEMO-FIDELITY-REMEDIATION-001, 2026-07-02) approved
rustls-tls as the correct backend for this MSSP threat model: corporate MITM proxy
resistance (rustls-webpki-roots compiles Mozilla Web PKI roots into the binary),
memory safety (no C-FFI via OpenSSL), TLS 1.2+ floor, and smaller Cargo.lock
footprint.

Cargo.lock delta from cf66151f: −151 lines. D1/D2/D3 form the immutable baseline.

### D5–D6 Background — Live xDome HTTPS Failure (DEFECT-ADAPTER-TLS-XDOME-LIVE-001)

On 2026-07-20, first live-tenant onboarding of Claroty xDome (client "monroe",
endpoint `https://api.claroty.com`, behind AWS Global Accelerator) showed that
prism cannot communicate directly with the real xDome API. Every query returned
E-SENSOR-030 "all targets failed"; a localhost HTTP/1.1 relay forwarding verbatim
to the same endpoint succeeded with 9 paginated upstream requests returning full data.

Source-confirmed root causes (from code inspection of production crates at the time
of bisection):

1. **`http2` not compiled in.** All production reqwest deps declare
   `default-features = false` without re-enabling `http2`. Prism is HTTP/1.1-only
   against an h2-preferring AWS Global Accelerator front-end.

2. **No User-Agent header.** `reqwest` sends no User-Agent by default when the
   feature is not enabled. UA-less + rustls/webpki fingerprint + h1-only is a
   recognized WAF block signature.

3. **Error evidence destroyed.** reqwest error source chains, non-2xx response
   bodies, and per-target FanOutErrors were all dropped before reaching E-SENSOR-030
   detail, making the root cause invisible without the relay bisection. The
   companion error-surfacing fix is DEFECT-SENSOR-ERROR-FLATTEN-001.

The relay proved request generation, auth, pagination, decompression, and OCSF
parsing were all correct. The failure was isolated to the direct TLS/HTTP exchange
with the AWS edge.

---

## Decision

Six binding workspace-wide decisions govern all reqwest usage:

**D1 — All workspace reqwest deps MUST use `default-features = false, features = ["rustls-tls"]`**

Every `reqwest` dependency entry across the entire workspace — `[dependencies]`,
`[dev-dependencies]`, and optional/feature-gated dependency entries — MUST be
declared with:

```toml
reqwest = { version = "...", default-features = false, features = ["rustls-tls", ...] }
```

Omitting `default-features = false` silently enables `native-tls` (reqwest's
compiled-in default), which violates this rule even if `features` is otherwise
correct. Workspace-level `[workspace.dependencies]` entries MUST also carry
`default-features = false`.

**D2 — native-tls is forbidden workspace-wide**

The `native-tls` feature and its aliases (`default-tls`, `native-tls-alpn`,
`native-tls-vendored`) MUST NOT appear as a reqwest feature in any Cargo.toml in
the workspace.

**D3 — New crates must declare `default-features = false` at first write**

There is no "add reqwest first, correct features in a follow-up" acceptable
practice — this is the pattern that produced the 11-crate gap requiring cf66151f.

**D4 — DTU stage-0 timing budgets assume ~0ms TLS init**

Stage-0 DTU scenario test windows (50s per BC-2.06.019) are calibrated for
`reqwest::Client` construction cost of approximately 0ms (rustls, pure Rust, no
system framework init). Widening the budget to mask native-tls overhead is
forbidden.

**D5 — `http2` feature MUST be included in production reqwest deps for sensor and plugin outbound clients (v2.0)**

The production `[dependencies]` reqwest entries in the following crates MUST include
`"http2"` in their features list alongside `"rustls-tls"`:

- `crates/prism-spec-engine/Cargo.toml`
- `crates/prism-sensors/Cargo.toml`
- `crates/prism-bin/Cargo.toml` — the single production `[dependencies]` reqwest entry
  (S-PLUGIN-PREREQ-D AC-9 shared outbound client)

Note: `crates/prism-bin/Cargo.toml` also carries a `[dev-dependencies]` reqwest entry
which includes `http2` — this is Cargo feature unification (harmless); the dev-dep
does not affect production builds and DTU dev-deps remain out of scope for D5.

The `http2` feature enables h2 negotiation via ALPN during TLS handshakes. It is
**additive**: when the server does not advertise h2 in ALPN, reqwest falls back to h1.
For plain HTTP (non-TLS) connections — as used by DTU test harnesses — ALPN does not
apply; DTU dev-deps are explicitly out of scope for D5.

**D6 — All outbound third-party HTTP client builders MUST call `.user_agent(...)` (v2.0; scope extended v2.1)**

Every `reqwest::Client::builder()` chain that produces a client used for any outbound
third-party HTTP call MUST include:

```rust
.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))
```

`concat!` produces a `&'static str` with zero runtime allocation. Scope:

- `build_http_client_with_custom_timeout` in `crates/prism-bin/src/spec_driven_adapter.rs`
  — canonical factory; adding here propagates to `build_http_client_with_timeout` (prism-bin)
  and consequently to `DeclarativeHttpAuthProvider` (which calls `build_http_client_with_timeout()`
  internally, per BC-2.16.014)
- Both `reqwest::Client::builder()` sites in `crates/prism-bin/src/boot.rs` that
  produce the `PluginRuntime` HTTP client (the `PRISM_DISABLE_PLUGIN_LOAD` fast-path
  builder and the normal-path builder)
- `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs`
  — factory for infusion `HttpLookupSource` outbound clients (real third-party
  enrichment/threat-intel HTTP); extended v2.1 per DEFECT-ADAPTER-TLS-XDOME-LIVE-001
  LOCAL adversary pass-2 OBS-4 (sibling-sweep gap in v2.0 enumeration); verified by
  `test_infusion_http_client_sends_prism_user_agent`

DTU test clients that call WireMock stubs are excluded.

---

## Rationale

**D1/D2/D3/D4:** The MSSP threat model is the primary driver. rustls-tls uses the
Mozilla Web PKI root store compiled into the binary; a corporate MITM proxy
installing a custom root CA into the OS certificate store will NOT intercept prism's
outbound sensor API credential traffic. native-tls inherits the OS trust store,
which is a liability for MSSP tooling accessing customer-tenant credentials. Memory
safety (no C-FFI via OpenSSL) and the TLS 1.2+ floor are additional positive
properties. The test-performance benefit (no ~65s Keychain init) is a consequence
of the correct production decision, not the reason for it.

**D5:** Cloud edge infrastructure (AWS Global Accelerator, Cloudflare, Akamai)
preferentially routes h2 clients and may apply different WAF profiles to h1-only
clients. Enabling h2 via ALPN changes the TLS ClientHello fingerprint and removes
the h1-only signal from WAF detection. The h2 feature is backwards-compatible
by design — ALPN negotiation falls back to h1 when the server doesn't support h2,
so there is no downside to enabling it workspace-wide for all production sensor
clients. Confirming via Alt-D and Alt-F below: native-tls is not the answer and
per-sensor opt-in adds complexity with no benefit.

**D6:** A User-Agent header allows sensor vendor WAFs and rate limiters to identify
and whitelist the prism client. Combined with h2 ALPN (D5), the client presents a
coherent "known client" fingerprint rather than an anonymous UA-less probe. The
specific value `concat!("prism/", env!("CARGO_PKG_VERSION"))` is a compile-time
constant (zero overhead), human-readable, and encodes the client version for vendor
debugging. The DEFECT-ADAPTER-TLS-XDOME-LIVE-001 bisection confirmed UA-absence was
a contributing factor in the xDome WAF block. The WAF-fingerprint-coherence reasoning
applies to ALL outbound third-party HTTP, not only sensor/plugin adapter clients —
infusion `HttpLookupSource` clients call real third-party enrichment and threat-intel
endpoints subject to the same cloud-edge WAF profiles; scoping D6 to sensor/plugin
clients only would recreate the xDome defect class at the enrichment surface (v2.1
scope extension, DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOCAL adversary pass-2 OBS-4).

**BC-2.16.014 propagation note:** `DeclarativeHttpAuthProvider` constructs its
`reqwest::Client` via `build_http_client_with_timeout()` (per BC-2.16.014
INV-014-007). Adding `.user_agent(...)` to `build_http_client_with_custom_timeout`
automatically satisfies D6 for the auth token acquisition client without touching
`prism-spec-engine` directly.

---

## Consequences

### Positive (v1.x — unchanged)
- DTU stage-0 scenario tests run with ~0ms TLS init; the 50s window provides ~800x
  timing margin (4 formerly-quarantined tests pass at ~0.05s each).
- Cargo.lock −151 lines; native-tls, openssl-sys, and macOS Keychain transitive
  deps removed permanently.
- All reqwest clients workspace-wide use a single consistent TLS backend — no
  test-vs-production TLS behavior divergence.
- Corporate MITM proxy interception of prism's outbound sensor API calls is
  structurally prevented (compiled-in Mozilla Web PKI roots).
- Memory-safe TLS across all HTTP surfaces; no C-FFI via OpenSSL.

### Positive (v2.0 — D5/D6)
- Direct HTTPS communication with cloud-edge-hosted sensor APIs (AWS Global
  Accelerator, Cloudflare WAF) is restored without a relay sidecar. The xDome
  production case (`api.claroty.com`) is the confirmed instance.
- h2 multiplexing is available on servers that support it (lower latency on
  repeated paginated fetches against the same sensor endpoint).
- `User-Agent: prism/{VERSION}` allows sensor vendors to identify and whitelist
  prism in WAF and rate-limit tiers.

### Positive (v2.1 — §D6 scope extension)
- Infusion `HttpLookupSource` clients (real third-party enrichment/threat-intel HTTP
  via `build_http_client_with_timeout` in `prism-spec-engine`) now present the same
  coherent WAF fingerprint as sensor/plugin clients, preventing the xDome defect
  class at the enrichment surface.

### Negative / Trade-offs
- **v1.x:** `ocsf-proto-gen` optional `download` feature will not trust
  corporate-proxy root CAs. Build-time schema download behind a corporate MITM
  proxy requires `CARGO_HTTP_CAINFO`. Accepted (build-time convenience, not
  production).
- **v2.0 (D5):** The `h2` crate and its transitive deps are added to Cargo.lock.
  Expected size increase: ~150–200 Cargo.lock lines. Correct tradeoff for h2
  capability on production sensor connections.

### Status as of v2.2 (2026-08-13)

D1–D4 in effect since cf66151f (2026-07-02); verified by 4 formerly-quarantined
DTU stage-0 tests passing at ~0.05s each after fix. D5/D6 PENDING implementation
by DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (v2.0). §D6 scope extended in v2.1 to include
`build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs`;
verified by `test_infusion_http_client_sends_prism_user_agent`. §D5 prism-bin entry
enumeration corrected in v2.2 (records-only; no behavioral change). Enforcement gate
(CI check) is a fast-follow maintenance story — existing codebase will be correct
after the story closes.

---

## Alternatives Considered

**Alt-A: native-tls in production, rustls-tls only in tests** — Rejected. Security
review approved rustls-tls as the correct production backend for the MSSP threat model.
Allowing native-tls in production creates test-vs-production TLS backend divergence and
reintroduces the MITM proxy liability.

**Alt-B: rustls-tls only in DTU test crates; ignore for others** — Rejected. A
workspace-wide rule is the only recurrence-proof posture. A partial rule creates a
maintenance surface where reviewers must remember which crates are in-scope.

**Alt-C: Vendor OpenSSL (`native-tls-vendored`)** — Rejected. Does not eliminate the
macOS Keychain init path (only the OpenSSL compile step is vendored). Conflicts with
the memory-safety goal. Adds a large C codebase to the supply chain.

**Alt-D: Switch to native-tls to resolve the xDome WAF issue** — **Explicitly
rejected.** native-tls is prohibited under D2 and remains so in v2.0. It does NOT
address the root cause (ALPN h2 negotiation + UA absence). Switching would
reintroduce ~65s macOS Keychain init overhead (D4 violation), reintroduce the
corporate MITM proxy interception path for sensor API credentials, and open the
C-FFI memory-unsafety surface via OpenSSL linkage.

**Alt-E: Force HTTP/1.1 on all clients** — Rejected for the xDome fix. Forcing h1
would degrade performance on h2-capable servers without resolving the User-Agent gap.
The problem is the absence of h2 negotiation changing the TLS fingerprint, not h2
itself.

**Alt-F: Per-sensor opt-in for http2 via sensor TOML config** — Rejected. h2 is
backwards-compatible (ALPN falls back to h1); enabling it workspace-wide for all
production sensor clients has no downside and ensures all future sensors benefit
automatically. Per-sensor configuration adds a new class of configuration errors.

---

## Source / Origin

**D1–D4 origin:** S-DEMO-FIDELITY-REMEDIATION-001, commit cf66151f (2026-07-02).
Root cause was DTU stage-0 test failures confirmed via test timing measurements.
Security review rationale is documented in the §Context §Security Review Outcome
subsection above. Brownfield implementation evidence: `build_http_client_with_custom_timeout`
in `crates/prism-bin/src/spec_driven_adapter.rs` and all workspace Cargo.toml reqwest
entries conform to D1/D2.

**D5/D6 origin:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001. Primary evidence:
`findings/prism-pql-deficiencies.md` §Finding 10 (authored 2026-07-20). Bisection
evidence: curl to `api.claroty.com` → HTTP/2 200; prism direct → connection fails
~140ms post-TLS; prism via HTTP/1.1 relay → full success (9 paginated requests,
HTTP 200). Code-confirmed root causes: `build_http_client_with_custom_timeout`
(the sole production reqwest client factory) has no `.user_agent(...)` call; no
production crate's Cargo.toml reqwest entry includes `http2` in features.

**§D6 v2.1 extension origin:** DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOCAL adversary
pass-2 OBS-4. Sibling-sweep gap: v2.0 §D6 enumerated scope omitted
`prism-spec-engine::pipeline::build_http_client_with_timeout` (the infusion
`HttpLookupSource` client factory). Implementation already correct per
`test_infusion_http_client_sends_prism_user_agent`; this amendment closes the
spec-enumeration gap.

---

## Enforcement Recommendation

A CI grep gate that fails on any reqwest entry missing `default-features = false`
or using `native-tls` would prevent regression:

```bash
# Fail if any Cargo.toml explicitly enables native-tls
grep -rn '"native-tls"\|native-tls-alpn\|native-tls-vendored\|default-tls' \
  crates/*/Cargo.toml && echo "FAIL: native-tls in reqwest dep" && exit 1
```

Detecting the absence of `default-features = false` reliably requires multi-line
TOML awareness. A robust gate uses `cargo metadata --format-version 1 | jq` to
enumerate all reqwest dependency entries and verify `default_features: false` in
the resolved graph.

**D5/D6 enforcement gap:** No automated gate currently verifies that `http2` is
present in production deps or that `.user_agent(...)` is called in every outbound
third-party client builder. The existing codebase will be correct after
DEFECT-ADAPTER-TLS-XDOME-LIVE-001. A follow-up story adding CI checks is the
correct vehicle.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.2 | 2026-08-13 | architect | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOCAL adversary pass-4 F-3 closure. §D5 prism-bin production entry count corrected: one `[dependencies]` reqwest entry (S-PLUGIN-PREREQ-D AC-9 shared outbound client), not two; total three production entries (prism-spec-engine, prism-sensors, prism-bin). Prism-bin `[dev-dependencies]` reqwest entry also carries `http2` (Cargo feature unification; harmless; DTU dev-deps remain out of scope for D5). Records-only — no decision or mechanism change. |
| 2.1 | 2026-08-13 | architect | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 LOCAL adversary pass-2 OBS-4 closure. §D6 scope extended to include `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs` (infusion `HttpLookupSource` outbound factory; sibling-sweep gap in v2.0 enumeration); verified by `test_infusion_http_client_sends_prism_user_agent`. §D6 header clarified to "all outbound third-party HTTP client builders" — universal scope and enumerated list now consistent. §D6 Rationale extended: WAF-fingerprint-coherence applies to ALL outbound third-party HTTP including infusion clients. §Source/Origin extended with §D6 v2.1 extension origin. D5 http2 feature and D1–D4 TLS decisions unchanged. |
| 2.0 | 2026-08-12 | architect | DEFECT-ADAPTER-TLS-XDOME-LIVE-001 adjudication. D5 added: `http2` reqwest feature MUST be included in production [dependencies] for prism-spec-engine, prism-sensors, and prism-bin — enables h2 ALPN negotiation with cloud-edge fronts; falls back to h1 gracefully; DTU dev-deps excluded. D6 added: scope is `build_http_client_with_custom_timeout` (covers all sensor adapter clients and, via delegation chain, `DeclarativeHttpAuthProvider`) and both `PluginRuntime` client builders in boot.rs — value is `concat!("prism/", env!("CARGO_PKG_VERSION"))`. Alt-D (native-tls) and Alt-E (force h1) explicitly rejected with rationale. related_bcs extended: BC-2.16.002, BC-2.16.014. anchor_stories extended: DEFECT-ADAPTER-TLS-XDOME-LIVE-001. Template conformed: sections renamed to Rationale, Alternatives Considered, Source / Origin per adr-template.md. Title updated to reflect D5/D6 scope. |
| 1.3 | 2026-07-27 | architect | FB80 SAC-2 promotion: S-DEMO-FIDELITY-REMEDIATION-001 promoted from SAC-2-UNVERIFIED to verified. Story v2.45 §Authority cites ADR-050 v1.2 §D1/§D3/§D4; scoped to AC-TLS only. SAC-2-UNVERIFIED comment block removed. |
| 1.2 | 2026-07-27 | architect | FB76 SAC-2 sweep: S-DEMO-FIDELITY-REMEDIATION-001 demoted to SAC-2-UNVERIFIED — story had no §Authority section at that time. |
| 1.1 | 2026-07-03 | architect | §Pre-Fix State table corrected (factual imprecision found during PR review). |
| 1.0 | 2026-07-02 | architect | Initial ACCEPTED. D1–D4 established by S-DEMO-FIDELITY-REMEDIATION-001 cf66151f. Security review APPROVED. |
