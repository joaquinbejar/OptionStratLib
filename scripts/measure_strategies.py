#!/usr/bin/env python3
"""Measure what a per-family feature split of optionstratlib-strategies could save (#532).

Prints, for the strategies crate and for analytics (the layer it sits on):

* the number of packages `cargo tree -e normal` resolves, so the difference is
  what strategies adds to a consumer's graph;
* the crate's own non-incremental check, debug build and release build time
  with every dependency already built (touch `src/lib.rs`, three samples),
  which bounds what any family feature could save. This metric is added on
  top of the M0-01 method (doc/BASELINE.md), which times incremental and
  clean builds of the whole graph;
* a clean check of analytics and everything below it, three samples after a
  `cargo fetch`, for scale (the M0-01 clean-check method);
* the size of each component's release `rlib`.

Every timing prints its median and range. It builds in a target directory of
its own under `target/measure-strategies`, with Cargo's build directory
pointed there as well so a `build.build-dir` in the user's Cargo config
cannot serve cached artifacts, and removes it at the end. The numbers
recorded in the crate docs came from this script; rerun it after a change
that could move them.

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
ENV = {
    **os.environ,
    "CARGO_TARGET_DIR": str(TARGET),
    "CARGO_BUILD_BUILD_DIR": str(TARGET),
    "CARGO_INCREMENTAL": "0",
}
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


def summary(samples: list[float]) -> str:
    return f"median {statistics.median(samples):.2f} s (range {min(samples):.2f} to {max(samples):.2f}, {len(samples)} runs)"


def own_times(crate: str, *args: str) -> list[float]:
    """Times to rebuild `crate` alone, its dependencies already built."""
    lib = ROOT / "crates" / crate / "src" / "lib.rs"
    cargo(*args, "-p", crate)
    samples = []
    for _ in range(RUNS):
        lib.touch()
        samples.append(timed(*args, "-p", crate))
    return samples


def clean_check_times(crate: str) -> list[float]:
    """Clean `cargo check` of `crate` and its graph, after a `cargo fetch`."""
    cargo("fetch", "-q")
    samples = []
    for _ in range(RUNS):
        shutil.rmtree(TARGET, ignore_errors=True)
        samples.append(timed("check", "-q", "-p", crate))
    return samples


def hardware() -> str:
    probes = (["sysctl", "-n", "machdep.cpu.brand_string"], ["sysctl", "-n", "hw.ncpu"])
    found = []
    for probe in probes:
        try:
            out = subprocess.run(probe, capture_output=True, text=True, check=False).stdout.strip()
        except OSError:
            out = ""
        if out:
            found.append(out)
    return ", ".join(found) or f"{os.cpu_count()} logical CPUs"


def packages(crate: str) -> int:
    out = cargo("tree", "-q", "-p", crate, "-e", "normal", "--prefix", "none")
    return len({line.removesuffix(" (*)").strip() for line in out.splitlines() if line.strip()})


def main() -> int:
    shutil.rmtree(TARGET, ignore_errors=True)
    try:
        print(cargo("--version").strip(), "|", subprocess.run(
            ["rustc", "--version"], capture_output=True, text=True, check=False
        ).stdout.strip(), "|", hardware())
        print("commit", subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, capture_output=True, text=True,
            check=False,
        ).stdout.strip())
        for crate in ("optionstratlib-analytics", "optionstratlib-strategies"):
            print(f"{crate}: {packages(crate)} packages (cargo tree -e normal)")
        print("clean check of optionstratlib-analytics and below:",
              summary(clean_check_times("optionstratlib-analytics")))
        for label, args in (("check", ("check", "-q")), ("debug build", ("build", "-q")),
                            ("release build", ("build", "-q", "--release"))):
            samples = own_times("optionstratlib-strategies", *args)
            print(f"optionstratlib-strategies own {label}:", summary(samples))
        rlibs = sorted((TARGET / "release" / "deps").glob("liboptionstratlib_*.rlib"))
        if not rlibs:
            raise SystemExit(f"no release rlib under {TARGET / 'release' / 'deps'}")
        for rlib in rlibs:
            name = rlib.name.rsplit("-", 1)[0].removeprefix("lib")
            print(f"{name} release rlib: {rlib.stat().st_size / 1e6:.1f} MB")
    finally:
        shutil.rmtree(TARGET, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
