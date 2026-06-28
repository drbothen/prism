---
document_type: positioning-narrative
status: capture
do_not_execute: true
candidate: true
audience: "customer / executive (non-technical)"
gated_on: "§5.1 brief-reframe human sign-off (PENDING)"
produced_by: product-owner
timestamp: "2026-06-28"
provenance: >
  Out-of-band side-analysis; plain-language rendering of the problem-framed positioning
  candidate. Synthesized from ADR-PROP-positioning-problem-framed.md (§3 three pillars,
  §4 candidate headlines, §5 binding honest concessions, §6 adversarial caveats, §8
  feature map) and ADR-PROP-competitive-positioning.md (D-C10-5 headline, D-C10-3 honest
  concessions). Touches no live artifacts — no STATE.md, no SESSION-HANDOFF.md, no live
  ADR registry, no BC, no story, no ARCH-INDEX.md.
---

# Prism — Plain-Language Positioning Narrative (DRAFT)

> **This is a draft candidate narrative pending human sign-off at §5.1.**
> It is not a final sales claim or approved external statement.

---

## One-Line Value Statement (DRAFT — for sign-off)

> **Prism lets your existing IT security team watch your industrial and operational
> systems too — your data never leaves your site, your team stops drowning in alerts,
> and an AI does the heavy lifting so you don't have to hire the OT security specialist
> you can't find.**

---

## The Problems We Hear from Security Teams

Security teams protecting industrial and operational environments — factories, utilities,
healthcare facilities, critical infrastructure — describe the same handful of painful
problems over and over. Here is how those problems sound in their own words.

---

### Theme A: "Keep my data home, but let my team see everything"

**Problem 3 — "We can't send our data to the cloud."**

For many organizations — especially utilities, manufacturers, and healthcare systems —
security and regulatory rules require that operational data stays on-site. Sending raw
sensor or network data to a cloud service is simply not an option. But every vendor seems
to want you to stream everything to their platform.

**Problem 4 — "I want my IT security team to cover our industrial systems, but they
don't speak OT."**

Most companies have a solid IT security team. Industrial systems (control systems,
sensors, factory equipment) speak entirely different technical languages — Modbus,
OPC-UA, Profinet — that IT analysts have never encountered. Getting useful visibility
into the factory floor means either retraining your whole IT team or hiring expensive
specialists. Neither is easy.

---

### Theme B: "Help me understand what I actually have"

**Problem 5 — "We don't even know what devices are on our network."**

You can't protect what you can't see. Many organizations running industrial or operational
technology have never had a clean, current list of what's actually connected. Devices get
added over years or decades without central tracking.

**Problem 6 — "I don't understand how my environment is connected or what matters most."**

Even if you have a device list, knowing which systems talk to which, which ones would be
catastrophic to lose, and how an attacker could move from one to another is a completely
different problem. Most security teams are flying blind on the relationships.

---

### Theme C: "Let me hunt threats affordably with the team I can actually staff"

**Problem 1 — "Threat hunting costs too much."**

Traditional security platforms require you to copy all your data into one central place
and keep it there. You pay for storage whether you use it or not. For operational
technology environments with large volumes of sensor and network data, this gets
expensive fast — often too expensive to justify.

**Problem 2 — "Tuning detections is a nightmare. Too many false alarms."**

Every security team deals with detection rules that fire too often on harmless activity.
Silencing them feels risky (what if the next one is real?). Tuning them properly takes
expert judgment and time you don't have. The result: alert fatigue and rules that nobody
trusts.

**Problem 7 — "We can't hire enough skilled people — especially people who know both
IT and OT security."**

The security talent shortage is real, and OT security specialists are among the hardest
people to find and keep. Most teams are stretched thin. The idea of building a
dedicated OT security practice from scratch is daunting, and often not realistic.

---

## How Prism Helps

### Theme A — Your data stays home; your IT team sees everything

**The pain:** You are forced to choose between keeping your data on-site and having your
IT security team actually see your operational environment. Neither the data-streaming
cloud vendors nor expensive OT-specialist consultants solve both problems at once.

**What Prism does:** Prism runs a small software component directly on your site — inside
your network, on your infrastructure. That component queries your industrial systems,
normalizes what it finds into the same readable format your IT analysts already work with,
and sends only the answers — not the raw data — back to the analyst console. Your raw
operational data never travels off your premises. Only the normalized, security-relevant
results do, encrypted end-to-end with a key that only you hold.

Because the translation into IT-readable format happens on your site, your existing IT
security team can query your factory floor, your control systems, and your sensor networks
using the same tools and language they already use for your corporate IT environment —
without learning a new protocol or hiring an OT specialist.

For environments that cannot connect to the internet at all — truly air-gapped operations
— Prism is designed to work fully offline, with updates delivered as signed, verifiable
packages.

**The business outcome:** No data leaves your site without your key. Your IT SOC covers
OT without retraining. Compliance auditors see data residency met by construction, not
by policy alone.

---

### Theme B — Know your environment: what's on it and how it connects

**The pain:** Incomplete asset inventories and no clear picture of how systems relate to
each other mean that when something goes wrong — or when you need to assess your risk
exposure — you are working from guesswork.

**What Prism does:** Prism connects to the OT asset discovery tools you already have —
platforms like Claroty, Armis, Dragos, Nozomi, and others — and pulls their inventory
into a single, searchable view alongside your IT assets. If you do not yet have an OT
discovery platform, Prism can directly check field devices itself as a fallback option
(note: direct device polling is a capability we are approaching carefully, with safety and
operational risk questions being resolved before we enable it broadly).

Beyond the inventory, Prism builds a live relationship map of your environment: which
systems connect to which, what industrial zone each device lives in (from your most
exposed corporate network down to your most sensitive control-system level), and how
critical each asset is. When a vulnerability appears in the news, Prism can immediately
show you which of your specific assets are affected, in which zones, with what criticality,
and whether any compensating controls are already in place.

Every answer Prism gives cites its source — which event, which asset record, which
sensor — so your team can verify and trust what they see.

**The business outcome:** A current, cross-domain asset inventory in one place.
Immediate, evidence-backed answers to "which of our systems is affected by this
vulnerability." An attack-path picture your team can actually act on.

---

### Theme C — Affordable threat hunting with the team you have

**The pain:** Hunting for threats is too expensive, too time-consuming, and requires
expertise your team does not have — especially across IT and OT together.

**What Prism does:**

On cost: Prism asks your existing security systems for answers on demand, instead of
copying all your data into one expensive central pile. You pay for what you actually
query, not for a warehouse of data sitting idle. Think of it as asking your systems
a question and getting an answer, rather than shipping all your records to a library
and hoping you can find what you need later.

On alert fatigue: Prism's detection system accumulates suspicious events as a risk
score on the specific device or user they concern, rather than firing a separate alarm
for every individual event. Your analyst sees the full story of a suspicious entity —
"this device has had five odd behaviors over three days" — instead of five separate
unrelated alerts. When you need to silence a noisy detection, Prism requires a written
reason and an expiration date; suppressions cannot silently linger forever. If a
detection rule starts misbehaving, Prism can automatically roll it back to its last
known-good version before it causes harm.

On talent: Prism includes an AI assistant that can take a plain-English description of
what you want to find — "show me any control-system device that communicated outbound
in the last 24 hours" — and turn it into a proper detection query. A junior IT analyst
with no OT background can author and run investigations that previously required a
senior OT security expert. The AI does the translation; your analyst stays in control
and approves every action before anything happens. Nothing is autonomous in the first
version of Prism. The AI recommends; your people decide.

For the cases that need deeper investigation, Prism's AI agent produces a structured
package: a written investigation report, a complete log of every query it ran (so you
can replay or audit its work), a list of every indicator it found, and a self-check
that verifies its own reasoning before handing you the result.

**The business outcome:** Hunting costs go down — you pay for queries, not a data
warehouse. Alert fatigue goes down — your team sees entity stories, not noise.
A junior IT analyst can cover OT investigations without requiring a specialist hire.

---

## Why This Is Different

- **Your data never leaves your site.** The raw content from your industrial systems stays
  on your premises. Only the normalized, security-relevant answers travel — encrypted with
  a key that only you hold. This is true in Prism's standard cloud-managed model, in a
  fully self-hosted model, and in a fully offline air-gap model. It is built into the
  architecture, not a configuration option.

- **Your IT team can cover OT without becoming OT experts.** The translation between
  industrial protocols and IT-readable security data happens on your site, automatically.
  Your existing analysts use the same tools and see the same kind of data they already
  know — just covering more of your environment.

- **An AI does the grunt work; your people stay in control.** Prism's AI assistant
  translates plain English into investigations, surfaces risk-scored entity timelines
  instead of alert floods, and hands your analyst a recommendation with full evidence
  attached. Every action requires human approval. This is a force-multiplier for the
  team you already have, not a replacement for human judgment.

- **Works even when you're disconnected from the internet.** For facilities that cannot
  connect to the cloud — truly isolated operational environments — Prism is designed
  to run fully on-site, with no internet dependency for normal operations. Configuration
  and intelligence updates arrive as cryptographically signed, verifiable packages
  delivered offline.

---

## Where We Are Today

Prism is in active development. The architecture described in this document reflects
decisions that have been made and are committed to — this is what we are building toward.
Several capabilities (including the out-of-the-box detection rule library, direct
industrial device polling, and the full AI investigation package) are on the near-term
roadmap and not yet shipped.

What this means for you: the cost model, the data residency guarantees, the IT/OT
convergence approach, and the core query and detection infrastructure are all
architecturally settled and being implemented. The OOTB rule library and some of the
AI agent's more advanced output features will follow.

We will not claim something ships until it ships. We will tell you clearly what is
available today and what is coming. That honesty is part of how we intend to earn
your trust.

---

> **Draft status:** This narrative is a candidate positioning document pending final
> review and sign-off. It is not an approved sales claim, product announcement, or
> external-facing statement. All product capabilities described as roadmap are subject
> to change.
