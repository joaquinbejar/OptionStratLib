#!/usr/bin/env python3
"""Prove the library builds and tests with the wall clock off (#944).

With the `clock` feature off, no package in the build may enable `chrono/clock`,
`expiration_date/clock` or `option_type/clock`, so `Utc::now` does not exist. For
every configuration below this script asks `cargo tree -e features -i chrono`,
the acceptance check of #944, over all targets and with dev-dependencies, because
features unify: a dev-dependency that enables the clock would make the test run
prove nothing. Each crate also has a `#[cfg(not(feature = "clock"))]` test that
fails if the clock turns out to be on (`tests/no_clock.rs`).

The default build must keep the clock, so that is asserted too.

Usage:
    check_no_clock.py               assert the trees, then lint, build and test
    check_no_clock.py --trees       assert the trees only
    check_no_clock.py --self-test   prove the tree assertion
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

FACADE = "optionstratlib"
COMPONENTS = [
    "optionstratlib-core",
    "optionstratlib-math",
    "optionstratlib-pricing",
    "optionstratlib-simulation",
    "optionstratlib-market",
    "optionstratlib-analytics",
    "optionstratlib-strategies",
    "optionstratlib-backtest",
    "optionstratlib-visualization",
]

# Every optional feature of a component, on top of the bare crate.
COMPONENT_FEATURES = {
    "optionstratlib-core": "schema",
    "optionstratlib-math": "schema",
    "optionstratlib-pricing": "schema",
    "optionstratlib-simulation": "schema",
    "optionstratlib-market": "io,async,synthetic,schema",
    "optionstratlib-analytics": "schema",
    "optionstratlib-strategies": "schema",
    "optionstratlib-backtest": "schema",
    "optionstratlib-visualization": "plotly,static_export",
}

# The facade with no capability, with each one alone, and with all of them.
# Each capability is built and tested by `make lint` as well (FACADE_FEATURE_SETS).
EVERYTHING = "visualization,synthetic,io,async,schema,plotly,static_export"
FACADE_TREES = [
    "",
    "math",
    "schema",
    "pricing",
    "market",
    "io",
    "async",
    "synthetic",
    "analytics",
    "strategies",
    "simulation",
    "backtest",
    "visualization",
    "plotly",
    "static_export",
    EVERYTHING,
]
FACADE_TESTS = ["", "pricing", "strategies", "backtest", EVERYTHING]

# What `cargo tree -e features -i chrono` prints when something turned it on.
LEAK_RE = re.compile(
    r'(?:chrono|expiration_date|option_type) feature "(?:clock|now|wasmbind)"|iana-time-zone'
)


def leaks(tree: str) -> list[str]:
    """The lines of a tree that show the clock enabled."""
    return [line for line in tree.splitlines() if LEAK_RE.search(line)]


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=ROOT, capture_output=True, text=True)


def tree(package: str, features: str, *, default: bool = False) -> str:
    command = ["cargo", "tree", "--color", "never", "-e", "features", "-i", "chrono", "--target", "all"]
    command += ["-p", package]
    if not default:
        command.append("--no-default-features")
    if features:
        command += ["--features", features]
    result = run(*command)
    if result.returncode != 0:
        raise SystemExit(f"cargo tree failed for {package} [{features}]:\n{result.stderr}")
    return result.stdout


def describe(package: str, features: str) -> str:
    return f"{package} --no-default-features" + (f" --features {features}" if features else "")


def check_trees() -> list[str]:
    problems = []
    configurations = [(package, "") for package in COMPONENTS]
    configurations += [(package, COMPONENT_FEATURES[package]) for package in COMPONENTS]
    configurations += [(FACADE, features) for features in FACADE_TREES]
    for package, features in configurations:
        lines = leaks(tree(package, features))
        if lines:
            problems.append(
                f"{describe(package, features)} enables the clock. "
                f"`cargo tree -e features -i chrono` shows:\n  " + "\n  ".join(lines)
            )
    for package in [FACADE, *(p for p in COMPONENTS if p != "optionstratlib-math")]:
        if not leaks(tree(package, "", default=True)):
            problems.append(f"{package} lost the clock in its default build")
    return problems


def cargo(*args: str) -> None:
    print("+ cargo " + " ".join(args), flush=True)
    env = dict(os.environ, LOGLEVEL="WARN")
    if subprocess.run(["cargo", *args], cwd=ROOT, env=env).returncode != 0:
        raise SystemExit(f"cargo {' '.join(args)} failed")


def build_and_test() -> None:
    configurations = [(package, "") for package in COMPONENTS]
    configurations += [
        (package, COMPONENT_FEATURES[package])
        for package in ("optionstratlib-market", "optionstratlib-visualization")
    ]
    configurations += [(FACADE, features) for features in FACADE_TESTS]
    for package, features in configurations:
        flags = ["-p", package, "--no-default-features"]
        if features:
            flags += ["--features", features]
        cargo("clippy", *flags, "--all-targets", "--", "-D", "warnings")
        cargo("build", *flags)
        cargo("test", *flags)


def self_test() -> int:
    clean = """chrono v0.4.45
\u251c\u2500\u2500 chrono feature "serde"
\u2514\u2500\u2500 chrono feature "std"
    \u2514\u2500\u2500 expiration_date v0.6.0
"""
    leaked = """chrono v0.4.45
\u251c\u2500\u2500 chrono feature "clock"
\u2502   \u2514\u2500\u2500 expiration_date feature "clock"
\u2514\u2500\u2500 chrono feature "std"
"""
    cases = {
        "a tree without the clock is clean": (leaks(clean), 0),
        "`clock` and what enables it are reported": (leaks(leaked), 2),
        "`now` alone is a leak": (leaks('chrono feature "now"'), 1),
        "`wasmbind` alone is a leak": (leaks('chrono feature "wasmbind"'), 1),
        "option_type forwards it": (leaks('option_type feature "clock"'), 1),
        "a package reading the time zone is a leak": (leaks("iana-time-zone v0.1.65"), 1),
        "another crate's `clock` is not ours": (leaks('tokio feature "clock"'), 0),
    }
    failed = [name for name, (found, want) in cases.items() if len(found) != want]
    for name in failed:
        print(f"self-test failed: {name}")
    if not failed:
        print(f"OK: {len(cases)} self-test cases")
    return 1 if failed else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    problems = check_trees()
    if problems:
        print("\n".join(problems))
        return 1
    print("OK: no configuration without `clock` enables `chrono/clock`, and the default builds keep it")
    if "--trees" not in sys.argv:
        build_and_test()
        print("OK: every configuration without `clock` lints, builds and tests")
    return 0


if __name__ == "__main__":
    sys.exit(main())
