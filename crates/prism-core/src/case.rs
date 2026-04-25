// S-1.02 — Case entity types and state machine.
//
// CaseStatus encodes exactly 12 valid transitions (VP-005, VP-006, VP-051):
//
//   Forward linear (4):
//     New → Acknowledged, Acknowledged → Investigating,
//     Investigating → Resolved, Resolved → Closed
//
//   Skip-ahead (6):
//     New → Investigating, New → Resolved, New → Closed,
//     Acknowledged → Resolved, Acknowledged → Closed,
//     Investigating → Closed
//
//   Reopen (2):
//     Resolved → Investigating, Closed → Investigating
//
//   Total: 12 valid, 13 invalid (including all 5 self-transitions).
//
// The canonical transition table is expressed as a const array so the Kani proofs
// and the runtime function share the exact same source of truth — no duplication.

use serde::{Deserialize, Serialize};

// ── Canonical transition table (single source of truth) ──────────────────────

/// All 12 valid `(from, to)` transitions per BC-2.14.002.
/// Used by both `can_transition_to` and the Kani proofs.
pub const VALID_TRANSITIONS: [(CaseStatus, CaseStatus); 12] = [
    // Forward linear
    (CaseStatus::New, CaseStatus::Acknowledged),
    (CaseStatus::Acknowledged, CaseStatus::Investigating),
    (CaseStatus::Investigating, CaseStatus::Resolved),
    (CaseStatus::Resolved, CaseStatus::Closed),
    // Skip-ahead
    (CaseStatus::New, CaseStatus::Investigating),
    (CaseStatus::New, CaseStatus::Resolved),
    (CaseStatus::New, CaseStatus::Closed),
    (CaseStatus::Acknowledged, CaseStatus::Resolved),
    (CaseStatus::Acknowledged, CaseStatus::Closed),
    (CaseStatus::Investigating, CaseStatus::Closed),
    // Reopen
    (CaseStatus::Resolved, CaseStatus::Investigating),
    (CaseStatus::Closed, CaseStatus::Investigating),
];

// ── CaseStatus ────────────────────────────────────────────────────────────────

/// Lifecycle status of a security case (BC-2.14.002).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum CaseStatus {
    New,
    Acknowledged,
    Investigating,
    Resolved,
    Closed,
}

impl CaseStatus {
    /// Returns `true` if transitioning from `self` to `next` is a valid move per
    /// the 12-transition table defined in `VALID_TRANSITIONS`.
    ///
    /// Self-transitions always return `false` (VP-006).
    pub fn can_transition_to(&self, next: CaseStatus) -> bool {
        VALID_TRANSITIONS.contains(&(*self, next))
    }

    /// Returns all reachable states from `self` as a static slice, used by the
    /// MCP tool for hint generation.
    pub fn valid_transitions(&self) -> &'static [CaseStatus] {
        match self {
            CaseStatus::New => &[
                CaseStatus::Acknowledged,
                CaseStatus::Investigating,
                CaseStatus::Resolved,
                CaseStatus::Closed,
            ],
            CaseStatus::Acknowledged => &[
                CaseStatus::Investigating,
                CaseStatus::Resolved,
                CaseStatus::Closed,
            ],
            CaseStatus::Investigating => &[CaseStatus::Resolved, CaseStatus::Closed],
            CaseStatus::Resolved => &[CaseStatus::Closed, CaseStatus::Investigating],
            CaseStatus::Closed => &[CaseStatus::Investigating],
        }
    }

    /// All `CaseStatus` variants in definition order (used by Kani exhaustive proofs).
    pub const ALL: [CaseStatus; 5] = [
        CaseStatus::New,
        CaseStatus::Acknowledged,
        CaseStatus::Investigating,
        CaseStatus::Resolved,
        CaseStatus::Closed,
    ];
}

// ── advance_case_state ────────────────────────────────────────────────────────

/// Error codes returned by `advance_case_state`, matching BC-2.14.002.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseTransitionError {
    /// E-CASE-004 — the requested transition is not in the 12-valid set.
    InvalidTransition,
    /// E-CASE-005 — the caller attempted a self-transition.
    SelfTransition,
}

/// Advance a case from `current` to `next`, enforcing the 12-transition table.
///
/// Returns `Ok(next)` for valid transitions.
/// Returns `Err(CaseTransitionError::SelfTransition)` for self-transitions (E-CASE-005).
/// Returns `Err(CaseTransitionError::InvalidTransition)` for all other invalid moves
/// (E-CASE-004).
///
/// Used by VP-051 exhaustive transition table proof.
pub fn advance_case_state(
    current: CaseStatus,
    next: CaseStatus,
) -> Result<CaseStatus, CaseTransitionError> {
    if current == next {
        return Err(CaseTransitionError::SelfTransition);
    }
    if current.can_transition_to(next) {
        Ok(next)
    } else {
        Err(CaseTransitionError::InvalidTransition)
    }
}

// ── DispositionCode ───────────────────────────────────────────────────────────

/// Disposition code applied when a case transitions to `Resolved`.
///
/// Enforcement (requiring a code on Resolved transitions) lives in prism-operations
/// (S-4.06), not here. This type is purely a domain value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DispositionCode {
    TruePositive,
    FalsePositive,
    Benign,
    Inconclusive,
    Duplicate,
    TestAlert,
}

// ── TimelineEntryType ─────────────────────────────────────────────────────────

/// Category of a case timeline entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimelineEntryType {
    Note,
    StatusChange,
    AlertLink,
    EvidenceLink,
    OtImpact,
}
