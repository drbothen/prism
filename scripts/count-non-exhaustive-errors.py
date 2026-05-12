#!/usr/bin/env python3
"""Count E0639 and E0004 errors in cargo --message-format=json output.

Usage: count-non-exhaustive-errors.py <json-log-file>

Used by check-non-exhaustive.sh to count #[non_exhaustive] violations
across all submodules without hitting rustc's per-file error limit.
(BC-2.01.013 AC-5 / F-LP2-OBS-001 S-PLUGIN-PREREQ-C)
"""
import json
import sys


def main():
    if len(sys.argv) < 2:
        print("Usage: count-non-exhaustive-errors.py <json-log-file>", file=sys.stderr)
        sys.exit(1)

    log_path = sys.argv[1]
    count = 0

    try:
        with open(log_path) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    m = json.loads(line)
                    if m.get("reason") == "compiler-message":
                        msg = m.get("message") or {}
                        code = msg.get("code") or {}
                        c = code.get("code", "") if isinstance(code, dict) else ""
                        if c in ("E0639", "E0004"):
                            count += 1
                except (json.JSONDecodeError, AttributeError):
                    pass
    except FileNotFoundError:
        print(f"Error: log file not found: {log_path}", file=sys.stderr)
        sys.exit(1)

    print(count)


if __name__ == "__main__":
    main()
