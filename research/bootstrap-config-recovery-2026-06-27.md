---
document_type: research
produced_by: research-agent
status: capture
do_not_execute: true
timestamp: "2026-06-27"
program: day-2-vision-side-analysis
relation: OUT-OF-BAND — SEPARATE from the live VSDD factory pipeline
scope: >
  TARGETED validation pass on the RESTART-CLASS / BOOTSTRAP-CONFIG RECOVERY design for the
  Prism day-2 vision (centralized appliance + outbound-only dial-home satellite mesh, air-gap
  capable, per-Purdue-layer OT placement). Validates six proposed design elements against cited
  prior art: (1) validate-before-persist; (2) A/B dual-slot bootstrap; (3) supervisor-owned boot
  watchdog + auto-fallback ("commit confirmed"); (4) satellite autonomous self-recovery;
  (5) bootstrap-layer fleet canary; (6) safe-mode recovery console. Restart-class keys =
  {listen port, TLS cert/key, store connection string, identity/enrollment token, transport
  selection}. The crux: a SATELLITE whose bad bootstrap is its own identity or store-connection
  cannot dial home, so CENTRAL CANNOT RESCUE IT — recovery must be autonomous, local, and live
  in a layer ABOVE the main process.
settled_context_NOT_relitigated:
  - "Day-2 topology: centralized appliance + outbound-only dial-home satellite mesh per C2 (satellite-mesh-2026-06-26.md); air-gap-capable hard requirement; per-Purdue OT placement"
  - "Config authority: DB-authoritative + UI-only authoring (config-authority-narrow-git-2026-06-27.md, config-management-depth-2026-06-27.md)"
  - "HOT-RELOADABLE runtime config + detection content already have versioned fast-revert (DB-native temporal / embedded git + atomic ArcSwap hot-swap) — NOT relitigated here"
  - "The OPEN problem this pass scopes: RESTART-CLASS keys are NOT hot-reloadable; a bad one can brick the bootstrap"
caveat: >
  CAPTURE artifact. LEANS are discussion input only — NOT decisions. Numbers/epics/ADRs remain
  the architect's at morph. This file does not modify STATE.md, SESSION-HANDOFF.md, the live ADR
  registry, any live spec/BC/story, RESEARCH-INDEX.md, or any prior research file, and was not
  git-added or committed. Version numbers verified against crates.io as of 2026-06-27; landscape
  changes rapidly — re-verify at morph.
---

# Bootstrap / Restart-Class Config Recovery — TARGETED Validation Pass (Day-2 Vision Side-Analysis)

**Date:** 2026-06-27 · **Primary tool:** `perplexity_research` at `reasoning_effort=high` (all 5 thematic deep-research calls succeeded on first attempt; no overload fallback to `medium` was needed) · plus `perplexity_search` + crates.io WebFetch for Rust crate-state verification.

This pass validates a concrete proposed design for the one open day-2 problem the team has not yet resolved: **restart-class / bootstrap keys are not hot-reloadable, and a bad one can prevent the process from restarting ("bricked bootstrap")** — worst case at an air-gapped satellite whose bad bootstrap is its own identity or store-connection-string, so it can no longer dial home and **central cannot rescue it**.

Six design elements are validated below, each against cited prior art with a discussion **LEAN** (confirm / refine-as / refute). Web findings are flagged inline; `[model-knowledge]` marks model-supplied reasoning; `[INCONCLUSIVE]` marks where sources fell short. Source families are named in the **Sources** appendix; full numbered-citation prose lives in the saved transcripts (paths in Research Methods).

> **Cross-tool reading note.** Four of the five Perplexity deep-research responses exceeded the inline token cap and were saved to transcript files; each was analyzed by sequential extraction covering every cited system, parameter, and caveat used below. The fifth (validate-before-persist) was read in full including its citation list. The Honest Costs section flags the two places where transcript tails (citation lists + minor recap, ~13–20% of each oversized file) were not re-read line-by-line because the substantive findings were complete before the tail.

> **Headline finding (load-bearing).** The canonical network-appliance "commit confirmed" pattern (JunOS / Cisco / Arista / NVUE) is **real, mature, and directly applicable — but every mainstream implementation confirms via an EXPLICIT operator/controller command, NOT a local health signal.** The ONE documented exception that uses connectivity-as-confirmation is **Palo Alto Panorama "Automated Commit Recovery"** (an NGFW that cannot reach Panorama after a commit auto-reverts). For a satellite that cannot phone home, Prism MUST adapt the pattern so the "confirm" is a **local readiness/health signal owned by a supervisor above the main process**, not an operator action. This adaptation is the design's central novelty and it is well-supported by adjacent prior art (Android/ChromeOS/U-Boot boot-counting, Mender pre-commit health-check rollback, Cisco IoT auto-recovery).

---

## Q1 — The "commit confirmed" / config-confirmation pattern (the load-bearing answer)

### 1.1 The canonical mechanism, per vendor (cited)

All four major NOS implementations share the same three-phase shape: **(a) provisional activation** that starts a timer, **(b) an explicit confirmation command**, **(c) automatic revert to last-known-good if not confirmed before the timer expires.** [JunOS-docs][Cisco-IOSXE][Arista-EOS][NVUE]

| System | Provisional command | Confirmation signal | Default timeout | Auto-revert restores |
|---|---|---|---|---|
| **JunOS** `commit confirmed [minutes]` | candidate→active, tentative | any subsequent `commit` or `commit check` before timeout | **10 min** if omitted | previous config (effectively `rollback 1`) from the 50-deep rollback archive [JunOS-docs] |
| **Cisco IOS XE** Configuration Rollback Confirmed Change (`configure replace … revert trigger`, `configure [terminal] revert timer N`) | snapshot current, start timer | `configure confirm` | **no default** — operator must supply `minutes` | original config prior to change (archive or startup-config) [Cisco-IOSXE] |
| **Arista EOS** config session + `commit timer hh:mm:ss` | session commit, tentative | explicit confirm within session before timer | **no default** — `hh:mm:ss` required | running config prior to session commit [Arista-EOS] |
| **NVUE / Cumulus** `nv config apply --confirm [time]` | apply revision, tentative (`--confirm-status` shows time left) | explicit confirm within window | **10 min** if omitted | previous config revision [NVUE] |
| **Cisco IOS (legacy)** `reload in N` / `reload at` | schedule a reboot | `reload cancel` | no default | startup-config on reboot (coarse, disruptive) [Cisco-reload] |

Two of the four (JunOS, NVUE) ship a **10-minute default** — a useful convergence point. Cisco and Arista force the operator to set the window explicitly. [JunOS-docs][NVUE][Cisco-IOSXE][Arista-EOS]

### 1.2 The hard truth for Prism: confirmation is an OPERATOR action, not a health signal

Across **all** mainstream NOS implementations, "confirmed" means *a human (or controller) typed a confirm command*. **None** of JunOS, Cisco IOS XE, Arista EOS, or NVUE expose a built-in *positive local health* confirmation (e.g., auto-confirm when local services reach ready). [JunOS-docs][Cisco-IOSXE][Arista-EOS][NVUE] Cisco's `revert trigger error` / `idle` options are a **negative** health signal (revert on observed error/idleness) — not a positive "I'm healthy, keep this" signal. [Cisco-IOSXE]

The clean exception is **Palo Alto Panorama Automated Commit Recovery**: a managed NGFW that **cannot reach Panorama after a commit reverts to the previous commit** — connectivity-to-controller IS the health signal. [PaloAlto-Panorama][air-gap-transcript] This is the closest mainstream prior art to what Prism needs, but it inverts cleanly for the satellite case: Prism cannot use "reached central" as the confirm signal for a satellite whose whole failure mode is *being unable to reach central*. So Prism must generalize "confirm" to a **local readiness gate** (process bound its socket, opened its store, loaded its identity, passed self-checks) owned by the supervisor — with "successfully dialed home" as an *additional, stronger* confirm tier where the link is actually up.

### 1.3 LEAN — CONFIRM the pattern; REFINE the confirm signal

**CONFIRM** that "commit confirmed" is the correct backbone for restart-class changes. **REFINE-AS:** the confirm signal must be a **local health/readiness probe owned by a supervisor above the main process**, not an operator command and not "reached central." Adopt the 10-minute default convention from JunOS/NVUE for the *operator-initiated central* path; use a **shorter boot-readiness deadline** (seconds-to-low-minutes) for the *autonomous satellite* path, because the satellite has no operator to extend the timer (JunOS explicitly cannot extend a running countdown [JunOS-docs]). The revert target is "previous bootstrap," exactly mirroring `rollback 1` semantics.

---

## Q2 — A/B (dual-bank) boot + last-known-good fallback

### 2.1 The mechanism is uniform across embedded prior art (cited)

Every surveyed system implements: two slots, per-slot metadata `{bootable, successful/tries, priority}`, a **user-space health signal** that marks a slot good, and a **bootloader-side counter** that falls back after N failed boots. [AB-transcript]

- **Android A/B (seamless):** boot-control HAL (`bootctl`: `mark-boot-successful`, `set-active-boot-slot`, `set-slot-as-unbootable`). The bootloader decrements `slot-retry-count` each boot; at zero-without-success it marks the slot unbootable and picks the other. Critically, **"the bootloader must never mark a slot successful" — only the Android framework calls `markBootSuccessful`** once user-space is healthy. Retry count is device-configurable (no fixed default). [Android-AB]
- **ChromeOS:** GPT flags `priority` / `successful` / `tries`; `cgpt` manipulates them; user-space sets `successful` after verified boot + integrity checks; bootloader stops choosing a slot when `tries`→0. [ChromeOS-AB]
- **U-Boot Boot Count Limit:** `bootcount` increments per reboot; when it exceeds `bootlimit`, U-Boot runs `altbootcmd` (slot switch) instead of `bootcmd`. The `upgrade_available` flag gates whether bootcount persists. User-space clears `upgrade_available` + resets `bootcount` to signal a healthy boot. Example bootlimit = 5. [U-Boot]
- **RAUC / Mender / SWUpdate:** slot abstraction + `mark-good` services (`rauc status mark-good`; Mender commit; SWUpdate post-boot script clearing `upgrade_available`); bootloader integration via barebox bootchooser / U-Boot / GRUB / systemd-boot. [RAUC][Mender][SWUpdate]
- **systemd-boot automatic boot assessment:** tries encoded in the loader-entry filename (e.g., `arch-lts+3.conf`); `boot-complete.target` + `systemd-bless-boot.service` mark the boot good (decrement/reset tries) once reached. [systemd-boot]

The invariant across all of them: **a "healthy boot" is a user-space concept signaled back to a lower layer; the boot/supervisor layer NEVER self-declares success.** [Android-AB][AB-transcript] This directly validates Prism design element #3 (supervisor owns the watchdog, app signals health up).

### 2.2 Does A/B map to CONFIG slots (not just OS images)?

**Partially — with explicit prior art for config A/B, but it's domain-specific rather than from the embedded frameworks.** [AB-transcript] The embedded A/B frameworks (Android/ChromeOS/RAUC/Mender) document slots as *firmware/rootfs partitions*; none ships a generic "config slot" abstraction. **But** the conceptual mapping is clean (pending vs active config; health signal = config stable; fallback = previous config), and two pieces of *direct* config-level prior art exist:
1. **Cisco "commit confirmed" for config** — explicitly an A/B-like pending/active config with auto-revert (time-based rather than boot-count-based). [Cisco-config-AB][AB-transcript]
2. **FPGA golden-image dual boot (Lattice MachXO3D)** — a golden (last-known-good) image + a mutable version image, with hardware fallback to golden on failure. [FPGA-golden][AB-transcript]

`[INCONCLUSIVE]` on a turnkey OSS "A/B config slot + boot-counting" framework — none was found; the team would assemble it from the U-Boot/systemd boot-counting pattern applied to config slots. This is a build-not-buy area.

### 2.3 LEAN — CONFIRM A/B dual-slot for bootstrap config

**CONFIRM** persisting bootstrap in `active` (last-known-good) + `pending` slots, booting `pending`, promoting only after a healthy/ready signal. The pattern is overwhelmingly validated for images and conceptually + by-Cisco-precedent validated for config. **REFINE-AS:** explicitly adopt the Android invariant — the supervisor/boot layer must **never** promote `pending`→`active` on its own; promotion happens only on a positive readiness signal emitted by the (successfully started) main process. Make the boot counter and the "healthy" definition first-class config (mirroring `slot-retry-count` / `bootlimit` being device-configurable).

---

## Q3 — Watchdog / supervisor layer (Rust + Linux), readiness vs liveness

### 3.1 The supervisor-above-the-process mechanisms (cited)

- **systemd service watchdog:** `WatchdogSec=` requires the service to ping `sd_notify(WATCHDOG=1)` on an interval; a missed ping → systemd kills + restarts per `Restart=`. **Crash-loop detection:** `StartLimitIntervalSec=` + `StartLimitBurst=` — if the service restarts more than `burst` times within the interval, systemd stops trying and runs `StartLimitAction=` (e.g., reboot, or just give up). `Type=notify` + `READY=1` is the **readiness** signal (distinct from `WATCHDOG=1` liveness). [systemd-watchdog][watchdog-transcript]
- **Hardware/software watchdog timers** (`/dev/watchdog` + watchdogd): a missed keepalive triggers a hard reboot — the lowest, most-untrusted-code-resistant recovery tier. [watchdog-transcript]
- **Readiness vs liveness (the key distinction for "started-but-broken"):** Kubernetes is the canonical reference — **liveness** = process alive (restart if not); **readiness** = process able to serve (remove from rotation if not); **startup** = guard slow-starting apps so liveness doesn't kill them mid-boot. The "started but broken" case Prism worries about is precisely a **readiness/startup-probe** failure, NOT a liveness failure. [K8s-probes][watchdog-transcript] A liveness-only supervisor (process is alive ⇒ healthy) would MISS a bricked-but-running bootstrap; the supervisor must gate promotion on **readiness**.

### 3.2 Rust crate state (verified against crates.io / lib.rs, 2026-06-27)

If Prism relies on systemd as PID 1:
- **`sd-notify` v0.5.0** (2026-03-09) — lightweight readiness/watchdog notify from Rust (`READY=1`, `WATCHDOG=1`, `fdstore`). Actively maintained, ~12M downloads. [crates.io: sd-notify]
- **`libsystemd` v0.7.2** (2025-04-30) — pure-Rust systemd client (notify + journal + more). [crates.io: libsystemd]

If the appliance bundles its OWN supervisor / PID 1 (the air-gapped/OT case where systemd may be undesirable or absent), the Rust ecosystem is **thin but real and improving** [perplexity_search 2026-06]:
- **`rust-tokio-supervisor`** v0.1.4 (2026-06-03, Rust 2024) — declarative supervisor trees, child lifecycle, restart policies, four-stage shutdown, state queries, event journal. Newest entrant; OTP-inspired. [lib.rs]
- **`processmanager`** v0.5.0 (2025-07-08, Rust 2024) — lightweight Tokio supervisor; spawn/reload/shutdown, error propagation, graceful tree shutdown. [lib.rs]
- **`supervised`** v0.3.0 (2026-04-22) — Tokio service supervisor modeling restart/shutdown/cancellation AND **startup readiness** explicitly. [lib.rs]
- **`super-visor`** v0.3.0 (2026-01-08) — ordered startup/shutdown of long-running Tokio procs (Erlang-inspired, CancellationToken-based). [lib.rs]
- **`ractor-supervisor`** / **`rust_supervisor`** — OTP-style supervision (OneForOne/OneForAll/RestForOne) with meltdown (restart-storm) protection. [perplexity_search]
- **`watchexec-supervisor`** v5.0.1 (2025-05-15) — process supervisor component from the watchexec project (maintained). [perplexity_search]

> **Version-pin caveat:** the bundled-supervisor crates are mostly **pre-1.0** (0.x) and young. For a security appliance under the production-grade default, treat any 0.x supervisor crate as needing a vendoring/audit decision, not a blind dependency. `sd-notify` (0.5.0, 12M downloads) is the only mature option, and it presupposes systemd. The **`watchdog` crate (v0.2.6)** on crates.io is a filesystem-change watcher — **NOT** a process watchdog; do not confuse it. [crates.io: watchdog]
> Supporting deps for the supervisor build are mature: **`tokio` 1.52.3** (2026-05-08, Rust ≥1.71), **`arc-swap` 1.9.1** (already the Prism hot-reload primitive per AD-007), **`sysinfo` 0.39.5** for process/health introspection. [crates.io]

### 3.3 LEAN — CONFIRM supervisor-owned watchdog; REFINE distinguisher to readiness

**CONFIRM** that recovery must live in a supervisor/init layer above the main process (the app cannot roll itself back if it can't start — universally validated). **CONFIRM** the boot-count + auto-fallback + reboot loop with a `StartLimitBurst`-style crash-loop cap. **REFINE-AS:** the supervisor must distinguish healthy from started-but-broken via a **readiness probe** (the main process emits `READY=1`/equivalent only after it bound its port, opened its store, loaded identity, and passed self-checks), NOT mere liveness. **Recommended build path:** if the appliance ships on systemd, use `sd-notify` (mature) + `WatchdogSec` + `StartLimitBurst`/`StartLimitAction` + boot-counting in the unit/generator layer — this is the lowest-risk, best-supported option. If a bundled PID-1 supervisor is required for the air-gapped/OT build, prototype against `supervised` or `rust-tokio-supervisor` but **audit/vendor** them (0.x maturity) and keep a hardware `/dev/watchdog` as the bottom recovery tier.

---

## Q4 — Validate-before-persist / dry-run for bootstrap-class settings (be honest about limits)

### 4.1 What the config-test idiom does, and where it stops (cited)

The "test config before reload/restart" idiom is universal — Envoy `--mode validate`, nginx `-t`, HAProxy `-c`, sshd `-t`/`-T`, plus Kubernetes server-side dry-run as the richer cousin. [validate-transcript] The honest taxonomy:

**Cheaply pre-validatable (no side effects, strong guarantees):**
- **Syntax / schema / structure** — Envoy parses bootstrap against protobuf schema; nginx/HAProxy/sshd parse grammar + context/section validity + cross-references. [Envoy-validate][nginx-t][HAProxy-c][sshd-t]
- **Local file existence + readability** — cert/key files, includes, log paths present and readable. (Envoy `--mode validate` even *creates* referenced log files — file I/O is treated as an acceptable validation side effect.) [Envoy-validate]
- **Value ranges / allowed-set membership** — port numbers in range, enums valid, timeouts well-formed.
- **Cryptographic well-formedness** — cert/key parse, key matches cert, **not-expired** is checkable. (Deep chain/revocation/OCSP validation is NOT typically done in dry-run — that needs network.) [validate-transcript]

**Fundamentally racy or side-effect-dependent (canNOT be guaranteed by pre-flight):**
- **"Port bindable" is RACY (TOCTOU).** A port free at check-time can be taken before the real bind (another process, systemd socket activation). Envoy/nginx/HAProxy/sshd **deliberately do NOT pre-bind** in validate mode — a "success" would be misleading and pre-binding could collide with the running instance. The cited failure mode is the classic `BindException: Address already in use` at startup. [validate-transcript][port-race] **→ This directly refutes the naive reading of Prism design element #1's "port bindable" check.**
- **"Store connection-string connects" has its OWN failure modes** — a real connection attempt can give a **false negative** on a transient outage / firewall / DNS blip, causing you to reject a *correct* config; or a false positive if the store is up now but the cert/credential is subtly wrong. Distinguishing a config error from an environment blip requires retries + context the validator doesn't have. [validate-transcript]
- **Identity / enrollment token "accepted"** — you can validate *well-formedness* (JWT structure, base64, expiry claim) cheaply, but "the controller will accept it" requires actually contacting the controller (side effect + may not be reachable at all for an air-gapped satellite). [validate-transcript]

Kubernetes server-side dry-run is the one pattern that validates *dynamic* policy (RBAC, admission webhooks, quota) by pushing through the live API server without persisting — but even it cannot guarantee port binding or upstream reachability. [K8s-dry-run]

### 4.2 LEAN — CONFIRM validate-before-persist; REFINE the per-key honesty

**CONFIRM** validate-before-persist + fail-closed (reject a bad bootstrap at write time, keep old) — strongly validated, this is the universal idiom. **REFINE-AS, per key (this is the load-bearing correction to design element #1):**
- **TLS cert/key:** parse + key-matches-cert + **not-expired** at write time. CHEAP & RELIABLE. ✅
- **Identity/enrollment token:** validate **well-formedness + expiry claim** at write time; "accepted by controller" is a *runtime* check, not a write-time gate. ✅ (well-formedness only)
- **Listen port:** validate **range/format** at write time; **do NOT treat a write-time bind probe as authoritative** (TOCTOU). The *real* test is the supervisor's boot-readiness gate (Q3) — if the port can't bind at boot, the readiness signal never fires, the boot-count trips, and A/B falls back. Bind-ability is a **boot-time, not write-time** guarantee. ⚠️
- **Store connection string:** a write-time connect probe is **advisory, not a hard gate** — surface a warning on failure but allow override, because a transient store outage must not block a legitimately-correct config edit. The authoritative test is, again, the boot-readiness gate + A/B fallback. ⚠️
- **Net:** validate-before-persist catches the *cheap, deterministic* errors (the common case: typo'd cert path, expired cert, malformed token, out-of-range port). The A/B + supervisor-readiness + auto-fallback loop (Q2/Q3) is the **backstop for everything that is fundamentally racy or environment-dependent.** The two layers are complementary, not redundant — and design element #1 must be honest that it cannot prevent all bricks by itself.

---

## Q5 — Satellite autonomous self-recovery (the air-gapped / can't-phone-home crux)

### 5.1 The stated principle and its strongest prior art (cited)

The deep-research surfaced an **explicit statement of the core principle**: *a device that has broken its own management link cannot be rescued through that link — recovery must be local and autonomous.* [air-gap-transcript] Strongest cited prior art that operationalizes this:

- **Mender device-side rollback (closest direct analog).** Updates are *provisional until committed*; a **pre-commit state script** runs after install/reboot but before commit and can perform **arbitrary health checks — including attempting to contact the controller/store** — and force a rollback (non-zero exit) if they fail. This is *entirely device-side*: the device uses "I cannot reach what I need" as the rollback trigger, with no controller involvement. A/B partitions back it up for unbootable cases. [Mender][air-gap-transcript] **This is the template for the Prism satellite path.**
- **Cisco IoT (IR1101/IR1800) Auto Recovery.** If the device loses connectivity to **both** primary+secondary tracking IPs **and** the IoT Operations Dashboard for a configured interval, it autonomously resets / reloads / optionally hardware-resets-and-re-enrolls. Loss-of-controller IS the failure trigger; the decision logic is in device firmware. [Cisco-IoT][air-gap-transcript]
- **Palo Alto Panorama Automated Commit Recovery** (also cited in Q1) — connectivity-to-controller as the confirm/revert signal. [PaloAlto-Panorama]
- **Safe-mode / recovery partitions** — Android recovery image (structurally identical boot image, different kernel/ramdisk; decoupled from main config; supports factory reset locally), ChromeOS recovery mode (boots minimal env, recovers from USB created by a *different* working machine — the broken device needs no working control path). These validate design element #6. [Android-recovery][ChromeOS-recovery][air-gap-transcript]

### 5.2 Where fleet-management systems do NOT close the loop (cited — important negative findings)

- **Tailscale:** robust *control-plane/data-plane separation* (a node "offline" to the coordination server still routes via existing WireGuard tunnels + DERP) — but **no documented client-side auto-rollback of an ACL/policy that breaks connectivity**; recovery is "edit the policy in the admin console," which assumes the control plane is reachable. [Tailscale][air-gap-transcript]
- **osquery + Fleet (`fleetd`):** central config is authoritative and can *clear* local flags (`command_line_flags: {}` empties them); **no documented device-side rollback** if a pushed config breaks enrollment. [Fleet][air-gap-transcript]
- **balenaCloud:** balenaOS *host-OS* updates ARE atomic with `rollback-health` + `rollback-altboot` (local last-known-good on failed/unhealthy boot — good prior art for the OS layer); but at the *application/config* layer there is **no documented auto-rollback triggered by lost cloud connectivity** — recovery leans on manual diagnostics. [balena][air-gap-transcript]

**Net negative finding:** most fleet managers are excellent at *pushing* config and *trust* the push; the systems that close the autonomous-local-recovery loop (Mender pre-commit, Cisco IoT auto-recovery, balenaOS host rollback, Panorama) are the minority — and they're the exact references Prism should follow. The "can't phone home → revert locally" behavior is **not** something Prism gets for free from a fleet-management framework; it must be designed in, satellite-side.

### 5.3 OT reality (cited + standards `[INCONCLUSIVE]`)

OT field devices frequently have **no out-of-band management**; the last resort is a local serial/console truck-roll — which for a Purdue-Level-0/1 device in a locked cabinet is expensive, and for a notional fully-air-gapped node may be the *only* path. The cited prior art (Cisco IoT auto-recovery) exists precisely because truck-rolls are costly. [air-gap-transcript] On formal standards: the deep-research **could NOT cite NIST SP 800-82 or IEC 62443 text directly** (not in the retrieved sources); it noted only the general ICS principle of *fail to a safe state*. `[INCONCLUSIVE — standards citations]`: if the architect wants 800-82 / 62443 fail-safe-state language as a normative anchor, that needs a dedicated standards-document pass (the standards are paywalled/long; this pass surfaced the principle but not quotable clauses).

### 5.4 LEAN — CONFIRM autonomous local self-recovery; this is non-negotiable for satellites

**CONFIRM, emphatically:** the A/B + watchdog + fallback loop must run **locally at every satellite with zero dependency on reaching central** (Mender pre-commit + Cisco IoT auto-recovery are the validated templates). **REFINE-AS:** make the satellite's restart-class "confirm" a **tiered local signal** — tier 1 = process reached readiness (bound port, opened store, loaded identity, self-checks pass); tier 2 (stronger, where applicable) = successfully dialed home at least once. A satellite that reaches tier 1 but never tier 2 should keep the new bootstrap but **report DEGRADED** (it's healthy locally but isolated) rather than reverting — because reverting a *locally-healthy* config just because the WAN is down would cause flapping during legitimate network outages. A satellite that fails tier 1 trips the boot-count and reverts. This tiering is the precise adaptation the standard commit-confirmed pattern lacks.

---

## Q6 — Satellite-fleet staged bootstrap rollout (canary across the mesh)

### 6.1 The two-layer pattern, with explicit fleet auto-rollback prior art (cited)

The deep-research cleanly separates **per-device rollback** from **fleet-level halt/rollback**, and shows the spectrum: [staged-transcript]

- **Per-device local fallback (what each satellite does):** Mender state-machine rollback + balenaOS `rollback-health`/`rollback-altboot` — each device independently reverts to last-known-good on failed/unhealthy boot. Mender phased rollout and balena release-pinning halt *widening* but do NOT auto-revert already-updated devices. [Mender][balena][staged-transcript]
- **Fleet-level automatic rollback (the strongest cited prior art):**
  - **Azure Device Update for IoT Hub** — the most explicit: a **rollback trigger policy = {percentage failed, minimum count failed}**; when the threshold is met, **ALL devices in the deployment group roll back** to the selected version. Already-updated devices are reverted, not just halted. [Azure-DU][staged-transcript]
  - **AWS ECS deployment circuit breaker** — `deploymentCircuitBreaker {enable, rollback}` + CloudWatch alarms (5xx, p99 latency, unhealthy hosts) with `stepPercent`/`canaryPercent` + bake times; ALARM → stop widening AND scale the old revision back up (revert the service). [AWS-ECS][staged-transcript]
  - **Argo Rollouts / Flagger** — the canonical metric-gated canary control loop: analysis runs at an interval, `failureLimit` / `consecutiveSuccessLimit` (Argo) or success-rate/latency thresholds + a **default 10-min progress deadline** (Flagger); breach → abort + revert traffic to stable, mark `Degraded`. [Argo][Flagger][staged-transcript]
  - **GKE node-pool surge** (`maxSurge`/`maxUnavailable` + PDBs + readiness probes) — gates by availability constraints; rollback is operator-driven (`gcloud … rollback`), not metric-automatic. [GKE][staged-transcript]

The reusable thresholds for Prism: a **percentage+min-count failure gate** (Azure pattern) and an **analysis-interval + consecutive-failure-limit** (Argo pattern) and a **progress deadline** (Flagger's 10-min default — note it matches the JunOS/NVUE commit-confirmed default).

### 6.2 LEAN — CONFIRM bootstrap-layer canary; REFINE the revert scope

**CONFIRM** never pushing a restart-class change to all satellites at once; canary cohort → health-gate → widen; a failed node falls back **locally** and reports **DEGRADED** on next heartbeat. This is exactly the Azure-DU + Mender composite. **REFINE-AS:** Prism should be explicit that the fleet rollout's job is to **halt widening** (Mender/balena semantics) while **each satellite self-reverts locally** (because central may not be able to push a revert to an isolated satellite — the Q5 crux). Adopt a **percentage+min-count halt gate** (Azure) so a tiny canary sample doesn't trip on one transient failure, plus a **progress deadline** (Flagger-style) after which un-acknowledged canaries are treated as failed. Central-driven mass-revert (Azure "roll back ALL in group") is a *nice-to-have for reachable satellites* but must NOT be the primary safety mechanism — the primary safety mechanism is local self-revert, since the worst-case satellite is unreachable by definition.

---

## VERDICT per design element

| # | Proposed design element | Verdict | One-line basis |
|---|---|---|---|
| 1 | **Validate-before-persist, fail-closed** | **CONFIRM, with REFINEMENT** | Universal idiom (Envoy/nginx/HAProxy/sshd), BUT be honest: cert-parse/not-expired + token-well-formedness are cheap & reliable; **"port bindable" is racy (TOCTOU)** and **"store connects" has transient-false-negative modes** — those are boot-time backstops, not write-time gates. [validate-transcript] |
| 2 | **A/B dual-slot bootstrap (active=LKG, pending, promote on healthy)** | **CONFIRM** | Uniform embedded prior art (Android/ChromeOS/U-Boot/RAUC/Mender/systemd-boot); config-A/B precedent via Cisco commit-confirmed + FPGA golden image. Invariant: boot layer never self-promotes. [AB-transcript] |
| 3 | **Supervisor-owned boot watchdog + auto-fallback (commit-confirmed)** | **CONFIRM, with REFINEMENT** | Supervisor-above-process is universal; systemd `WatchdogSec`+`StartLimitBurst` or a (young, 0.x, audit-required) Rust supervisor. REFINE: distinguish healthy from started-but-broken via **readiness probe**, not liveness. [watchdog-transcript] |
| 4 | **Satellite autonomous self-recovery (no central dependency)** | **CONFIRM (non-negotiable)** | "Can't rescue a device through a broken link" stated explicitly; Mender pre-commit health-check rollback + Cisco IoT auto-recovery are direct templates. REFINE: tiered confirm (local-ready vs dialed-home); locally-healthy-but-isolated → DEGRADED, not revert. [air-gap-transcript] |
| 5 | **Bootstrap-layer fleet canary (never all-at-once; local fallback + DEGRADED heartbeat)** | **CONFIRM** | Azure Device Update (%+min-count → group rollback) + Argo/Flagger (analysis-gate + progress deadline) + Mender/balena (per-device local rollback). REFINE: fleet halts widening; satellites self-revert locally. [staged-transcript] |
| 6 | **Safe-mode recovery console (both slots bad → minimal hardcoded boot, local-only admin)** | **CONFIRM** | Android recovery image + ChromeOS recovery mode + FPGA golden image are exactly this: a minimal, immutable, config-decoupled boot path with a local recovery interface. Essential for air-gapped/OT where there's no remote hand. [air-gap-transcript][AB-transcript] |

**Zero refutes.** All six elements are validated by prior art. Two (#1, #3) carry material refinements; the rest are confirmed largely as proposed. The design as a whole is a sound, well-precedented synthesis — its novelty (and its main design risk) is concentrated in the **satellite-can't-phone-home confirm-signal adaptation** (#4), which is correctly identified by the team as the crux and is supported but not turnkey-provided by prior art.

---

## Recommended bootstrap-recovery mechanism (concrete)

A four-layer recovery stack, from most-trusted/lowest to least:

1. **Write-time validate-before-persist (fail-closed) on the authoring path.** Validate cheap+deterministic properties: cert parses + key matches + not-expired; token well-formed + not-expired; port in range; transport selection valid; store-connection-string *format* valid (with an *advisory* connect probe that warns-but-allows-override). Reject the write if a deterministic check fails; keep the old bootstrap. (Catches the common case; honest that it can't catch racy/environment failures.)

2. **A/B dual-slot bootstrap.** Persist bootstrap in `active` (last-known-good) + `pending`. Boot `pending`. The boot/supervisor layer **never** self-promotes.

3. **Supervisor-owned boot watchdog + readiness-gated promotion + boot-count auto-fallback.** A supervisor above the main process boots `pending`, starts a watchdog, and waits for a **readiness signal** — the main process emits "ready" only after it bound its listen port, opened its store, loaded its identity, and passed self-checks (NOT mere process-alive). On readiness within the boot deadline → promote `pending`→`active`. On N failed/unhealthy boots (boot-count, à la U-Boot `bootlimit`/Android `slot-retry-count`) → revert to `active` and reboot. On a crash-loop cap (systemd `StartLimitBurst` analog) → stop and escalate to layer 4.
   - **Rust path:** if shipping on systemd, use **`sd-notify` 0.5.0** (`READY=1` readiness + `WATCHDOG=1` liveness) with `WatchdogSec`/`Restart=`/`StartLimitBurst`/`StartLimitAction`, and put boot-counting in the unit/generator layer. **This is the mature, lowest-risk option.** If a bundled PID-1 supervisor is required for the air-gapped/OT build, prototype against **`supervised`** or **`rust-tokio-supervisor`** but treat them as audit/vendor candidates (0.x), and keep a hardware **`/dev/watchdog`** as the bottom tier. (Backstop deps `tokio` 1.52.3 / `arc-swap` 1.9.1 / `sysinfo` 0.39.5 are all mature.)

4. **Safe-mode recovery console.** If BOTH slots are bad (or the crash-loop cap trips), boot a minimal, immutable, config-decoupled image (Android-recovery / ChromeOS-recovery / FPGA-golden model) that ignores dynamic config and exposes a **local-only admin endpoint** (serial/console/loopback) — so an operator with local/physical access always has a recovery path. Mandatory for air-gapped/OT where there is no remote hand.

**The satellite "can't-phone-home" path, resolved:** the satellite runs layers 1–4 entirely locally. Its restart-class "confirm" is a **tiered local signal**, NOT an operator action and NOT "reached central":
- **Tier 1 (revert trigger):** process reached *local* readiness. Failure to reach Tier 1 within the boot deadline trips the boot-count → revert to `active` (Mender pre-commit-script template). The supervisor owns this; no network needed.
- **Tier 2 (escalation, not revert):** successfully dialed home at least once. A satellite that reaches Tier 1 but never Tier 2 **keeps the new bootstrap and reports DEGRADED** on its next (best-effort) heartbeat — it must NOT revert a *locally-healthy* config merely because the WAN is down, or legitimate network outages would cause bootstrap flapping. (This is the precise adaptation the mainstream commit-confirmed pattern lacks, and the one Palo-Alto-Panorama-style "connectivity = confirm" gets *wrong* for an air-gapped node.)
- Central-driven mass-revert across reachable satellites (Azure-DU group-rollback) is a useful *additional* tier for reachable nodes, but is explicitly **not** the primary safety mechanism — the primary mechanism is autonomous local self-revert, because the worst-case satellite is unreachable by definition.

---

## Consolidated Open Design Questions

1. **What exactly constitutes "readiness" for a Prism satellite vs central appliance?** The supervisor's promotion gate hinges on this. Candidate predicate: listener bound + store handle open + identity loaded + a self-check query succeeds. Does "store handle open" mean RocksDB opened locally, the upstream store reachable, or both? (Affects whether a satellite with a bad *local* store can be distinguished from one with a bad *upstream* connection-string.)
2. **Boot deadline + boot-count + crash-loop cap values.** JunOS/NVUE/Flagger converge on 10 min for *operator/controller* windows; the *autonomous satellite boot* deadline should be much shorter (seconds–low-minutes) since no operator can extend it. What N (failed boots → fallback) and what crash-loop burst/interval? (U-Boot example bootlimit=5; Android leaves it device-configurable.)
3. **Tier-1-healthy-but-Tier-2-isolated dwell policy.** How long does a satellite stay DEGRADED-but-running on a new bootstrap it could never confirm to central before it *does* revert (if ever)? Never-revert-if-locally-healthy avoids flapping but risks a subtly-wrong-but-locally-OK config persisting. Is there a max isolation window?
4. **systemd dependency vs bundled PID-1.** Does the air-gapped/OT appliance build ship on systemd (→ mature `sd-notify` path) or must it bundle its own supervisor (→ young 0.x Rust crates needing audit/vendor)? This is an architecture decision with real maturity/risk consequences.
5. **Safe-mode console attack surface.** A local-only admin endpoint in safe mode is a recovery necessity but also a security surface (esp. for an LLM-facing security product with prompt-injection concerns per project memory). What authenticates to it? Physical presence only? This needs a security-reviewer pass before it's specced.
6. **Identity rotation interaction.** If the restart-class change IS a new identity/enrollment token and it's wrong, the satellite reverts to the *old* identity in `active`. Does central tolerate a satellite re-appearing on its *previous* identity after a failed rotation? (Enrollment/revocation semantics must round-trip with the A/B revert.)
7. **`[INCONCLUSIVE]` — formal OT standards anchor.** If 800-82 / 62443 fail-safe-state language is wanted as a normative basis for the safe-mode design, a dedicated standards-document pass is needed (this pass surfaced the principle but not quotable clauses).
8. **Config-A/B as a build-not-buy.** No turnkey OSS "A/B config-slot + boot-counting" framework was found; Prism assembles it from the U-Boot/systemd boot-counting pattern applied to config slots. Confirm the team accepts building this primitive.

---

## Honest Costs & Caveats

- **The hardest element (#4) is the least turnkey.** Prior art *validates the principle* (Mender pre-commit, Cisco IoT auto-recovery, Panorama) but none is a drop-in for "outbound-only dial-home satellite with tiered local confirm." Prism builds this; the references de-risk the design but don't supply the code.
- **"Port bindable" and "store connects" pre-flight checks will give false confidence if treated as hard gates.** The cited consensus across Envoy/nginx/HAProxy/sshd is that they deliberately do NOT pre-bind or pre-connect in validate mode for exactly this reason (TOCTOU + transient failure). Design element #1 must be scoped to the cheap/deterministic checks, with the boot-readiness gate as the real backstop. Over-promising on #1 is the most likely spec defect.
- **Rust bundled-supervisor crates are young (0.x).** Only `sd-notify` (0.5.0, 12M downloads) is mature, and it presupposes systemd. A bundled PID-1 supervisor for the air-gapped/OT build means depending on or vendoring a pre-1.0 crate — a real maturity cost under the production-grade default. Budget an audit.
- **A hardware watchdog (`/dev/watchdog`) is the only recovery tier that survives a supervisor bug.** If the supervisor itself is the broken component, only a hardware watchdog reboots the box. For a security appliance this should likely be mandatory, not optional — adds a hardware/platform requirement.
- **Safe-mode console is a new attack surface.** It must be specced with security-reviewer involvement; a local recovery endpoint that's exploitable defeats the appliance's security posture.
- **Transcript-tail caveat.** Four of five Perplexity responses exceeded the inline cap; the substantive bodies (every cited system, parameter, table, and caveat used above) were read, but the final ~13–20% of each oversized file (citation lists + minor recap) was not re-read line-by-line. The fifth (validate-before-persist) was read in full including citations. No finding above depends on an unread tail.
- **Version volatility.** All crate versions verified against crates.io/lib.rs on 2026-06-27; the supervisor-crate landscape is moving fast (several entrants in 2026). Re-verify at morph.
- **Standards `[INCONCLUSIVE]`.** No quotable NIST 800-82 / IEC 62443 clauses retrieved — only the general fail-safe-state principle.

---

## Sources (families; full numbered citations in saved transcripts)

- **Commit-confirmed transcript** [JunOS-docs, Cisco-IOSXE, Cisco-reload, Arista-EOS, NVUE, PaloAlto-Panorama]: Juniper TechLibrary `commit confirmed`/`commit at`/rollback; Cisco IOS XE Configuration Rollback Confirmed Change + `configure replace`/`revert timer`; Arista EOS config sessions + `commit timer`; NVIDIA NVUE `nv config apply --confirm`; Palo Alto LIVEcommunity + Panorama Automated Commit Recovery.
- **A/B boot transcript** [AB-transcript, Android-AB, ChromeOS-AB, U-Boot, RAUC, Mender, SWUpdate, systemd-boot, Cisco-config-AB, FPGA-golden]: AOSP A/B + boot control HAL/`bootctl`; ChromeOS GPT flags/`cgpt`; U-Boot Boot Count Limit (`bootcount`/`bootlimit`/`altbootcmd`); RAUC/Mender/SWUpdate docs + forums; systemd-boot boot counting + `boot-complete.target`; Cisco ASR commit-confirmed-for-config; Lattice MachXO3D dual-boot golden image.
- **Watchdog/supervisor transcript** [watchdog-transcript, systemd-watchdog, K8s-probes] + **crates.io/lib.rs verification** [sd-notify 0.5.0, libsystemd 0.7.2, rust-tokio-supervisor 0.1.4, processmanager 0.5.0, supervised 0.3.0, super-visor 0.3.0, ractor-supervisor, rust_supervisor, watchexec-supervisor 5.0.1, tokio 1.52.3, arc-swap 1.9.1, sysinfo 0.39.5, watchdog 0.2.6 (NOT a process watchdog)]: systemd.service man pages (WatchdogSec/Restart/StartLimit*); Kubernetes liveness/readiness/startup probe docs; crates.io + lib.rs version pages (verified 2026-06-27).
- **Validate-before-persist transcript** [validate-transcript, Envoy-validate, nginx-t, HAProxy-c, sshd-t, K8s-dry-run, port-race]: Envoy CLI `--mode validate` + bootstrap.proto + GH issues 499/39156; nginx `-t`/`-T` + GetPageSpeed/Gixy; HAProxy `-c` manual + blog; OpenSSH `sshd -t`/`-T` + `sshd-check-conf`; Kubernetes server-side dry-run; Oracle forum BindException (TOCTOU example); FRRouting dry-run exit-code issue; OpenTelemetry Collector `--dry-run` discussion.
- **Air-gapped recovery transcript** [air-gap-transcript, Mender, Cisco-IoT, Android-recovery, ChromeOS-recovery, Tailscale, Fleet, balena]: Mender pre-commit state scripts + rollback; Cisco IoT Operations Dashboard IR1101/IR1800 Auto Recovery; Android Boot/Recovery image model; Chromebook recovery docs; Tailscale ACL/grants docs + GH offline-semantics issue; FleetDM agent configuration docs; balenaCloud release pinning + balenaOS host-update rollback-health/rollback-altboot + forums.
- **Staged-rollout transcript** [staged-transcript, Azure-DU, AWS-ECS, Argo, Flagger, GKE, Mender, balena]: Mender phased rollout; balenaCloud release pinning + balenaOS update; Azure Device Update for IoT Hub deployment + rollback trigger policy; GKE node-pool surge/blue-green + PDBs; Argo Rollouts analysis (`failureLimit`/`consecutiveSuccessLimit`); Flagger Canary + progress deadline; AWS ECS deployment circuit breaker + CloudWatch alarms.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY, reasoning_effort=high)** | 5 | Deep multi-source synthesis on the 5 core themes: (Q1) commit-confirmed pattern across JunOS/Cisco/Arista/NVUE/Panorama; (Q2) A/B boot + boot-counting across Android/ChromeOS/U-Boot/RAUC/Mender/SWUpdate/systemd-boot + config-A/B mapping; (Q3) systemd watchdog + readiness-vs-liveness + Rust supervisor crate landscape; (Q4) validate-before-persist across Envoy/nginx/HAProxy/sshd + TOCTOU/transient-failure honesty; (Q5) air-gapped/can't-phone-home autonomous recovery across Mender/Cisco-IoT/Tailscale/Fleet/balena/Android/ChromeOS + OT reality; (Q6) staged/canary fleet rollout across Azure-DU/AWS-ECS/Argo/Flagger/GKE/Mender/balena. All 5 succeeded on first attempt at `high`. |
| Perplexity perplexity_reason | 0 | — |
| Perplexity perplexity_search | 1 | Raw ranked URLs for the Rust process-supervisor/watchdog crate landscape (2025/2026) — to name candidate crates for crates.io verification. |
| Perplexity perplexity_ask | 0 | — |
| Context7 | 0 | — (library APIs not the question; crate state verified directly via crates.io) |
| Tavily (any) | 0 | — |
| WebFetch (crates.io / lib.rs) | 8 | Version verification: sd-notify (0.5.0), libsystemd (0.7.2), tokio (1.52.3, confirmed via /versions), arc-swap (1.9.1), sysinfo (0.39.5), watchdog (0.2.6 — confirmed it is a filesystem watcher, NOT a process watchdog). |
| WebSearch | 0 | — |
| Training data | 2 areas (flagged) | `[model-knowledge]`: the readiness-vs-liveness distinction's *application* to the started-but-broken case; the inference that boot/supervisor layers "never self-promote" generalizes (each individually cited, generalization is model reasoning). |

**Total MCP tool calls:** 6 (5× `perplexity_research` at high + 1× `perplexity_search`). **Plus** 8 crates.io/lib.rs version-verification WebFetch calls.
**Training data reliance:** **low** — every design-element verdict is anchored to a cited deep-research finding; all version numbers verified against crates.io (NOT training data); the two model-knowledge flags are reasoning *over* cited evidence, not substitute facts.
