//! prism-security — Prompt Injection Defense (SS-09).
//!
//! S-1.10: Four-layer prompt injection defense system:
//!   1. Structural separation of untrusted data (BC-2.09.001)
//!   2. Provenance framing in tool descriptions (BC-2.09.002)
//!   3. Suspicious pattern detection with NFKC normalization (BC-2.09.003)
//!   4. Trust-level metadata on every response (BC-2.09.005)
//!
//! Scanning logic is pure. The only effectful component is `OnceLock<RegexSet>`
//! initialization. Mixed purity per `architecture/purity-boundary-map.md`.

#![allow(unexpected_cfgs)]

pub mod injection_scanner;
pub mod output_schema;
pub mod provenance;
pub mod trust_level;

// ─────────────────────────────────────────────────────────────
// Public re-exports
// ─────────────────────────────────────────────────────────────

pub use injection_scanner::{InjectionScanner, ScanInput, ScanResult};
pub use output_schema::{MetaEnvelopeSchema, OutputSchema, ResultsItemSchema};
pub use provenance::{ProvenanceFraming, SecurityWarning, ToolDescriptionTemplate};
pub use trust_level::TrustLevelExt;
