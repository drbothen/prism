---
document_type: analysis
title: "DEFECT-1 TLS Transport — Verification Finding (2026-09-02)"
date: "2026-09-02"
author: architect
status: CLOSED
conclusion: ALREADY_RESOLVED
blocking: false
relates_to: DEFECT-ADAPTER-TLS-XDOME-LIVE-001
pr_number: 237
pr_commit: 876e39c88
---

# DEFECT-1 TLS Transport — Verification Finding

**Verdict: DEFECT-1 is ALREADY RESOLVED. Direct HTTPS to `api.claroty.com` works with
the current rustls-tls stack. No remediation needed.**

---

## 1. Investigation Scope

The task was to confirm or refute whether prism can establish a direct HTTPS connection
to `api.claroty.com` using its rustls-tls stack after the Python relay
(`xdome-relay.py`) was decommissioned on 2026-08-26. The leading hypothesis was a
corporate MITM proxy / native cert-trust failure (rustls webpki-roots not loading OS
root store).

---

## 2. TLS Handshake Evidence

### 2a. Certificate Chain

`openssl s_client -connect api.claroty.com:443 -showcerts` and
`curl -vI https://api.claroty.com` both executed from this machine (2026-09-02):

| Depth | Subject | Issuer | Verify |
|-------|---------|--------|--------|
| 0 (leaf) | `CN=*.medigate.io` (SAN: `*.claroty.com`) | Amazon ECDSA 256 M01 | `verify return:1` |
| 1 (intermediate) | Amazon ECDSA 256 M01 | Amazon Root CA 3 | `verify return:1` |
| 2 (root) | Amazon Root CA 3 | Starfield Services Root CA G2 | `verify return:1` |

curl output: `SSL certificate verify ok`. openssl output: `Verification: OK` /
`Verify return code: 0 (ok)`.

**No MITM proxy interception detected.** The leaf cert serves a valid Amazon AWS
public CA chain, not a corporate CA.

### 2b. Amazon Root CA 3 in webpki-roots

The workspace uses `webpki-roots` (in `Cargo.lock` as `webpki-roots = "1.0.7"` —
the `reqwest` package block lists it as a direct dependency). The local crate source
(`~/.cargo/registry/src/.../webpki-roots-1.0.0/src/lib.rs`) explicitly contains an
`Amazon Root CA 3` trust anchor entry:

```
* Issuer: CN=Amazon Root CA 3 O=Amazon
* Subject: CN=Amazon Root CA 3 O=Amazon
* Label: "Amazon Root CA 3"
```

Amazon Root CA 3 is part of the Mozilla NSS root store. **rustls with default
webpki-roots DOES trust the cert chain presented by `api.claroty.com`.**

### 2c. rustls-native-certs is Absent

`grep "rustls-native-certs" Cargo.lock` returned no output. The
`rustls-tls-native-roots` reqwest feature is not declared in any `Cargo.toml`.
No `.add_root_certificate()` or `tls_built_in_root_certs(false)` calls exist in any
production client builder. **Prism uses ONLY webpki-roots, which is sufficient for
`api.claroty.com`.**

---

## 3. Current Client Construction — ADR-050 Compliance State

PR #237 (commit `876e39c88`, merged 2026-08-15) is confirmed as an ancestor of
current develop (`2edaaca78`) via `git merge-base --is-ancestor`.

### 3a. Cargo.toml Feature State (post-PR #237)

| Crate | reqwest features | http2 | rustls-tls | default-features |
|-------|-----------------|-------|------------|-----------------|
| `prism-spec-engine` | json, rustls-tls, **http2**, gzip, deflate, brotli | YES | YES | false |
| `prism-sensors` | json, rustls-tls, **http2** | YES | YES | false |
| `prism-bin` [dependencies] | json, rustls-tls, **http2** | YES | YES | false |
| `prism-bin` [dev-dependencies] | json, rustls-tls, http2 | YES | YES | false |

**ADR-050 §D1 (default-features = false):** Compliant — all four entries.
**ADR-050 §D2 (native-tls forbidden):** Compliant — no native-tls feature anywhere.
**ADR-050 §D5 (http2 in production deps):** Compliant — all three production crates.

### 3b. Cargo.lock Verification (AC-H2-001 / RG-008 red-gate)

The reqwest `[[package]]` block in `Cargo.lock` lists `"h2"` as a direct dependency —
this is the exact observable specified by AC-H2-001 and verified GREEN by RG-008
(`test_reqwest_http2_feature_active`) at merge time. **The http2 feature is active.**

### 3c. Client Builder Sites (ADR-050 §D6 — User-Agent)

Four production `reqwest::Client::builder()` chains all carry:
```rust
.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))
.timeout(Duration::from_secs(30))
```

| Builder | Location | Verified by |
|---------|----------|-------------|
| `build_http_client_with_timeout()` | `prism-spec-engine/src/pipeline.rs` | RG-006 AC-UA-001 |
| `build_http_client_with_custom_timeout()` | `prism-bin/src/spec_driven_adapter.rs` | RG-006 AC-UA-001 |
| Plugin fast-path builder | `prism-bin/src/boot.rs` | ADR-050 §D6 audit |
| Plugin normal-path builder | `prism-bin/src/boot.rs` | ADR-050 §D6 audit |

No builder calls `.add_root_certificate()`, `rustls-native-certs`, or
`danger_accept_invalid_certs`. **ADR-050 §D6 compliance: Compliant.**

---

## 4. Historical Root Cause (DEFECT-ADAPTER-TLS-XDOME-LIVE-001 — CLOSED)

The actual prior failure (discovered 2026-07-20, first live-tenant onboarding of
`monroe` against `api.claroty.com`) had TWO root causes, neither of which was
cert-trust:

**Root Cause A — No http2 feature compiled in:**
All production reqwest deps declared `default-features = false` without `http2`.
Result: prism was HTTP/1.1-only. `api.claroty.com` sits behind AWS Global Accelerator,
which prefers h2. A UA-less HTTP/1.1-only client with a rustls TLS fingerprint matched
a WAF block pattern.

**Root Cause B — No User-Agent header:**
`reqwest` emits no User-Agent by default. UA-less + h1-only + rustls fingerprint is
a recognized WAF block signature for this endpoint.

**Evidence that cert-trust was NOT the failure mode:**
The relay (`xdome-relay.py`) used Python `urllib` (HTTP/1.1-only, default Python UA,
OpenSSL system-cert fingerprint) and succeeded. The only differentiators between relay
success and prism failure were ALPN (h2 vs h1-only) and User-Agent presence. A
cert-trust failure would have also blocked the relay (which also used system certs, not
webpki-roots), but the relay succeeded. The leading hypothesis (corporate MITM / native
cert store) was never consistent with this relay evidence.

**Fix delivered:** PR #237 (`876e39c88`, merged 2026-08-15) added `"http2"` to all
four production reqwest entries and `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))`
to all four outbound client builders.

---

## 5. Post-Fix Live Validation

**D-2312 (2026-08-26):** Direct h2 transport to `api.claroty.com` confirmed healthy
end-to-end — `curl --http2` HTTP 200 (~1.2 MB, ~0.9s); reqwest 0.12.28 faithful
probe clean `END_STREAM`; real pre-fix prism binary fetched live CVE rows across 10
paginated pages, zero `E-QUERY-004`. Relay decommissioned; `monroe` overlay
`base_url` configured to `https://api.claroty.com` directly.

**D-2417 (2026-09-01):** A1 live release-validation executed on `monroe` with
`develop@1f805276` (binary `9f0ada1c...`). Result: 13/14 Claroty tables validated
live. The sole remaining v1 blocker was the `vulnerabilities` offset>0 hang — a
malformed API request (missing `sort_by` / `include_count` fields in the
`body_template`), unrelated to TLS transport.

---

## 6. Leading Hypothesis Assessment

**REFUTED. The corporate MITM / native cert-trust hypothesis does not apply here.**

| Hypothesis component | Evidence |
|----------------------|----------|
| Corporate MITM proxy intercepting `api.claroty.com` | NOT detected: openssl/curl both show Amazon public CA chain, `Verification: OK` |
| Corporate CA not in webpki-roots | NOT applicable: Amazon Root CA 3 IS in webpki-roots v1.0.7 (Mozilla NSS) |
| rustls rejecting the presented cert | NOT happening: the presented cert chains to a public CA that webpki-roots trusts |
| Need for `rustls-tls-native-roots` or `rustls-native-certs` | NOT needed for this endpoint; would contradict ADR-050 MITM-resistance rationale |

---

## 7. ADR Impact

**No ADR change required.** ADR-050 is correctly implemented and compliant.

ADR-050 explicitly chose webpki-roots over system certs for MITM resistance in an
MSSP threat model. That decision is validated by the fact that `api.claroty.com` uses
a standard public Amazon CA that webpki-roots trusts. The decision would only require
revisiting if prism were deployed in an environment where the production endpoints are
served through a corporate MITM proxy — that is a prospective concern, not a current
failure mode, and would require a new ADR rather than an amendment to ADR-050.

---

## 8. Blast Radius of Non-Issue

No code change is needed. The workspace is in a fully correct state:
- `rustls-tls` + `http2` on all three production crates
- User-Agent on all four outbound builders
- No native-tls, no native-certs
- Amazon Root CA 3 trusted by webpki-roots v1.0.7

---

## 9. Current v1 Blocker (Distinct from This Investigation)

The remaining v1 block (D-2420, 2026-09-02) is the `vulnerabilities` endpoint
offset>0 hang. Prism's `body_template` for the vulnerabilities step sends no
`sort_by` field and no `include_count` field; the OpenAPI contract
(`xdome_openapi_06.20.2026.json`) confirms these fields are required for paginated
requests (offset>0). This is an API conformance defect in the sensor TOML spec, not
a TLS or transport issue. Tracked under Action 1 (controlled test) and Action 2
(full endpoint conformance audit) per D-2420.

---

## 10. Recommended Delivery Plan

**Nothing to deliver for DEFECT-1.** The defect is closed; all fixes are on
`develop@2edaaca78`. No new story, no ADR amendment, no code change is warranted.

The only open action that could be considered here is a prospective story to add
`rustls-tls-native-roots` support for corporate MITM proxy deployments. This is a
post-v1 concern and would require:
- A concrete deployment scenario demonstrating a MITM proxy on the path to a sensor API
- An ADR-050 amendment documenting the decision to add system cert trust
- A story under a future maintenance wave (not v1-blocking)

**Immediate next step:** Proceed to Action 1 (vulnerabilities offset>0 controlled
test) and Action 2 (endpoint conformance audit) per D-2420 instructions.
