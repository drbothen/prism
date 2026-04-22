// Unit tests for CredentialName.
//
// AC coverage: AC-4, AC-5
// VP coverage: VP-011 (path traversal rejection)

#[cfg(test)]
mod tests {
    use crate::credentials::{CredentialName, CREDENTIAL_NAME_MAX_LEN};
    use crate::error::PrismError;

    // ── AC-4: path traversal "../../passwd" rejected ──────────────────────────

    #[test]
    fn test_BC_S_02_003_ac4_rejects_path_traversal_double_dot() {
        let result = CredentialName::new("../../passwd");
        assert!(
            matches!(result, Err(PrismError::InvalidCredentialName(_))),
            "path traversal with .. must be rejected"
        );
    }

    // ── AC-5: null byte rejected ──────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_ac5_rejects_null_byte() {
        let result = CredentialName::new("key\0value");
        assert!(
            matches!(result, Err(PrismError::InvalidCredentialName(_))),
            "null byte in name must be rejected"
        );
    }

    // ── VP-011: forward slash rejected ───────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_vp011_rejects_forward_slash() {
        let result = CredentialName::new("a/b");
        assert!(
            result.is_err(),
            "forward slash must be rejected (path traversal)"
        );
    }

    // ── VP-011: backslash rejected ────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_vp011_rejects_backslash() {
        let result = CredentialName::new("a\\b");
        assert!(
            result.is_err(),
            "backslash must be rejected (path traversal)"
        );
    }

    // ── VP-011: absolute path rejected ───────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_vp011_rejects_absolute_path() {
        let result = CredentialName::new("/etc/passwd");
        assert!(result.is_err(), "absolute path must be rejected");
    }

    // ── Empty string rejected ─────────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_rejects_empty_string() {
        let result = CredentialName::new("");
        assert!(
            matches!(result, Err(PrismError::InvalidCredentialName(_))),
            "empty credential name must be rejected"
        );
    }

    // ── Length exactly 128 is valid ───────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_accepts_name_at_max_length() {
        let name = "a".repeat(CREDENTIAL_NAME_MAX_LEN);
        let result = CredentialName::new(&name);
        assert!(
            result.is_ok(),
            "credential name of exactly {CREDENTIAL_NAME_MAX_LEN} chars must be accepted"
        );
    }

    // ── Length 129 is rejected ────────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_rejects_name_exceeding_max_length() {
        let name = "a".repeat(CREDENTIAL_NAME_MAX_LEN + 1);
        let result = CredentialName::new(&name);
        assert!(
            matches!(result, Err(PrismError::InvalidCredentialName(_))),
            "credential name longer than {CREDENTIAL_NAME_MAX_LEN} chars must be rejected"
        );
    }

    // ── Valid name accepted ───────────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_accepts_valid_name() {
        let result = CredentialName::new("crowdstrike-prod");
        assert!(result.is_ok(), "valid alphanumeric-hyphen name must be accepted");
    }

    // ── as_str round-trip ─────────────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_as_str_round_trip() {
        let name = CredentialName::new("my-sensor-key").unwrap();
        assert_eq!(name.as_str(), "my-sensor-key");
    }

    // ── Clone equality ────────────────────────────────────────────────────────

    #[test]
    fn test_BC_S_02_003_clone_equality() {
        let a = CredentialName::new("clone-test").unwrap();
        let b = a.clone();
        assert_eq!(a, b);
    }
}
