---
document_type: design-sketch
status: capture
do_not_execute: true
provenance: >
  2026-06-26 side-analysis — day-2 design sketch; PROPOSED, gated on brief-reframe
  sign-off; separate from live factory. Extends proposed ADR-052 (central credential
  custody) — does NOT duplicate the live ADR registry. ADR-052 does not yet exist as a
  file in .factory/specs/architecture/decisions/; this sketch is the design input for it.
traces_to:
  - matured-vision-day2-requirements.md §11.1 (credential storage server-grade)
  - matured-vision-day2-requirements.md §3.2 (satellite/residency)
  - .factory/specs/day2-ui-design/U1-admin-console-inventory.md (Screen U1-4, credential rotation UX)
  - project memory AD-017 (AI-opaque credentials)
  - project memory OrgSlug/OrgId multi-tenant boundary
  - CLAUDE.md §Conventions (newtype + redacted Debug, OrgSlug::new_unchecked audit-gate)
human_decisions_required:
  - HD-1: Default KMS provider for the built-in encrypted store (see §6)
  - HD-2: Built-in store crypto primitives selection (AES-256-GCM vs ChaCha20-Poly1305; see §4.2)
  - HD-3: DEK rotation policy — manual-only vs automatic scheduled rotation (see §4.4)
  - HD-4: Per-tenant vs per-credential DEK granularity (see §4.1)
  - HD-5: Satellite credential custody model — full local store vs central-vend-then-cache (see §7)
---

# SS-26 Secret Broker — Day-2 Design Sketch

> PROPOSED design capture. Status: sketch. Not a live ADR. Not a live spec.
> Gated on brief-reframe sign-off before any story decomposition or implementation.
> All section numbering is internal to this document.

---

## 1. Problem Statement

The current reference-based credential model (AD-017) resolves credentials on the
analyst's machine via env vars / CLI flags / vault paths. A per-analyst stdio process
keeps this simple: credentials live where the analyst runs Prism.

The central deployment pivot (matured-vision §3.1) breaks this assumption. A multi-tenant
central service must hold and resolve credentials for many orgs — without an analyst
laptop in the loop. The access-layer gap is: credential storage, resolution, rotation, and
audit from a server process, scoped per `OrgId`, AI-opaque, rotation-without-restart.

This sketch designs the server-side replacement for analyst-local credential resolution.
The contract with the rest of the system stays identical: a `CredentialRef` (opaque
reference) goes in; a resolved secret value comes out at the I/O boundary. The machinery
underneath changes.

---

## 2. Scope and Non-Scope

**In scope for this sketch:**

- `SecretBackend` trait definition (Rust async trait, sketch-level signature)
- Built-in self-hosted encrypted store design (envelope encryption, per-tenant DEK)
- External backend integration surface (Vault, AWS SM, GCP SM, Azure KV)
- Per-tenant DEK key hierarchy and encrypt/decrypt/rotate flows
- Short-lived credential vending via OAuth client-credentials + refresh
- AI-opacity preservation at the I/O boundary
- Satellite-local resolution for data-residency topologies (§3.2)
- Rotation, audit, scrubbing, encrypted backups
- New subsystem SS-26 candidate and BC family needs

**Not in scope:**

- Full BC authoring (product-owner scope at day-2 morph time)
- Full ADR-052 text (architect scope; this sketch is the design input)
- Implementation stories or story decomposition
- RBAC / authN/authZ for the admin API (covered by ADR-051 + E-CENTRAL-AUTHZ-001)
- Config management subsystem (§11.2 is a separate sketch concern)

---

## 3. `SecretBackend` Trait — Proposed Rust Signature

The trait is async. Each method is scoped to a `(OrgId, CredentialRef)` pair so the
implementation enforces per-tenant isolation at the type level.

```rust
// crate: prism-secret-broker (new crate, effectful shell layer)
// All methods are async; the trait is object-safe via async-trait or RPITIT (edition 2024).

use std::time::Duration;

/// Opaque reference to a stored credential. Carries no secret value.
/// Newtype; redacted Debug impl per CLAUDE.md §Conventions.
#[derive(Clone, Debug)]   // Debug impl: SecretRef(***REDACTED***)
pub struct SecretRef(String);

/// Resolved secret value — lives only in memory, never serialized, never logged.
/// Zeroized on Drop via the `zeroize` crate.
#[derive(zeroize::ZeroizeOnDrop)]
pub struct ResolvedSecret(Vec<u8>);

/// Per-tenant DEK identifier (displayed in audit; never the key material).
#[derive(Clone, Debug)]
pub struct DekId(String);

/// Metadata visible without decryption (for audit/UI). No secret value.
#[derive(Debug)]
pub struct SecretMetadata {
    pub ref_id: SecretRef,
    pub org_id: OrgId,
    pub label: String,
    pub credential_type: CredentialType,
    pub backend: BackendKind,
    pub dek_id: Option<DekId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_rotated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CredentialType {
    ApiKey,
    BearerToken,
    OAuthClientCredentials { token_url: String },
    VaultRef { path: String },
    AwsSecretsManagerRef { arn: String },
    GcpSecretManagerRef { resource_name: String },
    AzureKeyVaultRef { vault_url: String, secret_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendKind {
    BuiltIn,
    HashiCorpVault,
    AwsSecretsManager,
    GcpSecretManager,
    AzureKeyVault,
}

/// Error type for all SecretBackend operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SecretBrokerError {
    #[error("credential not found: {ref_id:?}")]
    NotFound { ref_id: SecretRef },
    #[error("org not found or DEK not initialized: {org_id:?}")]
    OrgNotFound { org_id: OrgId },
    #[error("decryption failed (DEK error or key rotation mismatch)")]
    DecryptionFailure,
    #[error("external backend unavailable: {backend:?} — {source}")]
    BackendUnavailable { backend: BackendKind, source: String },
    #[error("rotation conflict: credential in use by active connection")]
    RotationConflict,
    #[error("audit write failed: {source}")]
    AuditFailure { source: String },
    #[error("permission denied: analyst {analyst_id} cannot access credential for org {org_id:?}")]
    PermissionDenied { analyst_id: String, org_id: OrgId },
}

/// Analyst identity for per-resolution audit (binds to ADR-051 per-connection identity).
pub struct AnalystContext {
    pub analyst_id: String,
    pub connection_id: uuid::Uuid,
    pub org_id: OrgId,
}

#[async_trait::async_trait]
pub trait SecretBackend: Send + Sync + 'static {
    /// Store a new credential. Returns a SecretRef (the reference, not the value).
    /// The secret value is consumed and cleared from the input by the implementor.
    async fn store(
        &self,
        org_id: &OrgId,
        label: &str,
        credential_type: CredentialType,
        secret_value: &mut ResolvedSecret,
        ctx: &AnalystContext,
    ) -> Result<SecretRef, SecretBrokerError>;

    /// Resolve a SecretRef to its current secret value.
    /// The returned value is zeroized on drop; callers must not log or serialize it.
    /// Every call is audited with the AnalystContext.
    async fn resolve(
        &self,
        secret_ref: &SecretRef,
        ctx: &AnalystContext,
    ) -> Result<ResolvedSecret, SecretBrokerError>;

    /// Rotate an existing credential in-place (replace value; keep SecretRef stable).
    /// Hot-reload semantics: in-flight adapters that hold the old token can optionally
    /// receive an invalidation signal (see §5.3).
    async fn rotate(
        &self,
        secret_ref: &SecretRef,
        new_secret_value: &mut ResolvedSecret,
        ctx: &AnalystContext,
    ) -> Result<(), SecretBrokerError>;

    /// Permanently delete a credential. Dangerous action — see U1-4 screen.
    async fn delete(
        &self,
        secret_ref: &SecretRef,
        ctx: &AnalystContext,
    ) -> Result<(), SecretBrokerError>;

    /// List metadata for all credentials in an org (no values).
    async fn list_metadata(
        &self,
        org_id: &OrgId,
        ctx: &AnalystContext,
    ) -> Result<Vec<SecretMetadata>, SecretBrokerError>;

    /// Health check for the backend (liveness + connectivity).
    async fn health(&self) -> BackendHealth;

    /// Initialize a per-tenant DEK for a new org. Idempotent.
    /// Built-in store: creates and wraps a new DEK under the KMS master key.
    /// External backends: validates that the org's vault path/ARN is accessible.
    async fn init_org_dek(
        &self,
        org_id: &OrgId,
        ctx: &AnalystContext,
    ) -> Result<DekId, SecretBrokerError>;

    /// Rotate the per-tenant DEK (re-encrypts all credentials under a new DEK).
    /// This is a bulk operation; it holds an exclusive org-level write lock.
    async fn rotate_org_dek(
        &self,
        org_id: &OrgId,
        ctx: &AnalystContext,
    ) -> Result<DekId, SecretBrokerError>;
}

#[derive(Debug)]
pub struct BackendHealth {
    pub backend: BackendKind,
    pub reachable: bool,
    pub latency_ms: Option<u64>,
    pub detail: Option<String>,
}
```

**Key design decisions in the trait:**

- `resolve()` audits on every call (not just first use) — every credential access is traceable to an analyst + connection.
- `rotate()` takes a `&mut ResolvedSecret` to ensure the caller's copy is consumed and zeroized as part of the call.
- `init_org_dek()` / `rotate_org_dek()` are on the trait to support satellite-local implementations that manage their own DEK lifecycle without calling central.
- `BackendKind` is `#[non_exhaustive]` — new backends can be added without a breaking change to match arms.

---

## 4. Built-In Encrypted Store Design

### 4.1 Key Hierarchy

The built-in store uses a three-level key hierarchy: KMS master key → per-tenant DEK → per-credential ciphertext.

```
                    ┌─────────────────────────────┐
                    │  KMS Master Key              │
                    │  (external: AWS KMS, GCP     │
                    │   KMS, Azure Key Vault, or   │
                    │   local HSM / PKCS#11)       │
                    │                              │
                    │  [HD-1: KMS provider choice] │
                    └──────────────┬──────────────┘
                                   │  wraps (envelope encrypt)
               ┌───────────────────┼───────────────────┐
               ▼                   ▼                   ▼
     ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
     │ DEK: org-acme    │ │ DEK: org-beta    │ │ DEK: org-gamma   │
     │ dek-0x1a...      │ │ dek-0x2b...      │ │ dek-0x3c...      │
     │ (per OrgId)      │ │ (per OrgId)      │ │ (per OrgId)      │
     └────────┬─────────┘ └────────┬─────────┘ └────────┬─────────┘
              │ encrypts                │ encrypts              │ encrypts
     ┌────────▼──────┐       ┌──────────▼──────┐    ┌──────────▼──────┐
     │ crowdstrike   │       │ splunk-token    │    │ armis-bearer    │
     │ ciphertext    │       │ ciphertext      │    │ ciphertext      │
     │ (AES-256-GCM) │       │ (AES-256-GCM)   │    │ (AES-256-GCM)   │
     └───────────────┘       └─────────────────┘    └─────────────────┘
```

**Per-tenant DEK isolation bound to OrgId:**

- Each org has exactly one active DEK (plus a previous DEK during rotation window).
- Compromise of one org's DEK cannot decrypt any other org's credentials — the
  cryptographic isolation mirrors the existing multi-tenant OrgId boundary.
- DEK granularity is per-tenant by default (HD-4: per-credential granularity is more
  complex, provides narrower blast radius but higher operational overhead — human decides).
- The wrapped DEK (ciphertext-of-DEK) is stored alongside the credential ciphertext in
  the credential store table. The KMS master key is never stored locally.

### 4.2 Encryption Primitives

**Recommended: AES-256-GCM** (industry standard for envelope encryption; AWS KMS, GCP
KMS, and HashiCorp Vault all use AES-256-GCM natively for their data-key operations).

Alternative: ChaCha20-Poly1305 (better on systems without AES hardware acceleration;
relevant for OT satellites on constrained hardware). HD-2: human selects one algorithm
for the built-in store; a per-backend cipher enum can support both if needed.

Rust crate candidates: `aes-gcm` (RustCrypto; pure Rust; FIPS-validated path via
`openssl`). Key derivation: never use raw random bytes as a DEK; derive via HKDF if
generating locally. For KMS-backed stores, the KMS generates the DEK and returns it
wrapped — no local key generation required.

### 4.3 Encrypt / Decrypt / Rotate Flow — Step by Step

**Encrypt (store new credential):**

```
1. Caller passes secret_value (plaintext bytes) + org_id to store().
2. Broker fetches the org's active DEK from the DEK cache (or KMS if cache miss).
   DEK cache: Arc<DashMap<OrgId, (DekId, DEK_plaintext, expiry)>>
   — DEK held in-process memory, not persisted plaintext, evicted after idle TTL.
3. Broker generates a fresh 96-bit random nonce (IV) for AES-256-GCM.
4. Broker encrypts:
     ciphertext = AES-256-GCM.encrypt(key=DEK, nonce=IV, plaintext=secret_value)
     tag is appended (16 bytes, GCM authentication tag).
5. Broker zeroizes the plaintext secret_value (ResolvedSecret::drop via zeroize).
6. Broker persists to the credential store:
     Row: { ref_id, org_id, dek_id (FK → DEK table), nonce, ciphertext+tag, metadata }
7. Broker writes audit log: { event: "credential.stored", org_id, ref_id,
     analyst_id, connection_id, timestamp } — value never logged.
8. Returns SecretRef (the ref_id, not the value).
```

**Decrypt (resolve credential):**

```
1. Caller passes SecretRef + AnalystContext to resolve().
2. Broker fetches credential row: { dek_id, nonce, ciphertext+tag }.
3. Broker fetches the DEK by dek_id:
   a. Check DEK cache (Arc<DashMap>).
   b. Cache miss: call KMS.decrypt(wrapped_dek) → plaintext DEK.
      Cache the DEK with a short TTL (e.g., 5 minutes) to limit KMS call rate.
4. Broker decrypts:
     plaintext = AES-256-GCM.decrypt(key=DEK, nonce=nonce, ciphertext+tag)
     Decryption failure → SecretBrokerError::DecryptionFailure (possible DEK rotation mismatch).
5. Wraps plaintext in ResolvedSecret (zeroize-on-drop).
6. Writes audit log: { event: "credential.resolved", org_id, ref_id,
     analyst_id, connection_id, timestamp } — plaintext never logged.
7. Returns ResolvedSecret to the caller (sensor adapter only; never to PrismQL, MCP, logs).
```

**Rotate credential in-place:**

```
1. Caller passes SecretRef + new_secret_value + AnalystContext to rotate().
2. Broker verifies caller has rotate permission for the org (AnalystContext).
3. Broker fetches the current ciphertext row.
4. Broker re-encrypts new_secret_value under the CURRENT active DEK:
     new_nonce = random 96 bits
     new_ciphertext = AES-256-GCM.encrypt(key=DEK, nonce=new_nonce, plaintext=new_secret_value)
5. Broker atomically overwrites the credential row (no gap window in the store).
6. Broker zeroizes new_secret_value.
7. Broker emits hot-reload signal to adapters holding the old token (see §5.3).
8. Writes audit log: { event: "credential.rotated", org_id, ref_id, analyst_id, timestamp }.
```

**Rotate DEK (org-level re-key):**

```
1. Platform-Admin triggers rotate_org_dek(org_id).
2. Broker acquires an exclusive org-level write lock (no new credential reads during re-key).
3. KMS generates a new DEK + new wrapped DEK:
     new_dek_plaintext, new_wrapped_dek = KMS.generate_data_key()
4. For each credential row in the org:
   a. Decrypt ciphertext under old DEK.
   b. Re-encrypt plaintext under new DEK with fresh nonce.
   c. Write new row (new dek_id, new nonce, new ciphertext).
   — All writes in a single DB transaction; atomicity is critical.
5. Deactivate old DEK record (keep for audit; delete wrapped DEK material after retention window).
6. Release write lock.
7. Update DEK cache: evict old entry, populate new.
8. Audit log: { event: "dek.rotated", org_id, old_dek_id, new_dek_id, row_count, timestamp }.
```

### 4.4 DEK Rotation Policy

Current recommendation: manual rotation via Platform-Admin (audit-triggered). Automatic
scheduled rotation adds operational complexity (must handle rotation-in-progress state,
concurrent resolution races). HD-3: human decides if automatic DEK rotation scheduler is
in-scope for day-2 or day-3.

---

## 5. External Backend Integration

External backends do not use the built-in DEK layer — they delegate all encryption to the
external service. The `SecretBackend` trait is the same; the implementation calls the
external vault API instead.

### 5.1 Backend Implementations

| Backend | Resolve method | Store/Rotate | Notes |
|---------|---------------|--------------|-------|
| HashiCorp Vault | `GET /v1/{path}` | `PUT /v1/{path}` | KV v2; versioned secrets; rotation via Vault |
| AWS Secrets Manager | `GetSecretValue` (API) | `PutSecretValue` | ARN-addressed; rotation via AWS Lambda hook |
| GCP Secret Manager | `AccessSecretVersion` | `AddSecretVersion` | Resource-name addressed; auto-rotation via Cloud Scheduler |
| Azure Key Vault | `GetSecret` | `SetSecret` | Vault URI + secret name; rotation via Key Vault rotation policy |

For external backends, `resolve()` fetches the *current* version from the vault at call
time (or from a short-lived local cache with TTL to avoid per-query vault round-trips).
`rotate()` for external backends writes to the vault; Prism does not own the ciphertext.

### 5.2 Per-Tenant Backend Configuration

Each org has a `BackendConfig` stored in the config store (§11.2 scope), specifying which
backend to use and any backend-specific connection parameters (vault address, AWS region,
ARN prefix, etc.). This is set by Tenant-Admin or Platform-Admin; it is not a secret
itself (it is metadata about where secrets live).

```toml
[org.acme-corp.secret_backend]
kind = "built-in"      # or "hashicorp-vault" | "aws-secrets-manager" | ...

[org.beta-corp.secret_backend]
kind = "hashicorp-vault"
vault_addr = "https://vault.beta.io:8200"
vault_path_prefix = "secret/prism/beta-corp"
# auth: Vault AppRole or Kubernetes auth; auth token stored in the built-in store
# for the MSSP's own Prism deployment — the vault token is itself a secret.
```

The auth token used to access an external vault is itself stored in the built-in store
for the MSSP's Prism instance. This avoids the bootstrap problem: the built-in store is
always present; external vault auth credentials are stored there.

### 5.3 Hot Reload on Rotation

When `rotate()` succeeds (either backend), the broker emits an internal `CredentialRotated`
event on a per-org tokio broadcast channel. Sensor adapters that hold a cached bearer
token subscribe to this channel and refresh their HTTP client credentials on receipt. This
is the DC-002 hot credential reload path (authorized in S-RESILIENCE, matured-vision §4
G2), generalized to both built-in and external backends.

```rust
// Broker broadcast channel per OrgId (tokio::sync::broadcast)
pub struct RotationEvent {
    pub org_id: OrgId,
    pub ref_id: SecretRef,
    pub rotated_at: chrono::DateTime<chrono::Utc>,
}

// Adapter side: on receiving RotationEvent matching its credential ref,
// re-calls broker.resolve() to get the new token and refreshes its reqwest::Client.
```

---

## 6. KMS Provider Selection — HD-1

The built-in store requires a KMS provider to generate and wrap DEKs. Options:

| Option | Fit | Pros | Cons |
|--------|-----|------|------|
| AWS KMS | Cloud deployment | Managed, FIPS 140-2 L3 HSMs, pay-per-use | AWS lock-in; not available in air-gap |
| GCP Cloud KMS | Cloud deployment | FIPS 140-2, global availability | GCP lock-in |
| HashiCorp Vault Transit (as KMS) | Hybrid | Cloud + on-prem; open source core | Operational burden; another service to run |
| Azure Key Vault | Cloud deployment | FIPS 140-2 L3 for Premium tier | Azure lock-in |
| AWS CloudHSM / on-prem HSM | Regulated/air-gap | Highest security; customer-owned hardware | High cost; specialized ops |
| Sodiumoxide / in-process AEAD without external KMS | Air-gap / local dev | Zero external dependency | KMS master key is stored locally; key protection is the operator's problem |

**Recommendation:** Ship with an abstracted `KmsProvider` trait (same pattern as
`SecretBackend`) with two initial implementations: (a) AWS KMS and (b) a local
`SoftwareKms` backed by a OS-keychain-protected or file-encrypted root key for
air-gap/dev deployments. GCP/Azure/Vault Transit follow as additional implementations.
The `SoftwareKms` path is the default for single-node / satellite / air-gap deployments
where no external KMS is available. HD-1 is whether to ship AWS KMS first or make
`SoftwareKms` the only day-2 implementation with the hook defined for external KMS.

---

## 7. Satellite-Local Resolution (Data Residency — §3.2)

At OT/edge enclaves and air-gapped satellites, credential resolution must happen locally.
The satellite holds its own instance of the `SecretBackend` (built-in store with a
local KMS, e.g., `SoftwareKms` backed by an HSM or OS keychain at the satellite site).

```
   Central Prism                         Satellite (OT enclave)
   ┌──────────────────┐                  ┌────────────────────────────────┐
   │ SecretBackend    │                  │ SecretBackend (satellite-local) │
   │ (built-in store) │                  │ (built-in store, local KMS)    │
   │ Org: MSSP-central│  enrollment      │ Org: acme-ot-zone-a            │
   │                  │ ───────────────▶ │ trust anchor from central      │
   │                  │                  │                                │
   │ query planner    │  subplan request  │ query executor                 │
   │                  │ ───────────────▶ │   resolve(crowdstrike-cred)    │
   │                  │                  │     → resolved locally         │
   │                  │                  │     inject into HTTP client    │
   │                  │  OCSF results    │   execute query against Claroty│
   │                  │ ◀────────────── │   return normalized rows       │
   └──────────────────┘                  └────────────────────────────────┘
```

**Key property:** the resolved secret value never leaves the satellite. Only normalized,
sanitized OCSF rows transit upward. The central Prism service never sees the raw
credential or the raw sensor response.

HD-5: The satellite credential custody model has two variants:
- (a) **Full local store:** satellite provisions its own `SecretBackend` independently;
  Platform-Admin manages satellite credentials via a satellite-local admin channel.
  Simpler; best for strict air-gap. No credential sync to central.
- (b) **Central-vend-then-cache:** central issues short-lived credential tokens to the
  satellite at enrollment; satellite caches them encrypted under its local KMS.
  More complex; requires an enrollment protocol with expiry and renewal.

The matured-vision §3.2 enrollment protocol is not yet designed; this sketch recommends
(a) for day-2 simplicity, with (b) as a day-3 enhancement once the enrollment protocol
is settled.

---

## 8. AI-Opacity Invariant (AD-017 Hardened)

The AI-opacity invariant must be enforced at the architectural boundary, not by convention:

**Rule:** `ResolvedSecret` must never be assigned to a variable that flows into:
- PrismQL query results (DataFusion output batches)
- MCP tool response JSON
- `tracing::*!` log macros
- Agent context / LLM prompt strings

**Enforcement mechanisms (proposed):**

1. `ResolvedSecret` is a newtype with no `Serialize`, no `Display`, no `Debug` (only
   redacted Debug: `ResolvedSecret(***REDACTED***)`), and no `Clone`. It is opaque by
   construction.
2. The only consumer of `ResolvedSecret` is the sensor adapter's HTTP client builder:
   `reqwest::Client::builder().bearer_auth(secret.expose_secret())` — where
   `expose_secret()` is a narrow method that converts to `&str` for the HTTP client only,
   analogous to `secrecy::ExposeSecret`.
3. Adapter code must not store the `&str` beyond the `reqwest::RequestBuilder` call.
4. A future compile-fail gate (analogous to `tests/external/perimeter-violation/`) can
   enforce that `expose_secret()` is called only within the sensor adapter crates.
5. MCP tool output and PrismQL result rows contain only OCSF-normalized data; the broker
   ensures the raw credential never touches these paths.

This extends the existing AD-017 contract to the server path and is a genuine trust
differentiator over Query.AI (which stores credentials centrally but does not publish
an equivalent opacity contract).

---

## 9. Rotation, Audit, Scrubbing, Encrypted Backups

### 9.1 Rotation

- Built-in store: `rotate()` re-encrypts under the current active DEK in one atomic
  write (§4.3). No value gap between old and new (atomic overwrite, not delete+insert).
- External backends: `rotate()` writes new value to the vault; Prism does not own the
  ciphertext.
- Hot reload signal via broadcast channel (§5.3) on every rotation.
- Rotation overdue indicator: broker tracks `last_rotated_at` in metadata; exposes it via
  `list_metadata()`. The admin UI (U1-4) surfaces the "rotation overdue" warning at
  >90 days (configurable per-org).

### 9.2 Audit

Every `store()`, `resolve()`, `rotate()`, and `delete()` call writes an immutable audit
record binding `AnalystContext` (analyst_id + connection_id from ADR-051) to the
credential ref_id and timestamp. The audit log:
- Is append-only in the storage layer (no update/delete path in the credential audit table).
- Never contains the resolved value — only the reference (SecretRef) and the operation.
- Feeds into U1-5 (Audit Log Viewer) as `action: rotate-credential / resolve-credential`.
- Per-connection analyst identity (ADR-051) is the attribution anchor; every resolution
  is attributed to a specific analyst session.

### 9.3 Scrubbing

- `ResolvedSecret` implements `ZeroizeOnDrop` (via the `zeroize` crate). When the
  value goes out of scope, the memory is overwritten with zeros before deallocation.
- The DEK plaintext in the cache is also wrapped in a `SecretKey<[u8; 32]>` (secrecy
  crate) with zeroize-on-drop semantics.
- Log scrubbing: the existing `redacted Debug` discipline extends to all types in this
  subsystem. No secret value appears in structured log fields.

### 9.4 Encrypted Backups

- The credential store (ciphertext blobs + wrapped DEKs) can be backed up as-is —
  the ciphertext is already encrypted under DEKs that are themselves wrapped by the
  KMS master key. A backup without the KMS master key is useless to an attacker.
- Backup procedure: snapshot the credential store rows + wrapped DEK records; the
  backup is encrypted at rest by construction.
- DEK rotation (§4.3) ensures backups encrypted under old DEKs cannot be used with
  the current KMS master key unless the old wrapped DEK record is retained (and it
  must be, during the DEK rotation retention window).

---

## 10. Short-Lived Credential Vending (OAuth Path)

For sources that support OAuth client-credentials flow:

```
Broker                           Authorization Server (e.g., CrowdStrike)
  │                                        │
  │  stored: client_id + client_secret     │
  │  (under per-tenant DEK)                │
  │                                        │
  │  resolve() called by adapter           │
  │  ──── POST /oauth2/token ──────────────▶
  │       client_id=..., client_secret=... │
  │  ◀─── { access_token, expires_in } ────│
  │                                        │
  │  return access_token as ResolvedSecret │
  │  (access_token is short-lived; expires │
  │  in 30-60 min; broker caches it until  │
  │  expiry - 60s, then re-fetches)        │
```

**What is stored (long-term):** client_id (non-secret, stored as metadata) +
client_secret (encrypted under DEK). The short-lived access token is NOT stored;
it is fetched on demand and cached in-process with a TTL.

**What is stored for static-token sensors (Armis/Claroty):** the API key or bearer
token itself, encrypted under DEK. Rotation requires the admin to obtain a new token
from the sensor vendor and call `rotate()`. The hot-reload path (DC-002) fires on
`rotate()` so adapters pick up the new token without restart.

---

## 11. New Subsystem Candidate: SS-26 Secret Broker

**Proposed subsystem:** `SS-26 Secret Broker`

**Scope:** Stores, resolves, rotates, and audits credentials for all sensor adapters.
Enforces per-tenant DEK isolation. Injects resolved secrets at the I/O boundary only.
Never exposes values to PrismQL, MCP, logs, or agent context.

**Candidate BC family (to be authored by product-owner at day-2 morph time):**

| BC ID (placeholder) | Title | Key postcondition |
|---------------------|-------|-------------------|
| BC-11.1.001 | Credential resolution | resolve() returns correct plaintext; audits every call with analyst identity |
| BC-11.1.002 | Per-tenant DEK isolation | Credentials for org-A cannot be decrypted under org-B's DEK |
| BC-11.1.003 | Credential rotation | rotate() atomically replaces value; hot-reload signal fires; no gap window |
| BC-11.1.004 | AI-opacity invariant | ResolvedSecret never appears in PrismQL results, MCP output, or logs |
| BC-11.1.005 | DEK initialization | init_org_dek() is idempotent; a new org always gets an isolated DEK before first credential storage |
| BC-11.1.006 | Audit completeness | Every store/resolve/rotate/delete call produces an immutable audit record with analyst identity |
| BC-11.1.007 | Satellite-local resolution | At satellite deployments, resolve() completes locally; resolved value never transits to central |

**Module classification (proposed):**

- `prism-secret-broker` crate: CRITICAL tier (resolution and AI-opacity are
  security-invariant; DEK failure = total credential loss for affected org).
- `prism-secret-broker::kms` submodule: CRITICAL (key hierarchy correctness is a
  formal-verification target; property VP-???: "DEK decryption with wrong key always
  returns DecryptionFailure, never a silently incorrect plaintext").

---

## 12. Relationship to Proposed ADR-052

This sketch is the design input for ADR-052 (Central credential custody design). ADR-052
does not yet exist as a file. When day-2 morph proceeds:

1. The architect produces ADR-052 from this sketch, with the human decisions (HD-1 through
   HD-5) resolved.
2. The `SecretBackend` trait sketch becomes the formal Rust API in ADR-052.
3. The BC family placeholders in §11 above become live BCs authored by the product-owner.
4. SS-26 is added to ARCH-INDEX.md Subsystem Registry.
5. This sketch document remains as a design artifact but is superseded by ADR-052 for
   all binding decisions.

ADR-052 supersedes ADR-032 (per-client credential env-var convention) for the central
deployment path. The env-var convention remains valid for the per-analyst stdio path.

---

## 13. Open Decisions for Human

| ID | Decision | Context | Options |
|----|----------|---------|---------|
| HD-1 | Default KMS provider for built-in encrypted store | KMS wraps per-tenant DEKs. Must support air-gap/on-prem deployments. | (a) AWS KMS + SoftwareKms fallback; (b) SoftwareKms only for day-2 (hardware/cloud KMS as day-3 enhancement); (c) HashiCorp Vault Transit as the one KMS abstraction |
| HD-2 | Built-in store cipher: AES-256-GCM vs ChaCha20-Poly1305 | AES-256-GCM is the industry standard for envelope encryption and aligns with all major KMS providers. ChaCha20-Poly1305 is preferable on OT satellites without AES hardware acceleration. | (a) AES-256-GCM only; (b) AES-256-GCM default + ChaCha20-Poly1305 for satellite via per-backend config |
| HD-3 | DEK rotation — manual-only vs automatic scheduled | Automatic DEK rotation requires a scheduler, in-progress state management, and concurrent-read safety during re-key. Manual provides audit control. | (a) Manual-only for day-2, automatic scheduler as day-3; (b) Automatic from day-2 with configurable schedule per org |
| HD-4 | DEK granularity: per-tenant vs per-credential | Per-tenant DEK is simpler (one key per org, all credentials share it). Per-credential DEK narrows blast radius (one compromised credential's DEK does not expose others) but multiplies KMS operations and key storage. | (a) Per-tenant DEK (recommended for day-2 simplicity); (b) Per-credential DEK (stronger isolation, higher operational cost) |
| HD-5 | Satellite credential custody model | Full-local-store (simplest, best for air-gap) vs central-vend-then-cache (more integrated, requires enrollment protocol). | (a) Full local store for day-2; central-vend-then-cache as satellite v2; (b) Central-vend-then-cache from day-2 if enrollment protocol is designed concurrently |
