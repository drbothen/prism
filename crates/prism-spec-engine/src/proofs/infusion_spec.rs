//! VP-048 Kani proof harnesses — N fields produces exactly N UDF descriptors.
//!
//! Proves BC-2.19.001 postconditions:
//! - `InfusionRegistry::load_spec` with N valid distinct fields → exactly N descriptors.
//! - Any duplicate field name → `Err(InfusionError::DuplicateUdfName)`.
//!
//! Method: Kani (cfg-gated with `#[cfg(kani)]`).
//! These harnesses are authored here as Red Gate stubs — the actual proofs run
//! after S-1.14 implementation completes (Phase 5 formal-verify).
//!
//! # Red Gate note
//! The proof harnesses reference `InfusionRegistry::load_spec` which is currently
//! `unimplemented!()`. The Kani proofs will NOT compile/run until implementation
//! exists. This file establishes the proof structure.

#[cfg(kani)]
mod kani_proofs {
    use crate::infusion::{
        BuiltInSourceType, CredentialRef, InfusionField, InfusionRegistry, InfusionSpec,
        InfusionType,
    };
    use prism_core::InfusionError;

    /// Build an `InfusionSpec` with `n` distinct fields for Kani symbolic verification.
    ///
    /// All field names are distinct (kani::assume enforces uniqueness).
    fn build_spec_distinct_fields(n: usize) -> InfusionSpec {
        let mut fields = Vec::with_capacity(n);
        for i in 0..n {
            fields.push(InfusionField {
                name: format!("udf_field_{}", i),
                input_field: "device_ip".to_string(),
                input_type: "ip".to_string(),
                output_type: "string".to_string(),
                description: None,
                source_column: Some(format!("col_{}", i)),
            });
        }
        InfusionSpec {
            infusion_id: "test_infusion".to_string(),
            name: "Test Infusion".to_string(),
            infusion_type: InfusionType::LocalLookup,
            source: None,
            fields,
            pipe_stage: None,
            plugin_config: None,
            credentials: vec![],
            source_path: "test.infusion.toml".to_string(),
            cache_ttl_secs: None,
        }
    }

    /// VP-048 Harness 1: N distinct fields → exactly N descriptors.
    ///
    /// Traces to: BC-2.19.001 postcondition (INV-INFUSE-001).
    #[kani::proof]
    fn verify_n_fields_n_descriptors() {
        let n: usize = kani::any();
        kani::assume(n >= 1 && n <= 16);

        let spec = build_spec_distinct_fields(n);
        let registry = InfusionRegistry::new();

        let result = registry.load_spec(spec);

        match result {
            Ok(descriptors) => {
                kani::assert(
                    descriptors.len() == n,
                    "VP-048: N distinct fields must produce exactly N UDF descriptors",
                );
            }
            Err(_) => {
                kani::assert(
                    false,
                    "VP-048: distinct fields must not produce an error",
                );
            }
        }
    }

    /// VP-048 Harness 2: Duplicate UDF name → `Err(DuplicateUdfName)`.
    ///
    /// Traces to: BC-2.19.001 postcondition (duplicate UDF detection).
    #[kani::proof]
    fn verify_duplicate_udf_name_errors() {
        // Spec with two fields sharing the same UDF name.
        let duplicate_field = InfusionField {
            name: "duplicate_udf".to_string(),
            input_field: "device_ip".to_string(),
            input_type: "ip".to_string(),
            output_type: "string".to_string(),
            description: None,
            source_column: Some("country".to_string()),
        };
        let spec_with_duplicate = InfusionSpec {
            infusion_id: "dup_test".to_string(),
            name: "Duplicate Test".to_string(),
            infusion_type: InfusionType::LocalLookup,
            source: None,
            fields: vec![duplicate_field.clone(), duplicate_field],
            pipe_stage: None,
            plugin_config: None,
            credentials: vec![],
            source_path: "dup_test.infusion.toml".to_string(),
            cache_ttl_secs: None,
        };

        let registry = InfusionRegistry::new();
        let result = registry.load_spec(spec_with_duplicate);

        kani::assert(result.is_err(), "VP-048: duplicate UDF name must produce Err");
        if let Err(e) = result {
            kani::assert(
                matches!(e, InfusionError::DuplicateUdfName { .. }),
                "VP-048: error must be E-INFUSE-002 / DuplicateUdfName",
            );
        }
    }
}

// Compile-time sanity: the types referenced in proofs must exist.
#[cfg(test)]
mod compile_check {
    use crate::infusion::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

    /// This test verifies the proof harness types compile correctly.
    /// It does NOT call unimplemented methods — just constructs types.
    #[test]
    fn test_BC_2_19_001_proof_types_compile() {
        let _field = InfusionField {
            name: "geoip_country".to_string(),
            input_field: "device_ip".to_string(),
            input_type: "ip".to_string(),
            output_type: "string".to_string(),
            description: None,
            source_column: Some("country_iso_code".to_string()),
        };
        let _registry = InfusionRegistry::new();
        // Types compile — proof harness structure is valid.
    }
}
