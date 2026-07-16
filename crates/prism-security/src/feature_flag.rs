// S-1.08: Feature Flag Evaluator
//
// Story:  S-1.08 — prism-security: Feature Flags (P0 Core)
// BCs:    BC-2.04.001, BC-2.04.002, BC-2.04.003, BC-2.04.004
// VP:     VP-020 (Kani proof: compile-time disabled → always Deny)
//
// Architecture compliance rules:
//   - `check_permission` MUST default to Deny when no capability config is present (AD-019).
//   - Compile-time gate (Cargo feature absent) CANNOT be overridden by runtime TOML (BC-2.04.001).
//   - `BTreeMap` MUST be used for capability storage — NOT HashMap (BC-2.04.003).
//   - Both tiers must independently return Allow for the combined result to be Allow (BC-2.04.004).

use std::{collections::BTreeMap, sync::Arc};

use prism_core::{
    capability::{CapabilityPath, ClientCapabilities},
    error::PrismError,
    OrgRegistry, OrgSlug,
};

// ─────────────────────────────────────────────────────────────
// Tier-1: Compile-time feature gate model
// ─────────────────────────────────────────────────────────────

/// Represents the compile-time feature gate status for a write code family.
///
/// In production, this is determined by `#[cfg(feature = "...")]` gating.
/// Tests model it as a runtime bool per VP-020 feasibility assessment:
/// "Compile-time gate modeled as runtime bool in test; separate build-matrix
/// test covers the real cfg gate."
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileTimeGate {
    /// The Cargo feature is present in this binary.
    Present,
    /// The Cargo feature is absent — write code does not exist in this binary.
    Absent,
}

// ─────────────────────────────────────────────────────────────
// CapabilityCheckResult
// ─────────────────────────────────────────────────────────────

/// The outcome of a two-tier capability check, including the denial tier and
/// resolution trace required by the structured capability-denied errors
/// (E-FLAG-001 runtime tier / E-FLAG-002 compile tier, BC-2.04.015).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityCheckResult {
    /// Both tiers passed — the operation is permitted.
    Allowed,
    /// Denied by the compile-time tier (no write-endpoint declaration;
    /// registry-derived per BC-2.04.001 / BC-2.16.012).
    DeniedCompileTime {
        capability: String,
        client_id: String,
        /// Ordered resolution trace for the E-FLAG-002 structured error.
        resolution_trace: Vec<String>,
    },
    /// Denied by the runtime tier (capability not in client config).
    DeniedRuntime {
        capability: String,
        client_id: String,
        /// Ordered resolution trace for E-FLAG-001 structured error.
        resolution_trace: Vec<String>,
    },
}

// ─────────────────────────────────────────────────────────────
// FeatureFlagEvaluator
// ─────────────────────────────────────────────────────────────

/// Two-tier feature flag evaluator for write operations (BC-2.04.004).
///
/// Tier 1: compile-time Cargo feature gate (BC-2.04.001).
/// Tier 2: runtime per-client TOML capability configuration (BC-2.04.002).
///
/// Both tiers must independently return Allow for the combined result to be
/// Allowed. The compile-time gate is modeled here as a `CompileTimeGate`
/// enum passed at construction time; in production binaries the calling code
/// is absent if the feature is not compiled in.
///
/// Client capabilities are stored as `BTreeMap<String, ClientCapabilities>`
/// for deterministic iteration order required by the resolution trace
/// (BC-2.04.003 architecture compliance rule).
pub struct FeatureFlagEvaluator {
    /// Per-client capability maps keyed by client ID.
    /// `BTreeMap` required — NOT `HashMap` — for deterministic trace order.
    client_capabilities: BTreeMap<String, ClientCapabilities>,
    /// Organisation registry for multi-tenant client existence checks (BC-2.10.015).
    ///
    /// `client_exists` uses `org_registry.slug_exists(&OrgSlug)` — the ONLY
    /// authoritative path for org membership. `OrgSlug::new_unchecked` MUST NOT
    /// be used; call `OrgSlug::new(client_id)` (fallible, non-panicking) instead.
    ///
    /// BC-2.10.015: consulted by `client_exists` via `slug_exists(&OrgSlug)`.
    org_registry: Arc<OrgRegistry>,
}

impl FeatureFlagEvaluator {
    /// Construct a `FeatureFlagEvaluator` with pre-resolved per-client
    /// capability maps and an org registry for client existence checks.
    ///
    /// `client_capabilities` MUST be a `BTreeMap` — see architecture
    /// compliance rule in story spec.
    ///
    /// `org_registry` MUST be the live `Arc<OrgRegistry>` wired at boot via
    /// Arc-DI (ADR-022). The placeholder-construct anti-pattern is forbidden here
    /// (Standing Rule 3 §4; ADR-022 §C).
    pub fn new(
        client_capabilities: BTreeMap<String, ClientCapabilities>,
        org_registry: Arc<OrgRegistry>,
    ) -> Self {
        FeatureFlagEvaluator {
            client_capabilities,
            org_registry,
        }
    }

    /// Perform a two-tier capability check.
    ///
    /// # Parameters
    /// - `compile_gate`: whether the write code family is compiled in (Tier 1).
    /// - `client_id`: the client whose runtime capabilities are consulted (Tier 2).
    /// - `capability`: the dot-separated path to check (e.g., `"sensor.crowdstrike.containment"`).
    ///
    /// # Returns
    /// - `CapabilityCheckResult::Allowed` — both tiers pass.
    /// - `CapabilityCheckResult::DeniedCompileTime` — compile gate absent.
    /// - `CapabilityCheckResult::DeniedRuntime` — runtime capability missing or denied.
    ///
    /// # Invariant (VP-020)
    /// When `compile_gate == CompileTimeGate::Absent`, the result is ALWAYS
    /// `DeniedCompileTime` regardless of runtime capability configuration.
    pub fn check_permission(
        &self,
        compile_gate: CompileTimeGate,
        client_id: &str,
        capability: &str,
    ) -> CapabilityCheckResult {
        // Tier 1: compile-time gate. If absent, short-circuit immediately.
        // Runtime config cannot override a missing compile-time feature (BC-2.04.001).
        if compile_gate == CompileTimeGate::Absent {
            return CapabilityCheckResult::DeniedCompileTime {
                capability: capability.to_string(),
                client_id: client_id.to_string(),
                // P1-02 (2026-06-10 review pass-1): registry semantics — the
                // compile-time tier is Absent when no [[write_endpoints]] declaration
                // for the capability is loaded into the registry (BC-2.16.012).
                resolution_trace: vec![format!(
                    "compile-time=Absent: no [[write_endpoints]] declaration for '{}' \
                     (registry-driven dispatch, BC-2.16.012)",
                    capability
                )],
            };
        }

        // Tier 2: runtime per-client capability check.
        // Parse the capability path; invalid paths are deny-by-default.
        let path = match CapabilityPath::new(capability) {
            Ok(p) => p,
            Err(_) => {
                return CapabilityCheckResult::DeniedRuntime {
                    capability: capability.to_string(),
                    client_id: client_id.to_string(),
                    resolution_trace: vec![format!(
                        "runtime=Deny: invalid capability path '{}'",
                        capability
                    )],
                };
            }
        };

        // Look up client capabilities; unknown client → deny-by-default.
        let caps = match self.client_capabilities.get(client_id) {
            Some(c) => c,
            None => {
                return CapabilityCheckResult::DeniedRuntime {
                    capability: capability.to_string(),
                    client_id: client_id.to_string(),
                    resolution_trace: vec![format!(
                        "runtime=Deny: client '{}' not in configuration",
                        client_id
                    )],
                };
            }
        };

        let (allowed, explanation) = caps.is_allowed(&path);

        let trace_entry = match explanation.matched_path {
            Some(ref matched) => format!(
                "runtime={}: matched '{}' ({})",
                if allowed { "Allow" } else { "Deny" },
                matched.as_str(),
                explanation.reason
            ),
            None => format!("runtime=Deny: {}", explanation.reason),
        };

        if allowed {
            CapabilityCheckResult::Allowed
        } else {
            CapabilityCheckResult::DeniedRuntime {
                capability: capability.to_string(),
                client_id: client_id.to_string(),
                resolution_trace: vec![trace_entry],
            }
        }
    }

    /// Convert a `CapabilityCheckResult::Denied*` into a structured
    /// `PrismError::CapabilityDenied` (BC-2.04.015) — E-FLAG-001 for the
    /// runtime tier (`DeniedRuntime`) or E-FLAG-002 for the compile tier
    /// (`DeniedCompileTime`).
    ///
    /// Returns `None` if the result is `Allowed`.
    pub fn to_error(&self, result: &CapabilityCheckResult) -> Option<PrismError> {
        match result {
            CapabilityCheckResult::Allowed => None,

            CapabilityCheckResult::DeniedCompileTime {
                capability,
                client_id,
                resolution_trace,
            } => Some(PrismError::CapabilityDenied {
                capability: capability.clone(),
                client_id: client_id.clone(),
                // P2-02 (2026-06-10 review pass-2): the spec-pinned E-FLAG-002
                // message template, VERBATIM — three spec layers agree on it
                // (error-taxonomy.md E-FLAG-002 row, BC-2.04.015,
                // BC-2.04.001; spec wins per POL-24). Registry semantics,
                // not Cargo features — under registry-driven dispatch nothing is
                // "un-compiled"; the compile-time tier is Absent because the
                // sensor's TOML spec declares no [[write_endpoints]] for this
                // capability (BC-2.16.012).
                reason: format!(
                    "Write capability '{}' denied: no write-endpoint declaration \
                     (no [[write_endpoints]] entry in the sensor's TOML spec)",
                    capability
                ),
                // SNS-02 (2026-06-10 review): post-BC-2.16.012 the write pipeline is
                // registry-driven — the compile-time tier is derived from whether the
                // sensor's TOML spec declares [[write_endpoints]] sections that are
                // loaded into the WriteEndpointRegistry at boot. The {sensor}-write
                // Cargo features are empty test-gating declarations (see
                // prism-query/src/write_pipeline.rs registry-lookup site), so a
                // "rebuild with --features" suggestion is unactionable.
                suggestion: format!(
                    "Declare a [[write_endpoints]] section for '{}' in the sensor's \
                     TOML spec and ensure the spec is loaded at boot so the \
                     write-endpoint registry contains this capability \
                     (registry-driven write dispatch, BC-2.16.012).",
                    capability
                ),
                resolution_trace: resolution_trace.clone(),
            }),

            CapabilityCheckResult::DeniedRuntime {
                capability,
                client_id,
                resolution_trace,
            } => Some(PrismError::CapabilityDenied {
                capability: capability.clone(),
                client_id: client_id.clone(),
                reason: format!(
                    "Not enabled in client config: capability '{}' is not in the \
                     runtime configuration for client '{}'",
                    capability, client_id
                ),
                suggestion: format!(
                    "Add '{}' = 'Allow' under [clients.{}.capabilities] in your \
                     prism configuration file, then restart the prism server.",
                    capability, client_id
                ),
                resolution_trace: resolution_trace.clone(),
            }),
        }
    }

    /// Return true if `client_id` is a registered org in `OrgRegistry` (BC-2.10.015).
    ///
    /// Uses `OrgRegistry::slug_exists(&OrgSlug)` as the single authoritative gate.
    /// Parses `client_id` via `OrgSlug::new(client_id)` (fallible); returns `false`
    /// for any string that fails `OrgSlug` validation — no panic, no `new_unchecked`.
    ///
    /// # Invariant
    /// MUST NOT use `OrgSlug::new_unchecked` — that is a validation-bypass constructor
    /// forbidden in production code paths (CLAUDE.md §Conventions).
    pub fn client_exists(&self, client_id: &str) -> bool {
        // OrgSlug::new validates format (is_ok() / is_err() carries validity state).
        // Invalid client_ids (too long, bad chars) return is_err() → false.
        // No new_unchecked, no panic (AD-017 / CLAUDE.md §Conventions).
        let slug = OrgSlug::new(client_id);
        if slug.is_err() {
            return false;
        }
        self.org_registry.slug_exists(&slug)
    }

    /// Return all capability paths configured for a specific client.
    ///
    /// Used by `list_capabilities` to enumerate client-configured capability paths
    /// so that capabilities configured in TOML but absent from the `WriteEndpointRegistry`
    /// can be reported as `compile_time_disabled`.
    ///
    /// Returns an empty `Vec` if the client is not found in the configuration.
    pub fn capability_paths_for_client(&self, client_id: &str) -> Vec<String> {
        self.client_capabilities
            .get(client_id)
            .map(|caps| {
                caps.capabilities_for_display()
                    .into_iter()
                    .map(|(path, _effect)| path.as_str().to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Return all configured client IDs.
    ///
    /// Used by `list_capabilities` cross-client summary mode to enumerate clients.
    pub fn client_ids(&self) -> Vec<&str> {
        self.client_capabilities
            .keys()
            .map(|s| s.as_str())
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────
// Compile-time write feature gate wrappers
// ─────────────────────────────────────────────────────────────
//
// These functions return the compile-time gate status for each write code
// family. In production code they use `#[cfg(feature = "...")]` to determine
// the value. Tests can call them to verify the real binary gate, but the
// `check_permission` tests use the `CompileTimeGate` enum directly to model
// the 2×2 truth table per VP-020.

/// Returns `CompileTimeGate::Present` if `crowdstrike-write` is compiled in,
/// `CompileTimeGate::Absent` otherwise (BC-2.04.001).
pub fn crowdstrike_write_gate() -> CompileTimeGate {
    #[cfg(feature = "crowdstrike-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "crowdstrike-write"))]
    {
        CompileTimeGate::Absent
    }
}

/// Returns `CompileTimeGate::Present` if `cyberint-write` is compiled in.
pub fn cyberint_write_gate() -> CompileTimeGate {
    #[cfg(feature = "cyberint-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "cyberint-write"))]
    {
        CompileTimeGate::Absent
    }
}

/// Returns `CompileTimeGate::Present` if `claroty-write` is compiled in.
pub fn claroty_write_gate() -> CompileTimeGate {
    #[cfg(feature = "claroty-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "claroty-write"))]
    {
        CompileTimeGate::Absent
    }
}

/// Returns `CompileTimeGate::Present` if `armis-write` is compiled in.
pub fn armis_write_gate() -> CompileTimeGate {
    #[cfg(feature = "armis-write")]
    {
        CompileTimeGate::Present
    }
    #[cfg(not(feature = "armis-write"))]
    {
        CompileTimeGate::Absent
    }
}
