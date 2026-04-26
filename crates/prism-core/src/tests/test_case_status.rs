// Unit tests for CaseStatus state machine.
//
// AC coverage: AC-1, AC-2, AC-3
// VP coverage: VP-005 (12 valid transitions), VP-006 (no self-transitions),
//              VP-051 (advance_case_state exhaustive table)
//
// All tests pass (implementation complete).

#[cfg(test)]
mod tests {
    use crate::case::{advance_case_state, CaseStatus, CaseTransitionError, VALID_TRANSITIONS};

    // ── AC-1: forward linear New → Acknowledged ───────────────────────────────

    #[test]
    fn test_BC_S_02_001_ac1_new_to_acknowledged_is_valid() {
        // AC-1: New → Acknowledged returns true.
        assert!(CaseStatus::New.can_transition_to(CaseStatus::Acknowledged));
    }

    // ── AC-2: skip-ahead New → Closed ────────────────────────────────────────

    #[test]
    fn test_BC_S_02_001_ac2_new_to_closed_is_valid() {
        // AC-2: New → Closed (skip-ahead) returns true.
        assert!(CaseStatus::New.can_transition_to(CaseStatus::Closed));
    }

    // ── AC-3: Resolved → Closed state machine returns true ───────────────────

    #[test]
    fn test_BC_S_02_001_ac3_resolved_to_closed_state_machine_returns_true() {
        // AC-3: CaseStatus itself allows Resolved → Closed; disposition is
        // enforced in prism-operations, not here.
        assert!(CaseStatus::Resolved.can_transition_to(CaseStatus::Closed));
    }

    // ── VP-005: exactly 12 valid transitions ─────────────────────────────────

    #[test]
    fn test_BC_S_02_001_vp005_exactly_12_valid_transitions() {
        let mut count = 0usize;
        for from in CaseStatus::ALL {
            for to in CaseStatus::ALL {
                if from.can_transition_to(to) {
                    count += 1;
                }
            }
        }
        assert_eq!(
            count, 12,
            "expected exactly 12 valid transitions, got {count}"
        );
    }

    // ── VP-005: all 12 specific valid pairs return true ───────────────────────

    #[test]
    fn test_BC_S_02_001_vp005_all_12_valid_pairs_return_true() {
        for (from, to) in VALID_TRANSITIONS {
            assert!(
                from.can_transition_to(to),
                "{from:?} → {to:?} should be valid but returned false"
            );
        }
    }

    // ── VP-005: 13 invalid pairs return false ────────────────────────────────

    #[test]
    fn test_BC_S_02_001_vp005_invalid_transitions_return_false() {
        // Build complete 5×5 table; subtract VALID_TRANSITIONS; remainder must
        // all return false.
        for from in CaseStatus::ALL {
            for to in CaseStatus::ALL {
                if !VALID_TRANSITIONS.contains(&(from, to)) {
                    assert!(
                        !from.can_transition_to(to),
                        "{from:?} → {to:?} should be invalid but returned true"
                    );
                }
            }
        }
    }

    // ── VP-006: no self-transitions ───────────────────────────────────────────

    #[test]
    fn test_BC_S_02_001_vp006_no_self_transitions() {
        for status in CaseStatus::ALL {
            assert!(
                !status.can_transition_to(status),
                "{status:?} → {status:?} self-transition must return false"
            );
        }
    }

    // ── Sample backward/non-existent transitions return false ────────────────

    #[test]
    fn test_BC_S_02_001_acknowledged_to_new_is_invalid() {
        assert!(!CaseStatus::Acknowledged.can_transition_to(CaseStatus::New));
    }

    #[test]
    fn test_BC_S_02_001_closed_to_new_is_invalid() {
        assert!(!CaseStatus::Closed.can_transition_to(CaseStatus::New));
    }

    #[test]
    fn test_BC_S_02_001_resolved_to_new_is_invalid() {
        assert!(!CaseStatus::Resolved.can_transition_to(CaseStatus::New));
    }

    // ── VP-051: advance_case_state returns correct Ok/Err per pair ────────────

    #[test]
    fn test_BC_S_02_001_vp051_valid_transition_returns_ok() {
        // New → Acknowledged is valid.
        let result = advance_case_state(CaseStatus::New, CaseStatus::Acknowledged);
        assert_eq!(result, Ok(CaseStatus::Acknowledged));
    }

    #[test]
    fn test_BC_S_02_001_vp051_self_transition_returns_e_case_005() {
        let result = advance_case_state(CaseStatus::New, CaseStatus::New);
        assert_eq!(result, Err(CaseTransitionError::SelfTransition));
    }

    #[test]
    fn test_BC_S_02_001_vp051_invalid_non_self_returns_e_case_004() {
        // Acknowledged → New is not in the 12-valid set and is not a self-transition.
        let result = advance_case_state(CaseStatus::Acknowledged, CaseStatus::New);
        assert_eq!(result, Err(CaseTransitionError::InvalidTransition));
    }

    #[test]
    fn test_BC_S_02_001_vp051_exhaustive_25_pairs_correct_outcome() {
        // Exhaustive runtime verification of all 25 pairs.
        for from in CaseStatus::ALL {
            for to in CaseStatus::ALL {
                let result = advance_case_state(from, to);
                if from == to {
                    assert_eq!(
                        result,
                        Err(CaseTransitionError::SelfTransition),
                        "{from:?} → {to:?} self-transition must be E-CASE-005"
                    );
                } else if VALID_TRANSITIONS.contains(&(from, to)) {
                    assert_eq!(
                        result,
                        Ok(to),
                        "{from:?} → {to:?} valid transition must return Ok({to:?})"
                    );
                } else {
                    assert_eq!(
                        result,
                        Err(CaseTransitionError::InvalidTransition),
                        "{from:?} → {to:?} invalid transition must be E-CASE-004"
                    );
                }
            }
        }
    }

    // ── Reopen transitions ────────────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_001_resolved_to_investigating_reopen_is_valid() {
        assert!(CaseStatus::Resolved.can_transition_to(CaseStatus::Investigating));
    }

    #[test]
    fn test_BC_S_02_001_closed_to_investigating_reopen_is_valid() {
        assert!(CaseStatus::Closed.can_transition_to(CaseStatus::Investigating));
    }

    // ── valid_transitions slice matches VALID_TRANSITIONS ─────────────────────

    #[test]
    fn test_BC_S_02_001_valid_transitions_slice_consistent_with_can_transition_to() {
        for from in CaseStatus::ALL {
            let slice = from.valid_transitions();
            for &to in slice {
                assert!(
                    from.can_transition_to(to),
                    "valid_transitions() listed {to:?} for {from:?} but can_transition_to returned false"
                );
            }
            // Count must match entries in VALID_TRANSITIONS for this `from` state.
            let expected_count = VALID_TRANSITIONS.iter().filter(|(f, _)| *f == from).count();
            assert_eq!(
                slice.len(),
                expected_count,
                "valid_transitions() for {from:?} has wrong length"
            );
        }
    }
}
