---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: "day-2 vision SIDE-ANALYSIS (OUT-OF-BAND; separate from the live VSDD factory pipeline)"
scope: "DEPTH pass on OPEN implementation questions for §15 (On-Demand ML & Behavior Analytics). §15 framing is HUMAN-CONFIRMED and NOT re-litigated here."
source_of_truth_read: ".factory/specs/matured-vision-day2-requirements.md §15 (lines ~2073-2196)"
related_epics: [E-ML-ONDEMAND-001, E-ML-ONLINE-001, E-ML-PRIMITIVES-001]
related_gaps: [G-22, G-23, G-24, G-25]
leans_are: "discussion input only — NOT decisions. No ADR is created or amended by this file."
---

# On-Demand ML & Behavior Analytics — DEPTH Research (OPEN implementation questions)

> **CAPTURE-ONLY.** This is an out-of-band side-analysis artifact. It does NOT modify STATE.md, SESSION-HANDOFF.md,
> the ADR registry, any live spec/BC/story, or any prior research file. It does not get git-added or committed by this run.
> The §15 *framing* (on-demand/ephemeral ML; model-as-retention-tier; online learning; single-pass-streaming-only;
> ML-as-PrismQL-primitives; two model tiers; pluggable AI-opaque per-tenant backends) is **HUMAN-CONFIRMED 2026-06-25**
> and is treated as settled. This file researches **HOW** to build the genuinely-open pieces. All "Lean" rows are
> discussion input for the architect/PO, not decisions.

## Provenance & non-contradiction check

Read `.factory/specs/matured-vision-day2-requirements.md` §15.0–§15.10 before writing. The questions below are scoped to the
*open* mechanics that §15 explicitly leaves to architecture (it names the candidate algorithms, the model-as-tier framing,
and the controls, but not the concrete algorithm/crate selection, the persistence/snapshot mechanics, the verifiable update
semantics, the drift/poison engineering, the backend sandbox design, or the primitive→engine compilation path). Nothing
below contradicts §15; where this research would refine §15 prose (e.g., the §2.4 tradeoff text, G-26), that is flagged as a
PO action, not done here.

**Confidence legend:** [WEB] = grounded in a cited web finding; [CRATES.IO-VERIFIED 2026-06-27] = version checked live against
the crates.io API today; [model-knowledge] = from model training data, flagged; [INCONCLUSIVE] = research could not settle.

---

## Q1 — The streaming / single-pass algorithm toolkit (concrete + Rust-verified)

§15.4 names the algorithm families. This is the concrete production algorithm + maturity + numerical-stability + **mergeability**
(critical for distributed/satellite partial-aggregates per §3.2 Purdue-edge baselining) for each, with Rust-crate state verified.

### 1.1 Per-primitive algorithm table

| Primitive | Production algorithm | Numerical stability | Mergeable? (partial-aggregate combine) | Order-dependent? |
|---|---|---|---|---|
| **Streaming mean/variance** | Welford's online algorithm; **Chan–Golub–LeVeque (CGL)** pairwise form for merge [WEB] | High — accumulates `M2` as squared deviations from running mean, avoids catastrophic cancellation of the naive `Σx² − (Σx)²/n` [WEB] | **YES, exactly.** CGL: `n=nA+nB`, `μ=μA+Δ·nB/n`, `M2=M2A+M2B+Δ²·nA·nB/n` where `Δ=μB−μA`. Associative reduce-tree mergeable. [WEB] | No (mathematically order-agnostic over the multiset); tiny FP rounding variance under reorder [WEB] |
| **EWMA mean/variance** | `z_t = λ·x_t + (1−λ)·z_{t−1}`; EWMA-variance via EWMA of squared deviations / SPC S²-chart [WEB] | High — linear recurrence with coeffs in [0,1], no large-equal subtraction [WEB] | **NO simple additive merge.** State is intrinsically time-ordered; merging requires time-aware reconstruction or coarse-resolution re-EWMA. [WEB] | **YES — inherently time/order-dependent** (recent points weighted more) [WEB] |
| **Online z-score** | `z=(x−μ)/σ` over Welford or EWMA baseline [WEB] | Inherits baseline stability; regularize/clamp σ near zero [WEB] | Pointwise z not mergeable, but its **baseline (μ,σ) is** (via CGL) [WEB] | Follows baseline |
| **t-digest (quantiles)** | Dunning t-digest: sorted weighted centroids, 1-D k-means-style clustering, compression δ, tail-accurate [WEB] | High — full FP resolution emphasized in design; weight accumulation in rank order [WEB] | **YES (approximate).** Merge = combine + re-sort + re-compress centroids; map-reduce friendly. Minor compression-sequence differences. [WEB] | Approximately order-agnostic over the multiset (small compression-order differences) [WEB] |
| **DDSketch (quantiles, rel-error)** | Log-bucketed counts, `γ=(1+α)/(1−α)`, bucket `k=⌊log_γ x⌋`; relative-error guarantee [WEB] | Stable for realistic ranges; clamp pathological values [WEB] | **YES, exactly.** "First fully-mergeable, relative-error quantile sketch." Merge = add aligned bucket counts (same α). [WEB] | No — depends only on multiset [WEB] |
| **Count-Min sketch (frequency)** | d×w counter table, d hash functions; estimate = `min_j table[j,h_j(x)]`; one-sided (never underestimates) [WEB] | Trivial integer add/min; watch i64 overflow [WEB] | **YES, exactly.** Merge = element-wise add (same hashes/dims). "Conservative update" variant reduces overestimate. [WEB] | No — additive/commutative over the multiset [WEB] |
| **HyperLogLog (cardinality)** | Flajolet HLL; registers store max leading-zeros; harmonic-mean estimate; std-err ≈ `1.04/√m` [WEB] | Stable; bias correction at low/high range [WEB] | **YES, exactly.** Merge = register-wise **max**. Identical to single-pass over the union. [WEB] | No — registers depend only on the set of hashes [WEB] |
| **Reservoir sampling** | Vitter Algorithm R (uniform); A-Res / A-ExpJ (weighted) [WEB] | Trivial (RNG + integer compare); RNG quality matters [WEB] | **NO simple merge.** Per-shard reservoirs do NOT combine into a globally-uniform sample of size k without weighted re-sampling. [WEB] | Distribution order-independent on a single stream; reservoir *contents* are not additively mergeable [WEB] |
| **Online isolation forest** | Streaming iForest variants: reservoir-fed forest rebuild, sliding-window, or incremental trees [model-knowledge — search results did not surface a canonical streaming-iForest paper or a maintained Rust crate] | Comparisons/counts only; stability concern is drift + tree size [model-knowledge] | "Ensemble-of-ensembles" union of trees ≠ centralized training; not cleanly mergeable [model-knowledge] | Yes for windowed/drift-handling variants [model-knowledge] |
| **Half-space trees (HS-Trees)** | Streaming HS-Trees / one-class streaming anomaly (random half-space partitions, mass profiles over a reference + latest window) [WEB — named in §15 family and in the streaming-anomaly tail] | Mass counts only [WEB] | [INCONCLUSIVE] — not established by sources | Yes (window-based) |
| **Streaming clustering** | DenStream (fading micro-clusters, `w(t)=w0·2^{−λ(t−t0)}`), CluStream (micro-cluster pyramid), sequential/streaming k-means [WEB] | Centroid/weight updates stable [WEB] | Partial — micro-clusters can sometimes be combined, but fading weights are time-tied → **time/order-dependent** [WEB] | **YES** — fading + arrival-order clustering [WEB] |

### 1.2 Rust crate state — VERIFIED LIVE against crates.io 2026-06-27

| Primitive | Crate | Max version | Recency / liveness | Disposition |
|---|---|---|---|---|
| Mean/variance, EWMA, z-score, rolling stats | **`online-statistics`** | **0.2.6** [CRATES.IO-VERIFIED 2026-06-27] | **STALE — last publish 2022-09-10, ~1.4k recent downloads.** Low maintenance signal. | Usable as reference; **lean = vendor/lift the Welford+CGL+EWMA math (tiny, ~100 LOC) into a first-party `prism` module** rather than depend on a stale crate. Eat-our-own-dogfood + we need redacted-Debug/`#[non_exhaustive]` discipline anyway. |
| DDSketch | **`sketches-ddsketch`** | **0.4.0** [CRATES.IO-VERIFIED 2026-06-27] | **HEALTHY — updated 2026-03-18, ~14.4M total downloads.** Direct port of Datadog Go DDSketch; optional serde. | **Strong production candidate** for rel-error quantiles + fully-mergeable. |
| t-digest | **`tdigest`** (0.2.3, 2021) vs **`tdigests`** (Andy Lok) | `tdigest`=**0.2.3** (stale 2021); **`tdigests`=1.0.1** [CRATES.IO-VERIFIED 2026-06-27] | `tdigests` **HEALTHY — updated 2025-12-16, ~429k recent downloads**; explicitly documents merge-from-partitions. | **Prefer `tdigests` 1.0.1** over the older `tdigest` if absolute-error tail quantiles are wanted alongside DDSketch. |
| Count-Min + HLL + reservoir (bundle) | **`streaming_algorithms`** | **0.3.3** [CRATES.IO-VERIFIED 2026-06-27] | Updated 2025-10-14 but **low downloads (~7k recent)** — niche. SIMD count-min (conservative update) + HLL + reservoir + top-k. | Viable but **low adoption**; evaluate vs split crates below. |
| HyperLogLog (standalone) | **`hyperloglogplus`** | **0.4.1** [CRATES.IO-VERIFIED 2026-06-27] | Stale publish (2022-06) but **~2.4M recent downloads — high adoption, battle-tested.** HLL + HLL++. | **Strong HLL candidate** (higher adoption than the bundle). |
| Count-Min (standalone) | `count-min-sketch` / `probabilistic-collections` | `probabilistic-collections`=**0.7.0** (2020) [CRATES.IO-VERIFIED 2026-06-27] | **STALE (2020).** | Count-min is ~50 LOC; **lean = first-party impl** with i64 saturation + conservative-update, given staleness of all options. |
| Streaming clustering (DenStream/CluStream/streaming-kmeans) | — | — | **No maintained dedicated Rust crate surfaced.** `linfa-clustering` 0.8.1 has batch KMeans/DBSCAN/GMM/OPTICS (not streaming). | **MUST-BUILD** for the streaming variants (later/heavier tier). |
| Online isolation forest / HS-Trees | `extended-isolation-forest` (batch) | **0.2.3** [CRATES.IO-VERIFIED 2026-06-27] | **STALE (2022).** Batch Extended-iForest, optional serde. NOT streaming. | Batch iForest only; **streaming variant is MUST-BUILD** (later tier). |

**Day-2-first toolkit (lightweight statistical tier, §15.7) — all production-grade, mostly mergeable:**
Welford+CGL (first-party) · EWMA (first-party) · online z-score (first-party) · `sketches-ddsketch` 0.4.0 · `tdigests` 1.0.1 ·
`hyperloglogplus` 0.4.1 · count-min (first-party). The heavier learned tier (streaming iForest/HS-Trees/streaming clustering)
is **MUST-BUILD** in Rust — no maintained streaming crate exists today.

> **Lean (Q1):** the day-2-first tier is buildable now from healthy crates + ~300 LOC of first-party single-pass math; the
> heavy tier should be explicitly sequenced LATER (§15.7 already says this) because it is genuine build, not glue. Prioritize the
> **mergeable** primitives (Welford/CGL, DDSketch, count-min, HLL) for the satellite/Purdue-edge partial-aggregate story (§3.2) —
> EWMA, reservoir, and streaming-clustering are **not cleanly mergeable** and should be marked as edge-local-only or merged via
> documented approximations.

---

## Q2 — Model-as-retention-tier mechanics (serialize / store / snapshot / isolate)

§15.3 makes the model a first-class retention tier; §15.5 requires model versioning/snapshots so a finding's replay link points
at "model state AS OF the decision." OPEN: where it lives, how it serializes, how snapshots are made cheaply.

### 2.1 Serialization
- All the day-2 sketches are **compact** (tens of bytes to a few KB per entity/feature) [WEB] — radically smaller than NN models;
  this changes the calculus toward aggressive per-detection snapshotting.
- Observed production pattern: **versioned binary formats** (schema-version header) + serde with a **binary codec (bincode)** [WEB].
  `sketches-ddsketch` and `tdigests` expose optional serde; HLL libraries serialize register arrays with a schema-version byte [WEB].
- Count-min/HLL = fixed-shape arrays → contiguous binary blob; t-digest/DDSketch = header + variable centroid/bucket array [WEB].

> **Lean (2.1):** standardize a first-party `ModelState` serde envelope = `{schema_version, model_type, tenant_id (newtype),
> schema_scope (OCSF/native, §13.6), payload}` serialized with bincode, with redacted `Debug` (credential-safety discipline) and
> `#[non_exhaustive]`. Embed `schema_version` so format evolution doesn't strand old snapshots [WEB].

### 2.2 Store: RocksDB (the §3.3 hot tier) vs a dedicated model store
RocksDB feature fit is strong [WEB]: column families = logical sub-DBs (per-tenant isolation), snapshots = point-in-time reads,
Checkpoint API = durable consistent snapshot via SSTable **hard-links** (cheap), BackupEngine = incremental shared-file backups.

| Aspect | RocksDB embedded (the §3.3 tier) | Dedicated model registry (MLflow-style) |
|---|---|---|
| Latency for frequent streaming updates | In-process, low-latency [WEB] | Network/REST, occasional-load oriented [WEB] |
| Snapshot granularity | Fine (DB/CF-level snapshots + checkpoints) [WEB] | Coarse (manual model versions) [WEB] |
| Per-tenant isolation | Column families OR key-prefix [WEB] | Indirect (names/tags) [WEB] |
| Streaming-state fit | **High** | Low (registering millions of micro-versions impractical) [WEB] |

**Per-tenant CF caveat [WEB]:** RocksDB degrades badly at *thousands* of column families (config-file write + validation overhead).
A strict CF-per-tenant mapping does not scale to many tenants.

> **Lean (2.2):** **store model state in RocksDB (the §3.3 hot tier), NOT a separate store** — keeps the "model is a retention tier
> like the cache" framing literal and avoids a new subsystem. For per-tenant isolation use a **hybrid**: dedicated CF for a small
> number of large/high-isolation tenants; **key-prefix partitioning** (`tenant_id:schema_scope:entity_class:model_type:...`) within
> shared CFs for the long tail [WEB]. This matches prism's existing 19-CF RocksDB layout philosophy rather than fighting it.

### 2.3 Cheap snapshotting for replay/explainability (§15.5)
Candidates [WEB]: (a) RocksDB **snapshot** = point-in-time read view, but **ephemeral (does not survive restart)** — good for
in-flight reconstruction, not durable retention; (b) RocksDB **Checkpoint** = durable, hard-link-cheap, but DB/CF-level (coarse for
per-detection); (c) **content-addressed model versions** — hash the serialized `ModelState`, store by hash, detections reference the
hash; gives dedup (identical state reused) + tamper-evident integrity [WEB, extrapolated]; (d) **Flink-style state-changelog +
periodic materialization** — append per-update deltas, materialize consolidated snapshots periodically, reference (materialization-id
+ changelog-offset) [WEB]. River/scikit-multiflow give NO explicit per-detection snapshot prior art — this is an innovation area [WEB].

> **Lean (2.3):** because day-2 model state is KB-scale, **content-addressed snapshots (option c)** are the cheapest correctness-clean
> mechanism: on each detection that fires, serialize the relevant per-entity `ModelState`, hash it (e.g., SHA-256), write
> `model_snapshots[hash]→bytes` (dedup means unchanged state costs nothing), and store the hash on the finding's replay link (§14.5).
> Add a **bounded retention policy** (keep only hashes referenced by retained findings, per-tenant GC). This satisfies §15.5
> "model state as of the decision" without per-detection full copies. The Flink changelog model (option d) is the fallback if
> per-update replay (not just per-detection) is later required.

---

## Q3 — Online-learning update semantics + verification (§15.5 wants bounded + spec'd update fn with Kani/VP targets)

OPEN: what invariants are formally stateable for `update(state, x) → state'`, and how order-dependence is tested deterministically.

### 3.1 Formally stateable invariants (candidate VP/Kani targets)
The mergeable-sketch literature frames these as **monoid / commutative-monoid** laws, which is exactly what enables property testing
of merge correctness [WEB]. Invariant catalog grounded in [WEB]:

| Invariant | Applies to | Formal statement | Verification path |
|---|---|---|---|
| **Monotonic count** | all additive sketches | after k updates, `count == k` (no decrement) | Kani assert post-update; proptest |
| **Bounded state size** | all sketches | state size = f(params), independent of stream length (no unbounded alloc) — anti-DoS | Kani (no growth in loop); the existing `#[non_exhaustive]`/size discipline |
| **Count-min never-underestimates** | count-min | `est(x) ≥ true_freq(x)` (one-sided error) | proptest vs exact reference map [WEB] |
| **HLL register monotonicity** | HLL | `M_{t+1}[j] ≥ M_t[j]` ∀j; bounded by ρ width | Kani assert (small m, unroll registers) [WEB] |
| **Welford M2 non-negativity** | variance | `M2 ≥ 0` after each update | Kani (FP caveats near zero) [WEB] |
| **Merge = monoid** | mergeable sketches | `merge(a,merge(b,c))==merge(merge(a,b),c)`; `merge(a,id)==a`; HLL/count-min also commutative + (HLL) idempotent | proptest associativity/commutativity/identity [WEB] |
| **Idempotent replay** (explicit per sketch) | additive sketches NOT idempotent (replay doubles); design must dedup-before-update | property-replay test: state(D) vs state(D twice) | proptest replay [WEB] |
| **Order-sensitivity bound** | EWMA, t-digest, streaming-cluster | result under any permutation differs by ≤ ε | seeded permutation proptest (below) [WEB] |

Tooling reality [WEB]: Kani (CBMC frontend) supports `--unwindset`/`--show-loops` to verify small-register/small-table instances;
**no published Kani/CBMC case study verifies a sketch** — so prism would be charting it (matches prism's existing Kani VP-014/VP-015 muscle).

### 3.2 Deterministic order-dependence testing
[WEB] No off-the-shelf toolkit exists; the established synthesis is: **seeded permutation testing** (proptest generates a multiset;
a seeded PRNG produces N permutations; assert final state identical for order-agnostic sketches OR within ε for EWMA/t-digest) +
**reference-oracle differential testing** (exact stored-data oracle for variance/quantile/cardinality, assert sketch within error
bound) + **metamorphic relations** (replay→same, shuffle→same/within-ε, partition+merge→union, duplicate→predictable). proptest +
proptest-state-machine are the Rust vehicles [WEB].

> **Lean (Q3):** make the day-2 update functions **`Monoid`-shaped where mergeable** and assert the monoid laws + one-sided/monotonic
> invariants in proptest, with **Kani proofs for bounded-state + monotonic-count + count-min-never-underestimate on small fixed
> instances** as new VP entries (siblings to VP-014/VP-015). Test order-dependence with **seeded permutation + reference-oracle
> differential** harnesses. Explicitly classify each sketch as order-agnostic / order-bounded-ε / order-dependent in its spec so
> the verification target is unambiguous. This directly answers §15.5 "bound + spec the update function; candidate VP/Kani targets."

---

## Q4 — Concept drift + decay + poisoning resistance (concrete techniques)

§15.5 lists drift/decay/poisoning as IN-SCOPE controls. Concrete techniques + the core tension.

### 4.1 Drift detection algorithms [WEB]
| Algorithm | Monitors | Drift emphasis | False-positive control | Rust today |
|---|---|---|---|---|
| **ADWIN** | mean of numeric stream over two adaptive subwindows W0/W1 | abrupt + gradual | significance δ (Hoeffding-bound) | **No named Rust impl found**; `neural-drift` crate exists but algorithms unspecified [WEB] |
| **DDM** | classifier error-rate + std-dev vs historical min | abrupt + some gradual | warn at `p+s≥pmin+2·smin`, change at `+3·smin`; min-sample guard | scikit-multiflow (Py) only [WEB] |
| **EDDM** | mean DISTANCE BETWEEN errors | gradual (earlier than DDM) | ratio vs `p'max+2·s'max`, α=0.95/β=0.9 | River (Py) only [WEB] |
| **Page–Hinkley** | cumulative deviation from mean | gradual | threshold λ + offset | conceptual only; trivial to impl [WEB] |
| **KSWIN** | KS distance between two windows R,W (non-parametric) | general distributional | α (recommend <0.01); `dist>√(−ln α / r)` | River + scikit-multiflow (Py) only [WEB] |

**Rust gap [WEB]:** the drift-detector ecosystem is Python-dominated (River, scikit-multiflow, Frouros). The only Rust crate,
`neural-drift`, does not document which algorithms it implements → **treat drift detectors as MUST-BUILD in Rust** (each is small:
Page–Hinkley and ADWIN are the highest-value, ADWIN because it self-tunes window size = doubles as the forgetting mechanism).

### 4.2 Decay / forgetting [WEB]
EWMA decay (`λ` tunes memory) · sliding vs landmark vs **adaptive (ADWIN-driven) windows** · time-based exponential decay
`exp(−λΔt)` + adaptive forgetting factors · **DenStream fading factors** `w(t)=w0·2^{−λ(t−t0)}` attached to micro-clusters (decay
on the *model component*, not just raw data). Trade-off [WEB]: fast decay/small window = adapts fast but **poisons easily**;
slow decay/large window = poison-resistant but lags legitimate drift.

### 4.3 Poisoning / "boiling-frog" resistance [WEB]
- **Bounded update rates / learning-rate caps** — cap how far one update (or one window) can move the baseline.
- **Robust estimators** — median, **MAD** (median-absolute-deviation) instead of mean/σ, trimmed mean, **Huber/M-estimators** — far
  less movable by a few crafted outliers. (Streaming median/MAD via quantile sketch = ties back to t-digest/DDSketch in Q1.)
- **Anomaly-gated learning** — do NOT update the model from data already flagged anomalous.
- Threat-model note [WEB]: drift detectors (ADWIN/Page–Hinkley) cannot themselves distinguish adversarial drift from benign;
  attackers can ride just below detector thresholds → robustness must come from the estimator + gating, not the detector alone.

### 4.4 THE CORE TENSION (anomaly-gated learning ⟂ concept-drift adaptation)
[WEB] explicitly frames the conflict: refusing to learn from anomalous data starves the model of the very signal that legitimate
drift first presents as. The literature's resolutions (all surfaced [WEB] / [model-knowledge for dual-rate naming]):
1. **Dual-rate / dual-window models** — a fast-adapting and a slow-anchoring model; promote fast→slow only on persistence (a
   sustained "anomalous" trend that survives ⇒ drift, not attack).
2. **Quarantine / delayed incorporation** — flagged-anomalous data is held, not immediately learned; incorporated only after it
   recurs / persists / is confirmed benign.
3. **Human-in-the-loop confirmation** — analyst (or S3 agent, §15.8 agent-native) confirms a drift before the baseline absorbs it.
4. **Robust-estimator slow absorption** — robust estimators naturally damp single shifts so persistent change still eventually moves
   the baseline while one-shot poison does not.

> **Lean (Q4):** day-2-first = **robust estimators (median/MAD) + bounded per-window update rate + anomaly-gated learning** as the
> default baseline math (cheap, no detector needed). Add **ADWIN** (self-tuning window doubles as decay) + **Page–Hinkley** as the
> first two Rust drift detectors (MUST-BUILD, small). Resolve the core tension with a **dual-rate + quarantine** design: fast model
> flags, slow model only absorbs drift that *persists past a quarantine window* (or is human/agent-confirmed). This makes
> "boiling-frog" expensive (attacker must sustain the shift across the slow window AND past quarantine) while still adapting to
> legitimate change — and it is agent-native (§15.8) by routing confirmation to the S3 agent.

---

## Q5 — Pluggable model backends (built-in + external/BYO), AI-opaque, per-tenant, sandboxed

§15.7 mirrors the §11.1 secret-store BYO stance. OPEN: which Rust inference stack, and how to sandbox untrusted external models.

### 5.1 Rust ML/inference crate state — VERIFIED LIVE 2026-06-27
| Crate | Role | Max version | Liveness | Ephemeral on-demand fit |
|---|---|---|---|---|
| **`candle-core`** (Hugging Face) | minimalist tensor + NN inference; **explicit "make serverless inference possible" goal**; safetensors; WASM-compilable | **0.11.0** [CRATES.IO-VERIFIED 2026-06-27] | **HEALTHY — updated 2026-06-26, ~2.1M recent downloads** | **Best fit for ephemeral inference** — small binaries, fast cold start, WASM-able [WEB] |
| **`ort`** (ONNX Runtime binding) | production BYO-ONNX inference; CUDA/TensorRT/OpenVINO EPs; 3–5× faster / 60–80% less mem than Python [WEB] | **2.0.0-rc.12** [CRATES.IO-VERIFIED 2026-06-27] | **HEALTHY but still RC — updated 2026-03-05, ~4.4M recent downloads**; no stable 2.0 yet | **Best fit for BYO external models** (any framework → ONNX); heavier runtime → microservice/longer-lived ephemeral, not ultra-short [WEB] |
| **`tract-onnx`** (Sonos) | tiny pure-Rust ONNX/NNEF inference; edge/embedded; pulsified streaming inference | **0.23.3** [CRATES.IO-VERIFIED 2026-06-27] | **HEALTHY — updated 2026-06-19, ~835k recent downloads**; ~85% ONNX backend tests pass [WEB] | **Best fit for satellite/Purdue-edge (§3.2)** — tiny self-contained runtime, easy WASM, no big C++ dep [WEB] |
| **`linfa`** (+ `linfa-clustering`) | classical ML ("scikit-learn of Rust"): KMeans/DBSCAN/GMM/OPTICS, LARS, RandomForest, AdaBoost (BATCH) | **0.8.1** [CRATES.IO-VERIFIED 2026-06-27] | **HEALTHY — updated 2025-12-23, ~297k recent downloads** | **Best fit for on-demand classical TRAINING** (cheap, pure-Rust, trains+infers in one ephemeral job) [WEB] — but batch, not streaming |
| **`extended-isolation-forest`** | batch Extended iForest anomaly, optional serde | **0.2.3** [CRATES.IO-VERIFIED 2026-06-27] | **STALE (2022)** | batch only; on-demand-train OK; streaming variant MUST-BUILD [WEB] |
| **`burn`** | full pure-Rust DL train+infer; backend-agnostic (CPU/CUDA/Metal/WGPU/WASM); JIT | (healthy per [WEB]; not version-pinned here) | active [WEB] | best when **training in Rust** is required across backends; heavier init [WEB] |

### 5.2 Pluggable / sandboxed backend interface
[WEB] Trait-based backend abstraction = AI opacity at the code level (engine sees only the trait, not internals):
```
trait ModelBackend { fn load(&mut self, spec) -> Result<()>; fn infer(&self, inputs) -> Result<outputs>; fn train(&mut self, data) -> Result<()>; }
```
Concrete impls wrap candle/ort/tract/linfa/burn. For **untrusted external/BYO models**, in-process `dlopen`-style plugins give NO
isolation; the strong-isolation answer [WEB] is **WebAssembly via `wasmtime` + the Component Model + WASI 0.2** (WIT-defined typed
interfaces, capability-restricted). **`wasmtime` max version 46.0.1, updated 2026-06-24, ~6.7M recent downloads** [CRATES.IO-VERIFIED 2026-06-27]
— very healthy. ML-in-WASM is real but constrained [WEB]: ONNX Runtime Web and candle/tract/burn all compile to WASM; SIMD + threads
require explicit build flags; GPU in WASM needs WebGPU (still maturing); memory is bounded — so WASM-sandboxed ML inference works for
CPU small/medium models, with a perf tax.

This maps directly onto prism's existing **C4 WASM plugin sandbox** decision (the 52-plugin factory-dispatcher hook chain is WASM;
prism already runs untrusted-ish plugins as WASM) — so "external models as WASM plugins" is architecturally consistent, not novel.

> **Lean (Q5):** define a first-party `ModelBackend` trait (AI-opaque by construction — backend never receives raw creds, mirrors
> AD-017 + §11.1 BYO secret-store stance). Built-in backends: **candle (built-in learned tier) + the Q1 first-party statistical
> sketches (statistical tier)**. BYO/external tier: **`ort` for ONNX models that the platform trusts (process-isolated), and
> `wasmtime` Component-Model WASM plugins for UNTRUSTED external models** (capability-restricted, per-tenant, sandboxed — reuse the
> existing C4 WASM sandbox pattern). Use **`tract`** for the satellite/Purdue edge (§3.2) tiny-runtime case. Per-tenant isolation =
> the same RocksDB key-prefix/CF model from Q2 plus per-tenant WASM instance/store. Sequence: statistical + candle first; ort + WASM-BYO later.

---

## Q6 — Primitive → engine compilation (PrismQL ML primitives → streaming algorithms over the window, inside DETECT)

§15.6 exposes ML as `ANOMALY_SCORE`, `RARITY`, `FIRST_SEEN`, `BASELINE_DEVIATION`, `PEER_OUTLIER`, `PROFILE <entity> OVER <window>`,
usable inside `DETECT` (§14). OPEN: how each compiles to a streaming aggregate over the RetentionCache window / federated pull,
how `PROFILE…OVER` builds a per-entity baseline incrementally, how `PEER_OUTLIER` defines a peer group, and cost-bounding (§5.3).

### 6.1 The cross-engine execution model (how the industry does it) [WEB]
Every surveyed engine (ksqlDB, Spark Structured Streaming, Splunk SPL, Kusto/Sentinel, Elastic ML) uses the SAME shape:
**incremental aggregate keyed by entity, maintained per bounded window, feeding an anomaly/rarity score.** Concrete mappings:

| PrismQL primitive | Maps to | Industry precedent [WEB] |
|---|---|---|
| `PROFILE <entity> OVER <window>` | `GROUP BY entity` over a bounded time window, maintaining an incremental per-entity sketch (Welford/EWMA/t-digest) in a per-key state store | ksqlDB `WINDOW TUMBLING…GROUP BY user`; Kusto `make-series` per entity; Spark StateStore per session-key |
| `ANOMALY_SCORE(...)` | z-score / residual-outlier / product-of-frequencies over the profile | Splunk `anomalydetection` = product of per-field frequencies (histogram for numeric, count/N for categorical); Kusto `series_decompose_anomalies.ad_score` = residual + Tukey via `series_outliers`; Spark GMM `max_prob` + `approxQuantile` threshold |
| `BASELINE_DEVIATION(...)` | `(x − μ)/σ` over the incremental Welford/EWMA baseline | same baseline-then-deviation pattern as Kusto `series_decompose` baseline + residual |
| `RARITY(...)` | per-value frequency/cardinality relative to a population (global / per-entity / per-partition / windowed) | Splunk `rare` (least-common values in the search window); Elastic ML `rare` function (per-bucket, per-partition, **highly sensitive to bucket time**) |
| `FIRST_SEEN(...)` | first observation of a value per entity = `MIN(ts)` per (entity,value) OR a per-entity seen-set updated only on novel keys | implemented via `MIN`/`earliest` or stateful seen-set; no engine has a dedicated function (idiom) |
| `PEER_OUTLIER(...)` | score entity vs the distribution of its peer group (z-score/percentile of entity's metric within the cohort) | Sentinel UEBA "peer group analysis"; Splunk UBA peer groups = "patterns of similar behavior" |

**Internal algorithm opacity caveat [WEB]:** none of these products document *which* streaming algorithm backs the score (Welford vs
t-digest etc.) — they document the logical semantics. So prism gets to CHOOSE the algorithm (from Q1) and is the more-transparent
system if it documents the mapping.

### 6.2 `PROFILE…OVER` incremental baseline
[WEB] Per-entity profile = per-key state in a state store (RocksDB-backed in ksqlDB/Spark), updated incrementally per event,
truncated/reset by the window. Maps cleanly to prism: the per-entity sketch (Q1) is the profile; the window is the
RetentionCache hot window (§3.3) or the scoped federated pull (§15.2); the state store is the RocksDB model tier (Q2).

### 6.3 `PEER_OUTLIER` — how a peer group is DEFINED [WEB]
Three definition strategies surfaced:
1. **Attribute-based grouping** — peers = same department/role/geo/device/app (a `GROUP BY` cohort). Cheapest, most transparent.
2. **Clustering-based** — peers = same behavioral cluster (KMeans/GMM on per-entity features; Splunk UBA "patterns of similar behavior").
3. **Cohort-based** — peers = shared exposure (created same time, same software stack).
Scoring = z-score / percentile of the entity's metric within the peer-group distribution [WEB]. Vendor docs do NOT give exact
peer-scoring formulas (opaque) [WEB] — prism chooses.

> **Lean (6.3):** day-2-first `PEER_OUTLIER` = **attribute-based peer group** (`GROUP BY <peer_attrs>` cohort) scored by z-score of the
> entity vs the cohort's robust mean/MAD — it's a pure incremental aggregate, transparent, cheap, and needs no clustering model.
> Clustering-based peer groups are a LATER (heavy-tier) extension once streaming clustering (Q1 MUST-BUILD) lands.

### 6.4 Cost-bounding (§5.3 mandatory time-bound; §12.2 join-guard) — baseline can't trigger a runaway fetch
Industry techniques surfaced [WEB]: **time-bound predicates first** (Kusto best-practice: datetime predicates first, hit the time
index, eliminate whole shards) · **partition pruning** · **predicate ordering by selectivity** · **materialized views** for hot
aggregates · **cardinality control on GROUP BY** (high-cardinality entity keys blow up state) · **sampling** · **`approxQuantile`/
sketch approximations** (bounded-memory by construction) · **bucket-time as a sensitivity+cost lever** (Elastic). Query admission
control / quotas were NOT well-documented in the sources [INCONCLUSIVE on vendor specifics].

> **Lean (6.4):** the §15.1/§5.3 cost-bound is satisfied structurally because (a) the sketches are **bounded-state by construction**
> (the anti-DoS invariant from Q3), and (b) prism already has the mandatory time-bound + join-guard NFRs. Additionally enforce a
> **GROUP-BY-entity cardinality cap** on `PROFILE…OVER` (a baseline over an unbounded entity-set is the runaway risk, NOT the per-entity
> math) + **time-bound predicate pushdown first** (Kusto pattern) so the federated pull window is bounded before any profiling runs.
> The on-demand baseline pull must obey the same time-bound + join-guard as any query (§15.1 already states this) — the new control
> is the **entity-cardinality limit + per-query baseline-compute budget** (admission control), which the sources show is NOT a solved
> vendor pattern → prism designs it.

---

## Consolidated Open Design Questions (for architect/PO — NOT decided here)

1. **Vendor-vs-build line for the statistical tier.** `online-statistics` (0.2.6, stale) and `probabilistic-collections` (0.7.0,
   stale) argue for first-party Welford/CGL/EWMA/count-min; `sketches-ddsketch` (0.4.0, healthy), `tdigests` (1.0.1, healthy),
   `hyperloglogplus` (0.4.1, high-adoption) argue for depend. Where exactly is the line? (Affects E-ML-ONDEMAND-001.)
2. **Mergeability contract for the satellite/Purdue edge (§3.2).** EWMA, reservoir, streaming-clustering are NOT cleanly mergeable.
   Is edge-baselining restricted to the mergeable primitives (Welford/CGL, DDSketch, count-min, HLL), or do we accept approximate
   merges for the others? (Architecture decision; affects ADR "model-as-retention-tier".)
3. **Snapshot mechanism choice (Q2.3):** content-addressed per-detection snapshots vs Flink-style changelog+materialization. Per-detection
   replay (content-addressed) vs per-update replay (changelog) — which does §14.5/§15.5 explainability actually require?
4. **Which invariants become Kani proofs vs proptest-only** (Q3) — and are these new VP entries (VP-NNN) siblings to VP-014/VP-015?
5. **Dual-rate + quarantine parameters (Q4.4):** quarantine window length, fast/slow promotion threshold, and the human/agent-confirmation
   routing (§15.8 S3 agent) — these are policy knobs that need per-tenant defaults.
6. **Untrusted-BYO sandbox boundary (Q5.2):** does external-model-as-WASM-plugin reuse the exact C4 WASM sandbox, or a separate
   model-plugin sandbox with different capability grants (inference needs more compute than a hook plugin)?
7. **Entity-cardinality cap + baseline-compute admission budget (Q6.4)** — the genuinely-novel cost-bound; no vendor prior art.
8. **§2.4 tradeoff prose update (G-26)** — PO action; this research supports the "three-ways-to-long-baseline" reframe but does not write it.

## Recommended streaming-algorithm + backend toolkit (concrete, Rust-crate-named) — DISCUSSION INPUT

**Statistical tier (day-2 first, §15.7) — mostly mergeable, all production-grade:**
- Mean/variance: **first-party Welford + Chan–Golub–LeVeque** (mergeable, exact). [online-statistics 0.2.6 too stale to depend on]
- EWMA mean/variance: **first-party** (order-dependent → edge-local).
- Online z-score / BASELINE_DEVIATION: **first-party** over the above; **robust variant = median/MAD** for poison-resistance.
- Quantiles (rel-error, mergeable): **`sketches-ddsketch` 0.4.0**.
- Quantiles (tail-accurate, mergeable-approx): **`tdigests` 1.0.1** (only if abs-error tails needed alongside DDSketch).
- Cardinality (mergeable via max): **`hyperloglogplus` 0.4.1** (high adoption) — for `RARITY`/distinct-count.
- Frequency (mergeable via add): **first-party count-min with conservative update** [all crate options stale].
- Reservoir sampling (if needed): **`streaming_algorithms` 0.3.3** OR first-party Vitter-R (NOT mergeable → edge-local).

**Heavy/learned tier (LATER, §15.7) — MUST-BUILD for streaming variants:**
- Streaming iForest / HS-Trees / DenStream/CluStream/streaming-kmeans: **no maintained Rust crate → build.** `linfa` 0.8.1 +
  `extended-isolation-forest` 0.2.3 cover only the BATCH/on-demand-train case (usable for ephemeral-train-now).

**Inference / BYO backends:**
- Built-in learned: **`candle-core` 0.11.0** (serverless-inference goal, healthy).
- Trusted BYO (any framework → ONNX): **`ort` 2.0.0-rc.12** (process-isolated; still RC — pin carefully).
- Untrusted BYO: **`wasmtime` 46.0.1 Component-Model WASM plugins** (reuse C4 sandbox; CPU small/medium models; SIMD/threads via build flags).
- Satellite/Purdue edge (§3.2): **`tract-onnx` 0.23.3** (tiny pure-Rust runtime).

**Drift / decay / poison (MUST-BUILD in Rust — Python-only ecosystem):**
- Drift: **first-party ADWIN + Page–Hinkley** first (ADWIN self-tunes window = doubles as decay). `neural-drift` unverified → don't depend.
- Decay: EWMA + adaptive (ADWIN) window + DenStream fading factors.
- Poison: median/MAD robust estimators + bounded update rate + anomaly-gated learning + **dual-rate-with-quarantine** for the core tension.

**Persistence:** model state in **RocksDB (the §3.3 tier)**, bincode `ModelState` envelope (schema-versioned, redacted Debug,
`#[non_exhaustive]`), **content-addressed per-detection snapshots** for §15.5 replay, per-tenant via CF (few large tenants) +
key-prefix (long tail).

## Honest Costs & Caveats

- **The heavy tier is real engineering, not glue.** Streaming iForest / HS-Trees / streaming clustering have **no maintained Rust
  crate** (verified 2026-06-27). §15.7 already sequences this LATER; this research confirms that sequencing is correct, not optional.
- **The whole drift-detector ecosystem is Python.** ADWIN/DDM/EDDM/Page-Hinkley/KSWIN are River/scikit-multiflow; the lone Rust crate
  (`neural-drift`) does not document its algorithms. Prism MUST-BUILD these. They are small, but they are build.
- **`ort` is still 2.0.0-rc** (no stable 2.0 as of 2026-06-27) — pin exactly and budget for API churn if chosen for the BYO path.
- **`online-statistics`, `probabilistic-collections`, `extended-isolation-forest`, the older `tdigest`** are all **stale (2020–2022)**.
  Healthy alternatives exist for DDSketch/t-digest/HLL/candle/tract/wasmtime/linfa; the gaps (Welford/EWMA/count-min/drift) are first-party.
- **EWMA, reservoir, and streaming-clustering are NOT cleanly mergeable** — this constrains the satellite/Purdue partial-aggregate
  story (§3.2). The mergeable backbone is Welford/CGL + DDSketch + count-min + HLL.
- **No prior art for per-detection model snapshotting** in online-learning libraries (River/scikit-multiflow). The content-addressed
  approach is an extrapolation [WEB-extrapolated]; it is sound but unproven in this exact form for streaming models.
- **No published Kani/CBMC verification of a sketch.** The Q3 invariants are formally stateable, but proving them is novel work
  (consistent with prism's existing VP-014/VP-015 muscle, but new ground).
- **The core anomaly-gated-learning vs drift tension is genuinely unsolved in general** — dual-rate+quarantine is the best-evidenced
  mitigation, not a closed solution; it shifts attacker cost but does not eliminate boiling-frog risk.
- **Vendor cost-bounding (query admission control / per-query budgets) is under-documented** [INCONCLUSIVE]. The entity-cardinality
  cap + baseline-compute budget (Q6.4) is prism-original design with thin external precedent.
- **Internal scoring algorithms of Splunk/Kusto/Elastic are opaque** — they document logical semantics, not the backing algorithm,
  so the primitive→algorithm mapping (Q6) is prism's choice, informed by but not dictated by their behavior.
- **`reasoning_effort=high` deep-research outputs were large** (85k–106k chars each, 6 calls); each was read in substantive part from
  its saved tool-result file. Tails (largely conclusion/restatement) were sampled via grep, not read verbatim end-to-end — flagged for honesty.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY, reasoning_effort=high)** | 6 | Q1 streaming-algorithm toolkit + mergeability; Q2 model-state persistence/snapshot/per-tenant; Q3 streaming-update invariants + monoid framing + order-dependence testing; Q4 drift/decay/poisoning + the core tension; Q5 Rust ML/inference crates + WASM-sandbox BYO backends; Q6 primitive→engine compilation + peer-group + cost-bounding |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 (resolve-library-id) | 2 | Attempted Rust-crate doc lookup for online-statistics + sketches-ddsketch; Context7 returned only JS/Python/Julia analogs (no Rust crate docs) → fell back to crates.io API for version truth |
| Context7 (query-docs) | 0 | Not used — no Rust crate matched in resolve step |
| Tavily (any) | 0 | — |
| WebFetch (crates.io API) | 13 | **LIVE version verification 2026-06-27** of: sketches-ddsketch (0.4.0), online-statistics (0.2.6), tdigest (0.2.3), tdigests (1.0.1), linfa (0.8.1), candle-core (0.11.0), ort (2.0.0-rc.12), tract-onnx (0.23.3), extended-isolation-forest (0.2.3), probabilistic-collections (0.7.0), wasmtime (46.0.1), hyperloglogplus (0.4.1), streaming_algorithms (0.3.3) |
| WebSearch | 0 | — |
| Training data | 2 areas | Streaming-iForest canonical algorithm (no maintained crate/paper surfaced — flagged [model-knowledge]); "dual-rate" naming for the drift/poison tension resolution (concept [WEB], specific label [model-knowledge]) |

**Total MCP tool calls:** 8 (6 perplexity_research + 2 Context7 resolve). **Plus** 13 WebFetch crates.io version-verification calls.
**Compliance gate:** PASS — ≥1 perplexity_research call (6 of them, all reasoning_effort=high, the mandated PRIMARY tool for non-trivial depth research).
**Training data reliance:** **low** — every algorithm/crate claim is web-grounded or live-verified against crates.io; the two
[model-knowledge] flags (streaming-iForest details, dual-rate label) are explicitly marked and load-bearing only as discussion leans.
**Version-verification gate:** PASS — all 13 load-bearing Rust crates verified live against the crates.io API on 2026-06-27; NO version
number was taken from training data.
