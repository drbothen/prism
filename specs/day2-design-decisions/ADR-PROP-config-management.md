---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C9-1: Config authority — DB-authoritative, UI-only authoring, no hand-edited TOML in production"
  - "ADR-PROP-C9-2: Versioning split — runtime-config = DB-native temporal history; detection-content+recipes = real embedded git (git2); optional async git projection for runtime history is a nicety only"
  - "ADR-PROP-C9-3: Fast-revert — ArcSwap hot-swap, append-only/forward-only, satellite-auto-revert, seconds"
  - "ADR-PROP-C9-4: Approval-gate — DROPPED to DAY-3; configurable review workflows deferred"
  - "ADR-PROP-C9-5: Bootstrap recovery — 4-layer (validate-before-persist, A/B dual-slot, supervisor watchdog, satellite autonomous self-recovery + tiered local signal) + fleet-staged canary"
  - "ADR-PROP-C9-6: Canary mechanics — TWO-TIER apply model (HIGH-BLAST canary / LOW-BLAST direct+fast-revert); cohort = config-scope-dependent (tenant/site); soft regressions included in trip signals via CUSUM/ADWIN"
  - "ADR-PROP-C9-7: Schema versioning — HYBRID + per-domain split; HUB-AND-SPOKE migration chain; skip-version-release supported (skip-step forbidden); serde + RocksDB value bytes; synchronous at boot"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C9 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/config-schema-versioning-migration-2026-06-27.md (Q3 PRIMARY —
  three sonar-deep-research calls at reasoning_effort=high); research/config-management-depth-2026-06-27.md;
  research/config-authority-narrow-git-2026-06-27.md; research/git-as-primary-vs-write-behind-2026-06-27.md;
  research/bootstrap-config-recovery-2026-06-27.md. All C9 decisions are human-confirmed 2026-06-27.
  Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §16.4 (C9 decisions log entry)
  - day2-design-decisions/ADR-PROP-detection-engine-depth.md (C6 — shared CUSUM/ADWIN change-detector primitive for canary trip signals)
  - day2-design-decisions/ADR-PROP-central-deployment-access-layer.md (C1 — deployment access, shared-state, ops)
  - day2-design-decisions/ADR-PROP-satellite-mesh.md (C2 — residency, satellite autonomous self-recovery, dial-home escalation)
  - day2-design-decisions/ADR-PROP-siem-lake-federation.md (C5 — OCSF version axis; cold tier interacts with Iceberg schema migration posture)
  - matured-vision-day2-requirements.md §3.6 (coverage banner / resilience — soft regression signals for canary)
  - research/config-schema-versioning-migration-2026-06-27.md (Q3 PRIMARY research basis)
  - research/config-management-depth-2026-06-27.md
  - research/config-authority-narrow-git-2026-06-27.md
  - research/git-as-primary-vs-write-behind-2026-06-27.md
  - research/bootstrap-config-recovery-2026-06-27.md
  - CLAUDE.md (#[non_exhaustive] discipline — compile-time API guardrail; ArcSwap for config hot-reload AD-007; RocksDB CFs; serde versioning patterns)
---

# ADR-PROP — Config Management (C9)

> **STATUS: FULLY DECIDED 2026-06-27 (human) — Q1 (authority/versioning), fast-revert,
> bootstrap-recovery, Q2 (canary mechanics), and Q3 (schema-versioning / deployment-awareness)
> all RESOLVED.** This is a CAPTURE artifact for the side-analysis C9 program.
> `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to the
> morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/config-schema-versioning-migration-2026-06-27.md` — Q3 primary
> (three sonar-deep-research calls at `reasoning_effort=high` covering hybrid additive/explicit
> schema-versioning, skip-version mechanics, serialization format selection, and deployment-aware
> migration posture). `research/config-management-depth-2026-06-27.md`,
> `research/config-authority-narrow-git-2026-06-27.md`,
> `research/git-as-primary-vs-write-behind-2026-06-27.md`,
> `research/bootstrap-config-recovery-2026-06-27.md` — Q1, fast-revert, canary,
> bootstrap-recovery research bases.

> **Key correction honored.** `#[non_exhaustive]` is a **compile-time cross-crate API
> guardrail** with ZERO effect on serialization compatibility (serde does not see Rust
> visibility attributes at runtime; derive macros emit field access, not visibility checks).
> `#[non_exhaustive]` MUST be retained on all public config types per CLAUDE.md discipline;
> it must NEVER be cited as the mechanism that makes skip-version-safe deserialization work.
> The serialization-compat story is carried entirely by serde `#[serde(default)]` +
> tolerant deserialization + the explicit migration chain.

---

## Context

Prism is deployed across three operating models with radically different config-management
risk profiles:

- **SaaS (k8s blue-green, continuous):** rolling deployment, instant rollback, single
  authoritative DB, no skip-version pressure.
- **MSSP-managed (offline-signed-bundle, A/B appliance, watchdog):** operator-controlled
  upgrade bundles, A/B slot validation, satellite fleet with tiered self-recovery.
- **Client-managed self-operated:** highest skip-version exposure (e.g., v1.2 → v1.7);
  entire migration chain must be embedded in the binary; idempotent/atomic/resumable migration
  is non-negotiable.

C9 decides HOW config is authored, versioned, hot-reloaded, rolled out with canarying, safely
recovered on bad pushes, and schema-migrated across version skips. Seven decision blocks cover
the complete space.

A cross-cutting three-operating-model deployment matrix (SaaS / MSSP-managed / client-managed)
is the subject of a SEPARATE forthcoming `ADR-PROP-dual-deployment.md`. The migration posture
described in §D-C9-Q3-MODEL/SKIP below is deployment-conditional and cross-links to that document.

---

## Decision Ledger

### D-C9-Q1-AUTHORITY — Config Authority + UI-Only Authoring

**DECIDED 2026-06-27 (human).**

All configuration is **DB-authoritative**. The UI is the ONLY supported authoring surface in
production — hand-edited TOML in production is explicitly prohibited for runtime config. The
DB is the single source of truth for all runtime-config state.

**Rationale:** A single DB-authority model dissolves the dual-write problem (TOML on disk AND
DB: which wins?), enables atomic in-transaction versioning semantics (see D-C9-Q1-VERSION
below), and prevents operator-side config drift that is undetectable by the UI layer.

**Eat-your-own-dogfood consistency check:** The "built-in sensors config-driven" principle
(CLAUDE.md project memory — CrowdStrike/Cyberint/Claroty/Armis ship as TOML specs) applies to
the *sensor spec definition* at development time, not runtime config management. Those TOML
sensor specs live in version control and are ingested during deployment; they do NOT use the
hand-edited-in-production model being prohibited here. There is no conflict.

---

### D-C9-Q1-VERSION — Versioning Split by Domain

**DECIDED 2026-06-27 (human).**

Versioning is split by config domain because the authoring model and mutation patterns differ
fundamentally:

**RUNTIME CONFIG = DB-native temporal / system-versioned history:**
- "Git-semantics over the store" — append-only versioned rows in the DB with generation numbers,
  valid-from/valid-to timestamps, author, and reason. This is temporal table semantics.
- In-transaction exactly-once: a config change is a DB transaction that atomically writes
  the new generation and the old generation's valid-to. No dual-write to a git repository.
- The optional async git projection of runtime-config history (exporting config diffs to a
  real git repo as an audit view) is a NICETY ONLY — NOT authoritative, NOT the write path,
  NOT the recovery path. Operationally, this projection is off by default and carries no
  special semantics.
- High-blast-radius classes (satellite trust anchors, connector definitions, pushdown
  descriptors, retention policies) are flagged for canary + rollback ceremony (see D-C9-Q2
  canary mechanics below).

**DETECTION CONTENT + RECIPES = real embedded git (git2):**
- Detection rules, correlation specs, recipe library content use **real embedded git**
  (`git2 0.19.0` — `gix` not yet stable at capture time; decision to be revisited at morph
  if gix 1.0 stabilizes before implementation).
- Git IS the version axis: branching, commit history, tag-based versioning, and diff are
  the natural substrate for detection content lifecycle (shadow → canary → production promotion
  maps to git branch promotion).
- The **opt-in residency-gated remote** (GitHub or other remote for detection content and
  recipes) is DAY-2 scope for detection-content and recipes ONLY; it is OFF by default
  (air-gap-safe). Runtime config is never pushed to an external git remote.
- Narrow-cut verdict: detection-content-in-git is the mainstream pattern validated by prior
  art (Chronicle Rules Engine, Elastic detection-rules, Splunk ESCU — all use git-native
  versioning for rule content). It is NOT the mainstream pattern for operational config
  (no major infrastructure product uses real git as the write path for runtime config — they
  use DB + temporal tables, which Prism mirrors for runtime config).

[research/config-authority-narrow-git-2026-06-27.md; research/git-as-primary-vs-write-behind-2026-06-27.md]

---

### D-C9-FAST-REVERT — Fast Revert for Hot-Reloadable Config

**DECIDED 2026-06-27 (human).**

**Mechanism:** One-action restore of the prior generation of hot-reloadable config. Implemented
via atomic `ArcSwap` hot-swap (the AD-007 config hot-reload primitive already in Prism).
Semantics:

- **Append-only / forward-only — never rewrite history.** The "revert" is a NEW generation
  that points to the prior generation's content. The DB retains the failed generation in its
  history for audit. History is never mutated.
- **Seconds, no rebuild/restart.** ArcSwap load + atomic pointer swap. In-flight queries hold
  a snapshot reference across their lifetime and complete against the generation they started
  with.
- **Satellites self-revert + pick up on next dial-home.** Satellites carry their last-known-good
  generation locally; on a bad push the satellite falls back to the local generation and reports
  `DEGRADED` to the coordinator on next dial-home (NOT flap — see D-C9-Q2 below for the
  tiered-local-signal discipline).
- **Anchors the canary auto-rollback.** The same ArcSwap generation-swap that a human triggers
  for a manual revert is the atomic operation that the canary circuit-breaker triggers
  automatically on the cohort (see D-C9-Q2 below).

**Scope:** Fast-revert applies to **hot-reloadable config** (runtime config, feature flags,
detection rules, TTLs, retention policies, connector definitions, pushdown descriptors).
It does NOT apply to **restart-class / bootstrap keys** (crypto material, DB connection strings,
port bindings, mTLS trust anchors) — those use the A/B dual-slot bootstrap recovery
mechanism (D-C9-BOOTSTRAP below), not fast-revert.

[research/config-management-depth-2026-06-27.md]

---

### D-C9-APPROVAL — Approval Gate: DROPPED to DAY-3

**DECIDED 2026-06-27 (human).**

Configurable approval / review WORKFLOWS (multi-person sign-off, approval chains, per-client
or per-operator review gates) are **DROPPED from day-2** scope and explicitly deferred to
DAY-3. The complexity of a flexible approval-workflow engine exceeds day-2 scope.

The canary + fast-revert posture (D-C9-Q2 and D-C9-FAST-REVERT) provides the safety gate that
prevents unreviewed config from causing silent damage. Day-3 adds configurable review workflows
ON TOP of the canary/fast-revert safety net.

---

### D-C9-BOOTSTRAP — Bootstrap Recovery for Restart-Class Keys (4-Layer)

**DECIDED 2026-06-27 (human).**

Restart-class config (crypto material, DB connection strings, port bindings, mTLS trust anchors,
satellite join-tokens) uses a four-layer recovery posture layered from cheapest to most robust:

**Layer 1 — Validate-before-persist (CHEAP checks only at write time):**
- Cert parse + expiry check, token well-formedness, URI syntax validation.
- Port-bindable and store-connects-OK checks are RACY (the port may be available at write
  time but not at boot time; the store may be reachable now but not in 30 seconds) → these
  are NOT write-time gates. They are boot-time backstop checks (Layer 2/3).
- The validate-before-persist layer is a cheap syntactic guard, NOT a semantic correctness
  guarantee. Document this distinction explicitly in all downstream specs.

**Layer 2 — A/B dual-slot (active/pending):**
- `active` slot = last-known-good config.
- `pending` slot = new config.
- Promote `pending → active` ONLY AFTER the readiness probe passes against the new config.
- If the readiness probe fails, keep `active` as-is; mark `pending` as failed; alert.
- This is the K8s rolling-update pattern applied to per-process config slots.

**Layer 3 — Supervisor watchdog auto-fallback:**
- READINESS probe (not liveness) determines whether a config is viable at boot.
- N failed boots on the `pending` config → automatic fallback to `active` + reboot.
- **`sd-notify 0.5.0` / systemd integration: mature** — use for the systemd-supervised
  deployment path.
- **Bundled PID-1 (vendor: e.g., s6, runit, tini):** vendor maturity is 0.x for some options —
  flag for a day-2 maturity check before selecting the bundled-PID-1 for the MSSP-managed
  bundle path. Decision deferred to morph.
- Rationale: the watchdog auto-fallback is the last local defence before a boot-bricking
  config causes a site outage. It must not require a human SSH session to recover.

**Layer 4 — Satellite AUTONOMOUS self-recovery with TIERED LOCAL SIGNAL:**
- **Tier-1 (local signal only — "confirm or revert"):** if the satellite can validate the new
  config locally (cert parse, connectivity check to local resources), it applies; if local
  validation fails, it reverts to its last-known-good and reports `DEGRADED`.
- **Tier-2 (dial-home — escalation, NOT revert):** if the satellite is locally-healthy but
  cannot reach the coordinator, it reports `DEGRADED` on next successful dial-home. A
  locally-healthy-but-isolated satellite does NOT auto-revert on failed dial-home. This
  is the distinction between "my config is bad" (Tier-1 revert) and "my config is fine but
  the network is broken" (Tier-2 escalation). Flapping on network partitions is explicitly
  prohibited.

**Fleet-staged bootstrap canary:** The full bootstrap-recovery posture is wrapped by a fleet-
staged canary for A/B-appliance deployments (analogous to Azure Device Update percentage-rollout
+ minimum-device-count → group-rollback). The bootstrap canary ALWAYS uses the same
`active/pending` dual-slot boundary at each device; the fleet-staged layer controls HOW MANY
devices receive the `pending` push in each cohort.

**NEW ATTACK SURFACE:** The safe-mode console (the out-of-band recovery surface for a device
whose config is so broken it cannot boot) is a new attack surface. It must be
**security-reviewed before shipping** — route to security-reviewer at the relevant morph story.
Specifically: the safe-mode console must not be reachable over the normal network interfaces;
must require physical presence or an OOB channel; must not expose config content in cleartext.

**OPEN (NIST-800-82 / IEC-62443):** The normative fail-safe anchor for the bootstrap posture
against NIST SP 800-82 (industrial control system security) and IEC 62443 (industrial
cybersecurity) is a SEPARATE standards-compliance pass deferred to a dedicated morph story.
Recorded as an open item; NOT blocking day-2 architecture decisions.

[research/bootstrap-config-recovery-2026-06-27.md]

---

### D-C9-Q2-HEALTH — Canary Trip Signals (Soft + Hard Regressions)

**RESOLVED 2026-06-27 (human).**

Canary auto-rollback trip signals INCLUDE **soft regressions** in addition to hard-failure
signals:

**Hard signals (component/service failure):**
- Component load failure after config push.
- Connector load failure (a sensor adapter that was healthy is now erroring).
- Satellite reports `DEGRADED` after picking up the new config.
- Per-sensor fetch failure rate above threshold.

**Soft signals (behavioral degradation — included at CONSERVATIVE threshold):**
- Coverage-banner drop (§3.6 resilience metric) — fewer expected data sources answering queries.
- Availability-cache degradation — the ARC / availability cache is returning more misses or
  stale results than before the push.
- Query error-rate uptick — PrismQL query error rate materially higher than pre-push baseline.
- Empty-result-rate climb — sensors returning empty tables at higher rate than before the push
  (may indicate a config change broke a filter, a time-window, or a credential).
- Normalization-failure rate increase — more OCSF normalization errors than before.

**Change-detection primitive:** Soft-signal trip is implemented via the same CUSUM / ADWIN
primitive used in C6 detection auto-rollback (D-C6-3). The shared primitive is the C6 §D-C6-3
CUSUM/ADWIN family (Prism builds the change-detector ONCE and points it at different target
streams). For config-push canary, the stream is the config-health metrics above; for detection
canary (C6), the stream is alert-volume/cardinality.

**Error-asymmetry rationale:** Fast-revert is cheap and non-destructive (append-only, no history
rewrite, in-flight queries complete against their snapshot generation). The cost of a false
trip (rolling back a good config push) is operator inconvenience + re-push. The cost of a
missed regression (a bad config push silently degrades coverage for hours) is a security gap.
Error asymmetry strongly favors conservative trip thresholds.

**Correlation to this push:** The trip logic MUST correlate the regression signal to
THIS-CONFIG-PUSH-HITTING-THIS-COHORT before opening the circuit. An upstream source outage that
happens to coincide with a config push MUST NOT be misread as bad config. Correlation check:
signal onset time aligns with cohort receiving push; regression is NOT observed on control
cohort (cohort that has NOT received the push); signal subsides if the push is rolled back to
the control cohort. This is the false-positive discriminator for the trip signal itself.

[research/config-management-depth-2026-06-27.md; ADR-PROP-detection-engine-depth.md D-C6-3]

---

### D-C9-Q2-COHORT — Canary Cohort Unit

**RESOLVED 2026-06-27 (human).**

The canary cohort unit is **config-scope-dependent**:

- **Tenant-scoped config** (per-tenant feature flags, per-tenant retention policies, per-tenant
  connector credentials): cohort unit = TENANT. Roll out to one tenant, then a small group,
  then all tenants.
- **Fleet-distributed config** (satellite trust anchors, connector definitions, pushdown
  descriptors, retention policies that apply cluster-wide): cohort unit = SATELLITE / SITE.
  Roll out to one satellite/site, then a small group, then the full fleet.

A single config change that spans both tenant-scoped and fleet-distributed surfaces must be
split at the cohort boundary before rolling out. Compound changes that cannot be split are
treated as fleet-distributed (the more conservative scope).

---

### D-C9-Q2-TIERS — TWO-TIER Apply Model

**RESOLVED 2026-06-27 (human).**

Config pushes use a two-tier apply model based on blast radius:

**HIGH-BLAST classes → canary required:**
- Connector definitions (changes can break all fetch paths for affected sensors).
- Pushdown descriptors (changes can break all query-pushdown for affected sensors).
- Retention policies (changes can destroy data or cause runaway retention growth).
- Satellite trust anchors / mTLS material.
- Detection rule promotion to production (C6 D-C6-3 shadow→canary→production; same model).

HIGH-BLAST classes always traverse the full canary loop: deploy to one cohort → wait for bake
window → check soft + hard signals → proceed or roll back.

**LOW-BLAST classes → apply directly with fast-revert available:**
- Feature flags.
- Log-level / verbosity tuning.
- TTL adjustments (within policy bounds).
- UI-only config (dashboard layouts, notification preferences, display thresholds).
- Non-structural query hints.

LOW-BLAST classes apply directly to the full deployment with no staged cohort loop. Fast-revert
(D-C9-FAST-REVERT) is always available; the soft-signal monitoring runs continuously. If a
LOW-BLAST push triggers a soft signal, fast-revert is the response path (no canary cohort
machinery needed because the change is already full-fleet by the time the signal fires).

**Classification is LOCKED at the config-type level, not the value level.** A connector
definition that changes only the display name is still HIGH-BLAST because the class is
HIGH-BLAST. The classification is on the type of config, not the magnitude of the specific
change. This eliminates the need for change-magnitude analysis at deploy time.

[research/config-management-depth-2026-06-27.md]

---

### D-C9-Q3-MODEL — Schema Versioning: HYBRID + Per-Domain Split

**RESOLVED 2026-06-27 (human-confirmed). Four decisions.**

#### D-C9-Q3-MODEL — Hybrid + HUB-AND-SPOKE

Schema versioning uses a **hybrid additive-forward-compat-by-default + explicit-migration-chain**
model, with independent per-domain version registries:

**Additive forward-compat by default (serde `#[serde(default)]` + tolerant deserialization):**
The additive majority of config changes (adding optional fields, adding new enum variants,
adding new optional sections) requires ZERO migration code. serde `#[serde(default)]` handles
missing fields; tolerant deserialization handles unknown fields via `#[serde(deny_unknown_fields)]`
ABSENCE (i.e., `deny_unknown_fields` is NOT used on versioned config types — unknown fields are
ignored by default). This covers the common case with no migration overhead.

**Explicit per-domain `schema_version` + migration chain for the BREAKING subset:**
Breaking changes (field removal, field rename with semantic change, type change, new required
invariant, split/merge of a section, unit/enum remap) require explicit migration steps. The
architecture:

- **ONE migration-runner abstraction** with a registry of ordered, idempotent migration
  functions keyed by `(domain, from_version → to_version)`.
- **N independent per-domain version registries** — each domain (runtime-config, detection-
  content, RocksDB CFs, etc.) has its own `schema_version` integer and its own ordered
  migration chain. Domains never share version numbers.
- **HUB-AND-SPOKE conversion:** one canonical "current" schema per domain; migration spokes
  convert FROM older versions TO the canonical current. This keeps the migration graph linear
  (2×(N−1) total migration functions for N versions, not the quadratic N×(N-1)/2 that a
  point-to-point model requires). Borrowed from Kubebuilder's hub-and-spoke webhook pattern.

**Skip-version-RELEASE supported; skip-migration-STEP forbidden:**
- A deployment may skip one or more RELEASE versions (e.g., installing v1.7 on a system
  running v1.2). This is supported. The binary carries the FULL ordered migration chain for
  all intermediate steps. At boot, the runner replays the PENDING subset: from v1.2's schema
  it applies v1.2→v1.3, then v1.3→v1.4, ..., then v1.6→v1.7 sequentially.
- Skipping an individual MIGRATION STEP (jumping from v1.2 directly to v1.7's final state
  without running intermediate steps) is FORBIDDEN. Migration steps are the unit of
  correctness; each step has its own invariant checks. Step-skipping violates the idempotency
  and atomicity guarantees.

**Per-domain posture:**

| Domain | Versioning mechanism |
|--------|---------------------|
| Runtime config | Migration chain (Option A — DB-authority + UI-authoring makes explicit chain cheaper/safer than any surveyed product pattern). |
| Detection content + recipes | Git IS the version axis + thin content `schema_version` for rare structural change. |
| RocksDB hot data | Per-CF `schema_version` meta key (e.g., `__schema_meta__` CF key prefix) + on-open migration chain. CF-per-version REJECTED (rust-rocksdb issue #608 — excessive CF proliferation degrades compaction and memory; per-key-prefix/value-tag versioning is the validated alternative). |
| Iceberg + OCSF cold tier | Additive (D-C9-Q3-MODEL Option B absorbed by OCSF schema-version axis — C5 decision) + Iceberg native column-id evolution. Record OCSF version per partition. Build NO proprietary version axis for the cold tier. |

[research/config-schema-versioning-migration-2026-06-27.md §primary]

---

#### D-C9-Q3-SKIP — Bounded Skip-Version Window + LTS Required-Stops

**RESOLVED 2026-06-27 (human-confirmed).**

Skip-version jumps are supported within a **bounded supported window**. Older jumps (outside the
window) must stop at one or more **LTS "required-stop" versions** before reaching the current
release.

Pattern: GitLab LTS-required-stops (x.2/x.5/x.8/x.11) + Elastic forward-compat-range +
Consul forward-compat-range. These products bound their CI upgrade matrix by requiring customers
to pass through LTS points rather than supporting arbitrary N-to-M upgrades. Prism mirrors this
posture.

**MECHANISM IS DECIDED; business parameters are deferred to GA:**
- The required-stop-capable migration runner is built NOW as part of D-C9-Q3-MODEL.
- The EXACT supported window size (K minor versions without a required-stop) and the EXACT
  required-stop cadence (which releases are designated LTS required-stops) are a
  SUPPORT-POLICY / BUSINESS DECISION to be set at GA by the product owner. This is recorded
  as an **OPEN BUSINESS DECISION**, not a blocking architectural gap.

**Non-negotiable testing posture (bounded skip-version correctness demands this):**
- One golden fixture per released schema version per domain (serialized config object at
  that version's `schema_version`).
- Round-trip test: deserialize golden fixture → apply all pending migrations → re-serialize
  → verify invariants hold.
- Forward-migration test: from each historical golden fixture → apply chain to current →
  assert final state matches expected.
- Upgrade-matrix CI: across all supported-window skip-version pairs (not exhaustive all-pairs
  — only within the supported window). This bounds the CI matrix.

[research/config-schema-versioning-migration-2026-06-27.md §skip-version]

---

#### D-C9-Q3-FORMAT — Stay Serde + RocksDB Value Bytes

**RESOLVED 2026-06-27 (human-confirmed).**

The serialization format for versioned config is **serde 1.0.228 + additive evolution patterns
+ value-level version tag**. No new serialization-format dependency is added.

**Serde additive evolution patterns in use:**
- `#[serde(default)]` on all optional fields (new fields in a new version are optional with
  a sensible default).
- Internally-tagged version enums (`#[serde(tag = "schema_version")]`) for version dispatch.
- `#[serde(alias = "old_name")]` for field renames where semantics are preserved.
- `#[serde(rename = "new_name")]` for final canonical names.

**Rejected alternatives:**
- **savefile 0.20.4:** REJECTED as default. savefile has smaller size than serde-msgpack but
  it is a niche format with limited ecosystem. It MAY be reconsidered for a specific
  perf/footprint need (e.g., bulk RocksDB value storage with extreme size pressure), but it
  is NOT the default. Reserve for a measured, justified need at morph.
- **serde_version 0.5.1:** REJECTED. Abandoned crate, nightly-only. Not production-viable.

The value-level version tag means every serialized config value includes a `schema_version`
field (or a version discriminant in an internally-tagged enum). The migration runner reads this
field on open/load and determines whether migrations are needed before serving the value.

[research/config-schema-versioning-migration-2026-06-27.md §format]

---

#### D-C9-Q3-TIMING — Synchronous at Boot

**RESOLVED 2026-06-27 (human-confirmed).**

Runtime-config schema migrations replay **synchronously at startup, before the service begins
serving requests** (Grafana-style). Config volume is small (no terabyte datasets are being
migrated; this is TOML/JSON-equivalent config objects). Synchronous migration pairs cleanly
with the A/B dual-slot (Layer 2 bootstrap recovery): the `pending` slot is migrated and
validated in the boot sequence; promotion to `active` happens after the readiness probe passes.

**Background / lazy migration NOT adopted for runtime config.** Reserve background migration
for the case where system-versioned runtime-config history grows large enough that replaying
the full history at boot would be unacceptably slow (not a foreseeable problem for Prism-scale
config volumes at day-2). If that condition arises, the decision is reopened.

**RocksDB on-open migration** (for hot-data CFs) also runs synchronously during the `open()`
path, before the RocksDB handle is returned to callers. The migration is idempotent and
atomic: it reads the `__schema_meta__` key, applies pending steps in order, and writes the
updated meta key in a single `write_batch()`. A crash mid-migration is safe: on next open the
meta key shows the pre-crash version and the chain is replayed from that checkpoint.

[research/config-schema-versioning-migration-2026-06-27.md §timing]

---

### D-C9-Q3-DEPLOYMENT — Deployment-Aware Migration Posture

**RESOLVED 2026-06-27 (human-confirmed).**

Migration posture is deployment-conditional. The three operating models map to different
migration risk profiles:

| Operating model | Skip-version exposure | Migration posture |
|-----------------|----------------------|-------------------|
| **SaaS (k8s blue-green, continuous)** | LOW — walks every release, one step at a time | Forward chain barely exercised; required-stops not needed; blue-green blue/green enables instant rollback if a migration bricks the green instance. |
| **MSSP-managed (offline-signed-bundle, A/B appliance, watchdog)** | MEDIUM — bundle may skip a minor or two | Bundle MUST carry full ordered chain. A/B slot validates migrated state before cutover (Layer 2). Watchdog covers a boot-bricking migration (Layer 3). |
| **Client-managed self-operated** | HIGH — v1.2 → v1.7 is realistic | Requires full chain in binary + supported-window skip-version check + required-stop at the oldest in-window LTS + golden-fixture upgrade-matrix CI + idempotent / atomic / resumable on-open migration (bootstrap 4-layer covers crash-during-migration). |

The full three-operating-model deployment matrix (topology, release mechanics, OTA update
model, air-gap posture) is the subject of a SEPARATE forthcoming
`ADR-PROP-dual-deployment.md`. The migration posture above is the C9-local slice of that
cross-cutting decision; it cross-links to the deployment ADR-PROP as forthcoming.

---

## Provable Invariants (PIV-C9-*)

These are testable assertions that downstream verification properties (VP-NNN, morph-time)
must prove or test:

| ID | Invariant |
|----|-----------|
| **PIV-C9-001** | Config is NEVER authored outside the DB/UI authority in production. No filesystem TOML write path for runtime-config exists in the production code path. |
| **PIV-C9-002** | Fast-revert is APPEND-ONLY / FORWARD-ONLY. The "revert" operation creates a NEW generation row; it never mutates or deletes an existing generation row in the DB or git history. |
| **PIV-C9-003** | No migration STEP is ever skipped even on a skip-version RELEASE. The migration runner replays EVERY pending step in `from_version..current_version` order regardless of how many release versions were skipped. |
| **PIV-C9-004** | On-open RocksDB migration is idempotent. Running the same migration chain twice against a CF at the same `schema_version` produces the same result as running it once. |
| **PIV-C9-005** | On-open RocksDB migration is atomic per-step. A crash mid-step leaves the CF at the pre-step `schema_version` (meta key not advanced until the step write_batch commits). |
| **PIV-C9-006** | On-open RocksDB migration is resumable. After a crash-during-migration, the next open() detects the pre-crash `schema_version` and replays from that checkpoint. |
| **PIV-C9-007** | `#[non_exhaustive]` is NEVER cited as a serialization-compat guarantee in any spec, comment, or doc. The compile-time guardrail and the runtime serde evolution contract are explicitly decoupled in code comments and spec prose. |
| **PIV-C9-008** | Cold-tier (Iceberg + OCSF) carries NO proprietary version axis. Every cold-tier partition records an OCSF version field; Iceberg column-id evolution handles structural change. No `schema_version` meta key is added to Iceberg tables. |
| **PIV-C9-009** | HIGH-BLAST config classes ALWAYS traverse the canary loop before full-fleet deployment. There is no escape hatch that promotes a HIGH-BLAST config change directly to the full fleet without at least one canary cohort. (Exception: a human-authorized emergency bypass with a structured audit event.) |
| **PIV-C9-010** | LOW-BLAST config changes are NEVER routed through the canary cohort machinery. They apply directly (full-fleet) with fast-revert available. This invariant bounds the blast-radius of the canary infrastructure. |
| **PIV-C9-011** | Satellite autonomous self-recovery: a locally-healthy-but-isolated satellite MUST NOT auto-revert its config on a failed dial-home. The revert trigger is local-validation-failure ONLY, not network-partition. |
| **PIV-C9-012** | Canary trip correlation: the trip logic must verify the regression signal is correlated to this-config-push-hitting-this-cohort before opening the circuit. A signal observed equally on the control cohort (not yet receiving the push) MUST NOT trigger rollback. |

---

## Open Items (Morph-Time Architect Decisions)

| ID | Question |
|----|---------|
| **OQ-C9-1** | git2 0.19.0 vs gix for detection-content versioning. Decision: use git2 0.19.0 NOW (gix not yet stable at capture). Revisit at morph if gix 1.0 has shipped and has sufficient ecosystem maturity. The switching cost is low (detection-content git embedding is isolated behind a `ContentStore` trait). |
| **OQ-C9-2** | Exact supported skip-version window (K minors) + LTS required-stop cadence. OPEN BUSINESS DECISION — set at GA by product owner. The mechanism is built; the parameters are a support policy. |
| **OQ-C9-3** | Bundled PID-1 selection for the MSSP-managed appliance bundle. Options: s6, runit, tini, systemd-stub. sd-notify 0.5.0 (systemd) is mature; bundled-PID-1 0.x maturity check needed at morph before bundle selection. |
| **OQ-C9-4** | savefile 0.20.4 as a MEASURED perf/footprint opt-in for bulk RocksDB value storage. Not the default. Reserve for a specific measured need at morph. |
| **OQ-C9-5** | NIST-800-82 / IEC-62443 fail-safe normative anchor for bootstrap posture. Separate standards-compliance pass; not blocking architecture decisions. |
| **OQ-C9-6** | Safe-mode console design and security review. New attack surface (see D-C9-BOOTSTRAP). Must be security-reviewed (route to security-reviewer) before shipping the appliance bundle. |
| **OQ-C9-7** | Exact serde version tag structure for internally-tagged version enums in each domain's config types. Standard approach (`#[serde(tag = "schema_version")]` with integer discriminant); concrete type shapes are morph-time implementation. |
| **OQ-C9-8** | Canary trip threshold calibration for soft signals (coverage-banner drop %, availability-cache degradation %, query error-rate multiplier). CUSUM/ADWIN baseline from shadow/pre-push window; conservative thresholds. Concrete values = empirical calibration at morph. |

---

## Downstream SAP-1 Obligations (Not Actioned Here)

SAP-1 probe applies at implementation time. The following event types will be needed in
BC-2.16.002 Canonical Structured Event Catalog at morph:

- `event_type = "config.generation.written"` — emitted when a new config generation is committed.
  Fields: domain, generation_id, author, reason, schema_version, timestamp; audit role = config audit;
  recurrence = per write.
- `event_type = "config.generation.reverted"` — emitted when a fast-revert creates a revert
  generation. Fields: domain, reverted_from_generation, new_generation_id, author, reason; audit
  role = change-management audit; recurrence = per revert.
- `event_type = "config.canary.trip"` — emitted when the canary circuit-breaker trips. Fields:
  config_class, cohort_id, trip_signal, regression_metric, pre_push_baseline, post_push_value,
  correlated_to_push (boolean); audit role = canary audit; recurrence = per trip.
- `event_type = "config.canary.rolled_back"` — emitted when a canary triggers a rollback. Fields:
  config_class, cohort_id, generation_reverted; audit role = change-management audit.
- `event_type = "config.migration.completed"` — emitted when an on-open migration chain completes.
  Fields: domain, from_version, to_version, steps_applied, duration_ms; audit role = schema
  migration audit; recurrence = per open where migration was needed.
- `event_type = "config.migration.step.failed"` — emitted if an individual migration step fails
  (before retry/crash). Fields: domain, from_step_version, to_step_version, error; audit role =
  migration failure audit; recurrence = per failed step.
- `event_type = "config.satellite.reverted"` — emitted by a satellite when it auto-reverts due to
  local validation failure. Fields: satellite_id, config_class, generation_rejected, reason; audit
  role = satellite self-recovery audit; recurrence = per auto-revert event.

All seven categories are flagged here; BC-2.16.002 amendment is morph-time work.

---

## Honest Costs

| Item | Cost / Risk |
|------|-------------|
| **Two versioning systems in one product** | Runtime config uses DB-native temporal history; detection content uses embedded git. These are genuinely different systems. The boundary (DB-side vs git-side) must be maintained as a hard invariant; accidentally routing a runtime config change through git or a detection rule change through DB-temporal would break both the fast-revert and the detection-promotion models. |
| **Migration chain ownership is permanent** | Once Prism ships v1.0, every future version must carry the FULL ordered migration chain back to v1.0 (or the oldest supported-window LTS). This is unbounded maintenance work. The LTS required-stop mechanism bounds the DEPTH of the chain each binary must carry, but does not eliminate the obligation. This is the standard cost of any product that ships to self-managed deployments. |
| **skip-version testing matrix** | Golden fixtures + upgrade-matrix CI across the supported window is non-trivial to maintain as the version count grows. The LTS required-stop mechanism also bounds this, but there is no free lunch: supporting skip-version upgrades requires skip-version testing. |
| **gix immaturity (git2 0.19.0 risk)** | git2 is a mature C binding (libgit2 1.x); gix is pure Rust but still pre-1.0 on many subsystems. The git2 choice is pragmatic; plan for a gix migration when gix 1.0 is stable. |
| **CUSUM/ADWIN shared primitive must be production-quality** | The canary change-detector reuses the C6 circuit-breaker primitive. The quality of the shared primitive determines the quality of BOTH C6 detection auto-rollback AND C9 config canary trip detection. Get it right once. |
| **Safe-mode console attack surface** | Any out-of-band recovery console is a privileged access path. If it is not secured correctly (OOB channel, physical access requirement, no cleartext config exposure), it becomes a privilege escalation vector. Security review is mandatory before shipping. |

---

## Alternatives Considered and Rejected

### Alternative A: Hand-Edited TOML as a Supported Production Config Path

Allow operators to edit TOML files on disk as an alternative to the UI/DB path for runtime config.

**Rejected (D-C9-Q1-AUTHORITY) because:** TOML-on-disk and DB create a dual-authority problem —
when they disagree (after a crash, a partial sync, or a human edit during an automated push),
the system has no way to know which is authoritative. The DB-only authority model eliminates this
class of split-brain bugs. Sensor spec TOML (development-time definitions, not runtime config)
is unaffected by this decision.

### Alternative B: Real Git for Runtime Config (git as the Primary Write Path)

Use git as the write path for runtime config changes (git commit = config push).

**Rejected (D-C9-Q1-VERSION, D-C9-FAST-REVERT) because:** The research explicitly
(`research/git-as-primary-vs-write-behind-2026-06-27.md`) shows that no major infrastructure
product uses real git as the write path for runtime operational config — they use databases
with temporal semantics. Git-as-write-path for runtime config introduces: (1) TOCTOU races
between repo state and service state; (2) no transaction-atomic semantics (a git commit is not
a DB transaction; it can be partially applied to multiple config consumers); (3) ArcSwap hot-swap
cannot atomically point at a git tree head; (4) satellite pull timing is non-deterministic.
Real git is the CORRECT choice for detection content (where branching, history, and merge are
natural operations); it is NOT the correct choice for runtime config.

### Alternative C: Approve Every Config Change via a Review Workflow (Day-2)

Require a multi-person approval workflow for every config change in day-2.

**Rejected (D-C9-APPROVAL) because:** The complexity of a flexible approval-workflow engine
exceeds day-2 scope. The canary + fast-revert safety net provides the production-grade safety
guard for day-2. Approval workflows are a day-3 capability layered on top.

### Alternative D: Per-Version RocksDB Column Families for Schema Migration

Create a new RocksDB CF for each schema version (v1-cf, v2-cf, v3-cf) and migrate data by
copying rows between CFs.

**Rejected (D-C9-Q3-MODEL) because:** rust-rocksdb issue #608 documents degraded compaction
and memory behavior when CF count exceeds a few hundred. Version-keyed CF proliferation would
grow the CF count unboundedly with each schema bump. Key-prefix / value-tag versioning within a
single CF is the validated alternative.

### Alternative E: savefile as the Default Serialization Format (Replace serde)

Use savefile 0.20.4 as the default config serialization format for smaller footprint and faster
serialization.

**Rejected (D-C9-Q3-FORMAT) because:** savefile is a niche format with limited ecosystem
tooling, no standard debugging or inspection path (unlike JSON/TOML-convertible serde output),
and unknown forward-compat story under additive schema evolution. serde's ecosystem (schemars
for JSON Schema generation, serde_json for debug inspection, bincode for compact in-memory
representation) is mature and well-understood. savefile is preserved as a FUTURE opt-in for
specific measured perf/footprint needs only.

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **AD-007 (ArcSwap config hot-reload)** | D-C9-FAST-REVERT formalizes AD-007 into the authoritative fast-revert mechanism for hot-reloadable config. The morph-time ADR for fast-revert (ADR-TBD) must cite AD-007 as the implementation primitive. |
| **RocksDB ALL_DOMAINS CFs** | D-C9-Q3-MODEL adds a `__schema_meta__` key-prefix convention per CF for schema versioning. `prism-core/src/storage.rs ALL_DOMAINS` must be updated to include schema_version metadata for each CF. |
| **BC-2.16.002 §Postconditions** | Seven SAP-1 event type categories listed in §Downstream SAP-1 Obligations above (morph-time BC work). |
| **C6 D-C6-3 shared primitive** | The CUSUM/ADWIN change-detector is shared between C9 canary (config-health metrics) and C6 detection auto-rollback (alert-volume/cardinality). The implementation must be a single shared crate or module — not two independent implementations. |
| **ADR-PROP-dual-deployment.md (forthcoming)** | The migration posture in D-C9-Q3-DEPLOYMENT (SaaS / MSSP-managed / client-managed) is the C9-local slice of the cross-cutting deployment matrix. The forthcoming dual-deployment ADR-PROP must pick up D-C9-Q3-DEPLOYMENT and reconcile it with the full operating-model topology. |
| **security-reviewer** | D-C9-BOOTSTRAP flags the safe-mode console as a new attack surface requiring security review before shipping. Must be routed to security-reviewer at the morph story that implements the appliance bundle bootstrap recovery. |
| **git2 0.19.0 dependency** | Detection-content versioning adds `git2 0.19.0` to the Cargo workspace. This is a libgit2 C binding; it must pass `just check-ci` (deny + audit) at the morph story that implements detection-content versioning. |
| **matured-vision §16.4** | C9 decision block appended in-place (2026-06-27). |
