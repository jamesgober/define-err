#!/usr/bin/env python3
"""Times builds of the same error types written with define-err and thiserror.

Generates two crates, 20 enums of 8 variants each, plus two with one enum of
500 variants, then measures with cargo (dev/PLAN.md 11.1):

  clean        clean build of the 20-enum crate and its dependencies
  incremental  rebuild after editing one message
  large        rebuild of the 500-variant crate alone

Reports the median of --runs runs. Needs only cargo and Python 3.8. Results:
docs/PERFORMANCE.md.

    python tools/build-time/compare.py --runs 10
"""

import argparse
import json
import os
import pathlib
import platform
import shutil
import statistics
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
FLAVOURS = ("define_err", "thiserror")


def define_err_enum(name, index):
    return f"""
define_errors! {{
    pub enum {name} {{
        NotFound {{ id: u64 }} => "item {{id}} not found in table {index}",
        Denied {{ user: String, action: String }} => "{{user}} may not {{action}}",
        Short(page: u64, got: usize) => "short read on page {{page}}: got {{got}} bytes",
        Parse {{ #[source] source: core::num::ParseIntError, line: u32 }} => "line {{line}} did not parse",
        Io(#[from] source: std::io::Error) => "file i/o failed",
        Busy => "the table is busy",
        Closed => "the table is closed",
        Other {{ reason: String }} => "{{reason}}",
    }}
}}
"""


def thiserror_enum(name, index):
    return f"""
#[derive(Debug, Error)]
pub enum {name} {{
    #[error("item {{id}} not found in table {index}")]
    NotFound {{ id: u64 }},
    #[error("{{user}} may not {{action}}")]
    Denied {{ user: String, action: String }},
    #[error("short read on page {{0}}: got {{1}} bytes")]
    Short(u64, usize),
    #[error("line {{line}} did not parse")]
    Parse {{ #[source] source: core::num::ParseIntError, line: u32 }},
    #[error("file i/o failed")]
    Io(#[from] std::io::Error),
    #[error("the table is busy")]
    Busy,
    #[error("the table is closed")]
    Closed,
    #[error("{{reason}}")]
    Other {{ reason: String }},
}}
"""


def large_enum(flavour):
    if flavour == "define_err":
        body = "".join(
            f'        V{i} {{ id: u64 }} => "variant {i} failed for {{id}}",\n' for i in range(500)
        )
        return f"define_errors! {{\n    pub enum Large {{\n{body}    }}\n}}\n"
    body = "".join(
        f'    #[error("variant {i} failed for {{id}}")]\n    V{i} {{ id: u64 }},\n' for i in range(500)
    )
    return f"#[derive(Debug, Error)]\npub enum Large {{\n{body}}}\n"


def write_crate(path, flavour, source):
    (path / "src").mkdir(parents=True)
    dependency = (
        f"define-err = {{ path = {json.dumps(ROOT.as_posix())} }}"
        if flavour == "define_err"
        else 'thiserror = "2"'
    )
    (path / "Cargo.toml").write_text(
        f'[package]\nname = "{path.name}"\nversion = "0.0.0"\nedition = "2021"\n'
        f"\n[dependencies]\n{dependency}\n\n[workspace]\n",
        encoding="utf-8",
    )
    use = "use define_err::define_errors;\n" if flavour == "define_err" else "use thiserror::Error;\n"
    (path / "src" / "lib.rs").write_text(use + source, encoding="utf-8")


def edit(path, text):
    """Writes a source file with a modification time ahead of the last build,
    so a write inside the file system's timestamp granularity still counts."""
    path.write_text(text, encoding="utf-8")
    ahead = time.time() + 5
    os.utime(path, (ahead, ahead))


def cargo_build(path, target):
    """Times one build and refuses a build that compiled nothing: a fresh
    fingerprint would otherwise pass for a very fast rebuild."""
    start = time.perf_counter()
    done = subprocess.run(
        ["cargo", "build", "--offline", "--manifest-path", str(path / "Cargo.toml")],
        check=True,
        capture_output=True,
        text=True,
        env={**os.environ, "CARGO_TARGET_DIR": str(target)},
    )
    elapsed = time.perf_counter() - start
    if f"Compiling {path.name}" not in done.stderr:
        raise SystemExit(f"cargo did not rebuild {path.name}:\n{done.stderr}")
    return elapsed


def measure(work, flavour, runs):
    twenty = work / f"twenty_{flavour}"
    large = work / f"large_{flavour}"
    target = work / f"target_{flavour}"
    emit = define_err_enum if flavour == "define_err" else thiserror_enum
    write_crate(twenty, flavour, "".join(emit(f"Error{i:02}", i) for i in range(20)))
    write_crate(large, flavour, large_enum(flavour))
    for crate in (twenty, large):
        subprocess.run(
            ["cargo", "generate-lockfile", "--manifest-path", str(crate / "Cargo.toml")], check=True
        )
        subprocess.run(["cargo", "fetch", "--manifest-path", str(crate / "Cargo.toml")], check=True)

    clean = []
    for _ in range(runs):
        shutil.rmtree(target, ignore_errors=True)
        clean.append(cargo_build(twenty, target))

    lib = twenty / "src" / "lib.rs"
    text = lib.read_text(encoding="utf-8")
    incremental = []
    for run in range(runs):
        edit(lib, text.replace("the table is busy", f"the table is busy, attempt {run}", 1))
        incremental.append(cargo_build(twenty, target))

    _ = cargo_build(large, target)
    lib = large / "src" / "lib.rs"
    text = lib.read_text(encoding="utf-8")
    big = []
    for run in range(runs):
        edit(lib, text.replace("variant 7 failed", f"variant 7, run {run}, failed", 1))
        big.append(cargo_build(large, target))

    return {
        "clean": statistics.median(clean),
        "incremental": statistics.median(incremental),
        "large": statistics.median(big),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--runs", type=int, default=10)
    parser.add_argument("--work", help="working directory; a temporary one by default")
    args = parser.parse_args()

    work = pathlib.Path(args.work) if args.work else pathlib.Path(tempfile.mkdtemp(prefix="build-time-"))
    work.mkdir(parents=True, exist_ok=True)
    version = subprocess.run(["rustc", "-V"], capture_output=True, text=True, check=True).stdout.strip()
    print(f"{version}; {platform.platform()}; {os.cpu_count()} threads; {args.runs} runs")
    try:
        # Every step runs both flavours, so neither inherits a warmer disk.
        results = {flavour: measure(work, flavour, args.runs) for flavour in FLAVOURS}
    finally:
        if not args.work:
            shutil.rmtree(work, ignore_errors=True)
    print(f"{'median seconds':<14}{'define_err':>12}{'thiserror':>12}{'change':>10}")
    for step in ("clean", "incremental", "large"):
        ours, theirs = results["define_err"][step], results["thiserror"][step]
        print(f"{step:<14}{ours:>12.3f}{theirs:>12.3f}{(ours / theirs - 1) * 100:>+9.1f}%")


if __name__ == "__main__":
    main()
