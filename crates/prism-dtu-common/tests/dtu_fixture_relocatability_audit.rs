#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Audit gate for #285: DTU clones must not read fixtures from `CARGO_MANIFEST_DIR`
//! at runtime.
//!
//! `load_fixture` / `load_fixture_as` join their `crate_dir` argument with
//! `fixtures/<name>.json` and read it when called. Every caller passes
//! `env!("CARGO_MANIFEST_DIR")`, which expands at compile time to an absolute path
//! on the build machine, so a clone built anywhere else fails: at construction for
//! the crates that load in their constructor, and per request for the crates that
//! load inside route handlers, which is why health checks pass first.
//!
//! Nothing else catches a regression here. The whole test suite runs inside the
//! source tree, where the baked-in path still resolves, so reverting a call site to
//! `load_fixture` is invisible to every other test. This walks the sources instead,
//! which is the same compensating-control shape as
//! `prism-core/tests/new_unchecked_audit.rs`.
//!
//! Production code embeds fixtures with `include_str!` and parses them through
//! `embedded_fixture` / `embedded_fixture_as`. Test code may keep reading from disk:
//! tests always run in the source tree.

use std::path::{Path, PathBuf};

/// Sites permitted to call the runtime loaders outside `#[cfg(test)]`.
///
/// Format: (crate directory name, file suffix within `src/`). Adding an entry needs
/// a justification comment saying why that site cannot embed its fixture.
const ALLOWLISTED_RUNTIME_LOADERS: &[(&str, &str)] = &[];

/// Blanks out `#[cfg(test)] mod ... { ... }` spans, keeping line numbering intact.
///
/// Brace counting is line-based and does not track braces inside string literals, so
/// a `{` in a test-module string could end the span early. That direction is safe:
/// it can only make the audit scan more lines, never fewer.
fn strip_cfg_test_modules(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim_start().starts_with("#[cfg(test)]") {
            let mut j = i + 1;
            while j < lines.len() && j <= i + 3 && !lines[j].contains("mod ") {
                j += 1;
            }
            if j < lines.len() && lines[j].contains("mod ") {
                let mut depth: i32 = 0;
                let mut opened = false;
                let mut k = j;
                while k < lines.len() {
                    depth += i32::try_from(lines[k].matches('{').count()).unwrap_or(0);
                    depth -= i32::try_from(lines[k].matches('}').count()).unwrap_or(0);
                    if lines[k].contains('{') {
                        opened = true;
                    }
                    if opened && depth <= 0 {
                        break;
                    }
                    k += 1;
                }
                let end = k.min(lines.len().saturating_sub(1));
                out.extend(std::iter::repeat_n("", end - i + 1));
                i = end + 1;
                continue;
            }
        }
        out.push(lines[i]);
        i += 1;
    }
    out.join("\n")
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn dtu_clones_do_not_read_fixtures_at_runtime() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo when running tests");
    let crates_dir = Path::new(&manifest_dir)
        .parent()
        .expect("prism-dtu-common must live under crates/")
        .to_path_buf();

    let mut dtu_crates: Vec<PathBuf> = std::fs::read_dir(&crates_dir)
        .expect("crates/ must be readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("prism-dtu-") && n != "prism-dtu-common")
        })
        .collect();
    dtu_crates.sort();

    assert!(
        !dtu_crates.is_empty(),
        "found no prism-dtu-* crates to audit under {}",
        crates_dir.display()
    );

    let mut violations: Vec<String> = Vec::new();

    for crate_dir in &dtu_crates {
        let crate_name = crate_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_owned();
        let src = crate_dir.join("src");
        if !src.exists() {
            continue;
        }

        let mut files = Vec::new();
        walk_rs_files(&src, &mut files);
        files.sort();

        for file in files {
            let Ok(raw) = std::fs::read_to_string(&file) else {
                continue;
            };
            let relative = file
                .strip_prefix(&src)
                .unwrap_or(&file)
                .to_string_lossy()
                .to_string();

            if ALLOWLISTED_RUNTIME_LOADERS
                .iter()
                .any(|(c, f)| *c == crate_name && relative.ends_with(f))
            {
                continue;
            }

            for (idx, line) in strip_cfg_test_modules(&raw).lines().enumerate() {
                let trimmed = line.trim_start();
                // Doc comments and ordinary comments legitimately name the loaders
                // when explaining why a site embeds instead.
                if trimmed.starts_with("//") {
                    continue;
                }
                if line.contains("load_fixture(") || line.contains("load_fixture_as(") {
                    violations.push(format!(
                        "{crate_name}/src/{relative}:{} calls a runtime fixture loader outside #[cfg(test)]:\n    {}",
                        idx + 1,
                        trimmed
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "#285: DTU production code must embed fixtures with include_str! and parse them \
         via prism_dtu_common::embedded_fixture / embedded_fixture_as. A clone that reads \
         CARGO_MANIFEST_DIR at runtime cannot run on a host that did not build it.\n\n{}\n\n\
         If a site genuinely cannot embed, add (\"<crate>\", \"<file suffix>\") to \
         ALLOWLISTED_RUNTIME_LOADERS with a justification comment.",
        violations.join("\n\n")
    );
}
