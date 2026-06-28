---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C16-1: SF-1 BUILD Prism-native Rust edge tokenizing clearing house — SS-26 DEK + aes-gcm + RocksDB-CF vault"
  - "ADR-PROP-C16-2: Technique mix keyed by RSI field class — deterministic vaulted tokenization default / FF1 FPE narrow / redaction / NER free-text"
  - "ADR-PROP-C16-3: EDGE placement immediately after OCSF normalization — forced by INV-ADS-01/D-C2-12/P-ADS-03/Option-3"
  - "ADR-PROP-C16-4: RSI abstraction with pluggable profiles over OCSF data_classification wire format; BCSI as first concrete profile"
  - "ADR-PROP-C16-5: Per-field-class token-determinism DEFAULT MATRIX — deterministic for join-needed identifiers, tunable via Compliance-Profile masking axis"
  - "ADR-PROP-C16-6: Per-tenant token vault + DEK at edge — reuse SS-26; agent path zero vault wiring"
  - "ADR-PROP-C16-7: Detokenize-at-surface via C18 RBAC — transient, never re-persisted to Central, audited per CIP-004/007"
  - "ADR-PROP-C16-8: DUAL INDEX — raw human-IR in secure zone vs masked AI/RAG; vectors are sensitive-data-class; validates C12 on-box embedding"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Day-2 out-of-band SIDE-ANALYSIS capture. Decision item C16 — entity masking /
  RSI tokenizing clearing house. Human-confirmed decisions 2026-06-27. CAPTURE ONLY.
  Does NOT modify any live spec, ADR-registry artifact (specs/architecture/), BC, story,
  STATE.md, or SESSION-HANDOFF.md. No git operation performed. Real ADR numbers and
  formal ARCH-INDEX.md rows deferred to the morph execution cycle.
  touches_no_live_artifacts: true
seeded_from:
  - research/entity-masking-tokenization-2026-06-27.md (PRIMARY — technique matrix, RSI/OCSF
    model, edge-placement reasoning, dual-index, key custody, build-vs-buy analysis, crate
    verification, regulatory-regime synthesis)
cross_refs:
  - specs/day2-design-decisions/ARCHITECTURE-DESIGN-SYSTEM.md (P-ADS-07 AI-Opaque; INV-ADS-06;
    P-ADS-02/03; P-ADS-05; PAT-ADS-07 Two-Layer-Embedded-KG+Vector; Section C.2 conformance)
  - specs/day2-design-decisions/ADR-PROP-compliance-profiles.md (D-PROF-3 masking/bulk-export
    strictness axis — C16 is its concrete enforcement mechanism)
  - specs/day2-design-decisions/ADR-PROP-rbac-depth.md (C18 ABAC masking layer + detokenize-
    at-surface enforcement point)
  - specs/day2-design-decisions/ADR-PROP-nested-tenancy.md (per-tenant key custody; Option-3
    child-keyed DEK hierarchy)
  - specs/day2-design-decisions/secret-subsystem-sketch.md (SS-26 per-tenant DEK hierarchy —
    clearing house vault reuses this infrastructure)
  - specs/day2-design-decisions/ADR-PROP-prism-context.md (C12 on-box embedding + vector store;
    dual-index pattern; PIV-C12-2)
  - specs/day2-design-decisions/ADR-PROP-soar-actions-aro.md (C15 AI recommendations over masked
    data — deterministic-token correlation for cross-entity correlation)
  - matured-vision-day2-requirements.md §16.4 (C16 decision log)
  - CLAUDE.md AD-017 (AI-opaque credentials — C16 extends this to data)
---

# ADR-PROP — Entity Masking / RSI Tokenizing Clearing House: AI-Opaque DATA (C16)

> **STATUS: DECIDED 2026-06-27 (human).** Full decision record for C16 — the entity masking /
> tokenizing clearing house that extends AD-017 (AI-opaque credentials) to AI-opaque DATA.
> CAPTURE artifact (`do_not_execute: true`). Real ADR numbers and formal ARCH-INDEX.md rows
> deferred to morph execution. Seeded from `research/entity-masking-tokenization-2026-06-27.md`.

---

## 1 — Context and Scope

AD-017 established that credentials never transit AI context; a credential-broker at the
I/O boundary resolves references before passing any information to the model. C16 extends
this same principle to **regulated sensitive data fields**: the Prism LLM/agent path
(C7 ModelBackend, C12 GraphRAG, S3 agent runtime) must operate on surrogates — masked or
tokenized representations — while authorized humans can detokenize at the surface under
C18 RBAC.

The data analogue of the credential broker is a **tokenizing clearing house** that:
1. Intercepts sensor data immediately after OCSF normalization at the edge.
2. Identifies regulated sensitive fields by their RSI classification.
3. Replaces field values with surrogates (tokens, redacted markers, or format-preserving
   encrypted equivalents) according to a technique-mix policy.
4. Ensures the agent/ModelBackend path is never wired to the vault or to a detokenize route.
5. Enables authorized analysts to reveal raw values transiently via C18 RBAC-gated calls
   to the vault service within the secure zone.

This fulfills and enforces P-ADS-07 (AI-Opaque) and INV-ADS-06 for data, complementing the
credential-level enforcement that AD-017 already provides.

---

## 2 — Decisions

### D-C16-1 — CORE: Clearing House Extends AD-017 to AI-Opaque DATA

**Decision:** C16 is the **data analogue of the AD-017 credential broker**. The clearing
house sits at the same structural position as the credential broker — between raw data
acquisition and the AI/agent path — but governs sensitive data fields rather than credentials.

The invariant is parallel:
- AD-017: credentials never transit AI context; resolved at the I/O boundary by the broker.
- C16 (this ADR): regulated sensitive data fields never transit AI context; masked/tokenized
  at the edge clearing house before any transit to Central or the AI path.

The agent/ModelBackend path has ZERO vault wiring: no route to the token vault, no credentials
for detokenization, no role grant for reveal. This is not a policy configuration — it is a
structural wiring constraint, the same way AD-017 is enforced by the absence of credential
routing to the model layer.

**Source:** research/entity-masking-tokenization-2026-06-27.md §0 Executive Summary, §6.2.

---

### D-C16-2 — SF-1 DECIDED: BUILD Prism-Native Rust Clearing House

**Decision:** BUILD a Prism-native Rust tokenizing clearing house. REJECT HashiCorp Vault
Transform Enterprise as the primary mechanism.

**Chosen architecture:**
- Token vault = a DEK-guarded **RocksDB column family** reusing the SS-26 per-tenant DEK
  hierarchy. This is the same storage substrate as the rest of Prism's hot state; no new
  storage engine is introduced.
- Tokenization default = **AES-GCM** for token vault entries (envelope encryption of
  sensitive field → token mapping). The `aes-gcm` crate (0.11.0-rc.4, 2026-05-25,
  RustCrypto) provides this; it is the same primitive already in the SS-26 stack.
- FPE OPTIONAL (narrow) = **FF1** via the `fpe` crate (0.6.1, 2023-04-13) for the small
  subset of fields where a format-valid surrogate is required by a downstream consumer AND
  the field domain satisfies the NIST SP 800-38G minimum of 10^6 values.

**Considered and rejected:**

| Option | Rejection rationale |
|--------|---------------------|
| HashiCorp Vault Transform Enterprise | Licensed dependency; heavyweight external process; duplicates SS-26 per-tenant DEK custody already designed; air-gap deployment adds operational complexity for a capability Prism already owns the primitives for. |
| SaaS tokenization vaults (Skyflow, VGS) | SaaS-centric; vendor custody; poor fit for air-gap / BYOC / NERC-CIP deployments. |
| Self-hosted Vault Transform (OSS) | Transform is Enterprise-only; OSS Vault does not include FPE/tokenization transforms. |

**Build cost acceptance:** the Prism-native path is architecturally coherent with SS-26 and
avoids an additional runtime dependency. The clearing house is a new module but leverages
existing DEK hierarchy, RocksDB, and AES-GCM primitives.

**Source:** research §2.5 (vendor landscape), §2.6 (crate verification), §6 (key custody).

---

### D-C16-3 — Technique Mix Keyed by RSI Field Class

**Decision:** No single masking technique is universally correct. The clearing house applies
a **technique mix keyed by RSI field class** (driven by the active Compliance Profile's
masking strictness axis, D-PROF-3 in ADR-PROP-compliance-profiles.md):

| RSI field class | Default technique | Rationale |
|-----------------|-------------------|-----------|
| High-risk identifiers: IP address, hostname, asset_id, firewall-rule_id, BCSI configs | **Deterministic vaulted tokenization** | Joins preserved across tables and time; token mathematically unlinked to plaintext (vault-lookup only, no cipher inversion); vault is the isolation boundary. The default for most sensitive fields. |
| Fields where a downstream consumer requires a format-valid surrogate AND field domain ≥ 10^6 | **FF1 FPE** (optional, narrow) | Format preservation needed (e.g., for downstream schema validators); stateless key-based; acceptable only when vaulted token would break a consumer contract. Requires explicit RSI profile declaration. |
| Fields the agent never legitimately needs (e.g., raw BCSI config text, specific credential-adjacent fields) | **Full redaction** (irreversible) | Agent has no legitimate use case; redaction eliminates detokenization attack surface entirely. |
| Free-text fields (alert descriptions, log message bodies, incident notes) | **Presidio-style NER** span detection → tokenize or redact spans | Structured fields are OCSF-typed (major scope simplification); NER reserved for unstructured text where OCSF schema does not pre-classify the sensitive content. |

**Technique mix is tunable per RSI field class via the Compliance Profile masking axis**
(D-PROF-3 `[settings.masking]`). The profile may tighten (e.g., lock a BCSI-config field
to `hard_redact` in the `nerc-cip` profile) but may never loosen below the INV-ADS-06 floor.

**Source:** research §2 (technique matrix), §8 (recommended architecture item 1).

---

### D-C16-4 — EDGE PLACEMENT DECIDED (Forced, Not a Free Choice)

**Decision:** The clearing house runs at the **EDGE, immediately after OCSF normalization**,
before any transit to the Option-3 tenant-keyed Central cache.

This placement is **FORCED** by the following architectural constraints — it is NOT a design
preference that can be traded against implementation convenience:

1. **INV-ADS-01 (raw sensor data never at Central):** OCSF normalization converts raw vendor
   API schemas to OCSF; masking/tokenization immediately after normalization ensures that
   sensitive identifier VALUES (IP, hostname, asset_id) also never transit to Central. INV-ADS-01
   covers DATA FORMAT (OCSF vs raw); C16 extends this protection to DATA CONTENT.

2. **D-C2-12 HARD INVARIANT** (`ADR-PROP-satellite-mesh.md`): only OCSF-normalized results
   transit the conduit. Masking at the edge makes the masking boundary coincide with the
   conduit boundary — raw sensitive identifiers never enter the conduit in any form.

3. **P-ADS-03 (Derived-Results-Only-At-Central):** OCSF-normalized data is NOT PII-safe
   (hostnames, IPs, user accounts are OCSF first-class fields). Masking at the edge makes
   Central hold tokenized/derived records only; a Central breach cannot recover raw identifier
   values.

4. **Option-3 (Tenant-Keyed-Central-Persistence, P-ADS-04):** Central holds the tenant-keyed
   encrypted cache of derived results. Under edge masking, Central NEVER holds the raw values
   to mask at the surface — central-side analytics over raw identifiers is structurally
   impossible (by design), not merely governed by policy.

**Accepted cost (PIV-C16-001):** Central operates on surrogates + OCSF-normalized enums/
numerics + deterministic-token joins. Central-side analytics requiring raw values MUST push
the computation down to the edge/secure zone. This is a real constraint, not a blocker —
most cross-tenant analytics (anomaly detection, compliance scoring) work on behavioral
features and OCSF-enumerated values, not literal identifiers.

**Surface-boundary masking (rejected):** Central holding raw BCSI/topology/configs and
masking only at the ModelBackend input boundary would violate INV-ADS-01, P-ADS-03, and
Option-3 simultaneously. This option is a conformance failure, not a tradeoff.

**Source:** research §6.1 (decisive lean and reasoning chain), §8 item 2.

---

### D-C16-5 — RSI Classification: Declarative Tagging over OCSF Schema

**Decision:** Masking field selection uses **declarative RSI tagging over the OCSF schema**
rather than a general-purpose DLP classifier for structured fields.

**Why this is a major scope simplification:** Prism normalizes ALL sensor data to OCSF at
the adapter boundary (P-ADS-08, INV-ADS-07). The data-type axis for structured sensor fields
is ALREADY KNOWN from the OCSF schema — field semantic types (IP address, hostname, user
account, asset ID, config body) are encoded in the OCSF object model. Prism does NOT need
Presidio-grade NER for structured fields; it needs a declarative mapping of "which OCSF
field classes are RSI under which Compliance Profile."

**RSI abstraction (internal):** The **"Regulated Sensitive Information (RSI)"** abstraction
is Prism's internal model. It is NOT an industry-standard term (caveat: the field uses
"data classification taxonomy," "sensitive data elements," "protected data classes" — no
single adopted term exists). RSI is the internal abstraction; the wire/interchange
representation uses OCSF-native format.

**OCSF data_classification as wire format:** OCSF v1.2.0 (released 2024-04-23, PR #998)
ships a real `data_classification` profile with `confidentiality` and `data_type` attributes
applied to database, file, email, web_resource, and related objects. Current OCSF version
is v1.8.0 (2026-03-18). Prism expresses RSI classification over OCSF's existing
`data_classification.confidentiality` / `data_type` attributes as the interchange
representation, avoiding a proprietary wire format that would fork the ecosystem.

**Pluggable profiles:** The RSI model separates intrinsic field properties (data_type ×
confidentiality/privacy_category) from regulatory profiles (rules that interpret those
properties per regime). Profile layer: **BCSI** (NERC CIP-011), **PII** (GDPR), **PHI**
(HIPAA), **PCI** (PAN). A field may be claimed by multiple profiles in different tenant
contexts (a national ID is PHI in a HIPAA tenant, PII in a GDPR tenant); profiles interpret
the same classification, they do not re-tag.

**BCSI as first concrete profile** (consistent with C20 NERC-CIP research). The `profile:nerc-cip`
Compliance Profile activates BCSI masking posture via the `[settings.masking]` axis.

**NER reserved for free-text:** Presidio-style NER detection applies only to unstructured
free-text fields (alert descriptions, log message bodies) where OCSF schema does not
pre-classify the content.

**Source:** research §3 (classification/tagging model), §4 (OCSF data_classification
CORRECTION — it is shipped, not proposed).

---

### D-C16-6 — SF-5 DECIDED: Name = RSI; BCSI = First Profile; OCSF = Wire Format

**Decision (SF-5):**
- Internal abstraction name: **"Regulated Sensitive Information (RSI)"**
- First concrete profile: **BCSI** (consistent with C20 NERC-CIP research)
- Wire/interchange representation: **OCSF `data_classification`** (native, not proprietary)

Caveat recorded: RSI is NOT an industry-standard term. The decision is to use it internally
with OCSF-native wire format so external interoperability is preserved. If external naming
alignment becomes important, "Protected Data Classes" or direct OCSF `data_classification`
terminology are the runner-up options.

**Source:** research §8 (Universal-name recommendation).

---

### D-C16-7 — SF-3 DECIDED: Per-Field-Class Token-Determinism Default Matrix

**Decision (SF-3):** The token-determinism default matrix:

| Field class | Token mode | Rationale |
|-------------|-----------|-----------|
| Join-critical identifiers (asset_id, hostname, IP correlated across sensors/time) | **Deterministic** (same plaintext → same token per tenant) | Required for cross-table correlation (C15 AI recommendations, C12 entity graph, C11 intel matching via deterministic-token joins). Frequency/linkage-attack exposure documented as D-C16-7-TRADEOFF below. |
| Audit-trail-only fields (one-time event IDs, timestamps) | **Randomized** (token is per-event) | No join requirement; maximizes privacy. |
| Fields the agent never needs | **Redaction** (irreversible) | No token issued; eliminates detokenization surface entirely. |

**Determinism tradeoff (documented — D-C16-7-TRADEOFF):** Deterministic tokens expose
frequency/linkage-attack surface — a frequent token reveals a frequent identifier, and
cross-table token occurrence can reconstruct behavioral patterns without the raw value.
Mitigations:
1. Per-tenant keyspace (deterministic tokens for tenant A differ from tenant B even for the
   same plaintext) — already enforced by SS-26 per-tenant DEK isolation.
2. Periodic re-tokenization capability (OQ-C16-002) — supported architecturally by the vault
   CF design; scheduling policy is an open question.

**Tunable via Compliance Profile masking axis (D-PROF-3):** The `[settings.masking]`
per-field-class determinism setting is a tunable parameter within profile-declared bounds.
The `nerc-cip` profile may lock specific field classes to randomized tokens or to redaction
(tighten-only); it cannot loosen a more restrictive default.

**Source:** research §2.2 (deterministic tokenization), §8 item 1.

---

### D-C16-8 — KEY CUSTODY: Per-Tenant Token Vault + DEK at Edge

**Decision:** The token vault and DEK live at/near the EDGE (or in the customer's highest-trust
on-prem SOC cluster), NOT at Central. This reuses and extends the SS-26 per-tenant DEK
hierarchy.

**Architecture:**
- **Token vault:** a RocksDB column family keyed by `(tenant_id, token_id)` (or `(tenant_id,
  plaintext_hmac)` for deterministic lookup), DEK-wrapped per SS-26.
- **Per-tenant DEK:** same DEK hierarchy as all other SS-26-protected state. The clearing
  house introduces no new key management surface.
- **Central holds:** token values (ciphertext references) + masked OCSF records + per-tenant-
  DEK-encrypted derived results (P-ADS-04). Central has NO DEK for the token vault CF.
  Central can correlate via deterministic tokens; it cannot detokenize.
- **Authority separation:** the team owning edge + vault (customer security / KMS admin) holds
  DEKs; the central AI/analytics operator never sees raw values and never holds DEKs. This is
  the critical MSSP trust invariant for sensitive-data workloads.

**Agent path: ZERO vault wiring.** The ModelBackend / agent runtime path has no vault route,
no credential for the token CF, no detokenize role. This is structural (absent wiring), not
policy-governed (blocked wiring). This is the data analogue of AD-017.

**Source:** research §6.2 (token vault and DEK custody), §8 item 5.

---

### D-C16-9 — DETOKENIZE-AT-SURFACE via C18 RBAC

**Decision:** Authorized analysts may reveal masked field values via a dedicated detokenize
call. The path:
1. Analyst requests reveal of a masked field in the Central UI.
2. Central UI calls the **vault service inside the secure zone** (NOT Central itself; the call
   is proxied to the edge/satellite holding the vault).
3. The vault service gate: analyst identity (from per-connection OAuth, ADR-051 in
   `ADR-PROP-central-deployment-access-layer.md`) × RBAC role × tenant × token class ×
   Compliance Profile masking posture (C18 ABAC masking layer enforcement point).
4. On grant: raw value returned TRANSIENTLY to the client session.
5. Raw value is NEVER re-persisted to Central long-term storage.
6. The reveal event is AUDITED per CIP-004/007 ("who viewed BCSI") for BCSI-profile tenants;
   per relevant audit requirement for other profiles.

**Binding to C18:** The ABAC masking layer in C18's authz engine (D-C18-3) is the Policy
Enforcement Point for detokenize grants. Token class sensitivity tags are ABAC attributes;
the authz engine evaluates `analyst × sensitivity:RSI-field-class × tenant` at reveal time.
Detokenize is a CAPABILITIES grant on the analyst's authenticated identity, scoped per token
class and per tenant. The grant is RBAC-scoped, NEVER implicit from role membership alone.

**Source:** research §6.3 (detokenize-at-surface), `ADR-PROP-rbac-depth.md` D-C18-3.

---

### D-C16-10 — DUAL INDEX: Raw Human-IR vs Masked AI/RAG

**Decision (SF-4):** Deploy **two separate indexes** for embedding and similarity search:

| Index | Content | Access | Rationale |
|-------|---------|--------|-----------|
| **Human-IR index** | Raw text + raw embeddings (embed raw locally at edge) | Inside secure zone; NO agent access; human investigation + machine correlation only | Preserves semantic search fidelity for authorized analysts; raw embeddings embedded on-box (PIV-C12-2) per C12. |
| **AI/RAG index** | Masked view: sensitive identifiers → deterministic tokens, ultra-high-risk fields omitted, contextual text preserved (OCSF behavior/topology role, enums, numerics) | Agent may query | Agent receives only the masked view; joins work via deterministic tokens; literal-identifier semantic search is deliberately absent from the agent path. |

**Vectors are a SENSITIVE-DATA class.** Embedding-inversion attacks reconstruct 50–90%+
of source text from embeddings, including names and identifiers. Therefore:
- Vector stores require: encryption at rest (DEK-protected per P-ADS-04), per-tenant
  isolation (P-ADS-06), RBAC-gated access, audit trail.
- Vectors are treated with the same custody classification as the token vault.
- Raw embedding stays inside the edge trust boundary (C12 PIV-C12-2 on-box embedding
  invariant is load-bearing, not optional — it is the primary defense against embedding
  exfiltration).

**C12 on-box embedding validated:** C12's instinct to embed raw locally (fastembed/ort
in-process) is confirmed as the correct and required approach. An external embedding
service call for raw sensor text would violate both P-ADS-07 (AI-opaque) and this
decision's vector-sensitive-data classification.

**Mask-then-embed degradation (acceptable):** Masking high-entropy identifiers destroys
their lexical semantics in the AI/RAG index, but the useful security signal for SOC
workloads is in non-identifying context (behavior patterns, topology roles, OCSF
classification enums). The AI path operates on patterns, not literal identifiers.
Agents may be policy-blocked from queries targeting literal raw identifiers
("what happened to IP 10.0.0.5" → blocked; "what behavior patterns are associated with
this asset token" → permitted).

**Source:** research §7 (embedding tension analysis), §8 item 4.

---

## 3 — Invariants

| ID | Invariant |
|----|-----------|
| **PIV-C16-001** | Raw sensitive-field values NEVER transit the edge-to-Central conduit. The clearing house runs at the edge immediately after OCSF normalization. Placement is forced by INV-ADS-01 + D-C2-12 + P-ADS-03 + Option-3; it is not configurable. |
| **PIV-C16-002** | The agent/ModelBackend path is NEVER wired to the token vault, detokenize route, or any credential for the token CF. This is a structural wiring absence, not a policy restriction. |
| **PIV-C16-003** | Detokenized values are returned TRANSIENTLY to the analyst client session only; they are NEVER re-persisted to Central storage. Every detokenize call is audited. |
| **PIV-C16-004** | Vector stores (human-IR and AI/RAG) are sensitive-data-class stores: DEK-protected, per-tenant isolated, RBAC-gated, audited. Raw embeddings are embedded on-box (PIV-C12-2) and never shipped to an external embedding service. |
| **PIV-C16-005** | FPE (FF1) is an OPTIONAL technique, narrowly scoped to fields where format-valid surrogates are required AND field domain ≥ 10^6. Tokenization-default is mandatory; FPE may not substitute for the vault as the primary isolation boundary. |
| **PIV-C16-006** | Per-tenant token vault + DEK reside at or near the edge secure zone (reusing SS-26 hierarchy). Central holds token values only; it CANNOT detokenize without the edge vault. |

---

## 4 — Open Questions

| ID | Question | Owner | Priority |
|----|----------|-------|----------|
| **OQ-C16-001** | `fpe` crate maintenance risk: version 0.6.1 (2023-04-13), FF1-only (no FF3-1), stale ~3yr, pins `aes ^0.8` / `cipher ^0.4`. If FPE is exercised, Prism must pin to `fpe`'s exact `cipher ^0.4` constraint and reconcile against any `aes-gcm` 0.11.x upgrade path. Mitigation: make FPE genuinely optional (clearing house defaults to tokenization; FPE only activates when RSI profile explicitly requires it). Resolution: architect at morph — verify that `aes-gcm 0.11` and `fpe 0.6.1` can coexist in the workspace (check their transitive `cipher` / `aes` version constraints). | architect at morph | P1 |
| **OQ-C16-002** | Periodic re-tokenization scheduling: deterministic tokens expose linkage/frequency attack surface. Re-tokenizing (issuing new tokens while maintaining the mapping for backward join) reduces this. What is the default re-tokenization schedule, and how does it interact with Central's derived corpus (which holds token-keyed results)? Recommendation: defer periodic re-tokenization to a follow-on epic; v1 ships with per-tenant keyspace as the primary mitigation. | architect + product-owner at morph | P2 |
| **OQ-C16-003** | OCSF `data_classification.confidentiality` enum values: the research confirmed the profile exists (OCSF v1.2.0, PR #998) but did not enumerate the exact `confidentiality` enum values against schema.ocsf.io. Before these values are made load-bearing in the RSI tagging schema, the exact enum must be confirmed via schema.ocsf.io or the OCSF GitHub schema directory. Resolution: research-agent at morph. | research-agent at morph | P1 |
| **OQ-C16-004** | HIPAA Expert-Determination in-product mode: if Prism targets healthcare tenants, does the product ship an Expert-Determination-ready tokenization mode (with documented risk methodology for the PHI profile)? Current scope: BCSI is the first RSI profile; PII-GDPR, PCI, and PHI are defined as future profiles. HIPAA Expert-Determination documentation is a compliance deliverable for the PHI profile epic, not a v1 item. Record as OQ for that future decision. | product-owner at healthcare-tenant decision point | P3 |
| **OQ-C16-005** | NER library selection for free-text fields: Presidio is the reference architecture (Python) but Prism is Rust-native. Options: (a) sidecar Python Presidio process (cross-process NER); (b) Rust NLP bindings (llm-based via `candle` or ONNX via `ort`); (c) rule-based span detection (regex + pattern, no ML). This is a Phase-2+ decision; v1 BCSI profile targets structured OCSF fields only (NER not required for Phase 1). | architect at E-RSI-CLEARING-HOUSE-001 decomp | P2 |

---

## 5 — Regulatory Cross-Regime Synthesis

The research confirms that **reversible tokenization is permitted under every regime that
matters to the initial target verticals**, but each regime constrains custody:

| Regime | Tokenization permitted? | Custody requirement |
|--------|------------------------|---------------------|
| **GDPR** | Yes — tokenization = textbook pseudonymisation (Art 4(5)). Data remains in-scope; GDPR not exited by tokenization alone. | Keys/vault must be kept separately under technical/org measures. Per-tenant edge vault satisfies. |
| **HIPAA** | Yes, conditionally — token must not be derivable from the identifier (Safe Harbor); Expert-Determination can bless tokenization-with-separate-mapping if recipient risk is "very small." | Vault/mapping must remain with the regulated entity; separate from recipients. Edge custody satisfies. |
| **PCI DSS** | Yes — explicitly encouraged for scope reduction. Vault/detokenization path stays in scope; systems handling non-reversible tokens may be scoped out. | Per-tenant edge vault in scope; Central token-only path potentially out of scope. |
| **NERC CIP-011-3** | Yes — tokenizing BCSI in results sent to third parties is consistent with "prevent unauthorized access" provided the entity holds the mapping/keys. CIP-011-3 enforces entity-held-key + zero-plaintext-access model. | Entity holds keys (per-tenant edge DEK = customer/entity controlled). Central = zero-plaintext access. ✓ |

**Cross-regime conclusion:** The per-tenant edge vault + DEK design (D-C16-8) satisfies
the strictest custody requirement (NERC CIP-011-3 entity-held-key / zero-plaintext-access)
and generalizes correctly to all four regimes.

**Source:** research §5 (regulatory cross-regime synthesis).

---

## 6 — Proposed Epic

**E-RSI-CLEARING-HOUSE-001** (PROPOSED — not yet in STORY-INDEX). Implement the edge
tokenizing clearing house.

Scope:
- RSI field tagging schema over OCSF `data_classification` (structured fields; BCSI profile first)
- Token vault: new RocksDB CF (`token_vault`) under SS-26 per-tenant DEK hierarchy
- Tokenization engine: deterministic vaulted tokenization (AES-GCM + keyed PRF) as default
- FF1 FPE module: optional, activated by RSI profile declaration (gated on OQ-C16-001 resolution)
- Full-redaction path for irreversible field classes
- Clearing house placement in OCSF normalization pipeline (after normalization, before conduit transit)
- Per-field-class determinism matrix (per D-C16-7); tunable via Compliance Profile masking axis
- Detokenize-at-surface API (vault service endpoint, C18 RBAC-gated, transient, audited)
- Dual-index integration with C12 vector store (human-IR index with raw embeddings; AI/RAG index
  with masked view)
- Audit trail for detokenize events (CIP-004/007 audit binding for BCSI profile)
- `aes-gcm` crate pinning + `fpe` crate optional dependency (OQ-C16-001 resolution required first)

PROPOSED. Not registered in STORY-INDEX. Registration gated on morph execution.

---

## 7 — Cross-Wiring

| Feature | Cross-wire point |
|---------|-----------------|
| **AD-017 (AI-opaque credentials)** | C16 is the DATA analogue. The structural pattern is identical: broker at I/O boundary, zero wiring to the agent path, reference-based model. |
| **C18 RBAC Depth** | ABAC masking layer (D-C18-3) is the PEP for detokenize grants. Token class sensitivity tags are ABAC attributes evaluated at reveal time. C18 SF-2 (per-record PII unmask approval) is the workflow expression of this gate. |
| **C12 Prism Context** | C12 on-box embedding (PIV-C12-2) is validated as load-bearing by D-C16-10 (vector-sensitive-data classification). PAT-ADS-07 Two-Layer-Embedded-KG+Vector gains the dual-index layer from this decision. |
| **C15 AI Recommendations / ARO** | C15 computes recommendations over masked data. Deterministic-token joins enable cross-entity correlation across sensors without raw identifier transit. The masked AI/RAG index (D-C16-10) is the C15 query substrate. |
| **C19 Nested Tenancy** | Per-tenant DEK isolation from SS-26 / C19 SF-4 is the key-custody foundation. Token vault CFs are per-tenant-keyed by construction. Cross-tenant token lookup is structurally prevented by per-tenant DEK partitioning. |
| **C20 NERC-CIP** | BCSI is the first RSI profile (D-C16-6). The `profile:nerc-cip` Compliance Profile activates BCSI masking posture via `[settings.masking]`. CIP-011-3 entity-held-key model is satisfied by per-tenant edge vault (D-C16-8). |
| **Option-3 (P-ADS-04)** | Edge placement (D-C16-4) is forced by Option-3: Central holds tokenized/derived records under per-tenant DEK; raw sensitive identifiers never enter the Central cache. |
| **SS-26 Secret Broker** | Token vault CF reuses the SS-26 per-tenant DEK hierarchy. No new key management surface. Clearing house is an extension of the SS-26 architecture, not a parallel system. |
| **Compliance Profiles (ADR-PROP-compliance-profiles.md)** | C16 is the concrete mechanism for the `[settings.masking]` axis in D-PROF-3. The profile controls masking strictness (determinism, redaction policy, bulk-export posture) within C16's enforcement engine. |
| **P-ADS-07 / INV-ADS-06 (AI-Opaque)** | C16 is the ENFORCEMENT MECHANISM for P-ADS-07 and INV-ADS-06 applied to DATA. Every ADR-PROP conformance check on P-ADS-07 "Do AI/ML components receive feature vectors or masked data only?" is YES because of C16. |

---

## 8 — ADS Conformance Checklist

```
CONFORMANCE CHECKLIST — ADR-PROP-entity-masking.md (C16) — 2026-06-27

P-ADS-01: Central-Sole-Surface
  [YES] No new user-interaction surface introduced. Detokenize calls originate at the
        Central UI, authenticated via Central session, proxied to the edge vault service.
        Central remains the sole user-interaction surface; the satellite/edge vault is
        headless (a service, not a UI).

P-ADS-02: Operator-Zero-Access-At-Rest
  [YES] Token vault is DEK-protected under SS-26 per-tenant DEK (customer-held CMEK).
        The operator holds the encrypted token vault CF at the edge; no DEK is held
        at Central or by the operator infrastructure. Operator zero-access enforced.

P-ADS-03: Derived-Results-Only-At-Central
  [YES] Edge placement (D-C16-4) ensures only masked/tokenized records transit the
        conduit to Central. Raw sensitive identifiers (IPs, hostnames, BCSI configs)
        never reach Central. This is an unconditional structural constraint.
  [YES] No opt-in path for raw identifier transit to Central is introduced by C16.
        (C16 is a tightening mechanism, not a relaxation path.)

P-ADS-04: Tenant-Keyed-Central-Persistence
  [YES] Token vault CF uses RocksDB (edge) under SS-26 per-tenant DEK. Central-side
        records holding tokenized results are encrypted under the same per-tenant DEK
        hierarchy (P-ADS-04 / Option-3 lock). No PostgreSQL for token vault or masked
        result cache.
  [YES] C16 stores masked RECORDS at Central (derived outputs) and token MAPPING at
        the edge (not at Central). This is output caching (PAT-ADS-02), not input
        snapshotting (OQ-C8-DATASNAPSHOT) — the distinction is maintained.

P-ADS-06: Per-Tenant-Isolation
  [YES] Per-tenant token vault CF: all token mappings are tenant-partitioned by
        (tenant_id, token_id) key structure. SS-26 per-tenant DEK ensures cross-tenant
        token lookup is cryptographically prevented.
  [YES] Dual-index (D-C16-10): per-tenant isolation enforced on both human-IR and
        AI/RAG vector indexes (PIV-C12-5 in ADR-PROP-prism-context.md).

P-ADS-07: AI-Opaque
  [YES] Agent/ModelBackend path has ZERO vault wiring (D-C16-8, PIV-C16-002).
        This is the structural enforcement of P-ADS-07 for data.
  [YES] AI/RAG index exposes only masked view (D-C16-10). Raw embeddings embedded
        on-box; raw text never shipped to external embedding service.
  [YES] Credentials resolved outside AI loop (unchanged from AD-017; C16 adds the
        DATA layer to the existing credential-layer enforcement).

P-ADS-08: OCSF-Normalize-At-Boundary
  [YES] Clearing house runs AFTER OCSF normalization. It does not interfere with or
        bypass the normalization chokepoint; it operates on the already-normalized
        OCSF record. OCSF normalization boundary is unchanged.

P-ADS-09: Config-DB-Authoritative
  [YES] RSI field tagging schema (which OCSF fields are RSI under which profile) is
        configuration authored at Central DB, pushed as a signed bundle to the edge
        clearing house (PAT-ADS-03). No edge-local config authoring of masking rules.

P-ADS-10: Idempotent-Gated-Actions
  [YES] Detokenize calls pass through C18 ABAC gate (analyst × token class × tenant).
        The gate is system-configured (Compliance Profile masking axis), not
        agent-configured.
  [YES] Masking is deterministic per RSI field class: same input → same masked output
        for deterministic tokens; same input → `[REDACTED]` for redacted fields.
        No non-idempotent masking behavior.

INV-ADS check:
  [YES] INV-ADS-01: Raw sensor data never at Central — C16 enforces this for data content
                    (not just format) via edge placement.
  [YES] INV-ADS-02: Operator zero-access at rest — token vault DEK is customer-held (SS-26).
  [YES] INV-ADS-03: Per-tenant isolation — per-tenant token CF partitioning + per-tenant DEK.
  [YES] INV-ADS-04: Config authored only at Central — RSI tagging schema is Central-DB-authoritative.
  [YES] INV-ADS-05: Actions gated — detokenize is gated (C18 ABAC + Compliance Profile posture).
  [YES] INV-ADS-06: AI-opaque — C16 IS the enforcement mechanism for this invariant for data.
  [YES] INV-ADS-07: OCSF normalization unaffected — clearing house runs after normalization boundary.
  [YES] INV-ADS-08: Air-gap valid — token vault is a RocksDB CF at the edge; no internet-
                    dependent detokenization path; SoftwareKms (SS-26 HD-1) is the DEK backend.
  [YES] INV-ADS-09: Decision-level audit — every detokenize grant (subject, token class, tenant,
                    policy version, outcome) is logged per C18 D-C18-7 decision-level audit
                    requirement; BCSI profile adds CIP-004/007 audit binding.
```

All checklist items PASS. C16 is the concrete enforcement mechanism for P-ADS-07 and
INV-ADS-06 applied to DATA — the conformance checklist result for P-ADS-07 across all
other ADR-PROPs is YES because of this clearing house.

---

## 9 — Decision Provenance

| Decision ID | Topic | Human decision |
|-------------|-------|----------------|
| D-C16-1 | CORE: extends AD-017 to AI-opaque DATA | DECIDED — clearing house is the data analogue of the credential broker |
| D-C16-2 | SF-1: build vs buy | DECIDED — BUILD Prism-native Rust clearing house; Vault Transform Enterprise rejected |
| D-C16-3 | Technique mix | DECIDED — deterministic vaulted tokenization default / FF1 FPE narrow / redaction / NER free-text; keyed by RSI field class |
| D-C16-4 | Placement | DECIDED — EDGE, immediately after OCSF normalization; FORCED by INV-ADS-01/D-C2-12/P-ADS-03/Option-3 |
| D-C16-5 | Classification mechanism | DECIDED — declarative RSI tagging over OCSF schema; Presidio reserved for free-text |
| D-C16-6 | SF-5: naming | DECIDED — RSI internal abstraction; BCSI first profile; OCSF data_classification wire format |
| D-C16-7 | SF-3: determinism matrix | DECIDED — deterministic for join-critical identifiers; randomized for audit-only; redaction for agent-never-needs; tunable via Compliance Profile |
| D-C16-8 | Key custody | DECIDED — per-tenant token vault + DEK at edge; SS-26 reuse; Central holds tokens only; agent zero vault wiring |
| D-C16-9 | Detokenize-at-surface | DECIDED — C18 RBAC ABAC gate; transient; never re-persisted to Central; audited |
| D-C16-10 | SF-4: dual index + vectors | DECIDED — DUAL INDEX (raw human-IR in secure zone vs masked AI/RAG); vectors = sensitive-data-class; C12 on-box embedding validated as load-bearing |
