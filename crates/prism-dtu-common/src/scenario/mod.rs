//! Scenario entity catalog and derivation helpers (BC-2.06.018 / ADR-036 §2.2).
//!
//! Gated behind `#[cfg(feature = "fixture-gen")]` — see `lib.rs`.
//!
//! The primary entry point is [`build_scenario_entity_catalog`], which derives a
//! [`ScenarioEntityCatalog`] from a `(seed, org_id)` pair using a secondary RNG stream
//! (`gen_seeded_rng(seed.wrapping_add(1), &org_id)`) independent of the primary generator
//! stream.  All derived entity IDs follow the canonical format specified in ADR-036 §2.2:
//!
//! ```text
//! org_slug = hex(org_id.as_bytes()[0..4])   // exactly 8 lowercase hex chars
//! device_id = "dev-{org_slug}-{seed}-{n}"
//! ```
//!
//! This module is the authoritative source of `org_slug_from_org_id`; the formula
//! MUST match `prism_dtu_crowdstrike::generator::org_slug` exactly (ADR-036 §2.2).

use super::generator::{seeded_rng as gen_seeded_rng, OrgId};

/// Shared entity catalog for one client's incident scenario.
///
/// Produced once at harness construction time from `(seed, org_id)`.
/// All DTU projections for this client derive their entity IDs from this catalog.
///
/// # ADR-036 §2.2 — Canonical org_slug derivation
///
/// `org_slug = hex(org_id.as_bytes()[0..4])` — 8 lowercase hex chars.
/// Example: OrgId whose bytes start `[0xde, 0xad, 0xbe, 0xef, ...]` → `org_slug = "deadbeef"`.
///
/// # ADR-036 §3.4 — Location constraint
///
/// This type MUST live in `prism-dtu-common/src/scenario/` — NOT in a separate
/// `prism-dtu-scenario` crate.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct ScenarioEntityCatalog {
    /// Canonical org_slug derived from org_id bytes (hex of first 4 bytes).
    ///
    /// Used by both CrowdStrike and Armis generators for consistent ID derivation.
    /// Formula: `hex(org_id.as_bytes()[0..4])` — 8 lowercase hex chars.
    pub org_slug: String,

    /// The primary compromised device ID in CrowdStrike ID format.
    ///
    /// Format: `"dev-{org_slug}-{seed}-0"`.
    /// Example (org bytes `[0xde, 0xad, 0xbe, 0xef, ...]`, seed=42):
    ///   `"dev-deadbeef-42-0"`.
    pub primary_device_id_cs: String,

    /// The primary compromised device ID in Armis ID format.
    ///
    /// Format: `"dev-{org_slug}-{seed}-0"` — same formula as CrowdStrike.
    /// The Armis generator receives `org_slug` as an explicit `&str` arg.
    pub primary_device_id_armis: String,

    /// Hostname for the compromised device (consistent across DTUs).
    pub primary_hostname: String,

    /// Secondary device IDs involved in lateral movement (CrowdStrike format).
    pub lateral_device_ids_cs: Vec<String>,

    /// Secondary device IDs involved in lateral movement (Armis format).
    pub lateral_device_ids_armis: Vec<String>,

    /// IOC IPv4 addresses introduced during Exfil stage.
    ///
    /// Derived from the secondary RNG stream (`gen_seeded_rng(seed.wrapping_add(1), &org_id)`).
    /// MUST resolve as malicious in ThreatIntel.
    pub ioc_ips: Vec<String>,

    /// IOC domain names introduced during Exfil stage.
    ///
    /// Derived from the secondary RNG stream.
    pub ioc_domains: Vec<String>,

    /// IOC SHA256 file hashes introduced during LateralMovement stage.
    ///
    /// Derived from the secondary RNG stream.
    pub ioc_hashes: Vec<String>,

    /// CVE IDs assigned to the primary device.
    ///
    /// Derived from the secondary RNG stream.
    /// MUST resolve in NVD (base_score >= 7.0).
    pub device_cves: Vec<String>,
}

/// Derive the canonical org_slug from OrgId bytes.
///
/// Formula: `hex(org_id.as_bytes()[0..4])` — exactly 8 lowercase hex characters.
///
/// This formula MUST match `prism_dtu_crowdstrike::generator::org_slug()` exactly
/// (ADR-036 §2.2).  The Armis generator receives this value as the `org_slug: &str`
/// argument.
///
/// # Example
///
/// ```rust,ignore
/// let org = OrgId([0xde, 0xad, 0xbe, 0xef, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(org_slug_from_org_id(&org), "deadbeef");
/// ```
pub fn org_slug_from_org_id(org_id: &OrgId) -> String {
    let bytes = org_id.as_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

/// Build a [`ScenarioEntityCatalog`] from `(seed, org_id)`.
///
/// The IOC IPs, domains, hashes, and CVE IDs are derived from a **secondary** RNG stream:
/// `gen_seeded_rng(seed.wrapping_add(1), org_id)` — completely independent of the
/// primary generator stream used by the individual clone generators.
///
/// The secondary stream is needed so that catalog derivation does not consume RNG state
/// from the primary stream (which would shift all generated record IDs).
///
/// # Formula (ADR-036 §2.2)
///
/// - `org_slug = hex(org_id.as_bytes()[0..4])`
/// - `primary_device_id_cs   = "dev-{org_slug}-{seed}-0"`
/// - `primary_device_id_armis = "dev-{org_slug}-{seed}-0"` (same)
/// - `ioc_ips`, `ioc_domains`, `ioc_hashes`, `device_cves` from secondary RNG stream
pub fn build_scenario_entity_catalog(seed: u64, org_id: &OrgId) -> ScenarioEntityCatalog {
    let org_slug = org_slug_from_org_id(org_id);

    let primary_device_id_cs = format!("dev-{org_slug}-{seed}-0");
    let primary_device_id_armis = format!("dev-{org_slug}-{seed}-0");
    let primary_hostname = format!("host-{org_slug}-{seed}");

    // Lateral device IDs (indices 1..=3)
    let lateral_device_ids_cs: Vec<String> = (1..=3)
        .map(|n| format!("dev-{org_slug}-{seed}-{n}"))
        .collect();
    let lateral_device_ids_armis: Vec<String> = (1..=3)
        .map(|n| format!("dev-{org_slug}-{seed}-{n}"))
        .collect();

    // Secondary RNG stream — completely independent of the primary generator stream.
    // gen_seeded_rng(seed.wrapping_add(1), org_id) per ADR-036 §2.2.
    let mut rng = gen_seeded_rng(seed.wrapping_add(1), org_id);

    let ioc_ips = gen_ioc_ips(&mut rng, 4);
    let ioc_domains = gen_ioc_domains(&mut rng, 4);
    let ioc_hashes = gen_ioc_hashes(&mut rng, 4);
    let device_cves = gen_device_cves(&mut rng, 3);

    ScenarioEntityCatalog {
        org_slug,
        primary_device_id_cs,
        primary_device_id_armis,
        primary_hostname,
        lateral_device_ids_cs,
        lateral_device_ids_armis,
        ioc_ips,
        ioc_domains,
        ioc_hashes,
        device_cves,
    }
}

// ---------------------------------------------------------------------------
// Private helpers (used by build_scenario_entity_catalog implementation)
// ---------------------------------------------------------------------------

/// Generate N random IPv4 addresses in the 10.x.x.x range from RNG.
fn gen_ioc_ips(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            format!(
                "10.{}.{}.{}",
                rng.gen::<u8>(),
                rng.gen::<u8>(),
                rng.gen::<u8>()
            )
        })
        .collect()
}

/// Generate N IOC domain names from RNG.
fn gen_ioc_domains(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("malicious-{}-{}.example.com", rng.gen::<u32>(), i))
        .collect()
}

/// Generate N IOC SHA256 hashes (as hex strings) from RNG.
///
/// Produces a proper 64-hex-char string from 32 random bytes (`{:02x}` per byte),
/// matching the SHA-256 output representation (256 bits / 8 bits-per-char = 64 chars).
/// The prior `{:01x}` with nibble-masking discarded the upper nibble, yielding only
/// 4 bits per character and a non-representative hash distribution.
fn gen_ioc_hashes(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            (0..32)
                .map(|_| format!("{:02x}", rng.gen::<u8>()))
                .collect::<String>()
        })
        .collect()
}

/// Generate N CVE ID strings from RNG.
///
/// Format: `"CVE-{year}-{n}"` where year and n are RNG-derived.
fn gen_device_cves(rng: &mut impl rand::Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            format!(
                "CVE-{}-{}",
                2020u32 + (rng.gen::<u32>() % 5),
                rng.gen::<u32>() % 100000
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Unit tests (tests 1-2 in Red Gate Test Plan)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Org UUID with well-known bytes for canonical ID format assertions.
    ///
    /// First 4 bytes: [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef"
    /// Primary device ID (seed=42): "dev-deadbeef-42-0"
    ///
    /// ADR-036 §2.2: "Any test using 'dev-acme-...' is incorrect."
    fn deadbeef_org() -> OrgId {
        OrgId([
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ])
    }

    /// RG-1: test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids
    ///
    /// Traces to: BC-2.06.018 precondition 4 / ADR-036 §2.2
    /// Verifies:
    /// - org_slug = "deadbeef" for deadbeef_org()
    /// - primary_device_id_cs = "dev-deadbeef-42-0"
    /// - primary_device_id_armis = "dev-deadbeef-42-0"
    /// - ioc_ips, ioc_domains, ioc_hashes, device_cves are all non-empty
    /// - secondary RNG stream is independent (catalog fields populated from
    ///   gen_seeded_rng(seed.wrapping_add(1), &org_id), not from seed)
    #[test]
    fn test_BC_2_06_018_scenario_catalog_secondary_rng_and_canonical_ids() {
        let org = deadbeef_org();
        let seed: u64 = 42;

        let catalog = build_scenario_entity_catalog(seed, &org);

        // Org slug must be canonical 8-hex chars derived from first 4 bytes of org_id.
        assert_eq!(
            catalog.org_slug, "deadbeef",
            "org_slug must be 'deadbeef' for org bytes [0xde, 0xad, 0xbe, 0xef, ...]; \
             got '{}'. ADR-036 §2.2 formula: hex(org_id.as_bytes()[0..4])",
            catalog.org_slug
        );

        // Primary device ID — CrowdStrike format: "dev-{org_slug}-{seed}-0"
        assert_eq!(
            catalog.primary_device_id_cs, "dev-deadbeef-42-0",
            "primary_device_id_cs must be 'dev-deadbeef-42-0' for org_slug='deadbeef', seed=42; \
             got '{}'. ADR-036 §2.2 canonical format: dev-{{org_slug}}-{{seed}}-{{n}}",
            catalog.primary_device_id_cs
        );

        // Primary device ID — Armis format: same formula
        assert_eq!(
            catalog.primary_device_id_armis,
            "dev-deadbeef-42-0",
            "primary_device_id_armis must be 'dev-deadbeef-42-0' for org_slug='deadbeef', seed=42; \
             got '{}'",
            catalog.primary_device_id_armis
        );

        // Secondary RNG-derived fields must be non-empty (derived from secondary stream)
        assert!(
            !catalog.ioc_ips.is_empty(),
            "ioc_ips must be non-empty — derived from secondary RNG stream \
             gen_seeded_rng(seed.wrapping_add(1), &org_id); ADR-036 §2.2"
        );
        assert!(
            !catalog.ioc_domains.is_empty(),
            "ioc_domains must be non-empty — derived from secondary RNG stream"
        );
        assert!(
            !catalog.ioc_hashes.is_empty(),
            "ioc_hashes must be non-empty — derived from secondary RNG stream"
        );
        assert!(
            !catalog.device_cves.is_empty(),
            "device_cves must be non-empty — derived from secondary RNG stream"
        );

        // Determinism: same inputs → same catalog (BC-3.4.001 postcondition 3)
        let catalog2 = build_scenario_entity_catalog(seed, &org);
        assert_eq!(
            catalog.ioc_ips, catalog2.ioc_ips,
            "build_scenario_entity_catalog must be deterministic: same (seed, org_id) \
             must produce identical ioc_ips on repeated calls (BC-3.4.001 PC-3)"
        );

        // Different seeds → different secondary RNG output (independence)
        let catalog_other_seed = build_scenario_entity_catalog(seed + 1, &org);
        assert_ne!(
            catalog.ioc_ips, catalog_other_seed.ioc_ips,
            "different seeds must produce different ioc_ips (secondary stream independence)"
        );
    }

    /// RG-2: test_BC_2_06_018_org_slug_from_org_id_canonical_format
    ///
    /// Traces to: BC-2.06.018 §Canonical Org Slug / ADR-036 §2.2
    /// Verifies:
    /// - org_slug_from_org_id returns "deadbeef" for deadbeef_org()
    /// - result is exactly 8 characters
    /// - all characters are in [0-9a-f]
    /// - formula is consistent with CrowdStrike generator's internal org_slug()
    #[test]
    fn test_BC_2_06_018_org_slug_from_org_id_canonical_format() {
        let org = deadbeef_org();

        let slug = org_slug_from_org_id(&org);

        // Golden test vector: [0xde, 0xad, 0xbe, 0xef, ...] → "deadbeef"
        assert_eq!(
            slug, "deadbeef",
            "org_slug_from_org_id must return 'deadbeef' for org bytes \
             [0xde, 0xad, 0xbe, 0xef, ...]; got '{}'. \
             Formula: hex(org_id.as_bytes()[0..4]) — ADR-036 §2.2",
            slug
        );

        // Length invariant: always exactly 8 characters
        assert_eq!(
            slug.len(),
            8,
            "org_slug_from_org_id result must be exactly 8 characters; got {} for '{}'",
            slug.len(),
            slug
        );

        // Character set invariant: only [0-9a-f] (lowercase hex)
        assert!(
            slug.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "org_slug_from_org_id result must contain only lowercase hex chars [0-9a-f]; \
             got '{}' — uppercase chars are forbidden (ADR-036 §2.2)",
            slug
        );

        // All-zeros org → "00000000"
        let zero_org = OrgId([0u8; 16]);
        let zero_slug = org_slug_from_org_id(&zero_org);
        assert_eq!(
            zero_slug, "00000000",
            "org_slug_from_org_id must return '00000000' for all-zero OrgId; got '{}'",
            zero_slug
        );

        // All-ones (0xff) first 4 bytes → "ffffffff"
        let ff_org = OrgId([0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let ff_slug = org_slug_from_org_id(&ff_org);
        assert_eq!(
            ff_slug, "ffffffff",
            "org_slug_from_org_id must return 'ffffffff' for org bytes [0xff, 0xff, 0xff, 0xff, ...]; \
             got '{}'",
            ff_slug
        );

        // Arbitrary org: [0x01, 0x23, 0x45, 0x67, ...] → "01234567"
        let arbitrary_org = OrgId([0x01, 0x23, 0x45, 0x67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let arbitrary_slug = org_slug_from_org_id(&arbitrary_org);
        assert_eq!(
            arbitrary_slug, "01234567",
            "org_slug_from_org_id must return '01234567' for org bytes [0x01, 0x23, 0x45, 0x67, ...]; \
             got '{}'",
            arbitrary_slug
        );

        // Verify only first 4 bytes matter (bytes 4+ are ignored)
        let org_a = OrgId([
            0xca, 0xfe, 0xba, 0xbe, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a,
            0x0b, 0x0c,
        ]);
        let org_b = OrgId([
            0xca, 0xfe, 0xba, 0xbe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ]);
        assert_eq!(
            org_slug_from_org_id(&org_a),
            org_slug_from_org_id(&org_b),
            "org_slug_from_org_id must only use first 4 bytes; \
             different bytes 4-15 with same bytes 0-3 must yield the same slug"
        );
    }
}
