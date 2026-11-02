#!/usr/bin/env python3
"""Compares a run of `benches/floor.rs` with the committed baseline.

A hot path more than 5% over its baseline fails (dev/PLAN.md 11.2). Under a
quarter of a nanosecond is below what the floor can resolve, so a change that
small passes whatever its percentage. The baseline belongs to one machine: see
docs/PERFORMANCE.md before comparing on another.

    cargo bench --bench floor --all-features > floor.txt
    python tools/bench-compare.py floor.txt
"""

import pathlib
import re
import sys

BASELINE = pathlib.Path(__file__).resolve().parents[1] / "benches" / "baseline.txt"
ROW = re.compile(r"^(\w+)\s+([0-9.]+) ns$")


def read(path):
    rows = (ROW.match(line.strip()) for line in path.read_text(encoding="utf-8").splitlines())
    return {row[1]: float(row[2]) for row in rows if row}


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    base = read(BASELINE)
    new = read(pathlib.Path(sys.argv[1]))
    failed = 0
    for name, before in base.items():
        after = new.get(name)
        if after is None:
            print(f"MISSING    {name}")
            failed += 1
            continue
        slower = after > before * 1.05 and after - before > 0.25
        failed += slower
        print(f"{'SLOWER' if slower else 'ok':<10} {name:<30} {before:>8.2f} {after:>8.2f} ns")
    if failed:
        raise SystemExit(f"{failed} over the limit")
    print("NO REGRESSION")


if __name__ == "__main__":
    main()
