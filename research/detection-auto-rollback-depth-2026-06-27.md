---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day-2-vision-side-analysis
relation: OUT-OF-BAND — SEPARATE from the live VSDD factory pipeline
scope: >
  DEPTH pass on the AUTO-ROLLBACK CONTROL LOOP for detection rules (matured-vision §14.4
  staged rollout). Extends the C6 survey pass (detection-engine-depth-2026-06-27.md Q4),
  which the human deferred pending deeper research on the auto-rollback decision. This pass
  goes deep on the rollback MECHANISM; it does NOT re-survey the shadow/canary primitives C6 covered.
non_contradiction_basis:
  - "detection-engine-depth-2026-06-27.md Q4 (C6) — READ FIRST; this pass is consistent with and extends it"
  - "C6 Q4 settled-at-survey-level: NO SIEM ships Flagger-style auto-rollback for detection rules; assemble from primitives"
  - "C6 §14.1 lifecycle: shadow → canary → production → deprecated (NOT relitigated)"
  - "C6 Q4 LEAN guardrail: auto-rollback may DISABLE ROUTING but rule keeps EVALUATING in shadow; auto-disable of evaluation requires human sign-off"
  - "C6 Q5 (Sigma status levels) + C6 Q6 (RBA-over-suppression, no-silent-masking) — consistent reuse here"
  - "matured-vision-day2-requirements.md §14.4 (staged rollout) — the human DEFERRED the auto-rollback decision"
deferred_fork_being_researched:
  - "What 'rollback' DOES: demote-to-shadow (keep evaluating, stop routing) vs full-disable (stop evaluation) vs revert-to-prior-rule-version"
  - "How the circuit-breaker is designed (signals, thresholds, hysteresis, human boundary)"
  - "The subtlest hazard: distinguishing a BROKEN/noisy rule from a rule CORRECTLY catching a real attack campaign"
settled_decisions_NOT_relitigated:
  - "detection-as-query (PrismQL); MATCH_RECOGNIZE custom operator; correlation state on RocksDB; multi-tenant; alert model + routing + destinations (all per C6 frontmatter)"
caveat: >
  CAPTURE artifact. Leans are discussion input only — NOT decisions. Numbers/epics/ADRs
  remain the architect's at morph. This file does not modify STATE.md, SESSION-HANDOFF.md,
  the live ADR registry, any live spec/BC/story, RESEARCH-INDEX.md, or any prior research
  file (including detection-engine-depth-2026-06-27.md), and was not git-added or committed.
---

# Detection Auto-Rollback Control Loop — DEPTH Research Pass (Day-2 Vision Side-Analysis)

**Date:** 2026-06-27 · **Reasoning effort:** `perplexity_research` at `high` (all 6 primary calls succeeded on first attempt; no overload fallback was needed this run).

This pass researches the **AUTO-ROLLBACK CONTROL LOOP** that C6 (§14.4) deferred. C6 established at survey level that no SIEM ships Flagger-style auto-rollback for detection rules; it must be assembled from primitives, and the open fork was (a) what "rollback" *does* and (b) how the circuit-breaker is *designed*. **This pass does NOT re-survey the shadow/canary primitives** — it goes deep on the rollback mechanism, the no-ground-truth detection problem, the circuit-breaker design, the rollback-action-semantics fork, the promotion gates, and the legitimate-spike-vs-noise discrimination.

Six research questions, each with cited prior art and a discussion **LEAN**. Web findings flagged inline; `[model-knowledge]` marks model-supplied reasoning; `[INCONCLUSIVE]` marks where sources fell short. Source families are named in the **Sources** appendix; the full numbered-citation prose is in the saved transcripts (paths in Research Methods).

> **Cross-tool reading note.** All six Perplexity deep-research responses returned source-grounded prose with numbered citations and exceeded the inline token cap, so each was saved to a transcript file and analyzed by targeted extraction. The portions read cover every substantive parameter, finding, and caveat cited below; the Honest Costs section flags the two places where extraction was partial.

> **Non-contradiction with C6.** This pass is fully consistent with detection-engine-depth-2026-06-27.md Q4. Where C6 took a survey-level LEAN ("demote-to-shadow, never silently delete; auto-disable requires human sign-off"), this pass goes deeper and **confirms and sharpens** that LEAN with concrete prior art (revert-to-last-known-good as the dominant pattern; SOAR human-approval-gate guidance for coverage-reducing actions; the error-asymmetry argument). It does not reverse any C6 position.

---

## Q1 — Automated Canary Analysis (ACA): the deepest prior art for the control loop

### 1.1 The four-stage ACA control loop (cited)

Every surveyed ACA system — **Netflix Kayenta/Spinnaker**, **Flagger**, **Argo Rollouts** — is a closed feedback loop with four stages: (1) **metric collection** (pull canary + baseline time-series from Prometheus/Atlas/Stackdriver, distinguished by labels/tags); (2) **data preparation** (validate existence, handle NaNs, optional outlier removal); (3) **analysis** (statistical compare for Kayenta; threshold compare for Flagger/Argo, at a fixed interval for a fixed count); (4) **decision** (aggregate to a score/verdict → continue / pause-for-human / roll back). [Kayenta][Flagger][Argo]

### 1.2 Kayenta / Spinnaker — statistical judge (the most rigorous reference)

- **The judge is the `NetflixACAJudge`.** It runs four steps: data validation (empty array → metric labeled `NODATA`, does NOT auto-fail — an error counter with no errors is legitimately empty), data cleaning (NaN strategy `replace`/`remove`; IQR/Tukey-fence outlier removal at a configurable factor, clamped to the 1st/99th percentile), metric classification, and score computation. [Kayenta-Judge]
- **Statistical test = Mann–Whitney U (Wilcoxon rank-sum), nonparametric.** It computes a **Hodges–Lehmann estimator** of the canary-vs-baseline difference and a **98%-confidence interval** around it. A metric is classified **High**/**Low** only when ALL THREE hold: (a) the 98% CI falls entirely outside a **tolerance band** of ±25% of the |Hodges–Lehmann estimate| (a dead-zone that ignores small differences); (b) the **effect size** exceeds the configured `allowedIncrease`/`allowedDecrease`; (c) the metric direction permits it. Otherwise **Pass**. The 98% confidence level appears **fixed in the judge, not a user knob**. [Kayenta-Judge][model-knowledge: that 98% is non-configurable is inferred from the absence of any documented knob]
- **Scoring → verdict.** Per metric group: score = (Pass count / total non-`NODATA` count) × 100. Summary score = weighted average of group scores. Classified by **two thresholds**: `score ≥ passThreshold` → Pass; `marginalThreshold ≤ score < passThreshold` → Marginal; `score < marginalThreshold` → Fail (comparisons inclusive). Best-practice **starter values: marginal 75, pass 95** (explicitly "not universally correct — tune per app"). [Kayenta-Judge][Spinnaker-BP]
- **Automatic-fail overrides** (summary score forced to 0): any **critical-flagged** metric classified High/Low beyond its critical effect-size threshold (a "tripwire"), OR **≥50% of all metrics are `NODATA`** (monitoring broadly broken). [Kayenta-Judge]
- **Timing parameters:** `lifetime` (hours), `interval` (how often the judge runs), `delay` (warm-up; best-practice = 0 unless the app needs warm-up), `lookbackType` (growing = back to start; sliding = last interval only), metric `step`. **Number of judgment runs ≈ lifetime / interval** (best-practice example: 3h lifetime / 1h interval = 3 runs). **Minimum sample size: Kayenta needs ≥50 time-series points per metric per run** for the statistics to be reliable — this is the binding constraint coupling step × lifetime. **A run scoring below `marginalThreshold` fails the whole canary immediately; only the FINAL run must reach `passThreshold`.** [Spinnaker-BP][Kayenta-Timing]
- **Rollback trigger:** Spinnaker (not Kayenta) acts on the verdict — Pass → promote/continue; Marginal → typically pause for human judgment; Fail → abort + route all traffic back to the production baseline (**revert to baseline**). [Netflix-Blog][Kayenta-GCP]

### 1.3 Flagger — threshold-based control loop (the cleanest parameter set)

The `Canary` CR `analysis` block carries the concrete knobs: [Flagger-Spec][Flagger-AppMesh]
- **`interval`** — schedule interval, **default 60s** — how often the analysis loop runs (calls webhooks, checks metrics).
- **`threshold`** — **"max number of failed metric checks before rollback"** — the failure counter. Example specs use **5–10**.
- **`maxWeight`** — max % traffic to canary (e.g. 50). **`stepWeight`** — increment per successful run (e.g. 5 or 10).
- **`progressDeadlineSeconds`** — **default 600s (10 min)** — max time for the canary to make progress before automatic rollback (e.g. pods never become ready).
- **`primaryReadyThreshold` / `canaryReadyThreshold`** — % pods that must be ready before progressing (warm-up safeguard).
- **Metrics:** two built-ins (HTTP request-success-rate, request-duration) + custom `MetricTemplate` (Prometheus query → float compared to a `thresholdRange`). Example: success-rate ≥ 99%, duration ≤ 500ms.
- **Rollback semantics:** "on each run, Flagger calls webhooks, checks metrics, and if the failed-checks threshold is reached, **stops the analysis, routes traffic back to the primary, and scales the canary to zero.**" [Flagger-Spec][Flagger-AppMesh]
- **`[INCONCLUSIVE]`** — the docs say "max number of failed checks" but do NOT state whether the counter is **consecutive** or **cumulative-across-the-run**, nor whether it **resets after a successful run**. The phrasing (no "consecutive") leans cumulative, but this is an informed inference, not documented. [INCONCLUSIVE on Flagger counter-reset semantics]

### 1.4 Argo Rollouts — the most explicit hysteresis controls

`AnalysisTemplate` exposes the clearest anti-flap knobs: [Argo-Analysis]
- **`interval`** + **`count`** — poll frequency and total measurements (count unset → runs indefinitely).
- **`failureCondition` / `successCondition`** — PromQL/expression predicates per measurement.
- **`failureLimit`** — **max failed measurements before the analysis fails. Default 0 = tolerate zero failures (one failure fails the run); set −1 to disable.** Example: error count ≥ 10 marks a measurement failed; failureLimit 3 → run fails after 3 failed measurements.
- **`consecutiveSuccessLimit`** — **required number of CONSECUTIVE successes for the run to succeed. Default 0 = disabled.** This is the explicit hysteresis lever: e.g. "4 consecutive successes with at most 3 total failures."
- **Verdict matrix** (failureLimit has priority): failureLimit violated → **Fail** (regardless of successes); failureLimit not violated + consecutiveSuccessLimit satisfied → **Success**; neither → **Inconclusive** → **rollout PAUSES for human intervention** (not auto-rollback). [Argo-Analysis]
- **Rollback action:** a failed inline analysis marks the rollout **Degraded** and **sets canary weight back to 0** (revert to stable). Declarative/idempotent. [Argo-Analysis]

### 1.5 Hysteresis / anti-flap mechanics (cited synthesis)

Three distinct anti-flap mechanisms across the systems: (a) **statistical hysteresis** (Kayenta's 98% CI + ±25% tolerance band + effect-size gate require both *significant* and *practically meaningful* change before flagging); (b) **counter hysteresis** (Flagger `threshold`, Argo `failureLimit` + `consecutiveSuccessLimit` require *persistence* — multiple bad measurements, or a *streak* of good ones); (c) **timing hysteresis** (warm-up `delay`, `progressDeadlineSeconds`, ≥50-sample minimum, bake/soak windows prevent acting on too-little data). The **minimum-sample-size rule (≥50 in Kayenta)** is the single most transferable design constraint — you cannot judge a spike you have not sampled enough times. [Kayenta-Judge][Spinnaker-BP][Argo-Analysis][Flagger-Spec]

### 1.6 LEAN (discussion input only)

Borrow the **Argo Rollouts control-loop shape** for Prism's detection auto-rollback because its hysteresis is the most explicit: an evaluation **`interval`**, a **`failureLimit`** (= max bad-windows before tripping), and a **`consecutiveSuccessLimit`** (= clean-windows required before re-promotion after a half-open trial). Borrow the **Kayenta statistical discipline** for the *trip decision*: don't trip on a raw count — trip on a change that is both **statistically significant** (Q2 change-detector) AND **practically meaningful** (exceeds a tolerance band over the shadow baseline), and **require a minimum number of evaluation windows (the ≥50-sample analog)** before the breaker is even allowed to trip. Borrow Flagger's **`progressDeadlineSeconds`** idea as a per-stage **bake-time ceiling**. The **Marginal/Inconclusive → pause-for-human** path (Spinnaker Marginal, Argo Inconclusive) is the prior art for the §14.4 human-in-the-loop boundary — ambiguous verdicts escalate, they do not auto-act (this is the Q6 hold-and-escalate hook).

---

## Q2 — Detecting a bad rule rollout WITHOUT ground-truth labels (the detection-specific hard problem)

### 2.1 The zero-label constraint (cited, decisive)

The defining constraint: at/after rollout the platform has **no ground truth** about which of the new rule's alerts are TP/FP. Analyst dispositions lag **tens of minutes to hours** behind. Alert volume for a misbehaving rule can spike "by orders of magnitude relative to its pre-deployment or shadow-mode baseline," producing an **alert storm that saturates the SOC long before enough alerts are labeled to estimate FP-rate.** Therefore the rollout-health signal must come from **unlabeled stream statistics** (alert volume, alert-feature distributions, entity-cardinality), with **dispositions arriving later as delayed confirming feedback.** [Change-Survey][Drift-Survey]

**Critical corollary (cited):** the classic ML concept-drift detectors **DDM and EDDM explicitly require labeled error statistics and CANNOT be applied directly in the zero-label regime of a fresh rule** — though their sequential statistical logic is instructive. This rules out the "obvious" drift detector for the real-time trip decision and forces the unsupervised change-detector family. [Drift-Survey]

### 2.2 The online change-point / rate-spike detector family (cited parameters)

All operate on the unlabeled alert-volume (and cardinality) stream; the field is the same one used for SPC and ML drift detection: [Change-Survey][Drift-Survey][SPC]

- **CUSUM (cumulative sum control chart).** Recursively accumulates signed deviations of each observation from a reference value (the in-control mean) **minus a drift parameter `v`** that encodes the magnitude of change it is tuned to detect; the one-sided chart resets negative sums to 0 and **alarms when the cumulative sum exceeds a threshold `h`**. Tuned via `v` and `h` to hit a target **`ARL₀`** (average run length to false alarm under normal operation) while minimizing **`ARL₁`** (latency to detect a specified change). Best for **abrupt jumps** — directly suited to a post-deploy volume spike. [Change-Survey][SPC]
- **Page–Hinkley test.** A CUSUM variant for detecting a change in the mean of a Gaussian-ish signal; parameterized by a magnitude/`delta` (allowed drift) and a detection threshold `lambda`. Suited to gradual drift as well as spikes. [Change-Survey][Drift-Survey]
- **EWMA control chart.** `Z_t = λ·X_t + (1−λ)·Z_{t−1}` (smoothing parameter `λ ∈ (0,1]`), with control limits derived from a control constant **`L`** chosen to hit a target `ARL₀`. **Tuned to detect SMALL/gradual shifts** rather than abrupt massive jumps — complementary to CUSUM, less direct for a sudden storm. [SPC][Change-Survey]
- **ADWIN (adaptive windowing).** Maintains a window of recent items under a stationarity assumption; when the older sub-window becomes statistically inconsistent with the newer sub-window, the old part is **dropped** and a change is flagged. **Can track a non-stationary baseline** (adapts the window) — useful for diurnal SOC patterns. [Drift-Survey]
- **BOCPD (Bayesian Online Change-Point Detection, Adams–MacKay).** Maintains a posterior over the **run-length** (time since last change-point) governed by a **hazard rate**; can incorporate a non-stationary baseline by adjusting the hazard. Heavier but principled. [Change-Survey]

**Shared tradeoff (cited):** all are characterized by the **detection-latency vs false-alarm tradeoff** formalized as ARL₀/ARL₁ — tune for fewer false alarms (high ARL₀) and you detect slower (high ARL₁), and vice-versa. [Change-Survey][SPC]

### 2.3 The detection-specific signals (cited)

Formalize the rule's output as streaming random variables: alert-rate `λ_t`, distinct-entity cardinality `U_t`, raw alert count `N_t`. [Change-Survey]
- **Volume spike vs shadow baseline.** A misbehaving rule = significant increase in `λ_t` at/shortly after deploy, "significant" defined relative to the natural variability captured in the shadow-mode reference window. **This is the primary signal.**
- **Cardinality explosion vs duplicate storm.** A bad rule may make `U_t` rise sharply (firing on thousands of distinct entities), OR create a **"duplicate storm" where `N_t` grows while `U_t` stays flat** (repeated alerts on the same entities). The `N_t`/`U_t` ratio discriminates these. [Change-Survey]
- **Disposition lag as delayed validation.** Dispositions arrive later and can be **harnessed as delayed feedback** to *validate or refine* the unlabeled suspicion (and only then could DDM/EDDM be applied) — but the latency is "likely too long for first-line protection," so the real-time trip rests on volume + cardinality, with labels as confirmation. [Change-Survey]

### 2.4 The shadow-mode baseline = the reference window (cited, ties C6)

The corpus is explicit: running a new rule in **shadow/observation-only mode** before active enforcement lets the platform **estimate the in-control distribution of per-rule alert counts, entity cardinalities, and feature descriptors**, which "can be treated as **reference windows** for subsequent online change detection when the rule is promoted." This is exactly the reference-vs-current-window structure that concept-drift detection uses — **the C6 shadow stage IS the baseline-fitting stage for the Q2/Q3 breaker.** [Change-Survey][Drift-Survey]

### 2.5 Tie to C7 drift-detection (cited, as the task requested)

The change-detector family here (**CUSUM, Page–Hinkley, ADWIN, EWMA, BOCPD**) is **the same statistical family used for ML concept-drift / data-drift detection** — the surveys present them jointly, and DDM/EDDM (the labeled drift detectors) are CUSUM-lineage. The difference is only the **target stream**: C7 drift detection watches model-input/output distributions; Q2 watches alert-volume/cardinality. Prism can therefore build **one change-detection primitive** and point it at either target — a real architectural economy. [Drift-Survey][Change-Survey]

### 2.6 LEAN (discussion input only)

Use the **shadow-mode window as the reference baseline** (C6 §14.1 shadow stage does double duty — it fits the in-control distribution). For the real-time trip signal, **CUSUM on alert-rate `λ_t` is the primary detector** (tuned for abrupt jumps; `v`/`h` set from the shadow baseline's variability to a target `ARL₀`), with **a cardinality monitor on `U_t` and the `N_t`/`U_t` ratio** as a second independent signal (catches both cardinality-explosion and duplicate-storm). **Do NOT rely on DDM/EDDM for the trip** — they need labels Prism won't have in time; reserve disposition labels as **delayed validation** that *confirms* a trip was correct (and tunes thresholds) rather than *driving* it. **Build the change-detector as a shared primitive with C7 drift detection** (same family, different target). ADWIN is the natural choice if the baseline is strongly diurnal (self-adapting window); CUSUM is the natural choice for fast spike detection — likely **both**, gated together (feeds Q3 multi-signal gating).

---

## Q3 — Circuit-breaker design for detection-rule routing

### 3.1 The classic pattern + concrete parameters (cited)

The circuit-breaker pattern (Nygard "Release It!", Fowler, Hystrix, resilience4j, Azure docs) has **three states: closed / open / half-open.** Closed = traffic flows; on sustained failures the breaker **opens** (short-circuits); after a cool-down it goes **half-open**, permits a limited number of trial calls, and **re-closes if they succeed or re-opens (restarting the cool-down) if they fail.** [Fowler][Azure-CB]

**resilience4j concrete parameters:** [Resilience4j]
- `failureRateThreshold` (%) — at/above this failure rate the breaker opens.
- `slidingWindowType` (`COUNT_BASED` | `TIME_BASED`) + `slidingWindowSize` — last N calls, or calls in last N seconds, aggregated for the rate.
- `minimumNumberOfCalls` — **minimum sample before the breaker may trip** (prevents tripping on tiny samples — the resilience4j analog of Kayenta's ≥50 rule and Hystrix `requestVolumeThreshold`).
- `waitDurationInOpenState` — **cool-down**: how long the breaker stays open before half-open.
- `permittedNumberOfCallsInHalfOpenState` — number of trial calls in half-open; the half-open phase is **a fresh probing phase, not a continuation of prior failure history.**
- `slowCallRateThreshold` / `slowCallDurationThreshold` — trips on "slow" calls too.

**Hystrix concrete parameters:** `errorThresholdPercentage` (trip threshold), `requestVolumeThreshold` (min requests in the rolling window before it can trip), `sleepWindow` (cool-down before half-open trial). [Hystrix]

### 3.2 Mapping onto a detection rule's ROUTING (cited)

The corpus maps this cleanly (and notes no source ships it pre-built for detection rules — consistent with C6 Q4): [Detection-CB-Synthesis]
- **CLOSED** = rule **routes alerts normally**.
- **OPEN** = rule **stops routing (demoted)** because an alert-storm/spike tripped the breaker.
- **HALF-OPEN** = **trial re-enable** to test whether the spike subsided (a limited window of routed alerts evaluated against the breaker).
- The breaker sits **between detection rules and alert routing**, and **coexists with** (does not replace) existing notification throttles and downstream API rate limits (PagerDuty/ticketing). [Detection-CB-Synthesis]
- The resilience4j analogs for detections: `failureRateThreshold` → an **alert-rate threshold**; `minimumNumberOfCalls` → a **minimum number of alerts / minimum evaluation window** before the breaker may open; `waitDurationInOpenState` → the **cool-down** before a trial re-enable. [Detection-CB-Synthesis]

### 3.3 Threshold selection, window, hysteresis, multi-signal, per-tenant (cited + synthesis)

- **Absolute cap vs relative multiplier:** the design choice is an **absolute alert-rate cap** (alerts/hour) vs a **relative-to-shadow-baseline multiplier** (e.g. >Nx the shadow baseline). The sources discuss alert-rate spikes and quality indicators (FP-closure rate, analyst feedback) as trip inputs; the absolute-vs-relative framing is the natural circuit-breaker translation. [Detection-CB-Synthesis][model-knowledge: the explicit absolute-vs-relative dichotomy is the standard rate-limiter framing applied here]
- **Hysteresis / cool-down:** the sources are explicit — "without hysteresis, an alerting system may switch/trigger repeatedly as a value hovers near a threshold." The cool-down (open-state wait) + half-open trial **is** the hysteresis that prevents flap-rollback-flap; cool-down and half-open phases should be **operator-visible and adjustable**. [Detection-CB-Synthesis][SRE-Flap]
- **Multi-signal gating (N-of-M):** combine the Q2 signals — require, e.g., volume-spike AND cardinality/duplicate anomaly before tripping — so a single noisy signal doesn't trip the breaker. (The sources support combining alert-rate spikes with quality indicators; N-of-M is the standard multi-signal formalization.) [Detection-CB-Synthesis][model-knowledge for the explicit N-of-M framing]
- **Per-tenant vs global breaker:** the corpus notes **per-tenant breakers increase management overhead and state complexity** but are the right granularity when a rule misbehaves for one tenant but is fine for others. Since **Prism is multi-tenant and the canary unit is the tenant** (C6 Q4 LEAN, Q5 below), a **per-tenant breaker** is the natural fit — a rule trips for the noisy tenant without blinding it everywhere. [Detection-CB-Synthesis]

### 3.4 LEAN (discussion input only)

Model the §14.4 auto-rollback as a **per-tenant alert-rate circuit-breaker on the ROUTING path** (closed=route / open=demote-routing / half-open=trial-route), sitting between detection and routing and coexisting with downstream throttles. **Trip on a RELATIVE-to-shadow-baseline multiplier** (e.g. alert-rate > Nx the rule's shadow baseline for that tenant) rather than an absolute cap — relative respects per-rule, per-tenant natural volume and is self-calibrating from the Q2 baseline; pair it with an **absolute safety cap** as a backstop for the degenerate runaway case. **Gate the trip on N-of-M signals** (volume-spike from CUSUM + cardinality/duplicate anomaly) to avoid single-signal flaps. Set **`minimumNumberOfCalls`-equivalent** (minimum alerts / minimum windows) before the breaker may open (the ≥50-sample discipline). The **cool-down (`waitDurationInOpenState`) + half-open trial + `consecutiveSuccessLimit`-style clean-window requirement** is the anti-flap hysteresis. **Per-tenant breaker is the default** (Prism is multi-tenant); a global breaker is the escalation when the same rule trips across many tenants simultaneously (which itself is a strong "rule is broken, not one tenant under attack" signal — feeds Q6).

---

## Q4 — Rollback ACTION semantics (the deferred fork, researched)

### 4.1 What progressive-delivery systems actually DO on rollback (cited)

**The dominant pattern across ALL surveyed systems is REVERT-TO-LAST-KNOWN-GOOD:** "restore the last-known-good version and stop exposing the new version" — not merely pause without reverting, and not silently destroy the new version with no stable alternative. [Rollback-Synthesis]
- **Spinnaker/Kayenta:** revert-to-last-known-good — the previous server group becomes active and receives traffic; the new problematic server group is disabled. [Rollback-Synthesis]
- **Argo Rollouts:** failed analysis → Degraded → canary weight set to 0 (revert to stable); declarative desired-state model → **effectively idempotent** (re-running the rollback transitions to the same stable failed state; no auto-reattempt). [Argo-Analysis][Rollback-Synthesis]
- **Flagger:** route traffic to primary, scale canary to 0 — "revert to last-known-good + disable new version for future traffic." [Flagger-Spec][Rollback-Synthesis]
- **Feature flags (LaunchDarkly/Unleash):** revert the flag to its previous/last-known-good variation; **idempotent as a state transition** (reverting then reverting again keeps the same config). Operates at decision-logic level, so **no in-flight network requests** to worry about. [Rollback-Synthesis]
- **In-flight handling:** generally NOT explicitly documented; the inference across systems is that rollback affects **future** traffic/decisions, while **in-flight requests may finish on the canary** (e.g. Argo canary pods serve in-flight until they drain). Idempotency is **inferred from the declarative model**, not always stated outright. [Rollback-Synthesis][INCONCLUSIVE on explicit in-flight semantics]

### 4.2 The three detection-rule rollback semantics (cited, the fork)

The corpus directly contrasts the three: [Rollback-Synthesis]
- **(a) DEMOTE-TO-SHADOW** — rule **keeps evaluating but stops routing** alerts to analysts; continues producing shadow telemetry. The sources describe exactly this: a "shadow state where it continues to evaluate events but stops raising alerts," which **maintains coverage tracking and an audit trail even after disabling alerting**, "closely resembling shadow evaluation." It "**reduces noise while preserving coverage and auditability.**"
- **(b) FULL-DISABLE** — rule **stops evaluating entirely**: eliminates compute and noise **at the cost of a coverage blind spot where attacks may go undetected AND unlogged.** In regulated environments this is "a blind spot that must be justified and documented." [Rollback-Synthesis]
- **(c) REVERT-TO-PRIOR-RULE-VERSION** — roll back to the last-known-good version of the same rule: avoids the blind spot **but may reintroduce the prior version's known deficiencies and may not fix a fundamental design flaw** in the new logic. [Rollback-Synthesis]

### 4.3 What happens to already-emitted findings (cited)

Rollback "does not retract already-emitted alerts but instead affects only **future** notifications/evaluations." Already-routed findings persist (they may be annotated for context, but are not retracted). The rollback operation is **idempotent** as a state transition. [Rollback-Synthesis]

### 4.4 The human-in-the-loop boundary (cited, decisive)

The corpus is explicit and aligns with C6 Q4: **fully automated rollback should be constrained to reverting to safer states or suppressing routing/notifications; more destructive actions such as auto-DISABLING a detection rule should require explicit human sign-off.** [Rollback-Synthesis]

The strongest grounding is the **SOAR analogy:** SOAR best practice "balances full automation with human approval gates and requires analyst review for actions that could disrupt business operations, placing approval gates before critical steps such as account deactivation or network isolation." **Auto-disabling a detection (which silently reduces security coverage) is analogous to deactivating an account or isolating a host — it removes a layer of defense and therefore belongs behind a human-approval gate.** [Rollback-Synthesis][SOAR]

### 4.5 LEAN (discussion input only) — the fork resolved

**Take a clear position: DEMOTE-TO-SHADOW is the default auto-rollback action.** Rationale, fully grounded:
1. It is the **only one of the three that preserves coverage AND auditability** — the rule keeps evaluating, so there is no blind spot and a full audit trail persists (the cited "shadow evaluation maintains coverage tracking" finding). [Rollback-Synthesis]
2. It removes the *harm* the auto-rollback exists to stop (analyst noise / alert storm) by cutting **routing**, while incurring only the modest *cost* of continued evaluation compute. The blind-spot risk of full-disable is **strictly worse for a security tool** than the compute cost of keep-evaluating. [Rollback-Synthesis]
3. It respects the **error asymmetry** (Q6): if the spike was actually a real attack the rule correctly caught, demote-to-shadow **still detects and logs it** — the worst case is delayed analyst visibility, not blinded coverage. **Full-disable in that scenario is catastrophic** (the rule that caught the campaign is silenced at the worst possible moment). This single point is the decisive argument against auto-full-disable.

Secondary positions:
- **REVERT-TO-PRIOR-VERSION is the right *promote-time/manual* remediation, not the *auto* action** — it requires knowing the prior version was good (true after a canary, not guaranteed for a brand-new rule with no prior version) and it may reintroduce old deficiencies. Offer it as a **one-click human action** in the console, not the automatic reflex. [Rollback-Synthesis]
- **FULL-DISABLE must require explicit human sign-off** (the SOAR coverage-reducing-action gate). Auto-full-disable is forbidden by the same logic that gates SOAR account-deactivation. [Rollback-Synthesis][SOAR]
- Rollback is **idempotent** (declarative state: rule-state = `shadow`) and **does not retract already-emitted findings** (annotate, don't retract). [Rollback-Synthesis]

This **confirms and sharpens C6 Q4's survey LEAN** with concrete prior art; it does not reverse it.

---

## Q5 — Promotion gates (shadow → canary → production)

### 5.1 Auto-gated vs human-gated forward transitions (cited)

Progressive-delivery systems make some forward steps **auto-gated on metrics** and others **human-gated**: [Argo-Analysis][Flagger-Spec][Promo-Synthesis]
- **Argo Rollouts:** a `pause` step with a **duration** auto-resumes after the bake; a `pause` step with **no duration pauses indefinitely until a human runs `kubectl argo rollouts promote`** — an explicit **human approval gate** before exposing more users. Inline analysis between pauses gates the *automated* traffic increase on metric thresholds. [Argo-Analysis]
- **Spinnaker:** Marginal canary scores route to **manual judgment** stages; Pass auto-continues. [Promo-Synthesis]
- **Flagger:** fully metric-gated `stepWeight` increase up to `maxWeight` (auto-promotion at each shift), with no built-in human gate (it's the "fully automated" end of the spectrum). [Flagger-Spec]
- **Bake/soak time** = "the period during which a new version runs under controlled traffic while metrics are collected to inform promotion," ending once success criteria are satisfied. [Promo-Synthesis]

### 5.2 Progressive scope: percentage vs cohort/ring vs tenant (cited)

- **Percentage:** `stepWeight` progression (e.g. 5%→25%→50%→100%) with `iterations`/`interval`. [Flagger-Spec][Argo-Analysis]
- **Ring/cohort deployment** (the multi-tenant-relevant pattern): the corpus describes **deployment rings** — start in a ring/cohort while the rest stay on stable, promote ring-by-ring; **tenant and ring-based rollout extends this to cohort-based promotion in a security platform.** Gates are made **explicit in the deployment spec** rather than left to implicit operator judgment. [Promo-Synthesis]
- This validates **tenant-by-tenant as the canary unit** for Prism (one tenant → a few tenants → all tenants) — the ring analog with the tenant as the ring. [Promo-Synthesis]

### 5.3 Detection-as-code lifecycle + Sigma status (cited, ties C6)

- **Sigma `status` levels** encode maturity: **`experimental` → `test` → `stable`** (plus `deprecated`/`unsupported`); rules are refined and noise-reduced before promotion to high-sensitivity production alerting. CI maps these to deploy targets. [Sigma-Status][Promo-Synthesis]
- Detection platforms apply **"soak mode" for detections — matches are recorded but alerts are not generated** (= C6 shadow stage) — and treat rules as **versioned artifacts with explicit lifecycle states and promotion workflows**, GitOps-friendly. [Promo-Synthesis]
- Promotion criteria (FP-rate, alert volume, precision, MTTD) are **computable but organizational metrics, not standardized product gates** (consistent with C6 Q4) — and the *promotion to stable/production* step is where human review is typically required. [Promo-Synthesis][INCONCLUSIVE on a standardized auto-gate metric set — the corpus describes the metrics but not a vendor-standard threshold]

### 5.4 LEAN (discussion input only)

Map Prism's §14.1 lifecycle to **Sigma-style status with mixed gating**: **shadow → canary** auto-gates on the Q2/Q3 metrics (no storm, FP-proxy within tolerance over a bake window) — this transition is *safe to automate* because canary is scope-limited; **canary → production** requires **human sign-off** (the Argo no-duration-pause analog), because widening to all tenants is the high-blast-radius step. Use **tenant-by-tenant as the canary unit** (the ring pattern; C6 Q4 LEAN). Carry the **gate thresholds in the §14.1 `quality` block** so they are explicit-in-spec, not implicit operator judgment. Per-stage **bake-time** (a `progressDeadlineSeconds`-style ceiling) bounds how long each stage soaks. **Promotion gates are the FORWARD dual of the Q3 breaker** (same metrics; one promotes, one demotes) — build them on the same signal pipeline.

---

## Q6 — False-rollback / flapping + the legitimate-spike-vs-noise discrimination (the subtlest hazard)

### 6.1 The error asymmetry (cited, the governing principle)

The corpus states it plainly: **the same high-alert-volume pattern can mean either a detector correctly catching a real campaign OR a broken detector producing noise** — it is genuinely ambiguous from volume alone. And the **error asymmetry is severe: auto-disabling a detector during a real campaign is substantially worse than tolerating a period of noise.** This asymmetry "makes automated rollback logic in SOC platforms much more perilous than analogous logic in application deployment" and "**argues strongly for hold-and-escalate patterns and for treating rollback as a last resort informed by corroborating signals, sustained noise indicators, and human confirmation rather than as a fully autonomous reaction to any surge in alerts.**" [Spike-Synthesis][SRE-Alert]

### 6.2 Incident-vs-noise discrimination signals (cited)

How SOC/anomaly systems distinguish real incident from noise: [Spike-Synthesis]
- **Corroboration from independent rules:** "if multiple independent rules fire on the same entities or in the same time window, correlation logic may elevate these into a higher-confidence incident rather than treating them as noise." A spike in a **single** rule with **no corroboration** is more likely noise. **This is the strongest discriminator.** [Spike-Synthesis]
- **Threat-intelligence matches / known-campaign indicators** corroborate that a spike is real. [Spike-Synthesis]
- **Entity-graph concentration vs dispersion:** activity **concentrated on a small set of (especially high-value) entities** suggests a real incident; **uniform dispersion across many unrelated entities** suggests a noisy/mis-tuned rule. This directly maps to the Q2 `N_t`/`U_t` and cardinality signals (uniform dispersion = high cardinality with no clustering). [Spike-Synthesis]
- **Asset criticality:** spikes on high-criticality assets with corroboration → likely campaign; spikes on low-criticality entities with benign histories → less urgent / candidate for rule review. [Spike-Synthesis]
- **Caveat:** the sources apply these to **alert triage/escalation**, and "the surveyed sources do **not** explicitly extend them to automated rule rollback; that step remains conceptual." [Spike-Synthesis][INCONCLUSIVE on a documented spike→auto-rollback discriminator — Prism would be building novel territory here]

### 6.3 Anti-flapping (cited)

SRE/monitoring anti-flap practice transfers directly: flapping (rapid open↔resolved toggling) "creates toil and confusion," "is often indicative of thresholds too tight or evaluation windows too short," and **anti-flap relies on hysteresis, extended evaluation, grouping, and human oversight.** Decisions about **modifying configuration — including disabling alerts or changing thresholds — are typically made by human responders, not automation.** Prevent oscillation via cool-down, exponential backoff on repeated trips, and **requiring human confirmation before re-promotion after a rollback.** [SRE-Alert][SRE-Flap][Spike-Synthesis]

### 6.4 The safe action on ambiguity (cited)

When the system cannot confidently tell broken-rule from real-attack: **hold-and-escalate to human detection engineers / incident commanders rather than auto-rollback.** The corpus suggests **automated rollback might ONLY be considered when a rule produces extremely high alert volume with VERY LOW incident yield over a PROLONGED window** (persistent noise, not a transient spike) — and even then, a rule whose alerts corroborate other activity "should be recognized as a corroborating source rather than a primary noise generator." [Spike-Synthesis]

### 6.5 LEAN (discussion input only)

**Make corroboration the master gate on the rollback decision, not volume alone.** Before the Q3 breaker is allowed to *open*, check the Q6 discriminators: (1) **is the spike corroborated** by other independent rules firing on the same entities/window, or by threat-intel matches? (2) **is it concentrated** on few (high-value) entities, or **uniformly dispersed** (high cardinality, no clustering)? **Corroborated + concentrated = likely real attack → DO NOT auto-rollback; ESCALATE to a human** (the rule may be doing its job at the worst moment). **Uncorroborated + uniformly dispersed + sustained over a prolonged window with low incident yield = likely broken → demote-to-shadow is safe.** This makes the breaker trip on **"persistent noise with no incident corroboration,"** never on **"transient high-signal spike,"** honoring the error asymmetry. Encode anti-flap as: **cool-down + exponential backoff on repeated trips + human confirmation required before re-promotion after any rollback** (no auto re-promote loop). **Ambiguous spikes (high volume, partial corroboration) → hold-and-escalate, never auto-act** — this is the production-grade safe default and reuses the Q1 Marginal/Inconclusive→pause-for-human prior art.

---

## Consolidated Open Design Questions

For the architect at morph (NOT decided here):

1. **Trip signal weighting / N-of-M composition** — exactly which Q2 signals (CUSUM-on-`λ_t`, cardinality-on-`U_t`, `N_t/U_t` duplicate-ratio) compose into the breaker trip, and the N-of-M threshold. LEAN: require volume-spike AND a cardinality/duplicate anomaly.
2. **Relative-multiplier value + absolute backstop** — the `Nx`-over-shadow-baseline multiplier and the absolute alerts/hr safety cap. Where do they live — §14.1 `quality` block vs per-deployment config? (Carried-open from C6 Q4 open #9.)
3. **CUSUM/ADWIN parameterization** — `v`/`h` (CUSUM) or window (ADWIN) derived from the shadow baseline to a target `ARL₀`; the acceptable detection-latency (`ARL₁`) for a security context.
4. **Minimum-window count before the breaker may open** (the ≥50-sample analog) — how many evaluation windows of shadow baseline are required before active monitoring is statistically trustworthy.
5. **Corroboration data model** — how "spike corroborated by independent rules / threat-intel / entity-concentration" is computed in real time and fed into the trip gate (this is the novel, undocumented-in-prior-art piece — `[INCONCLUSIVE]` in the corpus).
6. **Cool-down + backoff schedule + half-open trial size** — `waitDurationInOpenState`, exponential-backoff on repeated trips, and how many trial-routed alerts (and how many clean windows, the `consecutiveSuccessLimit` analog) are required to re-close.
7. **Per-tenant vs global breaker state management** — confirm per-tenant default (Prism multi-tenant) and the escalation rule when the same rule trips across many tenants (a "rule broken, not tenant-attacked" global signal).
8. **canary → production human-gate UX** — where the human sign-off lives (S2-console / MCP / CLI per C6 settled "rule editor" decision) and what evidence it surfaces.
9. **Shared change-detector primitive boundary** — confirm Q2 detector and C7 drift detector share one implementation pointed at different streams.

---

## Recommended auto-rollback control loop for Prism detections (concrete)

Discussion input only. A buildable shape consistent with C6 Q4 and all settled decisions:

**Signals (Q2):**
- Primary: **CUSUM on per-rule per-tenant alert-rate `λ_t`**, `v`/`h` calibrated from the shadow-mode baseline to a target `ARL₀` (tolerate diurnal variation; consider ADWIN if baseline is strongly non-stationary).
- Secondary: **cardinality monitor on distinct-entity `U_t` + the `N_t/U_t` duplicate-ratio** (catches cardinality-explosion and duplicate-storm).
- Delayed validation: **analyst dispositions** confirm/tune trips after the fact (NOT in the real-time path; DDM/EDDM only applicable once labels exist).
- Shared with C7 drift detection (one primitive, different target stream).

**Trip gate (Q3 + Q6):** breaker opens only when **N-of-M signals fire** (volume-spike AND cardinality/duplicate anomaly), **trip is RELATIVE** (alert-rate > `Nx` shadow baseline, with an absolute backstop cap), **AND the spike fails the Q6 corroboration test** (uncorroborated by independent rules/threat-intel AND uniformly dispersed rather than concentrated on high-value entities) **AND it is sustained over a prolonged window** (not a transient spike) **AND** a minimum-window count of baseline exists. Corroborated + concentrated spikes are **excluded from auto-trip** and **escalated to a human**.

**Action (Q4):** on trip, **DEMOTE-TO-SHADOW** (per-tenant): rule **keeps evaluating** (no coverage blind spot, full audit trail) but **stops routing** to analysts. Idempotent (state = `shadow`). **Already-emitted findings are annotated, not retracted.** **FULL-DISABLE is NOT auto-performed** — it requires explicit human sign-off (SOAR coverage-reducing-action gate). **REVERT-TO-PRIOR-VERSION is offered as a one-click human action**, not the auto reflex.

**Hysteresis / anti-flap (Q1 + Q6):** **cool-down** (`waitDurationInOpenState`) before a **half-open trial re-route**; require **`consecutiveSuccessLimit` clean windows** to re-close; **exponential backoff** on repeated trips; **human confirmation required before re-promotion** after any rollback (no auto re-promote loop).

**Human boundary (Q4 + Q5 + Q6):** safe to fully automate = **demote-routing-to-shadow** (reversible, coverage-preserving). Requires human sign-off = **full-disable of evaluation** and **canary → production promotion**. Ambiguous high-signal spikes = **hold-and-escalate, never auto-act**.

**Promotion gates (Q5):** shadow → canary auto-gates on metrics over a bake window; canary → production is human-gated; **tenant-by-tenant canary unit**; thresholds carried in the §14.1 `quality` block.

---

## Honest Costs & Caveats

1. **No SIEM ships this — Prism assembles it (confirmed, consistent with C6 Q4).** The prior art is *progressive delivery* (Kayenta/Flagger/Argo) + *circuit-breaker* (resilience4j/Hystrix) + *change-detection* (CUSUM/ADWIN/BOCPD) + *SOC triage* — each mature in its own domain, **none integrated for detection-rule auto-rollback.** Prism owns the integration and its correctness. [Detection-CB-Synthesis][Spike-Synthesis]
2. **The corroboration gate (Q6) is the novel, hardest, least-documented piece.** Sources apply corroboration/entity-concentration to alert *triage*, but **explicitly do NOT extend it to automated rollback** — "that step remains conceptual." Prism builds the spike→auto-rollback discriminator with no vendor template. This is where a correctness bug = either a blinded rule during an attack (catastrophic) or a noise storm that never trips. [Spike-Synthesis][INCONCLUSIVE]
3. **The error asymmetry is the governing safety constraint, not a tuning detail.** Auto-disabling a working rule during a real campaign is catastrophic; tolerating noise is merely costly. This is *why* the LEAN is demote-to-shadow (never auto-full-disable) and hold-and-escalate on ambiguity. Any design that auto-disables evaluation, or trips on volume alone without corroboration, is unsafe under this asymmetry. [Spike-Synthesis]
4. **Zero-label real-time constraint is real.** FP-rate is unknowable in real time; the trip rests on unlabeled volume/cardinality with labels as delayed validation. DDM/EDDM (the obvious drift detectors) **need labels and cannot drive the real-time trip.** [Change-Survey][Drift-Survey]
5. **`[INCONCLUSIVE]` items:** Flagger's failure-counter consecutive-vs-cumulative semantics and reset behavior are undocumented; in-flight-request handling during rollback is inferred not stated; the standardized detection-promotion metric thresholds are organizational not vendor-standard; the spike→auto-rollback discriminator is conceptual in the corpus. None of these block the LEAN; all are flagged for the architect.
6. **"Never roll back a working rule" is not an absolute guarantee — it is a safety POSTURE** (corroboration gate + concentration test + sustained-window requirement + hold-and-escalate + demote-not-disable + human re-promote gate). Like C6 Q6's "never silently mask a TP," it is transparency + asymmetry-aware caution, not a proof. State this plainly in the spec. [Spike-Synthesis]
7. **Extraction caveat.** All six deep-research responses exceeded the inline token cap and were analyzed by targeted extraction of the parameter-, finding-, and caveat-bearing passages (every concrete parameter, the three rollback semantics, the error-asymmetry and corroboration findings, and the human-boundary guidance were read in full). The narrative connective tissue between extracted passages was not read line-by-line; no parameter or load-bearing finding cited above is unverified, but the full numbered-citation maps live in the saved transcripts (Research Methods).

---

## Sources (primary source families surfaced by the deep-research passes)

> Source families the Perplexity `sonar-deep-research` responses grounded their numbered citations in. Exact URLs/numbered maps are in the saved transcripts (Research Methods).

- **[Kayenta][Kayenta-Judge][Kayenta-Timing][Kayenta-GCP][Netflix-Blog][Spinnaker-BP]** Netflix Kayenta + Spinnaker Automated Canary Analysis: NetflixACAJudge (Mann–Whitney U, Hodges–Lehmann, 98% CI, ±25% tolerance band, effect-size gates, group scoring, pass/marginal thresholds 95/75 starter, critical-metric + 50%-NODATA auto-fail), analysis lifetime/interval/delay/lookback/step, ≥50-sample rule (Spinnaker best-practices, Kayenta docs, Netflix Tech Blog, Google Cloud Kayenta blog).
- **[Flagger][Flagger-Spec][Flagger-AppMesh]** Flagger Canary CR: interval (60s default), threshold (max failed checks), maxWeight/stepWeight, progressDeadlineSeconds (600s default), readiness thresholds, MetricTemplate/thresholdRange, revert-to-primary + scale-to-zero rollback (flagger.app docs + AWS App Mesh progressive-delivery blog).
- **[Argo][Argo-Analysis]** Argo Rollouts AnalysisTemplate: interval/count, failureLimit (default 0), consecutiveSuccessLimit (default 0), verdict matrix (Inconclusive → pause-for-human), setWeight/pause steps, manual promote gate, Degraded→weight-0 rollback, declarative/idempotent model (argo-rollouts docs).
- **[Change-Survey][SPC]** Online change-point / rate-spike detection: CUSUM (drift parameter v, threshold h, ARL₀/ARL₁), Page–Hinkley, EWMA (λ smoothing, control constant L), ADWIN (adaptive window), BOCPD/Adams–MacKay (hazard rate, run-length); statistical process control literature; alert-volume/cardinality/duplicate-storm formalization; shadow-mode-as-reference-window.
- **[Drift-Survey]** ML concept-drift / data-drift detection surveys (e.g. Gama et al. lineage): DDM/EDDM require labels (cannot drive zero-label real-time trip); shared CUSUM/ADWIN/Page–Hinkley/BOCPD family with change-point detection (the C7 tie); river/scikit-multiflow streaming-ML detector implementations.
- **[Fowler][Azure-CB][Resilience4j][Hystrix][Detection-CB-Synthesis]** Circuit-breaker pattern: Nygard/Fowler closed/open/half-open; resilience4j (failureRateThreshold, slidingWindowType/Size, minimumNumberOfCalls, waitDurationInOpenState, permittedNumberOfCallsInHalfOpenState, slowCallRateThreshold); Hystrix (errorThresholdPercentage, requestVolumeThreshold, sleepWindow); Azure architecture circuit-breaker docs; synthesis mapping closed=route/open=demote/half-open=trial onto detection-rule routing, per-tenant vs global, hysteresis/cool-down.
- **[Rollback-Synthesis][SOAR]** Rollback-action semantics: revert-to-last-known-good as dominant pattern (Spinnaker/Argo/Flagger/LaunchDarkly/Unleash); demote-to-shadow vs full-disable (coverage blind spot) vs revert-to-prior-version tradeoff; idempotency from declarative model; future-not-already-emitted; SOAR human-approval-gate guidance for coverage-reducing actions (account-deactivation/host-isolation analogy).
- **[Spike-Synthesis][SRE-Alert][SRE-Flap]** Legitimate-spike-vs-noise discrimination: error asymmetry (false-rollback-during-attack ≫ tolerating noise); corroboration via independent rules + threat-intel; entity-graph concentration vs uniform dispersion; asset criticality; hold-and-escalate on ambiguity; Google SRE book alert-fatigue/flapping; anti-flap via hysteresis/extended-evaluation/cool-down/backoff; humans (not automation) make disable/threshold decisions; Rapid7/SIEM-correlation triage context.
- **[Promo-Synthesis][Sigma-Status]** Promotion gates: Argo manual-promote pause-gate vs metric-gated auto-promotion; bake/soak time; deployment rings / cohort / tenant-by-tenant; Sigma status experimental→test→stable; "soak mode" for detections; detection-as-code versioned-artifact lifecycle (SigmaHQ spec + detection-engineering maturity writing).

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 6 | All 6 at `reasoning_effort=high`, `strip_thinking=true`: (Q1) ACA control-loop parameters — Kayenta Mann–Whitney/judge/thresholds, Flagger/Argo CR params, hysteresis; (Q2) zero-label bad-rollout detection — CUSUM/Page-Hinkley/ADWIN/EWMA/BOCPD parameters, alert-storm, cardinality, disposition lag, drift-family tie; (Q3) circuit-breaker design — resilience4j/Hystrix params, closed/open/half-open mapping to routing, per-tenant, multi-signal; (Q4) rollback action semantics — revert-to-last-known-good, demote-shadow vs full-disable vs revert-version, in-flight/idempotency, human boundary/SOAR; (Q5) promotion gates — auto vs human, bake time, rings/tenant, Sigma status; (Q6) legitimate-spike-vs-noise — error asymmetry, corroboration, entity-concentration, anti-flap, hold-and-escalate. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 0 | — |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (no library-API question this pass; control-loop is conceptual/cross-vendor) |
| Tavily tavily_search | 0 | — |
| Tavily tavily_research | 0 | — |
| WebFetch | 0 | — |
| WebSearch | 0 | — |
| Training data | ~4 areas (flagged inline) | Kayenta 98%-CI-is-fixed inference; absolute-vs-relative-cap + N-of-M explicit framings (standard rate-limiter/multi-signal formalization applied to the cited trip inputs); in-flight/idempotency inferred from declarative model where docs were silent. All tagged `[model-knowledge]` / `[INCONCLUSIVE]` inline. |

**Total MCP tool calls:** 6 (6 `perplexity_research` @ high).
**Training data reliance:** **low** — every substantive parameter (CUSUM v/h/ARL, EWMA λ/L, resilience4j/Hystrix knobs, Kayenta thresholds, Flagger/Argo CR fields) and every load-bearing finding (revert-to-last-known-good dominance, demote-shadow coverage preservation, SOAR human-gate, error asymmetry, corroboration discriminators) is grounded in a cited web source family. Model knowledge is confined to clearly-tagged framing/inference where vendor docs were silent.

**Overload-resilience note:** This run hit no API overload — all 6 high-effort deep-research calls succeeded on first attempt. The medium-effort fallback was not exercised.

**Saved deep-research transcripts (full numbered-citation prose):**
- Q1 (ACA): `tool-results/mcp-perplexity-perplexity_research-1782574847575.txt`
- Q2 (zero-label change detection): `tool-results/mcp-perplexity-perplexity_research-1782574900341.txt`
- Q3 (circuit-breaker): `tool-results/mcp-perplexity-perplexity_research-1782575221830.txt`
- Q4 (rollback semantics): `tool-results/mcp-perplexity-perplexity_research-1782575241473.txt`
- Q5 (promotion gates): `tool-results/mcp-perplexity-perplexity_research-1782575475784.txt`
- Q6 (spike-vs-noise): `tool-results/mcp-perplexity-perplexity_research-1782575511262.txt`
(under `/Users/jmagady/.claude/projects/-Users-jmagady-Dev-prism/1cbcd55e-1092-4bcc-ab2e-65460a5c2bee/`)
