---
document_type: security-triage
title: "S-WAVE-A-ENGINE-001 — Rule 9 Cookie Name Charset: Security Triage"
author: security-reviewer
date: "2026-07-24"
status: OPEN
version: "1.0"
story_id: S-WAVE-A-ENGINE-001
blocking_cascade: true
perimeter_impact: REOPENS-WAVE-A
decision: "(B) REQUIRES SPEC AMENDMENT — E-SPEC-027 template (a) error message and BC-2.16.009 Rule 9 constraint text must change to describe RFC 6265 tchar; code-only fix would produce a misleading error message (POL-24 violation). Wave-A spec perimeter must reopen."
---

# S-WAVE-A-ENGINE-001 — Security Triage: Rule 9 Cookie Name RFC 6265 Charset Gap

Produced by security-reviewer in response to a pre-TDD uncertainty/research pass finding.
No source code files or `.factory/specs/` artifacts were modified by this review.

---

## Summary

Rule 9 of BC-2.16.009 validates the `header_scheme` TOML field. For the `"cookie:<name>"` form, the current frozen spec constrains `<name>` to: non-empty and no colon. This constraint does not cover the full RFC 6265 `cookie-name` / `token` character set. Characters including `;`, `=`, SPACE, TAB, high bytes, and all 16 RFC 9110 §5.6.2 delimiter characters other than `:` are accepted by the `http` crate's `HeaderValue` gate and reach the wire as malformed cookie names — or, for CTL characters (`\n`, `\r`, `\0`), are rejected by `HeaderValue` with a deferred opaque error at `.build()`/`.send()` time.

Two security findings result:

| ID | Title | Severity | CWE |
|----|-------|----------|-----|
| SEC-001 | Cookie pair injection via RFC 6265 non-token characters in `<name>` | HIGH | CWE-20 / CWE-74 |
| SEC-002 | Deferred `HeaderValue` rejection — opaque `"builder error"` for CTL characters | MEDIUM | CWE-390 |

Classification: **(B) REQUIRES SPEC AMENDMENT** — see §5.

---

## 1. Threat Model Determination

### 1.1 Who authors sensor TOML specs?

Two paths exist:

**Path A — Operator-authored (bundled or on-disk specs):** The four canonical sensor specs (`crowdstrike.sensor.toml`, `cyberint.sensor.toml`, `claroty.sensor.toml`, `armis.sensor.toml`) live in `crates/prism-sensors/specs/` and are committed to the repository by the MSSP operator/vendor. A non-operator cannot modify these files without OS-level write access to the deployment host's prism directory. `header_scheme` values in these files are fully under operator control.

**Path B — Runtime addition via `add_sensor_spec` (capability-gated):** CAP-029 states: "The `add_sensor_spec` tool is gated by `sensor_spec.write` capability and follows the hidden-tools pattern (BC-2.04.005)." The capability system (CAP-005) is deny-by-default; `sensor_spec.write` must be explicitly granted per-client in `prism.toml`. `add_sensor_spec` is classified as `ToolClass::WriteTool` (audit fail-closed). The tool accepts a free-form `toml_content` string which includes `header_scheme`.

**Conclusion:** `header_scheme` is controlled exclusively by parties who either (a) have OS-level write access to the specs directory, or (b) hold the `sensor_spec.write` capability granted by the MSSP operator. There is no code path that allows an arbitrary party to influence `header_scheme` without one of these gates.

### 1.2 Does any code path let a non-operator influence `header_scheme`?

No unauthenticated or unprivileged path was found. `validate_config` (the dry-run sibling of `add_sensor_spec`) also requires `sensor_spec.write` and does not persist specs. The MCP stdio transport has no authentication layer (CAP-034: "The stdio transport carries no authentication — the OS process boundary (analyst's user account) is the trust boundary"), which means the OS account running the prism process is implicitly trusted for all MCP calls. Any client holding `sensor_spec.write` by operator grant can call `add_sensor_spec`.

### 1.3 Deployment-model threat verdict

| Scenario | Attacker capability required | Finding classification |
|----------|-----------------------------|-----------------------|
| Built-in specs only (no `add_sensor_spec` grant) | OS-level write access to specs dir (operator/insider) | Trusted-config footgun |
| `sensor_spec.write` granted to an analyst client | MCP client session with the granted capability | **Active injection vector** |

The multi-tenant MSSP deployment (multiple clients, shared infrastructure, per-client capability grants) is the production configuration. The operator CAN grant `sensor_spec.write` to a client. If they do, a client can submit a spec with a crafted `header_scheme = "cookie:<name>"` where `<name>` contains `;` or `=`. Rule 9's current constraint does not prevent this. The threat is real — not theoretical.

---

## 2. SEC-001: Cookie Pair Injection via RFC 6265 Non-Token Characters

### SEC-001: Cookie Name RFC 6265 Tchar Charset Gap — Cookie Pair Injection Vector

- **Severity:** HIGH
- **CWE:** CWE-20 (Improper Input Validation) — root cause; CWE-74 (Injection) — consequence
- **OWASP:** A03:2021 - Injection

**Attack Vector:**

A party holding `sensor_spec.write` submits a spec where `header_scheme` uses a cookie name containing RFC 6265-illegal characters. Rule 9's current constraint (non-empty, no colon) does not reject these characters because `HeaderValue::from_str()` accepts them without sanitization.

The as-built injection site (pre-Rule 9 implementation) in `pipeline.rs` line ~1003:
```rust
// Current (hardcoded name):
req.header("Cookie", format!("access_token={}", token.as_str()))

// Post-Rule-9 (parameterized, per S-WAVE-A-ENGINE-001 T-C01):
req.header("Cookie", format!("{name}={token}"))
//         ^^^^^^^^^^^^^^^^^  <-- name extracted from header_scheme[7..]
```

With `header_scheme = "cookie:sid=x; admin"`, the name is `"sid=x; admin"` and:
```
format!("{name}={token}")  →  "sid=x; admin={token}"
Cookie header value:           Cookie: sid=x; admin={token}
```

The server receives TWO cookie pairs: `sid=x` and `admin={token}`. The auth credential ends up under the `admin` cookie key, not the intended `access_token` (or whatever name the spec author intended). The sensor API likely rejects the auth request (wrong cookie name).

**Complete character acceptance matrix** (from RFC 6265, RFC 9110 §5.6.2, and `http` crate 1.4.0 source, as verified in the research file):

| Character class | HeaderValue gate | Wire result |
|----------------|-----------------|-------------|
| `\n`, `\r`, `\0` and CTLs except TAB | REJECTED at `.header()` call (deferred to `.build()`/`.send()`) | Spec loads; every query returns opaque `"builder error"` |
| TAB (0x09) | ACCEPTED | Malformed cookie name on wire |
| SP (0x20) | ACCEPTED | Malformed cookie name on wire |
| **`;`** | **ACCEPTED** | **Cookie pair injection** |
| **`=`**  | **ACCEPTED** | **Cookie pair injection / name-value boundary corruption** |
| `( ) < > @ , \ " / [ ] ? { }` (other RFC 9110 delimiters) | ACCEPTED | Malformed cookie name on wire |
| High bytes (0x80–0xFF) | ACCEPTED | Malformed cookie name on wire (non-ASCII) |
| Valid tchar: `A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ \` | ~` | ACCEPTED | Correct RFC 6265 compliant cookie name |
| `:` | ACCEPTED by HeaderValue; **REJECTED by Rule 9 (as-specified)** | Caught by existing constraint |

**Impact:**

1. **Credential misrouting:** The auth token is mapped to an attacker-chosen cookie key rather than the spec-intended name. The token is still sent to the correct sensor API server (not to an attacker-controlled endpoint — `base_url` validation is independent). However, the credential is dispatched in a structurally incorrect way that invalidates the auth request at the target.

2. **Cookie pair injection:** `;` in `<name>` injects additional `key=value` pairs into the `Cookie` header sent to the sensor API. Depending on the sensor API's cookie handling, injected pairs could: (a) be silently ignored (auth still fails on wrong primary name), (b) trigger unexpected behavior if the API processes multiple cookie pairs, or (c) in a hypothetical scenario where the sensor API reflects cookies in responses, enable further attack chaining.

3. **Attack scope:** The injection affects outbound HTTP requests from prism to the external sensor API. It does NOT affect other tenants' requests (sensor requests are scoped to `(client_id, sensor_id)`). It does NOT exfiltrate the credential to an attacker-controlled endpoint.

**Evidence:** Research file `/Users/jmagady/Dev/prism/.factory/research/wave-a-engine-story-uncertainty-research.md` RQ-1 conclusion:
> "The `;`/`=`/SP gap is a **silent cookie-injection vector**, not just a hygiene issue."

**Proposed Mitigation:** See §5 (B classification) and §6 (recommended control).

---

## 3. Credential-Exposure Assessment

**The auth credential (token) is NOT exfiltrated to an attacker-controlled endpoint.** The injection mechanism only affects requests from prism to the external sensor API whose `base_url` is separately declared and validated in the spec. An attacker who can write `header_scheme` cannot use it to redirect the credential to a server they control — they would also need to control `base_url`, which is a separate field validated by Rule 1.

**Credential misrouting:** With `;` injection (`header_scheme = "cookie:sid=x; admin"`), the token is mapped to the `admin` cookie key. The value of `admin` IS the auth token. It travels to the correct sensor API but under the wrong key name. The sensor API rejects the auth because the expected cookie name (`access_token`) is absent. The credential is misrouted within the intended server communication, not exfiltrated.

**AD-017 cross-check (`header_scheme` in error messages):** The E-SPEC-027 template (a) substitutes `{value}` with the declared `header_scheme` value. For example, if `header_scheme = "cookie:foo;bar"` is submitted and Rule 9 fires (after the fix), the error message would include `"cookie:foo;bar"`. This is config text, not a credential value. AD-017 is not violated — `header_scheme` is a structural field, not a secret.

Under the CURRENT (pre-fix) Rule 9, a malicious spec like `header_scheme = "cookie:foo;bar"` passes Rule 9 silently and produces no error message at all. The malicious value does not appear in logs under the current behavior. After the fix, the value appears in the E-SPEC-027(a) rejection message — this is the DESIRED behavior (spec rejected, malicious value logged for operator forensics).

---

## 4. SEC-002: Deferred HeaderValue Rejection — Opaque Builder Error

### SEC-002: Opaque "builder error" for CTL Characters in Cookie Name

- **Severity:** MEDIUM
- **CWE:** CWE-390 (Detection of Error Condition Without Action at an Inappropriate Phase)
- **OWASP:** N/A (reliability/diagnosability finding, no direct OWASP Top 10 mapping)

**Finding:**

`\n`, `\r`, `\0`, and other CTL characters (except TAB) cause `HeaderValue::from_str()` to return `Err(InvalidHeaderValue)`. However, `reqwest` does not surface this error at `.header()` call time. It stores it internally as `self.request = Err(builder_error)` and surfaces it only at `.build()` or `.send()`, where `Display` produces the literally opaque string `"builder error"` (the root cause is two `.source()` hops down in the chain).

**Consequence:** A spec containing `header_scheme = "cookie:foo\nbar"` passes spec-load validation under the current no-colon Rule 9 (CRLF has no colon, non-empty). It loads cleanly at boot. Every subsequent query against the sensor fails with:
```
SpecEngineError::HttpRequestFailed { message: "builder error", ... }
```
The error message contains no indication that the cookie name contains a CRLF sequence. The operator's only recourse is to examine the raw TOML spec and manually identify the illegal character.

**Relationship to SEC-001:** The SEC-001 fix (enforcing RFC 6265 tchar charset in `validate_header_scheme`) also rejects CTL characters, since CTL characters are not in the tchar set. Therefore, the SEC-001 fix fully subsumes and eliminates the SEC-002 failure mode as a side effect. If SEC-001 is fixed, SEC-002 cannot occur.

**Proposed Mitigation:** Fix SEC-001 (tchar validation at spec-load time). This eliminates the deferred-error path entirely. As a belt-and-suspenders measure, the Rule 9 implementation MAY additionally attempt `HeaderValue::from_str(&format!("{name}=probe"))` at spec-load and emit a specific E-SPEC-027(a) error if it fails — but this is redundant once tchar validation is in place and is not required for the security fix.

---

## 5. (A)/(B) Classification — The Critical Decision

### Operative Frozen Spec Text

**From BC-2.16.009 Rule 9** (v1.23, frozen at 3/3 strict CLEAN per D-2011):

> `"cookie:<name>"` — inject `Cookie: <name>={token}`, where `<name>` is the non-empty cookie name substring after `cookie:` (`<name>` must be non-empty and must NOT contain a colon)

**From E-SPEC-027 template (a) in BC-2.16.009 §Error Conditions** (frozen, same version):

> `"sensor '{sensor_id}' has invalid header_scheme = '{value}'. Valid values: bearer, raw, cookie:<name> (non-empty name required, no colon in name)"`

**From E-SPEC-027 template (a) in error-taxonomy.md** (v2.66, frozen):

> Template (a) fires when `header_scheme` value is syntactically invalid: not one of `"bearer"`, `"raw"`, or `"cookie:<name>"` with a non-empty name containing no colon.

### The Classification Question

Can full RFC 6265 tchar validation be implemented within the story as a legitimate STRENGTHENING of the frozen spec's minimum constraint (Classification A), or does the error message template require amendment (Classification B)?

### Analysis

The frozen spec's constraint is: **non-empty AND no colon**. The RFC 6265 fix requires: **non-empty AND every byte is a tchar character** (77 permitted chars: `! # $ % & ' * + - . ^ _ \` | ~ DIGIT ALPHA`). The tchar set does not include `:`, so the tchar constraint subsumes the no-colon constraint. Logically, the tchar constraint is strictly stronger.

**However**: the error message template explicitly names the constraint as `"(non-empty name required, no colon in name)"`. This is not a general statement of validity — it is an enumeration of the SPECIFIC conditions that make a cookie name invalid.

If `validate_header_scheme` rejects `header_scheme = "cookie:foo;bar"` (no colon present in `foo;bar`) and returns the frozen error message, the message states: `"... (non-empty name required, no colon in name)"`. An operator reading this message would think: "But `foo;bar` has no colon — why is it rejected?" The message is factually incomplete: `;` is the actual reason for rejection, but the message only mentions `:`.

**This is a material diagnostic failure, not a minor wording issue.** Misleading error messages at spec-load time are the primary mechanism by which operators learn what to fix in their TOML specs. A message that says "no colon" when the real constraint is "tchar only" will cause operators to make incorrect corrections.

**POL-24 (source-of-truth byte-identity between taxonomy and code)** requires that the code's error message match the taxonomy VERBATIM. If the validation logic expands, the taxonomy message must expand to match. There is no POL-24-compatible path to implement tchar validation without updating the message.

**Classification: (B) REQUIRES SPEC AMENDMENT.**

Minimum spec changes required:
1. **BC-2.16.009 Rule 9** — `cookie:<name>` constraint description: replace "must be non-empty and must NOT contain a colon" with "must be non-empty and must consist entirely of RFC 6265 `cookie-name` / `token` characters (tchar: `A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ \` | ~`)"
2. **E-SPEC-027 template (a)** in BC-2.16.009 §Error Conditions AND in error-taxonomy.md: replace `"(non-empty name required, no colon in name)"` with `"(non-empty name required, RFC 6265 token characters only: A-Z a-z 0-9 ! # $ % & ' * + - . ^ _ \` | ~)"` or equivalent accurate description
3. **BC-2.16.009 Edge Cases**: Add entries for `;`-in-name, `=`-in-name, SP-in-name, CTL-in-name rejection (approximately EC-009-043 through EC-009-046)
4. **BC-2.16.009 Canonical Test Vectors**: Add a vector for RFC 6265 non-token character rejection

**Impact of (B)**: Reopens the Wave-A spec perimeter; resets BC-5.39.001 to 0/3. The scope is narrow (one rule, one error message template, a handful of edge cases). It does NOT require changes to the coherence matrix, error codes, other rules, or any other BC.

---

## 6. POL-36 / Q3 Adjudication — Safety Confirmation

The Q3 adjudication in `/Users/jmagady/Dev/prism/.factory/proposals/wave-a-engine-story-adjudication-Q1-Q5.md` ruled:

> "Rule 9 validates the SYNTAX and AUTH_TYPE COHERENCE of `header_scheme`. It does NOT restrict cookie names to an allowlist. `'cookie:cyberint_session'` would pass Rule 9's syntactic check. A Rule 9 allowlist restricting cookie names to `'access_token'` would encode Cyberint-specific knowledge in the engine — a direct POL-36 violation."

**The RFC 6265 tchar charset constraint is NOT a POL-36 violation and is compatible with the Q3 ruling.** The distinction is:

| Type | Example | POL-36 status |
|------|---------|--------------|
| **Name allowlist** (Q3-prohibited) | Only `access_token` and `session_id` are valid cookie names | VIOLATION — encodes sensor-specific knowledge |
| **Charset constraint** (proposed fix) | Cookie names must consist of RFC 6265 tchar characters | SAFE — applies to ALL cookie names equally, regardless of sensor identity |

The tchar constraint permits infinitely many valid cookie names: `access_token`, `session_id`, `api_key_1`, `Bearer_Token`, any RFC 6265-compliant string. It bans no specific name. Any sensor spec author remains free to choose any cookie name they want, as long as it uses only tchar characters. The constraint is about character legality, not name selection.

**The Q3 ruling is preserved. No POL-36 conflict exists with the recommended fix.**

---

## 7. Recommended Control

### Routing

| Step | Agent | Action |
|------|-------|--------|
| 1 | `vsdd-factory:product-owner` | Amend BC-2.16.009 Rule 9 + E-SPEC-027 template (a) error message per §5 |
| 2 | `vsdd-factory:architect` | Update ADR-053 §D2 validation table if it echoes "no colon" wording |
| 3 | `vsdd-factory:adversary` | Re-verify amended spec (new 3/3 clean streak required) |
| 4 | `vsdd-factory:implementer` | After spec converges: implement tchar validation in `validate_header_scheme` |

### Implementation (Step 4, code-side)

Replace the bare "no colon" check in the planned `validate_header_scheme` function with a tchar allowlist:

```rust
// RFC 6265 §4.1.1 cookie-name = token; RFC 9110 §5.6.2 tchar
fn is_valid_cookie_name_tchar(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| matches!(b,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' |
        b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~' |
        b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z'
    ))
}
```

This function naturally rejects `:` (not in tchar), `;`, `=`, SP, TAB, CTL chars, and high bytes. The existing "non-empty" and "no colon" conditions are fully subsumed. The tchar validation is the SINGLE correct implementation of the amended Rule 9.

**SEC-002 side effect:** The tchar validation eliminates the deferred `"builder error"` path for CTL characters entirely — they fail Rule 9 at spec-load time with a specific E-SPEC-027(a) error. No additional code change is needed for SEC-002.

### Wire-shape test (per wire-shape assertion discipline, 2026-07-13)

Add a test asserting the serialized `Cookie` header bytes for a known-valid spec to prevent regression to the deferred-error path:

```rust
// After build_request() with header_scheme = "cookie:access_token" and token = "tok123":
// The Cookie header value MUST be exactly b"access_token=tok123" on the wire.
// This catches any future regression where header construction is deferred or malformed.
```

---

## 8. Summary Table

| Finding | Severity | CWE | Root Cause | Exploitability | Fix Classification |
|---------|----------|-----|-----------|---------------|-------------------|
| SEC-001: Cookie pair injection (`; = SP TAB high-byte`) | HIGH | CWE-20 / CWE-74 | No-colon rule does not cover RFC 6265 tchar charset | Requires `sensor_spec.write` capability | **(B) Spec amendment** |
| SEC-002: Opaque `"builder error"` for CTL chars | MEDIUM | CWE-390 | HeaderValue rejection deferred to `.send()` with opaque display | Requires `sensor_spec.write` capability | Fixed as side effect of SEC-001 |

**Both findings BLOCK progression of S-WAVE-A-ENGINE-001.** The Wave-A spec perimeter must reopen for the Rule 9 cookie name constraint amendment before the story can enter TDD. The amendment scope is narrow; the adversarial cascade cost is real but bounded.

---

## Appendix A: Evidence Sources

| Source | Finding supported |
|--------|------------------|
| `/Users/jmagady/Dev/prism/.factory/research/wave-a-engine-story-uncertainty-research.md` RQ-1 | RFC 6265 tchar set; `http` 1.4.0 `HeaderValue` acceptance table; reqwest deferred-error semantics; BLOCKING recommendation |
| BC-2.16.009 v1.23 Rule 9 §Validation Rules 9, §Error Conditions E-SPEC-027, EC-009-030..032 | Frozen spec constraint text quoted above |
| error-taxonomy.md v2.66 E-SPEC-027 | Frozen error message template (a) quoted above |
| `/Users/jmagady/Dev/prism/.factory/proposals/wave-a-engine-story-adjudication-Q1-Q5.md` Q3 | POL-36 no-allowlist ruling; tchar constraint compatibility confirmed |
| CAP-029, CAP-034 in `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/capabilities.md` | `add_sensor_spec` capability gate; OS process boundary as trust boundary |
| `crates/prism-mcp/src/server.rs` lines 1742–1743, 3875–3925 | `add_sensor_spec` classified WriteTool; implementation confirming `sensor_spec.write` gate |
| `crates/prism-spec-engine/src/pipeline.rs` line ~998–1003 | As-built cookie injection site (hardcoded `access_token`); post-T-C01 shape |
