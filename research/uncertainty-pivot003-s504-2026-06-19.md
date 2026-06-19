# Research: Uncertainty Resolution — PIVOT-003 True-DTU Fidelity + S-5.04 Rust Capability

**Date:** 2026-06-19
**Type:** general (technology verification)
**Topics:** CrowdStrike behavior `ioc_type` enum, Cyberint alert IOC wire schema, Rust `Retry-After` parsing
**Status:** complete (2 CONFIRMED-with-correction, 1 INCONCLUSIVE-with-recommendation, 1 CONFIRMED)

> Scope note: This document records external research findings ONLY. It does not modify any story or spec artifact. Routing of any resulting code/spec change is the orchestrator's responsibility (architect for ADR/enum-policy, implementer for code).

---

## Item 1 — CrowdStrike behavior `ioc_type` enum (PIVOT-003)

### Question
Does the real CrowdStrike Falcon detection/alert API return `behaviors[].ioc_type` values of `{hash, domain, filename, registry, cmdline}`, and does it EVER return `ipv4`/`ipv6` on detection behaviors? Did the migration from `/detects/*` to `/alerts/*` change the taxonomy? The prism DTU clone currently asserts `ioc_type ∈ {hash, domain, filename, registry, cmdline}` (NOT ipv4/ipv6).

### Verdict: **CORRECTED** (the literal enum values are wrong; the ipv4/ipv6 exclusion is CONFIRMED-correct)

Two distinct problems with the current prism enum, plus one thing it got right:

**1a. The literal token values are incorrect (HIGH confidence correction).**
The real behavior `ioc_type` taxonomy, as documented by every independent integration that explicitly maps `.resources[].behaviors[].ioc_type` (ThreatQ CrowdStrike Insight EDR CDF; Cortex XSOAR CrowdStrike Falcon integration "possible values"), uses **algorithm-qualified and `_key`-suffixed** tokens — NOT the bare tokens prism currently asserts:

| prism current token | Real CrowdStrike `behaviors[].ioc_type` token(s) | Status |
|---|---|---|
| `hash` | `hash_sha256`, `hash_md5` (two distinct values, algorithm-qualified) | WRONG — bare `hash` is not emitted |
| `domain` | `domain` | CORRECT |
| `filename` | `filename` | CORRECT |
| `registry` | `registry_key` | WRONG — real token is `registry_key`, not bare `registry` |
| `cmdline` | (not an `ioc_type` value at all) | WRONG — `cmdline` is a SEPARATE behavior field (`behaviors.cmdline`), never an `ioc_type` |

Evidence convergence: ThreatQ's mapping table is explicitly keyed to `.resources[].behaviors[].ioc_type` and lists `domain → FQDN`, `filename → Filename`, `hash_md5 → MD5`, `hash_sha256 → SHA-256`, `registry_key → Registry Key`. Cortex XSOAR documents `CrowdStrike.Detections.behaviors.ioc_type` with possible values `hash_sha256`, `hash_md5`, `domain`, `filename`. Both list `behaviors.cmdline` as a *separate* field — confirming `cmdline` is process-command-line context, not an IOC type. Multiple SDKs (FalconPy `detects.py`, PSFalcon, gofalcon `domain.MsaDetectSummariesResponse`) corroborate the `behaviors[]`-with-`ioc_type`-and-`cmdline` shape.

**1b. ipv4/ipv6 exclusion is CORRECT (MEDIUM-HIGH confidence).**
The prism clone's decision to NOT include `ipv4`/`ipv6` on detection behaviors is well-supported. `ipv4`/`ipv6` are first-class IOC types in CrowdStrike's *separate* subsystems — the **custom IOC management API** (`/iocs/combined/indicator/v1`) and the **device indicator query API** (`/indicators/queries/devices/v1?type=...`, accepting `sha256|md5|domain|ipv4|ipv6`) — but no documentation or behavior-specific mapping (ThreatQ, XSOAR) lists `ipv4`/`ipv6` for `.resources[].behaviors[].ioc_type`. Third-party tools (Netskope Threat Exchange, FortiSOAR) that *appear* to list `ipv4`/`ipv6` "IOC types" are aggregating across endpoint detections + the IoC-management page into a unified internal schema — they do not demonstrate `behaviors[].ioc_type == ipv4`. **Caveat:** this is absence-of-evidence (no public OpenAPI enum exists), not a normative CrowdStrike enumeration. It is the correct production-grade default, but a tolerant parser (see below) is warranted.

**1c. Detects → Alerts migration did NOT change the taxonomy (HIGH confidence).**
The Detects service collection is officially deprecated (deprecated 2024-10-01, decommission 2025-09-30) in favor of the Alerts service collection (`GetQueriesAlertsV1`, `PostEntitiesAlertsV2`). All evidence indicates the IOC taxonomy was *preserved*, not changed: Elastic's CrowdStrike REST-API alert integration exposes `crowdstrike.alert.ioc_type` / `crowdstrike.alert.ioc_value` (same naming, not `indicator_type`); Google SecOps documents the migration as operational (permission + endpoint swap) with interchangeable detection-monitoring feeds and no breaking-change warning on IOC fields. No source shows a new/incompatible `ioc_type` taxonomy introduced by the migration.

### Implication for the DTU clone
If the goal is *true-DTU fidelity* (matching the real wire shape), the asserted enum should be corrected to `{hash_sha256, hash_md5, domain, filename, registry_key}`, `cmdline` should be removed from the IOC-type enum (it is a sibling field), and the ipv4/ipv6 *exclusion* from detection behaviors should be retained. Because the CrowdStrike OpenAPI does not publish a normative exhaustive enum, the production-grade parser should treat unknown `ioc_type` strings as a non-fatal "Other/Unknown" category (log + preserve raw string) rather than hard-failing — there may be undocumented or licence-gated types (e.g. URL/email observables) not present in the public integration mappings. **Route to architect** (enum-policy / DTU fidelity contract) then implementer.

### Sources (Item 1)
- CrowdStrike Detects API (deprecation notice): https://developer.crowdstrike.com/api-reference/collections/detects/
- CrowdStrike Alerts API: https://developer.crowdstrike.com/api-reference/collections/alerts/
- CrowdStrike IOC management API: https://developer.crowdstrike.com/api-reference/collections/ioc/
- ThreatQ CrowdStrike Insight EDR CDF (`.resources[].behaviors[].ioc_type` mapping table): https://helpcenter.threatq.com/Integration_Documentation/cdf/CrowdStrike_Insight_EDR_CDF.htm and PDF guide https://helpcenter.threatq.com/assets/PDFs/Integrations/CrowdStrike_Insight_EDR_CDF_Guide_v1.1.1.pdf
- Cortex XSOAR CrowdStrike Falcon integration (`behaviors.ioc_type` possible values + separate `behaviors.cmdline`): https://xsoar.pan.dev/docs/reference/integrations/crowdstrike-falcon and https://github.com/demisto/content/blob/master/Packs/CrowdStrikeFalcon/Integrations/CrowdStrikeFalcon/README.md
- Elastic CrowdStrike integration (`crowdstrike.alert.ioc_type`): https://www.elastic.co/docs/reference/integrations/crowdstrike
- Google SecOps Detects→Alerts migration advisory: https://security.googlecloudcommunity.com/google-security-operations-2/crowdstrike-detects-api-deprecation-5926
- Device indicator query types (ipv4/ipv6 context): https://www.blinkops.com/blog/how-to-search-for-iocs-across-devices-in-crowdstrike
- Netskope Threat Exchange CrowdStrike plugin (unified ipv4/ipv6 aggregation): https://docs.netskope.com/en/crowdstrike-plugin-for-threat-exchange-2/

---

## Item 2 — Cyberint alert IOC wire schema (PIVOT-003)

### Question
What is the real Cyberint (Check Point External Risk Management / Argos Edge) alerts-API response schema for IOC fields? Does an alert carry an `ioc` object, an `iocs` array, and an `alert_data` object with `ip`/`domain`/`url`? What are the EXACT JSON wire keys for IOC type and value (`type` vs `ioc_type` vs `indicator_type`; `value` vs `ioc_value`)? prism models `Alert { ioc: Option<Ioc>, iocs: Vec<Ioc>, alert_data: Option<AlertData> }` with `Ioc { ioc_type→serde(rename="type"), value }`.

### Verdict: **INCONCLUSIVE on inner IOC keys; CORRECTED on the singleton `ioc` field; CONFIRMED on `iocs` array + `alert_data`/`url`**

Check Point/Cyberint does NOT publish an open, detailed OpenAPI/Swagger for the `/alert/api/v1/alerts` endpoint. The schema was reconstructed from a Check Point support article (sk182975, Azure Sentinel integration) and SOAR/CDF integration docs (FortiSOAR, ThreatQ, Cortex XSOAR). Findings by sub-claim:

**2a. Singleton top-level `ioc` field — NOT documented anywhere (CORRECTION flag, MEDIUM confidence).**
The endpoint is `https://<tenant>.cyberint.io/alert/api/v1/alerts`. Its documented schema (sk182975) declares `iocs` as `type: array` and a `mitre` array of strings. **No source — Sentinel, FortiSOAR, ThreatQ, XSOAR alerts — references a singular top-level `ioc` object.** prism's `Alert.ioc: Option<Ioc>` therefore has no public-evidence basis and is likely modeling a field the API does not emit. This does not break parsing (an absent field deserializes to `None`), but it is dead schema surface for true-DTU fidelity. **Flag for architect/implementer review** — confirm via live tenant whether `ioc` ever appears; if not, remove it.

**2b. `iocs` array — CONFIRMED.**
sk182975's JSON schema declares `iocs` as `type: array`. The plural naming and the absence of any singular `ioc` strongly indicate the array is the canonical IOC container. prism's `iocs: Vec<Ioc>` is correct in shape.

**2c. `alert_data` object with `url` — CONFIRMED; `ip`/`domain` — UNCONFIRMED (plausible).**
FortiSOAR's Cyberint connector shows `alert_data` containing `url` plus a nested `screenshot { id, name }`. ThreatQ's Argos Edge CDF shows `alert_data` containing a nested `csv { id, name }`. So `alert_data` and `alert_data.url` are confirmed. **`alert_data.ip` and `alert_data.domain` are NOT shown in any public snippet** — plausible for IP/domain-centric alert types but unverified. prism's `alert_data: Option<AlertData>` is correct in shape; the specific `ip`/`domain` sub-fields need live validation.

**2d. Inner IOC wire keys (`type`/`value` vs `ioc_type`/`ioc_value`) — INCONCLUSIVE.**
This is the crux question and it cannot be answered from public documentation. What IS documented:
- Cyberint's **Risk Intelligence Feed** (a *separate* surface from alerts) canonically uses `ioc_type` and `ioc_value` — confirmed by Cortex XSOAR's Check Point EM Feed integration (`Cyberint.indicator.ioc_type` = "The indicator type", `Cyberint.indicator.ioc_value` = "The indicator value") and corroborated by ThreatQ's `ioc_type`/`ioc_value`/`ioc_attr` pattern.
- The **inner structure of the alert `iocs[]` elements is not exposed by any public source.** Whether alert-embedded IOCs reuse `ioc_type`/`ioc_value` (matching the feed) or use a simplified `type`/`value` (a common API-design choice when embedding) cannot be determined.

**Assessment of prism's current model** (`Ioc { ioc_type with serde(rename="type"), value }`): prism is betting on the *short* form (`type`/`value`). This is a *plausible* guess for embedded IOCs but is **unverified** and directly conflicts with the only documented Cyberint IOC naming convention (`ioc_type`/`ioc_value` in the feed). The two equally-plausible hypotheses are not distinguishable from public evidence.

### Recommendation (Item 2)
Resolve by empirical validation against a live Cyberint/ERM tenant (`GET /alert/api/v1/alerts`, inspect raw `iocs[]` element keys) OR by obtaining the partner/customer OpenAPI under NDA — this is the only way to get a normative answer. Until then, a production-grade deserializer should accept BOTH conventions defensively (serde `alias`): e.g. `#[serde(rename = "type", alias = "ioc_type")]` for the type key and `#[serde(alias = "ioc_value")]` for the value key. This makes the clone robust whichever convention the live API uses, and matches the "don't hard-code a single expected key" guidance. **Route to architect** (DTU fidelity contract decision: short-form bet vs dual-alias) then implementer; flag the `Alert.ioc` singleton for removal.

### Sources (Item 2)
- Check Point sk182975 (Azure Sentinel ERM alerts; `/alert/api/v1/alerts`, `iocs: array`, `mitre`): https://support.checkpoint.com/results/sk/sk182975
- FortiSOAR Cyberint connector (`alert_data.url`, `screenshot{id,name}`): https://docs.fortinet.com/document/fortisoar/1.1.0/cyberint/1066/cyberint-v1-1-0
- ThreatQ Cyberint Argos Edge CDF (`alert_data`, `csv`): https://helpcenter.threatq.com/assets/PDFs/Integrations/Cyberint_Argos_Edge_CDF_Guide_v1.0.1.pdf
- Cortex XSOAR Check Point EM Feed (`ioc_type`/`ioc_value` in the FEED, not alerts): https://xsoar.pan.dev/docs/reference/integrations/cyberint-feed
- Cortex XSOAR Cyberint alerts integration (`cyberint-alerts-fetch`, schema not exposed): https://xsoar.pan.dev/docs/reference/integrations/cyberint
- Cyberint Risk Intelligence Feed datasheet: https://e.cyberint.com/hubfs/Cyberint_Risk_Intelligence_Feed_Datasheet.pdf
- Check Point ERM services: https://www.checkpoint.com/services/infinity-global/external-risk-management-services/

---

## Item 3 — Rust `Retry-After` parsing capability (S-5.04)

### Question
(a) Does reqwest 0.12 parse `Retry-After` for the caller or return raw string? (b) Recommended Rust approach to parse BOTH delta-seconds and HTTP-date (IMF-fixdate) forms? (c) Is `httpdate` still maintained (latest version + last release)? (d) Can chrono 0.4 parse IMF-fixdate reliably (which function)? prism pins chrono 0.4.44, reqwest 0.12.28, NO httpdate.

### Verdict: **CONFIRMED** (with a clear minimal-dependency recommendation)

**3a. reqwest 0.12 does NOT parse `Retry-After` (CONFIRMED).**
reqwest 0.12 exposes `Retry-After` only as a raw `HeaderValue` via `response.headers().get(RETRY_AFTER)`. There is no `RetryAfter` type, no `response.retry_after()` method. Parsing/retry policy is intentionally left to the caller or to middleware (`reqwest-retry-after` + `reqwest_middleware`, which internally use `headers_retry_after` → `httpdate`). Confirmed by reqwest docs and by the existence/framing of the `reqwest-retry-after` crate ("adds support for the Retry-After header").

**3b. Recommended parsing approach (CONFIRMED idiom).**
Per RFC 7231, `Retry-After` is dual-form. The idiomatic, spec-conformant order:
1. Trim whitespace; attempt `s.parse::<u64>()` → delta-seconds → `Duration::from_secs(n)`.
2. On failure, parse as IMF-fixdate HTTP-date → absolute time → `Duration` relative to `SystemTime::now()`, clamping past/negative results to `Duration::ZERO` ("retry immediately").
Use `u64` (not `u32`) for delta-seconds to avoid overflow on large/hostile values. Represent intermediate meaning as an enum (`Delay(Duration)` / `Until(SystemTime)`) if you want to log the original form; accept an injected `now: SystemTime` for testability.

**3c. `httpdate` IS maintained — latest v1.0.3, last release ~2022 (~3 years ago) (CONFIRMED via crates.io).**
- Latest version: **1.0.3** (crates.io, verified 2026-06-19).
- Last release: "almost 3 years ago" per crates.io metadata (≈ 2022; the 1.0.x line is stable).
- MSRV 1.56.0, MIT OR Apache-2.0, ~481 SLoC, 11 versions, ~465M all-time downloads.
- Assessment: NOT abandoned — it is in stable maintenance mode. HTTP date formats (IMF-fixdate + legacy RFC 850/asctime) do not change, so infrequent releases reflect stability, not neglect. It provides exactly `parse_http_date` (→ `SystemTime`) and `fmt_http_date`. It is depended on transitively by the `headers`/`reqwest-retry-after` stack. Production-acceptable.

**3d. chrono 0.4 parses IMF-fixdate reliably via `DateTime::parse_from_rfc2822` (CONFIRMED — strongest possible evidence).**
chrono's own docs for `parse_from_rfc2822` include the literal worked example:
> `assert_eq!` on `"Wed, 18 Feb 2015 23:16:09 GMT"` → parses to offset `0`, 2015-02-18 23:16:09.
The docs explicitly document obsolete-zone support: `GMT`/`UT`/`Z` → `+0000` (Z identical to +0000; GMT/UT recognized), single-letter "military" zones → `-0000` (ambiguous, per RFC 2822 correction). So `DateTime::parse_from_rfc2822("Wed, 21 Oct 2025 07:28:00 GMT")` succeeds and yields the correct UTC instant. IMF-fixdate is a constrained subset of RFC 2822, so `parse_from_rfc2822` is the correct function.

**Critical pitfall (CONFIRMED):** Do NOT use `DateTime::parse_from_str` / `NaiveDateTime::parse_from_str` with a `%Z` format string (e.g. `"%a, %d %b %Y %H:%M:%S %Z"`) — chrono issue #1575 shows this FAILS on `"Sun, 28 Apr 2024 11:05:00 GMT"` with "not enough information for a unique date and time". `%Z` (named-zone) parsing is unreliable; the dedicated `parse_from_rfc2822` is the only correct chrono path. Also do not use `NaiveDateTime` (HTTP dates are timezone-aware/UTC).

### Recommendation (Item 3) — minimal-dependency, production-grade
prism already pins chrono 0.4.44 + reqwest 0.12.28 and has no httpdate. The minimal-dependency path is to **add NO new crate** and implement a small helper:
1. `s.trim().parse::<u64>()` → `Duration::from_secs(n)` for delta-seconds.
2. else `chrono::DateTime::parse_from_rfc2822(s)` → `.with_timezone(&Utc)` → `SystemTime` (`.into()`) → `duration_since(now)`, clamp `Err`/past to `Duration::ZERO`.
3. Return a typed `Result` with explicit error variants (empty / invalid-delta / invalid-http-date); treat parse failure as advisory (log + fall back to the client's own backoff). Unit-test `"120"` and `"Wed, 21 Oct 2025 07:28:00 GMT"` plus a past-date edge case with an injected `now`.

Adding `httpdate` (v1.0.3) is an equally valid, slightly-more-HTTP-strict alternative (it also handles legacy RFC 850/asctime, which `parse_from_rfc2822` does not). But given the project's existing chrono dependency and the "minimal dependency" constraint, the chrono path is preferred — real servers emit IMF-fixdate, which `parse_from_rfc2822` handles. **Route to implementer** (S-5.04); no architecture decision required (mechanical, answerable in scope).

### Sources (Item 3)
- reqwest docs (raw header access, no Retry-After parsing): https://docs.rs/reqwest/
- reqwest-retry-after (middleware, confirms reqwest lacks native support): https://crates.io/crates/reqwest-retry-after and https://docs.rs/reqwest-retry-after
- headers-retry-after (uses httpdate): https://docs.rs/headers-retry-after
- httpdate crates.io (v1.0.3, ~3yr last release, 465M downloads — verified 2026-06-19): https://crates.io/crates/httpdate
- httpdate repo: https://github.com/pyfisch/httpdate
- chrono DateTime docs (`parse_from_rfc2822` GMT example + obsolete-zone support — verified 2026-06-19): https://docs.rs/chrono/latest/chrono/struct.DateTime.html
- chrono issue #1575 (`%Z` parsing pitfall): https://github.com/chronotope/chrono/issues/1575

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 3 | Deep multi-source synthesis on (1) CrowdStrike behavior ioc_type taxonomy + Detects→Alerts migration, (2) Cyberint alert IOC wire schema, (3) Rust Retry-After parsing across reqwest/chrono/httpdate. reasoning_effort=high on all three. |
| Perplexity perplexity_ask | 1 | ≤2-sentence factual confirmation that chrono parse_from_rfc2822 accepts the GMT token. |
| Tavily tavily_extract | 1 (2 URLs) | Live verification of httpdate's current crates.io version/release date and chrono's parse_from_rfc2822 docs (GMT worked example). |
| Training data | 0 areas | All claims sourced to web findings; no reliance on model knowledge for version numbers or wire schemas. |

**Total MCP tool calls:** 5
**Training data reliance:** low — every verdict is grounded in cited live sources; version numbers verified directly against crates.io; chrono GMT support verified against the live docs.rs worked example.

### Confidence summary
- Item 1: HIGH on token-value correction (multiple independent behavior-keyed mappings agree); MEDIUM-HIGH on ipv4/ipv6 exclusion (absence-of-evidence, no public normative enum); HIGH on migration-preserves-taxonomy.
- Item 2: INCONCLUSIVE on inner IOC keys (no public source exposes alert `iocs[]` element structure) — empirical/NDA validation required; CONFIRMED on `iocs` array + `alert_data.url`; CORRECTION-flag on singleton `ioc`.
- Item 3: HIGH/CONFIRMED on all four sub-questions (verified against live crates.io + docs.rs).
