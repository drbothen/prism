# Capability/Permission Deny Patterns for MSSP Security Tooling

**Date:** 2026-04-13
**Type:** General (technology research)
**Status:** Complete
**Purpose:** Evaluate allow+deny permission patterns for hierarchical capability resolution in Prism's per-client feature flag system. Extends the existing feature-flag-research.md which identified the need but did not address deny semantics.

---

## Table of Contents

1. [Platform Comparison: How Major Systems Handle Allow + Deny](#1-platform-comparison)
2. [MSSP/SOAR Platform Capability Gating](#2-msspsoar-platform-capability-gating)
3. [Allow + Deny Set vs Leaf-Only vs Ordered Rules](#3-pattern-comparison)
4. [Rust Crate Ecosystem for Authorization](#4-rust-crate-ecosystem)
5. [MSSP-Specific Deny Patterns](#5-mssp-specific-deny-patterns)
6. [MCP Tool Gating Patterns](#6-mcp-tool-gating-patterns)
7. [Recommendation for Prism](#7-recommendation-for-prism)

---

## 1. Platform Comparison: How Major Systems Handle Allow + Deny

### AWS IAM (Verified via docs.aws.amazon.com)

**Model:** Explicit Deny > Explicit Allow > Implicit Deny

AWS IAM is the gold standard for allow+deny policy systems. The evaluation order:

1. **Explicit Deny** -- If any policy (identity-based, resource-based, SCP, RCP, permissions boundary) contains an explicit `"Effect": "Deny"` that matches the request, the request is **denied**. Full stop. No allow can override this.
2. **Explicit Allow** -- If no explicit deny exists and any applicable policy contains `"Effect": "Allow"`, the request is **allowed**.
3. **Implicit Deny** -- If no policy matches at all, the request is **denied** (default deny).

**Key design properties:**
- **Identity + Resource policies:** UNION (either can allow)
- **Identity + Permissions Boundary:** INTERSECTION (both must allow)
- **Identity + SCPs/RCPs:** INTERSECTION (all must allow)
- Explicit deny in ANY layer overrides allow from ANY other layer

**Relevance to Prism:** The "explicit deny wins" model is powerful but introduces complexity. AWS mitigates this with IAM Access Analyzer and policy simulation tools. Prism would need similar tooling to debug capability resolution if deny is added.

### Azure RBAC

**Model:** Allow-only with deny assignments (added later)

Azure RBAC was originally purely additive -- role assignments grant permissions, and permissions are unioned across all role assignments. Deny assignments were added later (2019) as a separate mechanism:

- Deny assignments are evaluated BEFORE role assignments
- Deny assignments override matching role assignments
- Deny assignments cannot be directly created by users -- they are created by Azure Blueprints and managed applications

**Key insight:** Azure tried to keep it simple with allow-only, then had to bolt on deny for enterprise scenarios (Blueprints needed "even the subscription owner cannot do X"). The bolt-on nature means deny is a second-class citizen in the UX.

**Relevance to Prism:** Azure's experience validates that starting with allow-only is viable, but deny may be needed later for "allow all sensors except..." patterns.

### Kubernetes RBAC

**Model:** Purely additive (no deny mechanism)

Kubernetes RBAC is the simplest model:
- `Role` / `ClusterRole` define permissions (verbs on resources)
- `RoleBinding` / `ClusterRoleBinding` assign roles to subjects
- Permissions are UNIONED across all bindings
- There is **no deny**. The only way to remove a permission is to remove the role binding.

**Footgun:** If a ClusterRoleBinding grants `get pods` cluster-wide, you cannot exclude a specific namespace. This has been a long-standing pain point (Kubernetes issue #56689 proposed deny rules but was not implemented as of early 2025).

**Relevance to Prism:** The K8s model is the simplest to reason about but cannot express "allow all CrowdStrike operations EXCEPT containment for client X." Prism's MSSP use case likely needs this.

### HashiCorp Vault Policies

**Model:** Allow + Deny with path globbing, most-specific-path wins

Vault policies use path-based matching with capabilities:

```hcl
# Allow reading secrets under secret/data/
path "secret/data/*" {
  capabilities = ["read", "list"]
}

# But deny a specific subpath
path "secret/data/production/*" {
  capabilities = ["deny"]
}
```

Resolution rules:
1. Policies are path-matched against the request path
2. Most specific path match wins (longest prefix)
3. If the most specific match has `deny` capability, the request is denied
4. If multiple policies apply at the same specificity, the UNION of capabilities is taken
5. `deny` at a more-specific path overrides `allow` at a less-specific path
6. The root policy is special and grants all access
7. Glob patterns: `*` matches within a path segment, `+` matches a single segment (Vault 1.10+)

**Key insight:** Vault's model is the closest to what Prism needs -- hierarchical paths with deny override at specific nodes. The "most-specific-path wins" rule is intuitive for operators.

**Relevance to Prism:** This is the strongest analog. Capability paths like `sensor.crowdstrike.containment` map directly to Vault-style path policies.

### Linux Capabilities

**Model:** Bitmask sets (permitted, effective, inheritable, bounding, ambient)

Linux capabilities use three bitmask sets per process:
- **Permitted (P):** Upper bound of capabilities the process CAN have
- **Effective (E):** Capabilities currently active (subset of Permitted)
- **Inheritable (I):** Capabilities preserved across exec()

The bounding set acts as a deny mechanism:
- If a capability is NOT in the bounding set, it CANNOT be in the permitted set (even if inherited)
- `prctl(PR_CAPBSET_DROP, cap)` removes from the bounding set -- this is irreversible

**Key insight:** The bounding set is a hard deny ceiling. It is the "even root cannot re-grant this" mechanism. Prism's compile-time cargo features serve a similar role -- if `crowdstrike-write` is not compiled in, no runtime config can enable it.

**Relevance to Prism:** The two-tier model (bounding set = compile-time features, effective set = runtime flags) is exactly what the existing feature-flag-research recommends. Adding a deny layer would be analogous to adding per-process capability dropping.

---

## 2. MSSP/SOAR Platform Capability Gating

### Palo Alto XSOAR (Cortex XSOAR)

**Model:** Role-based with per-integration instance permissions

- **Roles** define broad permissions (Analyst, Admin, Read-Only)
- **Integration instances** are configured per client/tenant
- Permission gating happens at the integration instance level: an analyst with "Analyst" role can only see/use integration instances assigned to their role
- There is no "allow CrowdStrike but deny containment" granularity -- the entire integration instance is either accessible or not
- For MSSP multi-tenancy, XSOAR uses separate tenants (XSOAR multi-tenant) with tenant-level role assignments
- Playbook-level permissions gate which playbooks analysts can execute

**Key insight:** XSOAR gates at the integration-instance level, not at the operation level. "Client A's CrowdStrike" is a different integration instance from "Client B's CrowdStrike." Analysts are assigned to tenant contexts.

### Splunk SOAR (Phantom)

**Model:** Role-based with asset-level permissions

- **Assets** map to specific sensor API connections (analogous to Prism's per-client sensor configs)
- **Roles** gate which assets and actions an analyst can use
- **App actions** are categorized (investigate, contain, correct, generic)
- Permissions can be set per action category per role: e.g., "Analyst role can run investigate actions but not contain actions"
- This is the closest to operation-level deny among SOAR platforms

**Key insight:** Splunk SOAR's action-category model is interesting -- it gates by operation type (investigate vs. contain) rather than by specific API call. This aligns with Prism's risk classification (read, reversible write, irreversible write).

### Swimlane

**Model:** Role-based with record-level and workspace-level access

- Swimlane uses workspaces for multi-tenancy (one workspace per client)
- Permissions are role-based within workspaces
- Integration permissions are workspace-scoped -- you cannot selectively deny specific integration actions within a workspace
- The model is simpler than XSOAR/Splunk SOAR

### Summary: MSSP Platform Patterns

| Platform | Granularity | Deny Support | Multi-Tenant Model |
|----------|-------------|-------------|-------------------|
| XSOAR | Integration instance | No (all-or-nothing per instance) | Separate tenants |
| Splunk SOAR | Action category per role | Partial (exclude action categories) | Asset-level isolation |
| Swimlane | Workspace-level | No | Workspace per client |
| **Prism (proposed)** | **Operation-level per client** | **Yes (this research)** | **Per-client config** |

Prism's proposed operation-level granularity exceeds what existing SOAR platforms offer. This is a differentiator but also means there is no strong precedent to copy.

---

## 3. Pattern Comparison: Allow+Deny Set vs Leaf-Only vs Ordered Rules

### Pattern A: Allow + Deny Sets (AWS IAM, Vault)

**How it works:** Two separate collections -- an allow set and a deny set. Deny always wins over allow.

```toml
[clients.acme.capabilities]
allow = ["sensor.crowdstrike.*", "sensor.claroty.read"]
deny = ["sensor.crowdstrike.containment"]
```

Resolution: `is_allowed(cap) = (cap matches allow) AND NOT (cap matches deny)`

**Pros:**
- Expressive: can say "allow all CrowdStrike EXCEPT containment"
- Well-understood (AWS IAM has trained a generation of engineers)
- Audit-friendly: "why was this denied?" has a clear answer (matched deny rule X)

**Cons:**
- **Order-independence confusion:** New operators may not understand that deny ALWAYS wins (AWS IAM's #1 support issue)
- **Hidden denials:** A deny rule added at a broad level can silently break capabilities that were previously working
- **Testing complexity:** Must test both "does allow work?" and "does deny override work?" for every path
- **Footgun -- overly broad deny:** `deny = ["sensor.*"]` accidentally denies reads too

**Error rate in practice:** MEDIUM. AWS IAM policy errors are extremely common. AWS invested heavily in IAM Access Analyzer, policy simulator, and access advisor to mitigate this. For a system with 10-50 clients (Prism's scale), the risk is lower than AWS's millions-of-users scale.

### Pattern B: Leaf-Only Enumeration (Kubernetes RBAC)

**How it works:** No hierarchy. Each allowed capability is explicitly listed. No deny -- absence means denied.

```toml
[clients.acme.capabilities]
enabled = [
  "sensor.crowdstrike.read",
  "sensor.crowdstrike.alert_acknowledge",
  "sensor.claroty.read",
]
# Everything not listed is denied
```

Resolution: `is_allowed(cap) = cap IN enabled_set`

**Pros:**
- **Simplest to reason about:** If it is not in the list, it is denied. Period.
- **Easiest to audit:** The config IS the permission set. No resolution logic needed.
- **No footguns:** Cannot accidentally deny something you intended to allow
- **Grep-friendly:** `grep containment clients.toml` tells you exactly which clients have it

**Cons:**
- **Verbose for broad grants:** Enabling all CrowdStrike operations requires listing each one
- **No "allow all except X" pattern:** Must enumerate every allowed operation
- **Config drift:** Adding a new tool requires updating every client config that should have it

**Error rate in practice:** LOW for existing capabilities, but HIGH for new capability rollout (forgetting to add new capabilities to client configs).

### Pattern C: Ordered Rules / Priority Rules (Firewall-style)

**How it works:** Rules are evaluated in order. First match wins.

```toml
[[clients.acme.rules]]
pattern = "sensor.crowdstrike.containment"
effect = "deny"

[[clients.acme.rules]]
pattern = "sensor.crowdstrike.*"
effect = "allow"

[[clients.acme.rules]]
pattern = "*"
effect = "deny"
```

Resolution: Walk rules top-to-bottom, first match wins.

**Pros:**
- Maximum expressiveness
- Can express any combination of allow/deny
- Familiar to network/security engineers (firewall ACLs)

**Cons:**
- **Order-dependent:** Swapping two lines changes behavior. This is the #1 source of firewall misconfigurations.
- **Extremely difficult to audit:** "Why was X denied?" requires tracing through the rule list
- **Merge complexity:** How do you merge default rules with client-specific rules?
- **Not TOML-friendly:** Ordered arrays of tables are fragile in config files

**Error rate in practice:** HIGH. Firewall rule ordering errors are pervasive in the industry. Cisco, Palo Alto, and Fortinet all invest heavily in rule analysis/optimization tools because humans consistently get ordering wrong.

### Pattern D: Hierarchical with Explicit Override (Vault-style) -- RECOMMENDED

**How it works:** Allow/deny at any node in the hierarchy. More-specific node wins. Ties go to deny.

```toml
[clients.acme.capabilities]
"sensor.crowdstrike" = "allow"           # Allow all CrowdStrike operations
"sensor.crowdstrike.containment" = "deny" # But deny containment specifically
"sensor.claroty" = "allow"
# Everything not mentioned: denied (implicit deny)
```

Resolution:
1. Find the most-specific path that matches
2. If it says "allow", allow. If it says "deny", deny.
3. If no path matches, implicit deny.

**Pros:**
- **Intuitive:** More specific overrides less specific (natural hierarchy)
- **Expressive:** Can do "allow all except X" without listing every operation
- **Auditable:** "Why denied?" = "Most specific matching rule at path X says deny"
- **Order-independent:** The hierarchy determines precedence, not config file ordering
- **TOML-friendly:** Each entry is a key-value pair

**Cons:**
- **Parent-child ambiguity:** What does `sensor.crowdstrike = allow` mean for `sensor.crowdstrike.containment` if containment is not mentioned? (Answer: allowed, by inheritance)
- **Implicit inheritance can surprise:** Adding a new child capability is automatically allowed if the parent is allowed
- **Slightly more complex resolution** than leaf-only (but still ~30 lines of code)

**Error rate in practice:** LOW-MEDIUM. The "most specific wins" rule is intuitive. The main risk is implicit inheritance -- a new tool added under an allowed parent is automatically allowed, which in a security context means new capabilities must be carefully placed in the hierarchy.

### Comparison Matrix

| Property | Allow+Deny Sets | Leaf-Only | Ordered Rules | Hierarchical Override |
|----------|----------------|-----------|---------------|----------------------|
| **Expressiveness** | High | Low | Highest | High |
| **Operator error rate** | Medium | Low | High | Low-Medium |
| **Audit clarity** | Medium | Highest | Low | High |
| **"Allow all except X"** | Yes | No | Yes | Yes |
| **New capability safety** | Deny unless added to allow | Deny unless added | Depends on rule order | Inherits from parent -- CAUTION |
| **Config verbosity** | Medium | High | Medium | Low |
| **Merge complexity** | Low (union allows, union denies) | Low (union sets) | High (order matters) | Low (overlay specific paths) |
| **Compliance friendliness** | Good (can dump allow/deny) | Best (config = truth) | Poor (order-dependent) | Good (tree is inspectable) |

---

## 4. Rust Crate Ecosystem for Authorization with Deny Support

> **CONFIDENCE: MEDIUM.** Crate versions and download counts are from training data (cutoff May 2025). MUST be verified against crates.io before final decision. WebFetch and Context7 were denied for this session.

### Cedar Policy (Amazon)

- **Crate:** `cedar-policy` (latest known: 4.x as of early 2025)
- **Maintained by:** Amazon (same team as AWS Verified Permissions)
- **Model:** Allow + Forbid policies. Forbid always overrides permit. This is the AWS IAM model expressed as a standalone policy language.
- **Language:** Cedar has its own policy language (not TOML):
  ```cedar
  permit(
    principal,
    action == Action::"ContainHost",
    resource in Client::"acme-corp"
  );
  
  forbid(
    principal,
    action == Action::"ContainHost",
    resource in Client::"globex"
  );
  ```
- **Rust-native:** Written in Rust. First-class Rust API.
- **Hierarchical:** Supports entity hierarchies (entities can be members of groups)
- **Deny support:** YES -- `forbid` policies always override `permit` policies

**Assessment for Prism:**
- PROS: Battle-tested (backs AWS Verified Permissions), Rust-native, formally verified, supports hierarchies
- CONS: Requires learning Cedar policy language, overkill for ~50 clients with ~20 capabilities, introduces a policy language into TOML-based config, adds ~significant dependency footprint
- **Verdict: OVERKILL for Prism's scale.** Cedar is designed for millions of entities and thousands of policy rules. Prism has dozens of clients and a fixed set of ~20 capabilities. The cognitive overhead of Cedar policy language is not justified.

### Casbin (casbin-rs)

- **Crate:** `casbin` (latest known: 2.x)
- **Model:** Configurable -- supports ACL, RBAC, ABAC via model configuration files
- **Deny support:** Yes, via policy effect configuration. Can configure "allow-override" (any allow wins), "deny-override" (any deny wins), or priority-based.
- **Hierarchical:** Supports role hierarchies (RBAC)
- **Language:** Uses its own model/policy file format (PERM meta-model)

**Assessment for Prism:**
- PROS: Flexible, well-known in the authorization space, multiple language SDKs
- CONS: Generic framework -- requires significant configuration. The PERM model adds conceptual overhead. The Rust crate's maintenance cadence has been variable. Async support was incomplete as of early 2025.
- **Verdict: POSSIBLE but adds unnecessary abstraction.** Casbin's strength is its model flexibility, but Prism's model is fixed and simple.

### Oso (oso-rs)

- **Crate:** `oso` (company shut down operations in late 2023, open-sourced the engine)
- **Model:** Polar policy language with allow/deny
- **Status:** Open-source but no longer commercially supported. Community maintenance.
- **Deny support:** Yes, via Polar language

**Assessment for Prism:**
- **Verdict: NOT RECOMMENDED.** No commercial support, uncertain maintenance future.

### Custom Implementation (~200 lines)

The existing feature-flag-research.md recommends a custom `HashSet<String>` implementation (~150 lines). Extending this to support deny requires:

- Changing from `HashSet<String>` to `HashMap<String, Effect>` where `Effect` is `Allow | Deny`
- Adding resolution logic: walk from most-specific to least-specific, first match wins
- Adding implicit deny as the final fallback

This is approximately **200-250 lines of Rust** including tests. No external dependency needed.

### Recommendation

**Custom implementation with the hierarchical override pattern (Pattern D).** The authorization crate ecosystem offers powerful solutions (Cedar, Casbin) that are designed for much larger scale than Prism needs. The custom implementation:
- Keeps config in TOML (no second policy language)
- Is auditable in a single file
- Has zero external dependencies for the authorization logic
- Can be formally tested with property-based tests (all possible capability paths)

---

## 5. MSSP-Specific Deny Patterns

### Do MSSPs Need "Allow All Except X"?

**YES.** Based on MSSP workflow analysis (mssp-workflow-research.md) and domain knowledge:

**Common scenarios requiring deny:**

1. **New client onboarding -- graduated enablement:**
   - Start with read-only across all sensors
   - Enable alert acknowledgement after analyst training
   - Enable containment after SOC manager approval
   - This is pure allow escalation -- no deny needed

2. **Client contractual restrictions:**
   - "Client allows CrowdStrike management but explicitly prohibits network containment because their OT network is air-gapped and containment could cause safety incidents"
   - This IS an "allow all except X" pattern
   - Without deny: must enumerate every CrowdStrike operation except containment (fragile -- breaks when new operations are added)
   - With deny: `sensor.crowdstrike = allow` + `sensor.crowdstrike.containment = deny`

3. **Regulatory constraints:**
   - Healthcare clients may allow reading alerts but deny any write operation that could disrupt patient-facing systems
   - Financial clients may allow containment during market-closed hours only (time-based deny -- out of scope for Prism v1 but deny infrastructure enables future extension)

4. **Incident response escalation:**
   - During an active incident, temporarily allow containment for a normally read-only client
   - This requires being able to override the client's normal deny -- adds complexity
   - Prism v1 approach: require config file edit + server restart (acceptable for v1)

### Do MSSPs Typically Enumerate or Use Broad Grants?

**Both, depending on maturity:**

| MSSP Maturity | Pattern | Why |
|---------------|---------|-----|
| **Early stage** (5-15 clients) | Leaf-only enumeration | Simple, auditable, each client is hand-configured |
| **Growth stage** (15-50 clients) | Template-based with overrides | Client "profiles" (read-only, standard, full) with per-client exceptions |
| **Enterprise** (50+ clients) | Hierarchical with deny | Too many clients for per-operation enumeration. Profiles + exceptions. |

Prism is targeting the growth-to-enterprise transition (dozens of clients, 4 sensors with 15-25 operations each). The template-with-overrides pattern maps directly to the hierarchical override model.

### Recommended MSSP Config Pattern

```toml
# Client profiles (templates)
[profiles.read-only.capabilities]
"sensor" = "allow"              # Allow all sensor reads
"sensor.*.write" = "deny"       # Deny all writes (explicit, not just absence)

[profiles.standard.capabilities]
"sensor" = "allow"
"sensor.*.containment" = "deny" # Standard clients cannot contain

[profiles.full.capabilities]
"sensor" = "allow"              # Everything allowed

# Per-client config
[clients.acme-corp]
profile = "standard"

[clients.acme-corp.capabilities]
# Override: Acme specifically allows CrowdStrike containment
"sensor.crowdstrike.containment" = "allow"
# But NOT Claroty containment (inherits deny from standard profile)

[clients.globex]
profile = "read-only"
# No overrides -- strictly read-only
```

Resolution order:
1. Client-specific capability (most specific path match)
2. Profile capability (most specific path match)
3. Implicit deny

---

## 6. MCP Tool Gating Patterns

### MCP Specification Support

The MCP spec (as of early 2025) provides these mechanisms relevant to capability gating:

1. **`tools/list`** -- Server declares available tools. Can be dynamic.
2. **`notifications/tools/list_changed`** -- Server notifies client that tool list has changed. Triggers re-fetch.
3. **Tool annotations** -- `readOnlyHint`, `destructiveHint`, `idempotentHint` (added in MCP spec 2025-01). These are hints, not enforcement.
4. **No built-in authorization.** The MCP spec explicitly does not define authorization. It is the server's responsibility.

### Published MCP Permission Patterns

> **CONFIDENCE: LOW.** WebSearch and broader research tools were denied. This assessment is based on training data (May 2025). The MCP ecosystem is evolving rapidly.

**Known patterns:**

1. **Anthropic's guidance:** The MCP spec documentation suggests that servers should implement their own authorization and use `tools/list` dynamically to hide unauthorized tools. This aligns with Prism's existing "hidden tools" pattern.

2. **No known MCP server with allow+deny capability model.** Most MCP servers either expose all tools unconditionally or use simple boolean flags. The sophistication Prism needs (per-client hierarchical capabilities with deny) appears novel in the MCP ecosystem.

3. **MCP auth spec (2025):** The MCP specification added an authentication/authorization flow (OAuth 2.1 based) for HTTP transport. This is transport-level auth, not tool-level capability gating. Prism uses stdio, so this does not directly apply.

### How Prism Should Gate MCP Tools

The existing feature-flag-research.md recommendation is sound and should be extended:

```
Tool registration flow:
1. Load client config (profile + overrides)
2. Resolve capabilities (hierarchical with deny)
3. For each tool:
   a. Check compile-time gate (#[cfg(feature)])
   b. Check resolved capability for this client
   c. If both pass: register tool in tools/list
   d. If either fails: omit from tools/list
4. Provide list_capabilities meta-tool for discoverability
5. On client context switch: re-resolve, send tools/list_changed
```

---

## 7. Recommendation for Prism

### Use Hierarchical Override with Deny (Pattern D) + Profiles

**This extends the existing feature-flag-research.md recommendation.** The original research recommended `HashSet<String>` (allow-only). This research recommends upgrading to `HashMap<String, Effect>` with deny support.

### Design

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

/// Resolved capability set for a specific client.
/// Built at config load time by merging profile + client overrides.
pub struct ClientCapabilities {
    client_id: String,
    /// Map of capability paths to their effect.
    /// More-specific paths override less-specific.
    /// Paths not present: implicit deny.
    rules: BTreeMap<String, Effect>,
}

impl ClientCapabilities {
    /// Check if a capability is allowed.
    /// Walks from the exact path up to the root, returning the
    /// effect of the most-specific matching rule.
    /// If no rule matches: implicit deny.
    pub fn is_allowed(&self, capability: &str) -> bool {
        // Check exact match
        if let Some(effect) = self.rules.get(capability) {
            return *effect == Effect::Allow;
        }
        // Walk up the hierarchy
        let mut path = capability;
        while let Some((parent, _)) = path.rsplit_once('.') {
            if let Some(effect) = self.rules.get(parent) {
                return *effect == Effect::Allow;
            }
            path = parent;
        }
        // No match: implicit deny
        false
    }

    /// Return the resolution trace for audit/debugging.
    /// Shows which rule determined the outcome.
    pub fn explain(&self, capability: &str) -> CapabilityExplanation {
        // ... returns the matching rule or "implicit deny"
    }
}
```

### Why Not Allow-Only (as originally proposed)?

The original `HashSet<String>` approach from feature-flag-research.md has a critical gap:

**Scenario:** Client Acme has CrowdStrike with all operations enabled, but their contract prohibits containment of OT-network hosts. With allow-only:

```toml
# Must enumerate EVERY CrowdStrike operation except containment:
[clients.acme.capabilities]
enabled = [
  "sensor.crowdstrike.read",
  "sensor.crowdstrike.alert_acknowledge",
  "sensor.crowdstrike.alert_update",
  "sensor.crowdstrike.rtr_session",
  "sensor.crowdstrike.quarantine",
  # ... 10 more operations
  # When a new operation is added, must remember to add it here
]
```

With hierarchical deny:
```toml
[clients.acme.capabilities]
"sensor.crowdstrike" = "allow"
"sensor.crowdstrike.containment" = "deny"
```

The deny version is:
- 2 lines vs 15+
- Self-maintaining when new operations are added
- Explicitly documents the restriction (the deny is visible in config)

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Base model** | Hierarchical override (Pattern D) | Best balance of expressiveness, auditability, and error rate |
| **Deny semantics** | Most-specific path wins, implicit deny fallback | Matches Vault model. Intuitive for operators. |
| **Profiles** | Support client profiles with per-client overrides | Reduces config duplication for 50+ clients |
| **Glob patterns** | NOT in v1. Exact paths + hierarchy only. | Globs add complexity and regex-matching footguns. Hierarchy provides sufficient expressiveness. |
| **Time-based deny** | NOT in v1. Static config only. | Time-based rules require a scheduler and introduce race conditions. Defer. |
| **Conflict resolution** | Most-specific wins. If same specificity: deny wins. | "When in doubt, deny" -- security-first principle. |
| **Explanation/debug** | `explain()` method returns resolution trace | Critical for operator debugging and compliance audit. Prevents "why is this denied?" confusion. |
| **Implementation** | Custom ~250 lines Rust. No external dependency. | Cedar/Casbin are overkill. Custom code is auditable and testable. |

### Migration Path from Allow-Only

If v1 ships with allow-only (`HashSet<String>`) as originally proposed:

1. **v1.0:** `HashSet<String>` -- capabilities are simply present or absent
2. **v1.x:** Upgrade to `BTreeMap<String, Effect>` -- existing configs are auto-migrated (all entries become `Effect::Allow`)
3. **v2.0:** Add profiles

The data structure change is backward-compatible: an allow-only config is a valid hierarchical config where all entries are `Allow`.

### TOML Schema (Final Recommendation)

```toml
# prism.toml

# ── Profiles ──────────────────────────────────────────────
[profiles.read-only.capabilities]
"sensor" = "allow"
"sensor.*.write" = "deny"

[profiles.standard.capabilities]
"sensor" = "allow"
"sensor.*.containment" = "deny"

[profiles.full.capabilities]
"sensor" = "allow"

# ── Defaults ──────────────────────────────────────────────
[defaults]
profile = "read-only"  # Safest default

# ── Clients ───────────────────────────────────────────────
[clients.acme-corp]
display_name = "Acme Corporation"
profile = "standard"

[clients.acme-corp.capabilities]
# Override: allow CrowdStrike containment specifically
"sensor.crowdstrike.containment" = "allow"

[clients.globex]
display_name = "Globex Corporation"
# No profile override -- inherits "read-only" from defaults
# No capability overrides

[clients.initech]
display_name = "Initech"
profile = "full"

[clients.initech.capabilities]
# Override: deny RTR even though profile is "full"
"sensor.crowdstrike.rtr" = "deny"
```

### Resolution Algorithm (Complete)

```
resolve(client_id, capability_path):
  1. Check client-specific rules (most-specific path match)
     -> If found: return that effect
  2. Check profile rules (most-specific path match)
     -> If found: return that effect
  3. Check default profile rules (most-specific path match)
     -> If found: return that effect
  4. Return: implicit deny
```

"Most-specific path match" within a rule set:
```
For capability "sensor.crowdstrike.containment":
  Check "sensor.crowdstrike.containment"  (exact)
  Check "sensor.crowdstrike"              (parent)
  Check "sensor"                          (grandparent)
  First match wins.
```

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebFetch | 1 | AWS IAM policy evaluation logic (docs.aws.amazon.com) -- SUCCESSFUL |
| WebFetch | 4 (denied) | HashiCorp Vault policies, Cedar README, crates.io for cedar-policy and casbin |
| WebSearch | 3 (denied) | AWS IAM logic, Vault policies, MSSP SOAR capability gating |
| Context7 | 3 (denied) | cedar-policy, casbin-rs, oso crate documentation |
| Local files | 4 | product-brief.md, feature-flag-research.md, mssp-workflow-research.md, phase-0 directory scan |
| Training data | 6 areas | Azure RBAC deny assignments, Kubernetes RBAC no-deny limitation, HashiCorp Vault policy model, Linux capabilities bitmask model, MSSP SOAR platform permission models (XSOAR/Splunk SOAR/Swimlane), Cedar/Casbin/Oso crate assessments |

**Total MCP tool calls:** 11 attempted, 1 successful (AWS IAM WebFetch)
**Training data reliance:** HIGH -- Most external research tools were denied. The platform comparisons (Azure, K8s, Vault, Linux capabilities) rely on training data. The core design recommendations are architecture patterns that are well-established and unlikely to have changed, but specific crate versions and MSSP platform feature details should be verified.

### Verification Actions Required

1. **Verify cedar-policy crate version and API** on crates.io -- may have released significant updates post-May 2025
2. **Verify casbin-rs async support status** -- was incomplete in early 2025, may be resolved
3. **Search for MCP authorization patterns** -- the MCP ecosystem is growing rapidly; someone may have published capability-gating patterns by April 2026
4. **Confirm Azure RBAC deny assignments** are still limited to Blueprint/managed-app creation (Microsoft may have opened this up)
5. **Check XSOAR multi-tenant permission model** -- Palo Alto frequently updates their MSSP features
6. **Verify HashiCorp Vault `+` segment wildcard** syntax (added Vault 1.10) -- confirm still the mechanism
