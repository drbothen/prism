---
document_type: adr
adr_id: "ADR-050"
title: "Workspace reqwest TLS Backend — rustls-tls Mandatory, native-tls Forbidden"
status: ACCEPTED
date: "2026-07-02"
modified: "2026-07-03"
version: "1.1"
producer: architect
subsystems_affected: [SS-01, SS-16, SS-17, SS-22]
supersedes: []
superseded_by: null
amends: null
anchor_stories: [S-DEMO-FIDELITY-REMEDIATION-001]
related_adrs: [ADR-014, ADR-049]
related_bcs: [BC-2.06.019]
locked_decisions: []
wiring_deferred_to: null
---

# ADR-050: Workspace reqwest TLS Backend — rustls-tls Mandatory, native-tls Forbidden

## Status

ACCEPTED v1.1 (2026-07-03). Established during S-DEMO-FIDELITY-REMEDIATION-001 to
resolve 4 deterministically-failing DTU stage-0 scenario tests caused by macOS
native-tls/Security.framework Keychain initialization overhead (~65s/process cold
start). Commit cf66151f standardized all remaining dev-dependency and optional-dep
reqwest entries to `default-features = false, features = ["rustls-tls"]`.

---

## Context

### Root-Cause: Keychain Init Cost in Test Processes

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
systematically timing out due to TLS backend mismatch. The failure was initially
misdiagnosed as WASMtime CPU contention (the same oversubscription class addressed
by ADR-049).

### Pre-Fix State (at S-DEMO-FIDELITY-REMEDIATION-001)

| Cargo.toml section | Pre-fix state | Post-fix (cf66151f) |
|--------------------|--------------|---------------------|
| prism-bin, prism-spec-engine, prism-sensors `[dependencies]`; prism-ocsf `[dev-dependencies]` | Already `default-features = false, features = ["rustls-tls"]` | Unchanged — already correct |
| prism-bin `[dev-dependencies]` | Missing `default-features = false` — native-tls active | Fixed |
| prism-dtu-{armis,claroty,crowdstrike,cyberint,jira,nvd,pagerduty,slack,threatintel} `[dev-dependencies]` | Missing `default-features = false` — native-tls active | Fixed (9 crates) |
| ocsf-proto-gen optional `download` feature dep | Missing `default-features = false` — native-tls active | Fixed |

Cargo.lock delta: −151 lines (native-tls, openssl-sys, and macOS Keychain transitive
deps removed from the dependency graph entirely).

### Security Review Outcome

Security review (S-DEMO-FIDELITY-REMEDIATION-001, 2026-07-02) **APPROVED**
rustls-tls as the correct backend for this project's threat model:

- **Corporate MITM proxy resistance.** rustls-tls uses `rustls-webpki-roots`
  (Mozilla Web PKI root store, compiled into the binary). A corporate MITM proxy
  that installs a custom root CA into the OS system certificate store will NOT
  intercept prism's outbound sensor API calls — by design. For an MSSP tool
  querying customer-tenant sensor APIs, operator-controlled proxy interception of
  credentials in flight is a threat, not a feature.
- **Memory safety.** rustls is pure Rust; native-tls links to OpenSSL (or
  Security.framework on macOS), introducing C-FFI memory-unsafety surface.
- **TLS version floor.** rustls enforces TLS 1.2+ by default; native-tls inherits
  the system-policy default, which may be lower on older macOS versions.
- **Supply-chain surface.** openssl-sys + native-tls + macOS Keychain transitive
  deps removed entirely. Smaller Cargo.lock footprint reduces advisory exposure.

**Accepted tradeoff.** When the `ocsf-proto-gen` optional `download` feature fetches
OCSF JSON schemas at build time, the rustls-tls backend will not trust corporate
root CAs installed in the OS certificate store. This is acceptable: the download
feature is a build-time convenience, not a production code path. Developers behind
a corporate MITM proxy can set `CARGO_HTTP_CAINFO` for Cargo itself or run the
download outside the proxy.

---

## Decisions

### D1 — All workspace reqwest deps MUST use `default-features = false, features = ["rustls-tls"]`

Every `reqwest` dependency entry across the entire workspace — `[dependencies]`,
`[dev-dependencies]`, and optional/feature-gated dependency entries — MUST be
declared with:

```toml
reqwest = { version = "...", default-features = false, features = ["rustls-tls", ...] }
```

`default-features = false` is mandatory regardless of which other features are
listed alongside it. Omitting this field silently enables `native-tls` (reqwest's
compiled-in default), which violates this rule even if `features` is otherwise
correct.

**Workspace-level coordination:** if the workspace root `[workspace.dependencies]`
section carries a reqwest entry, it MUST include `default-features = false`. Per-crate
feature additions (e.g., `features = ["json", "stream"]`) via
`reqwest = { workspace = true, features = [...] }` are fine — those are additive
and do not re-enable native-tls.

### D2 — native-tls is forbidden workspace-wide

The `native-tls` feature and its aliases (`default-tls`, `native-tls-alpn`,
`native-tls-vendored`) MUST NOT appear as a reqwest feature in any Cargo.toml in
the workspace. This includes explicit feature declarations and is a consequence of
omitting D1's `default-features = false`.

### D3 — New crates must declare `default-features = false` at first write

When a new workspace crate adds reqwest for the first time, `default-features = false`
and `features = ["rustls-tls"]` must be present at declaration time. There is no
"add reqwest first, correct features in a follow-up" acceptable practice — this is
the pattern that produced the 11-crate gap that required cf66151f.

### D4 — DTU stage-0 timing budgets assume ~0ms TLS init

Stage-0 DTU scenario test windows (50s per BC-2.06.019) are calibrated for
`reqwest::Client` construction cost of approximately 0ms (rustls, pure Rust, no
system framework init). Tests that fail systematically due to TLS backend overhead
are a configuration error (D1/D2 violation), not a flaky test requiring a wider
timing budget. Widening the budget to mask native-tls overhead is forbidden.

---

## Considered Alternatives

### Alt-A: native-tls in production, rustls-tls only in tests

Rejected. The security review approved rustls-tls as the correct production backend
for this MSSP threat model. Allowing native-tls in production would create a
backend mismatch between test and production execution paths, reducing test fidelity
and reintroducing the MITM proxy liability.

### Alt-B: rustls-tls only in DTU test crates; ignore for others

Rejected. The root cause was a missing `default-features = false` that can recur in
any crate at any point. A workspace-wide rule is the only recurrence-proof posture.
A partial rule creates a maintenance surface where reviewers must remember which
crates are in-scope and which are not.

### Alt-C: Vendor OpenSSL to normalize startup cost

Rejected. `native-tls-vendored` adds a large C codebase to the supply chain and
does not eliminate the macOS Keychain init path — only the OpenSSL compile step is
vendored; Security.framework calls remain. It also conflicts with the memory-safety
goal.

---

## Consequences

### Positive
- DTU stage-0 scenario tests that create `reqwest::Client` run with ~0ms TLS init;
  the 50s window provides ~800x timing margin (cf66151f verification: 4 formerly-
  quarantined tests pass at ~0.05s).
- Cargo.lock −151 lines; native-tls, openssl-sys, and macOS Keychain transitive
  deps removed from the dependency graph permanently.
- All reqwest clients workspace-wide use a single consistent TLS backend — no
  test-vs-production TLS behavior divergence.
- Corporate MITM proxy interception of prism's outbound sensor API calls is
  structurally prevented (rustls-webpki-roots, compiled-in Mozilla Web PKI roots).
- Memory-safe TLS across all HTTP surfaces; no C-FFI via OpenSSL.

### Tradeoff
- `ocsf-proto-gen` optional `download` feature will not trust corporate-proxy root
  CAs installed in the OS certificate store. Build-time schema download behind a
  corporate MITM proxy requires `CARGO_HTTP_CAINFO` or off-proxy execution.
  Documented; accepted (build-time convenience path, not production).

---

## Enforcement Recommendation

A CI grep gate that fails on any reqwest entry missing `default-features = false`
or using `native-tls` would prevent regression. The native-tls detection is
straightforward:

```bash
# Fail if any Cargo.toml explicitly enables native-tls
grep -rn '"native-tls"\|native-tls-alpn\|native-tls-vendored\|default-tls' \
  crates/*/Cargo.toml && echo "FAIL: native-tls in reqwest dep" && exit 1
```

Detecting the absence of `default-features = false` reliably requires multi-line
TOML awareness, since reqwest deps are frequently expressed across multiple lines.
A robust gate uses `cargo metadata --format-version 1 | jq` to enumerate all
reqwest dependency entries and verify `default_features: false` in the resolved
graph. This is a small but non-trivial script (analogous to the
`tests/external/perimeter-violation/` compile-fail gate in ADR-012).

**Engineering judgment: fast-follow story.** The existing codebase is correct
(cf66151f fixes all known violations). A fast-follow maintenance story scoped to
"add CI reqwest-tls-backend check to Justfile + ci.yml" is the correct vehicle:

- This ADR establishes the binding rule immediately.
- The gate prevents future regressions but is not required to make the current
  branch production-grade — the codebase is already correct.
- The reliable multi-line TOML check requires its own story with test coverage and
  a Justfile recipe, which is out of scope for the remediation branch.

The fast-follow story must cite this ADR and close with a passing CI run that
rejects a synthetic Cargo.toml violating D1 or D2.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-07-03 | architect | §Pre-Fix State table corrected (factual imprecision found during PR #208 PR-LEVEL adversarial review). Row 1 expanded: prism-ocsf `[dev-dependencies]` was already `rustls-tls, default-features=false` (unchanged by story) and is now listed in the "already correct" group alongside prism-bin/prism-spec-engine/prism-sensors `[dependencies]`. Row 2 narrowed: prism-spec-engine and prism-sensors had NO separate `[dev-dependencies]` reqwest entry and were NOT changed by cf66151f — removed from the "Fixed" row, which now lists only prism-bin `[dev-dependencies]`. Decision text (D1–D4), §Decisions, §Considered Alternatives, §Consequences, §Enforcement Recommendation, and §Context unchanged. |
| 1.0 | 2026-07-02 | architect | Initial ACCEPTED. S-DEMO-FIDELITY-REMEDIATION-001 cf66151f establishes workspace-wide reqwest TLS backend convention. D1: `default-features = false, features = ["rustls-tls"]` mandatory in all Cargo.toml sections; D2: native-tls + aliases forbidden; D3: new-crate declaration rule; D4: DTU stage-0 timing-budget calibration assumes rustls ~0ms init. Security review APPROVED (MSSP MITM threat model, memory-safety, supply-chain). Enforcement gate: fast-follow story (multi-line TOML check complexity; cf66151f fixes all current violations). |
