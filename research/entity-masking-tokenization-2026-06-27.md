---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
topic_slug: entity-masking-tokenization
provenance: >
  Cited research pass for Prism Day-2 SIDE-ANALYSIS item C16 — entity masking /
  tokenizing clearing house (AI never sees sensitive data). CAPTURE/research only.
  Modifies no live spec/BC/ADR/STATE.md/SESSION-HANDOFF.md. No git operation performed.
  Feeds a DISCUSSION on extending AD-017 (AI-opaque credentials) to AI-opaque DATA.
traces_to:
  - matured-vision day2 SIDE-ANALYSIS C16 (entity masking / tokenizing clearing house)
  - project memory AD-017 (AI-opaque credentials) → extend to data
  - .factory/research/nerc-cip-support-2026-06-27.md (C20: BCSI as first RSI profile)
  - C12 (entity model + on-box embeddings); C15 (AI recommendations over masked data)
  - Option-3 (tenant-keyed central cache); C18 (RBAC); C19 (nested tenancy)
  - Architecture Design System: P-ADS-07 AI-Opaque, Operator-Zero-Access, Central-Sole-Surface
  - Edge-Computes / Central-Surfaces split
---

# Entity Masking & Tokenizing Clearing House — Cited Research (C16 Side-Analysis)

> PROPOSED discussion input. Status: capture. Not a spec, not an ADR, not a vision change.
> Section numbering is internal to this document. Everything is "as of 2026-06-27";
> standards versions and crate versions change.

## 0. Executive Summary (the answer up front)

C16 extends AD-017's existing **AI-opaque credentials** invariant to **AI-opaque DATA**: a *clearing house* that masks/tokenizes regulated sensitive fields so the LLM/agent (C7 ModelBackend) operates on surrogates while authorized humans can still detokenize at the surface under RBAC.

Five load-bearing findings drive the recommendation:

1. **No single technique fits everything; the answer is a technique *mix* keyed by field class.** Reversible **tokenization** (vaulted, deterministic where joins are needed) is the right default for high-risk identifiers (IPs, hostnames, asset IDs, firewall-rule IDs). **Format-Preserving Encryption (FF1)** is the right tool only where the consumer needs a format-valid surrogate AND the field domain ≥ 10^6 (NIST minimum). **Redaction** (irreversible) is right for fields the agent never legitimately needs. [pass-1; NIST SP 800-38G]

2. **The clearing house belongs at the EDGE, immediately after OCSF normalization — not at the surfacing boundary.** This makes the *masking boundary coincide with the true data-sensitivity boundary*: raw BCSI/topology/configs never transit to the tenant-keyed central cache (Option-3) at all. Central holds masked surrogates + derived features only; a central breach cannot recover raw values. This is the architecturally decisive lean and it reconciles cleanly with Edge-Computes / Central-Surfaces. [reason-pass T2]

3. **Embeddings are NOT a safe masking output — embedding-inversion attacks reconstruct 50–90%+ of source text including names/diagnoses/identifiers.** Therefore vectors must be treated as sensitive data, and the C12 "on-box embedding" instinct is *correct and load-bearing*: embed RAW locally inside the edge trust boundary, never ship raw to an external embedding service, and surface only masked text to the agent. Mask-then-embed degrades semantics but is acceptable for the AI-facing index because the useful security signal is mostly in non-identifying *context* (behavior, topology role), not the literal identifier. [reason-pass T1; refs 6,7,9,10,12,15]

4. **The universal name "Regulated Sensitive Information (RSI)" is defensible but NOT an industry-standard term — there is no single adopted term.** The functional equivalents in the wild are "data classification taxonomy," "sensitive data elements," "protected data classes," and (newly) OCSF's **`data_classification` profile** (REAL and shipped in OCSF v1.2.0, 2024-04-23 — see §4, this corrects a stale "proposal-only" claim). RSI-with-pluggable-profiles (BCSI / PII-GDPR / PHI-HIPAA / PCI) is the right *internal* model; we should expose it over OCSF's existing `data_classification.confidentiality` / `data_type` attributes rather than invent a wire format. [pass-2; OCSF releases]

5. **Reversible tokenization is permitted under every regime that matters to us, but each regime constrains custody differently.** PCI *encourages* reversible tokenization (scope reduction); GDPR treats it as *pseudonymisation* (data stays in-scope, keys kept separate); HIPAA accepts it only via Safe-Harbor (token must not be PHI-derived) or Expert-Determination (risk "very small" for the recipient); NERC CIP-011 doesn't name tokenization but its data-centric/entity-held-key model is exactly satisfied by per-tenant DEK custody at the edge. [pass-2; §5]

**LEANS (full detail §8):** edge-placed clearing house + tokenization-default/FPE-where-format-needed/redaction-where-never-needed; per-tenant token vault + DEK living in the edge/secure-zone (NOT central, NOT reachable by the agent); on-box raw-embedding with dual human/AI indexes; RSI internal abstraction surfaced via OCSF `data_classification`; detokenize-at-surface only via C18 RBAC. Genuine human sub-forks in §9.

---

## 1. Read-coverage & sourcing honesty

- **Read in-repo:** `RESEARCH-INDEX.md`; `nerc-cip-support-2026-06-27.md` (full §0–7, the C20 RSI recommendation + BCSI definition + CIP-011-3 entity-held-key model); `central-deployment-access-layer-2026-06-26.md` (SS-26 secret broker, AD-017, tenant model, Streamable-HTTP central plane).
- **NOT read (out of scope for a capture pass):** full BC bodies; SS-26 §4–7 DEK hierarchy detail (cited from the central-deployment research's summary).
- **External evidence:**
  - 2× `perplexity_research` (`sonar-deep-research`, reasoning_effort=high) — techniques+vendors+Rust crates (~92K chars, read ~70%); classification+regulatory regimes (~92K chars, read ~72%). Both outputs exceeded the token cap; I read the substantive technique/regulatory sections in full and used Grep to confirm the air-gap/crate-conclusion sections existed. The §8/§9 of file-1 (air-gap deployment patterns) and the BCSI tail of file-2 were NOT fully parsed — but those topics are covered by the in-repo NERC research and direct crates.io verification, which is the stronger evidence path.
  - 1× `perplexity_reason` — the two Prism-specific architectural tensions (embedding-vs-masking; clearing-house placement), with embedding-inversion citations.
  - **crates.io API (direct, authoritative)** — version verification for `fpe`, `aes`, `aes-gcm`.
  - **WebSearch + WebFetch (GitHub/OCSF)** — OCSF `data_classification` profile status (CORRECTED a stale perplexity claim).
- **Honesty flags:** the perplexity passes are vendor/marketing-heavy on the tokenization-vs-FPE framing; I have down-weighted vendor claims and leaned on NIST + the crates.io ground truth for version-specific assertions. Where a regime's text was only partially in the retrieved corpus (HIPAA 18-identifier list, GDPR Recital 26 verbatim), that is flagged inline.

---

## 2. Q1 — Masking / tokenization techniques (the toolbox)

### 2.1 The property matrix (what each technique actually guarantees)

Synthesized from pass-1 §7.1, NIST SP 800-38G, and vendor docs. "Reversible" = operationally reversible by an authorized holder of keys/vault.

| Technique | Reversible | Deterministic option | Preserves referential integrity (joins) | Preserves format | Custody requirement |
|---|---|---|---|---|---|
| **Vaulted tokenization** | Yes (via vault lookup) | Yes (configurable) | Yes, if deterministic tokens | Optional | Secure token vault (self-host or vendor) |
| **Vaultless tokenization** | Yes (via crypto device/key) | Yes (configurable) | Yes, if deterministic config | Optional | Crypto keys / HSM (no vault DB) |
| **FPE (FF1 / FF3-1)** | Yes (via key) | Yes (fixed key+tweak) | Yes (same plaintext→same ciphertext) | **Yes (by design)** | Key management (KMS/HSM) |
| **Redaction (full)** | **No (irreversible)** | n/a | No | No | None (value destroyed) |
| **Deterministic masking (substitution/hash)** | Only if mapping retained | Yes | Yes | Sometimes | Mapping/keys if reversible |
| **Randomized masking** | Usually no | No | No | Sometimes | None if irreversible |

Two design axes cut across the table and matter most for Prism:

- **Reversible vs irreversible.** Reversible = tokenization, FPE, mapped masking → these are *pseudonymisation* (GDPR). Irreversible = full redaction, non-mapped substitution → can approach *anonymisation*. [pass-1 §5]
- **Deterministic vs randomized.** Deterministic preserves joins/correlation (a tokenized `asset_id` still joins across tables) at the cost of frequency/linkage-attack exposure. Randomized maximizes privacy but breaks joins. [pass-1 §6]

### 2.2 Tokenization: vaulted vs vaultless

- **Vaulted** — sensitive value → token, both stored in a secure vault; detokenize by reverse lookup. Token alone is mathematically non-invertible (not produced by a cipher), so token-store compromise without the vault yields nothing. Operationally reversible while the vault exists → GDPR pseudonymisation, not anonymisation. [pass-1 §2.2]
- **Vaultless** — token produced by a keyed cryptographic function (often format-preserving); detokenize by re-applying the function with the key. No mapping DB → scales better, but security collapses to *key* management and the technique converges with FPE. [pass-1 §2.3]
- **Convergent / deterministic tokenization** (HashiCorp Vault Transform's term) = same plaintext → same token, enabling joins without detokenization. Vault's default tokenization is *non-convergent* (random per encode). [pass-1 §2.4, §6.3]

### 2.3 Format-Preserving Encryption — NIST SP 800-38G (FF1 / FF3 / FF3-1)

- NIST SP 800-38G specifies two FPE modes, **FF1** and **FF3**, built as Feistel constructions over a block cipher (AES). Ciphertext keeps the plaintext's alphabet and length → fits existing schemas/validators. [pass-1 §3.1; NIST]
- **FF3 was broken.** Durak & Vaudenay published a practical attack exploiting FF3's tweak/domain structure; NIST concluded FF3-as-originally-specified should not be used as a general-purpose FPE. The fix → **FF3-1**, which constrains the tweak (8→6 effective bytes; two tweak bytes forced to zero). FF3-1 is a *revision* of FF3, not a new algorithm. [pass-1 §3.2; NIST FF3 advisory]
- **Domain-size minimum.** The SP 800-38G Rev.1 draft *strengthened* the minimum domain size to a **requirement of 10^6** for FF1 and FF3-1 — small domains (short codes, small enums) are insecure under FPE and must use tokenization instead. [pass-1 §3.3]
- **When FPE vs tokenization:** FPE is *stateless* (store key, not mappings) → scales for high-volume frequent-decrypt streams; but a key compromise decrypts *everything* (single point of failure). Tokenization is *stateful* (vault) → vault is a hard isolation boundary; token compromise alone reveals nothing. **For an AI-opaque pipeline where the agent must never hold reversible ciphertext, tokenization is preferable** — tokens have no mathematical link to plaintext, so even a future agent-log leak reveals nothing without the separately-guarded vault. [pass-1 §3.4]

### 2.4 Redaction, pseudonymization, anonymization

- **Redaction** = irreversible removal/placeholder; right for fields the agent never legitimately needs. Partial redaction (last-4-of-card) may still be reversible if the full value lives elsewhere. [pass-1 §4.2]
- **Pseudonymization (GDPR Art 4(5))** = transform such that re-attribution needs *additional info kept separately*. Tokenization/FPE/mapped-masking all produce pseudonymous (still-personal, still-in-scope) data. [pass-2 §4.1]
- **Anonymization (GDPR Recital 26)** = irreversible, re-identification "not reasonably likely by any party." High bar; quasi-identifier linkage often defeats naive direct-identifier removal. Only aggressive redaction/generalization gets here. [pass-2 §4.1; EDPB Guidelines 01/2025 on pseudonymisation]

### 2.5 Vendor landscape & air-gap implementability

| Vendor / product | Model | Air-gap / on-prem (no external SaaS)? |
|---|---|---|
| **HashiCorp Vault Transform** (Enterprise) | FPE *and* tokenization transformations; convergent option; roles/TTL/rotation; tokenization writes mappings to primary storage, FPE is stateless (stores keys only) | **Yes** — deployable fully on-prem/private-VPC; full custody retained. The closest off-the-shelf fit for a self-hosted clearing house. [pass-1 §2.2, §3.1] |
| **Skyflow Data Privacy Vault** | Vault-as-a-service; tokenize/mask/encrypt inside vendor vault | SaaS-centric; vendor holds vault. Poor fit for air-gap. [pass-1 §2.2] |
| **Very Good Security (VGS)** | Tokenization API; non-relational tokens in VGS Vault | SaaS; vendor custody. Poor fit. [pass-1 §2.2] |
| **Protegrity** | Enterprise FPE (FF1 over AES-256), preserves delimiters | On-prem capable; commercial license. [pass-1 §3.1] |
| **AWS (KMS/Macie/etc.)** | Cloud KMS + classification | Cloud-dependent; not air-gap. |

**Verdict for Prism:** the air-gap/on-prem-capable requirement (NERC CIP, defense) rules out the SaaS vaults. The realistic options are **(a) self-hosted Vault Transform** as a dependency, or **(b) a Prism-native Rust clearing house** built on the crypto primitives below. Given Prism's existing SS-26 secret broker + per-tenant DEK hierarchy, **(b) is the architecturally coherent path** — the token vault becomes another RocksDB column-family-backed store guarded by the existing DEK machinery, avoiding a heavyweight external Vault Enterprise dependency.

### 2.6 Rust-native primitives (crates.io — VERIFIED 2026-06-27)

| Crate | Latest version | Last updated | Covers | Notes |
|---|---|---|---|---|
| **`fpe`** | **0.6.1** | **2023-04-13** | **FF1 only** (NIST SP 800-38G) | RustCrypto-adjacent; pure-Rust; no-std capable. **FF3-1 is NOT implemented.** Stale (~3yr). Dev-dep on `aes ^0.8`, runtime deps `cipher ^0.4`, `cbc ^0.1`, `num-bigint ^0.4`. [crates.io API] |
| **`aes`** | 0.9.1 (crates.io API max) / RustCrypto stable line is 0.8.x | 2026-05-27 | AES block cipher (pure Rust) | Note: `fpe 0.6.1` pins `aes ^0.8` as a dev-dep; a clearing house would pin the version `fpe` is compatible with. **Verify the exact `aes` version that `fpe` builds against before pinning** — the 0.9.1 max-version report needs reconciliation against `fpe`'s `cipher ^0.4` constraint. [crates.io API] |
| **`aes-gcm`** | 0.11.0-rc.4 | 2026-05-25 | AEAD (Galois/Counter Mode) | For envelope-encrypting the token vault itself + DEK-wrapped values. RC, not yet stable-1.0. [crates.io API] |

**Crate finding (load-bearing):** There is **mature pure-Rust FF1** (`fpe`) but it is **stale and FF1-only — no FF3-1**. For Prism this is acceptable because: (1) FF1 is the NIST-preferred mode (FF3→FF3-1 history makes FF1 the safer default anyway); (2) tokenization (our recommended default) doesn't need FPE at all — it needs AES-GCM + a keyed PRF + a vault, all available pure-Rust. **A dependency on the stale `fpe` crate is a risk** (unmaintained, FF1-only); the lean is to make FPE *optional* (only for the narrow "consumer needs format-valid surrogate" case) and build the tokenization default on `aes-gcm` + the existing DEK subsystem.

---

## 3. Q3 — Classification / tagging: how we decide WHAT to mask (the RSI model)

### 3.1 Industry pattern: two-axis classification → policy binding

Every mature platform converges on **(data-type axis) × (sensitivity axis)**, then binds masking/access policy to the tags: [pass-2 §2.1]
- **Snowflake** — auto-classifies columns into a *semantic category* (NAME, EMAIL, …) + *privacy category* (`IDENTIFIER` / `QUASI_IDENTIFIER` / `SENSITIVE`), then **tag-based masking** auto-applies a policy when a tag lands; new columns get classified+masked automatically. [pass-2 §2.2]
- **Microsoft Fabric Dynamic Data Masking** — central policy masks designated columns at query time for non-privileged roles; privileged roles see raw. Explicit caveat: DDM is *not* a defense against exhaustive reconstruction queries — it is presentation-layer, not a substitute for access control. [pass-2 §2.2]
- **BigQuery PII Classifier (open source)** — customer-defined *data classification taxonomy* (PII types × confidentiality levels), auto-discovers+tags, binds column-level ACLs. [pass-2 §2.2]

### 3.2 Detection mechanisms (how fields get tagged)

- **Schema/lexical** — column-name heuristics + declared types. [pass-2 §2.3]
- **Pattern + checksum** — regex + Luhn (card), format validators (SSN, IP). [pass-2 §2.3]
- **NER (for unstructured/free-text fields)** — spaCy-NER + rule hybrid; **Microsoft Presidio** is the reference architecture: a detached *identification* engine (NER + regex + checksum + context recognizers, pluggable) feeding a separate *anonymization* engine (mask / redact / replace / tokenize). [pass-2 §2.4, §2.5]
- **Prism note:** because Prism normalizes to OCSF at the *edge connector boundary* (project core architecture), the field-type axis is *already known from the OCSF schema* — Prism does NOT need a Presidio-grade classifier for structured sensor data; it can drive masking from a **declarative field-sensitivity tagging of the OCSF schema** (which OCSF field classes are RSI under which profile). NER/Presidio is only needed for free-text fields (e.g., alert descriptions, log message bodies). This is a major scope simplification vs general-purpose DLP.

### 3.3 The RSI abstraction with pluggable PROFILES

The C20 NERC research's recommendation holds and is reinforced here: model a **sector-neutral "Regulated Sensitive Information (RSI)" abstraction** that separates *intrinsic field properties* (is it an identifier? quasi-identifier? what semantic type?) from *regulatory profiles* (rules that interpret those properties per regime). [pass-2 §3.1, §3.3]

- Intrinsic layer: `data_type` (IP, hostname, asset_id, person_name, …) × `confidentiality`/`privacy_category` (IDENTIFIER / QUASI_IDENTIFIER / SENSITIVE).
- Profile layer (pluggable): **BCSI** (NERC CIP-011), **PII** (GDPR), **PHI** (HIPAA), **PCI** (PAN) — each profile is a rule-set mapping intrinsic tags → masking action (tokenize / FPE / redact) + custody + who-can-detokenize.
- A single field can be claimed by multiple profiles in different tenants/contexts (a national-ID is PHI in a HIPAA tenant, PII in a GDPR tenant) — profiles *interpret* the same classification, they don't re-tag.

---

## 4. OCSF field-sensitivity — CORRECTION to a stale finding

The deep-research pass-2 reported that OCSF's data-classification profile was "only a GitHub-issue proposal, not adopted." **This is OUTDATED. Verified via OCSF GitHub releases (2026-06-27):**

- OCSF **`data_classification` profile is REAL and SHIPPED** — added in **OCSF v1.2.0 (released 2024-04-23, PR #998)**, along with a `data_classification` object, `data_security` object, etc. The profile was applied to `database`, `databucket`, `email`, `file`, `metadata`, `product`, `resource_details`, and `web_resource` objects. Latest OCSF release is **v1.8.0 (2026-03-18)**. [WebFetch github.com/ocsf/ocsf-schema/releases]

**Implication for Prism:** since Prism already normalizes to OCSF, the RSI classification should be **expressed over OCSF's existing `data_classification.confidentiality` / `data_type` attributes** rather than inventing a proprietary tagging wire format. The internal RSI/profile model is ours; the *interchange representation* should be OCSF-native. (The exact enum values of `confidentiality` should be confirmed against schema.ocsf.io before being made load-bearing — I confirmed the profile exists and which objects carry it, not the full enum.)

---

## 5. Q5 — What each regime requires of masking (custody & reversibility)

| Regime | What it requires | Reversible tokenization OK? | De-identification accepted |
|---|---|---|---|
| **GDPR** (Art 4(5), Recital 26, EDPB Guidelines 01/2025) | Pseudonymisation = re-attribution info kept *separately* under technical/org measures. Pseudonymous data **stays in scope**. | **Yes** — tokenization is textbook pseudonymisation, but does NOT exit GDPR. | Only true anonymisation (re-id "not reasonably likely") exits scope. [pass-2 §4.1] |
| **HIPAA** (45 CFR 164.514) | **Safe Harbor** = remove 18 enumerated identifiers + no actual knowledge of residual re-id risk. **Expert Determination** = qualified expert documents risk "very small" for the anticipated recipient. | **Yes, conditionally** — token must NOT be derivable from the identifier (else not Safe-Harbor); Expert-Determination can bless a tokenized-with-separate-mapping scheme if recipient risk is "very small." HIPAA text never says "tokenization." | Coded/tokenized data may still be PHI to the entity holding the mapping. [pass-2 §4.2] |
| **PCI DSS** (PCI SSC Tokenization Guidelines) | Replace PAN with token; vault + tokenization system stay *in CDE scope*; tokens must resist prediction. **Not** a privacy/identifiability standard — it's a *scope-reduction* standard. | **Yes — explicitly encouraged.** Systems handling only non-reversible tokens may be scoped OUT; the vault/detokenization path stays in scope. | No "de-identified cardholder data" concept; data is in-scope or out-of-scope by reversibility + connectivity. [pass-2 §4.3] |
| **NERC CIP-011-3** (BCSI) | Data-centric protection: encryption + provisioned access + secure disposal; entity-held keys, provider zero-plaintext-access (post CIP-004-7/CIP-011-3, eff. 2024-01-01). Doesn't name tokenization. | **Yes** — tokenizing BCSI in logs sent to third parties is consistent with "prevent unauthorized access," provided the entity holds the mapping/keys. | No de-identification concept; protect the info wherever it resides. [pass-2 §4.4; nerc-cip-support-2026-06-27.md §3] |

**Cross-regime synthesis:** reversible tokenization is permitted everywhere we care about; the binding constraint is uniformly **custody** — keys/vault/mapping must be held by the regulated entity (or the edge), separated from the consumer, with auditable access. This is *exactly* the CIP-011-3 entity-held-key / zero-plaintext-access model and it generalizes to all four profiles. The per-tenant DEK custody Prism already designed (SS-26) satisfies the strictest of these.

---

## 6. Q2/Q6 — Clearing-house placement & key custody (the architecture)

### 6.1 Placement: EDGE, after OCSF normalization (decisive lean)

Two candidate placements:
- **(i) Edge** — mask/tokenize immediately after OCSF normalization, before any transit to central/cache.
- **(ii) Surface** — central cache holds raw; mask only just before the ModelBackend.

**Lean: (i) edge placement.** Reasoning chain [reason-pass T2]:
1. The hard constraint is "agent never sees raw" AND the system must be air-gap/on-prem-capable for BCSI tenants.
2. Edge placement makes the *masking boundary coincide with the true data-sensitivity boundary*. Raw BCSI/topology/configs **never leave the local trust zone** — they never reach the Option-3 tenant-keyed central cache.
3. Blast radius of a central compromise (insider, misconfig, breach) is limited to surrogates + derived features. Under (ii), central is a single high-value store of raw regulated data across tenants — the opposite of CIP-011-3's intent.
4. It reconciles with **Edge-Computes / Central-Surfaces** and **Operator-Zero-Access / P-ADS-07 AI-Opaque**: central genuinely cannot see raw, not just "promises not to look."

Cost of (i): central-side analytics that genuinely need raw values must be pushed down to the edge/secure zone (or operate on deterministic tokens). This is a real constraint, not a blocker — most cross-tenant analytics work on OCSF-normalized enums/numerics + deterministic-token joins.

### 6.2 Token vault & DEK custody

Under edge placement [reason-pass T2 §consequences]:
- **Per-tenant token vault + DEK live at/near the edge** (or in the customer's highest-trust on-prem SOC cluster) — NOT in central. Central stores token values + masked records but holds **no DEK** → it can correlate (deterministic tokens) but cannot detokenize.
- The **agent/ModelBackend path is not wired to the vault at all** — no route, no creds, no role. This is the data analogue of AD-017's credential-broker-at-I/O-boundary.
- **Authority separation:** the team owning edge+vault (customer security/KMS) holds DEKs; the central analytics/AI plane operator never sees raw and never holds DEKs. Critical when central is multi-tenant or jointly operated.

### 6.3 Detokenize-at-surface via RBAC (C18)

- Analyst UI shows masked records from central. On "reveal," the UI calls the **vault service inside the secure zone**, RBAC/ABAC gates by analyst identity × tenant × token class, returns raw transiently to the client session.
- Detokenized values are **NOT written back to central long-term storage** — transient, heavily audited (ties the CIP-007/CIP-004 "who viewed BCSI" audit requirement from the NERC research §4).
- This binds to C18 RBAC and the central-deployment per-connection OAuth identity (ADR-051 in the central-deployment research): the detokenization grant is a capability on the analyst's authenticated identity.

---

## 7. Q4 — AI-opaque data flow + the embedding tension (C12 interaction)

### 7.1 Embeddings are sensitive — inversion attacks are real

**Load-bearing finding:** embedding-inversion research reconstructs **50–90%+ of source text** from embeddings, including names and medical diagnoses; security guidance now treats embeddings as "just as sensitive as the data they derive." [reason-pass T1; refs IronCore Labs, FINOS AI governance, Cyborg, arXiv]. Therefore: **you cannot use "embed it" as a masking step**, and you cannot ship raw text to an external embedding service or store vectors in an untrusted vector DB without exfiltrating the data.

### 7.2 The right architecture: on-box raw embedding + dual index

This *validates C12's on-box-embedding instinct as load-bearing*, not optional:
- **Embed RAW locally inside the edge trust boundary** (on-box embedding model). Raw never leaves; only vectors + masked text leave.
- Treat the **vector store as a sensitive data store** (encryption at rest, per-tenant isolation, RBAC, audit) — same custody class as the token vault.
- **Dual-index pattern:**
  - **Human-IR index** (inside secure zone): raw text + raw embeddings — for machine correlation + authorized human investigation. Agent has NO access.
  - **AI/RAG index** (what the agent can query): **masked view** — sensitive identifiers → deterministic tokens, ultra-high-risk fields omitted, *contextual* text preserved (behavior, topology role, OCSF enums). Either re-embed the masked view into an "LLM-safe" vector space, or use metadata-filter retrieval and rely less on dense semantics for the most sensitive fields.
  - **Everything crossing the AI boundary is from the masked view only**, regardless of how the underlying raw was embedded for the human index.

### 7.3 Mask-then-embed degradation — acceptable for security workloads

Masking high-entropy identifiers (IPs, hostnames, rule IDs) destroys *their* lexical semantics, but for SOC workloads the useful signal is mostly in non-identifying context ("failed login from partner VPN," "edge firewall in DMZ"). The agent reasons over patterns/roles/behaviors + stable tokens (`ASSET_TOKEN_A`), which is what AI-SOC recommendations (C15) actually need. We lose literal-identifier semantic search in the AI path (acceptable — that's a human-IR-index capability), and can even *policy-block* agent queries that target raw identifiers ("tell me about IP 10.0.0.5"). [reason-pass T1]

### 7.4 Optional advanced defense (defer-candidate)

If vectors must ever sit in less-trusted infra: embedding-perturbation schemes (EntroGuard-style entropy-driven, bound-aware noise within an ε radius) degrade inversion while preserving nearest-neighbor retrieval. [reason-pass T1 ref 7]. This is active research, adds embedding↔store coupling, and should be a **later-phase option**, not v1 — v1 relies on classic isolation + encryption + on-box embedding.

---

## 8. ANALYSIS + LEANS (recommended architecture)

**Recommended C16 clearing-house architecture:**

1. **Technique mix, keyed by field class (driven by RSI profile):**
   - High-risk identifiers (IP, hostname, asset_id, firewall-rule_id, BCSI configs) → **deterministic vaulted tokenization** (joins preserved; token mathematically unlinked to plaintext; vault is the isolation boundary). Default.
   - Fields where a downstream consumer genuinely needs a *format-valid* surrogate AND domain ≥ 10^6 → **FF1 FPE** (optional, narrow).
   - Fields the agent never legitimately needs → **full redaction** (irreversible).
   - Free-text fields → Presidio-style NER detection → tokenize/redact spans.

2. **Placement: EDGE, immediately after OCSF normalization.** Raw never transits to the Option-3 central cache. Central holds surrogates + derived features only.

3. **Classification: declarative RSI tagging over the OCSF schema** (structured fields' type axis is already known from OCSF; OCSF's shipped `data_classification` profile, v1.2.0+, is the interchange representation). Presidio-grade classifier reserved for free-text.

4. **AI-opaque data flow: on-box raw embedding + dual index.** Human-IR index (raw, secure zone, no agent access) vs AI/RAG index (masked view only). Vectors are sensitive-data-class. C12 on-box embedding is load-bearing, not optional.

5. **Reversibility / key custody: per-tenant token vault + DEK at the edge/secure zone** (reuse SS-26 DEK hierarchy; vault = a DEK-guarded RocksDB CF rather than external Vault Enterprise). Agent path has zero vault wiring (data analogue of AD-017).

6. **Detokenize-at-surface only via C18 RBAC** on the authenticated analyst identity (ADR-051 per-connection OAuth); transient client-side reveal; never re-persisted to central; audited per CIP-004/007.

**Universal-name recommendation:** Adopt **"Regulated Sensitive Information (RSI)"** as Prism's *internal* abstraction with **BCSI as the first concrete profile** (consistent with the C20 NERC research). Caveat to the human: RSI is *not* an industry-standard term — there is no single adopted term (the field uses "data classification taxonomy," "sensitive data elements," "protected data classes," and OCSF's `data_classification`). RSI is a fine internal name *provided* the wire/interchange representation is OCSF-native (`data_classification.confidentiality` / `data_type`), so we don't fork the ecosystem. If the human prefers an externally-recognizable term, **"Protected Data Classes"** (matches institutional usage) or aligning directly to OCSF `data_classification` terminology are the runner-up options.

---

## 9. Genuine sub-forks needing a HUMAN decision

1. **Build-vs-buy the clearing house.** Self-hosted **HashiCorp Vault Transform Enterprise** (off-the-shelf, FPE+tokenization, licensed dependency) vs **Prism-native Rust clearing house** on `aes-gcm` + SS-26 DEK + a RocksDB-CF token vault. Lean: native (coherent with existing DEK subsystem, no Vault Enterprise dependency, air-gap-clean) — but this is a real scope+maintenance decision. The stale FF1-only `fpe` crate (0.6.1, 2023) means native-FPE carries a maintenance risk; mitigated by making FPE optional and tokenization the default.

2. **Edge-placement cost acceptance.** Edge masking means central-side analytics needing raw values must push down to edge/secure zone. Is the human willing to accept that central operates only on surrogates + OCSF-normalized features + deterministic-token joins? (This is the load-bearing tradeoff of the recommended architecture.)

3. **Deterministic vs randomized token policy per field.** Deterministic = joins/correlation across tenants/time (needed for C15 recommendations, entity correlation) but exposes frequency/linkage attacks. Randomized = max privacy, no joins. Likely per-field-class policy — needs a human-ratified default matrix.

4. **Dual-index vs single-index-with-row-views for embeddings.** Separate human-IR and AI/RAG vector indexes (clean isolation, 2× storage) vs single index with policy-based row-level masked views (less storage, more policy-engine risk). Lean: dual index for v1 (isolation > storage). Architect call.

5. **RSI external naming.** Accept "RSI" internally, or align externally to OCSF `data_classification` / "Protected Data Classes." Product/positioning decision.

6. **HIPAA Expert-Determination path.** If Prism ever targets healthcare tenants, does the *product* ship an Expert-Determination-ready tokenization mode (documented risk methodology), or is that the customer's compliance burden? Defer-candidate but flag now.

---

## 10. Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) masking/tokenization techniques + NIST FF1/FF3-1 + vendor landscape + Rust crates; (2) data classification/tagging + OCSF + GDPR/HIPAA/PCI/NERC de-identification requirements. Both reasoning_effort=high. |
| Perplexity perplexity_reason | 1 | Synthesis over the two Prism-specific architectural tensions (embedding-vs-masking; clearing-house edge-vs-surface placement) with embedding-inversion evidence. |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (crate facts taken from crates.io API directly, which is more authoritative for version numbers). |
| Tavily | 0 | — |
| WebFetch | 5 | crates.io API for `fpe`/`aes`/`aes-gcm` versions + `fpe` deps; OCSF GitHub releases for `data_classification` profile version/date. |
| WebSearch | 1 | OCSF `data_classification` profile adoption status (corrected stale perplexity claim). |
| Read (in-repo) | 3 | RESEARCH-INDEX; nerc-cip-support research; central-deployment research. |
| Training data | 2 areas | (a) general framing of vaulted/vaultless tokenization mechanics; (b) RocksDB-CF / SS-26 integration framing — both cross-checked against in-repo research and the perplexity passes; flagged where used. |

**Total MCP tool calls:** 3 (2 perplexity_research + 1 perplexity_reason). Plus 6 web verification fetches/searches and 3 in-repo reads.
**Training data reliance:** low — every non-obvious claim is sourced to a perplexity deep-research pass, the crates.io API (verified versions), NIST SP 800-38G, the OCSF release notes, or prior in-repo cited research. The one CORRECTION (OCSF `data_classification` is shipped, not proposed) was caught by cross-validating the perplexity claim against the authoritative GitHub release notes — a worked example of why the cross-validation layer is mandatory.

**Deviation note (per agent mandate):** `perplexity_research` was used as PRIMARY for both non-trivial topics, satisfying the bias rule. `perplexity_reason` was used only for synthesis over gathered evidence (the two architectural tensions), which is its correct role.
