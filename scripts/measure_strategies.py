#!/usr/bin/env python3
"""Measure what a per-family feature split of optionstratlib-strategies could save (#532).

Prints, for the strategies crate and for analytics (the layer it sits on):

* the number of packages `cargo tree -e normal` resolves, so the difference is
  what strategies adds to a consumer's graph;
* the crate's own non-incremental check, debug build and release build time
  with every dependency already built (touch `src/lib.rs`, median of three),
  which bounds what any family feature could save;
* a clean check of analytics and everything below it, for scale;
* the size of each component's release `rlib`.

It builds in a target directory of its own under `target/measure-strategies`
and removes it at the end. The numbers recorded in the crate docs came from
this script; rerun it after a change that could move them.

Usage: scripts/measure_strategies.py
"""

from __future__ import annotations

import os
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target" / "measure-strategies"
ENV = {**os.environ, "CARGO_TARGET_DIR": str(TARGET), "CARGO_INCREMENTAL": "0"}
RUNS = 3


def cargo(*args: str) -> str:
    result = subprocess.run(
        ["cargo", *args], cwd=ROOT, env=ENV, capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        raise SystemExit(f"cargo {' '.join(args)} failed")
    return result.stdout


def timed(*args: str) -> float:
    start = time.monotonic()
    cargo(*args)
    return time.monotonic() - start


def own_time(crate: str, *args: str) -> float:
    """Median time to rebuild `crate` alone, its dependencies already built."""
    lib = ROOT / "crates" / crate / "src" / "lib.rs"
    cargo(*args, "-p", crate)
    samples = []
    for _ in range(RUNS):
        lib.touch()
        samples.append(timed(*args, "-p", crate))
    return statistics.median(samples)


def packages(crate: str) -> int:
    out = cargo("tree", "-q", "-p", crate, "-e", "normal", "--prefix", "none")
    return len({line.removesuffix(" (*)").strip() for line in out.splitlines() if line.strip()})


def main() -> int:
    shutil.rmtree(TARGET, ignore_errors=True)
    try:
        print(cargo("--version").strip(), "|", subprocess.run(
            ["rustc", "--version"], capture_output=True, text=True, check=False
        ).stdout.strip())
        for crate in ("optionstratlib-analytics", "optionstratlib-strategies"):
            print(f"{crate}: {packages(crate)} packages (cargo tree -e normal)")
        print(f"clean check of optionstratlib-analytics and below: "
              f"{timed('check', '-q', '-p', 'optionstratlib-analytics'):.1f} s")
        for label, args in (("check", ("check", "-q")), ("debug build", ("build", "-q")),
                            ("release build", ("build", "-q", "--release"))):
            seconds = own_time("optionstratlib-strategies", *args)
            print(f"optionstratlib-strategies own {label} (median of {RUNS}): {seconds:.2f} s")
        for rlib in sorted((TARGET / "release" / "deps").glob("liboptionstratlib_*.rlib")):
            name = rlib.name.rsplit("-", 1)[0].removeprefix("lib")
            print(f"{name} release rlib: {rlib.stat().st_size / 1e6:.1f} MB")
    finally:
        shutil.rmtree(TARGET, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
