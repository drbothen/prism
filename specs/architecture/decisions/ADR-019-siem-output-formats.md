---
document_type: adr
adr_id: "ADR-019"
title: "SIEM Output Formats"
status: PROPOSED
date: "2026-05-03"
version: "0.4"
producer: architect
subsystems_affected: [SS-18]
supersedes: null
superseded_by: null
inputs:
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/research-findings.md
  - .factory/STATE.md
  - .factory/stories/S-4.08-action-delivery.md
  - .factory/specs/architecture/decisions/ADR-012-src-convention.md
  - .factory/specs/architecture/decisions/ADR-016-action-delivery-framework.md
anchor_stories: [S-4.08]
aligns_with: [ADR-012]
references_phase3_siblings: [ADR-016]
locked_decisions: [D-212]
references_research:
  - "R-8: CEF/LEEF Rust ecosystem audit; ArcSight CEF Implementation Standard; IBM QRadar LEEF v2 Format Guide"
verification_properties: [VP-144]
new_workspace_crate: prism-siem-formats
---

# ADR-019: SIEM Output Formats

## Status

PROPOSED 2026-05-03, v0.4. Pending review and acceptance prior to S-4.08 story remediation completion.

## Context

MSSP customers ingest security events into SIEM platforms — predominantly ArcSight (CEF) and
IBM QRadar (LEEF). The Wave 4 Action Delivery Framework (ADR-016) exposes a syslog destination
that must serialize Prism events into one of these wire formats before transmission.

A Rust crate survey conducted during Wave 4 pre-flight (R-8, accessed 2026-05) found:

- **rust-cef**: last commit 2021; unmaintained; no LEEF support; abandoned per R-8.
- **No published LEEF crate** exists in the Rust ecosystem as of R-8 audit date.

Depending on an unmaintained encoder for a security-critical output format creates unacceptable
maintenance risk and a known supply-chain gap. D-212 (LOCKED) formalizes the in-house decision.
The syslog destination defined in ADR-016 §2.10 must have a correct, tested, vendored encoder
beneath it; without one, the action delivery pipeline cannot produce valid CEF or LEEF wire bytes.

This ADR records the crate layout, encoder APIs, escape rules, error contract, proptest
invariants, VP assignment, workspace integration, and maintenance cadence for the
`prism-siem-formats` crate. Transport-layer concerns (syslog socket, TLS, framing) are
exclusively ADR-016's domain; this ADR ends at `Vec<u8>`.

## Decision

We will build `prism-siem-formats`, a new workspace crate, containing in-house CEF v0 and
LEEF 2.0 encoders validated by proptest invariants, following ADR-012 layout conventions,
with no transport-layer or `prism-core` dependencies.

### §1 — Crate Placement and Layout

A new workspace crate following ADR-012 `src/` convention:

```
crates/prism-siem-formats/
├── Cargo.toml
├── src/
│   ├── lib.rs              # root; re-exports CefEncoder, LeefEncoder, FormatError
│   ├── cef/
│   │   ├── mod.rs
│   │   └── v0.rs           # CefEvent struct + Encoder impl
│   ├── leef/
│   │   ├── mod.rs
│   │   └── v2.rs           # LeefEvent struct + Encoder impl
│   └── error.rs            # FormatError enum
└── tests/
    ├── cef_v0_proptest.rs
    ├── leef_v2_proptest.rs
    └── round_trip_proptest.rs
```

`crates/prism-siem-formats` is added to the root `Cargo.toml` `[workspace] members` list.

**Dependencies (minimal)**:

```toml
[dependencies]
serde = { version = "1", default-features = false, features = ["derive"] }
chrono = { version = "0.4", default-features = false, features = ["serde"] }

[dev-dependencies]
proptest = "1"
```

No transport-layer dependencies. No syslog crate. No `prism-core` dependency (pure format
logic; no business types from core). Encoders return `Vec<u8>`; the caller (`prism-operations`
syslog destination, per ADR-016 §2.10) handles all socket I/O and framing.

### §2 — CEF v0 Encoder API

```rust
pub mod cef {
    pub mod v0 {
        use std::collections::BTreeMap;
        use crate::error::FormatError;

        pub struct CefEvent {
            pub device_vendor: String,
            pub device_product: String,
            pub device_version: String,
            pub signature_id: String,
            pub name: String,
            pub severity: u8,                         // 0..=10 enforced in constructor
            pub extension: BTreeMap<String, String>,  // BTreeMap for deterministic output
        }

        impl CefEvent {
            /// Returns FormatError::SeverityOutOfRange if severity > 10.
            pub fn new(/* fields */) -> Result<Self, FormatError>;
        }

        pub struct Encoder { _priv: () }

        impl Encoder {
            pub fn new() -> Self;
            pub fn encode(&self, event: &CefEvent) -> Result<Vec<u8>, FormatError>;
            pub fn encode_into(
                &self,
                event: &CefEvent,
                buf: &mut Vec<u8>,
            ) -> Result<(), FormatError>;
        }
    }
}
```

### §3 — CEF v0 Format Rules

Per the ArcSight CEF Implementation Standard (R-8):

**Wire format**:
```
CEF:0|Device Vendor|Device Product|Device Version|Signature ID|Name|Severity|Extension
```

**Header field escaping** (fields 2–7):

| Character | Encoded form |
|-----------|-------------|
| `\` | `\\` |
| pipe `\|` | `\|` |
| LF `\n` | `FormatError::InvalidHeaderChar` — hard rejection |
| CR `\r` | `FormatError::InvalidHeaderChar` — hard rejection |

**Extension escaping** (key=value block after field 8):

| Character | Encoded form |
|-----------|-------------|
| `\` | `\\` |
| `=` | `\=` |
| pipe `\|` | NOT escaped (allowed in extension values per spec) |
| `\n` | two-char literal `\n` |
| `\r` | two-char literal `\r` |

**Severity**: integer 0–10 only. Wave 4 emits ONLY integer form for determinism; legacy
string forms (`Low`, `Medium`, `High`, `Very-High`) are not produced.

**OCSF `severity_id` to CEF severity mapping:** The encoder accepts a CEF severity integer (0–10). Downstream callers (S-4.08 syslog destination) are responsible for mapping OCSF `severity_id` to CEF severity before calling `CefEvent::new`:

| OCSF `severity_id` | OCSF Label | CEF Severity |
|--------------------|------------|-------------|
| 0 | Unknown | 0 |
| 1 | Informational | 1–3 (use 2 as canonical midpoint) |
| 2 | Low | 4 |
| 3 | Medium | 6 |
| 4 | High | 8 |
| 5 | Critical | 10 |

This mapping is NOT enforced by the encoder (the encoder accepts any valid 0–10 integer). The mapping is documented here as a convention for `prism-operations` callers; the `prism-siem-formats` crate remains caller-mapping-agnostic.

**Determinism**: `BTreeMap` ensures lexicographic key ordering; `encode(event) == encode(event)`
for any valid input (INV-CEF-005).

### §4 — LEEF 2.0 Encoder API

```rust
pub mod leef {
    pub mod v2 {
        use std::collections::BTreeMap;
        use crate::error::FormatError;

        pub enum Delim {
            Tab,           // default; encoded as \t (0x09)
            Pipe,          // |
            Caret,         // ^
            Custom(char),  // validated against forbidden set on construction (see §5 below)
        }

        pub struct LeefEvent {
            pub vendor: String,
            pub product: String,
            pub version: String,
            pub event_id: String,
            pub delim: Delim,
            pub attributes: BTreeMap<String, String>,
        }

        pub struct Encoder {
            syslog_prefix: Option<(String, String)>, // (host, app)
        }

        impl Encoder {
            /// No syslog prefix; suitable for non-syslog transports.
            pub fn new() -> Self;
            /// Prepends RFC 3164/5424 syslog header before LEEF header.
            pub fn with_syslog_header(host: String, app: String) -> Self;
            pub fn encode(&self, event: &LeefEvent) -> Result<Vec<u8>, FormatError>;
        }
    }
}
```

### §5 — LEEF 2.0 Format Rules

Per the IBM QRadar LEEF v2 Format Guide (R-8):

**Wire format (no syslog prefix)**:
```
LEEF:2.0|Vendor|Product|Version|EventID|DelimChar|attr1=val1<delim>attr2=val2...
```

**Wire format (with syslog prefix)**:
```
<syslog-header> LEEF:2.0|Vendor|Product|Version|EventID|DelimChar|attr1=val1<delim>...
```

Syslog header and LEEF header are separated by a single space. Format follows RFC 3164
by default.

**Delimiter field (6th pipe-delimited field)**: always emitted explicitly for robustness,
even for `Delim::Tab`. Custom delimiters are emitted as the raw character.

**`Delim::Custom(c)` validation — forbidden set:** The following characters are rejected at `Delim::Custom` construction, returning `FormatError::InvalidAttributeKey` with reason `"forbidden delimiter"`:

| Character | Reason |
|-----------|--------|
| Tab `\t` (0x09) | Reserved for `Delim::Tab` |
| Pipe `\|` | Reserved for `Delim::Pipe` and LEEF header delimiters |
| Caret `^` | Reserved for `Delim::Caret` |
| Newline `\n` (0x0A) | Record separator in syslog |
| Carriage return `\r` (0x0D) | Record separator in syslog |
| Equals `=` | LEEF attribute key-value separator |
| Double-quote `"` | Ambiguous quoting in SIEM parsers |
| NUL `\0` (0x00) | C-string terminator; invalid in syslog framing |

Allowed: any other ASCII printable character or single-codepoint Unicode character not in the forbidden set above.

**Attribute key rules**: keys containing tab (`\t`), pipe (`|`), or caret (`^`) are rejected
with `FormatError::InvalidAttributeKey`. Safe characters: `A-Z`, `a-z`, `0-9`, `_`, `-`, `.`.

**Attribute value rules**: if a value contains the chosen delimiter, return
`FormatError::DelimiterCollision`. All other characters are permitted.

**QRadar built-in attribute keys** (`src`, `dst`, `srcPort`, `dstPort`, `usrName`, `proto`,
`cat`, `sev`, `devTime`, `devTimeFormat`) are documented in module-level doc comments;
the encoder does NOT enforce these keys.

### §6 — Error Type

```rust
pub enum FormatError {
    InvalidHeaderChar { field: &'static str, ch: char },
    SeverityOutOfRange(u8),
    InvalidAttributeKey { key: String, reason: &'static str },
    DelimiterCollision { key: String, value_excerpt: String },
    EmptyRequiredField(&'static str),
}
```

Single flat enum — no nested error chains — for exhaustive downstream matching in
`prism-operations` without `source()` traversal.

### §7 — Proptest Invariants

**`tests/cef_v0_proptest.rs`**:
- **INV-CEF-001**: For arbitrary valid `CefEvent`: output starts with `CEF:0|`; contains
  exactly 7 unescaped pipes; extension round-trips correctly.
- **INV-CEF-002**: Header field with `\n` returns `Err(InvalidHeaderChar)`.
- **INV-CEF-003**: `CefEvent::new` with severity > 10 returns `Err(SeverityOutOfRange)`.
- **INV-CEF-004**: Extension value with `=` produces `\=` in output.
- **INV-CEF-005**: Idempotence — `encode(event) == encode(event)`.

**`tests/leef_v2_proptest.rs`**:
- **INV-LEEF-001**: Output starts with `LEEF:2.0|` (or `<syslog-prefix> LEEF:2.0|`).
- **INV-LEEF-002**: Key with tab/pipe/caret returns `Err(InvalidAttributeKey)`.
- **INV-LEEF-003**: Value with chosen delimiter returns `Err(DelimiterCollision)`.
- **INV-LEEF-004**: Non-tab custom delimiter is always emitted as 6th pipe field.
- **INV-LEEF-005**: Idempotence.

**`tests/round_trip_proptest.rs`**:
- **INV-RT-001**: Arbitrary valid `CefEvent`: encode → parse → re-encode produces
  byte-identical output.
- **INV-RT-002**: Arbitrary valid `LeefEvent`: same byte-stability guarantee.
- **INV-RT-003**: No SIEM-toxic characters survive in illegal positions: raw `|` never
  unescaped in a CEF header field; raw `=` never unescaped in a CEF extension key;
  chosen delimiter never unescaped in a LEEF attribute value.

### §8 — Verification Property

**VP-144** — CEF v0 + LEEF 2.0 encoder correctness.

Scope: INV-CEF-001..005, INV-LEEF-001..005, INV-RT-001..003 all hold across arbitrary
valid and invalid inputs. Tool: proptest. Phase: P1. Module: `prism-siem-formats`.
Story: S-4.08.

VP-143 is reserved for sibling ADR-016 (Action Delivery Framework); VP-144 is used here
with no collision.

### §9 — Workspace Integration

1. Add `"crates/prism-siem-formats"` to root `Cargo.toml` `[workspace] members`.
2. `prism-operations` adds: `prism-siem-formats = { path = "../prism-siem-formats" }`.
3. No `prism-core` dependency; consumers map their domain types to `CefEvent` / `LeefEvent`.
4. ADR-012 `src/` layout compliance confirmed per §1.
5. **ARCH-INDEX SS-18 update (story-writer / state-manager task):** In the same burst that introduces `prism-siem-formats`, update the ARCH-INDEX.md SS-18 (Action Delivery Engine) subsystem registry entry to include `prism-siem-formats` in the crate list. This ensures the subsystem-to-crate mapping remains accurate.

### §10 — Maintenance and Version Cadence

- Crate version: `0.1.0` at Wave 4 introduction; `1.0.0` after Wave 5 production validation.
- ArcSight CEF v0: stable (no v1 in production per R-8). Quarterly spec drift review.
- IBM LEEF 2.0: stable per R-8. Quarterly spec drift review.
- `rust-cef` tombstone: do not re-evaluate; decision locked per D-212.

## Rationale

The absence of a maintained Rust encoder for either format is the primary forcing function.
`rust-cef` has been abandoned since 2021 (R-8); no LEEF crate exists at all. For a security
product whose output feeds customer SIEM platforms, a stale or unvalidated encoder is a direct
path to silent data loss or SIEM parse failures — both of which erode analyst trust and create
support escalations.

Building in-house provides full control over escape-rule fidelity. CEF and LEEF have subtle
escaping asymmetries: CEF allows pipe in extension values but not headers; LEEF uses a
configurable attribute delimiter whose collision must be detected per-event. These rules are
best verified with proptest generating adversarial strings, not manual unit tests.

The pure `Vec<u8>` output contract (no transport dependency) satisfies the separation of
concerns demanded by ADR-016: the delivery subsystem owns framing and socket I/O; the format
crate owns wire bytes only. This boundary also enables future reuse — a file-sink or SFTP
batch export destination can consume the same encoder without depending on syslog machinery.

BTreeMap for extension and attribute ordering guarantees deterministic output, which is a
testability requirement (INV-CEF-005, INV-LEEF-005) and a correctness aid for SIEM correlation
engines that parse repeated events from the same source.

The flat `FormatError` enum (no `source()` chain) is the correct choice for a format-only
crate. Callers perform simple action-on-error (log and skip, or substitute a sanitized event);
they do not need cause chains. This keeps the error handling surface minimal and the crate
dependency-free from `thiserror` or `anyhow`.

## Consequences

### Positive

- Format encoders are fully vendored; no third-party crate maintenance risk or supply-chain exposure.
- Proptest coverage (13 invariants across 3 test files) validates escaping correctness for
  arbitrary inputs, including adversarial payloads generated by a property engine.
- Pure `Vec<u8>` output is maximally composable — compatible with any transport (syslog,
  file sink, SFTP batch, test capture).
- BTreeMap extension ordering guarantees deterministic output, simplifying test assertions
  and SIEM log correlation.
- Small, single-purpose crate with minimal compile-time footprint (3 runtime deps, 1 dev dep).

### Negative / Trade-offs

- Encoder spec drift is now an in-house responsibility; quarterly review obligation per §10.
- +1 workspace crate to maintain, document, and gate in CI.
- Round-trip tests (INV-RT-001/002) require a naive in-test parser used solely for test
  purposes; this is test-only code but adds a bounded maintenance surface.

### Status as of 2026-05-02

PROPOSED. `prism-siem-formats` does not yet exist in the workspace. Implementation begins
in Wave 4 story S-4.08; VP-144 transitions to `verified` after S-4.08 proptest suite passes
in CI.

## Alternatives Considered

- **Depend on rust-cef**: REJECTED. Last commit 2021; no LEEF support; known escaping gaps
  per R-8. Unacceptable for a security product emitting events to customer SIEMs (D-212).
- **Defer LEEF 2.0 to a future wave; ship CEF-only in Wave 4**: REJECTED per D-212.
  IBM QRadar customers represent a material segment of the MSSP target base; shipping an
  incomplete SIEM integration creates adoption blockers.
- **Webhook-only output; skip syslog/SIEM in Wave 4**: REJECTED. Syslog is the dominant
  ingest path for on-premises SIEM in OT/ICS environments (Armis, Claroty customer base).
  Webhook-only would exclude a significant portion of the target customer footprint.
- **Embed encoding inline in prism-operations syslog destination**: REJECTED. Coupling
  format logic to the transport layer makes escape-rule changes ripple into the delivery
  subsystem. A dedicated crate enables independent versioning, isolated proptest coverage,
  and future reuse by other output destinations.

## Phase 4.A Pass 3 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 3 fix-burst (2026-05-02). Version bumped 0.2 → 0.3.

- **P3-ADR-019-A-H-003 fix (ADR-016 section reference drift):** All three occurrences of "ADR-016 §10" corrected to "ADR-016 §2.10" (Syslog Destination — CEF / LEEF). Lines 44, 95, and 416 (Source/Origin section).

## Phase 4.A Pass 1 Remediation Notes

Applied during Wave 4 Phase 4.A adversarial Pass 1 fix-burst (2026-05-02). Version bumped 0.1 → 0.2.

- **P1-ADR-019-A-H-001 fix:** `subsystems_affected: [SS-18]` was already correct; no frontmatter change needed. §9 (Workspace Integration) now includes an explicit task: "Story-writer / state-manager: in the same burst that introduces `prism-siem-formats`, update ARCH-INDEX.md SS-18 entry to include the new crate."
- **P1-ADR-019-A-M-002 fix:** `Delim::Custom(c)` forbidden character set specified in §5 (LEEF 2.0 Format Rules). Eight forbidden characters: `\t`, `|`, `^`, `\n`, `\r`, `=`, `"`, `\0`. Allowed: any other ASCII printable or single-codepoint Unicode.
- **P1-ADR-019-A-M-004 fix:** OCSF `severity_id` to CEF severity mapping table added to §3 (CEF v0 Format Rules). Mapping: 0→0, 1→2, 2→4, 3→6, 4→8, 5→10. Encoder remains caller-mapping-agnostic; S-4.08 syslog destination is responsible for applying the mapping.

## Changelog

| Version | Change ID | Date | Author | Notes |
|---------|-----------|------|--------|-------|
| 0.4 | F-PSweep-H-001 | 2026-05-03 | architect | Proactive structural sweep fix: added `## Status` H2 body section matching siblings ADR-013/015/016/017/018 — was the only Wave 4 ADR missing it; v0.3 → v0.4. |
| 0.3 | P3-ADR-019-A-H-003 | 2026-05-02 | architect | All three occurrences of "ADR-016 §10" corrected to "ADR-016 §2.10"; v0.2 → v0.3. |
| 0.2 | P1-ADR-019-A-H-001 | 2026-05-02 | architect | Version bumped 0.1 → 0.2 per Pass 1 fix-burst. |
| 0.1 | P1-ADR-019-A-H-001, P1-ADR-019-A-M-002, P1-ADR-019-A-M-004 | 2026-05-02 | architect | Initial ADR; SS-18 subsystem confirmed; Delim::Custom forbidden set specified; OCSF severity_id → CEF mapping table added. |

---

## Source / Origin

- **D-212** (LOCKED): Wave 4 pre-flight architectural decision log — in-house CEF + LEEF
  encoder mandated after R-8 Rust ecosystem audit found no maintained crates.
- **R-8**: `.factory/cycles/wave-4-operations/preflight-findings/research-findings.md` —
  CEF/LEEF Rust ecosystem audit, accessed 2026-05.
- **ADR-016 §2.10**: Action Delivery Framework syslog destination — identifies `prism-siem-formats`
  as the required dependency for wire-format serialization.
- **S-4.08**: `.factory/stories/S-4.08-action-delivery.md` — anchor story whose acceptance
  criteria require both CEF and LEEF syslog output to be functional.
- **ArcSight CEF Implementation Standard**: https://www.microfocus.com/documentation/arcsight/arcsight-smartconnectors/cef-implementation-standard/
- **IBM QRadar LEEF v2 Format Guide**: https://www.ibm.com/support/knowledgecenter/SS42VS_DSM/com.ibm.dsm.doc/b_dsm_guide.pdf
