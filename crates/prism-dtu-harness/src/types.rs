//! Core harness types: `IsolationMode`, `DtuType`, `OrgKey`, `CustomerSpec`.
//!
//! These are pure-core value types — no I/O, no async.
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.1 — `IsolationMode` variants (Logical, Network)
//! - ADR-011 §2.2 — `(OrgId, DtuType)` keyed HashMap for logical-mode routing
//! - D-059          — Device ID prefix format `dev-{org_slug}-{seed}-{index}`

use prism_core::ids::OrgId;
use prism_core::tenant::OrgSlug;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// IsolationMode
// ---------------------------------------------------------------------------

/// The isolation strategy used when the harness spins up DTU clone instances.
///
/// `#[non_exhaustive]` — future waves may add `IsolationMode::Process` or
/// `IsolationMode::Container` without breaking downstream match arms.
///
/// (ADR-011 §2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IsolationMode {
    /// In-process isolation: all clones run as Tokio tasks in the same OS process.
    ///
    /// Per-org state is segregated via `(OrgId, DtuType)`-keyed HashMaps.
    /// Each clone binds a distinct OS-assigned loopback TCP port.
    ///
    /// This is the mode implemented by S-3.3.03.
    /// (ADR-011 §2.2; BC-3.5.001)
    Logical,

    /// Network isolation: each `(OrgId, DtuType)` gets its own OS-assigned TCP port.
    ///
    /// All listeners are pre-allocated simultaneously before any `start_on` call
    /// (D-058 pre-allocate rule: no retry-on-EADDRINUSE loop). The
    /// `customer_endpoints` table is populated atomically during `build()` and
    /// is immutable for the harness lifetime (BC-3.5.002 Invariant 1).
    ///
    /// Catches cross-process routing bugs — a request bearing `OrgId(A)`
    /// credentials routed to `OrgId(B)`'s port receives HTTP 401 from the wrong
    /// clone's auth middleware, making the routing error observable.
    ///
    /// Implemented by S-3.3.04 (BC-3.5.002).
    /// (ADR-011 §2.3; BC-3.5.002)
    Network,
}

// ---------------------------------------------------------------------------
// DtuType
// ---------------------------------------------------------------------------

/// Identifies the type of DTU behavioral clone.
///
/// Strongly-typed enum for use as a `HashMap` key in harness data structures.
/// Variants correspond to the sensor types registered in `prism-core::dtu`
/// (the authoritative classification source per ADR-007 §2.3).
///
/// `#[non_exhaustive]` — new sensor integrations in future waves (e.g., S-6.11
/// through S-6.19) add new variants without breaking downstream match arms.
///
/// (ADR-011 §2.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DtuType {
    /// Claroty xDome OT/IoT asset management.
    Claroty,
    /// Armis asset intelligence platform.
    Armis,
    /// CrowdStrike Falcon endpoint security.
    CrowdStrike,
    /// Cyberint threat intelligence.
    Cyberint,
    /// PagerDuty incident management (Shared mode).
    PagerDuty,
    /// Jira issue tracker (Shared mode).
    Jira,
    /// Slack notification hub (Shared mode).
    Slack,
    /// NVD vulnerability database (Shared mode).
    Nvd,
    /// ThreatIntel feed aggregator (Shared mode).
    ThreatIntel,
    /// Demo/test-only server. Not for production use.
    DemoServer,
}

impl std::fmt::Display for DtuType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DtuType::Claroty => "claroty",
            DtuType::Armis => "armis",
            DtuType::CrowdStrike => "crowdstrike",
            DtuType::Cyberint => "cyberint",
            DtuType::PagerDuty => "pagerduty",
            DtuType::Jira => "jira",
            DtuType::Slack => "slack",
            DtuType::Nvd => "nvd",
            DtuType::ThreatIntel => "threatintel",
            DtuType::DemoServer => "demo-server",
        };
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// OrgKey
// ---------------------------------------------------------------------------

/// Composite key for per-clone state in harness `HashMap`s.
///
/// Every harness-internal map that holds per-clone state (endpoints,
/// crash channels, shutdown senders, task handles) is keyed on `OrgKey`.
///
/// (ADR-011 §2.2 — `(OrgId, DtuType)`-keyed HashMaps)
pub type OrgKey = (OrgId, DtuType);

// ---------------------------------------------------------------------------
// CustomerSpec
// ---------------------------------------------------------------------------

/// Per-customer configuration for a single harness registration.
///
/// Built by `HarnessBuilder::with_customer` (from registry lookup) and
/// optionally mutated by `HarnessBuilder::with_customer_overrides`.
///
/// (Story S-3.3.03, Task 3; BC-3.5.001 precondition 2)
#[derive(Debug, Clone)]
pub struct CustomerSpec {
    /// Canonical org identity.
    pub org_id: OrgId,

    /// Display slug (e.g., `"acme-corp"`).
    pub org_slug: OrgSlug,

    /// Which DTU clone types to spin up for this org.
    ///
    /// Defaults to all four Security Telemetry types (Claroty, Armis,
    /// CrowdStrike, Cyberint) for Client-mode orgs.
    pub dtu_types: Vec<DtuType>,

    /// Deterministic RNG seed passed to each clone's `StubConfig`.
    ///
    /// Used for reproducible fixture generation per D-059.
    /// Convention: `dev-{org_slug}-{seed}-{index}` device ID format.
    pub seed: u64,

    /// Artificial latency (ms) to inject per clone via `LatencyLayer`.
    ///
    /// Defaults to 0 (no artificial latency).
    pub latency_ms: u64,

    /// Test hook: force a specific bind address instead of `127.0.0.1:0`.
    ///
    /// Used to exercise `PortConflict` error paths in tests (BC-3.5.001 EC-003).
    /// `None` means OS-assigned ephemeral port on loopback (the default).
    pub bind_override: Option<std::net::SocketAddr>,

    /// Test hook: artificial startup delay (ms) injected before the clone task starts.
    ///
    /// Used to exercise `StartupTimeout` error paths in tests (BC-3.5.001 EC-005, D-058).
    /// `None` means no artificial delay (the default).
    pub startup_delay_ms: Option<u64>,
}

impl CustomerSpec {
    /// Create a `CustomerSpec` with default settings for the given org.
    ///
    /// Starts all four Security Telemetry DTU types with seed=42 and no latency.
    pub fn new(org_id: OrgId, org_slug: OrgSlug) -> Self {
        Self {
            org_id,
            org_slug,
            dtu_types: vec![
                DtuType::Claroty,
                DtuType::Armis,
                DtuType::CrowdStrike,
                DtuType::Cyberint,
            ],
            seed: 42,
            latency_ms: 0,
            bind_override: None,
            startup_delay_ms: None,
        }
    }
}
