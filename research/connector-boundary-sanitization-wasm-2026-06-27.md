---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
provenance: "side-analysis discussion input (day-2 vision SIDE-ANALYSIS program); OUT-OF-BAND, separate from the live VSDD factory pipeline. Does not modify vision/specs/ADRs/STATE/handoff/prior research."
topic: "Connector-boundary normalization/sanitization: buildable Rust mechanism + hot-path cost (D-C4-4 quarantine+relabel); WASM capability-sandbox prior art (D-C4-3)"
builds_on: "dynamic-schema-connectors-2026-06-27.md (C4 Topic 6 owns the THREAT MODEL: OWASP LLM01, CWE-20/400/1007, UTS#39, CAPEC-146). This pass fills the IMPLEMENTATION + COST gap C4 left open."
feeds: "Day-2 vision discussion on the MANDATORY connector-boundary sanitization layer (all connectors incl. live OCSF sensors) and the WASM-connector escape-hatch. DISCUSSION input only."
scope_note: "Targeted hardening pass, NOT a broad survey. Does NOT re-derive the threat model (C4 owns it)."
---

# Connector-Boundary Sanitization (buildable Rust + hot-path cost) and WASM Capability-Sandbox Prior Art

**Side-analysis / discussion input — NOT a spec, vision, or ADR change.** `do_not_execute: true`.

This pass exists because the human decided in C4 that the connector-boundary normalization/sanitization layer is **MANDATORY FOR ALL CONNECTORS — including the live OCSF security sensors (CrowdStrike / Cyberint / Claroty / Armis)** — and that hostile identifiers are **quarantined + relabeled, not rejected** (D-C4-4). C4 (`dynamic-schema-connectors-2026-06-27.md` Topic 6) established the POLICY/threat model (OWASP LLM01 indirect prompt injection, CWE-20/400/1007, UTS#39 confusables, CAPEC-146 schema poisoning) but **not the buildable Rust mechanism, its hot-path cost, the identifiers-vs-values scope boundary, the quarantine/relabel mechanism, or the WASM capability-sandbox guarantees**. Putting this chokepoint on the **live sensor path** makes mechanism + cost load-bearing. **C4 owns the threat model; this pass does not re-derive it.** "Leans" below are discussion input only.

> **Read-coverage honesty.** Two of the high-effort `perplexity_research` runs (Topic-2/4 value-sanitization and the WASM deep dive) exceeded the inline token cap and were persisted to tool-result files that are single ~85–91 KB lines which `Read` cannot paginate and `Grep` will not print as context. I read the **first ~62 KB of the 91 KB Rust-crate run** (the load-bearing crate-evaluation + pipeline-order material — sections 1 through 5.2) and could NOT read its final ~29 KB (a performance-discussion + conclusion recap that restates earlier content). To compensate I ran **two medium-effort, inline-readable** `perplexity_research` queries that fully cover the spotlighting/value-scanning and WASM-sandbox material, and I **independently verified every load-bearing crate/runtime version against the crates.io API** (the most authoritative source — Context7 did not carry these niche crates). Net: no load-bearing claim rests only on an unread file.

---

## Executive Summary (~14 lines)

1. **The Rust sanitization mechanism is buildable TODAY from maintained crates.** The pipeline = `unicode-normalization` (NFC) + manual bidi/control codepoint reject + `unicode-script` (single-script) + `unicode-security` (UTS#39 skeleton, mixed-script, restriction-level) + a length cap. Versions verified on crates.io 2026-06-27 below. [crates.io][P-rust]
2. **`unicode-security` 0.1.2 (2024-09-12, Manish Goregaokar)** is the one production-viable UTS#39 crate: it re-exports `confusable_detection::skeleton`, `GeneralSecurityProfile`, `MixedScript` + `is_potential_mixed_script_confusable_char`, and `RestrictionLevel`/`RestrictionLevelDetection`, tracks `UNICODE_VERSION`, and is `no_std`-capable. It is **pre-1.0 with an open "implement all of UTS#39" tracking issue and an open (Mar-2025) CJK-ideograph confusable gap** — viable, but own the gaps. [P-rust][unicode-security-dr]
3. **`unicode-skeleton` is effectively abandoned: latest 0.1.1 dated 2017-10-08, pinned to UTS#39 v10.0.0.** Do NOT depend on it for a 2026 security tool — `unicode-security::skeleton` supersedes it. [crates.io][P-rust]
4. **`unicode-normalization` 0.1.25 (2025-10-30)** is mature and the canonical NFC source; `.nfc()` is an iterator adapter and ASCII is canonically stable (effectively a fast-path). `unicode-script` 0.5.8 (2025-12-03) and `unicode-bidi` 0.3.18 (2024-12-16) are current and maintained. `decancer` 3.3.3 (2025-07-16) is maintained but is an *aggressive lossy cleanser* (not UTS#39-normative) — wrong tool for identity-preserving identifiers; the abandoned `confusables` 0.1.0 (2023) should be avoided. [crates.io]
5. **Trojan-Source / CVE-2021-42574 (bidi) is best handled by an explicit codepoint reject**, not a crate: forbid `U+202A–U+202E`, `U+2066–U+2069`, `U+200E/U+200F`, and the broader `Default_Ignorable_Code_Point` / zero-width set in identifiers. `unicode-bidi` is for layout analysis, not identifier defense. [P-rust][trojan-source]
6. **Correct pipeline ORDER:** (0) reject empty/over-length bytes → (1) UTF-8 validate + trim → (2) reject bidi/control/zero-width codepoints → (3) NFC normalize → (4) single-script allowlist / mixed-script check → (5) length cap on code points → (6) compute confusable skeleton + collision-check. Reject bidi/control BEFORE normalize so normalization never "launders" a hostile char. [P-rust]
7. **IDENTIFIERS-vs-VALUES is the sharpest finding: do them with DIFFERENT rigor.** No public evidence shows production systems run full UTS#39 skeleton scans over every cell value on the hot path; the defensible, evidence-aligned architecture is **heavy Unicode/confusable sanitization on the BOUNDED identifier set ONCE at schema-pin time (cacheable)**, and a **LIGHTER, bounded-cost structural treatment (encoding/delimiting + invisible-char strip + token-budget cap) on the UNBOUNDED value stream**. [P-val][owasp-llm01][willison]
8. **Spotlighting (Microsoft Research, Hines et al., arXiv:2403.14720) is the demonstrated value-stream defense.** Three variants — delimiting, datamarking, encoding(base64) — drove indirect-injection attack-success-rate from **>50% to <2% on GPT-family models**, encoding strongest. Caveats: model-capability-dependent, token/latency overhead, NOT complete vs adaptive adversaries. This is *demonstrated*, not theater — but it is mitigation, not guarantee. [P-val][spotlight-arxiv][spotlight-msrc]
9. **Confusable substitution is NOT defeated by NFKC alone** — only a TR39 *skeleton* reliably collapses cross-script confusables; and modern frontier models *correctly interpret* confusable substitutions, so "the model will fail to read it" is a false assumption. This validates skeleton-on-identifiers. [P-val][confusable-llm]
10. **QUARANTINE + RELABEL (D-C4-4) is buildable deterministically:** relabel a hostile/suspicious identifier to a collision-safe placeholder `col_<base32(hash(original_bytes))[..N]>` (or ordinal `col_0001`), and retain the original raw bytes + skeleton + flags in an **audit-only, non-agent-facing** field, encoded (base32/hex/punycode) so the raw hostile string never re-enters agent context. Punycode (RFC 3492 / IDNA) is the canonical "reversible encode of hostile-unicode-to-ASCII" prior art. Collision-safety comes from hashing the raw bytes (not the display form) + an explicit collision check against already-assigned placeholders. [P-val][punycode]
11. **Hot-path cost is dominated by the value stream, and an ASCII fast-path makes the common case nearly free.** NFC over pure-ASCII is a pass-through; identifier sanitization is amortized to ~zero by normalize-once-cache-by-pinned-schema. The defensible architecture keeps the guarantee by **(a) caching identifier sanitization at pin time, (b) `is_ascii()` fast-path skipping Unicode work on ASCII values, (c) only stripping invisible/bidi chars + applying spotlight-encoding on values at ingest, never a per-cell skeleton scan.** [P-rust][P-val]
12. **WASM capability-sandbox (D-C4-3) prior art is strong and current.** Wasmtime (`max_stable 46.0.1`, LTS line 36.x; crates.io 2026-06-24) on **WASI Preview 2 / component model** gives **no ambient authority**: default `WasiCtxBuilder` grants **no FS preopens, denies IP name lookup, denies all socket addresses** — the guest reaches network/FS ONLY via host-granted capabilities. DoS bounds compose from **fuel** (`Config::consume_fuel` + `Store` fuel methods), **epoch interruption** (`Config::epoch_interruption` + `Store::set_epoch_deadline` + `Engine::increment_epoch`), and **memory/instance limits** (`StoreLimitsBuilder` + `Store::limiter` / `ResourceLimiter`). [P-wasm][wasmtime-wasictx][wasmtime-config]
13. **Extism 1.30.0 (crates.io 2026-06-04) is maintained and production-oriented**, wrapping the same model behind a manifest (`allowed_hosts`, `allowed_paths`, memory `MaxPages`, `timeout_ms`). It contrasts sharply with **Airbyte's `AIRBYTE_ENABLE_UNSAFE_CODE`** which runs arbitrary Python in-process with **full ambient authority and explicitly "no sandboxing."** A properly-sandboxed wasmtime host provides what Airbyte's path cannot: confinement to pre-approved FS/net + bounded CPU/memory. [P-wasm][extism-manifest][airbyte-custom]
14. **CODEBASE-RECONCILIATION FLAG (not resolved here):** Prism already ships a plugin SDK (a `threatintel-lookup` plugin exists under `crates/prism-spec-engine/plugins/`). The day-2 WASM-connector path MUST be reconciled against this existing SDK at capture/morph time. That is an internal codebase task for architect+implementer, out of scope for this external-prior-art pass.

---

## Topic 1 — Buildable Rust Sanitization Mechanism (crate + API state VERIFIED)

### 1.1 Crate inventory — versions verified against the crates.io API, 2026-06-27

| Crate | Latest ver | Released | Role in pipeline | Maintenance verdict |
|-------|-----------|----------|------------------|---------------------|
| **unicode-normalization** | **0.1.25** | **2025-10-30** | NFC normalization (`.nfc()` iterator adapter); `is_nfc()`/QuickCheck | **Mature, production.** Canonical UAX#15 impl. [crates.io] |
| **unicode-security** | **0.1.2** | **2024-09-12** | UTS#39: `skeleton`, `GeneralSecurityProfile`, `MixedScript`, `RestrictionLevel(Detection)`, `UNICODE_VERSION` | **Viable, pre-1.0.** Active-ish (author Manish Goregaokar; CJK-confusable issue open Mar-2025; "implement all of UTS#39" tracking issue open). Own the gaps. [crates.io][unicode-security-dr][P-rust] |
| **unicode-script** | **0.5.8** | **2025-12-03** | UAX#24 `Script` / `Script_Extension` via `char` ext trait | **Maintained, production.** [crates.io] |
| **unicode-bidi** | **0.3.18** | **2024-12-16** | UAX#9 bidi *layout* analysis (NOT an identifier defense) | **Maintained**, but not the right tool for identifier bidi-reject. [crates.io][P-rust] |
| **decancer** | **3.3.3** | **2025-07-16** | Aggressive lossy confusable/homoglyph *removal* (not UTS#39-normative) | **Maintained** but WRONG tool for identity-preserving identifiers; possible UI-display aid only. [crates.io][P-rust] |
| **unicode-skeleton** | 0.1.1 | **2017-10-08** | TR39 skeleton (legacy) | **ABANDONED / stale** (pinned to UTS#39 v10.0.0). Superseded by `unicode-security::skeleton`. AVOID. [crates.io][P-rust] |
| **confusables** | 0.1.0 | 2023-08-23 | compile-time confusables table, `Confusable` trait | **Low-maintenance / single 2023 release.** AVOID for new work; spec-currency unclear. [crates.io][P-rust] |
| (rustc internal) | — | — | `confusable_idents` lint = per-ident skeleton in a hashmap, collision → warn (RFC 2457, based on UTS#39 §4) | Reference *model*, not a reusable dependency. [P-rust] |

**Load-bearing correction to a plausible assumption:** the *intuitive* pick `unicode-skeleton` is the trap — it has had **no release since 2017**. The maintained skeleton lives in **`unicode-security::confusable_detection::skeleton`**. This is the single most important version fact in this pass.

### 1.2 NFC normalization — `unicode-normalization`

- API: extension trait `UnicodeNormalization` on `Iterator<Item=char>`; `input.chars().nfc().collect::<String>()`. `is_nfc(s)` returns a QuickCheck `IsNormalized::{Yes,No,Maybe}` so already-normalized inputs avoid allocation. [P-rust]
- **ASCII is canonically stable under NFC** (Basic Latin neither decomposes nor recomposes) — so ASCII identifiers/values pass through at iterate-and-copy cost; QuickCheck returns `Yes` immediately. This is the de-facto ASCII fast-path. [P-rust]
- **NFC vs NFKC lean:** Prism should use **NFC for identity preservation** (a column name must round-trip to the real source column) and **separately detect/forbid compatibility characters** via the security profile, rather than letting NFKC silently fold full-width/circled/ligature forms (which changes identity). NFKC's folding is attractive for *defeating compatibility-confusables* but it mutates the identifier — wrong for something that must address a real source column. C4 reject-don't-widen discipline applies. [P-rust]

### 1.3 Trojan-Source / bidi + control rejection (CVE-2021-42574)

Do this with an **explicit codepoint reject**, before normalization. Forbidden in identifiers (and stripped/flagged in values):
- Bidi overrides/embeddings/isolates: `U+202A LRE, U+202B RLE, U+202D LRO, U+202E RLO, U+2066 LRI, U+2067 RLI, U+2068 FSI, U+2069 PDI`.
- Directional marks: `U+200E LRM, U+200F RLM`.
- Broader: `Default_Ignorable_Code_Point` set + zero-width (`U+200B/C/D`, word joiner `U+2060`), and C0/C1 controls (tab/newline/null must never appear in an identifier). [P-rust][trojan-source]

A `matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | ...)` check is sufficient and ~free. `unicode-security`'s `internalSkeleton` already *removes* `Default_Ignorable` before mapping, but C4's posture (and Trojan-Source's lesson) is to **reject the raw identifier outright if it carries these**, not merely skeleton-launder them. [P-rust]

### 1.4 The recognizer pipeline — ORDER and per-step cost

Recommended order (each step linear-time; "cost" is for typical ≤64-cp strings):

| # | Step | Crate / mechanism | Cost on short string | Why this position |
|---|------|-------------------|----------------------|-------------------|
| 0 | Byte-length pre-cap + non-empty | raw `len()` | ~0 (one compare) | Bound work before any Unicode op (CWE-400). |
| 1 | UTF-8 validate + trim ASCII whitespace | std | ~0 | Establish a valid char stream. |
| 2 | **Reject bidi/control/zero-width codepoints** | manual `matches!` | ~0 (one pass, no tables) | **Before** normalize so NFC can't launder a hostile char. |
| 3 | **NFC normalize** | `unicode-normalization` `.nfc()` | ASCII ≈ pass-through; non-ASCII = table lookups, µs-scale | Skeleton + script checks are defined over normalized text. |
| 4 | Single-script allowlist / mixed-script check | `unicode-script` + `unicode-security::MixedScript`/`RestrictionLevelDetection` | one pass + per-char property lookup | Cheaper than skeleton; rejects most hostile mixed-script before the costliest step. |
| 5 | Length cap on code points | `.chars().count()` vs cap | ~0 | Final bound (e.g. ≤128 ident, ≤few-hundred comment). |
| 6 | **Confusable skeleton + collision check** | `unicode-security::skeleton` into a `HashSet<String>` | most expensive: NFD + ignorable-strip + prototype map + NFD, then hash lookup | Last, on the survivors only — mirrors rustc's `confusable_idents`. |

Skeleton (step 6) is the heaviest because it decomposes, strips, prototype-maps, recomposes, then hashes — but for the **bounded identifier set computed once at pin time** the amortized per-query cost is **zero** (see Topic 3). [P-rust]

**Lean (Topic 1):** Build the recognizer from `unicode-normalization` + manual bidi/control reject + `unicode-script` + `unicode-security` (skeleton/mixed-script/restriction-level). Drop `unicode-skeleton` and `confusables` (stale/low-maintenance). Use the order above; reject-bidi-before-normalize is the load-bearing ordering rule.

---

## Topic 2 — Identifiers-only vs Values-too (the sharpest scope question)

### 2.1 What the evidence shows

- **No public evidence** that production LLM data-access systems run full UTS#39 skeleton/confusable scans over **every cell value on the hot path** — claims either way would be overstated; flagged **[INCONCLUSIVE]** as to a universal standard. What IS documented: schema identifiers / tool descriptors / prompt templates are defined by developers and can be heavily sanitized **at design/pin time**, whereas unbounded value streams get *lighter, scalable* treatment (delimiting/spotlighting/classifier filters), with heavy Unicode work pushed to **ingest/index time or high-risk channels only**. [P-val][owasp-llm01][willison]
- **OWASP LLM01** does NOT prescribe per-value skeleton scanning; it prescribes *segregating and clearly denoting external content*, output-format constraints, I/O filtering, least-privilege, and human approval for high-risk actions. [owasp-llm01]
- **Confusable substitutions are real against values too**, and crucially **NFKC alone does NOT collapse most confusable pairs — only a TR39 skeleton does** — and **frontier models correctly interpret confusable substitutions** (so "the model can't read it" is false). This argues for skeleton on the bounded identifier set, and at least *invisible-char strip + canonicalization* on values, but full per-cell skeleton is cost-prohibitive on the hot path. [confusable-llm][P-val]

### 2.2 Lean — two tiers, different rigor

- **IDENTIFIERS (bounded, pinned, done ONCE):** full recognizer pipeline incl. skeleton + mixed-script + restriction-level + quarantine/relabel. Cache the sanitized result keyed by the pinned schema version. This is the heavy tier and it is *off the hot path*.
- **VALUES (unbounded, per query result, hot path):** do NOT skeleton-scan every cell. Apply a **bounded-cost** treatment: (a) `is_ascii()` fast-path (skip all Unicode work — the common case for security telemetry); (b) for non-ASCII, strip invisible/bidi/zero-width + Unicode-tag (`U+E0000–U+E007F`) characters and NFC-normalize; (c) deliver values to the agent inside a **spotlighting envelope** (delimited/datamarked, or base64-encoded for high-risk fields) so the agent treats them as opaque data; (d) cap total value-derived token budget per context (CWE-400 / model-DoS). Reserve heavier per-value confusable canonicalization for explicitly high-risk fields or offline ingest, not the universal hot path. [P-val][spotlight-arxiv][confusable-llm]
- **This is the C4 reconciliation:** C4 said "sanitization layer mandatory for ALL connectors incl. OCSF sensors." That mandate is satisfied by routing **identifiers** of every connector (sensors included) through the heavy tier at pin time, and **values** through the bounded hot-path tier — NOT by skeleton-scanning every sensor cell, which would tank latency for no demonstrated gain.

---

## Topic 3 — Hot-path performance cost (mandatory on the LIVE OCSF sensor path)

### 3.1 Cost shape

- **NFC**: linear, per-char table lookups; **ASCII = pass-through** via QuickCheck `Yes`. Sub-microsecond for a 64-char ASCII string; low-µs for non-ASCII. [P-rust]
- **Script lookup / mixed-script**: one pass, per-char property lookup — cheap, comparable to NFC. [P-rust]
- **Skeleton**: heaviest (NFD → ignorable-strip → prototype-map → NFD → hash). This is the step you must keep OFF the per-query/per-cell path. [P-rust]
- The literature explicitly notes full Unicode skeleton/confusable analysis is **cost-prohibitive to run on every hot-path value**, which is why systems tier it to ingest/index/high-risk. [P-val]

### 3.2 Techniques to keep the guarantee without tanking sensor-query latency

1. **Normalize-once-and-cache for identifiers.** A connector's identifier set is fixed by the pinned schema (C4 discover-then-pin). Run the full recognizer ONCE at pin time; cache `{raw → sanitized_label, skeleton, flags, placeholder?}`. Per-query cost for identifiers = a cache lookup. Re-pin invalidates the cache (C4 drift workflow).
2. **ASCII fast-path on values** (`str::is_ascii()` is a cheap, vectorizable scan; for ASCII, skip NFC + script + skeleton entirely and only check for C0/C1 control bytes). Security telemetry is overwhelmingly ASCII, so the common case is near-free.
3. **Scan-on-ingest, not on-read.** Where Prism materializes into the Iceberg cold tier or a buffer, do the value normalization/invisible-strip once at ingest; reads serve already-clean data.
4. **Bounded value treatment, not skeleton.** On the hot path, value defense = invisible/bidi/tag strip + NFC + spotlight-envelope + token-budget cap. No per-cell skeleton.
5. **SIMD/vectorized validation.** `is_ascii()` and UTF-8 validation are already SIMD-friendly in std; that is the bulk of the per-value cost. [P-rust][P-val]

**Lean (Topic 3):** The defensible architecture is *identifiers heavy-and-cached at pin time* + *values light-and-bounded with an ASCII fast-path on ingest*. The mandatory-on-OCSF requirement is met with negligible steady-state query latency because the heavy work is amortized to pin/ingest time and ASCII telemetry skips the Unicode machinery.

---

## Topic 4 — Quarantine + Relabel mechanism (D-C4-4)

### 4.1 Mechanism

- **Decision (per C4 D-C4-4):** suspicious/hostile identifiers are **quarantined + relabeled, not rejected** — except on hard violations (control/bidi codepoints, over-length) where reject is appropriate (you cannot safely round-trip a control char into a real source column anyway).
- **Deterministic safe placeholder:** generate `col_<token>` where `token` is either (a) an **ordinal** within the pinned schema (`col_0001`, deterministic by declaration order — simplest, human-stable), or (b) `base32(BLAKE3/SHA-256(raw_bytes))[..N]` (content-addressed, stable across re-pins if the raw bytes are unchanged). Hash the **raw bytes**, not the display/normalized form, so two visually-identical confusables get *distinct* placeholders (they are genuinely different columns).
- **Collision-safety:** maintain a per-schema `HashSet` of assigned placeholders; on collision (hash prefix clash or ordinal reuse) extend the hash prefix / bump the ordinal until unique. This is a bounded loop over a bounded identifier set.
- **Reversible, auditable, NON-agent-facing record:** retain `{original_raw_bytes (encoded), nfc_form, skeleton, scripts, flags[]}` in an **audit/debug field that never enters agent context**. Encode the raw original with **punycode (RFC 3492 / IDNA)** or base32/hex so the hostile unicode is rendered as inert ASCII for logs/operator review — punycode is the canonical "reversibly encode hostile-unicode-to-ASCII" prior art (IDN homograph defense). The operator/SOC analyst can see *what the attack was* without the agent ever ingesting the raw string. [P-val][punycode]
- **Surface, don't hide:** emit the quarantine as a structured event (downstream: would need a BC-2.16.002 Canonical Structured Event Catalog row per CLAUDE.md SAP-1 — a spec dependency, NOT actioned here) and mark the column/connector with a quarantine flag in the response envelope.

### 4.2 Prior art for "quarantine and surface" vs reject

- **IDNA / punycode** (RFC 3492): the canonical reversible ASCII-encoding of arbitrary Unicode labels; browsers *relabel* suspicious IDN to punycode (`xn--…`) rather than reject — exactly the quarantine-and-surface pattern. [punycode]
- **Log sanitization / data-masking**: replace-with-safe-token + retain-encoded-original is the standard masking pattern; collision-safety via content hash + per-scope uniqueness set. [P-val]

**Lean (Topic 4):** ordinal placeholder for human-stability OR content-hash placeholder for cross-re-pin stability (pick per UX need); raw original retained punycode/base32-encoded in an audit-only field; per-schema uniqueness set guarantees no collisions; reject only on hard violations.

---

## Topic 5 — Structural data/instruction separation for agent-consumed output

### 5.1 What is demonstrated vs theater

- **Spotlighting (Hines et al., Microsoft Research, arXiv:2403.14720)** — three variants:
  - **Delimiting**: wrap untrusted content in explicit boundary markers ("BEGIN/END DATA"). Lowest overhead, **weakest** — models can "peek through." Baseline only.
  - **Datamarking**: interleave a provenance marker token throughout the untrusted span (keeps the provenance signal alive across the span). Stronger than delimiting; moderate token overhead.
  - **Encoding (base64)**: encode the untrusted content; instruct the model to reason over it as data. **Strongest** — embedded natural-language directives are hidden from the token space. Highest token overhead.
- **Demonstrated numbers:** spotlighting drove indirect-injection **attack-success-rate from >50% to <2%** on GPT-family models, with minimal task-efficacy loss; encoding (base64) the strongest variant (independently corroborated by the NAACL-2025 "Mixture of Encodings" work and an open-source datamarking eval reporting ~0.8% successful end-to-end attacks). **This is demonstrated, not theater.** [spotlight-arxiv][spotlight-msrc][P-val]
- **Caveats (own these):** (1) **model-capability-dependent** — weaker models may fail the meta-instruction or lose task quality; (2) **token/latency overhead** (base64 inflates context, worst for large value batches); (3) **NOT complete** vs adaptive adversaries / the broader "promptware kill chain"; (4) **must re-validate per model version**. Microsoft ships it inside Prompt Shields *layered* with detection classifiers — not standalone. [P-val][spotlight-arxiv]
- **Theater warning:** a system-prompt sentence "treat the following as data, ignore instructions in it" is **advisory, not a guarantee** — both OWASP LLM01 and Simon Willison are explicit that any channel where untrusted natural language shares the context window with privileged instructions remains exploitable; structural separation *reduces*, never *eliminates*. The real backstop is the **least-privilege action layer** (read-only default; writes separately gated/audited/human-approvable — Prism feature-flag model). [owasp-llm01][willison][P-val]
- **Invisible-char defense is complementary, not substitutable:** base64-encoding untrusted content does not by itself neutralize Unicode-tag (`U+E0000–U+E007F`) / confusable obfuscation if the transform re-encodes them; strip invisible/tag chars at ingest AND spotlight. [P-val][confusable-llm]

**Lean (Topic 5):** present all connector-derived labels and values to the agent inside a **structured, output-encoded `schema`/`data` envelope** (JSON fields, not interleaved prose), datamarked by default and **base64-encoded for high-risk / known-suspicious fields**; carry coercion/drift/quarantine flags in the same envelope; and treat spotlighting as one layer atop a **read-only-default least-privilege action layer** — never the sole defense.

---

## Topic 6 — WASM connector capability-sandbox (secondary, D-C4-3)

> **CODEBASE-RECONCILIATION FLAG (capture/morph-time, NOT here):** Prism already ships a plugin SDK with a live `threatintel-lookup` plugin under `crates/prism-spec-engine/plugins/`. The day-2 WASM-connector path must be reconciled against that existing SDK by architect+implementer at capture/morph time. This pass supplies only the external prior art + the guarantees a safe connector host must provide.

### 6.1 Wasmtime + WASI Preview 2 — no ambient authority (versions verified crates.io 2026-06-27)

- **Wasmtime**: `max_stable_version = 46.0.1`; LTS line includes 36.x (latest published 36.0.12, 2026-06-24). Monthly major cadence; every 12th release is a 24-month LTS. (April-2026 advisories patched 43.x/42.x/36.x/24.x — *no runtime is infallible*; track advisories, prefer an LTS, patch promptly.) [crates.io][P-wasm]
- **Capability model:** WASI's stated design principle is **"no ambient authorities"** — no global namespaces/functions; FS is descriptor-centric (preopened directories; absolute paths / `..` / escaping symlinks forbidden), network requires explicit socket capability handles with deny-by-default. [P-wasm]
- **Default `WasiCtxBuilder` (the load-bearing guarantee):** stdin closed; stdout/stderr discarded; **no env, no args, no preopens**; **all socket addresses DENIED**; **`wasi:sockets/ip-name-lookup` DENIED**. A guest built from defaults can do **no FS and no network** until the host explicitly grants a capability. [wasmtime-wasictx][P-wasm]
- **Embedding types:** `Engine` (holds `Config`), `Store` (per-instance host state + limits), `Linker` (wires ONLY the host functions/WASI interfaces you choose), `Component` (compiled component, instantiated via `Linker`), `bindgen!` macro (generates type-safe host bindings from a WIT world). For a fully virtual FS, implement the `wasmtime-wasi` host traits directly instead of `WasiCtxBuilder`. [P-wasm][wasmtime-bindgen][wasmtime-component]
- **Host guarantee (under bug-free runtime):** guest cannot open arbitrary FS paths or escape preopens; cannot do network/DNS beyond host-granted handles; cannot touch host memory outside its linear memory or `Store` data not passed to it; the only ambient-authority avenue is whatever host functions you register — so **register the minimum**. [P-wasm]

### 6.2 DoS bounds — how they compose

| Bound | API (current) | Bounds | Note |
|-------|---------------|--------|------|
| **Fuel** | `Config::consume_fuel(true)` + `Store` set/query fuel (`Store::set_fuel`, fuel-consumed) | total instructions/work; trap on exhaustion; default store starts with **0 fuel** (traps immediately if enabled and unfunded) | Deterministic CPU bound; per-call/per-tenant budget. [wasmtime-store][wasmtime-config] |
| **Epoch interruption** | `Config::epoch_interruption(true)` + `Store::set_epoch_deadline` + `Engine::increment_epoch` | wall-clock / scheduler preemption; trap or async-yield at deadline | Time-based preemption + fair scheduling. [wasmtime-store][wasmtime-config] |
| **Memory/instance limits** | `StoreLimitsBuilder` + `Store::limiter` / `ResourceLimiter` (async: `ResourceLimiterAsync`) | linear-memory bytes, table elements, instance/memory/table counts (default 10,000 each; memory default unbounded → MUST set) | Prevents memory-exhaustion DoS; note it bounds *guest* allocations, not all host-internal allocs. [wasmtime-resourcelimiter][wasmtime-storelimits] |

These compose: fuel (deterministic CPU) + epoch (time preemption) + StoreLimits (memory/instances) + WASI deny-by-default (no ambient FS/net) = a confined, bounded nano-process. [P-wasm]

### 6.3 Extism — maintained wrapper; and the Airbyte contrast

- **Extism 1.30.0** (crates.io 2026-06-04) — **maintained, production-oriented**. Manifest config: `allowed_hosts` (empty = no HTTP; wildcards allowed), `allowed_paths` (host→guest FS mounts, WASI-preopen-like), memory `MemoryOptions { MaxPages }` (256 pages ≈ 16 MiB), `timeout_ms` (uninterruptible-recovery hard stop). Guidance stresses least-privilege host functions, per-tenant instances, I/O validation. Built on the WASM sandbox; whether it defaults to wasmtime specifically is **[model-knowledge / not confirmed by retrieved sources]**. [extism-manifest][P-wasm]
- **Airbyte `AIRBYTE_ENABLE_UNSAFE_CODE`**: enables Python "Custom Components" that run **arbitrary code in-process with full ambient authority**, documented as **UNSAFE / EXPERIMENTAL with "no sandboxing guarantees,"** disabled by default, admin-opt-in via env var. [airbyte-custom]
- **What a properly-sandboxed wasmtime host provides that Airbyte's path does not:** confinement to **pre-approved FS paths and network endpoints (no ambient authority)** + **bounded CPU (fuel/epoch) and memory (StoreLimits)** — i.e., a hostile or buggy connector cannot read the host FS, phone home to an arbitrary endpoint, spin forever, or exhaust memory. Airbyte's unsafe-code path offers none of these. [P-wasm][airbyte-custom]

**Lean (Topic 6):** if Prism builds a code-connector escape-hatch, base it on **Wasmtime WASI-P2 components** with default-deny `WasiCtxBuilder` + minimal `Linker` host functions (network/FS ONLY via host-mediated capability functions) + fuel + epoch + StoreLimits; Extism is a viable higher-level wrapper if a manifest UX is wanted. Either way it is the **audited, opt-in** boundary (C4 Topic 4 "stronger than Airbyte's UNSAFE_CODE"), reconciled against the existing prism plugin SDK at capture/morph time.

---

## Recommended Prism sanitization architecture (ordered, concrete)

1. **Two-tier sanitization by data class.**
   - **Identifier tier (heavy, at schema-pin time, cached):** byte-cap → UTF-8/trim → reject bidi/control/zero-width codepoints → NFC (`unicode-normalization` 0.1.25) → single-script allowlist + mixed-script/restriction-level (`unicode-script` 0.5.8 + `unicode-security` 0.1.2) → code-point length cap → confusable skeleton (`unicode-security::skeleton`) + collision check → **quarantine+relabel** suspicious (hard-violation = reject). Cache `{raw → label, skeleton, flags, placeholder}` keyed by pinned schema version; invalidate on re-pin.
   - **Value tier (light, bounded, hot path / ingest):** `is_ascii()` fast-path (ASCII → only C0/C1 control check, skip Unicode machinery) → else strip invisible/bidi/zero-width/Unicode-tag + NFC → token-budget cap. **No per-cell skeleton scan.**
2. **Quarantine + relabel (D-C4-4):** deterministic `col_<ordinal>` or `col_<base32(hash(raw_bytes))[..N]>`; per-schema uniqueness set for collision-safety; original retained punycode/base32-encoded in an **audit-only, non-agent-facing** field; emit a structured quarantine event (SAP-1 catalog row — downstream spec dependency).
3. **Structural separation for agent output:** deliver labels + values inside an output-encoded JSON `schema`/`data` envelope, **datamarked by default, base64-encoded for high-risk/suspicious fields** (spotlighting); carry coercion/drift/quarantine/lossy flags in the same envelope.
4. **Action-layer backstop:** read-only default; any write capability separately gated + audited + human-approvable (Prism feature-flag model). Spotlighting is a layer, not the guarantee.
5. **Mandatory for ALL connectors (incl. OCSF sensors)** — satisfied by routing every connector's *identifiers* through the heavy tier at pin time and *values* through the bounded hot-path tier (NOT by skeleton-scanning sensor cells).
6. **WASM escape-hatch (if built):** Wasmtime WASI-P2, default-deny `WasiCtxBuilder`, minimal `Linker`, fuel + epoch + StoreLimits; network/FS only via host-mediated capability functions; audited opt-in; **reconcile against the existing prism plugin SDK at capture/morph time.**

## Open Design Questions

1. **Placeholder scheme:** ordinal (`col_0001`, human-stable) vs content-hash (`col_<base32hash>`, stable across re-pins) — pick per onboarding/UX need. (Lean: content-hash for stability; ordinal for readability.)
2. **NFC vs NFKC:** NFC preserves identity (needed to address real source columns) + separately forbid compatibility chars; confirm Prism does not want NFKC folding (which mutates identity). (Lean: NFC + compatibility-char reject.)
3. **Value-tier rigor for high-risk fields:** which fields (if any) warrant per-value confusable canonicalization vs the default invisible-strip + spotlight? (Lean: only explicitly flagged high-risk fields; default = strip + spotlight.)
4. **Spotlighting variant per surface:** datamark-default vs base64-for-suspicious — and re-validation cadence per model version. (Lean: datamark default, base64 for quarantined/high-risk; re-validate on model upgrade.)
5. **`unicode-security` pre-1.0 + CJK gap:** acceptable for v1, or vendor/pin + add a compensating allowlist for unsupported scripts? (Lean: pin + restriction-level "highly restrictive" default + treat unsupported-script identifiers as quarantine candidates.)
6. **Structured event catalog (SAP-1):** quarantine/relabel/coercion/drift events each likely need a BC-2.16.002 Canonical Structured Event Catalog row — downstream spec dependency, NOT actioned here.
7. **WASM vs existing plugin SDK reconciliation:** internal codebase question for architect+implementer at capture/morph time.

## Honest Costs & Caveats

- **`unicode-security` is pre-1.0 (0.1.2) with documented incompleteness** (open "implement all of UTS#39" tracking issue; open Mar-2025 CJK-ideograph confusable gap). Viable, but not a finished UTS#39 implementation — own the gaps with a conservative restriction-level default and quarantine-on-unsupported. [P-rust][unicode-security-dr]
- **`unicode-skeleton` (2017) and `confusables` (2023, single release) are stale/low-maintenance** — do not adopt; `unicode-security::skeleton` is the maintained path. The intuitive crate pick is the trap. [crates.io]
- **Spotlighting is demonstrated mitigation, NOT a guarantee.** >50%→<2% ASR is benchmark-conditioned (specific models/corpora), model-capability-dependent, token/latency-costly, and defeated by adaptive multi-stage attacks. It must sit atop a least-privilege read-only action layer. [P-val][spotlight-arxiv][owasp-llm01][willison]
- **"Universal per-value skeleton scanning" is NOT established practice** — flagged **[INCONCLUSIVE]**; the tiered architecture is the *defensible inference* from the evidence (identifiers heavy/cached, values light/bounded), not a documented industry standard. [P-val]
- **No public "schema-as-prompt-injection via column name" exploit case study exists** (C4 already flagged this; precautionary posture extrapolated from OWASP LLM01 + CAPEC-146 + CWE-20/400/1007 + UTS#39). [owasp-llm01]
- **No WASM runtime is infallible** — wasmtime shipped 12 advisories (incl. 2 critical) in April-2026; the capability guarantee holds *under a bug-free runtime* and demands prompt patching + (ideally) an LTS pin + defense-in-depth for the most sensitive workloads. [P-wasm]
- **Hot-path cost is genuinely low ONLY with the ASCII fast-path + pin-time identifier caching + scan-on-ingest.** Without those, naively running NFC+script+skeleton per cell would tank sensor-query latency — the mandatory-on-OCSF requirement is affordable *only because* the heavy work is amortized off the per-query path. [P-rust][P-val]
- **Two high-effort research files were partially/un-read** (token-cap, unpaginatable single-line files); the medium-effort inline runs + crates.io version verification fully cover their load-bearing content (see Read-coverage honesty note). Performance *numbers* for NFC/skeleton throughput were qualitative (linear, ASCII-fast, skeleton-heaviest), not benchmarked — flagged **[INCONCLUSIVE]** on absolute throughput figures; the architectural conclusion (tier + cache + ASCII fast-path) does not depend on a specific µs figure.
- **Existing prism plugin SDK reconciliation is explicitly deferred** to capture/morph-time architect+implementer work (codebase task, not external research).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 5 | (1) Rust Unicode-security crate ecosystem + recognizer pipeline + Trojan-Source bidi — Topic 1, `effort=high` (91 KB; read first ~62 KB inline-via-file, the load-bearing crate/pipeline material). (2) Indirect-injection via DATA VALUES, identifiers-vs-values scope, spotlighting, quarantine/relabel — Topics 2/4/5, `effort=high` (85 KB persisted; superseded by the inline medium run below). (3) WASM capability-sandbox deep dive — Topic 6, `effort=high` (87 KB persisted; superseded by the inline medium run below). (4) Spotlighting numbers + identifiers-vs-values cost-tier — Topics 2/5, `effort=medium` (INLINE-READABLE, fully captured). (5) Wasmtime WASI-P2 + DoS bounds + Extism + Airbyte contrast — Topic 6, `effort=medium` (INLINE-READABLE, fully captured). |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 1 | `resolve-library-id` for `unicode-security` — Context7 does NOT carry the niche UTS#39 Rust crates (returned unicode-segmentation/width/names2/ICU instead), so crate-version verification fell back to the crates.io API (more authoritative for version state). DataFusion API state was Context7-verified in the prerequisite C4 file and is not re-verified here. |
| Tavily (all) | 0 | — |
| WebFetch | 9 | crates.io API version verification (the authoritative registry): unicode-security 0.1.2, unicode-skeleton 0.1.1 (2017!), unicode-normalization 0.1.25, decancer 3.3.3, unicode-script 0.5.8, unicode-bidi 0.3.18, confusables 0.1.0 (2023), wasmtime 46.0.1 (LTS 36.x), extism 1.30.0 — all as of 2026-06-27. |
| WebSearch | 0 | — |
| Read | 2 | C4 prerequisite file (grounding); first ~62 KB of the persisted high-effort Rust-crate research result. |
| Grep / Glob | 4 | Locate sections in unpaginatable persisted result files; enumerate `.factory/research/` for index/style. |
| Training data | ~2 areas | Extism-defaults-to-wasmtime (flagged [model-knowledge]); punycode/IDNA as reversible-encode prior art (RFC 3492, widely documented). All flagged inline. |

**Total MCP tool calls:** 6 (5 × `perplexity_research` [3 high-effort + 2 medium] + 1 × Context7). Plus 9 crates.io WebFetch version verifications (load-bearing per the brief's "verify CURRENT Rust crate API state").
**Training data reliance:** low — every crate/runtime version verified against the crates.io API; every mechanism/threat-tier claim web-sourced and source-named; UTS#39/spotlighting/WASI claims cite primary specs/papers/docs. Only Extism's default runtime and punycode-as-prior-art rest on model knowledge, each flagged. Findings date-stamped as of 2026-06-27; technology landscape changes rapidly.

### Citation key

**Crate/runtime versions (crates.io API, 2026-06-27):**
- **[crates.io]** crates.io/api/v1/crates/{unicode-security 0.1.2 · unicode-skeleton 0.1.1 (2017-10-08) · unicode-normalization 0.1.25 · unicode-script 0.5.8 · unicode-bidi 0.3.18 · decancer 3.3.3 · confusables 0.1.0 · wasmtime 46.0.1 / 36.0.12 · extism 1.30.0}.
- **[unicode-security-dr]** docs.rs/unicode-security + github.com/unicode-rs/unicode-security — `confusable_detection::skeleton`, `GeneralSecurityProfile`, `MixedScript`/`is_potential_mixed_script_confusable_char`, `RestrictionLevel(Detection)`, `UNICODE_VERSION`; open UTS#39-tracking + CJK-confusable (Mar-2025) issues.

**Topic 1 (Rust mechanism) — [P-rust] = perplexity_research high-effort (Rust Unicode-security crates):**
- crate APIs (unicode-normalization `.nfc()`/`is_nfc`; unicode-script `UnicodeScript`; unicode-security re-exports; unicode-skeleton legacy; rustc `confusable_idents` / RFC 2457).
- **[trojan-source]** Boucher & Anderson, "Trojan Source: Invisible Vulnerabilities" / CVE-2021-42574 — bidi-override identifier attacks (U+202A–U+202E, U+2066–U+2069, U+200E/F).

**Topics 2/4/5 (values, scope, spotlighting, quarantine) — [P-val] = perplexity_research (medium, inline):**
- **[spotlight-arxiv]** arxiv.org/abs/2403.14720 — Hines et al., "Defending Against Indirect Prompt Injection Attacks With Spotlighting" (delimiting/datamarking/encoding; >50%→<2% ASR).
- **[spotlight-msrc]** microsoft.com/.../msrc/blog/2025/07/how-microsoft-defends-against-indirect-prompt-injection-attacks — Prompt Shields + spotlighting in production.
- **[confusable-llm]** paultendo.github.io/posts/confusable-llm-attack-vectors — NFKC alone does not collapse confusables (only TR39 skeleton does); frontier models correctly read confusable substitutions; `isClean`/`canonicalise`.
- **[owasp-llm01]** genai.owasp.org/llmrisk/llm01-prompt-injection — segregate/denote external content; least-privilege; human approval; no fool-proof prevention.
- **[willison]** simonwillison.net/2023/Nov/27/prompt-injection-explained + simonw.substack.com/p/prompt-injection-explained-with-video — structural separation reduces but does not eliminate; dual-LLM pattern; action layer is the risk.
- **[punycode]** RFC 3492 (Punycode) / IDNA — reversible ASCII encoding of Unicode labels; browser relabel-to-`xn--` quarantine-and-surface precedent. [model-knowledge as applied to placeholder design]
- (corroboration) aclanthology.org/2025.naacl-short.21 "Mixture of Encodings"; github.com/realArcherL/spotlighting-datamarking (~0.8% end-to-end ASR); trendmicro.com invisible-prompt-injection (Unicode-tag U+E0000–U+E007F).

**Topic 6 (WASM) — [P-wasm] = perplexity_research (medium, inline) + docs:**
- **[wasmtime-wasictx]** docs.wasmtime.dev/api/wasmtime_wasi/struct.WasiCtxBuilder.html — defaults: no preopens, addresses denied, ip-name-lookup denied.
- **[wasmtime-config]** docs.wasmtime.dev/api/wasmtime/struct.Config.html — `consume_fuel`, `epoch_interruption`.
- **[wasmtime-store]** docs.rs/wasmtime/latest/wasmtime/struct.Store.html — fuel methods, `set_epoch_deadline`, `Engine::increment_epoch`, default 0 fuel.
- **[wasmtime-resourcelimiter]** docs.rs/wasmtime/latest/wasmtime/trait.ResourceLimiter.html — `memory_growing` etc.; bounds guest allocs only.
- **[wasmtime-storelimits]** docs.wasmtime.dev/api/wasmtime/struct.StoreLimitsBuilder.html — memory/table/instance limits (default 10,000; memory unbounded).
- **[wasmtime-bindgen]** docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html; **[wasmtime-component]** docs.wasmtime.dev/api/wasmtime/component/struct.Component.html.
- WASI design: github.com/WebAssembly/WASI/blob/master/docs/DesignPrinciples.md (no ambient authority); jdriven.com WASI capability-based networking; bytecodealliance.org/articles/wasmtime-security-advisories (Apr-2026, 43.0.x/36.0.x patches).
- **[extism-manifest]** docs.rs/extism/latest/extism/struct.Manifest.html + systemshardening.com/articles/wasm/extism-plugin-security — allowed_hosts/allowed_paths/MaxPages/timeout_ms; least-privilege host fns.
- **[airbyte-custom]** docs.airbyte.com/platform/connector-development/connector-builder-ui/custom-components — UNSAFE/EXPERIMENTAL, no sandboxing, `AIRBYTE_ENABLE_UNSAFE_CODE`.
