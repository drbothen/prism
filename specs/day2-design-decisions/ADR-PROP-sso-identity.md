---
document_type: proposed-adr
status: capture
do_not_execute: true
provenance: "2026-06-26 side-analysis — day-2 design decision capture; PROPOSED, gated on brief-reframe sign-off; separate from live factory."
decision_source: "matured-vision-day2-requirements.md §11.3 SSO bullet; §11.5 G-12 (fine-grained RBAC); research/ui-requirements-2026-06-25.md §7 (SSO wizard, RBAC); ADR-051 (per-connection analyst identity binding)"
real_adr_number_pending: true
reserved_range: "ADR-047..054 already assigned; architect to allocate from ADR-055+ at morph time"
traces_to:
  - "matured-vision-day2-requirements.md §11.3 SSO bullet (enhancement over Query)"
  - "matured-vision-day2-requirements.md §11.4 (Query 2-role RBAC gap)"
  - "matured-vision-day2-requirements.md §11.5 G-12 (fine-grained RBAC)"
  - "matured-vision-day2-requirements.md §11.1 (server credential custody, per-tenant DEK)"
  - "matured-vision-day2-requirements.md §3.1 (central deployment, per-connection analyst identity)"
  - "research/ui-requirements-2026-06-25.md §7 (SSO wizard, per-tenant IdP, RBAC)"
  - "ADR-051 (per-connection analyst identity model — identity source for audit + authZ)"
binds_to_epics:
  - E-CENTRAL-AUTHZ-001
  - E-UI-ADMIN-001
  - E-CENTRAL-TRANSPORT-001
scim_status: OPEN_HUMAN_DECISION
---

# ADR-PROP: Enterprise SSO Identity — Per-Tenant OIDC + SAML 2.0 + Fine-Grained RBAC

## Status

**CAPTURE** — the core decision (support BOTH OIDC and SAML 2.0 from day one; per-tenant IdP config;
fine-grained RBAC beyond the 2-role Query.io model) is the architect's recommendation with supporting
rationale. SCIM provisioning is flagged for explicit human resolution (see Open Decisions).
This ADR is PROPOSED until morphed into the live ADR registry at morph time.

---

## Context

Prism's central deployment (§3.1, DC-005) and the full multi-surface UI directive (§11.3) require
enterprise-grade identity integration from day one. Three interrelated concerns:

1. **Authentication (authN):** How do analysts log in to the central Prism service? Per-analyst
   credentials (username + password) are a viable fallback but not an enterprise differentiator.
   Enterprise buyers expect federation into their existing Identity Provider (IdP).

2. **Authorization (authZ):** What can each authenticated analyst do? Prism's day-2 RBAC model
   must support fine-grained roles (connector/dataset/dashboard-scoped) beyond the coarse 2-role
   model documented in Query.io (Team Admin / Team Member). This is G-12 (§11.5).

3. **Identity propagation:** The per-connection analyst identity established at authN must propagate
   through the entire Prism stack — to PrismQL execution (BC-2.05.002 amendment), credential
   resolution (§11.1 rotation audit), and the audit log. This binds tightly to ADR-051.

### Why OIDC AND SAML 2.0, not one or the other

The MSSP/enterprise buyer landscape bifurcates cleanly:

| Buyer type | Preferred IdP | Protocol |
|-----------|--------------|---------|
| Cloud-native / modern enterprise | Okta, Entra ID (Azure AD), Google Workspace, Auth0, AWS Cognito | **OIDC** (OAuth 2.0 + ID token) |
| Traditional enterprise / regulated | Active Directory Federation Services (ADFS), Ping Identity, Sailpoint, legacy SAML-only IdPs | **SAML 2.0** |
| OT/ICS customers (Prism Satellite topology) | Air-gapped ADFS, on-prem Ping, Shibboleth | **SAML 2.0** |

An MSSP building on Prism serves all three buyer categories. Supporting only OIDC locks out the
regulated and OT customer segments; supporting only SAML 2.0 locks out cloud-native buyers.
Both protocols must be supported from day one.

**Competitive differentiator:** Query.io does not document SSO support in its public documentation
(confirmed in the 2026-06-25 research pass, §11.3 SSO bullet). Shipping OIDC + SAML 2.0 per-tenant
IdP configuration on day-2 launch is a genuine differentiator for MSSP buyers who face compliance
or security-policy requirements mandating SSO (SOC 2, ISO 27001, many enterprise procurement checklists).

---

## Decision

### A. Protocol support: BOTH OIDC and SAML 2.0 from day one

Both protocols are supported at the per-tenant IdP configuration level. A tenant configures
EITHER an OIDC provider OR a SAML 2.0 IdP (or both, if their environment requires). The central
Prism service acts as an OIDC Relying Party and a SAML 2.0 Service Provider.

**OIDC flow:**
1. Analyst navigates to the Prism console (S2) or API.
2. Prism redirects to the tenant's configured OIDC Authorization Server.
3. The IdP authenticates the analyst (MFA, Passkey, etc. — managed by the IdP, not Prism).
4. The IdP returns an ID token (JWT) + access token to Prism.
5. Prism validates the ID token (issuer, audience, signature, expiry), extracts the analyst's
   identity claims (`sub`, `email`, `groups` or `roles`), and issues a Prism session token.
6. The session token propagates to per-connection analyst identity (ADR-051) and all downstream.

**SAML 2.0 flow (SP-initiated, the standard for web SPs):**
1. Analyst navigates to the Prism console and selects their organization.
2. Prism generates a SAML AuthnRequest and redirects the analyst's browser to the tenant's IdP.
3. The IdP authenticates and returns a signed SAML Assertion to Prism's ACS (Assertion Consumer Service) endpoint.
4. Prism validates the assertion (signature, conditions, audience restriction, not-before/not-after).
5. Prism extracts identity claims from the assertion attributes (NameID + configured attribute mappings).
6. Prism issues a Prism session token; identity propagates per ADR-051.

**Implementation vehicle:** A well-maintained Rust library is preferred for both protocols.
As of 2026, the Rust ecosystem for OIDC/OAuth 2.0 (openidconnect crate, oauth2 crate) is more
mature than the Rust SAML 2.0 ecosystem. If a production-quality Rust SAML 2.0 implementation
does not meet the bar at implementation time (see OD-3 in Open Decisions), the architect-approved
fallback is to use a sidecar identity proxy (Dex, or HashiCorp Boundary) that exposes a
normalized OIDC interface to Prism, with the proxy handling SAML 2.0 internally. This preserves
the single-protocol internal Prism code path while supporting SAML 2.0 for tenants externally.

### B. Per-tenant IdP configuration lifecycle

Each Prism tenant configures its own IdP independently. The lifecycle is:

1. **Configure** — a Tenant-Admin provides IdP metadata (OIDC: issuer URL + client credentials;
   SAML: IdP metadata XML or metadata URL). Prism parses and validates the metadata.
2. **Validate** — Prism performs a metadata health check: OIDC discovery endpoint reachable +
   cert valid; SAML IdP descriptor parsed + signing cert valid + SSO URL reachable.
3. **Test login** — Tenant-Admin initiates a test authentication flow against the configured IdP
   in a sandboxed session. Prism reports success or a structured failure reason (unreachable IdP,
   expired cert, audience mismatch, group claim not found, etc.).
4. **Activate** — after successful test login, the Tenant-Admin promotes the IdP configuration
   to `active`. From this point, all new analyst sessions for the tenant route through the
   configured IdP.
5. **Fallback** — if the active IdP becomes unreachable, the Prism admin UI (U1) supports
   emergency access via a Prism-local break-glass account (stored in the built-in encrypted
   secret store, §11.1) for the Tenant-Admin only. This prevents full tenant lockout during
   IdP outage. The break-glass account is audit-logged and rate-limited.

IdP configuration is versioned and audited (§11.2 config versioning). A Tenant-Admin cannot
delete an active IdP config without first switching tenants to an alternative or Prism-local auth.

### C. Group/role mapping into fine-grained RBAC (G-12)

Identity tokens (OIDC claims or SAML attributes) carry group membership from the IdP
(e.g., `groups: ["soc-analyst", "detection-engineers"]`). Prism maps these IdP groups to
Prism roles via a per-tenant configurable mapping table. Prism does NOT read the IdP's directory
directly — only what the IdP asserts in the token.

**Fine-grained role model (G-12, §11.5):**

Query.io's 2-role model (Team Admin / Team Member) is documented as a competitive gap. Prism's
role model is connector/dataset/dashboard-scoped:

| Role | Scope | Permissions |
|------|-------|------------|
| Platform-Admin | Cross-tenant (Prism operator) | All tenants; system config; user impersonation (with audit) |
| Tenant-Admin | Per-tenant | User management; IdP config; connector management; billing; break-glass |
| Detection-Engineer | Per-tenant | Create/edit/delete detection rules; manage retention policies; all analyst permissions |
| Security-Analyst | Per-tenant | Run queries; view detection findings; create/edit cases; save queries; read-only connectors |
| Connector-Admin | Per-tenant | Add/edit/delete connector configs; credential rotation; connector health |
| Read-Only | Per-tenant | View queries, findings, cases; no create/edit/delete; no connector config |
| Custom roles | Per-tenant | Tenant-Admin can define custom roles from a permission set (connector-scoped, dataset-scoped, dashboard-scoped) |

Custom roles use a JSON role-definition model (informed by the Grafana pattern cited in §7 of the
UI research). A role definition is: `{ "name": "...", "permissions": [...] }` where each permission
is a typed capability (e.g., `connector:read`, `connector:write`, `detection:create`,
`query:execute`, `case:manage`).

**Group→role mapping configuration:**
```
# Per-tenant IdP group → Prism role mapping
[sso.group_role_map]
"soc-tier1" = "Security-Analyst"
"detection-team" = "Detection-Engineer"
"prism-admins" = "Tenant-Admin"
# Unmapped groups → default to Read-Only (conservative)
[sso.defaults]
unmapped_group_role = "Read-Only"
```

This mapping is managed by the Tenant-Admin in U1 (Admin console) and is audited/versioned.

### D. Identity binding to ADR-051 (per-connection analyst identity)

Every Prism session token carries the analyst's resolved Prism identity (`AnalystIdentity` entity
from domain-spec, §5.2):

- `analyst_id` (Prism-internal stable identifier, derived from the IdP `sub` claim + `org_id`)
- `org_id` (tenant boundary)
- `roles` (the Prism roles resolved from IdP group mapping at login time)
- `session_id` (per-connection; used for audit log correlation and session revocation)
- `auth_method` (OIDC or SAML 2.0; recorded for audit)
- `idp_issued_at` / `expires_at` (from the original ID token; Prism session expiry is bounded by IdP assertion validity)

This identity structure flows through the entire Prism stack per ADR-051:
- PrismQL execution: `analyst_id` + `org_id` bound to query context (cross-analyst isolation invariant DI-NEW-006)
- Credential resolution: every credential access audited with `analyst_id` + `session_id`
- Audit log: every state-mutating action (connector create/edit, rule create/edit, case status change, credential rotate) carries the full identity record

Session tokens are HTTP-only, Secure, SameSite=Strict cookies (binding to ADR-PROP-web-stack §Security canon, point 5). They are not stored in the TypeScript layer.

### E. IdP certificate management

Both OIDC (JWK key sets) and SAML 2.0 (IdP signing cert) require certificate rotation handling:

- **OIDC:** Prism fetches the JWKS from the IdP's discovery endpoint on session validation. If the
  IdP rotates its signing keys (standard practice), the next token validation fetches fresh JWKS
  automatically. A 5-minute cache on JWKS (standard practice) with jitter prevents thundering herd.
- **SAML 2.0:** The IdP signing certificate has an expiry date. Prism monitors the expiry date and
  emits a structured warning in the audit log + U1 admin console when the IdP cert is within 30 days
  of expiry. The Tenant-Admin must rotate the IdP metadata before expiry. Prism does NOT auto-rotate
  IdP certs (those are controlled by the customer's IdP team, not Prism).

---

## SCIM Provisioning — Recommendation (Flagged for Human Decision)

**What SCIM does:** SCIM 2.0 (System for Cross-domain Identity Management, RFC 7643/7644) is a
standard API for automated user provisioning and deprovisioning. When SCIM is enabled between an
enterprise IdP (Okta, Entra ID) and Prism, the IdP automatically:
- Creates a Prism user record when an employee joins the group authorized for Prism access.
- Updates the record when the employee's groups or attributes change.
- Deprovisions (disables or deletes) the Prism user when they leave the authorized group (or the company).

**Why it matters for MSSP/enterprise buyers:**
- Without SCIM, user management is manual: when an analyst leaves, an admin must manually revoke
  their Prism access. This is an audit and security risk (stale accounts).
- With SCIM, deprovisioning happens automatically within minutes of the IdP removing the user from
  the Prism-access group. This is a hard requirement in many enterprise procurement checklists and
  SOC 2 / ISO 27001 audits.
- Large MSSP teams turn over analysts. SCIM at scale is not optional for large MSSP customers.

**Architect recommendation: INCLUDE SCIM 2.0 in day-2 scope.** Rationale:

1. **Security posture.** Timely deprovisioning is a genuine security control. Manual-deprovisioning
   is an account-lifecycle risk that enterprise buyers flag as a security gap.
2. **Procurement friction.** Many enterprise and regulated-industry procurement checklists have
   a SCIM checkbox. Absence is a deal-blocker, not a "nice to have."
3. **Implementation cost is bounded.** SCIM 2.0 is a REST API with defined endpoints
   (`/Users`, `/Groups`). It is not a complex integration; it is a CRUD API with some filter semantics.
   The backend already has user record management (U1 admin console); SCIM is an additional
   delivery mechanism for the same operations. Estimated: 3-5 implementation stories, not an epic.
4. **Timing advantage.** Competitors that lack SCIM (or document it poorly) create a window.
   Query.io does not document SCIM.

**However, this is flagged for human decision (OD-2 below).** The rationale against including it
is that it adds implementation scope to the already-large day-2 authZ epic (E-CENTRAL-AUTHZ-001).
If the human determines that SCIM is not blocking for the target early customers, deferring it to
a follow-on epic (E-CENTRAL-AUTHZ-002) is architecturally safe — SCIM touches only the user record
layer, not the core authN/authZ flows. But SCIM must be a concrete planned epic with a delivery
date, not a vague "later."

---

## Consequences

### Positive

- OIDC + SAML 2.0 day-one support captures both cloud-native and regulated-enterprise buyer segments.
- Per-tenant IdP config with test-login lifecycle removes onboarding friction for new MSSP customers.
- Fine-grained RBAC (G-12) is a documented differentiator over Query.io's 2-role model.
- Group→role mapping from IdP groups means Prism does not need its own directory — it delegates
  identity management to the customer's existing IdP, reducing Prism's compliance footprint.
- Identity propagates into audit trail and credential resolution (ADR-051 binding) from day one.

### Negative / Risks

- **SAML 2.0 Rust ecosystem maturity.** The `samael` crate (most maintained Rust SAML 2.0 library
  as of 2026) is less battle-tested than the Java/Go/Python SAML libraries. If `samael` does not
  meet the bar at implementation time, the sidecar-proxy fallback (Dex) must be evaluated. Dex adds
  a deployment dependency but is production-proven.
- **Per-tenant IdP configuration surface is a new attack vector.** A malicious Tenant-Admin could
  configure a rogue IdP that issues tokens for any `sub` value, enabling impersonation. Mitigation:
  (a) Platform-Admins can audit and revoke tenant IdP configs; (b) IdP metadata validation at
  configure time (cert pinning option); (c) `org_id` scoping ensures cross-tenant impersonation is
  architecturally impossible even with a rogue IdP (one tenant's IdP cannot issue tokens scoped to
  another tenant's `org_id`).
- **SCIM scope.** If included in day-2 scope, adds 3-5 implementation stories to E-CENTRAL-AUTHZ-001
  or a follow-on epic. Manageable but must be explicitly committed.
- **Break-glass account security.** The break-glass account (Tenant-Admin local fallback during IdP
  outage) must be tightly controlled: long random password, stored in the §11.1 built-in encrypted
  secret store, accessible only to the Tenant-Admin, with a separate audit stream. If compromised,
  it bypasses SSO controls. Mitigations: hardware token / TOTP required for break-glass activation;
  Platform-Admin notified on any break-glass use.

---

## Alternatives Considered

### OIDC only (SAML 2.0 as future work)

**Rejected.** SAML 2.0 is required by the OT/ICS and regulated-enterprise buyer segments Prism
targets via the Satellite topology. Deferring SAML 2.0 creates a market gap at launch that is
hard to close retroactively (buyers who evaluate and reject due to SAML gap rarely re-evaluate).
The sidecar-proxy option (Dex) means SAML 2.0 can be shipped even if the native Rust SAML library
is not ready — there is no valid technical excuse to defer SAML at day-2 launch.

### Prism-local identity only (username + password, no federation)

**Rejected for enterprise posture, retained as a fallback.** Username + password authentication is
maintained as the fallback for: (a) break-glass admin access, (b) customers who explicitly do not
have an IdP, (c) single-developer / small-team deployments. It is NOT the default for enterprise/MSSP
customers. Shipping SSO-only with no local auth fallback creates an unacceptable lockout risk.

### Delegate entirely to a third-party auth service (Auth0, Clerk, WorkOS)

**Rejected.** Prism has air-gap and on-prem customer requirements (OT/ICS satellite topology). A
third-party SaaS auth provider cannot function in air-gapped deployments. WorkOS specifically
targets SaaS-only deployments and cannot be self-hosted. Auth0 can be deployed on-prem as Private
Cloud but adds a heavy deployment dependency. The per-tenant IdP federation model Prism needs
(where Prism IS the SP and each customer brings their own IdP) is better implemented as a
first-party layer over standard protocols than by adding a SaaS dependency that cannot be
air-gapped.

---

## Open Decisions for Human

| # | Question | Stakes | Recommendation |
|---|----------|--------|----------------|
| OD-1 | **SCIM 2.0 in day-2 scope or deferred?** See SCIM section above for full rationale. In scope = 3-5 additional stories in E-CENTRAL-AUTHZ-001 or a new E-CENTRAL-AUTHZ-002. Deferred = concrete epic with a delivery date (not vague "later"). | Security posture, enterprise procurement, MSSPs with analyst churn | **Recommend: INCLUDE in day-2.** Deprovisioning is a genuine security control, not cosmetic. If scope pressure requires deferral, the architect recommends E-CENTRAL-AUTHZ-002 as a follow-on epic, scoped and dated, NOT a vague backlog item. |
| OD-2 | **SAML 2.0 implementation path: native Rust vs sidecar proxy (Dex)?** The `samael` crate is the most maintained Rust SAML 2.0 implementation as of 2026 but is less battle-tested than Go/Java SAML libraries. Human: acceptable to use `samael` in production with adversarial review, or prefer the Dex sidecar? | Implementation risk vs deployment simplicity | **Recommend: attempt native Rust (`samael`) with adversarial review; fall back to Dex if adversary finds critical gaps.** Dex adds a Go binary deployment dependency, which is architecturally cleaner than the implementation risk it avoids — but native Rust is preferable if the library passes review. Architect to evaluate at implementation time. |
| OD-3 | **JIT (just-in-time) user provisioning vs SCIM-only?** JIT provisioning: the first time an analyst authenticates via SSO, Prism automatically creates their user record if their IdP group maps to a Prism role (no manual admin action required; no SCIM needed for basic user creation). SCIM adds automated deprovisioning and attribute sync. These are complementary. Scope JIT-provisioning in the core authN flow (low effort, high UX value) and SCIM separately? | Onboarding friction, deprovisioning security | **Recommend: JIT provisioning IN E-CENTRAL-AUTHZ-001 (it is trivial once group→role mapping is implemented). SCIM deprovisioning as the human-decision OD-1 item above.** JIT and SCIM are not either/or; JIT handles creation, SCIM handles lifecycle updates and deprovisioning. |
| OD-4 | **MFA enforcement at Prism level vs IdP level?** Enterprise SSO delegates MFA to the IdP (the IdP asserts the authentication strength in the token; Prism trusts the assertion). For customers with Prism-local auth (no IdP), Prism must enforce MFA itself (TOTP at minimum). Scope: is Prism-level TOTP required in day-2, or is "Prism-local auth is fallback/break-glass only; MFA enforced at IdP for SSO users" acceptable? | Security posture, compliance (SOC 2) | **Recommend: TOTP for Prism-local (break-glass) accounts in day-2 scope. SSO users rely on IdP-enforced MFA (Prism cannot and should not override IdP MFA). This satisfies SOC 2 requirements without duplicating MFA for SSO users.** |
| OD-5 | **Session duration defaults.** ADR-051 and ADR-PROP-web-stack both mention session expiry and idle timeout. The concrete defaults must be specified: recommended defaults are 8-hour max session lifetime + 30-minute idle timeout, with Tenant-Admin override in the per-tenant config. Human: are these defaults acceptable, or is a specific session duration policy required for target customers? | Compliance, UX (analysts hate constant re-auth) | **Recommend: 8h max + 30min idle as defaults; Tenant-Admin configurable (longer for OT environments where re-auth disrupts shift handoffs; shorter for high-security regulated environments). Defaults land at the conservative end of industry practice.** |
