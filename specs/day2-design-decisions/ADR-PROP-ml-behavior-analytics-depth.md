---
document_type: proposed-adr
status: capture
do_not_execute: true
decided: "2026-06-27 (human)"
candidate_adr_slots:
  - "ADR-PROP-C7-1: Satellite/edge mergeability posture — PRIMARY=mergeable-only primitives; ALT=approximate-merge to test empirically"
  - "ADR-PROP-C7-2: Pluggable model backend set — full four-backend commitment in day-2 (candle/statistical/ort/wasmtime/tract)"
  - "ADR-PROP-C7-3: Drift/decay/poisoning architecture — dual-rate + quarantine (honest caveat: NOT a closed solution)"
  - "ADR-PROP-C7-4: Model replay/explainability — per-update changelog + periodic materialization (per-detection content-addressed rejected)"
produced_by: architect
timestamp: "2026-06-27"
provenance: >
  Side-analysis C7 capture; human-confirmed decisions 2026-06-27 session.
  Research basis: research/ml-behavior-analytics-depth-2026-06-27.md (six perplexity_research
  sonar-deep-research calls at reasoning_effort=high covering Q1 streaming algorithm toolkit +
  mergeability, Q2 model-state persistence/snapshot/per-tenant, Q3 streaming-update invariants +
  monoid framing + order-dependence testing, Q4 drift/decay/poisoning + the core tension, Q5 Rust
  ML/inference crates + WASM-sandbox BYO backends, Q6 primitive→engine compilation + peer-group +
  cost-bounding; plus 13 live crates.io version-verification calls; plus 2 Context7 resolve
  attempts). Does NOT modify live ADR files, ARCH-INDEX.md, STATE.md, SESSION-HANDOFF.md, or any
  live factory artifact.
traces_to:
  - matured-vision-day2-requirements.md §15 (On-Demand ML & Behavior Analytics — HUMAN-CONFIRMED 2026-06-25)
  - matured-vision-day2-requirements.md §15.3 (model as first-class retention tier)
  - matured-vision-day2-requirements.md §15.4 (online/continuous learning — single-pass-streaming-only)
  - matured-vision-day2-requirements.md §15.5 (honest limits + controls in scope)
  - matured-vision-day2-requirements.md §15.6 (ML-as-PrismQL-primitives)
  - matured-vision-day2-requirements.md §15.7 (two model tiers + pluggable backends)
  - matured-vision-day2-requirements.md §15.8 (OT/satellite/Purdue-edge baselining — §3.2)
  - matured-vision-day2-requirements.md §16.4 (C7 decisions log entry)
  - day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md (C4 WASM sandbox — wasmtime 46.0.1 Component-Model reuse)
  - day2-design-decisions/ADR-PROP-detection-engine-depth.md (C6 — §14.5 replay-link machinery reuse; detection state RocksDB CF)
  - day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md (C1 — RocksDB as the §3.3 hot/model tier)
  - epics E-ML-ONDEMAND-001, E-ML-ONLINE-001, E-ML-PRIMITIVES-001
  - research/ml-behavior-analytics-depth-2026-06-27.md (primary research basis — all six Q1–Q6 depth questions)
  - research/edge-ml-mergeability-depth-2026-06-27.md (C7 FOLD depth research — D-C7-1 resolution; Parts 1–6)
  - CLAUDE.md (#[non_exhaustive] discipline; AD-017 AI-opaque credentials; RocksDB CFs; SAP-1 structured event catalog; error taxonomy)
---

# ADR-PROP — On-Demand ML & Behavior Analytics Depth (C7)

> **STATUS: DECIDED 2026-06-27 (human).** This is a CAPTURE artifact for the side-analysis C7
> program. `do_not_execute: true`. Real ADR numbers and formal ARCH-INDEX.md rows are deferred to
> the morph execution (post-demo, post-T14, gated on brief-reframe sign-off §5.1).

> **Research basis:** `research/ml-behavior-analytics-depth-2026-06-27.md` — six
> `perplexity_research` (sonar-deep-research, `reasoning_effort=high`) calls covering:
> Q1 streaming/single-pass algorithm toolkit + mergeability + Rust-crate state (live crates.io
> verified); Q2 model-state persistence / snapshot / per-tenant mechanics; Q3 online-learning
> update semantics + monoid invariants + order-dependence testing; Q4 concept drift + decay +
> poisoning resistance + the core tension; Q5 Rust ML/inference crates + WASM-sandboxed BYO
> backends; Q6 primitive→engine compilation + peer-group definition + cost-bounding. Plus 13 live
> WebFetch crates.io version-verification calls, all performed 2026-06-27.

> **Settled decisions NOT relitigated in this capture.** The following §15 decisions are
> HUMAN-CONFIRMED 2026-06-25 and are treated as the immutable foundation for C7:
> on-demand/ephemeral ML (no always-on ingestion-time pipeline); model-as-retention-tier (third
> retention tier alongside raw hot/cold); online/continuous learning with single-pass-streaming-only
> algorithms; model covers what Prism touches + optional scheduled sampling; two model tiers
> (lightweight statistical day-2-first, heavier learned tier later); pluggable AI-opaque per-tenant
> model backends; ML-as-PrismQL-primitives (ANOMALY_SCORE, RARITY, FIRST_SEEN, BASELINE_DEVIATION,
> PEER_OUTLIER, PROFILE…OVER inside DETECT).

---

## Context

Section 15 of the matured-vision establishes the ML & behavior analytics model at the requirements
level. This C7 ADR-PROP captures DEPTH and IMPLEMENTATION decisions — HOW to build the open pieces
that §15 left unresolved. Six open implementation questions drove the research pass:

1. **D-C7-1 / Q1 + §3.2:** Which streaming algorithms are cleanly mergeable for satellite/Purdue-edge
   partial aggregation, and which are not? What is the architecture posture?
2. **D-C7-2 / Q5:** What is the full set of pluggable model backends committed in day-2, and what
   are the honest costs?
3. **D-C7-3 / Q4:** How do drift detection, decay, and poisoning resistance compose — and what is
   the honest limit of the chosen solution?
4. **D-C7-4 / Q2 + Q3 + §15.5:** What snapshot/replay mechanism does the per-update model audit
   trail use, and where does the model live?

The implementation leans (Q1 statistical toolkit, Q3 verification strategy, Q6 primitive→engine
compilation) were confirmed in the same human session without objection.

---

## Decision Ledger

### D-C7-1 — Satellite/Edge Mergeability: MERGEABLE-EXACT REPRESENTATIONS BROADLY (C7 FOLD RESOLVED 2026-06-27)

**DECIDED 2026-06-27 (human); C7 FOLD RESOLVED 2026-06-27 (depth research
`research/edge-ml-mergeability-depth-2026-06-27.md`). Resolves the §3.2 / §15.8
satellite/Purdue-edge partial-aggregate design question.**

**Prior posture (going in):** "PRIMARY = restrict to cleanly-mergeable primitives; ALTERNATIVE =
approximate-merge to test empirically." The C7 FOLD upgrades this: representation-change escape
hatches make formerly non-mergeable online primitives **mergeable-EXACT**, so the primary posture
is no longer a narrow restriction — it is the **broad default via mergeable-exact representations**.

#### C7 Backbone — Fully Mergeable by Theorem (unchanged from original capture)

| Primitive | Algorithm | Merge semantics |
|---|---|---|
| Streaming mean/variance | Welford + Chan–Golub–LeVeque (CGL) | Exact associative reduce-tree (CGL formula) |
| Relative-error quantiles | DDSketch (sketches-ddsketch 0.4.0) | Exact — add aligned bucket counts |
| Frequency | Count-min with conservative update (first-party) | Exact — element-wise add |
| Cardinality | HyperLogLog (hyperloglogplus 0.4.1) | Exact — register-wise max |

#### C7 FOLD Escape Hatches — Formerly Non-Mergeable, Now Mergeable-Exact by Representation Change

The depth research (`research/edge-ml-mergeability-depth-2026-06-27.md`, Parts 2–4,
six `perplexity_research` sonar-deep-research calls at `reasoning_effort=high`) establishes that
non-mergeability for EWMA, reservoir, and streaming clustering is an algebraic property of the
**exposed state representation**, not of the underlying statistic. Switching representation
eliminates the obstruction entirely at zero accuracy cost:

| Primitive | Original state (non-mergeable) | Escape-hatch representation | Merge semantics | Confidence |
|---|---|---|---|---|
| **EWMA mean/variance** | Scalar `z_t` (conflates magnitudes + time-ordering irreversibly) | **Forward-decay sufficient statistics `(U, V)`**: `U = Σ xᵢ·exp(λtᵢ)`, `V = Σ exp(λtᵢ)`; decayed avg = `U/V` (the `exp(−λT)` factors cancel) | **Mergeable-EXACT** — `(U₁+U₂, V₁+V₂)` is exact; identity property holds; same decay-function values as backward EWMA (proven). Engineering note: `exp(λt)` grows unbounded; periodic landmark re-basing required to avoid overflow (known, solvable). | [LIT-SETTLED] — Cormode–Shkapenyuk–Srivastava–Xu forward-decay; exploited in production by FDCMSS |
| **Reservoir sampling** | Algorithm R with count-based retention (`k/nᵢ` per shard) | **Random-key / bottom-k (Efraimidis–Spirakis)**: each item assigned key `K = u^(1/w)`; sample = top-k keys globally; merge = union of per-shard top-k sets + re-select global top-k | **Mergeable-EXACT** — commutative monoid; no per-shard count `nᵢ` relay required (eliminating a coordination dependency and a minor disclosure vector) | [LIT-SETTLED] — Efraimidis–Spirakis; Cohen–Kaplan bottom-k; Tirthapura–Woodruff optimal distributed protocol |
| **Streaming clustering (BIRCH CF-vectors, DenStream, CluStream)** | Fading-weight micro-clusters treated as partially combinable | **Additive CF-vectors `(N, LS, SS)`**: BIRCH CF Additive Theorem — `CF(C₁ ∪ C₂) = CF(C₁) + CF(C₂)` exactly; time-reconciliation = scale faded CFs to a common `t_ref` before merge | **Mergeable-EXACT** at the CF level; two bounded-error sources remain: (a) fading-weight time-alignment across shards (error bounded by `2^{λΔ}` where Δ = clock skew — controllable by clock-sync discipline + low λ); (b) final macro-clustering coreset error (`(1+ε)` approximation that exists single-machine too, not introduced by distribution) | CF additivity [LIT-SETTLED] — BIRCH SIGMOD 1996; time-alignment bound [LIT-EXTRAPOLATED]; macro-drift on real data **PRISM-MUST-MEASURE** |

**The C7 FOLK human hypothesis on clustering is CONFIRMED BY THEOREM.** The earlier C7 framing that
streaming clustering was "not cleanly mergeable" was too pessimistic about the arithmetic — the
BIRCH CF Additive Theorem is a proven algebraic identity. The approximation lives only in
time-alignment (bounded, controllable) and macro-clustering (exists single-machine).

#### Upgraded Primitive Assignment Table

| Primitive | Original D-C7-1 assignment | Post-fold assignment |
|---|---|---|
| Welford/CGL mean-var | Mergeable-exact | Mergeable-exact (unchanged) |
| DDSketch / Count-Min / HLL / KLL | Mergeable-exact | Mergeable-exact (unchanged) |
| **EWMA** | Central-only | **Mergeable-exact via forward-decay `(U,V)` representation** — maintain `(U,V)` per feature per decay rate at edge; landmark re-basing required at edge |
| **Reservoir sampling** | Central-only | **Mergeable-exact via random-key / bottom-k representation** — replace Algorithm-R with Efraimidis–Spirakis at edge |
| **Streaming clustering** | Central-only | **Mergeable-exact at CF level** — maintain additive `(N, LS, SS)` (+temporal fields for CluStream; faded `(W, LS, SS)` for DenStream); macro-drift on real security data **MUST be measured empirically** (ARI/NMI/CMM vs centrally-computed reference) |
| Streaming isolation forest / HS-Trees | Central-only | Central-only (unchanged — no equivalent algebraic escape hatch; ensemble-of-ensembles union ≠ centralized training) |

**Scalar-state approximate-merge (scalar EWMA, Algorithm-R + weighted re-sampling) is now a
CONSTRAINED-EDGE FALLBACK ONLY**, not a primary path. Its error bounds are extrapolated (not
literature-settled); it should only be used for legacy/constrained edges where the extra scalar
genuinely cannot be afforded, and only after Part 6 empirical measurement confirms acceptable error
for Prism's λ/N/skew regime.

#### Empirical Validation Item (Narrowed from the Prior Open Question)

The empirical bake-off at the edge-ML build milestone (OQ-C7-1) is **narrowed** by the fold. The
broad "mergeable-only vs approximate-merge" fork is resolved in favor of mergeable-exact
representations. The remaining empirical item is:

**Macro-clustering drift test:** measure whether BIRCH CF-vector merges preserve macro-cluster
fidelity under realistic security data + adversarial cross-shard skew (up to 10:1 ratio). Metrics:
ARI, NMI, purity, centroid drift, CMM vs centrally-computed reference clustering. Acceptance
criterion: ΔFNR ≤ ~0.5pp, ΔFPR ≤ ~2pp, ΔPR-AUC ≥ −0.01 across ALL adversarial regimes. This
measurement drives the final macro-clustering disposition; it is not a fork between representation
approaches.

#### Privacy Axis (Separate from D-C7-1)

The depth research establishes a critical invariant: **representation coarsening is NOT a privacy
mechanism** (Part 5, literature-settled). Deterministic post-processing of a non-private aggregate
remains non-private under the DP post-processing theorem. This distinction matters because it
prohibits any downstream spec from justifying approximate-merge "for privacy" or treating
representation coarsening as satisfying a residency-adjacent privacy requirement.

If a formal privacy guarantee on conduit-crossing aggregates is ever required (a plausible
MSSP/cross-tenant requirement), the answer is **local differential privacy on mergeable DP
sketches** — a SEPARATE decision axis from D-C7-1, not solved by coarsening. Flagged as an open
secondary axis if not already addressed (see Open Questions).

**Relationship to C2 satellite mesh:** D-C2-1 (satellite transport) and D-C2-12 (partial-failure
relay extending BC-2.01.010) already govern how satellites propagate partial results. D-C7-1 adds
the sketch-level semantic: what partial result is valid to relay. With the fold, the `(U,V)` EWMA
tuple and the `(N, LS, SS)` CF vector are each "a normalized aggregate, not raw" per D-C2-12 — they
cross the conduit exactly in the spirit of that invariant. The satellite-local credential
resolution hard invariant (AD-017 / D-C2) applies to ML state: model state artifacts at a satellite
are AI-opaque and never transit raw credential content.

[research/ml-behavior-analytics-depth-2026-06-27.md — Parts 1–6 (C7 FOLD depth research)]
[research/ml-behavior-analytics-depth-2026-06-27.md §Part 2 — EWMA forward-decay escape hatch]
[research/ml-behavior-analytics-depth-2026-06-27.md §Part 3 — random-key/bottom-k substitute]
[research/ml-behavior-analytics-depth-2026-06-27.md §Part 4 — CF additivity theorem]
[research/ml-behavior-analytics-depth-2026-06-27.md §Part 5 — privacy≠residency; coarsening≠DP]
[research/ml-behavior-analytics-depth-2026-06-27.md §Part 6 — empirical test plan (narrowed)]
[research/ml-behavior-analytics-depth-2026-06-27.md §Per-primitive mergeability verdict table]

---

### D-C7-2 — Model Backends: Commit the Full Pluggable-Backend Set in Day-2

**DECIDED 2026-06-27 (human). Resolves §15.7 pluggable-backend design with version commitments.**

Day-2 commits **all four backend types** (plus the first-party statistical tier). This is the
**larger-build option** — the human chose it consciously over the lighter statistical+candle-first
alternative (see Alternatives Considered).

**First-party `ModelBackend` trait (AI-opaque by construction):**
```rust
trait ModelBackend {
    fn load(&mut self, spec: ModelSpec) -> Result<()>;
    fn infer(&self, inputs: InferenceInputs) -> Result<InferenceOutputs>;
    fn train(&mut self, data: TrainingBatch) -> Result<()>;
}
```
- Backend never receives raw credentials — mirrors AD-017 + §11.1 BYO secret-store stance.
- Per-tenant isolated: each tenant has its own `ModelBackend` instance; backends do NOT share state
  across tenants.
- The trait is `#[non_exhaustive]`-compatible — adding new methods is a breaking change; design
  with a versioned extension pattern.

**Four concrete backend implementations:**

| Backend | Crate + version | Role | Isolation | Crates.io verified |
|---|---|---|---|---|
| **First-party statistical** | First-party Rust (Welford/CGL/EWMA/count-min/drift-detectors) + sketches-ddsketch 0.4.0 + hyperloglogplus 0.4.1 + tdigests 1.0.1 | Statistical sketches tier (day-2-first) | In-process, per-tenant key-prefix in RocksDB | 2026-06-27 |
| **candle-core** | candle-core 0.11.0 | Built-in learned tier (first-party neural models); explicit "serverless inference" goal; WASM-compilable | In-process | 2026-06-27 |
| **ort** (ONNX Runtime) | ort 2.0.0-rc.12 | **TRUSTED BYO** — any framework → ONNX; process-isolated; 3–5× faster/60–80% less memory than Python runtime | Process-isolated (separate process or thread-isolated executor) | 2026-06-27 |
| **wasmtime Component-Model** | wasmtime 46.0.1 | **UNTRUSTED BYO** — third-party or customer-supplied models as WASM plugins; capability-restricted; per-tenant WASM instance | WASM sandbox — **REUSE the C4 sandbox pattern** from ADR-PROP-dynamic-schema-connectors.md (D-C4-3; Wasmtime WASI-P2; no ambient authority; fuel + epoch + StoreLimits DoS bounds) | 2026-06-27 |
| **tract-onnx** | tract-onnx 0.23.3 | Satellite/Purdue-edge tiny runtime (pure-Rust ONNX/NNEF, pulsified streaming inference, easy WASM, no C++ dep) | In-process at satellite | 2026-06-27 |

**Sandbox reuse note (WASM-ML = C4 sandbox):** the untrusted-BYO model backend uses the same
Wasmtime WASI-P2 sandbox as the C4 dynamic-schema WASM connectors. See Open Questions for
whether inference workloads need different capability grants than hook/connector plugins (they
likely need more CPU budget — this is an open question for morph). The sandbox mechanism is the
same; the grant configuration may differ.

**HONEST COSTS (stated plainly; not implied away):**

- **`ort` is still RC (2.0.0-rc.12, no stable 2.0 as of 2026-06-27).** Pin the version exactly.
  Budget for API churn between rc releases. The stable 1.x API (ort 1.16.x) is an alternative if
  2.0 API stability is still blocking at morph time — flag as PIV-C7-1.

- **WASM-ML has a performance tax.** CPU small/medium models run fine in WASM. SIMD acceleration
  requires explicit build flags (`wasm32-unknown-unknown` + SIMD feature flags). GPU-in-WASM needs
  WebGPU, which is still maturing. The sandbox is appropriate for untrusted BYO models where the
  performance tax is an acceptable cost of isolation; it is NOT appropriate for latency-critical
  in-process statistical sketches (which use the first-party statistical backend instead).

- **This is the larger-build option.** The lighter alternative (statistical + candle first, ort and
  WASM-BYO deferred) was rejected by the human in favor of committing the full set now. The heavier
  sequencing is a deliberate scope choice, not an incremental fallback. See Alternatives Considered.

- **`candle-core` and `tract-onnx` are healthy and suitable.** `ort` and `wasmtime` are also
  healthy (high-adoption, recently updated) but carry their own honest cost notes above.

**Sequencing within the backend set:** statistical + candle first (day-2-first, §15.7); ort +
WASM-BYO later (within day-2 scope, not deferred to a future brief cycle). `tract` for the
satellite edge context only (§3.2 / D-C7-1 satellite build milestone).

[research/ml-behavior-analytics-depth-2026-06-27.md §Q5, §5.1–5.2 LEAN]

---

### D-C7-3 — Drift / Decay / Poisoning: Dual-Rate + Quarantine

**DECIDED 2026-06-27 (human). Resolves the §15.5 drift/decay/poisoning controls design.**

**Day-2 baseline math:** robust estimators (median/MAD for poisoning resistance) + bounded
per-window update rate (cap how far one update can move the baseline) + anomaly-gated learning
(do NOT update the model from data already flagged anomalous by the same model).

**Drift detectors (MUST-BUILD in Rust — the ecosystem is Python-only):**
The entire drift-detector ecosystem (ADWIN, DDM, EDDM, Page-Hinkley, KSWIN) is dominated by
Python frameworks (River, scikit-multiflow, Frouros). The lone Rust crate (`neural-drift`) does
not document its algorithms. Prism MUST build drift detectors first-party. Build order:
1. **ADWIN** (Adaptive Windowing) — highest priority: self-tuning window doubles as the
   decay/forgetting mechanism; significance parameter δ (Hoeffding-bound); monitors mean of a
   numeric stream via two adaptive subwindows; detects abrupt + gradual drift.
2. **Page-Hinkley** — second: cumulative deviation from mean; gradual drift; trivial implementation
   (~30 LOC); fast first validation target.

**Decay / forgetting:** EWMA decay (λ tunes memory) + adaptive (ADWIN-driven) window (ADWIN
self-tunes when to reset the window = the forgetting trigger) + DenStream fading factors
`w(t) = w₀ · 2^(−λ(t−t₀))` attached to micro-clusters when the streaming clustering tier is
built (later/heavier tier).

**Dual-rate + quarantine design (resolves the core tension):**

| Layer | Role | Adapts? |
|---|---|---|
| **Fast model** | Flags anomalies against the current baseline; high sensitivity | YES — absorbs all non-anomalous updates |
| **Slow model** | The stable baseline; provides the reference the fast model scores against | CONDITIONAL — only absorbs drift that persists past the quarantine window |
| **Quarantine buffer** | Holds flagged-anomalous data; not immediately learned by either model | Pending review |
| **Promotion gate** | Drift promotes quarantined data to the slow model only if it persists for the quarantine window length OR is human/S3-agent-confirmed (§15.8 agent-native) | Explicit |

**Boiling-frog attacker must:** sustain the shifted behavior across the full slow-window AND past
the quarantine window AND past any human/agent confirmation step. This raises the cost of
slow-poison attacks meaningfully vs a single-rate model. Dual-rate + quarantine SHIFTS attacker
cost — it does NOT eliminate poisoning risk.

**HONEST CAVEAT — stated plainly, not implied away:**
The anomaly-gated-learning vs concept-drift tension is **genuinely unsolved in general.** Any
anomaly-gated learning system faces the fundamental contradiction: refusing to learn from
anomalous data starves the model of the first signal that legitimate drift presents as. Dual-rate +
quarantine is the best-evidenced mitigation from the literature (backed by [WEB] citations in the
research); it is NOT a proof that the system is poisoning-safe. The spec must say this plainly:
**"dual-rate + quarantine shifts attacker cost; it does not eliminate boiling-frog risk."** Do not
imply a guarantee.

**Relationship to EWMA mergeability (updated by C7 FOLD):** EWMA is the primary decay mechanism
for the fast model. Under the original D-C7-1 framing, EWMA was central-only. The C7 FOLD
(D-C7-1 updated 2026-06-27) establishes that the forward-decay `(U,V)` representation is
mergeable-EXACT — so the fast model using `(U,V)` EWMA state CAN participate in satellite
fan-in. In practice, satellite/edge baselines in the primary posture still use Welford/CGL +
ADWIN-driven window reset as the forgetting mechanism (for simplicity and because ADWIN itself
does not require the `(U,V)` form). The `(U,V)` escape hatch becomes relevant when the fast-model
EWMA at the edge needs to fan in to central rather than run independently.

[research/ml-behavior-analytics-depth-2026-06-27.md §Q4, §4.1–4.4 LEAN; §Q4 §4.4 core tension]

---

### D-C7-4 — Model Replay / Explainability: Per-Update Changelog + Periodic Materialization

**DECIDED 2026-06-27 (human). Resolves §15.5 "model versioning/snapshots; a finding's replay link
points at model state as of the decision."**

**Chosen mechanism: per-update changelog + periodic materialization.**

Append every model update as a delta to a per-tenant, per-entity-class changelog. Materialize
consolidated snapshots periodically (configurable cadence, e.g., every N updates or every T hours).
A finding's replay link (§14.5 ADOPT-4) references a **(materialization-id + changelog-offset)
pair** → enables replaying EVERY update, not just the per-detection model state.

**Mechanics:**
- **Changelog format:** bincode-serialized `ModelUpdateDelta` entries (schema-versioned envelope;
  `#[non_exhaustive]`; redacted `Debug` for any tenant-identifying or credential-adjacent fields).
- **Materialization:** a periodic background task serializes the current `ModelState` as a
  `ModelStateSnapshot` keyed by `(materialization_id, tenant_id, entity_class, schema_scope)`.
  The `ModelState` envelope:
  ```
  ModelState {
      schema_version: u32,      // bincode format evolution
      model_type: ModelType,    // statistical sketch | candle | ort | wasm-byo
      tenant_id: TenantId,      // newtype, redacted Debug
      schema_scope: SchemaScope, // OCSF class or native schema (§13.6)
      payload: Vec<u8>,         // backend-specific serialized state
  }
  ```
  `ModelState` is `#[non_exhaustive]`, bincode-serialized, redacted `Debug`.
- **Replay link:** `finding.replay_link = (materialization_id, changelog_offset)`.

**Storage:** RocksDB (the §3.3 hot/model tier), NOT a separate model registry.
Per-tenant isolation: dedicated Column Family (CF) for a small number of large/high-isolation
tenants; **key-prefix partitioning** (`tenant_id:schema_scope:entity_class:model_type:...`) within
shared CFs for the long tail. **DO NOT map CF-per-tenant for the long tail** — RocksDB degrades
badly at thousands of CFs (config-file write + validation overhead). This matches the existing
19-CF RocksDB layout philosophy.

**HONEST COST (explicitly captured per human directive):**
The per-update changelog + periodic materialization approach is heavier than the rejected
content-addressed per-detection snapshot alternative (see Alternatives Considered). It requires:
- A changelog append on every model update (not just on detection fire).
- A periodic materialization task (background, configurable, bounded by CF retention policy).
- A `(materialization-id, changelog-offset)` reference scheme for replay links (vs a simple hash).

The human chose this for **full per-update auditability** — the finding's replay link can
reconstruct the exact sequence of model updates that led to the detection, not only the model
state at detection time.

**Bounded retention:** the changelog must have a bounded retention policy (e.g., keep N most-recent
materializations + all changelog entries since the oldest retained materialization; GC older
entries). This bounds storage growth. Policy configuration is per-tenant + operator-configurable.

**No prior art:** neither River, scikit-multiflow, nor any surveyed production online-learning
library documents per-update model state audit trails for security finding replay. This is
novel design ground for Prism.

[research/ml-behavior-analytics-depth-2026-06-27.md §Q2, §2.3 LEAN; §2.2 RocksDB fit]

---

## Implementation Leans (Confirmed 2026-06-27, Human Non-Objection)

These leans were presented in the research and confirmed without objection. They are captured as
decided implementation directions for morph-time ADR authorship. They inform story ACs and epic
scoping but are NOT binding decisions at the ADR-PROP level — the architect confirms crate pins
and detailed API choices at morph against the workspace's actual dependency tree.

### L-C7-1 — Statistical Toolkit: Vendor-vs-Build Line

**Day-2-first statistical tier — production toolkit:**

| Primitive | Approach | Crate + version | Rationale |
|---|---|---|---|
| Streaming mean/variance | **First-party** | None (Welford + CGL, ~100 LOC) | `online-statistics` 0.2.6 stale (2022); first-party is tiny + gives `#[non_exhaustive]` + redacted `Debug` discipline |
| EWMA mean/variance | **First-party** | None | Same rationale; order-dependent → not an isolated crate concern |
| Online z-score / BASELINE_DEVIATION | **First-party** | None | Over Welford or EWMA baseline; robust variant = median/MAD for poisoning resistance |
| Relative-error quantiles | **Depend** | sketches-ddsketch 0.4.0 | HEALTHY (2026-03-18, ~14.4M downloads); fully mergeable; direct Datadog Go DDSketch port |
| Tail-accurate quantiles | **Depend (if needed)** | tdigests 1.0.1 | HEALTHY (2025-12-16, ~429k downloads); explicitly documents merge-from-partitions |
| Cardinality | **Depend** | hyperloglogplus 0.4.1 | High adoption (~2.4M recent downloads); HLL + HLL++ |
| Frequency | **First-party** | None (count-min with conservative update, ~50 LOC) | All count-min crates stale (probabilistic-collections 0.7.0 is 2020); first-party justified |
| Drift detectors | **First-party** | None (ADWIN + Page-Hinkley) | Python-only ecosystem; `neural-drift` undocumented algorithms — D-C7-3 MUST-BUILD |

**Heavy/learned tier (LATER, §15.7) — MUST-BUILD for streaming variants:**
Streaming iForest / HS-Trees / DenStream / CluStream / streaming-kmeans: **no maintained Rust
crate exists (verified 2026-06-27)**. `linfa` 0.8.1 (HEALTHY, 2025-12-23) + `extended-isolation-forest`
0.2.3 (STALE, 2022) cover only the batch/on-demand-train case. §15.7 already sequences this
LATER; this research confirms that sequencing is not optional — the heavy tier is genuine build,
not glue.

**Vendor-vs-build line:** depend on healthy, high-adoption mergeable-sketch crates
(`sketches-ddsketch`, `hyperloglogplus`, optionally `tdigests`); build first-party where crates
are stale (Welford/CGL/EWMA/count-min) or where the ecosystem is Python-only (drift detectors,
streaming clustering, streaming iForest).

[research/ml-behavior-analytics-depth-2026-06-27.md §Q1 §1.2; day-2-first toolkit summary]

---

### L-C7-2 — Verification Strategy: Monoid Laws + Kani Proofs + Seeded-Permutation Proptests

**For all streaming update functions, apply the following verification strategy:**

**Step 1 — Classify each sketch by order-sensitivity:**
- **Order-agnostic:** Welford/CGL, DDSketch, count-min, HLL — result is order-invariant over the
  multiset (tiny FP rounding variance excepted for Welford).
- **Order-bounded-ε:** t-digest — approximately order-agnostic with small compression-order
  differences.
- **Order-dependent:** EWMA, reservoir, streaming-clustering — result depends on arrival order.

Each sketch's spec must state its order-sensitivity classification explicitly. This makes the
verification target unambiguous before writing test harnesses.

**Step 2 — Verify mergeable sketches as commutative monoids:**
For all order-agnostic mergeable sketches (Welford/CGL, DDSketch, count-min, HLL), assert the
monoid laws via proptest:
- Associativity: `merge(a, merge(b,c)) == merge(merge(a,b), c)`
- Identity: `merge(a, identity()) == a`
- Commutativity (where applicable — HLL and count-min): `merge(a,b) == merge(b,a)`
- HLL idempotency: `merge(a,a) == a`

**Step 3 — Kani proofs for bounded-invariants (new VP siblings to VP-014/VP-015):**
For fixed small instances (small register count for HLL; small table size for count-min; bounded
loop for Welford), assert via Kani:
- **Bounded state size** — `size_of(sketch_state)` is constant after N updates regardless of
  stream length (anti-DoS invariant). This is the ML analog of VP-014 (size limit).
- **Monotonic count** — after k updates, `sketch.count() == k` (no decrement).
- **Count-min never-underestimates** — `est(x) >= true_freq(x)` (one-sided error guarantee).
- **HLL register monotonicity** — `M_{t+1}[j] >= M_t[j]` for all registers j.
- **Welford M2 non-negativity** — `M2 >= 0.0` after each update (FP caveat near zero; clamp in
  implementation).

No published Kani/CBMC case study verifies a streaming sketch. These proofs are novel ground,
consistent with prism's existing VP-014/VP-015 muscle.

**Step 4 — Order-dependence testing for order-agnostic and order-bounded-ε sketches:**
- **Seeded permutation proptest:** proptest generates a multiset; a seeded PRNG produces N
  permutations; assert final state identical (order-agnostic) or within ε (order-bounded-ε).
- **Reference-oracle differential:** exact stored-data oracle for variance/quantile/cardinality;
  assert sketch result is within the documented error bound.
- **Metamorphic relations:** `state(D)` == `state(permuted D)` for order-agnostic; `state(D)` ==
  `merge(state(D[:k]), state(D[k:]))` for mergeable; replay-doubles: `state(D twice)` vs
  `state(D)` is predictable.

**VP-NNN allocation:** the exact VP-NNN numbers and Kani vs proptest assignments are open questions
for architect/morph (see Open Questions). The VP-INDEX.md + verification-architecture.md +
verification-coverage-matrix.md must be updated in a single burst when VP-NNNs are assigned.

[research/ml-behavior-analytics-depth-2026-06-27.md §Q3, §3.1–3.2 LEAN]

---

### L-C7-3 — Primitive → Engine Compilation (PrismQL ML Functions)

**Each PrismQL ML primitive maps to a specific incremental aggregate over a bounded window:**

| PrismQL primitive | Streaming mechanism | Implementation notes |
|---|---|---|
| `PROFILE <entity> OVER <window>` | `GROUP BY entity` over a bounded time window; per-entity Welford/CGL or EWMA sketch in a windowed RocksDB state store | Window = RetentionCache hot window (§3.3) OR scoped federated pull (§15.2); per-entity sketch = the profile; state store = RocksDB model tier (D-C7-4 CF) |
| `ANOMALY_SCORE(...)` | z-score / residual-outlier / product-of-frequencies over the entity's profile | Welford-based z-score for numeric baselines; robust variant = (x − median) / MAD; product-of-per-field frequencies for multivariate |
| `BASELINE_DEVIATION(...)` | `(x − μ) / σ` over the incremental Welford/EWMA baseline | Inherits from PROFILE; robust variant uses median/MAD |
| `RARITY(...)` | Per-value frequency / cardinality relative to a population (count-min for frequency; HLL for cardinality); produces a rarity score (inverse frequency) | Scoped: global / per-entity / per-partition / windowed |
| `FIRST_SEEN(...)` | First observation of a value per entity — per-entity seen-set updated only on novel keys (Set-based; persisted in RocksDB under entity key-prefix) | No dedicated Rust crate needed; standard Set + RocksDB key |
| `PEER_OUTLIER(...)` — day-2-first | **Attribute-based peer group** (`GROUP BY <peer_attrs>` cohort); scored by z-score of entity's metric vs cohort robust mean/MAD | Pure incremental aggregate; transparent; no clustering model required. Clustering-based peer groups = LATER (heavier tier, needs streaming clustering MUST-BUILD) |

**Cost-bounding (the genuinely-novel control, thin vendor prior art):**
Every ML primitive in a query is subject to the existing mandatory time-bound + join-guard NFRs
(§5.3 / §12.2). Two additional controls are MUST-BUILD (no vendor prior art):

1. **GROUP-BY-entity cardinality cap on `PROFILE…OVER`:** a profile over an unbounded entity-set
   is the runaway risk (not the per-entity sketch math, which is bounded-state by construction per
   L-C7-2). Enforce a per-query cap on the number of distinct entity-groups a `PROFILE` expression
   may compute. Exceeding the cap = structured query error (E-QUERY-NNN), NOT a silent truncation.

2. **Per-query baseline-compute admission budget:** a per-query budget (wall-clock + RocksDB reads)
   for baseline-compute operations in ANOMALY_SCORE / PEER_OUTLIER / PROFILE. Exceeding the budget
   degrades gracefully (partial result with coverage disclosure) per the §15.1 cost-bounded model.

**Time-bound predicate pushdown first (Kusto pattern):** datetime predicates must be pushed down
BEFORE any profiling operation runs. The federated pull window is bounded before profiling starts.
This is a C3 capability-descriptor + query-planner obligation (see ADR-PROP-capability-descriptor-pushdown.md
D-C3-2 for the missing time-bound injection).

[research/ml-behavior-analytics-depth-2026-06-27.md §Q6, §6.1–6.4 LEAN]

---

## Open Questions (Architect / Morph-Time)

| # | Question | Status | Dependency |
|---|---------|--------|------------|
| **OQ-C7-1** | D-C7-1 macro-clustering drift empirical test (NARROWED by C7 FOLD 2026-06-27) — the broad "mergeable-only vs approximate-merge" fork is resolved (EWMA and reservoir now have mergeable-exact representations; see D-C7-1). The remaining empirical item is specifically: measure whether BIRCH CF-vector merges preserve macro-cluster fidelity under realistic security data + adversarial cross-shard skew (up to 10:1). Metrics: ARI, NMI, purity, centroid drift, CMM vs centrally-computed reference. Acceptance: ΔFNR ≤ ~0.5pp, ΔFPR ≤ ~2pp, ΔPR-AUC ≥ −0.01 across all adversarial regimes. | Open — deferred to edge-ML build milestone (sub-epic of E-ML-ONLINE-001 or E-SATELLITE-MESH) | Edge-ML implementation milestone |
| **OQ-C7-2** | VP-NNN number allocation for sketch verification targets (L-C7-2) — which get Kani proofs vs proptest-only; what are the VP-NNN numbers; when are they added to VP-INDEX.md, verification-architecture.md, and verification-coverage-matrix.md (must be a single atomic burst per VP-INDEX propagation obligation) | Open architect decision at morph | Morph-time, when VP-NNNs can be allocated without conflicting with Phase 3 live VP work |
| **OQ-C7-3** | Dual-rate + quarantine policy knobs — quarantine window length; fast/slow-model promotion threshold; human/agent-confirmation routing (§15.8 S3 agent dispatch on quarantine confirmation); per-tenant defaults vs global defaults. Are these runtime-configurable? (Ties to §15.5 policy-governed model artifact.) | Open design decision at morph | Morph-time, before E-ML-ONLINE-001 story decomposition |
| **OQ-C7-4** | Untrusted-BYO-model WASM sandbox capability grants — is the exact C4 sandbox configuration (connector WASM plugin, hook-level grants) appropriate for inference workloads, or does the model-plugin sandbox need different capability grants (more CPU budget, different memory ceiling, possibly different WASI permissions)? Both reuse wasmtime 46.0.1 Component-Model; the question is the grant configuration. | Open architect decision at morph (PIV-C7-3 in the interim: document the rationale and check whether the existing plugin host grants are sufficient for inference) | Morph-time, before D-C7-2 WASM-BYO backend implementation |
| **OQ-C7-5** | Entity-cardinality cap value + baseline-compute admission budget design — the genuinely-novel cost-bound (L-C7-3); no vendor prior art. Default values and the degradation behavior (partial result vs error code; which E-QUERY-NNN variant). | Open design decision at morph | Morph-time, before E-ML-PRIMITIVES-001 story decomposition |
| **OQ-C7-6** | Changelog retention policy parameters — keep-N-materializations, max-changelog-age, per-tenant vs global configuration. Bounded GC must avoid unbounded storage growth. | Open design decision at morph | Morph-time, before E-ML-ONLINE-001 story decomposition |
| **PIV-C7-1** | `ort` RC → stable: at morph time, check whether a stable `ort` 2.0.x has been released (currently 2.0.0-rc.12 as of 2026-06-27). If stable exists, pin stable. If still RC, budget explicitly for API churn. | Pre-implementation verification | Morph-time Cargo.toml dependency pinning |
| **PIV-C7-2** | **Edge-ML aggregation uses mergeable-exact sufficient-statistic representations (invariant).** At implementation time, verify: (a) EWMA at edge uses forward-decay `(U,V)` form, NOT scalar `z_t` — confirm landmark re-basing is designed in scope (not deferred); (b) reservoir at edge uses random-key/bottom-k (Efraimidis–Spirakis), NOT Algorithm-R + weighted-re-sample; (c) clustering at edge exposes additive CF-vectors `(N, LS, SS)`, NOT only scalar cluster centroids. A code path that stores scalar EWMA state at the edge and attempts a weighted-average merge is a **P1 violation** of D-C7-1 as updated by the C7 FOLD. | Representation correctness gate | Edge-ML implementation milestone (E-ML-ONLINE-001 satellite scope) |
| **PIV-C7-3** | **Coarsening ≠ privacy (invariant enforcement).** At morph time and at any story/PR that touches the ML aggregation path: confirm that no spec, story AC, or code comment cites representation coarsening or approximate-merge as a privacy mechanism or as satisfying a privacy requirement. Residency = aggregate-only movement (satisfied regardless of approximation level); privacy = a separate axis requiring local-DP on mergeable DP sketches if a formal guarantee is needed. Any claim of the form "approximate-merge for privacy" is a **P1 finding**. | Pre-implementation verification and ongoing adversary probe | Any story touching ML aggregation or satellite relay |

---

## Downstream SAP-1 Obligations (Not Actioned Here)

Several event types implied by C7 decisions will need BC-2.16.002 Canonical Structured Event
Catalog rows at morph time. Flagged here; NOT actioned (SAP-1 probe scope is per-story at
implementation time, not at ADR-PROP capture time).

- `event_type = "ml.model.update"` — emitted on each incremental model update (changelog append);
  fields: tenant_id, entity_class, schema_scope, model_type, update_sequence_no, changelog_offset.
  Audit role = model audit trail; recurrence = per update (bounded-rate by D-C7-3 update cap).
- `event_type = "ml.model.materialization"` — emitted when a periodic snapshot is materialized;
  fields: tenant_id, entity_class, schema_scope, materialization_id, changelog_offset_range.
  Audit role = model snapshot audit; recurrence = per materialization.
- `event_type = "ml.drift.detected"` — emitted when an ADWIN / Page-Hinkley detector fires;
  fields: tenant_id, entity_class, schema_scope, detector_type (ADWIN/PHT), drift_severity.
  Audit role = baseline integrity; recurrence = per detection.
- `event_type = "ml.quarantine.pending"` — emitted when anomaly-gated learning holds data in the
  quarantine buffer; fields: tenant_id, entity_class, quarantine_start_time, anomaly_score.
  Audit role = poisoning-resistance audit; recurrence = per quarantine-window-entry.
- `event_type = "ml.quarantine.promoted"` — emitted when quarantined data is promoted to the slow
  model (persists past quarantine window or confirmed by human/S3-agent); fields: tenant_id,
  entity_class, confirmation_method (timeout/human/agent).
  Audit role = poisoning-resistance audit; recurrence = per promotion.
- `event_type = "ml.anomaly_score.computed"` — emitted when an ANOMALY_SCORE / BASELINE_DEVIATION
  / PEER_OUTLIER / RARITY primitive fires in a DETECT; fields: tenant_id, entity_class,
  primitive_type, score, model_state_ref (materialization_id + changelog_offset).
  Audit role = detection explainability; recurrence = per detection-query result row.

All six categories above are flagged; BC-2.16.002 catalog rows are morph-time work.

---

## Honest Costs

| Item | Cost / Risk |
|------|-------------|
| **The heavy tier (streaming iForest / HS-Trees / streaming clustering) is real engineering, not glue.** | No maintained Rust crate exists (verified 2026-06-27). §15.7 already sequences this LATER; this research confirms that sequencing is not optional. Streaming clustering + streaming isolation forest are genuine multi-month builds. The day-2-first statistical tier is feasible with the toolkit above. |
| **Drift detectors are Python-only in the ecosystem.** | ADWIN, DDM, EDDM, Page-Hinkley, KSWIN are all in River / scikit-multiflow / Frouros. `neural-drift` Rust crate does not document its algorithms. MUST-BUILD both ADWIN and Page-Hinkley first-party. They are small but they are build, not depend. |
| **`ort` is still RC (2.0.0-rc.12).** | No stable 2.0 release as of 2026-06-27. Pin exactly; budget for API churn between rc.N versions. |
| **WASM-ML has a performance tax.** | CPU small/medium models are fine; SIMD/threads need build flags; GPU-in-WASM needs WebGPU (immature). The untrusted-BYO WASM backend is for isolation, not performance. |
| **This is the larger-build option.** | The human chose the full four-backend commitment over statistical+candle-first. The additional backends (ort, wasmtime, tract) each have their own integration surface. The full-backend commitment is a deliberate scope choice, not an oversight. |
| **The anomaly-gated-learning vs drift tension is genuinely unsolved in general.** | Dual-rate + quarantine is the best-evidenced mitigation. It shifts attacker cost; it does NOT eliminate boiling-frog risk. State this plainly in all downstream specs and UX. |
| **No published Kani/CBMC verification of a streaming sketch.** | The Q3 invariants are formally stateable; Prism would be charting novel ground. This is consistent with VP-014/VP-015 muscle but budgets real proof-development time. |
| **The per-update changelog + materialization mechanism is novel (no prior art in online-learning libraries).** | River/scikit-multiflow provide no per-detection snapshot prior art. The design is extrapolated from Flink changelog semantics applied to KB-scale sketch state. Soundness is reasonable by analogy; empirical confirmation is a build milestone. |
| **Vendor cost-bounding (per-query ML admission control) is under-documented in the industry.** | The entity-cardinality cap + baseline-compute budget (L-C7-3 OQ-C7-5) is Prism-original design with thin external precedent. It is the single most genuinely-novel cost-bound in the ML primitives design. |

---

## Alternatives Considered and Rejected

### Alternative A: Statistical + Candle First; ort and WASM-BYO Deferred (Lighter Backend Set)

Commit only the statistical tier + candle-core in day-2. Defer `ort` (trusted BYO ONNX) and
`wasmtime` (untrusted BYO WASM) to a future brief cycle.

**Rejected (D-C7-2) because the human explicitly chose the full-backend set:**
- The §15.7 pluggable-backends stance ("mirror the §11.1 secret-store BYO stance: built-in AND
  external/BYO") implies day-2 commitment to the BYO tier, not just the built-in tier.
- Deferring BYO to a future cycle would make the day-2 ML feature incomplete from a customer
  perspective: customers who have existing trained models cannot bring them until a future cycle.
- The larger-build option was chosen consciously; the honest costs are captured above. This is a
  DELIBERATE scope decision, not an oversight.

### Alternative B: Content-Addressed Per-Detection Snapshots (Lighter Replay Mechanism)

On each detection that fires, serialize the relevant per-entity `ModelState`, hash it (SHA-256),
write `model_snapshots[hash] → bytes` (dedup means unchanged state costs nothing), and store the
hash on the finding's replay link. No changelog; no periodic materialization.

**Rejected (D-C7-4) because the human chose per-update auditability:**
- Content-addressed per-detection snapshots enable replaying "the model state AT the decision" but
  NOT "every model update that led to the decision."
- The per-update changelog enables full audit trail reconstruction: an analyst can replay the exact
  sequence of model state transitions, not just the final state at detection time.
- The heavier mechanism (changelog + materialization) was chosen for the stronger auditability
  guarantee. The lighter mechanism is recorded here as the rejected alternative — it is a valid
  design for a system that requires only per-detection (not per-update) auditability.

### Alternative C: Approximate-Merge as Primary for Edge ML (No Restriction to Mergeable-Only Primitives)

Allow all primitives at the satellite/edge and merge them at central via documented approximations
(time-aware re-EWMA, weighted re-sampling for Algorithm-R reservoir).

**Rejected as PRIMARY posture (D-C7-1, superseded by C7 FOLD 2026-06-27):**
- The C7 FOLD establishes that this framing was a false dichotomy. The "non-mergeable primitives"
  that would have required approximate-merge (EWMA, reservoir) instead have **mergeable-exact**
  representation alternatives (forward-decay `(U,V)` for EWMA; random-key/bottom-k for reservoir).
  The correct answer is: use the mergeable-exact representation, not the approximate-merge.
- Scalar-state approximate-merge (scalar EWMA, Algorithm-R + weighted re-sampling) retains its
  status as a **constrained-edge fallback only** — for legacy edges where the representation
  change genuinely cannot be made. Error bounds for those fallbacks are extrapolated (not
  literature-settled) and must be measured before use.
- Approximate-merge is NOT the general answer; mergeable-exact representation is. This alternative
  is superseded at the general level by the fold, while the fallback case remains documented.

### Alternative D: Dedicated Model Registry (MLflow-Style) Separate From RocksDB

Store model artifacts in an external model registry (MLflow, BentoML, Hugging Face Hub) rather
than in RocksDB.

**Rejected (D-C7-4) because:**
- Prism's streaming sketch state is KB-scale (per-entity). A dedicated model registry designed for
  MB–GB neural model artifacts is the wrong fit for frequently-updated, tiny sketch state.
- The "model is a retention tier" framing (§15.3) makes RocksDB the literal answer: the model is
  a tier in the same data store as the hot cache. Using a separate system defeats this unification.
- In-process RocksDB gives low-latency per-entity state access. A network-round-trip model registry
  would add unacceptable latency to the per-detection scoring path.
- The per-tenant CF / key-prefix isolation (D-C7-4) is already the prism pattern for the existing
  19-CF RocksDB engine. The model tier follows the same pattern.

---

## Ripple Effects (Must Be Picked Up at Morph Time)

| Affected area | Ripple |
|---------------|--------|
| **E-ML-ONDEMAND-001** | L-C7-3 primitive→engine compilation table defines the streaming mechanism for each PrismQL ML primitive. The epic's ACs must include the GROUP-BY-entity cardinality cap + baseline-compute admission budget (OQ-C7-5) as mandatory cost-bound controls. Time-bound predicate pushdown first (Kusto pattern) must be an explicit AC. |
| **E-ML-ONLINE-001** | D-C7-3 dual-rate + quarantine design; D-C7-4 changelog + materialization; L-C7-1 statistical toolkit (vendor-vs-build line for Welford/CGL/EWMA/count-min/drift-detectors). ADWIN + Page-Hinkley MUST-BUILD. The honest caveat on poisoning resistance must appear in the epic definition and user-facing documentation, NOT just in internal spec. |
| **E-ML-PRIMITIVES-001** | L-C7-3 primitive→engine compilation table. PEER_OUTLIER day-2-first = attribute-based (clustering-based LATER). Entity-cardinality cap + admission budget (OQ-C7-5) are the novel cost controls. |
| **E-ML-ONLINE-001 satellite scope** | D-C7-1 as updated by C7 FOLD (2026-06-27): edge ML uses mergeable-exact representations broadly — forward-decay `(U,V)` for EWMA, random-key/bottom-k for reservoir, additive CF-vectors for clustering. PIV-C7-2 verifies representation correctness at implementation. Remaining empirical item: macro-clustering drift test (ARI/NMI/CMM). ADWIN at edge (ADWIN doubles as decay mechanism, does not require EWMA); `tract-onnx` 0.23.3 for the inference runtime at satellite. |
| **VP-INDEX.md + verification-architecture.md + verification-coverage-matrix.md** | L-C7-2 sketch invariants → new VP-NNN entries (OQ-C7-2). When VP-NNNs are allocated, the three documents MUST be updated in a single atomic burst per the VP-INDEX propagation obligation (CLAUDE.md source-of-truth invariants). Do NOT add VPs without updating all three in the same commit. |
| **BC-2.16.002 §Postconditions** | Six SAP-1 event type categories (§Downstream SAP-1 Obligations above) — morph-time BC work. |
| **ADR-PROP-dynamic-schema-connectors.md (C4)** | D-C7-2 WASM-BYO backend reuses the C4 WASM sandbox; OQ-C7-4 must confirm that the connector-plugin WASM grant configuration is appropriate for inference or produce a separate model-plugin-sandbox ADR at morph. |
| **§15.5 controls table** | D-C7-3 resolves the drift/decay/poisoning row with the dual-rate + quarantine design. The §15.5 table cell for "Adversarial poisoning" should reference D-C7-3 + the honest caveat at morph-time PO spec update. |
| **§2.4 tradeoff prose (G-26)** | This is a PO action (the three-ways-to-long-baseline reframe); flagged per the research instructions. Do NOT write it here. C7 confirms the "online-learn a model" path is viable; the online-ML softening of the §2.4 honest tradeoff is now fully grounded. |
| **matured-vision §16.4** | C7 decision block appended in-place (2026-06-27); C7 FOLD marked complete in-place (2026-06-27) — D-C7-1 resolved. |

---

## References

| Document | Role |
|---|---|
| `research/ml-behavior-analytics-depth-2026-06-27.md` | Primary C7 research basis (Q1–Q6, six sonar-deep-research calls at `reasoning_effort=high`; 13 live crates.io version-verifications) |
| `research/edge-ml-mergeability-depth-2026-06-27.md` | **C7 FOLD depth research** — mergeable-summaries formal taxonomy (Agarwal et al. PODS 2012); EWMA forward-decay escape hatch (Cormode–Shkapenyuk–Srivastava–Xu); random-key/bottom-k fully-mergeable substitute (Efraimidis–Spirakis, Cohen–Kaplan); BIRCH CF additivity theorem; privacy vs residency distinction; empirical test plan |
| `research/satellite-mesh-2026-06-26.md` | C2 satellite mesh — D-C2-12 hard-residency; partial-result fan-in; aggregate-only movement |
| `day2-design-decisions/ADR-PROP-dynamic-schema-connectors.md` | C4 WASM sandbox (D-C4-3) reused by D-C7-2 WASM-BYO model backend |
| `day2-design-decisions/ADR-PROP-detection-engine-depth.md` | C6 — §14.5 replay-link machinery reuse; detection state RocksDB CF |
| `day2-design-decisions/ADR-PROP-storage-engine-taxonomy.md` | C1 — RocksDB as the §3.3 hot/model tier |
