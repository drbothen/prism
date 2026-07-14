#!/usr/bin/env python3
"""
hash-plugin-source.py <source-dir>

Compute a deterministic SHA-256 over all git-tracked files in <source-dir>.
Used by the CI staleness gate for committed .prx plugin artifacts.

Algorithm: git ls-files | sort | hash(path + content) per file.
Platform-neutral: identical output on macOS, Linux, Windows for same file content.
"""
import hashlib
import pathlib
import subprocess
import sys


def main() -> None:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <source-dir>", file=sys.stderr)
        sys.exit(1)

    source_dir = sys.argv[1]

    # Resolve the git repository root so that path lookups are always anchored
    # to the repo root regardless of the directory from which this script is
    # invoked.  Without this, git ls-files returns repo-root-relative paths
    # but pathlib.Path(rel_path).is_file() resolves them against CWD, causing
    # every is_file() call to return False when CWD != repo root — the script
    # would then silently print sha256("") (e3b0c442...) instead of failing.
    toplevel_result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
    )
    if toplevel_result.returncode != 0:
        print(
            f"error: git rev-parse --show-toplevel failed: {toplevel_result.stderr}",
            file=sys.stderr,
        )
        sys.exit(1)
    repo_root = pathlib.Path(toplevel_result.stdout.strip())

    # Run git ls-files from the repo root so output paths are consistently
    # repo-root-relative on every invocation.
    result = subprocess.run(
        ["git", "ls-files", "--", f"{source_dir}/"],
        capture_output=True,
        text=True,
        cwd=repo_root,
    )
    if result.returncode != 0:
        print(f"error: git ls-files failed: {result.stderr}", file=sys.stderr)
        sys.exit(1)

    tracked = sorted(line for line in result.stdout.splitlines() if line)
    if not tracked:
        print(f"error: no tracked files found under {source_dir}/", file=sys.stderr)
        sys.exit(1)

    h = hashlib.sha256()
    files_hashed = 0
    for rel_path in tracked:
        p = repo_root / rel_path  # anchored to repo root, not CWD
        if not p.is_file():
            continue
        h.update(rel_path.encode("utf-8") + b"\n")
        h.update(p.read_bytes())
        files_hashed += 1

    # Fail loud: if git ls-files reported tracked files but none could be read,
    # path resolution has gone wrong.  Printing sha256("") would silently pass
    # the CI staleness gate with an incorrect hash.
    if files_hashed == 0:
        print(
            f"error: git ls-files listed {len(tracked)} tracked file(s) under "
            f"{source_dir}/ but none could be read — path resolution failure "
            f"(repo_root={repo_root}, cwd={pathlib.Path.cwd()})",
            file=sys.stderr,
        )
        sys.exit(1)

    print(h.hexdigest())


if __name__ == "__main__":
    main()
