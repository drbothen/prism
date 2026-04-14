# Feature Flag Systems for Prism MCP Server

**Date:** 2026-04-13
**Type:** General (technology research)
**Status:** Complete
**Purpose:** Evaluate runtime feature flag approaches for gating dangerous write operations per-client in a Rust MCP server.

---

## Table of Contents

1. [Rust Crates for Runtime Feature Flags](#1-rust-crates-for-runtime-feature-flags)
2. [Compile-Time vs Runtime Tradeoffs](#2-compile-time-vs-runtime-tradeoffs)
3. [Feature-Gated MCP Tools Pattern](#3-feature-gated-mcp-tools-pattern)
4. [Confirmation and Approval for Dangerous Operations](#4-confirmation-and-approval-for-dangerous-operations)
5. [Per-Client Config in MSSP Tools](#5-per-client-config-in-mssp-tools)
6. [Existing MCP Servers with Feature Flags](#6-existing-mcp-servers-with-feature-flags)
7. [Recommendation](#7-recommendation)

---

## 1. Rust Crates for Runtime Feature Flags

### Assessment: No Mature Rust-Native Runtime Feature Flag Crate Exists for This Use Case

The Rust ecosystem's "feature flag" landscape is dominated by **compile-time** cargo features, not runtime toggles. The crates that do exist fall into two categories:

#### Category A: Bitflag/Enum Libraries (Not What We Need)

| Crate | What It Actually Is | Relevance |
|-------|-------------------|-----------|
| `flagset` | Compile-time bitflag enums (like C bitfields). Generates `enum` types with bitwise operations. | **Not relevant.** These are type-level flags, not runtime toggles. |
| `bitflags` | Same category as flagset -- bitwise flag manipulation. | **Not relevant.** |
| `enumflags2` | Same category. | **Not relevant.** |

#### Category B: Remote Feature Flag SDKs

| Crate | What It Is | Last Known Status | Relevance |
|-------|-----------|------------------|-----------|
| `unleash-client-rust` | SDK for Unleash feature flag service (requires running Unleash server). | Active project, but requires an external server. | **Overkill.** Prism runs as a per-analyst local process. No remote server. |
| `growthbook-sdk-rust` | SDK for GrowthBook (remote A/B testing + feature flags). | Exists but targets SaaS/web use cases. | **Wrong domain.** A/B testing, not security operation gating. |
| `openfeature` | OpenFeature SDK for Rust (vendor-neutral feature flag protocol). | Early stage as of mid-2025. | **Interesting standard but immature.** Requires a provider backend. |

#### Category C: What We Actually Need

**None of the existing crates match Prism's requirements.** Prism needs:
- Config-file-driven (TOML), not remote-service-driven
- Per-client scoping (not global toggles)
- Hierarchical flag paths (`sensor.crowdstrike.write`)
- Audit logging on check/toggle
- Safe defaults (deny by default)

**This is a ~150-line custom implementation**, not a third-party dependency. The data structure is a `HashSet<String>` or tree of permission paths loaded from TOML config, checked at tool dispatch time.

> **CONFIDENCE: MEDIUM.** This assessment is based on training data (knowledge cutoff May 2025). WebSearch and Context7 tools were unavailable to verify current crate registry state. A crate published after May 2025 could exist. **Verification needed:** `cargo search feature-flag` and check crates.io before finalizing.

---

## 2. Compile-Time vs Runtime Tradeoffs

### Recommendation: Two-Tier Approach

| Layer | Mechanism | Purpose |
|-------|-----------|---------|
| **Tier 1: Cargo features** | `--features crowdstrike-write` | Binary distribution control. An MSSP operator can build a Prism binary that physically cannot contain write code. Defense in depth -- the code doesn't exist in the binary. |
| **Tier 2: Runtime flags** | TOML config per client | Per-client enablement within a binary that has write capability compiled in. The analyst's config determines which clients have which write operations enabled. |

### Why Both Tiers

**Tier 1 alone is insufficient** because Prism is a single binary serving multiple clients. You cannot have different cargo features per client at runtime.

**Tier 2 alone is insufficient** because defense-in-depth matters for security tooling. If the binary physically cannot execute `crowdstrike_contain_host()` because the function was never compiled, no runtime config mistake can activate it.

### Implementation Pattern

```toml
# Cargo.toml
[features]
default = ["read-all"]
read-all = ["crowdstrike-read", "claroty-read", "armis-read", "cyberint-read"]
crowdstrike-write = []
claroty-write = []
armis-write = []
all-write = ["crowdstrike-write", "claroty-write", "armis-write"]
```

```rust
// In tool registration:
#[cfg(feature = "crowdstrike-write")]
{
    if client_config.has_flag("sensor.crowdstrike.containment") {
        registry.register(CrowdStrikeContainTool::new());
    }
}
```

The `#[cfg]` gate is compile-time. The `has_flag()` check is runtime. Both must pass.

---

## 3. Feature-Gated MCP Tools Pattern

### Key Decision: Hidden vs Visible-But-Disabled

There are two approaches for tools gated behind feature flags:

| Approach | Behavior | Pros | Cons |
|----------|----------|------|------|
| **Hidden** | Tool does not appear in `tools/list` response | Clean tool list. AI agent cannot attempt disabled operations. No confusion. | Agent cannot explain WHY an operation isn't available. No discoverability. |
| **Visible-but-disabled** | Tool appears in `tools/list` with annotation, returns structured error on call | Agent can explain "containment is available but not enabled for this client." Discoverability. | Clutters tool list. Agent might repeatedly attempt disabled operations. |

### Recommendation: Hidden by Default, with a Meta-Tool for Discovery

**Do not expose disabled tools in `tools/list`.** Instead:

1. **Conditionally register tools** at server initialization based on the active client's flags.
2. **Provide a `list_capabilities` meta-tool** that returns all possible tools and their enablement status per client. This lets the agent discover what's possible without cluttering the active tool list.

```
Tool: list_capabilities
Input: { "client_id": "acme-corp" }
Output: {
  "sensor.crowdstrike.read": { "enabled": true },
  "sensor.crowdstrike.containment": { "enabled": false, "reason": "Not enabled in client config" },
  "sensor.crowdstrike.rtr": { "enabled": false, "reason": "Feature not compiled (crowdstrike-write)" }
}
```

This is the pattern the axiathon reference uses -- plugins declare capabilities in their manifest, and the host decides what to register. The `PluginManifest.permissions` field (`["network:listen", "file:read"]`) is the same concept.

### MCP Protocol Consideration

MCP's `tools/list` is called once at session start (or on `notifications/tools/list_changed`). Prism runs as stdio per-analyst, so the tool list is effectively static per session. If the analyst switches client context mid-session, Prism should:

1. Re-evaluate feature flags for the new client
2. Send `notifications/tools/list_changed` to trigger the MCP client to re-fetch `tools/list`
3. The new tool list reflects the new client's enabled tools

This is clean and spec-compliant.

---

## 4. Confirmation and Approval for Dangerous Operations

### Patterns from Security Tooling

#### Pattern 1: Dry-Run Default

All write operations have a `dry_run: bool` parameter (default `true`). The agent must explicitly set `dry_run: false` to execute. This adds a natural confirmation step in conversation:

```
Agent: "I'll contain host X. Let me first do a dry run."
[calls contain_host(host_id: X, dry_run: true)]
"Dry run successful. This would isolate host X from the network. Shall I proceed?"
User: "Yes, proceed."
[calls contain_host(host_id: X, dry_run: false)]
```

#### Pattern 2: Confirmation Token

The first call returns a confirmation token with details of the action. The second call must include the token:

```rust
// Step 1: Request
contain_host(host_id: "abc") -> {
    "confirmation_required": true,
    "token": "ct_7f3a...",
    "summary": "Isolate host abc (10.0.1.5) from network",
    "expires_in_seconds": 300
}

// Step 2: Confirm
confirm_action(token: "ct_7f3a...") -> { "status": "executed" }
```

This is stronger than dry-run because the token is tamper-evident and time-bounded. CrowdStrike's own API uses a similar pattern -- Real Time Response (RTR) sessions require establishing a session before executing commands.

#### Pattern 3: Risk Classification with Tiered Gates

| Risk Level | Gate | Example |
|------------|------|---------|
| **Low** (read) | No gate | List alerts, get device info |
| **Medium** (reversible write) | Dry-run default | Acknowledge alert, add tag |
| **High** (irreversible write) | Confirmation token + audit log | Contain host, quarantine file |
| **Critical** (destructive) | Not exposed via MCP at all | Delete sensor, wipe endpoint |

### Recommendation for Prism

Combine Pattern 1 and Pattern 3:

- **Read operations:** No gate.
- **Reversible writes:** `dry_run: true` default. Single-step execution with `dry_run: false`.
- **Irreversible writes (containment, network isolation):** Confirmation token pattern. Two MCP tool calls required.
- **Destructive operations:** Not exposed. Period.

Audit logging is mandatory for all write operations regardless of tier.

---

## 5. Per-Client Config in MSSP Tools

### Typical MSSP Config Schema Pattern

MSSP platforms universally follow this pattern: a **global config** with per-client overrides. The merge order is: `defaults < client-specific`.

### Recommended TOML Schema for Prism

```toml
# prism.toml -- Global config

[defaults]
# These apply to all clients unless overridden
log_level = "info"

[defaults.capabilities]
# Safe defaults: everything off
sensor_read = true           # Read is safe, enabled by default
sensor_write = false         # All writes off by default
containment = false
real_time_response = false
alert_acknowledge = false

# Per-client configuration
[clients.acme-corp]
display_name = "Acme Corporation"

[clients.acme-corp.sensors.crowdstrike]
enabled = true
api_base = "https://api.crowdstrike.com"
# credential_ref points to a secret store, never inline
credential_ref = "vault://acme-corp/crowdstrike"

[clients.acme-corp.capabilities]
# Override defaults for this client
sensor_write = true
containment = true
# real_time_response remains false (inherits default)

[clients.globex]
display_name = "Globex Corporation"

[clients.globex.sensors.crowdstrike]
enabled = true
api_base = "https://api.crowdstrike.com"
credential_ref = "vault://globex/crowdstrike"

# No capabilities override -- inherits all defaults (read-only)
```

### Hierarchical Flag Resolution

The capability path `sensor.crowdstrike.containment` resolves as:

1. Check `clients.{id}.capabilities.sensor.crowdstrike.containment` -- explicit per-client per-sensor
2. Check `clients.{id}.capabilities.containment` -- per-client category
3. Check `clients.{id}.capabilities.sensor_write` -- per-client broad write toggle
4. Check `defaults.capabilities.sensor.crowdstrike.containment` -- global per-sensor
5. Check `defaults.capabilities.containment` -- global category
6. Check `defaults.capabilities.sensor_write` -- global broad write toggle
7. **Default: false** -- deny if nothing matched

This is a standard hierarchical override pattern. The key principle: **more-specific flags override less-specific ones, and the final fallback is always deny.**

### Implementation in Rust

```rust
use std::collections::HashSet;

/// Resolved capability set for a specific client.
/// Built at config load time by merging defaults + client overrides.
pub struct ClientCapabilities {
    client_id: String,
    /// Flat set of enabled capability paths.
    /// e.g., {"sensor.crowdstrike.read", "sensor.crowdstrike.containment"}
    enabled: HashSet<String>,
}

impl ClientCapabilities {
    /// Check if a capability is enabled.
    /// Checks exact match first, then walks up the hierarchy.
    /// "sensor.crowdstrike.containment" checks:
    ///   1. "sensor.crowdstrike.containment" (exact)
    ///   2. "sensor.crowdstrike.write" (parent category)
    ///   3. "sensor.crowdstrike" (sensor-level)
    ///   4. "sensor.write" (global write)
    pub fn is_enabled(&self, capability: &str) -> bool {
        // Exact match
        if self.enabled.contains(capability) {
            return true;
        }
        // Walk up: "a.b.c" -> check "a.b", then "a"
        let mut path = capability;
        while let Some(parent) = path.rsplit_once('.').map(|(p, _)| p) {
            if self.enabled.contains(parent) {
                return true;
            }
            path = parent;
        }
        false
    }
}
```

### Audit Trail

```rust
/// Emitted when a capability check occurs for a write operation.
#[derive(Debug, Serialize)]
pub struct CapabilityCheckEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub client_id: String,
    pub capability: String,
    pub result: CapabilityCheckResult,
    pub tool_name: String,
    pub trace_id: String,
}

#[derive(Debug, Serialize)]
pub enum CapabilityCheckResult {
    Allowed,
    Denied { reason: String },
    DryRun,
}
```

Use `tracing` with structured fields:

```rust
tracing::info!(
    client_id = %client_id,
    capability = %cap,
    result = "denied",
    tool = "crowdstrike_contain_host",
    "capability check"
);
```

---

## 6. Existing MCP Servers with Feature Flags

### Direct Evidence: Limited

No widely-known MCP server implementations (as of training data cutoff May 2025) explicitly use a "feature flag" system for tool gating. However, several related patterns exist:

#### The mcp-claroty-xdome Reference (In This Repo)

The Python MCP server in `.references/mcp-claroty-xdome` does NOT implement feature flags, but it does have:
- A `config://server` resource that exposes a `features` list -- this is informational, not a gate
- Tool providers that register tools unconditionally

This confirms the gap: the existing reference implementation has no per-client capability gating.

#### The Axiathon Plugin System (In This Repo)

The axiathon spike has a plugin manifest with `permissions: Vec<String>` (e.g., `["network:listen", "file:read"]`). This is the closest pattern to what Prism needs. The permission model is:
- Declared on the plugin manifest
- Checked by the host before loading
- Namespaced with colon separators (`network:listen`)

Prism's capability model can directly adopt this pattern, changing the separator from `:` to `.` for TOML compatibility and adding hierarchical resolution.

#### Claude Desktop MCP Config

Claude Desktop's `claude_desktop_config.json` configures which MCP servers are available, but does not gate individual tools within a server. Tool-level gating is the server's responsibility.

### Indirect Evidence: The MCP Spec's `tools/list` + Dynamic Registration

The MCP specification supports dynamic tool lists via `notifications/tools/list_changed`. This is the intended mechanism for servers to add/remove tools at runtime. Prism should use this when switching client context.

> **CONFIDENCE: LOW for "existing MCP servers with feature flags."** WebSearch was unavailable. MCP server implementations are proliferating rapidly (post-November 2024). It is plausible that feature-flagged MCP servers exist in open source that were published after my training data cutoff.

---

## 7. Recommendation

### Build a Custom Two-Tier Capability System

**Do not add a third-party dependency for this.** The requirements are specific (TOML-driven, per-client, hierarchical, audit-logged) and the implementation is small (~200-300 lines of Rust).

### Architecture Summary

```
                    Compile-Time                    Runtime
                    ============                    =======
                    
Cargo.toml          [features]                      prism.toml
features:           crowdstrike-write               [clients.acme.capabilities]
                    claroty-write                   containment = true
                    armis-write
                         |                               |
                         v                               v
                    #[cfg(feature)]                  ClientCapabilities
                    gates in code                   .is_enabled("sensor.crowdstrike.containment")
                         |                               |
                         +----------- BOTH MUST PASS ----+
                                          |
                                          v
                                   Tool registered in
                                   MCP tools/list
```

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Compile-time gating? | Yes, cargo features for write code families | Defense in depth. Operator can distribute read-only binaries. |
| Runtime gating? | Yes, TOML per-client capabilities | Multi-client. Different clients have different risk appetites. |
| Disabled tool visibility? | Hidden from `tools/list`, discoverable via meta-tool | Clean agent experience. No wasted tool calls. |
| Dangerous operation gate? | Dry-run default for reversible; confirmation token for irreversible | Matches security tooling norms. Natural conversation flow. |
| Third-party crate? | No. Custom ~250-line implementation. | No existing crate fits. Simple enough to own. |
| Audit trail? | `tracing` structured events for all write capability checks | Standard Rust observability. Integrates with existing tracing infra. |
| Default posture? | Deny all writes unless explicitly enabled | MSSP security requirement. Principle of least privilege. |
| Config format? | TOML with `[defaults]` + `[clients.{id}]` override | Consistent with Rust ecosystem norms. Human-readable. |

### Files to Create

| File | Purpose | Approximate Size |
|------|---------|-----------------|
| `crates/prism-core/src/capabilities.rs` | `ClientCapabilities`, `CapabilityCheck`, hierarchical resolution | ~150 lines |
| `crates/prism-core/src/capabilities/audit.rs` | Audit event types, tracing integration | ~50 lines |
| `crates/prism-config/src/client.rs` | TOML deserialization for `[clients.{id}.capabilities]` | ~100 lines |
| Tests | Unit tests for hierarchical resolution, deny-by-default, audit emission | ~200 lines |

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebSearch | 1 (denied) | Attempted: Rust runtime feature flag crates |
| WebFetch | 2 (denied) | Attempted: crates.io API for flagset, feature flag search |
| Context7 | 1 (denied) | Attempted: flagset crate documentation |
| Grep/Read (local) | 8 | Examined axiathon plugin manifest, mcp-claroty-xdome patterns, project context |
| Training data | 4 areas | Rust crate ecosystem (flagset, unleash, growthbook, bitflags); MCP protocol tool listing behavior; MSSP config patterns; security confirmation patterns |

**Total MCP tool calls:** 4 attempted, 4 denied
**Training data reliance:** HIGH -- All external research tools were denied. Crate version numbers, last-update dates, and download counts are NOT verified against live registries. The core architectural recommendations (custom implementation, two-tier gating, hierarchical resolution) are sound patterns independent of specific crate versions, but the crate landscape assessment MUST be verified before finalizing.

### Verification Actions Required

1. Run `cargo search feature-flag` and `cargo search feature-toggle` to check for crates published after May 2025
2. Check crates.io for `openfeature` Rust SDK maturity (was early-stage as of mid-2025)
3. Search GitHub for "mcp server feature flag" to find any open-source implementations
4. Verify rmcp 0.8 supports `notifications/tools/list_changed` for dynamic tool list updates
