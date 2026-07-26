> **Vendored reference** — Originally located at `demo-soc/findings/prism-armis-endpoint-plan.md`
> in the external `test-soc` repository (`/Users/jmagady/Dev/test-soc`), which is outside the
> prism repo and inaccessible to CI, reviewers, and other agents. Vendored here 2026-07-25 to
> close adversary finding **F-WASE-P64-HIGH-007**: ADR-053 §D1 "No-OpenAPI governance" makes
> the Confirmed/Partial/Unconfirmed confidence tiers in this document the binding grounding
> contract for Armis spec authoring; that contract must be auditable from inside the prism
> repository. Content is preserved verbatim from the original. Per-repo citations inside this
> document (e.g., `crates/…:line` provenance notation) are pinned to commit **e116a587** of the
> prism `develop` branch as of 2026-07-19 and may have drifted in subsequent development.

---

# Armis Centrix Sensor-Spec Build-Out Plan — Endpoint Audit

**Date:** 2026-07-20
**Audience:** Prism Armis spec maintainer + Prism engineering (auth item)
**Source:** Armis dev docs (dev.armis.com), the official v1→v3 migration
guide, and multiple production connectors (Google Chronicle, Cortex XSOAR,
Swimlane, Brinqa, Sumo Logic, Hunters). NO downloaded OpenAPI existed for
Armis — this is web-corroborated, with confidence flags inline. Parallel to
`prism-xdome-endpoint-plan.md` / `prism-xdome-spec-gaps.md`.

## Context for external readers

This document is one of a findings set from building a SOC (Security Operations Center) automation demo and onboarding the first live client on top of **Prism**. It is written to stand alone; the glossary and provenance below define every project-specific term used across the set (not all terms appear in this document).

**Glossary**
- **Prism** — a multi-client security-sensor query layer, served as an MCP (Model Context Protocol) server named `prism`. Fronts sensor adapters (CrowdStrike, Armis, Claroty/xDome, Cyberint) and returns OCSF-normalized rows via a query DSL.
- **PrismQL / PQL** — Prism's query language (SQL mode, pipe mode `FROM t | where…`, hybrid).
- **OCSF** — Open Cybersecurity Schema Framework (the normalized event schema Prism maps into). **MCP** — Model Context Protocol. **SOC** — Security Operations Center. **IOC** — Indicator of Compromise. **CVSS** — Common Vulnerability Scoring System. **PLC** — Programmable Logic Controller (industrial device). **CTD** — Claroty's Continuous Threat Detection product. **EDR** — Endpoint Detection & Response.
- **Sensor spec / overlay** — a TOML file describing a sensor's tables, columns, HTTP steps, and `response_path` (a JSONPath into the API response body). Base specs: `<config-dir>/specs/<sensor>.sensor.toml`; per-client overlays: `<config-dir>/specs/customers/<client>/<sensor>.sensor.toml`. `${…}` tokens are request-templating placeholders; `options=["INDEX"]` marks a column push-down-eligible.
- **DTU** — Digital Twin Universe: behavioral CLONES of the real sensor APIs used by the demo. Clients `org-a`/`org-b`/`org-c` are DTU clones; **`monroe`** is the first REAL client (a live Claroty xDome tenant).
- **AQL / ASQ** — Armis Query Language. **FQL** — Falcon (CrowdStrike) Query Language. Both are server-side filter languages.
- **The demo loop** — a scheduled harness that every 15 min polls each client through Prism, investigates new alerts, writes local ticket files, and SIMULATES JIRA submission (`DEMO-####` keys in a local `jira-mock.jsonl`; no real JIRA is written). **TP/FP/BTP** — analyst dispositions: True Positive / False Positive / Benign True Positive.
- **relay** — a localhost-only HTTP/1.1 proxy forwarding Prism's requests to the real xDome API; workaround for finding F10.
- **E-SENSOR-030** — Prism's error for "all upstream sensor targets failed" (an adapter/connectivity failure, not a Prism-internal bug).
- **Bucket A vs Bucket B** — Bucket A = sensor-spec authoring gaps (fixable by editing a TOML, no code change). Bucket B = Prism engine/adapter defects (need code changes).
- **F-numbers (F1–F12)** — findings in `prism-pql-deficiencies.md`. **A-numbers (A1–A5)** — Claroty spec gaps in `prism-xdome-spec-gaps.md`. **LIVE-DRIFT-NNN** — points where the shipped spec diverged from the real live API. **BC-/ADR-/S- IDs** — Prism's internal behavioral-contract / architecture-decision / story identifiers, cited as evidence a behavior is specified rather than accidental.

**Provenance**
- **Date:** 2026-07-20. **Workspace:** `/Users/jmagady/Dev/test-soc`.
- **Prism source** (all `crates/…:line` and `.prism*/specs/…` references): repo `/Users/jmagady/Dev/prism`, branch `develop`, commit **e116a587** (2026-07-19). Line numbers are pinned to that commit and may drift on other revisions.
- **xDome OpenAPI** (ground-truth for Claroty field/endpoint claims): xDome API 1.0.0 (OpenAPI 3.1.0), file `~/Downloads/openapi (1).json`.
- **Armis/CrowdStrike** claims are web/SDK-corroborated (no downloaded OpenAPI) — see each doc's Source line and confidence flags.
- Companion findings live beside this file in `demo-soc/findings/`; siblings are cited for extra detail only — each doc is actionable on its own.

## What Prism maps today (the "have")

`.prism/specs/armis.sensor.toml`: TWO tables — `devices`, `alerts` — both via
`GET /api/v1/search/?aql=<AQL string>`, `auth_type = "bearer_static"`,
`page_size = 25`, response extracted at `$.data.results`.

## Architectural note — Armis is AQL-centric (unlike xDome's per-endpoint POST)

Armis exposes a single unified search surface (`GET /api/v1/search/`) driven by
**AQL/ASQ** (`aql=in:<collection> <field predicates>`). "Endpoints to add" is
therefore mostly "additional `in:` COLLECTIONS reachable via the one search
endpoint," plus a few dedicated REST endpoints. There are also two API
generations (see below).

---

## 🔴 CRITICAL — Auth fidelity finding (file as a formal finding)

Prism declares `auth_type = "bearer_static"`. Against the real Armis v1 API
this is **wrong on two counts**:

1. **Wrong lifecycle.** Armis v1 is **token-exchange with short-lived
   tokens**: a long-lived *secret key* (Centrix UI → Settings → API
   Management) is POSTed (form-encoded `secret_key=…`) to
   `POST /api/v1/access_token/`, returning
   `{"success":true,"data":{"access_token":"…","expiration_utc":"<UTC ts>"}}`.
   Token is short-lived; **no refresh token** — you re-POST the secret key to
   get a fresh one. `bearer_static` treats the credential as permanent → no
   refresh path → **guaranteed failure when the token expires** (exactly the
   class of outage that hit monroe/xDome with the expired token). Needs a
   `token_exchange` auth type (secret_key form-POST → `$.data.access_token`,
   re-exchange on expiry). If the spec engine has no such auth type, that's a
   spec-ENGINE gap, not just a spec edit.
2. **Wrong header format.** v1 expects `Authorization: <raw token>` — **NO
   `Bearer` prefix** (confirmed independently: Brinqa, Google Chronicle
   ingestion script). `bearer_static` sends `Bearer <token>`, which will fail
   auth against a live tenant. Directly analogous to the xDome
   LIVE-DRIFT-001/002 items. (Spec's own comment notes the previous
   `api_key` label was retired in favor of `bearer_static` — the `api_key`
   label was actually closer to reality.)

**Two API generations** (do not conflate): Prism targets **legacy v1**
(`https://<tenant>.armis.com/api/v1/`, AQL search, raw-token header). The
dev-portal default is now **v3** (`https://api.armis.com/v3/`, true OAuth2
client-credentials → `Bearer` token + `expires_in`, structured
`POST /v3/assets/_search`). Armis states **v1/v2 are NOT deprecated** — so
Prism's v1 targeting is valid, but v3 is the strategic surface and, if
adopted, would fit a standard `oauth2_client_credentials` type.

---

## Pushdown — FULL server-side filtering (matches the xDome finding)

AQL is entirely server-side; clients are not expected to post-filter. So —
like xDome — **Prism can and should push predicates to Armis; the ceiling is
Prism-side, not the API.**

- **Time-range: confirmed** via native relative predicate
  `aql=in:alerts timeFrame:"1 Hours"` (Chronicle), absolute ranges, and a `tz`
  anchor param.
- **Field predicates: confirmed** — `in:devices name:(system)`,
  `in:alerts alertId:(57)`, `in:vulnerabilities cveId:(CVE-…)`.
- **Prism implication:** the spec already treats `aql` as verbatim-passthrough
  and marks `last_seen`/`created_at` as `INDEX`. The correct augmentation is
  to compile Prism time-window predicates into an AQL `timeFrame:"N Hours"`
  clause (NOT a `?after=` URL param — contrast CrowdStrike FQL and xDome
  `after_seconds_ago`). Pagination is offset-based (`from`/`length`, follow
  `data.next`); likely max `length` ~1000 (Chronicle default; not doc-stated)
  — raise `page_size` from the conservative 25 toward ~1000 (the inferred API
  max; validate against a live tenant).

---

## Tier 1 — core security telemetry (spec first)

| Collection / endpoint | Table (proposed) | Value |
|---|---|---|
| `in:devices` | `armis_devices` (FIX FIELDS) | asset inventory; field-name fidelity fixes below |
| `in:alerts` | `armis_alerts` (FIX FIELDS) | **keeps native `severity`** (High/Med/Low) — opposite of xDome; add array fields |
| `in:vulnerabilities` | `armis_vulnerabilities` (NEW) | **highest-value gap** — CVE-per-asset; enrichment pivot |
| `in:activity` | `armis_activity` (NEW) | behavioral/traffic events (activityUUIDs) — hunting, alert corroboration |

## Tier 2 — context / topology

`in:connections` (flows / lateral movement) · `in:users` (identity
correlation) · `in:sites` + `in:boundaries` (segmentation; also dedicated
`/api/v1/sites/`, `/boundaries/`) · `in:applications` /
`in:businessapplications` (software & business-service inventory).

## Tier 3 — infra / admin

`in:policies` (or `GET /api/v1/policies/`) · `GET /api/v1/collectors/`
(sensor/appliance health — relevant to the F6 health-probe gap) ·
`in:operatingsystems`, `in:riskfactors` (fleet posture) · custom-property
configs (`GET /api/v1/device-custom-property-configurations/`), report-results
(`GET /api/v1/report-results/{report_id}/`).

## Excluded — mutating endpoints (read-only pipeline)

`PATCH /alerts/{id}/` (status), `POST/DELETE /devices/{id}/tags/`,
`POST /devices/_bulk/`, site/boundary/report create-modify-delete. (If Prism
ever gains a governed write path, alert-status PATCH is the natural first
analyst action — not today's scope.)

---

## Field-fidelity gaps vs current spec (same class as Claroty Gap-CL-005)

**alerts** — current columns don't match the real API:
- `name` → **`title`**
- `created_at` → **`time`** (no separate updated_at on v1 alerts)
- `device_id` (scalar) → **`deviceIds` (array)**; also missing
  `connectionIds[]`, `activityUUIDs[]`, `type`, `classification`,
  `policyId`/`policyTitle`
- `policy_name` → **`policyTitle`/`policyId`**

**devices** — `ip_address`→`ipaddress`, `mac_address`→`macAddress`,
`os_name`→`operatingSystem`, `risk_score`→`riskLevel`; missing high-value
`ipv6`, `model`, `operatingSystemVersion`, `site`, `sensor`, **`purdueLevel`**
(ICS/OT), `boundaries`, `tags`.

⚠️ **DTU-vs-real caveat:** the current column names may be aligned to the DTU
clone's Rust struct, not the real API. Since the DTU is the stated
source-of-truth but has drifted from live Armis field names, DTU ↔ real API
must be reconciled before a live Armis client — otherwise the demo spec and a
live spec diverge. (Cross-ref `dtu-fidelity-gaps.md`: the demo's Armis field
names may match the DTU clone's Rust struct rather than the real Armis API, so
a spec authored against the demo will not match a live Armis tenant.)

---

## Confirmed / partial / unconfirmed

- **Confirmed:** v1 auth flow + raw-token header; `/search/` params+envelope;
  10 core `in:` collections; server-side time (`timeFrame`) + field pushdown;
  alert & device field names incl. **native severity**; v1 not deprecated.
- **Partial:** exact v1 token lifetime (only `expiration_utc`, no fixed
  duration); max `length` (~1000 inferred); `in:policies`/`sites`/`boundaries`
  as AQL collections (inferred from REST endpoints); ASQ time grammar beyond
  `timeFrame:"N Hours"`.
- **Unconfirmed:** native MITRE ATT&CK field on v1 alerts (Armis markets ICS
  ATT&CK coverage but no live v1 payload showed a `mitreAttackTechniques`
  array — do NOT spec without live validation); standalone `in:tags`
  collection (tags are a device predicate, not a collection).

## Severity contrast across sensors (running tally)

| Sensor | Native alert severity? |
|---|---|
| Claroty xDome | ❌ none (prioritize via device_risk_score in relations) |
| **Armis** | ✅ **yes** — `severity` High/Med/Low → OCSF severity |
| CrowdStrike | (audit in progress) |
