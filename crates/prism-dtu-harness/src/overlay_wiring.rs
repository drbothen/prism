//! Overlay TOML wiring helper for multi-tenant DTU test harnesses.
//!
//! Writes per-org sensor overlay TOML files to a caller-supplied directory,
//! suitable for consumption by `SpecLoader::load_all` from `prism-spec-engine`.
//!
//! # Story anchor
//!
//! S-DEMO-MULTI-TENANT-DTU-001 (BC-2.06.017 Postcondition 3)
//!
//! # Perimeter constraint (BC-2.06.017 INV-PERIMETER-001)
//!
//! This module writes raw TOML strings only. It does NOT import
//! `prism-spec-engine`, `prism-sensors`, or `prism-query` types.
//! The `tempfile` crate is a `[dev-dependency]` only — the caller (test code)
//! owns the `TempDir` and passes `dir.path()` to this function (U-005).
//!
//! # Output layout
//!
//! For each `(org_slug, sensor_id)` entry in the harness socket map, writes:
//!
//! ```text
//! {dir}/customers/{org_slug}/{sensor_id}.sensor.toml
//! ```
//!
//! with content:
//!
//! ```toml
//! extends = "{sensor_id}"
//! instance_id = "{sensor_id}@{org_slug}"
//! base_url = "http://{socket_addr}"
//! ```
//!
//! This format is consumed by `SpecLoader::load_all` overlay walk semantics
//! (S-CONFIG-MULTI-TENANT-OVERRIDE-001 / BC-2.06.012).

use crate::multi_instance::MultiInstanceHarness;

/// Write per-org sensor overlay TOML files from a `MultiInstanceHarness` socket map.
///
/// For each `(org_slug, sensor_id)` → `SocketAddr` entry in
/// `harness.socket_map()`, creates the directory
/// `{dir}/customers/{org_slug}/` and writes the file
/// `{sensor_id}.sensor.toml` containing:
///
/// ```toml
/// extends = "{sensor_id}"
/// instance_id = "{sensor_id}@{org_slug}"
/// base_url = "http://{socket_addr}"
/// ```
///
/// The caller is responsible for creating and owning the `TempDir`; this
/// function receives only the `&Path` (U-005: no `tempfile` import in `src/`).
///
/// Returns `Ok(())` once all overlay files are written, or the first
/// `std::io::Error` encountered during directory creation or file write.
///
/// (BC-2.06.017 Postcondition 3 — after this function + `SpecLoader::load_all`,
/// `ResolvedSensorSpec` entries for each org carry the correct distinct `base_url`)
pub fn write_overlay_temp_dir(
    harness: &MultiInstanceHarness,
    dir: &std::path::Path,
) -> std::io::Result<()> {
    // For each (org_slug, sensor_id) → SocketAddr in the harness socket_map,
    // write:  {dir}/customers/{org_slug}/{sensor_id}.sensor.toml
    // with content (raw TOML string — INV-PERIMETER-001: no spec-engine imports here):
    //   extends = "{sensor_id}"
    //   instance_id = "{sensor_id}@{org_slug}"
    //   base_url = "http://{socket_addr}"
    //
    // `extends` links this overlay to the TYPE spec (required by OverlayLoader).
    // `instance_id` must equal `{sensor_id}@{org_slug}` (INV-SCALAR-003).
    // `base_url` routes this org's sensor queries to the correct DTU instance.
    //
    // (BC-2.06.017 Postcondition 3 / BC-2.06.012)
    for ((org_slug, sensor_id), socket_addr) in harness.socket_map() {
        // Create the per-org customer directory.
        let customer_dir = dir.join("customers").join(org_slug);
        std::fs::create_dir_all(&customer_dir)?;

        // Write the overlay TOML file.
        let toml_path = customer_dir.join(format!("{sensor_id}.sensor.toml"));
        let toml_content = format!(
            "extends = \"{sensor_id}\"\n\
             instance_id = \"{sensor_id}@{org_slug}\"\n\
             base_url = \"http://{socket_addr}\"\n"
        );
        std::fs::write(&toml_path, toml_content)?;
    }
    Ok(())
}
