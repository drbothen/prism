---
adr_id: ADR-001
title: "DTU Rate-Limit Pattern — Per-Clone Semantics vs FailureLayer"
document_type: architecture-section
level: ADR
section: decisions/ADR-001-dtu-rate-limit-pattern
version: "1.0"
status: accepted
producer: state-manager
timestamp: 2026-04-22T00:00:00
phase: phase-3-dtu-wave-0
inputs: [specs/architecture/dtu-assessment.md, cycles/phase-3-dtu-wave-0/wave-gates/wave-0-retrospective.md]
traces_to: ARCH-INDEX.md
flagged_by: adversary F-WV0-003 (wave 0 retrospective gate)
---

# ADR-001: DTU Rate-Limit Pattern

> **Sizing guidance:** Each architecture section file targets 800-1,200 tokens
> (~50-80 lines of markdown). If this section exceeds 1,500 tokens, consider
> splitting it further into sub-sections.

## [Section Content]

### Context
The adversary flagged that prism-dtu-threatintel and prism-dtu-nvd each implement
their own rate-limit logic (AtomicU32 counter + threshold in ThreatIntel; dual-bucket
5/30s + 50/30s window in NVD) instead of using prism-dtu-common::FailureLayer's
RateLimit variant. The two clones use **different semantic models** for the same
concept, and neither matches FailureLayer::RateLimit { after_n_requests, retry_after_secs }.

### Decision
**Per-clone rate-limit is intentional.** Each DTU clone that needs API-realistic
rate-limit behavior implements its own logic matching the real vendor API's rules.

FailureLayer::RateLimit stays in prism-dtu-common as a **generic failure-injection
utility** for tests that do NOT require realistic rate-limit semantics (e.g.,
fidelity-validator tests that just need to exercise the 429 response path).

### Rationale
1. Real vendor APIs have heterogeneous rate-limit rules:
   - NVD: 5 requests / 30s unauthenticated, 50 / 30s authenticated
   - Crowdstrike (future): 6000 requests / 1 minute by token tier
   - ThreatIntel aggregator (today's ThreatIntel clone): per-subscription tiered
   A single generic layer cannot capture this diversity without becoming a
   configuration-driven rules engine (over-engineering for test infrastructure).
2. L2+ fidelity REQUIRES matching the real API's rate-limit semantics. A generic
   counter is L1 fidelity at best.
3. FailureLayer remains valuable for L1 generic-injection scenarios and future
   DTUs where L1 is sufficient.

### Consequences
- Each new DTU clone (S-6.07..S-6.19) will implement its own rate-limit if realistic
  semantics are required. A canonical pattern should be established in the next
  L2+ clone story's acceptance criteria.
- prism-dtu-common::FailureLayer MUST NOT be required by DTU clone stories; it's
  optional infrastructure.
- Documentation/fidelity-guide.md (not yet written) should distinguish L1 vs L2+
  fidelity requirements per clone.

### Alternatives Considered
- (A) Promote rate-limit to the trait: makes BehavioralClone bloated, forces L1 clones to implement fake logic.
- (B) Configuration-driven rate-limit rules engine in common: speculative over-engineering; current DTU count is 14, not 100.
- (C) Status quo (accepted): each clone owns its rate-limit, FailureLayer stays generic.

### Related
- F-WV0-003 (wave 0 retrospective adversary)
- Story: S-6.06 prism-dtu-common scope
- Future: each L2+ clone story's ACs should reference this ADR
