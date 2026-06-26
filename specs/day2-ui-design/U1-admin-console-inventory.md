---
document_type: ux-spec
surface: U1
status: PROPOSED/DRAFT
do_not_execute: true
produced_by: ux-designer
timestamp: "2026-06-26"
traces_to:
  - matured-vision-day2-requirements.md §11.1 (credential storage)
  - matured-vision-day2-requirements.md §11.2 (config management)
  - matured-vision-day2-requirements.md §11.3 §11.3.1 §11.5 (UI surfaces, G-8/G-9/G-12)
  - research/ui-requirements-2026-06-25.md §7 (multi-tenant admin/RBAC)
  - research/ui-requirements-2026-06-25.md §3–§4 (principles, RBAC)
  - research/ui-webstack-admin-rbac-2026-06-25.md
provenance: >
  2026-06-25 side-analysis addendum — day-2 UI design. Produced by ux-designer.
  do_not_execute. Separate from the live factory pipeline.
  Pending PO + architect adjudication at day-2 morph time.
  All content is PROPOSED/DRAFT.
note: >
  This spec covers the U1 multi-tenant admin/ops console (E-UI-ADMIN-001).
  Investigations console screens are in S2-investigations-console.md.
  RBAC permission model ties to ADR-051 (per-connection analyst identity),
  ADR-052 (central credential custody), and §11.5 G-12 (fine-grained RBAC).
---

# U1 — Multi-Tenant Admin / Ops Console
## UX Specification (PROPOSED/DRAFT)

> All content in this document is PROPOSED/DRAFT, pending PO + architect adjudication
> at day-2 morph time. Decision references are to matured-vision-day2-requirements.md
> sections and ui-requirements-2026-06-25.md decision IDs.

---

## 0. Cross-Screen Architecture

### 0.1 Information Architecture Map

```
╔══════════════════════════════════════════════════════════════════╗
║  U1 GLOBAL SHELL                                                 ║
║  ┌──────────────────────────────────────────────────────────┐   ║
║  │  [≡] Prism Admin  |  [org: Acme MSSP ▾]   [⌘K]  [🔔] [👤] ║   ║
║  └──────────────────────────────────────────────────────────┘   ║
║                                                                  ║
║  LEFT NAV                     MAIN CONTENT AREA                 ║
║  ─────────────────────────                                       ║
║  🔍 Investigations  (→ S2)                                       ║
║  📊 Dashboards      (→ S2)                                       ║
║  🔌 Connectors                                                   ║
║  🛡 Admin                                                        ║
║     └ Tenants                                                    ║
║     └ Users & Roles                                              ║
║     └ Credentials                                                ║
║     └ Audit Log                                                  ║
║     └ SSO                                                        ║
║  ⚙  Settings                                                     ║
║     └ Health                                                     ║
║     └ Config / Policies                                          ║
╚══════════════════════════════════════════════════════════════════╝
```

The U1 admin console shares the same global shell as S2 (the investigations console), with the Admin and Settings sections visible only to roles with admin permissions. A user with a Tenant-Admin role will see admin items scoped to their tenant. A Platform Admin (MSSP operator) sees all tenants.

### 0.2 Sectioned IA (§7)

| Section | Who sees it | Content |
|---------|-------------|---------|
| Investigations | All roles | S2 navigation shortcut (same nav items as S2) |
| Dashboards | All roles | S2 dashboard shortcut |
| Connectors | Connector-Admin + above | Connector list, health, configure-schema wizard, credential rotation shortcut |
| Admin | Tenant-Admin + above | Tenants, Users & Roles, Credentials, Audit Log, SSO |
| Settings | Tenant-Admin + above | Health/observability, Config/policies |

### 0.3 Role Model (§7; §11.5 G-12; UI-D4)

Fine-grained RBAC beyond the Query.io two-role model (Team Admin / Team Member). Grafana JSON-role-definition pattern: roles are defined as permission sets, assignable per-user or per-group, scoped to tenant or sub-resources.

**Built-in roles (PROPOSED; PO to finalize):**

| Role | Scope | Capabilities |
|------|-------|-------------|
| **Platform-Admin** | All tenants (MSSP operator) | All capabilities on all tenants; cross-tenant dashboards; tenant create/delete |
| **Tenant-Admin** | One tenant | Full user/role management; connector config; credential management; SSO; audit log; all S2 capabilities |
| **Security-Analyst** | One tenant | All S2 investigation/query/detection capabilities; read-only connector health; cannot manage users/creds/SSO |
| **Detection-Engineer** | One tenant | All S2 capabilities; author + manage detection rules; query library management; cannot manage users/creds/SSO |
| **Connector-Admin** | One tenant | Add/edit/delete connectors; configure-schema wizard; credential rotation; cannot view investigation data |
| **Read-Only** | One tenant | View-only access to S2 dashboards, cases, alerts; no query execution; no config |

**Custom roles** (day-2 enhancement; §11.5 G-12): Tenant-Admin and Platform-Admin can define custom roles by composing permission sets. Role definition stored as JSON per-tenant in the central config store (§11.2). Example custom role: "SOAR-Integration" with only the permissions to read findings and post to alert destinations.

**Resource-scoped permissions** (day-2 enhancement): individual connectors, detection-rule groups, and dashboard sets can have per-resource ACLs narrowing access further. Example: Connector-Admin for Splunk only (not CrowdStrike).

### 0.4 Dangerous-Action Guards (UI-D4; §7)

A set of destructive or high-impact actions require an escalated confirmation pattern — not just a standard "Are you sure?" dialog:

1. **Consequence-restating dialog**: the dialog explicitly states the consequence in plain language ("This will permanently delete tenant Acme Corp and all associated data, users, and credentials. This cannot be undone.")
2. **Re-enter-name confirmation**: the user must type the name of the resource being deleted/changed (e.g., "Type 'Acme Corp' to confirm").
3. **Optional MFA step**: for the highest-risk actions (delete tenant, rotate all credentials, change SSO provider), an additional MFA challenge can be configured by Platform-Admin.
4. **Audit log entry**: every dangerous action creates an immutable audit log entry regardless of whether it succeeds or is cancelled.

**Actions requiring the full escalated pattern:**
- Delete tenant
- Delete user (permanent)
- Rotate a credential secret
- Change SSO configuration
- Disable/delete a detection rule in production
- Bulk delete audit logs (restricted to Platform-Admin; additional MFA always required)

---

## Screen U1-1 — Tenant Management

**Traces to:** §11.3 (multi-tenant service); §11.1 (per-tenant DEK); §7 (tenant management); G-8, G-9; UI-D4

### Purpose & Persona

Create, view, configure, and delete tenants. Platform-Admin only. Shows the multi-tenant landscape of the Prism deployment.

**Persona:** Platform-Admin (MSSP operator).

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Tenants                           [+ New Tenant]  [Export CSV]  │
│  ─────────────────────────────────────────────────────────────── │
│  [Search…]  [Status ▾]  [Plan ▾]  [Created ▾]                   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ Acme Corp        ACTIVE  12 users  4 connectors  2026-01-15  ││
│  │   Admin: jdoe@acme.com  |  SSO: SAML/Okta                   ││
│  │   Data isolation: per-tenant DEK (key ID: dek-0x1a...)       ││
│  │   [Configure] [View health] [Audit log] [Delete ⚠]          ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ Beta Corp        ACTIVE  3 users   1 connector   2026-03-10  ││
│  │   Admin: admin@beta.io  |  SSO: not configured              ││
│  │   [Configure] [View health] [Audit log] [Delete ⚠]          ││
│  └──────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- Per-tenant row shows: name, status (Active/Suspended/Deleted), user count, connector count, creation date, admin contact, SSO status, DEK key reference (key ID only, never the key material) — (§11.1)
- "Delete ⚠" triggers the full dangerous-action escalated confirmation pattern (§0.4)
- "Data isolation" row shows that each tenant has an isolated per-tenant DEK; the key ID is displayed for audit traceability but the key material is NEVER shown

### Tenant Detail / Configuration

Clicking "Configure" opens a full-width tenant configuration page:

```
┌──────────────────────────────────────────────────────────────────┐
│ Tenant: Acme Corp                                                 │
│ ─────────────────────────────────────────────────────────────── │
│ [Overview] [Users & Roles] [Connectors] [Credentials] [SSO]     │
│ [Policies] [Audit Log] [Health]                                  │
│ ─────────────────────────────────────────────────────────────── │
│ TAB CONTENT (see individual screens below)                       │
└──────────────────────────────────────────────────────────────────┘
```

The tenant configuration page uses the same tab-based layout as Case Detail (S2 Screen 2); each tab corresponds to a U1 admin screen scoped to this tenant.

### States

| State | Rendering |
|-------|-----------|
| Loading | Skeleton rows |
| Empty (no tenants) | "No tenants configured. [+ Create first tenant]" |
| Tenant suspended | Row shows "SUSPENDED" badge; [Resume] and [Delete] actions only |
| Tenant provisioning | Row shows "PROVISIONING" spinner; configuration not yet available |
| DEK initialization error | Row shows "DEK ERROR" badge; [Retry DEK initialization] link |

### Dangerous Actions

- **Delete tenant**: full escalated pattern (§0.4): consequence dialog + type tenant name + optional MFA

---

## Screen U1-2 — User & Role Management

**Traces to:** §11.5 G-12 (fine-grained RBAC); §7 (RBAC, dangerous-action guards); UI-D4; ADR-051

### Purpose & Persona

Manage users within a tenant. Assign roles. Create custom roles. Enforce least-privilege defaults. Scoped to current tenant for Tenant-Admin; cross-tenant view for Platform-Admin.

**Persona:** Tenant-Admin; Platform-Admin.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Users & Roles — Acme Corp       [+ Invite User]  [+ New Role]  │
│  ─────────────────────────────────────────────────────────────── │
│  TABS: [Users] [Roles] [Groups]                                  │
│  ─────────────────────────────────────────────────────────────── │
│  [Search…]  [Role ▾]  [Status ▾]  [Last active ▾]               │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ jsmith@acme.com   Tenant-Admin   Active  2026-06-25  [Edit]  ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ alice@acme.com    Security-Analyst  Active  2026-06-24  [Edit]││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ bob@acme.com      Detection-Engineer  Pending invite  [Resend]││
│  └──────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

### User Invite / Edit Flow

```
Invite user:
1. Enter email address
2. Select role(s) — dropdown shows built-in + custom roles
   └─ Role description shown below dropdown:
      "Security-Analyst: can investigate, query, and triage.
       Cannot manage users, connectors, or credentials."
3. Optional: set resource-scoped overrides (advanced; collapsible)
   └─ "Can access: [all connectors ▾]  |  [all dashboards ▾]"
4. [Send invite]

Least-privilege default: the role wizard always defaults to
Read-Only. The admin must actively select a higher role.
```

### Roles Tab (Custom Role Builder)

```
┌──────────────────────────────────────────────────────────────────┐
│  Roles — Acme Corp          [+ New custom role]                  │
│  ─────────────────────────────────────────────────────────────── │
│  BUILT-IN ROLES (read-only — cannot be edited)                   │
│  Platform-Admin · Tenant-Admin · Security-Analyst                │
│  Detection-Engineer · Connector-Admin · Read-Only                │
│                                                                   │
│  CUSTOM ROLES                                                     │
│  ├─ SOAR-Integration   2 users   [Edit] [Delete ⚠]              │
│  │  Permissions: findings.read · alerts.read · destinations.write │
│  │                                                               │
│  └─ Compliance-Viewer  1 user   [Edit] [Delete ⚠]              │
│     Permissions: dashboards.read · reports.read · cases.read     │
└──────────────────────────────────────────────────────────────────┘
```

**Custom role builder:**
- Select permissions from a checklist organized by resource type (Investigations, Detections, Connectors, Admin)
- "Explain this permission" tooltip on each checkbox
- Role definition exported/imported as JSON (Grafana JSON-role-definition pattern)
- Preview: "A user with this role CAN: [list]. They CANNOT: [list]."

### States

| State | Rendering |
|-------|-----------|
| Pending invite | "Pending" badge; [Resend] and [Cancel invite] |
| User deactivated | "Inactive" badge; [Reactivate] |
| Last active >90 days | Amber "Dormant" badge with tooltip suggesting review |
| Role edit — in use | "This role is assigned to N users. Changes will apply immediately." |

### Dangerous Actions

- **Remove user (permanent):** consequence dialog + type email + optional MFA
- **Remove Tenant-Admin role from the last admin:** blocked with "Cannot remove the last Tenant-Admin. Promote another user first."
- **Delete custom role (in use):** blocked with "This role is assigned to N users. Reassign or remove those users first."

### Accessibility (UI-D3)

- Role selector uses `<select>` with progressive disclosure for advanced options
- Custom role permission checklist: grouped `<fieldset>` per resource type; `<legend>` labels
- All destructive buttons have aria-label describing the consequence (e.g., "Remove alice from tenant — this will revoke all access")

---

## Screen U1-3 — Connector Config + Schema-Mapping Wizard

**Traces to:** §13 (static vs dynamic connectors); §13.2 (configure-schema workflow); §11.2 (config RBAC + audit); §7 (connector config wizard); E-CONNECTOR-DYNAMIC-001; UI-D4

### Purpose & Persona

Full connector configuration: add, edit, delete connectors. Static connectors (security sensors) — credential onboarding only. Dynamic connectors (SIEMs, lakes, custom SQL/LDAP) — full schema introspection + configure-schema wizard.

**Persona:** Connector-Admin; Tenant-Admin.

### Layout (Connector List)

```
┌──────────────────────────────────────────────────────────────────┐
│  Connectors — Acme Corp         [+ Add Connector]                │
│  ─────────────────────────────────────────────────────────────── │
│  [Search…]  [Type ▾: Static / Dynamic / All]  [Health ▾]        │
│                                                                   │
│  STATIC CONNECTORS (pre-mapped; credential only)                 │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ ✓ CrowdStrike  Healthy  Static  OCSF  [Edit creds] [Test]   ││
│  │   Schema: 5 tables (TOML spec v1.3.2)  Latency: 320ms avg  ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ △ Armis        Degraded  Static  OCSF  [Edit creds] [Test]  ││
│  │   Connect timeout since 13:45 UTC  [Diagnose] [View errors] ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  DYNAMIC CONNECTORS (schema introspection required)              │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ ✓ Splunk-Main  Healthy  Dynamic  42 tables → OCSF            ││
│  │   Last introspected: 2026-06-24  Mapping v3  [Edit] [Test]  ││
│  │   [Configure schema] [View mapping] [Re-introspect]          ││
│  └──────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

### Configure-Schema Wizard (Dynamic Connectors)

A 5-step wizard for the schema-mapping workflow (§13.2). Triggered by [Configure schema] or when adding a new dynamic connector.

**Step 1: Authenticate**

```
┌──────────────────────────────────────────────────────────────────┐
│  Configure Schema — Splunk-Main (Step 1 of 5)           [Cancel] │
│  ─────────────────────────────────────────────────────────────── │
│  Connector type: Splunk (dynamic)                                │
│  Endpoint: https://splunk.acme.com:8089                         │
│                                                                   │
│  Credentials: [From secret store ▾: splunk-main-token]           │
│  (credential value is never displayed here — reference only)     │
│  [Test authentication]  ✓ Connected (splunk 9.1.2)               │
│                                                                   │
│  [Next: Introspect Schema →]                                     │
└──────────────────────────────────────────────────────────────────┘
```

**Step 2: Introspect Schema**

```
┌──────────────────────────────────────────────────────────────────┐
│  Configure Schema — Splunk-Main (Step 2 of 5)           [Cancel] │
│  ─────────────────────────────────────────────────────────────── │
│  Introspecting: querying index catalog and data models…          │
│  ████████░░ 80%                                                  │
│                                                                   │
│  Discovered: 156 indexes, 42 data models, 1.2M events (sample)  │
│                                                                   │
│  Auto-partition detection:                                        │
│  ✓ index field: 'index'                                         │
│  ✓ time field: '_time' → maps to OCSF 'time'                    │
│  ✓ host field: 'host' → maps to OCSF 'device.hostname'          │
│                                                                   │
│  [← Back]  [Next: Map Fields →]                                  │
└──────────────────────────────────────────────────────────────────┘
```

**Step 3: Map Fields to OCSF (or native schema)**

```
┌──────────────────────────────────────────────────────────────────┐
│  Configure Schema — Splunk-Main (Step 3 of 5)           [Cancel] │
│  ─────────────────────────────────────────────────────────────── │
│  Normalization target:                                           │
│  [Map to OCSF (security telemetry) ●]  [Keep native schema ○]   │
│                                                                   │
│  FIELD MAPPINGS (AI-assisted; review before saving)              │
│  Source field         → OCSF path           Confidence  [Edit]  │
│  ─────────────────────────────────────────────────────────────── │
│  src_ip               → src_endpoint.ip     HIGH   ✓            │
│  user                 → actor.user.name     HIGH   ✓            │
│  action               → activity_name       MEDIUM ⚠ [Review]   │
│  custom_field_xyz     → (unmapped)          —      [Map ▾]       │
│                                                                   │
│  ⚠ 3 fields could not be automatically mapped. Review required.  │
│  [← Back]  [Next: Preview Data →]                                │
└──────────────────────────────────────────────────────────────────┘
```

- AI-assisted field mapping: S3 agent suggests OCSF mappings; suggestions labeled "AI" + confidence; always editable — UI-D2 trust-first
- Unmapped fields: analyst can manually map, map to a custom extension, or mark as intentionally unmapped
- "Keep native schema" option: for non-security data (SQL databases, AD/LDAP) where OCSF normalization is not appropriate (§13.6 multi-schema)

**Step 4: Preview + Validate**

```
┌──────────────────────────────────────────────────────────────────┐
│  Configure Schema — Splunk-Main (Step 4 of 5)           [Cancel] │
│  ─────────────────────────────────────────────────────────────── │
│  Sample data (20 rows from last 1h):                             │
│                                                                   │
│  AG Grid preview: [OCSF fields] ← mapped from Splunk source      │
│  time | src_endpoint.ip | actor.user.name | activity_name | …   │
│  2026-06-25T14:20 | 10.0.1.5 | alice | user-logon | …           │
│                                                                   │
│  ⚠ 2 rows: activity_name = null (mapping produced null)          │
│  [Adjust mapping for these rows]  [Accept and proceed]           │
│                                                                   │
│  Validation: ✓ OCSF required fields present  ✓ Types match       │
│  [← Back]  [Next: Save & Activate →]                             │
└──────────────────────────────────────────────────────────────────┘
```

**Step 5: Save & Activate**

```
┌──────────────────────────────────────────────────────────────────┐
│  Configure Schema — Splunk-Main (Step 5 of 5)           [Cancel] │
│  ─────────────────────────────────────────────────────────────── │
│  Mapping summary:                                                │
│  42 tables · 156 fields mapped · 3 unmapped (intentional)        │
│  Version: v4 (was v3)  Change log: [view diff]                   │
│                                                                   │
│  ⚠ Mapping changes affect 3 active detection rules that query    │
│  this connector. [View affected rules]                           │
│                                                                   │
│  [← Back]  [Activate mapping]                                    │
│  → This will be stored versioned with rollback available.        │
└──────────────────────────────────────────────────────────────────┘
```

### States

| State | Rendering |
|-------|-----------|
| Connector auth failing | Step 1 shows error; wizard blocked at step 1 until auth succeeds |
| Introspection in progress | Step 2 shows progress bar; [Cancel introspection] available |
| Introspection timeout | Error state: "Introspection timed out. Check connectivity and try again." |
| Mapping saved | Success toast: "Schema mapping v4 activated for Splunk-Main." |
| Rollback to previous mapping | Available in [Edit] → [Version history] → [Restore v3] |

### Dangerous Actions

- **Delete connector**: full escalated pattern; if the connector is referenced by active detection rules, the consequence dialog lists those rules: "Deleting this connector will break N active detection rules: [list]. They will be automatically disabled."
- **Activate a new mapping that changes existing field paths**: confirmation shows which detection rules will be affected

---

## Screen U1-4 — Credential Rotation UX

**Traces to:** §11.1 (credential storage, server-grade); §7 (credential rotation UX, write-only/masked); G-8; UI-D4

### Purpose & Persona

Manage connector credentials and LLM API keys (for S3). Write-only/masked — secrets are NEVER displayed after entry. The only operations are: add, rotate, test, and (dangerous) delete.

**Persona:** Connector-Admin; Tenant-Admin.

### Core UX Principle: Write-Only / Never-Display

**Credentials are write-only.** After a secret is saved:
- The value is NEVER displayed (not masked with asterisks that can be revealed — the value is simply not available in the UI)
- The UI shows: credential name, type, creation date, last rotation date, last tested date, status
- "View value" does not exist. This matches the project memory's AI-opaque credential principle (AD-017, project_ai_opaque_credentials.md)
- The stored metadata (name, type, dates, status) IS visible for audit purposes
- This applies to both the built-in secret store AND external vault backends

### Layout (Credential List)

```
┌──────────────────────────────────────────────────────────────────┐
│  Credentials — Acme Corp           [+ Add Credential]            │
│  ─────────────────────────────────────────────────────────────── │
│  [Search…]  [Type ▾]  [Connector ▾]  [Status ▾]                 │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ crowdstrike-api-key   API Key   CrowdStrike   ACTIVE         ││
│  │   Added: 2026-01-15  Last rotated: 2026-05-01  Last test: ✓ ││
│  │   Backend: Built-in encrypted store (per-tenant DEK)        ││
│  │   [Rotate ↻]  [Test connection]  [Delete ⚠]                ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ armis-bearer-token    Bearer Token  Armis        ACTIVE      ││
│  │   Added: 2026-02-10  Last rotated: never  ⚠ Rotation overdue ││
│  │   [Rotate ↻]  [Test connection]  [Delete ⚠]                ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ vault-splunk-token    Vault ref.  Splunk        ACTIVE       ││
│  │   Backend: HashiCorp Vault (vault.acme.com)                 ││
│  │   Path: secret/acme/splunk/token  Last resolved: 2m ago     ││
│  │   [Test connection]  [Delete ⚠]  (rotation managed by Vault)││
│  └──────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

### Add Credential Flow

```
Add Credential:
1. Name: [crowdstrike-new-key]  (no spaces; used as reference ID)
2. Type: [API Key ▾ | Bearer Token | OAuth Client Credentials |
          Vault Reference | AWS Secrets Manager Reference]
3. Credential value entry:
   ┌─────────────────────────────────────────────────────────────┐
   │  Secret value: [•••••••••••••••••••••] (masked input)       │
   │  Confirm:      [•••••••••••••••••••••]                      │
   │                                                             │
   │  ⓘ This value will be stored encrypted under your           │
   │     tenant's Data Encryption Key (DEK). It cannot be        │
   │     retrieved or displayed after saving.                    │
   └─────────────────────────────────────────────────────────────┘
4. Backend: [Built-in encrypted store ● | External Vault ○]
   └─ If External Vault: vault type, address, path/ARN/resource
5. Connector assignment: [assign to: CrowdStrike ▾]
6. [Test connection before saving]  ✓ Connected
7. [Save credential]

AI-opacity note rendered in form:
"This credential will NEVER appear in query results, MCP
 tool output, logs, or AI agent context. The value is
 encrypted server-side and injected at the I/O boundary only."
```

### Rotate Credential Flow

```
Rotate: crowdstrike-api-key

1. "Enter the new credential value. The old value will be
   replaced immediately upon saving."
   ┌─────────────────────────────────────────────────────────────┐
   │  New secret value: [•••••••••••••••••••••]                  │
   │  Confirm:          [•••••••••••••••••••••]                  │
   └─────────────────────────────────────────────────────────────┘
2. [Test new credential before activating]  ✓ Connected

3. HOT RELOAD OPTION (§3.6 DC-002; §11.1 hot credential reload):
   [Apply without restart ● (recommended)]
   [Apply on next restart ○ (if hot reload unavailable)]

4. [Activate rotation]

After activation:
- Old value is securely overwritten
- Audit log entry: "Credential crowdstrike-api-key rotated by
  jsmith at 2026-06-25T15:30 UTC. Hot reload applied."
- "Last rotated" timestamp updated
```

### Credential Health Indicators

| Status | Meaning | Action |
|--------|---------|--------|
| ACTIVE | Credential is current, last test passed | None needed |
| ⚠ Rotation overdue | Last rotated >90 days (configurable) | [Rotate ↻] |
| ✗ Test failed | Last connection test failed | [Rotate ↻] or [Diagnose] |
| Vault unreachable | External vault backend not responding | [Check vault connectivity] |
| DEK error | Decryption of stored credential failed | Contact Platform-Admin |

### External Vault Integration (§11.1)

- Supported backends: HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager, Azure Key Vault
- For Vault-referenced credentials: the UI shows the vault path/ARN (the reference), never the resolved value
- "Last resolved" shows when Prism last successfully fetched the secret from the vault backend
- Rotation for external vault credentials is managed in the vault itself; [Test connection] verifies Prism can still resolve the reference

### States

| State | Rendering |
|-------|-----------|
| Credential test in progress | Spinner + "Testing..." |
| Credential test passed | "✓ Connected (response 200, 320ms)" |
| Credential test failed | "✗ Connection failed: [error code]. [View details]" |
| Hot reload complete | "Credential rotated and hot-reloaded. No restart required." |
| Rotation required (connector error) | Banner on the relevant connector's row in U1-3: "Credential error — rotate or check credentials" |

### Dangerous Actions

- **Delete credential**: consequence dialog: "Deleting crowdstrike-api-key will immediately disconnect the CrowdStrike connector. Queries to CrowdStrike will fail until a new credential is added. Type 'crowdstrike-api-key' to confirm." + optional MFA

---

## Screen U1-5 — Audit Log Viewer

**Traces to:** §11.2 (audit + versioning); §7 (audit-log viewer); UI-D4

### Purpose & Persona

Immutable audit trail of all administrative actions, analyst actions, and AI actions. Searchable and filterable. Optional AI clustering of related events.

**Persona:** Tenant-Admin; Platform-Admin; Compliance officer; Security reviewer.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Audit Log — Acme Corp                   [Export CSV]            │
│  ─────────────────────────────────────────────────────────────── │
│  [Search…] [Actor ▾] [Action ▾] [Resource ▾] [Outcome ▾]        │
│  [Time ▾: Last 7d]  [Severity ▾]                                 │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ 2026-06-25T15:30 jsmith      rotate-credential  crowdstrike  ││
│  │   Outcome: SUCCESS  Hot reload applied  IP: 10.0.1.1         ││
│  │   [View details]                                              ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ 2026-06-25T15:25 prism-ai    generate-query     case-042     ││
│  │   Outcome: SUCCESS  Query: SELECT * FROM…  [View full query] ││
│  │   AI action — approved by jsmith                             ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ 2026-06-25T14:00 alice       run-query          federated    ││
│  │   Outcome: SUCCESS  Sources: 3 queried, 1 degraded          ││
│  │   [View query]  [View results summary]                       ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  AI CLUSTERING (optional; toggle)                                │
│  [Cluster related events] → shows groups of related actions       │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **TanStack Virtual** for the log list (can be very long)
- **Immutable records**: no edit/delete from the UI; records are write-once (bulk delete requires Platform-Admin + MFA via dangerous-action pattern)
- **AI actor entries**: AI-generated actions are clearly marked with "AI action" label + the human who approved (if applicable)
- **Source coverage in query logs**: when an analyst's query ran against degraded sources, the audit record notes which sources answered/degraded — traces directly to §3.6
- **AI event clustering** (optional toggle): S3 agent groups related audit events (e.g., "5 events related to the Armis credential rotation at 15:30") — always labeled "AI grouping", never replaces raw events
- **Export**: CSV download with all visible columns; date-range scoped

### Filters

| Filter | Options |
|--------|---------|
| Actor | User name/email; "prism-ai" for AI actions; "system" for automated actions |
| Action | add-user, remove-user, rotate-credential, configure-connector, change-role, run-query, create-case, AI-generate-query, login, logout, etc. |
| Resource | connector name, credential name, user name, case ID, etc. |
| Outcome | SUCCESS, FAILURE, BLOCKED (dangerous-action confirmation cancelled) |
| Severity | INFO, WARNING, HIGH (for blocked or failed destructive operations) |

### States

| State | Rendering |
|-------|-----------|
| No logs in time range | "No audit events found for the selected filters." |
| Streaming new events | New rows appear at top; "N new events" badge if scrolled down |
| Log rotation warning | Banner: "Audit logs older than 90 days will be archived to [cold storage]. [Configure retention]" |
| Export queued | "Exporting… [N%]  [Cancel]"; download link appears when ready |

---

## Screen U1-6 — Health / Observability Dashboard

**Traces to:** §7 (health/observability dashboards); §3.6 (partial-result semantics); G-9; E-UI-ADMIN-001

### Purpose & Persona

Real-time and historical health metrics for the Prism deployment. Connector up/down, query/ingestion latency, DataFusion execution times, errors. Tenant-scoped for Tenant-Admin; cross-tenant for Platform-Admin.

**Persona:** Tenant-Admin; Platform-Admin; SRE/DevOps operator.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Health — Acme Corp    [Tenant ▾: Acme Corp]  [Time: 1h ▾]      │
│  ─────────────────────────────────────────────────────────────── │
│  PLATFORM STATUS: ✓ All systems operational                      │
│  ─────────────────────────────────────────────────────────────── │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Connectors   │  │ Queries      │  │ Errors (1h)          │   │
│  │ 3/4 healthy  │  │ Median: 1.2s │  │ 12 (↑ from 4)        │   │
│  │ △ Armis down │  │ P95: 8.4s    │  │ 8 timeout · 4 other  │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
│                                                                   │
│  CONNECTOR STATUS                   QUERY LATENCY TREND          │
│  ✓ CrowdStrike  320ms avg           [ECharts line: P50/P95]      │
│  ✓ Cyberint     580ms avg                                        │
│  ✓ Claroty      740ms avg           DATAFUSION EXEC TIMES        │
│  △ Armis        ✗ timeout           [ECharts bar: per source]    │
│                                                                   │
│  ERROR LOG (recent)                                              │
│  14:55 Armis connect_timeout (5s) — CCS degraded                │
│  14:32 Armis connect_timeout (5s) — CCS degraded                │
│  [View all errors]  [View Armis diagnostic]                      │
│                                                                   │
│  SATELLITE HEALTH (if deployed)                                  │
│  ✓ SAT-US-EAST  12ms RTT  ✓  |  △ SAT-OT-ZONE-A  degraded      │
└──────────────────────────────────────────────────────────────────┘
```

### Key Components

- **ECharts** for all latency and error trend charts
- **Connector status tiles:** per-connector health; latency avg/P95; click → connector-specific deep-dive view
- **DataFusion exec times:** per-query-plan breakdown (parse, plan, execute, normalize); helps identify slow connectors or expensive query plans
- **Satellite health summary** (if satellite topology deployed; §3.2): RTT per satellite + degraded-subtree status
- **Alerting rules** (day-2): configurable alerts on health thresholds (e.g., "Alert me if P95 query latency > 10s" or "Alert me if any connector is down for > 5 minutes") — routes to admin notification channel

### Cross-Tenant View (Platform-Admin)

Platform-Admin sees an aggregated view across all tenants:

```
Tenants: [All ● | Select... ○]

Tenant        Connectors  P95 Latency  Errors (1h)  Status
Acme Corp     3/4 healthy  8.4s         12           △ Degraded
Beta Corp     1/1 healthy  1.1s         0            ✓ Healthy
```

---

## Screen U1-7 — SSO Wizard

**Traces to:** §11.3 (SSO differentiator over Query); §7 (SSO wizard — SAML/OIDC per-tenant); E-UI-ADMIN-001; UI-D4

### Purpose & Persona

Configure SAML or OIDC single sign-on per tenant. Validate and test the SSO configuration before activating. Per-tenant (Tenant-Admin); visible to Platform-Admin for all tenants.

**Persona:** Tenant-Admin; IT administrator.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  SSO Configuration — Acme Corp     [Protocol: SAML ▾]           │
│  ─────────────────────────────────────────────────────────────── │
│  Status: ✓ SSO active (Okta SAML)                               │
│  ─────────────────────────────────────────────────────────────── │
│                                                                   │
│  PRISM SERVICE PROVIDER DETAILS (read-only; share with your IdP) │
│  Entity ID:   https://prism.acme.io/saml/acme-corp              │
│  ACS URL:     https://prism.acme.io/saml/acme-corp/acs          │
│  Metadata:    [Download SP Metadata XML]                         │
│                                                                   │
│  IDENTITY PROVIDER CONFIGURATION                                 │
│  IdP metadata URL:  [https://acme.okta.com/app/prism/metadata]  │
│  OR upload metadata XML: [Choose file…]                          │
│  Certificate: [Valid until 2027-01-01]  [Update]                │
│                                                                   │
│  ATTRIBUTE MAPPING                                               │
│  NameID format:   [Email ▾]                                      │
│  Groups claim:    [groups]  → used for role auto-assignment      │
│  Role mapping:    [Edit role mapping rules ▾]                    │
│   admin group → Tenant-Admin role                               │
│   analyst group → Security-Analyst role                         │
│                                                                   │
│  [Test SSO login (opens test flow)]  ✓ Last test passed 2026-06-20│
│  [Save & Activate]  [Deactivate SSO ⚠]                          │
└──────────────────────────────────────────────────────────────────┘
```

### OIDC Configuration

Same layout as SAML with OIDC-specific fields:
- Client ID, Client secret (write-only; never displayed after save), OIDC discovery URL
- Scopes: `openid email profile groups` (configurable)
- Claims mapping: `email` claim → user email; `groups` claim → role assignment

### Test SSO Flow

1. Click [Test SSO login]
2. A new browser tab opens the IdP login page
3. Admin logs in with a test account
4. On success: test tab shows "SSO test passed — welcome, [email]"
5. Main admin page: "✓ Last test passed [timestamp]"
6. On failure: error shown with SAML/OIDC error details (assertion validation failure, attribute mapping error, etc.)

### States

| State | Rendering |
|-------|-----------|
| SSO not configured | "SSO is not configured. [Set up SAML] or [Set up OIDC]" |
| SSO in review (pending test) | "SSO configured but not yet tested. [Test SSO login] before activating." |
| SSO test failed | Red banner: "SSO test failed: [error]. [View details] [Retry test]" |
| SSO active | Green banner: "SSO is active. Users are redirected to your IdP for login." |
| SSO certificate expiring soon | Amber banner: "IdP certificate expires in 14 days. [Update certificate]" |

### Dangerous Actions

- **Deactivate SSO**: consequence dialog: "Deactivating SSO will require all users to log in with their Prism password. Users who were invited via SSO-only (no Prism password) will be unable to log in until they reset their password. Type 'deactivate' to confirm."
- **Change SSO provider**: same escalated pattern; warns that all active SSO sessions will be invalidated

---

## Screen U1-8 — Policy Store / Config Management

**Traces to:** §11.2 (central config store + versioning/audit/rollback + GitOps apply); G-9; E-CENTRAL-OPS-001

### Purpose & Persona

Versioned configuration management for connector specs, retention policies, detection policy overrides, and RBAC policy definitions. Declarative/GitOps model.

**Persona:** Tenant-Admin; Detection-Engineer (retention policy authoring); Platform-Admin.

### Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  Config & Policies — Acme Corp    [Apply config ▾]  [History ▾] │
│  ─────────────────────────────────────────────────────────────── │
│  TABS: [Connectors] [Retention Policies] [RBAC Policies]         │
│                     [Detection Overrides]                        │
│  ─────────────────────────────────────────────────────────────── │
│  Current config version: v42 (applied 2026-06-25T14:00)          │
│  [View diff from v41]  [Rollback to v41]                         │
│                                                                   │
│  RETENTION POLICIES                                              │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │ default-security-sensors   7d TTL   All OCSF sensors         ││
│  │   Applied to: CrowdStrike, Cyberint, Claroty, Armis           ││
│  │   Tier: Hot (RocksDB) → Cold (Iceberg) after 24h            ││
│  │   [Edit] [Delete ⚠]                                          ││
│  ├──────────────────────────────────────────────────────────────┤│
│  │ long-term-ot               90d TTL  Claroty + Armis (OT)    ││
│  │   Tier: Cold (Iceberg)  Schema: OCSF network-activity        ││
│  │   [Edit] [Delete ⚠]                                          ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│  [+ Add retention policy]                                        │
└──────────────────────────────────────────────────────────────────┘
```

### GitOps Apply

The declarative config model: admin can apply a TOML config bundle via:
- **UI apply**: compose/edit policies in the UI; click [Apply]
- **CLI apply**: `prism config apply --file connector-specs.toml` (detected as equivalent)
- **Version history**: every applied config is versioned; diff viewer shows changes between versions; one-click rollback to any previous version

### Config Isolation (Multi-Tenant)

- Configs are per-`OrgId`; editing Acme Corp's policies cannot affect Beta Corp
- Platform-Admin can view all tenants' configs but edits are always scoped to the selected tenant
- Hot-reload: config changes apply via arc-swap per-tenant without affecting in-flight queries (§11.2)

---

## Open Design Questions for PO / Architect

The following questions should be resolved at day-2 morph time before finalizing the UX spec and dispatching E-UI-ADMIN-001 stories.

1. **Tenant hierarchy depth:** Does the MSSP use case require sub-tenants (e.g., client company → subsidiary)? The current spec is flat (Platform-Admin → Tenants → Users). Multi-level tenant hierarchies significantly complicate the permission model.

2. **User invite vs SSO-only:** When SSO is configured, should user accounts be pre-provisioned (admin creates the user, SSO provides the credentials) or JIT-provisioned (user logs in via SSO and account is created on first login)? JIT provisioning requires auto-role-assignment from SSO groups/claims; the current wizard spec shows group→role mapping but the JIT flow needs more design.

3. **Role + resource scope granularity:** The spec proposes resource-scoped ACLs (per-connector, per-dashboard). How fine-grained should this be for day-2? Implementing per-resource ACLs is a significant permission model complexity; it may be better to defer to day-3 and ship only role-scoped permissions in day-2.

4. **Credential rotation automation:** Should Prism support automated credential rotation on a schedule (e.g., "rotate this credential every 90 days automatically" with email notification to the admin)? The current spec is manual-rotation-only. Automated rotation requires integration with the secret backend's rotation mechanism (HashiCorp Vault supports this natively; built-in store would need its own rotation scheduler).

5. **Audit log retention and archival:** What is the required audit log retention period? MSSP/regulated environments may require 1–7 years. The current spec shows a "90 days before archival" note. Long-term audit retention requires a separate cold-storage strategy for audit logs specifically.

6. **SSO group synchronization:** Should group membership changes in the IdP (e.g., a user added to the "analyst" group in Okta) be reflected in Prism in real-time, on next login, or only when explicitly synced? Real-time sync requires SCIM provisioning (out of scope per current spec).

7. **MFA for the web UI:** The spec shows optional MFA for dangerous actions. Should the product also require MFA for all Platform-Admin logins (regardless of action)? This is a baseline security control that MSSP customers will expect.

8. **Config drift detection (§11.2 enhancement):** The spec references "drift detection" as a capability for the GitOps config model. Should the U1 UI surface config drift alerts ("Connector CrowdStrike spec in the platform does not match the last committed version in your Git repo")? This requires a Git integration that may be out of scope for day-2.

9. **Health alerting rules UI:** The current spec notes "configurable alerts on health thresholds" as a day-2 enhancement but does not fully design the alerting rule editor. Is this in-scope for the initial U1 delivery, or can health alerting route through the existing detection rule infrastructure (detect on health events)?

10. **Compliance posture / security page:** The matured vision (§16.3) notes adopting a security/privacy/compliance documentation page ("AI-opacity, residency, SOC2"). Should U1 include an in-product compliance posture screen (showing current data residency, audit log coverage, security settings summary) for MSSP customer conversations, or is this a documentation/marketing-site concern?
