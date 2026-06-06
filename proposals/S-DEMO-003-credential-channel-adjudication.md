---
document_type: architect-proposal
title: "S-DEMO-003 Credential Channel Adjudication"
author: architect
date: "2026-06-06"
status: HUMAN-DECISION-RECORDED
version: "1.1"
story_id: S-DEMO-003
blocking_cascade: false
decision: "OPTION-A — implement Tier-3 keyring resolution (human approved 2026-06-06)"
adr: "ADR-034-tier3-keyring-resolution-org-id-threading.md"
---

# S-DEMO-003 — Credential Channel Adjudication

Produced by architect agent for orchestrator + human decision gate.
No code or spec files were modified.

---

## 1. Gap Confirmation with Evidence

### CRIT-1: Write channel (keyring) is permanently disconnected from read channel (resolve_credential)

The gap is confirmed and structurally complete. The evidence chain:

**Write side — `CredentialStore::set` via `KeyringBackend`**
- `keyring.rs:32–34`: `fn namespace_key(tenant: &OrgSlug, sensor: &str, name: &CredentialName)` produces `"{slug}/{sensor}/{name}"`, e.g. `"demo-org/crowdstrike/client_id"`.
- This is the legacy `CredentialStore` trait path (distinct from `CredentialStoreOrgId`).
- `KeyringBackend::set` writes the secret under this key to the OS keyring.
- The story spec (S-DEMO-003 §Architecture Compliance Rules) specifies that `demo-setup.sh` should call `CredentialStore::set(org_id, sensor_id, name, value)` via `prism-credentials`. At implementation time this routes to `KeyringBackend::set` → `namespace_key(OrgSlug, ...)` → key = `"demo-org/crowdstrike/client_id"`.

**Read side — `resolve_credential`**
- `resolution.rs:111`: Tier 1 + Tier 2 env var chain is attempted first.
- `resolution.rs:173`: Falls through to Tier 4: `crate::crud::credential_status(client_id, sensor_id, credential_name).await`. This reads the in-memory `CREDENTIAL_STORE` thread-local map (`crud.rs:121`). This map is populated ONLY by `configure_credential_source()` — never by `KeyringBackend::set`.
- Tier 3 keyring read: `resolution.rs:18,92` explicitly documents "not implemented here, delegated to CRUD store lookup." No `CredentialStoreOrgId::get_by_org` call exists anywhere in `resolve_credential`.

**NET**: Writing via `CredentialStore::set` → keyring at key `"demo-org/crowdstrike/client_id"` is permanently invisible to `resolve_credential`. There is no Tier 3 branch in the resolver. AC-002 (demo works end-to-end) cannot pass as the story is currently specified.

### CRIT-2: Namespace mismatch between legacy `CredentialStore::set` and `CredentialStoreOrgId::get_by_org`

Even if Tier 3 were implemented in `resolve_credential`, it would call `CredentialStoreOrgId::get_by_org`:
- `keyring.rs:259`: `get_by_org` uses `namespace_key_by_org_id(org_id, ...)` → key = `"{org_id_uuid}/{sensor}/{name}"`.
- But `KeyringBackend::set` uses `namespace_key(OrgSlug, ...)` → key = `"{org_slug}/{sensor}/{name}"`.

These are different keyring entries. A write via the legacy `CredentialStore::set` path (`"{slug}/..."`) will NEVER be found by `CredentialStoreOrgId::get_by_org` (which reads `"{uuid}/..."`). `namespace.rs:12–13` makes this explicit: "The legacy slug-keyed format has been removed from this module."

### Does a `prism credential set` write register a CRUD `credential_status` row?

No. `KeyringBackend::set` (`keyring.rs:100–139`) writes directly to the OS keyring and updates only the sidecar `CredentialIndex`. It does NOT call `configure_credential_source` and does NOT write to the thread-local `CREDENTIAL_STORE` in `crud.rs`. The CRUD store and the keyring are entirely separate subsystems. A write to one has no effect on the other.

**Therefore**: Tier 4 CRUD lookup (resolution.rs:173) after a `prism credential set` keyring write will always return `Ok(None)` → `CredentialResolutionError::NotFound`. Every demo query fails with E-AUTH-005.

---

## 2. What Was the Canonical Intended Demo Credential Channel?

Reading ADR-032 and BC-2.06.003 v1.3 (both dated 2026-06-03, authored during S-DEMO-002):

**ADR-032 §Decision / Resolution tier order:**
1. Tier 1: `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}_FILE`
2. Tier 2: `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}`
3. Tier 3: OS keyring via `CredentialStoreOrgId::get_by_org` (OrgId-UUID key)
4. Tier 4: CRUD store

**ADR-032 §Boot-step-5 org-aware probe:** explicitly defines Tier 1/2 wildcard scan across all registered orgs as the primary probe mechanism. The error message template in BC-2.06.003 §Error Cases cites the per-client Tier 2 env var format as the primary remediation path: `Set PRISM_CLIENTS_<ORG_SLUG_UPPER>_SENSORS_...`.

**BC-2.06.003 §Canonical Test Vectors:** "Tier 2 direct | `acme` | `armis` | `bearer_token` | `PRISM_CLIENTS_ACME_SENSORS_ARMIS_BEARER_TOKEN=abc123`" — env vars are the worked examples for demo/test scenarios, not keyring.

**BC-2.06.003 §Tier 3 note:** The boot probe uses "legacy OrgSlug-keyed format `{org_slug}/{sensor_id}/{ref_name}`" for backwards compatibility. This is explicitly marked as temporary ("Future stories may migrate").

**Conclusion**: Tier 2 env vars (`PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}`) are the canonically intended credential channel for demos, tests, and single-operator deployments. Tier 3 keyring was SPECIFIED in the architecture but was never fully implemented in the resolver — the spec listed it but the ADR itself called it a backwards-compat fallback. Tier 3 keyring resolution in `resolve_credential` is a gap introduced during S-DEMO-002 implementation (the resolver was rewritten to the new per-client format but Tier 3 was deferred as "delegated to CRUD store" — which effectively killed it).

The `prism credential set` subcommand as specified in S-DEMO-003 surfaces the OLD `CredentialStore::set` path, which predates ADR-032 and does not connect to ANY currently active resolution tier.

---

## 3. Fix Options — Scope, Effort, Risk, Spec Impact

### Option A: Implement Tier-3 keyring read in `resolve_credential` + reconcile CRIT-2 namespace

**What it does:** Add a Tier 3 branch to `resolve_credential` that calls `CredentialStoreOrgId::get_by_org(org_id, sensor_id, credential_name)`. This requires threading `OrgId` into `resolve_credential` (currently it only takes org slug). Also requires `prism credential set` to use `CredentialStoreOrgId::set_by_org` (OrgId-keyed, not legacy slug) so the write and read namespaces align.

**Scope:**
- `resolution.rs`: Add Tier 3 branch between env resolution and CRUD lookup. Thread `OrgId` or `Arc<dyn CredentialStoreOrgId>` into `resolve_credential` — currently its signature is `(client_id: &str, sensor_id: &str, credential_name: &str)`. Requires an `OrgRegistry` lookup to convert `client_id` (slug) → `OrgId`.
- `credential_cli.rs`: Must call `CredentialStoreOrgId::set_by_org` not `CredentialStore::set`. Requires loading `prism.toml` to map the provided org slug to `OrgId`.
- BC-2.06.003: Story spec S-DEMO-003 AC-005 claims the subcommand writes "per BC-2.03.004" — but BC-2.03.004 uses the OrgId key. The claim is ALMOST correct but the implementation path in the story spec points to the wrong trait.
- All callers of `resolve_credential` must be updated to supply an `OrgId` or `Arc<KeyringBackend>`.

**Effort:** MEDIUM-HIGH. Signature change to `resolve_credential` is a sibling-site blast radius change (TD-VSDD-060). The OrgRegistry dependency must be threaded through to every resolution callsite. Not trivial.

**Risk:** `resolve_credential` is a foundational function called in every sensor fetch. Changing its signature and adding I/O (keyring call) to the hot path requires careful testing. The keyring Tier 3 is asynchronous (requires `spawn_blocking`) and adds latency.

**Spec amendments required:**
- S-DEMO-003 AC-005: Update namespace claim to specify OrgId-keyed write via `CredentialStoreOrgId::set_by_org`.
- S-DEMO-003 §Architecture Compliance Rules: Correct "Use `CredentialStore::set(org_id, sensor_id, name, value)`" to `CredentialStoreOrgId::set_by_org`.
- BC-2.06.003: Document Tier 3 as now IMPLEMENTED (the spec already specifies it; the gap is implementation, not spec).
- ADR for the `resolve_credential` signature change (threading `OrgId` or `CredentialStoreOrgId`).

**Assessment:** This is the architecturally complete path — it delivers what BC-2.06.003 Tier 3 specifies. It is NOT a "wiring not redesign" fix; it is a genuine capability gap fill (Tier 3 was always spec'd but was never implemented). It requires human sign-off as a scope expansion. Implementation complexity is non-trivial and deserves its own story or explicit expansion of S-DEMO-003 scope.

---

### Option B: Re-scope the demo to the env-var channel (Tier 2) — `demo-setup.sh` exports `PRISM_CLIENTS_*` vars

**What it does:** `demo-setup.sh` does NOT call `prism credential set` for the boot/query path. Instead, it exports the `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` env vars directly (or writes them to a sourced `.env` file). The runtime already reads Tier 2 env vars — no code change required. `prism credential set` is either (a) removed from the demo scope, or (b) repurposed as a convenience tool whose persistence caveat is documented: "writes to OS keyring but env var channel is used for query-time resolution."

**Scope:**
- `scripts/demo-setup.sh`: Replace keyring-write step 7 with env var export or `.env` file generation. `demo-run.sh` sources the `.env` before launching `prism start`.
- S-DEMO-003 AC-005: Repurposed. If `prism credential set` is kept as a feature, its AC must honestly state its limitation: "writes to OS keyring for future use; runtime query-time resolution uses env vars (Tier 2) for the demo scenario." Alternatively AC-005 is replaced with an AC for the `.env` file generation step.
- BC-2.06.003 / ADR-032: No amendment required. The env-var channel is already the canonical spec'd path.
- BC-2.03.005: `configure_credential_source` (CRUD MCP path) is still available for programmatic use. No conflict.
- `prism credential set`: If the subcommand is kept, its scope is reduced: it's a convenience writer for future Tier 3 use, not the demo bootstrap path.

**Effort:** LOW. The env var channel already works. This is script-level change + story AC amendment.

**Risk:** LOW. No changes to production Rust code. Env var channel is exercised by S-DEMO-002 tests — it is already proven working.

**Spec amendments required:**
- S-DEMO-003 AC-005: Reframe `prism credential set` as "convenience CLI for future keyring Tier 3 use" with explicit caveat that demo flow uses env vars. OR remove AC-005 entirely and replace with "demo-setup.sh generates `.env` file with per-client Tier 2 env vars."
- S-DEMO-003 §Architecture Compliance Rules: Replace keyring-write table row with env-var file generation row.
- S-DEMO-003 Tasks: Replace step 7 (keyring write) with env var generation step.
- Optional: Remove `rpassword` from Library & Framework Requirements if `prism credential set` is descoped.

**Assessment:** Lowest risk, fastest to ship. Consistent with what ADR-032 and BC-2.06.003 specify as the CANONICAL demo channel. The env-var channel was the architect's intended demo path — the `prism credential set` subcommand in S-DEMO-003 was an addition by story-writer that does not align with the resolution chain. Option B is the production-grade choice for this cycle: it ships a working demo end-to-end. Option A (full Tier 3) is the production-grade choice for Tier 3 as an eventual feature — it belongs in its own story with its own spec.

---

### Option C: `prism credential set` writes through the CRUD store (Tier 4) instead of keyring

**What it does:** `handle_credential_set()` calls `configure_credential_source(ConfigureCredentialRequest { ..., source: CredentialRef { kind: CredentialRefKind::Env, reference: env_var_name } })`. The CLI writes a CRUD metadata row that tells Tier 4 "this credential lives at env var `PRISM_CLIENTS_DEMO_ORG_SENSORS_CROWDSTRIKE_CLIENT_ID`." The user still provides the value (via rpassword stdin read) but the value is stored... where? The CRUD store holds METADATA (source references), not values.

**Structural problem:** BC-2.03.005 §Postconditions is explicit: "`configure_credential_source` accepts source type references only (`env`, `file`, `vault`, `keyring`) — never raw credential values." The CRUD store is a METADATA store. It cannot hold a secret value. `prism credential set` is supposed to write the secret value, not a pointer to where it lives. If the CLI only writes an "env" metadata row to CRUD, the value still needs to be put somewhere that Tier 1/2 (env var) or Tier 3 (keyring) can find it.

Option C therefore requires EITHER: (a) the CLI writes the value to an env var (which is not persistent across processes) OR (b) the CLI writes the value to the keyring AND registers a `Keyring` CRUD row. Path (b) converges on Option A (Tier 3 must be readable) plus a CRUD registration step. The CRUD "keyring" backend in `resolve_from_backend` (`resolution.rs:351–399`) is not implemented — only "env" is.

**Assessment:** Option C is architecturally incoherent for a value-writing subcommand. The CRUD store is a source-reference catalog, not a secret store. Option C dissolves into Option A (if keyring) or Option B (if env var). It is not a viable standalone fix.

---

## 4. Recommendation

**Option B is the recommended fix for S-DEMO-003.**

**Rationale:**

1. **It matches the canonical architecture.** ADR-032 and BC-2.06.003 v1.3 designate Tier 2 env vars (`PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}`) as the standard credential channel for multi-tenant deployments. The demo is a single-tenant use case. Env vars are exactly what the architect specified for this scenario.

2. **It is "wiring not redesign" at the demo layer.** The resolution chain already reads Tier 2. No production Rust code changes. The fix is: generate the correct env vars in `demo-setup.sh`, source them in `demo-run.sh`, and update AC-005 to reflect this.

3. **Option A is real-feature scope.** Implementing Tier 3 in `resolve_credential` requires threading `OrgId` through to the hot query path, adding an async keyring call to every credential resolution, and handling the slug→OrgId lookup at resolution time. This is a non-trivial architectural change that deserves its own story (`S-CRED-TIER3-KEYRING-RESOLUTION` or similar), its own BC amendment, and its own adversarial pass. Bundling it into S-DEMO-003 violates the story's stated scope (5 points, 1 day, LOW risk) and the production-grade default's prohibition on scope expansion as a casual fix.

4. **`prism credential set` can survive descoped.** The subcommand is still useful as a persistence tool for operators who want to pre-load credentials into the OS keyring for a future Tier 3 implementation. It can ship in S-DEMO-003 with an honest caveat in AC-005 and the runbook: "credentials written here are stored in the OS keyring and will be read by the Tier 3 resolver when S-CRED-TIER3-KEYRING-RESOLUTION is delivered; the demo bootstrap uses Tier 2 env vars." This is NOT a defer-pattern — it is accurate documentation of a partial capability that is genuinely useful now.

5. **Demo AC-002 (end-to-end) is achievable this cycle with Option B.** With Option A, AC-002 requires a blast-radius refactor that may exceed the story's capacity.

**Is this a "wiring not redesign" fix?** Yes, at the demo layer. The env-var channel is fully wired in production code. The fix is in shell scripts and story AC reframing. The `resolve_credential` hot path is unchanged.

**Does it require human sign-off?** Yes — because it changes the UX contract for `prism credential set`. AC-005 was spec'd as "bootstraps demo credentials for end-to-end use." Under Option B, the demo bootstrap path changes from keyring to env vars. The human must confirm: (a) is `prism credential set` kept in this story with a reduced scope, or (b) is it removed from S-DEMO-003 entirely and deferred to the Tier 3 story?

**Proposed future story:** `S-CRED-TIER3-KEYRING-RESOLUTION` — "Implement Tier 3 keyring read in `resolve_credential`; thread OrgId through resolution chain; align `prism credential set` with OrgId-keyed write path; make `prism credential set` the end-to-end credential bootstrap for operator deployments." This makes `prism credential set` the intended UX it was designed to be — just on the correct cycle.

---

## 5. Secondary Findings for Orchestrator Routing

These are confirmed HIGH findings independent of the CRIT-1 architectural gap. They require fix-burst dispatch to the implementer but are not architectural decisions.

### HIGH-2: Shellcheck in Justfile only, not in GitHub CI (AC-008 gap)

**Evidence:** S-DEMO-003 AC-008: "When `shellcheck scripts/demo-*.sh` is executed in CI." The story risk mitigations state "Add shellcheck to the Justfile `check-ci` recipe or CI matrix if not already present." CI means GitHub Actions CI (`.github/workflows/ci.yml`), not just local `just check-ci`. If `just check-ci` is not invoked from the GitHub CI matrix, AC-008 is unmet.

**Required check by implementer:** Does `.github/workflows/ci.yml` call `just check-ci` (or `shellcheck` directly) as a step? If not, the CI job must be added. **Route to: implementer** (code change to CI YAML + Justfile).

### HIGH-3: `resolve_org_slug` swallows `prism.toml` read errors (from adversary finding)

**Note on evidence basis:** `credential_cli.rs` does not yet exist (S-DEMO-003 is unimplemented). The adversary's finding anticipates that the story implementer will write a `resolve_org_slug` helper that reads `prism.toml` and defaults to `"demo-org"` when the file is missing or invalid. This pattern — swallowing read errors and returning a silent default — is a SOUL.md §4 violation (don't swallow errors) and an ADR-022 §wiring violation (boot context is the authoritative source of org data, not a re-read of prism.toml from credential_cli.rs).

**Correct implementation discipline (SID-1 preemptive):** `credential_cli.rs` MUST NOT re-read `prism.toml` independently from `BootContext`. The correct approach: load the full `PrismConfig` via the existing boot-step-2 path, extract the org list, require `--org-slug` when multiple orgs are present, and error clearly when prism.toml is missing/invalid. Silent defaults are forbidden. **Route to: implementer** (spec-level discipline to enforce during TDD, no spec amendment needed).

### HIGH-1: Runbook documents retired `DEMO_ORG_*` env format

**Evidence:** ADR-032 §Alternatives Considered: "The retired global `{SENSOR}_{REF}` format (e.g. `ARMIS_BEARER_TOKEN`) is NOT used." BC-2.06.003 v1.3 §Description: "The v1.2 claim ... was false — the prior code used a global `{SENSOR}_{REF}` format that is not client-scoped." Any runbook or demo script that references `DEMO_ORG_ARMIS_BEARER_TOKEN` or similar global format is citing the retired convention and will not work with the production resolver.

**Correct format:** `PRISM_CLIENTS_DEMO_ORG_SENSORS_ARMIS_BEARER_TOKEN` (for org slug `demo-org`). The runbook and `demo-setup.sh` MUST use only the `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` format. **Route to: implementer** (the scripts and runbook have not been written yet; this is a preemptive discipline note for the implementer to enforce during TDD).

---

## Spec Amendments Required (per Recommendation: Option B)

| Document | Change | Owner |
|----------|--------|-------|
| S-DEMO-003 AC-005 | Reframe: `prism credential set` writes to OS keyring for FUTURE Tier 3 use; demo bootstrap uses Tier 2 env vars via `demo-setup.sh`. Remove claim that keyring write enables end-to-end demo. | product-owner |
| S-DEMO-003 §Architecture Compliance Rules | Replace "Use `CredentialStore::set`..." row with "demo-setup.sh generates Tier 2 env vars `PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}` for each org/sensor/credential triple; `demo-run.sh` sources them before launching `prism start`" | product-owner |
| S-DEMO-003 Tasks step 7 | Replace "write credentials to keyring" with "generate `.env` file with per-client Tier 2 env vars; source in demo-run.sh" | product-owner |
| BC-2.06.003 | No amendment needed — Tier 2 env var channel is already the canonical path. The Tier 3 gap is an existing known gap documented in the spec. | — |
| ADR-032 | No amendment needed. | — |
| (Future) New story | `S-CRED-TIER3-KEYRING-RESOLUTION`: Implement Tier 3 keyring read in `resolve_credential`; align `prism credential set` with OrgId-keyed write path. | story-writer (future cycle) |

**Human decision required:**
1. Approve Option B as the fix approach for S-DEMO-003.
2. Decide: keep `prism credential set` in S-DEMO-003 scope (with reduced AC-005), or defer the entire subcommand to the Tier 3 story?
3. Confirm: create `S-CRED-TIER3-KEYRING-RESOLUTION` as a backlog story (LOW urgency — demo ships on env vars; Tier 3 is an operator-UX enhancement, not a correctness blocker for the demo).

---

## 6. Human Decision Recorded (2026-06-06)

**Decision: OPTION A — implement Tier-3 OS-keyring credential resolution NOW in S-DEMO-003.**

The human approved expanding S-DEMO-003 scope to implement Tier-3 fully so that `prism credential set` becomes load-bearing end-to-end. The `prism credential set` subcommand STAYS in S-DEMO-003.

**ADR authored:** `ADR-034-tier3-keyring-resolution-org-id-threading.md` in `.factory/specs/architecture/decisions/`.

The Option-B amendments in §5 above (Spec Amendments Required per Recommendation: Option B) are SUPERSEDED and do NOT apply. The correct spec amendments are those defined in ADR-034 §D6.

### Chosen Design Summary (from ADR-034)

**Resolver signature (ADR-034 §D1 + D2):**
```rust
pub async fn resolve_credential(
    client_id: &str,                                       // org slug (Tier 1/2 env-var derivation)
    sensor_id: &str,
    credential_name: &str,
    org_id: Option<&OrgId>,                                // pre-resolved slug→OrgId (by PrismCredentialResolver)
    keyring: Option<&Arc<dyn CredentialStoreOrgId>>,       // None → skip Tier 3
) -> Result<SecretString, CredentialResolutionError>
```

`client_id` (org slug) continues to drive Tier 1/2 env-var name derivation. `org_id` (pre-resolved OrgId) drives Tier 3 keyring lookup. Slug→OrgId resolution happens in `PrismCredentialResolver::resolve` in `prism-spec-engine` (which may import `OrgRegistry`), NOT inside `prism-credentials` (architecture compliance rule: `prism-credentials` must not import `OrgRegistry`). When both `org_id` and `keyring` are `Some`, Tier 3 is active between the env-var tiers and Tier 4.

**`PrismCredentialResolver` becomes a struct (ADR-034 §D2):**
```rust
pub struct PrismCredentialResolver {
    org_registry: Arc<OrgRegistry>,
    keyring: Arc<dyn CredentialStoreOrgId>,
}
```

`PrismCredentialResolver::new(org_registry, keyring)` is the production construction path. The unit-struct form is removed. All 5 test doubles (`MockCredentialResolver`, `NotFoundCredentialResolver`, `BackendUnavailableCredentialResolver`, plus any others) gain the `org_registry` parameter to `CredentialResolver::resolve` and ignore it.

**Namespace reconciliation (ADR-034 §D3):**
`credential_cli.rs handle_credential_set` writes via `CredentialStoreOrgId::set_by_org` using the `org_id` UUID read from `PrismConfig.orgs[n].org_id` in `prism.toml`. The legacy `CredentialStore::set` path is replaced entirely. The `resolve_org_slug` demo-org fallback is removed (SOUL.md §4 — HIGH-3 fix).

**Error semantics (ADR-034 §D4):**
Keyring miss → fall through to Tier 4. Keyring backend error (locked, unavailable, spawn panic) → hard `BackendUnavailable` with error code `E-CRED-003`.

**Boot wiring (ADR-034 §D5):**
`BootContext` gains `credential_store_org_id: Arc<dyn CredentialStoreOrgId>`. Step 5 exposes the `KeyringBackend` via both trait types. Step 9A wires `Arc::clone(&ctx.org_registry)` + `Arc::clone(&ctx.credential_store_org_id)` into auth provider construction.

### Route List (who changes what)

| Specialist | Artifact(s) to modify |
|------------|----------------------|
| product-owner | BC-2.06.003 v1.3 §Tier 3 (mark IMPLEMENTED); `error-taxonomy.md` (add E-CRED-003); S-DEMO-003 AC-005 + §Architecture Compliance Rules (OrgId-keyed write) |
| test-writer | `crates/prism-credentials/tests/bc_2_06_003_tier3_keyring_resolution.rs` (RG-034-001, RG-034-002); `crates/prism-bin/tests/bc_2_03_007_credential_set_org_id_keyed.rs` (RG-034-004); `credential_cli.rs` unit test for HIGH-3 (RG-034-003) |
| implementer | All files listed in ADR-034 §File Create / Modify List — 9 modified files + 2 created test files |
| state-manager | Add ADR-034 row to `ARCH-INDEX.md` ADR Registry; commit `.factory/` artifacts |

### Effort Re-estimate

8–10 story points, MEDIUM risk. See ADR-034 §D7 breakdown.
