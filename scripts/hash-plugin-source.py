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

    result = subprocess.run(
        ["git", "ls-files", "--", f"{source_dir}/"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"error: git ls-files failed: {result.stderr}", file=sys.stderr)
        sys.exit(1)

    tracked = sorted(line for line in result.stdout.splitlines() if line)
    if not tracked:
        print(f"error: no tracked files found under {source_dir}/", file=sys.stderr)
        sys.exit(1)

    h = hashlib.sha256()
    for rel_path in tracked:
        p = pathlib.Path(rel_path)
        if not p.is_file():
            continue
        h.update(rel_path.encode("utf-8") + b"\n")
        h.update(p.read_bytes())

    print(h.hexdigest())


if __name__ == "__main__":
    main()
