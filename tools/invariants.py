"""Checks the rules of dev/PLAN.md that no compiler or lint enforces.

    python3 tools/invariants.py

Each check prints one line; the script exits non-zero if any fails.

- NFR-1: no runtime dependency in any feature mode (`cargo tree`).
- NFR-8: `unsafe` only in `src/facade/slot.rs`, at most 40 lines, each with a
  `SAFETY:` comment just above it.
- 7.2: `code`, `severity`, `category`, and `kinds` use nothing from `facade`,
  `record`, or `macros`, so they can move to their own crate (OD-6).
- NFR-12: every `compile_error!` message of `define_errors!` is checked by a
  compile-fail case in `tests/ui`, except a guard no input reaches.
- NFR-14: lines of code in `src` (no blank lines, comments, or unit tests)
  stay under 6,000, the point where OD-6 is reopened.
"""

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
failed = False


def report(ok, message):
    global failed
    failed |= not ok
    print(("ok    " if ok else "FAIL  ") + message)


def code_lines(text):
    """The lines of a file that are code, with a trailing unit-test module cut."""
    lines = text.split("\n")
    for i, line in enumerate(lines[:-1]):
        if line.strip() == "#[cfg(test)]" and lines[i + 1].lstrip().startswith("mod "):
            lines = lines[:i]
            break
    return [line for line in lines if line.strip() and not line.strip().startswith("//")]


def check_dependencies():
    modes = [[], ["--all-features"], ["--no-default-features"], ["--no-default-features", "--features", "alloc"]]
    for mode in modes:
        out = subprocess.run(
            ["cargo", "tree", "--edges", "normal", "--prefix", "none", *mode],
            cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout.split()
        crates = sorted({word for word in out if not word.startswith(("v", "(")) and word != "*"})
        report(crates == ["define-err"], f"no runtime dependency ({' '.join(mode) or 'default features'}): {crates}")


def check_unsafe():
    slot = SRC / "facade" / "slot.rs"
    others = []
    for path in sorted(SRC.rglob("*.rs")):
        if path != slot and any(re.search(r"\bunsafe\b", line) for line in code_lines(path.read_text(encoding="utf-8"))):
            others.append(path.relative_to(ROOT).as_posix())
    report(not others, f"unsafe only in src/facade/slot.rs (also found in: {others or 'none'})")

    lines = slot.read_text(encoding="utf-8").split("\n")
    sites = [i for i, line in enumerate(lines) if re.search(r"\bunsafe\b", line) and not line.strip().startswith("//")]
    unexplained = [i + 1 for i in sites if not any("SAFETY:" in lines[j] for j in range(max(0, i - 4), i))]
    report(len(sites) <= 40, f"unsafe lines in slot.rs: {len(sites)} (limit 40)")
    report(not unexplained, f"every unsafe line has a SAFETY: comment above it (missing at lines: {unexplained or 'none'})")


def check_boundary():
    bad = []
    for name in ("code", "severity", "category", "kinds"):
        for path in [SRC / f"{name}.rs", *sorted((SRC / name).rglob("*.rs"))]:
            if not path.exists():
                continue
            for number, line in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
                if re.search(r"crate::(facade|record|macros)\b", line):
                    bad.append(f"{path.relative_to(ROOT).as_posix()}:{number}")
    report(not bad, f"codes, severity, category, kinds use nothing from facade, record, macros ({bad or 'clean'})")


# The emitter's last arm: reached only if the parser passes on a field it
# should have rejected, so no input reaches it today. Kept as a guard.
UNREACHABLE = {
    "define_errors!: a field must be `name: Type` with at most one of `#[from]`, `#[source]`, `#[location]`, `#[retry_after]` before it",
}


def check_ui_coverage():
    messages = set()
    for path in sorted((SRC / "macros").rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        for match in re.finditer(r'compile_error!\s*\{\s*"((?:[^"\\]|\\.)*)"', text):
            messages.add(match.group(1).replace('\\"', '"').replace("\\\\", "\\"))
    expected = "\n".join(path.read_text(encoding="utf-8") for path in (ROOT / "tests" / "ui").glob("*.stderr"))
    messages -= UNREACHABLE
    missing = sorted(message for message in messages if message not in expected)
    report(not missing, f"{len(messages) - len(missing)} of {len(messages)} compile_error! messages have a tests/ui case")
    for message in missing:
        print(f"      no case: {message}")


def check_size():
    total = sum(len(code_lines(path.read_text(encoding="utf-8"))) for path in SRC.rglob("*.rs"))
    report(total < 6000, f"lines of code in src: {total} (target under 5,000; OD-6 reopens at 6,000)")


check_dependencies()
check_unsafe()
check_boundary()
check_ui_coverage()
check_size()
sys.exit(1 if failed else 0)
