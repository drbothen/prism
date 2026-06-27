---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: "day-2 vision SIDE-ANALYSIS (OUT-OF-BAND; separate from the live VSDD factory pipeline)"
scope: "DEPTH pass on the SATELLITE/PURDUE-EDGE MERGEABILITY decision deferred to empirical test (D-C7-1). Goes deep on APPROXIMATE-MERGE techniques for the non-mergeable primitives (EWMA, reservoir, streaming clustering), their per-primitive ERROR characterization, the residency/privacy angle, and a concrete RUNNABLE empirical test plan."
source_artifacts_read:
  - ".factory/research/ml-behavior-analytics-depth-2026-06-27.md (C7 — Q1 mergeability table + §3.2 edge-baselining note; PRIMARY LEAN = restrict edge baselining to cleanly-mergeable primitives; D-C7-1 deferral)"
  - ".factory/research/satellite-mesh-2026-06-26.md (C2 — satellite chain relays only normalized aggregates upward; residency-by-construction; partial-result fan-in; D-C2-12 hard-residency)"
related_decisions: [D-C7-1, D-C2-12]
non_rederivation: "Does NOT re-derive the C7 per-primitive mergeability table or the C2 mesh design. Builds ON them: the C7 table establishes WHICH primitives are non-mergeable; this pass establishes HOW WELL the non-mergeable ones can be approximately merged, what error that costs, and how to MEASURE it to settle D-C7-1 per primitive."
leans_are: "discussion input only — NOT decisions. No ADR is created or amended by this file. Leans inform the architect/PO/human conversation that adjudicates D-C7-1."
---

# Edge-ML Mergeability — DEPTH Research (settling D-C7-1 per primitive: mergeable-only vs approximate-merge)

> **CAPTURE-ONLY.** Out-of-band side-analysis artifact. Does NOT modify STATE.md, SESSION-HANDOFF.md,
> the ADR registry, any live spec/BC/story, RESEARCH-INDEX.md, or any prior research file. Not git-added
> or committed by this run. All "Lean" rows are discussion input, not decisions.

## Provenance, scope, and the question this pass answers

In C7 (`ml-behavior-analytics-depth-2026-06-27.md` §15, open question 2), the human deferred the
**satellite/Purdue-edge mergeability decision (D-C7-1)** to an empirical test. The framing already settled by C7 + C2:

- **C7 PRIMARY LEAN:** restrict edge baselining to *cleanly-mergeable* primitives — Welford / Chan–Golub–LeVeque
  (CGL) mean-variance, DDSketch quantiles, Count-Min frequency, HyperLogLog cardinality. These combine **exactly**
  under arbitrary merge trees (associative/commutative monoid merge).
- **C7 ALTERNATIVE-TO-TEST:** allow the *non-mergeable* primitives — EWMA, reservoir sampling, streaming clustering —
  at the edge via documented approximations.
- **C2 mesh invariant (D-C2-12 hard-residency):** the satellite chain relays **only normalized aggregates** upward
  across the conduit; raw stays in-zone; partial-result fan-in accumulates per-hop. Whatever crosses the conduit is
  an aggregate/model-state, not raw telemetry.

**This pass does NOT re-derive the C7 table or the C2 mesh.** It goes deep on the three open questions C7 left to
measurement: (a) *how* the non-mergeable primitives can be approximately merged, (b) *how much accuracy is lost*
versus a true single-stream / central-recompute result, and (c) the *concrete runnable experiment* that decides
mergeable-only vs approximate-merge **per primitive** for SECURITY behavioral baselining (where a wrong baseline =
a missed anomaly / a spurious anomaly).

**Confidence legend:** [WEB] = grounded in a cited web finding from the deep-research pass; [LIT-SETTLED] = a proven
theorem / established result in the cited literature; [LIT-EXTRAPOLATED] = mathematically sound but the source flags
it as an extrapolation not explicitly published; [model-knowledge] = from model training data, flagged;
[INCONCLUSIVE] = research could not settle. **Every numeric error claim distinguishes LITERATURE-SETTLED from
PRISM-MUST-MEASURE.**

---

## Part 1 — The mergeable-summaries foundation (why the C7 backbone merges and the others don't)

**Source: Agarwal, Cormode, Huang, Phillips, Wei, Yi, "Mergeable Summaries" (PODS 2012 / ACM TODS).** [WEB][LIT-SETTLED]

### 1.1 The formal taxonomy (this is the frame for the whole pass)

The paper defines a *summary function* `S(D, ε)` mapping a dataset + accuracy parameter to a bounded-size structure,
and a *merge algorithm* `A`. Three precise tiers emerge [WEB][LIT-SETTLED]:

| Tier | Formal property | Consequence for the satellite chain |
|---|---|---|
| **Fully mergeable** | ∃ binary merge `⊕` s.t. `S(D₁,ε) ⊕ S(D₂,ε) = S(D₁⊎D₂,ε)`, with size ≤ `k(|D₁|+|D₂|,ε)`, AND `⊕` is **associative + commutative with identity** (a commutative monoid), so error+size are preserved under **arbitrary merge trees**, not just left-to-right. | Edge nodes combine in any tree shape; per-hop fan-in is exact. This is the C7 backbone. |
| **One-way / streaming-mergeable** | Correct for *sequential* single-stream construction, but no associative/commutative binary merge preserves the error bound. Merging degrades ε (e.g. Greenwald–Khanna: merging two ε-summaries yields a 2ε-summary; error grows with merge depth). | Usable on one stream; degrades across the tree. |
| **Non-mergeable** | No binary operator on the summaries can produce a valid summary of the union without extra information, because **the summary state is not a sufficient statistic for the union** under the scheme's semantics. | EWMA + reservoir(Algorithm R) land here — but with important escape hatches (Parts 2–3). |

**Key insight [WEB][LIT-SETTLED]:** mergeability is an *algebraic* property of the **exposed state**, not of the
underlying statistic. The mean is decomposable from sufficient statistics `(n, μ, M₂)` — but only if the algorithm
*exposes* those statistics. EWMA's scalar state `z_t` does not expose them, which is exactly why it's non-mergeable
*as a scalar* even though a decayed average is decomposable from richer state (Part 2's escape hatch).

### 1.2 The proven fully-mergeable summaries (the C7 backbone — confirmed exact)

[WEB][LIT-SETTLED] These are the C7 backbone, with the merge op and what it preserves:

| Primitive | Merge op | Guarantee preserved | Citation |
|---|---|---|---|
| **CGL / Welford mean-variance** | `n=n₁+n₂; δ=μ₂−μ₁; μ=μ₁+δ·n₂/n; M₂=M₂₁+M₂₂+δ²·n₁·n₂/n` | **EXACT** (algebraic identity, zero error) | Chan–Golub–LeVeque [WEB] |
| **DDSketch** | element-wise add of aligned log-bucket counts | relative-error α preserved exactly; "first **fully-mergeable** relative-error quantile sketch" | Masson et al. VLDB 2019 [WEB] |
| **Count-Min** | element-wise add of counter table (linear sketch: `L(f₁+f₂)=L(f₁)+L(f₂)`) | one-sided ε·N bound preserved | Cormode–Muthukrishnan [WEB] |
| **HyperLogLog** | register-wise **max** | identical to single-pass over the union (std-err ≈ 1.04/√m) | Flajolet [WEB] |
| **KLL / Mergeable randomized quantiles** | concat buffers + re-compact | ε-rank preserved under arbitrary merges; size independent of N | Agarwal et al.; Karnin–Lang–Liberty (KLL, optimal `O((1/ε)loglog(1/δ))`) [WEB] |

> Note: Greenwald–Khanna is **only one-way mergeable** (merging doubles ε) [WEB][LIT-SETTLED] — it is NOT in the
> fully-mergeable backbone. The C7 quantile choice (DDSketch) is the correct one for the edge precisely because GK
> degrades across a tree and DDSketch does not.

### 1.3 Where EWMA / reservoir / clustering fall — and the precise broken invariant

[WEB][LIT-SETTLED] The paper places EWMA and Algorithm-R reservoir in the **non-mergeable** tier, with the precise
obstruction named for each:

- **EWMA** — broken invariant: **time-ordering**. The scalar `z_t` conflates the magnitudes *and* the ordering of
  past values irreversibly; different global interleavings of the same multiset produce different global EWMA but
  identical per-shard states (Part 2 counterexample). The state is not a sufficient statistic for the union.
- **Reservoir (Algorithm R)** — broken invariant: **uniform inclusion probability**. A k-reservoir holds each local
  item with prob `k/n_i`, but the reservoir does not expose the random keys/weights needed to recompute global
  inclusion probabilities `k/n`. Naive union over-weights small shards (Part 3).
- **Streaming clustering** — the paper does not treat it directly, but the follow-on clustering literature (Part 4)
  shows the obstruction is **narrower than C7 implied**: the underlying CF arithmetic is additive; only the
  **fading-weight time-alignment** and the **final macro-clustering** introduce approximation.

> **Lean (Part 1):** the C7 backbone (CGL/DDSketch/Count-Min/HLL/KLL) is fully-mergeable-by-theorem and should be
> the **default edge baseline math**, unconditionally. The interesting question is whether the three non-mergeable
> primitives have *escape hatches* that make them acceptably-mergeable. Parts 2–4 show that **two of the three do**
> (EWMA via forward-decay reformulation; clustering via additive CF vectors), while the third (reservoir) has a
> clean fully-mergeable *substitute* (random-key sampling). The genuine "approximate-merge with bounded error"
> case is narrower than the C7 alternative framing assumed.

---

## Part 2 — EWMA approximate merge: error + the forward-decay escape hatch

**Sources:** Cormode–Korn–Tirthapura (time-decaying aggregates on out-of-order streams); Cormode–Shkapenyuk–
Srivastava–Xu (**forward decay**); Cohen–Wang (general decay sums); FDCMSS (forward-decay heavy hitters). [WEB]

### 2.1 The non-mergeability counterexample (LITERATURE-SETTLED, exact)

[WEB][LIT-SETTLED] Two shards, common λ. Shard 1 sees `(a,b)`, shard 2 sees `(c,d)`. Per-shard states
`z_{1,2}=λb+(1−λ)λa`, `z_{2,2}=λd+(1−λ)λc` are **identical** regardless of how the four values interleave globally.
But global EWMA under order `a,b,c,d` ≠ global EWMA under order `c,d,a,b` (the geometric weights `λ(1−λ)^age`
differ). Any merge function reading only the shard states must output one value → cannot equal both. **No exact
scalar-EWMA merge exists.** Generalizes: shard states live in an N-dim space; the global EWMA depends on the
full interleaving, which has been irreversibly lost.

### 2.2 The four approximate-merge techniques + their error (LITERATURE-EXTRAPOLATED)

The deep-research pass is explicit: the streaming literature studies **decayed aggregates expressed via per-event
timestamps**, not the merging of *scalar EWMA states*. So techniques (1)–(3) below have **no published error bound**;
the source supplies a sound extrapolated analysis. Technique (4) is the literature-settled answer.

| # | Technique | Mechanism | Error vs true single-stream EWMA | Status |
|---|---|---|---|---|
| 1 | **Time-aware weighted combination** | `Z = Σ wᵢ(T)·zᵢ(tᵢ) / Σ wᵢ(T)`, `wᵢ` ∝ effective sample size / recency `exp(−β(T−tᵢ))` | Bounded by `R · Σ\|w̃_{i,k} − w^global_{i,k}\|` (range × total-variation distance between approximate and true per-event weight distributions). **Small** when λ large (fast decay, dominated by recent events) + shard rates similar; **grows** with effective memory `1/λ`, with N (interleaving complexity), and with cross-shard rate skew. | [LIT-EXTRAPOLATED] |
| 2 | **Re-EWMA over coarsened per-shard summaries** | shards emit periodic summary points; central runs a 2nd-level EWMA | Composite = convolution of two exponential kernels → slower effective decay, **lags** true EWMA on sudden change; error grows with reporting interval M and `\|λ_s,λ_c,λ_target\|` mismatch | [LIT-EXTRAPOLATED] |
| 3 | **Timestamped decay-correction** | rescale each shard's state `ẑᵢ(T)=zᵢ(tᵢ)·exp(−β(T−tᵢ))` to a common reference time | Treats the whole state as one event at `tᵢ`, ignoring that each constituent event should decay by its own age. Error ∝ spread of timestamps within a shard and elapsed `(T−tᵢ)`; **small** if shard events are tightly clustered near `tᵢ` and `(T−tᵢ)` is short; **large** for long-span / sparsely-updated shards | [LIT-EXTRAPOLATED] |
| 4 | **Central recompute from per-shard sufficient statistics (forward decay)** | shards relay forward-weighted sums `Uᵢ=Σ xᵢₖ·exp(λtᵢₖ)`, `Vᵢ=Σ exp(λtᵢₖ)`; central computes `A(T)=ΣUᵢ/ΣVᵢ` | **EXACT** (zero merge error) | [WEB][LIT-SETTLED] |

### 2.3 THE ESCAPE HATCH — forward decay restores mergeability that backward decay (EWMA) lacks

This is the load-bearing finding of Part 2, and it directly answers the human's question 5. [WEB][LIT-SETTLED]

- **Backward decay (standard EWMA):** weight of event at `tᵢ` for a query at `T` is `exp(−λ(T−tᵢ))` — depends on the
  query time, so the scalar state is query-time-coupled and non-mergeable.
- **Forward decay (Cormode–Shkapenyuk–Srivastava–Xu):** record `exp(λtᵢ)` at *arrival time* (a landmark-relative
  forward age), independent of any future query. Then `S(T)=exp(−λT)·Σ exp(λtᵢ)·xᵢ` and `W(T)=exp(−λT)·Σ exp(λtᵢ)`,
  so the decayed average `A(T)=S(T)/W(T)=ΣUᵢ/ΣVᵢ` — the `exp(−λT)` factors **cancel**. For *exponential* decay,
  forward and backward decay produce **identical decay-function values** (proven), so forward decay yields the same
  decayed average — but its sufficient statistics `(Uᵢ, Vᵢ)` are **time-independent and mergeable by simple
  addition**. This is exploited in production by FDCMSS (forward-decay Count-Min Space-Saving for time-faded heavy
  hitters). [WEB][LIT-SETTLED]

**Implication for D-C7-1:** "EWMA is non-mergeable" is true *only for the scalar-state representation*. If the edge
maintains `(U, V)` (two scalars per feature per decay rate) instead of (or alongside) the scalar `z`, the decayed
average becomes **exactly mergeable** — moving EWMA from the "approximate-merge" column into the "mergeable-exact"
column at the cost of one extra scalar of state and a numeric-stability caveat (`exp(λt)` grows unbounded; forward
decay needs periodic landmark re-basing to avoid overflow — a known, solvable engineering detail [WEB]).

> **Lean (Part 2):** do **not** ship scalar-EWMA approximate-merge. Ship the **forward-decay sufficient-statistic
> form `(U,V)`** at the edge — it is exactly mergeable, costs one extra scalar, and aligns with the C2 invariant
> (the `(U,V)` tuple is a normalized aggregate, not raw). EWMA-as-`(U,V)` joins the C7 backbone. Reserve scalar-state
> approximate-merge (techniques 1–3) only for a legacy/constrained-edge fallback where the extra scalar genuinely
> cannot be afforded — and only after the test plan (Part 6) measures its error for Prism's λ/N/skew regime, because
> those error bounds are EXTRAPOLATED, not literature-settled. Numeric-stability landmark re-basing must be designed
> in-scope, not deferred.

---

## Part 3 — Reservoir sampling merge: the bias, the correction, and the fully-mergeable substitute

**Sources:** Vitter Algorithm R; Efraimidis–Spirakis "Weighted Random Sampling over Data Streams" (A-Res / A-ExpJ);
Cohen–Kaplan bottom-k sketches; priority sampling; Tirthapura–Woodruff "Optimal Random Sampling from Distributed
Streams Revisited"; Cormode et al. continuous distributed sampling (ISWoR); MapReduce reservoir sampling. [WEB]

### 3.1 The naive-union bias (LITERATURE-SETTLED, exact)

[WEB][LIT-SETTLED] N shards each run Algorithm R with reservoir size k over `nᵢ` items. Naive union of all `Nk`
items then subsample k uniformly gives, for an item on shard i:

```
Pr[x ∈ global sample] = (k/nᵢ) × (1/N) = k / (N·nᵢ)
```

— **inverse in shard size `nᵢ`**. Worked example from the source: `n₁=10³`, `n₂=10⁶`, `k=100` → items on the small
shard are **2000× more likely** to be sampled than items on the large shard. This is classic two-stage cluster-
sampling bias. The ratio of inclusion probabilities between shards i,j is exactly `nⱼ/nᵢ`. **Unacceptable** for a
globally-uniform sample whenever shard sizes differ by orders of magnitude (the OT/MSSP reality: a quiet L1 zone vs
a busy enterprise zone).

### 3.2 The correction — weighted re-sampling (sound; LIT-EXTRAPOLATED as a continuous protocol)

[WEB] To restore global uniformity, second-stage selection probability for shard i must be `pᵢ = nᵢ/n` (large
shards contribute proportionally more reservoir items). Implement via A-Res with per-item weight `wᵢ = nᵢ/k`; total
weight `= n`; the combined inclusion probability collapses to `(k/nᵢ)·(nᵢ/n) = k/n`, independent of i — globally
uniform. **Requires each shard to relay its reservoir AND its item count `nᵢ`.** The two-shard "balls-and-bins"
form (select shard with prob `m/(m+n)`, then a random reservoir item) is the special case. The source flags this as
a sound *extrapolation* of two-stage sampling theory, not an explicitly-published continuous algorithm.

### 3.3 THE SUBSTITUTE — random-key / bottom-k sampling IS fully mergeable

This is the load-bearing finding of Part 3 (parallels Part 2's forward-decay escape hatch). [WEB][LIT-SETTLED]

Efraimidis–Spirakis assign each item a **random key** `K = u^(1/w)` (or `−ln(u)/w`); the sample is the **top-k keys**.
Because keys are assigned per item, i.i.d., independent of shard boundaries, the global top-k keys over the union are
**necessarily contained in the union of the per-shard top-k sets**. So merge = union + re-select global top-k. The
top-k-keys operation is **associative + commutative** → the summary is a **commutative monoid** → **fully mergeable**.
This is the same mechanism behind **bottom-k sketches** (Cohen–Kaplan) and **priority sampling**, and it is what
MapReduce reservoir sampling and the Tirthapura–Woodruff optimal distributed protocol actually use under the hood. [WEB][LIT-SETTLED]

- For the **uniform** case (all weights equal): random-key reservoir = uniform reservoir, still fully mergeable, and
  **does not require per-shard counts `nᵢ`** (unlike the Algorithm-R weighted correction). [WEB][LIT-SETTLED]
- For the **weighted** case: keys encode weights directly; merge is unchanged.

So reservoir sampling is non-mergeable **only in the Algorithm-R count-based representation**. Switch the
representation to random-key / bottom-k and it becomes fully mergeable by theorem — exactly like EWMA→forward-decay.

### 3.4 Measuring sample non-uniformity (feeds the test plan)

[WEB] To quantify how far a merged sample is from uniform-over-the-population: **Kolmogorov–Smirnov distance**
`Dₙ = supₓ|Fₙ(x)−F(x)|` (continuous features, vs the global CDF); **chi-square goodness-of-fit** `Σ(Oᵢ−Eᵢ)²/Eᵢ`
(categorical, vs global category proportions); **total variation distance** `½Σ|p−q|`; **Wasserstein / earth-mover**;
and **inclusion-probability bias** (max deviation of empirical inclusion prob from `k/n`). All standard; KS +
chi-square are the literature norm for reservoir representativeness.

> **Lean (Part 3):** ship **random-key (Efraimidis–Spirakis / bottom-k) sampling** at the edge, NOT Algorithm-R +
> weighted-merge. Rationale: fully mergeable by theorem, associative/commutative across the tree, and — critically —
> needs **no per-shard count `nᵢ` relayed** (the count-relay is itself a tiny extra disclosure and a coordination
> dependency). Like forward-decay EWMA, this moves reservoir from "approximate-merge" to "mergeable-exact" by a
> representation change at zero accuracy cost. Reserve Algorithm-R + weighted-re-sampling only as a fallback for an
> already-deployed Algorithm-R edge, and only after the test plan measures its KS/chi-square bias under Prism's
> shard-skew regime.

---

## Part 4 — Streaming clustering: MORE mergeable than C7 implied (CF vectors are additive)

**Sources:** Zhang–Ramakrishnan–Livny BIRCH (SIGMOD 1996); Aggarwal–Han–Wang–Yu CluStream (VLDB 2003); Cao–Ester–
Qian–Zhou DenStream (SDM 2006); Har-Peled–Mazumdar & Feldman–Langberg (mergeable coresets); streaming coreset
merge-and-reduce. [WEB]

### 4.1 The CF additivity theorem (LITERATURE-SETTLED, EXACT — confirms the human's hypothesis)

[WEB][LIT-SETTLED] BIRCH's Cluster Feature `CF(C) = (N, LS, SS)` = (count, linear sum, squared-norm sum). The
**CF Additive Theorem**: for disjoint `C₁, C₂`,

```
CF(C₁ ∪ C₂) = CF(C₁) + CF(C₂) = (N₁+N₂, LS₁+LS₂, SS₁+SS₂)
```

— **exact**, because each component is a sum over points and sums are additive over set union. Centroid `μ=LS/N`,
radius `R=√(SS/N − ‖μ‖²)`, and the entire k-means cost contribution are computable from the merged CF with **zero
loss**. The CF vector is a (commutative-monoid) **mergeable coreset** for first/second moments. [WEB][LIT-SETTLED]

The human's hypothesis is **confirmed by theorem**: the micro-cluster CF merge IS exact. C7's "streaming clustering
is not cleanly mergeable" was *too pessimistic* about the arithmetic — the approximation lives elsewhere (4.3, 4.4).

### 4.2 CluStream + DenStream inherit additivity

- **CluStream** [WEB][LIT-SETTLED]: micro-cluster = temporal extension `(N, LS_x, SS_x, LS_t, SS_t)` — additive, same
  theorem. The **pyramidal time frame** computes a recent window by **subtracting** an older snapshot from the current
  one (linear, exact). Distributed: each shard keeps its pyramid; the central node sums snapshots and subtracts —
  addition/subtraction commute, so merge-then-window = window-then-merge. The **only** temporal approximation is the
  coarseness of which snapshot times are kept (bounded in the paper: `t_c − t_s ≤ (1+1/α^{l−1})·h`).
- **DenStream** [WEB][LIT-SETTLED]: fading micro-cluster = weighted CF `(W(t), LS(t), SS(t))` with `wᵢ(t)=2^{−λ(t−tᵢ)}`.
  At a **fixed reference time**, faded CFs are additive (sums over disjoint sets). Advancing a CF in time with no new
  points is just scalar multiplication by `2^{−λ(t−t₀)}` of all three components.

### 4.3 Where approximation actually enters (the human's refinement, confirmed)

[WEB] Exactly the two places the human predicted, plus pruning:

1. **Fading-weight time-reconciliation across shards.** To merge shard A's CF (decayed to `t_A`) with shard B's
   (decayed to `t_B`) at a common `t_ref`, scale each by `2^{−λ(t_ref−tᵢ)}`. This is **exact under accurate clocks**.
   With clock skew `|δᵢ| ≤ Δ`, the worst-case multiplicative weight error is `2^{±λΔ}` [WEB][LIT-EXTRAPOLATED] —
   small for low λ / good sync, larger for fast decay / poor sync. **Bounded**, and the bound is explicit.
2. **Final macro-clustering step.** Running k-means/DBSCAN over merged micro-clusters vs over centrally-collected raw
   data is approximate — but this is the *general coreset approximation* (Har-Peled–Mazumdar: clustering on a coreset
   gives a `(1+ε)` approximation to the optimal cost; Feldman–Langberg unify offline+streaming; mergeable coresets
   compose under merge-and-reduce). [WEB][LIT-SETTLED] This approximation exists *even in the single-machine case* —
   it is not introduced by distribution.
3. **Pruning divergence.** DenStream prunes outlier micro-clusters below a weight threshold. If shards prune at
   different times/thresholds, the merged set may omit points that a central run would have kept — a divergence from
   *algorithmic decisions*, not from CF arithmetic. [WEB]

### 4.4 Implementation caveat (flagged honestly)

[WEB] River's DenStream/CluStream implementations have reported index-management bugs (deleted/merged micro-cluster
indices, misaligned weights). The math is additive; *correct merging in practice* requires disciplined index/weight
handling. For Prism this is a build-quality note: the CF merge is trivially correct on paper, easy to get subtly
wrong in code → needs proptest coverage of the merge (associativity/commutativity of CF addition; time-rescale
correctness).

> **Lean (Part 4):** treat the **micro-cluster CF merge as mergeable-exact** (it is, by theorem). The edge maintains
> additive `(N, LS, SS)` (+ `LS_t, SS_t` for CluStream temporal, or faded `(W, LS, SS)` for DenStream). The
> **two** genuine approximation sites are (a) cross-shard fading-weight time-alignment — error bounded by `2^{λΔ}`,
> controllable by clock-sync discipline + low λ, MEASURABLE; and (b) macro-clustering coreset error — a `(1+ε)`
> coreset guarantee that exists single-machine too, so distribution adds little. **Disposition: approximate-merge
> acceptable for clustering, with the approximation confined and bounded** — but the bound on (a) is extrapolated and
> the macro-drift on real security data must be measured (Part 6, ARI/NMI/CMM). Pruning policy must be made
> **uniform across shards** (same threshold, reference-time-aligned) to eliminate divergence source (3).

---

## Part 5 — The residency / privacy angle (and the critical privacy≠residency distinction)

**Sources:** Google federated analytics; "Cardinality Estimators Do Not Preserve Privacy" (HLL); Count-Min privacy
attacks; "Knock Knock Who's There?" + Zero-Knowledge MIA (membership inference on aggregate location); differentially-
private linear sketches (NeurIPS); RAPPOR; Apple Samplable Anonymous Aggregation; pan-private streaming; EU Art. 29
WP anonymization opinion; IEC 62443. [WEB]

This section is deliberately brief and ties to C2's D-C2-12 hard-residency.

### 5.1 Residency vs privacy are DIFFERENT guarantees (LITERATURE-SETTLED)

- **Residency** = *where* data is processed/stored and *what crosses a boundary*. The C2 invariant (raw stays in-zone;
  only normalized aggregates cross the conduit) **satisfies residency-by-construction** regardless of approximation
  level. Whether the aggregate is exact or coarsened does **not** change residency compliance — residency cares that
  a *function of the raw data* (not the raw records) crossed, which is true either way. [WEB][LIT-SETTLED]
- **Privacy** = *what an observer can infer* from what crossed. This is where approximation *might* matter — but
  mostly doesn't, formally (5.2).

### 5.2 Aggregates leak; sketches are NOT inherently private; coarsening is NOT a privacy mechanism (LITERATURE-SETTLED)

- **Aggregates leak.** Membership-inference on aggregate location time-series reaches **0.99–1.0 AUC** on raw
  aggregates; reconstruction attacks recover features from gradients/aggregates. Aggregation alone is not a privacy
  guarantee. [WEB][LIT-SETTLED]
- **Sketches are "almost as sensitive as raw data."** "Cardinality Estimators Do Not Preserve Privacy" proves strong
  aggregation (mergeability without precision loss) is **incompatible** with any reasonable privacy definition →
  HLL is privacy-poor; Count-Min has demonstrated extraction attacks; quantile sketches reveal distributional detail.
  The C7-backbone sketches that cross the conduit must be treated as **sensitive**, not as anonymization. [WEB][LIT-SETTLED]
- **Coarsening / approximate-merge is deterministic post-processing.** It **reduces information-theoretic mutual
  information** but confers **no formal privacy guarantee** — by the DP post-processing theorem, deterministic
  functions of a non-private aggregate are still non-private, and of a DP aggregate keep the *same* ε (can't improve
  it). So "approximate-merge leaks less" is **true info-theoretically but false as a formal guarantee**; an attacker's
  task (e.g. membership inference on a distinctive device signature) may survive coarsening intact. [WEB][LIT-SETTLED]

### 5.3 The actual privacy lever — DP, and DP sketches stay mergeable

[WEB][LIT-SETTLED] Differentially-private linear sketches (add calibrated noise at init; linearity → still
mergeable), RAPPOR (local-DP randomized response, aggregatable), KLL-private quantiles (sublinear-space DP), and
Apple's Samplable Anonymous Aggregation are all **mergeable AND privacy-bearing**. Local-DP fits the satellite
topology: each zone noises before relaying upward; central composes. Coarsening can *support* DP by lowering
sensitivity (less noise needed) but is not a substitute.

> **Lean (Part 5):** the approximate-merge decision (D-C7-1) is **orthogonal to residency** — both exact and
> approximate merges satisfy C2's residency-by-construction equally; do not justify approximate-merge on residency
> grounds. It is **weakly relevant to privacy** (coarsening reduces info-theoretic leakage but gives no formal
> guarantee). If Prism ever needs a *formal* privacy guarantee on what crosses the conduit (a plausible MSSP /
> cross-tenant requirement), the answer is **local differential privacy on mergeable DP sketches**, NOT coarsening —
> and DP sketches remain mergeable, so this composes with the C7 backbone. Flag as a **separate future decision**
> (privacy posture of conduit-crossing aggregates), distinct from D-C7-1. Do not conflate the two.

---

## Part 6 — The concrete empirical test plan (the deliverable for settling D-C7-1)

**Sources for methodology:** DDSketch / t-digest accuracy-experiment structure; CluStream/DenStream/MOA evaluation;
AF-Stream approximate-vs-exact "is it accurate enough" framing; KS / chi-square goodness-of-fit; ARI/NMI/CMM. [WEB]
The *structure* below is literature-settled; the **error→detection-quality mapping and the security acceptance
thresholds are bespoke** (the source is explicit that this mapping must be crafted for the security context — PRISM-MUST-DESIGN).

### 6.0 The universal harness (applies to all three primitives)

- **(a) Baseline = ground truth:** exact single-stream / central-recompute over the unified globally-time-ordered
  stream. This is the "truth" every treatment is scored against. [WEB — the standard sketch-paper structure]
- **(b) Treatment:** partition the same stream across N shards; each shard maintains the edge primitive; merge via the
  candidate approximate-merge; compare to baseline.
- **(c) Run BOTH the mergeable-exact form and the approximate form** as treatments, so the test also *validates the
  escape hatches*: forward-decay EWMA `(U,V)` and random-key reservoir should show ~zero merge error (a correctness
  check), while scalar-EWMA-merge and Algorithm-R+weighted-merge show the approximate-merge error. This makes the
  test a per-primitive *decision* between exact-form and approximate-form, not just a pass/fail.

### 6.1 Per-primitive error metrics (LITERATURE-SETTLED metrics)

| Primitive | Metrics vs central baseline | Source |
|---|---|---|
| **EWMA** | pointwise relative deviation `δ_t=(m^merged−m^central)/m^central`; **MARE** (mean abs rel err), **max** `\|δ_t\|`, **RMSE**, **MAPE**; plus **temporal lag** (cross-correlation lag at max correlation; sign-disagreement rate of consecutive changes) | [WEB] (DDSketch rel-error + forecasting MAPE) |
| **Reservoir** | **KS distance** vs global CDF (continuous); **chi-square** GoF vs global category proportions; **total variation**; **Wasserstein**; **inclusion-probability bias** (max dev from `k/n`) | [WEB] |
| **Clustering** | **ARI**, **NMI**, **purity** vs central reference clustering; **centroid drift** (mean Euclidean displacement of matched centroids); **CMM** (Cluster Mapping Measure — streaming-specific, handles splits/merges/births/deaths under drift) | [WEB] |

### 6.2 Workload / data generators (LITERATURE-SETTLED tooling; security mapping bespoke)

1. **Stationary** — MOA `RandomRBFGenerator` (fixed Gaussian mixture). Isolates pure merge/partition error (a
   non-zero error here = architectural flaw, not drift). [WEB]
2. **Diurnal/seasonal** — modulate feature mean/variance sinusoidally `μ_t = μ₀ + A·sin(2πt/24)` (login rates,
   connection counts). Tests whether the merge tracks periodic baselines or lags. [WEB]
3. **Drifting** — `RandomRBFGeneratorDrift` (gradual center movement), SEA / Agrawal concepts (abrupt), recurring
   (return to prior regime). Drift severity = center-movement distance per unit time. [WEB]
4. **Adversarial cross-shard skew** — one shard receives a disproportionate / shifted slice (targeted-attack subnet);
   skew ratios 1:1, 2:1, 5:1, 10:1. This is the case most likely to break approximate-merge: a busy enterprise zone
   diluting a quiet-but-attacked OT zone, or vice versa. **The decisive workload for the OT/MSSP residency story.** [WEB]

### 6.3 Parameter sweeps

- **Shard count N** ∈ {1 (central control), 4, 8, 64, 128, 256} × {uniform, skewed shard sizes} × {flat-merge,
  tree-merge} (tree-merge tests associativity — the exact forms must be order-invariant; the approximate forms may not be).
- **EWMA decay λ** ∈ {0.01, 0.05, 0.1, 0.2, 0.5}; merge frequency / window {1min, 5min, 1hr}.
- **Reservoir size k** ∈ {100, 500, 1000, 5000, 10000}; merge strategy {naive-union, weighted-re-sample, random-key}.
- **Clustering** — micro-clusters/shard, total micro-clusters, macro `k`, fading λ, macro-cluster horizon, pruning
  threshold; **clock-skew Δ** swept explicitly (to validate the `2^{λΔ}` bound from Part 4).
- **Skew ratio** and **drift severity** as first-class adversarial axes → produce *accuracy-vs-adversarialness* curves.

### 6.4 The error→detection-quality mapping (PRISM-MUST-DESIGN; this is the security-specific core)

The statistical error metrics are not the decision criterion — **detection-quality degradation is**. The source
supplies the mechanistic bridge for a z-score detector [WEB]:

```
z^merged_t − z^central_t  ≈  −Δm_t / σ_t        (small variance-error regime)
```

i.e. a baseline mean-error `Δm` shifts every z-score by `Δm/σ`, flipping threshold crossings near `z_thresh`. So:

- For **EWMA**: a measured MAPE translates to a z-shift `≈ MAPE·(μ/σ)`. If `μ ≫ σ` (high signal-to-baseline), 1% MAPE
  is a tiny z-shift → negligible detection impact. If `μ ≈ σ` or σ small, the same 1% is detection-significant. **The
  acceptance threshold on EWMA error is therefore conditioned on the baseline mean/variance ratio and `z_thresh`.**
- For **reservoir**: KS/chi-square bias → skew in detector *training/calibration* data → FPR/FNR shift (the source
  notes this is indirect; measure by training the detector on merged vs central reservoir and comparing PR-AUC).
- For **clustering**: ARI/centroid-drift → cluster-boundary shift → anomalies absorbed into benign clusters (FN) or
  benign points fragmented out (FP).

**Downstream detection metrics computed under central vs approximate baseline:** ΔFPR, ΔFNR, ΔPrecision, ΔRecall,
ΔPR-AUC, with injected ground-truth anomalies. [WEB]

### 6.5 Acceptance thresholds + decision rule (PRISM-MUST-DESIGN — bespoke, with a recommended starting rule)

The literature gives the *structure* (AF-Stream's "define an acceptable error threshold, show approximate stays
within it") but **not** the numbers for security. Recommended starting decision rule (discussion input, to be ratified):

For each primitive × each operational regime (especially the adversarial-skew regime):

1. **mergeable-exact** if the exact form (forward-decay `(U,V)` / random-key reservoir / CF-merge) is available — it
   has ~zero merge error by theorem; **prefer it unconditionally**. (For EWMA and reservoir this is the recommended
   default per Parts 2–3.)
2. **approximate-merge acceptable** iff, across ALL swept regimes including 10:1 adversarial skew and abrupt drift:
   - `ΔFNR ≤ ε_FN` (recommend a **tight** bound, e.g. ΔFNR ≤ 0.5 percentage points — a missed anomaly is the
     expensive error in security), AND
   - `ΔFPR ≤ ε_FP` (recommend a looser bound, e.g. ΔFPR ≤ 2 pp — alert fatigue matters but is less catastrophic), AND
   - `ΔPR-AUC ≥ −δ_AUC` (recommend δ_AUC ≤ 0.01), AND
   - the error is **monotone / non-pathological** in N (no blow-up at high shard counts).
3. **central-only** if the approximate-merge violates the above in *any* adversarial regime — i.e. the primitive must
   be computed at central from raw, OR (preferred) the residency invariant forces it to remain **edge-local-only**
   (computed and consumed at the edge, never merged upward) rather than centrally-recomputed-from-raw, because raw
   cannot cross the conduit (C2 D-C2-12). **This last point is the residency twist:** "central-only" in a residency-
   constrained mesh cannot mean "ship raw to central"; it means "this baseline is edge-local and does not participate
   in cross-conduit fan-in."

### 6.6 How the result feeds D-C7-1

Per primitive, the test yields one of: `mergeable-exact` (adopt the exact form), `approximate-merge-acceptable`
(adopt the approximate form, document the error envelope + the regimes where it holds), or `central-only/edge-local`
(do not merge upward). That per-primitive verdict **is** the D-C7-1 decision, now evidence-backed rather than leaned.

---

## Per-primitive mergeability verdict + recommended edge disposition (DISCUSSION INPUT)

| Primitive | C7 said | This pass finds | Recommended edge disposition | Confidence |
|---|---|---|---|---|
| Welford/CGL mean-var | mergeable-exact | mergeable-exact (theorem) | **mergeable-exact** — backbone | [LIT-SETTLED] |
| DDSketch / Count-Min / HLL / KLL | mergeable-exact | mergeable-exact (theorem) | **mergeable-exact** — backbone | [LIT-SETTLED] |
| **EWMA** | NOT cleanly mergeable | scalar-state non-mergeable; **forward-decay `(U,V)` form is mergeable-EXACT** | **mergeable-exact via forward-decay `(U,V)`** (one extra scalar + landmark re-basing). Scalar approximate-merge = constrained-edge fallback only, error EXTRAPOLATED → must measure. | escape hatch [LIT-SETTLED]; scalar-merge error [LIT-EXTRAPOLATED] |
| **Reservoir** | NOT cleanly mergeable | Algorithm-R non-mergeable; **random-key / bottom-k form is mergeable-EXACT** (no per-shard count needed) | **mergeable-exact via random-key/bottom-k sampling**. Algorithm-R + weighted-re-sample = fallback only, bias measurable via KS/chi-square. | escape hatch [LIT-SETTLED]; bias [LIT-SETTLED]/[LIT-EXTRAPOLATED] |
| **Streaming clustering** | partially mergeable (time-tied) | **CF-vector merge is EXACT** (theorem); approximation confined to (a) fading-weight time-alignment (bounded `2^{λΔ}`) + (b) macro-clustering coreset error + (c) pruning divergence | **approximate-merge acceptable** — CF-merge exact, approximation confined & bounded; uniform cross-shard pruning + clock-sync discipline; macro-drift on real data MUST be measured (ARI/NMI/CMM) | CF additivity [LIT-SETTLED]; skew/macro-drift [LIT-EXTRAPOLATED → PRISM-MUST-MEASURE] |

**Headline:** the C7 alternative ("allow non-mergeable primitives via approximations") is **largely unnecessary** —
two of the three non-mergeable primitives (EWMA, reservoir) have **representation-change escape hatches that are
mergeable-exact by theorem**, and the third (clustering) is mergeable-exact at the CF level with only bounded,
confined approximation. The genuine "approximate-merge with extrapolated/unbounded error" case (scalar-EWMA-merge,
Algorithm-R+weighted-merge) is a *fallback*, not a *primary path*. This **strengthens the C7 PRIMARY LEAN** (restrict
to mergeable primitives) by showing the restriction costs almost nothing: pick the mergeable *representation* of each.

## The empirical test plan (concrete, runnable) — summary

Baseline = exact central recompute over the unified ordered stream. Treatment = N-shard partial-aggregate + candidate
merge (run BOTH the exact-form and the approximate-form per primitive). Metrics: EWMA → MARE/RMSE/MAPE + lag;
reservoir → KS/chi-square/Wasserstein/inclusion-bias; clustering → ARI/NMI/purity/centroid-drift/CMM. Generators:
stationary (RandomRBF), diurnal (sinusoidal), drift (RandomRBFGeneratorDrift / SEA / recurring), **adversarial
cross-shard skew (1:1…10:1)** — the decisive OT/MSSP workload. Sweeps: N {1..256} × {uniform,skewed} × {flat,tree};
λ {0.01..0.5}; k {100..10000}; clock-skew Δ (validates the `2^{λΔ}` clustering bound); skew-ratio + drift-severity as
adversarial axes. **Decision criterion is detection-quality, not statistical error:** map error→z-shift `≈ −Δm/σ`,
measure ΔFNR/ΔFPR/ΔPR-AUC of the downstream detector under central vs merged baseline with injected anomalies.
Decision rule: prefer mergeable-exact; accept approximate-merge only if ΔFNR ≤ ~0.5pp AND ΔFPR ≤ ~2pp AND ΔPR-AUC ≥
−0.01 across ALL adversarial regimes; else central-only — which in this residency mesh means **edge-local-only**
(raw cannot cross the conduit), not ship-raw-to-central.

## Consolidated Open Design Questions (for architect/PO — NOT decided here)

1. **Adopt the forward-decay `(U,V)` EWMA representation as backbone?** (Part 2.3). Costs one extra scalar + landmark
   re-basing; makes EWMA mergeable-exact. Affects the C7 §15.4 primitive set + the "model-as-retention-tier" ADR.
2. **Adopt random-key/bottom-k as the canonical edge sampler?** (Part 3.3). Mergeable-exact, no per-shard count relay.
   Replaces Algorithm-R in the C7 toolkit for the edge case. (Crate note: C7 found `streaming_algorithms` 0.3.3 has
   reservoir but low adoption; a first-party bottom-k is ~tens of LOC — verify crate state at build time, do not pin
   from this doc.)
3. **Clustering: uniform cross-shard pruning policy + clock-sync requirement.** (Part 4.3). What Δ (clock skew) bound
   does the mesh guarantee, and what λ ceiling keeps `2^{λΔ}` weight-error negligible? Ties C2 enrollment/transport.
4. **Is a formal privacy guarantee on conduit-crossing aggregates required?** (Part 5.3). If yes → local-DP on
   mergeable DP sketches, a **separate decision from D-C7-1**, not solved by coarsening. Ties D-C2-12 + AD-017.
5. **Acceptance thresholds (ΔFNR/ΔFPR/ΔPR-AUC) ratification.** (Part 6.5). The recommended {0.5pp, 2pp, 0.01} are
   discussion starting points; the human/PO sets the security risk tolerance. FNR-tight is the security default.
6. **"central-only" semantics in a residency mesh = edge-local-only.** (Part 6.5). Confirm that a primitive failing
   the merge test stays edge-local (consumed in-zone, never fanned in), since raw cannot cross the conduit.
7. **Does the test plan become a real Prism test artifact** (a `prism-ml` bench/proptest harness run WHEN building
   edge-ML), or a one-off study? The CF-merge associativity/commutativity proptests (Part 4.4) should be permanent
   regardless.

## Honest Costs & Caveats

- **The escape hatches are the strongest, most useful finding — and they are LITERATURE-SETTLED.** Forward-decay
  mergeability (EWMA) and random-key/bottom-k mergeability (reservoir) and CF additivity (clustering) are all proven
  theorems, not extrapolations. This is high-confidence.
- **The scalar-EWMA and Algorithm-R approximate-merge ERROR BOUNDS are EXTRAPOLATED**, not published. The source is
  explicit: the streaming literature studies timestamped decayed aggregates, not the merging of scalar EWMA states or
  count-based reservoirs. If Prism chooses an approximate-merge fallback, its error envelope is **unproven until
  measured** (Part 6). Do not ship a scalar-EWMA-merge claiming a known error bound — there isn't one in the literature.
- **The `2^{λΔ}` clustering clock-skew bound is extrapolated** from the exponential-fading multiplicative property;
  the DenStream/CluStream papers do not analyze cross-shard clock skew. Bounded and plausible, but Prism-measured.
- **Macro-clustering drift on REAL security data is PRISM-MUST-MEASURE.** The coreset `(1+ε)` guarantee is for
  k-means cost, not for ARI/NMI on security behavioral clusters; the actual cluster-quality drift under adversarial
  skew is the empirical question Part 6 exists to answer.
- **The error→detection-quality mapping and the acceptance thresholds are BESPOKE.** The `Δz ≈ −Δm/σ` model is sound
  but the numeric thresholds (ΔFNR/ΔFPR) are Prism's risk-tolerance call, not a literature constant. The source is
  explicit that this translation "must be crafted specifically for the security context."
- **Privacy: coarsening is NOT a privacy mechanism** (deterministic post-processing). Anyone arguing approximate-merge
  "for privacy" is mistaken on the formal point — it reduces info-theoretic leakage but gives no guarantee. Residency
  is satisfied by aggregate-only movement *regardless* of approximation. These are settled and load-bearing.
- **Implementation correctness ≠ mathematical mergeability.** River's DenStream/CluStream bugs show CF-merge is easy
  to get subtly wrong in code. Prism needs proptest coverage of the merge ops (monoid laws), not just the math.
- **Read-coverage honesty.** Six `perplexity_research` (sonar-deep-research, `reasoning_effort=high`) calls returned
  81K–93K chars each, saved to tool-result files. Each was read substantively from offset 0 (29K–34K tokens, ~62–71%
  of each file) covering the load-bearing theory, techniques, error analysis, and methodology; the tails (largely
  conclusion/restatement + a few metric specifics) were located via Grep, not read verbatim end-to-end — flagged for
  honesty. The single-line JSON format prevented offset-paginated reads of the tails; Grep confirmed the tail topics
  (ARI/centroid-drift; KS/chi-square; conclusions; acceptance-threshold numbers) were present and consistent with the
  synthesized methodology. No high-effort call failed on overload, so no medium-effort fallback was triggered.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY, reasoning_effort=high)** | 6 | (1) mergeable-summaries formal theory + taxonomy (Agarwal et al.; fully-vs-one-way-vs-non-mergeable; monoid framing; where EWMA/reservoir/clustering fall); (2) EWMA approximate-merge techniques + error + the forward-decay mergeability escape hatch (Cormode forward decay, Cohen–Wang, FDCMSS); (3) reservoir merge bias + weighted-re-sample correction + the random-key/bottom-k fully-mergeable substitute (Efraimidis–Spirakis, Cohen–Kaplan, Tirthapura–Woodruff); (4) streaming-clustering CF additivity (BIRCH/CluStream/DenStream) + time-reconciliation + macro-clustering coreset error; (5) residency/privacy of aggregate-only movement + sketch privacy + DP-mergeable sketches + privacy≠residency; (6) the concrete empirical test-plan methodology + error metrics + generators + sweeps + error→detection mapping |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (no single-library API question; the topic is algorithmic/theoretical prior art. No new Rust version claim was made — crate state for bottom-k/forward-decay is deferred to build-time verification per Open Question 2, NOT pinned from training data) |
| Tavily (all) | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~2 areas | Flagged inline: the `2^{λΔ}` clock-skew bound interpretation (derived from the cited exponential-fading property, [LIT-EXTRAPOLATED]); the recommended numeric acceptance thresholds in Part 6.5 (PRISM-MUST-DESIGN, presented as discussion starting points, not literature constants) |

**Total MCP tool calls:** 6 (all `perplexity_research` at `reasoning_effort=high`, the mandated PRIMARY tool for
non-trivial depth research).
**Compliance gate:** PASS — ≥1 `perplexity_research` call (6 of them, all reasoning_effort=high). No MCP-UNAVAILABLE
escalation needed.
**Training-data reliance:** **low** — every algorithm/theorem/technique claim is web-grounded via the six high-effort
deep-research calls; the two training-data areas are explicitly flagged and are either extrapolations from a cited
property (`2^{λΔ}`) or discussion-input numbers the human must ratify (acceptance thresholds), not load-bearing facts.
**Version-verification gate:** N/A — this pass makes NO Rust version claims; the one crate-state note (Open Q2,
bottom-k sampling crate) explicitly defers to build-time crates.io verification rather than pinning from training data.
**Resilience:** all 6 high-effort calls succeeded on first attempt; the retry-then-fallback-to-medium contingency was
not needed.
